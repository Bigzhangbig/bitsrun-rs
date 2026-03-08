use tokio::sync::mpsc;
use log::info;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HardwareEvent {
    Refresh,
}

#[cfg(target_os = "macos")]
mod macos {
    use super::HardwareEvent;
    use core_foundation_sys::runloop::*;
    use log::info;
    use mach2::kern_return::kern_return_t;
    use std::{
        ffi::c_void,
        ptr,
        sync::atomic::{AtomicU32, Ordering},
        sync::Arc,
        thread,
    };
    use tokio::sync::mpsc;

    #[link(name = "IOKit", kind = "framework")]
    extern "C" {
        fn IORegisterForSystemPower(
            r: *mut c_void,
            p: *mut *mut c_void,
            c: Option<unsafe extern "C" fn(*mut c_void, u32, u32, *mut c_void)>,
            n: *mut u32,
        ) -> u32;
        fn IODeregisterForSystemPower(n: *mut u32) -> kern_return_t;
        fn IOAllowPowerChange(k: u32, i: i64) -> kern_return_t;
        fn IONotificationPortGetRunLoopSource(p: *mut c_void) -> CFRunLoopSourceRef;
        fn IONotificationPortDestroy(p: *mut c_void);
    }

    const K_IOPM_MESSAGE_CLAMSHELL_STATE_CHANGE: u32 = 0xE0000100;
    const K_IOMESSAGE_SYSTEM_HAS_POWERED_ON: u32 = 0xE0000300;
    const K_CLAMSHELL_STATE_BIT: usize = 0x1;

    struct State {
        tx: mpsc::Sender<HardwareEvent>,
        root_port: AtomicU32,
    }

    unsafe extern "C" fn power_callback(refcon: *mut c_void, _: u32, msg: u32, arg: *mut c_void) {
        let s = &*(refcon as *const State);
        match msg {
            K_IOPM_MESSAGE_CLAMSHELL_STATE_CHANGE => {
                if (arg as usize & K_CLAMSHELL_STATE_BIT) == 0 {
                    info!("[Monitor] Lid opened");
                    let _ = s.tx.try_send(HardwareEvent::Refresh);
                }
            }
            K_IOMESSAGE_SYSTEM_HAS_POWERED_ON => {
                info!("[Monitor] System wake");
                let _ = s.tx.try_send(HardwareEvent::Refresh);
            }
            0xE0000270 | 0xE0000280 => {
                // Sleep related
                let port = s.root_port.load(Ordering::SeqCst);
                if port != 0 {
                    IOAllowPowerChange(port, arg as i64);
                }
            }
            _ => {}
        }
    }

    pub fn start_lid_monitor(tx: mpsc::Sender<HardwareEvent>) {
        // Lid Monitor
        thread::spawn(move || unsafe {
            let mut port: *mut c_void = ptr::null_mut();
            let mut notifier: u32 = 0;
            let s = Arc::new(State {
                tx,
                root_port: AtomicU32::new(0),
            });
            let refcon = Arc::into_raw(s.clone()) as *mut c_void;
            let root =
                IORegisterForSystemPower(refcon, &mut port, Some(power_callback), &mut notifier);
            if root != 0 {
                s.root_port.store(root, Ordering::SeqCst);
                CFRunLoopAddSource(
                    CFRunLoopGetCurrent(),
                    IONotificationPortGetRunLoopSource(port),
                    kCFRunLoopDefaultMode,
                );
                CFRunLoopRun();
                IODeregisterForSystemPower(&mut notifier);
                IONotificationPortDestroy(port);
            }
            let _ = Arc::from_raw(refcon as *const State);
        });
    }
}

#[cfg(target_os = "linux")]
mod linux {
    use super::HardwareEvent;
    use log::{info, warn};
    use tokio::sync::mpsc;
    use zbus::{proxy, Connection};
    use futures_util::stream::StreamExt;

    #[proxy(
        interface = "org.freedesktop.NetworkManager",
        default_service = "org.freedesktop.NetworkManager",
        default_path = "/org/freedesktop/NetworkManager"
    )]
    trait NetworkManager {
        #[zbus(property)]
        fn connectivity(&self) -> zbus::Result<u32>;

        #[zbus(property)]
        fn active_connections(&self) -> zbus::Result<Vec<zbus::zvariant::OwnedObjectPath>>;
    }

    #[proxy(
        interface = "org.freedesktop.login1.Manager",
        default_service = "org.freedesktop.login1",
        default_path = "/org/freedesktop/login1"
    )]
    trait LogindManager {
        #[zbus(signal)]
        fn prepare_for_sleep(&self, active: bool) -> zbus::Result<()>;
    }

    pub fn start_linux_monitor(tx: mpsc::Sender<HardwareEvent>) {
        tokio::spawn(async move {
            match Connection::system().await {
                Ok(conn) => {
                    let nm_proxy = NetworkManagerProxy::new(&conn).await;
                    let login_proxy = LogindManagerProxy::new(&conn).await;

                    match (nm_proxy, login_proxy) {
                        (Ok(nm), Ok(login)) => {
                            info!("[Monitor] Linux D-Bus monitor started (NetworkManager & logind)");
                            let mut connectivity_updates = nm.receive_connectivity_changed().await;
                            let mut active_conn_updates = nm.receive_active_connections_changed().await;
                            let mut sleep_updates = login.receive_prepare_for_sleep().await.expect("Failed to listen to sleep signals");

                            loop {
                            tokio::select! {
                                Some(update) = connectivity_updates.next() => {
                                    if let Ok(val) = update.get().await {
                                        info!("[Monitor] Connectivity changed: {}", val);
                                        let _ = tx.try_send(HardwareEvent::Refresh);
                                    }
                                }
                                Some(_) = active_conn_updates.next() => {
                                    info!("[Monitor] Active connections changed (Roaming/SSID switch)");
                                    let _ = tx.try_send(HardwareEvent::Refresh);
                                }
                                Some(signal) = sleep_updates.next() => {
                                    if let Ok(args) = signal.args() {
                                        if !args.active {
                                            info!("[Monitor] System wake detected from logind");
                                            let _ = tx.try_send(HardwareEvent::Refresh);
                                        }
                                    }
                                }
                            }
                            }

                        }
                        _ => warn!("[Monitor] Failed to create D-Bus proxies"),
                    }
                }
                Err(e) => warn!("[Monitor] Failed to connect to system D-Bus: {}", e),
            }
        });
    }
}

pub fn start_hardware_monitor() -> mpsc::Receiver<HardwareEvent> {
    let (tx, rx) = mpsc::channel(10);

    let tx_net = tx.clone();
    // Network Monitor (Cross-platform via netwatcher)
    let _ = netwatcher::watch_interfaces(move |upd| {
        let mut changed = false;
        // Focus on added interfaces that already have IP addresses
        for idx in &upd.diff.added {
            if let Some(iface) = upd.interfaces.get(idx) {
                if (iface.name.starts_with("en") || iface.name.starts_with("eth") || iface.name.starts_with("wl")) 
                    && !iface.ips.is_empty() 
                {
                    info!("[Monitor] New interface with IP detected: {}", iface.name);
                    changed = true;
                }
            }
        }
        // Precision monitoring for IP address changes on existing interfaces
        for (idx, diff) in &upd.diff.modified {
            if let Some(iface) = upd.interfaces.get(idx) {
                if iface.name.starts_with("en") || iface.name.starts_with("eth") || iface.name.starts_with("wl") {
                    if !diff.addrs_added.is_empty() || !diff.addrs_removed.is_empty() {
                        info!("[Monitor] IP address changed on interface: {}", iface.name);
                        changed = true;
                    }
                }
            }
        }
        if changed {
            let _ = tx_net.try_send(HardwareEvent::Refresh);
        }
    })
    .map(|h| Box::leak(Box::new(h)));

    #[cfg(target_os = "macos")]
    macos::start_lid_monitor(tx);

    #[cfg(target_os = "linux")]
    linux::start_linux_monitor(tx);

    rx
}
