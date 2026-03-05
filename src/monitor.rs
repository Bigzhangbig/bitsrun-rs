use tokio::sync::mpsc;

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

    pub fn start_monitor(tx: mpsc::Sender<HardwareEvent>) {
        let tx_net = tx.clone();
        // Network Monitor
        let _ = netwatcher::watch_interfaces(move |upd| {
            let mut changed = false;
            for idx in &upd.diff.added {
                if let Some(iface) = upd.interfaces.get(idx) {
                    if iface.name.starts_with("en") {
                        info!("[Monitor] Interface added: {}", iface.name);
                        changed = true;
                    }
                }
            }
            for (idx, diff) in &upd.diff.modified {
                if let Some(iface) = upd.interfaces.get(idx) {
                    if iface.name.starts_with("en")
                        && (!diff.addrs_added.is_empty() || !diff.addrs_removed.is_empty())
                    {
                        info!("[Monitor] Interface modified: {}", iface.name);
                        changed = true;
                    }
                }
            }
            if changed {
                let _ = tx_net.try_send(HardwareEvent::Refresh);
            }
        })
        .map(|h| Box::leak(Box::new(h)));

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

pub fn start_hardware_monitor() -> mpsc::Receiver<HardwareEvent> {
    let (tx, rx) = mpsc::channel(10);
    #[cfg(target_os = "macos")]
    macos::start_monitor(tx);
    rx
}
