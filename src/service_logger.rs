use log::{Level, Metadata, Record, SetLoggerError};
use std::env;
use std::ffi::{OsStr, OsString};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::os::windows::ffi::OsStrExt;
use std::path::PathBuf;
use std::ptr;
use std::sync::{Mutex, OnceLock};

use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::System::Registry::{RegCreateKeyExW, RegSetValueExW, HKEY_LOCAL_MACHINE, REG_OPTION_NON_VOLATILE, REG_EXPAND_SZ, REG_DWORD, KEY_SET_VALUE};
use windows_sys::Win32::System::EventLog::{
    DeregisterEventSource, RegisterEventSourceW, ReportEventW, EVENTLOG_ERROR_TYPE,
    EVENTLOG_INFORMATION_TYPE, EVENTLOG_WARNING_TYPE,
};

struct ServiceLogger {
    file: Mutex<Option<File>>,
    event_source: HANDLE,
}

// Safety: HANDLE is thread-safe for Event Log operations
unsafe impl Send for ServiceLogger {}
unsafe impl Sync for ServiceLogger {}

static SOURCE_NAME: OnceLock<Vec<u16>> = OnceLock::new();

impl log::Log for ServiceLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let msg = format!("{}", record.args());
            
            // Write to file
            if let Ok(mut file_guard) = self.file.lock() {
                if let Some(file) = file_guard.as_mut() {
                    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
                    // Ignore write errors
                    let _ = writeln!(file, "[{}] [{}] {}", timestamp, record.level(), msg);
                }
            }

            // Write to Event Log
            let (event_type, event_id) = match record.level() {
                Level::Error => (EVENTLOG_ERROR_TYPE, 1),
                Level::Warn => (EVENTLOG_WARNING_TYPE, 2),
                _ => (EVENTLOG_INFORMATION_TYPE, 3),
            };

            let wide_msg: Vec<u16> = OsStr::new(&msg).encode_wide().chain(std::iter::once(0)).collect();
            let strings = [wide_msg.as_ptr()];

            unsafe {
                ReportEventW(
                    self.event_source,
                    event_type,
                    0,
                    event_id, 
                    ptr::null_mut(),
                    1, // num strings
                    0,
                    strings.as_ptr(),
                    ptr::null_mut(),
                );
            }
        }
    }

    fn flush(&self) {
        if let Ok(mut file_guard) = self.file.lock() {
            if let Some(file) = file_guard.as_mut() {
                let _ = file.flush();
            }
        }
    }
}

fn ensure_event_source_registered(source: &str) {
    let subkey = {
        let mut s = OsString::from("SYSTEM\\CurrentControlSet\\Services\\EventLog\\Application\\");
        s.push(source);
        s
    };
    let mut hkey: windows_sys::Win32::System::Registry::HKEY = std::ptr::null_mut();
    let wide_subkey: Vec<u16> = subkey.encode_wide().chain(std::iter::once(0)).collect();
    unsafe {
        let _ = RegCreateKeyExW(
            HKEY_LOCAL_MACHINE,
            wide_subkey.as_ptr(),
            0,
            ptr::null_mut(),
            REG_OPTION_NON_VOLATILE,
            KEY_SET_VALUE,
            ptr::null(),
            &mut hkey,
            ptr::null_mut(),
        );
        // Prefer generic message resource DLL to avoid "no resource section" / missing description
        let is_x64 = std::env::var("PROCESSOR_ARCHITECTURE").map(|s| s.contains("64")).unwrap_or(true);
        let msg_file = if is_x64 {
            OsStr::new("%SystemRoot%\\Microsoft.NET\\Framework64\\v4.0.30319\\EventLogMessages.dll")
        } else {
            OsStr::new("%SystemRoot%\\Microsoft.NET\\Framework\\v4.0.30319\\EventLogMessages.dll")
        };
        let wide_msg: Vec<u16> = msg_file.encode_wide().chain(std::iter::once(0)).collect();
        let name_event_message_file: Vec<u16> = OsStr::new("EventMessageFile").encode_wide().chain(std::iter::once(0)).collect();
        let _ = RegSetValueExW(
            hkey,
            name_event_message_file.as_ptr(),
            0,
            REG_EXPAND_SZ,
            wide_msg.as_ptr() as *const u8,
            (wide_msg.len() * 2) as u32,
        );
        // Also set ParameterMessageFile to same DLL for insertion strings
        let name_param_message_file: Vec<u16> = OsStr::new("ParameterMessageFile").encode_wide().chain(std::iter::once(0)).collect();
        let _ = RegSetValueExW(
            hkey,
            name_param_message_file.as_ptr(),
            0,
            REG_EXPAND_SZ,
            wide_msg.as_ptr() as *const u8,
            (wide_msg.len() * 2) as u32,
        );
        // TypesSupported = Error|Warning|Information (0x7)
        let types_supported_name: Vec<u16> = OsStr::new("TypesSupported").encode_wide().chain(std::iter::once(0)).collect();
        let types_supported: u32 = 0x7;
        let _ = RegSetValueExW(
            hkey,
            types_supported_name.as_ptr(),
            0,
            REG_DWORD,
            &types_supported as *const u32 as *const u8,
            std::mem::size_of::<u32>() as u32,
        );
    }
}

pub fn init(service_name: &str) -> Result<(), SetLoggerError> {
    let wide_name: Vec<u16> = OsStr::new(service_name).encode_wide().chain(std::iter::once(0)).collect();
    let event_source = unsafe { RegisterEventSourceW(ptr::null(), wide_name.as_ptr()) };

    let mut log_path = env::current_exe().unwrap_or_else(|_| PathBuf::from("."));
    log_path.pop();
    log_path.push("bitsrun_service.log");

    // Ensure event source is registered in registry to avoid "path not found"
    ensure_event_source_registered(service_name);

    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)
        .ok();

    let logger = ServiceLogger {
        file: Mutex::new(file),
        event_source,
    };

    SOURCE_NAME.set(wide_name).ok();

    log::set_boxed_logger(Box::new(logger)).map(|()| log::set_max_level(log::LevelFilter::Info))
}

pub fn log_event(event_id: u32, level: Level, message: &str) {
    if let Some(name) = SOURCE_NAME.get() {
        unsafe {
            let handle = RegisterEventSourceW(ptr::null(), name.as_ptr());
            let (event_type, eid) = match level {
                Level::Error => (EVENTLOG_ERROR_TYPE, event_id),
                Level::Warn => (EVENTLOG_WARNING_TYPE, event_id),
                _ => (EVENTLOG_INFORMATION_TYPE, event_id),
            };
            let wide_msg: Vec<u16> = OsStr::new(message).encode_wide().chain(std::iter::once(0)).collect();
            let strings = [wide_msg.as_ptr()];
            ReportEventW(
                handle,
                event_type,
                0,
                eid,
                ptr::null_mut(),
                1,
                0,
                strings.as_ptr(),
                ptr::null_mut(),
            );
            let _ = DeregisterEventSource(handle);
        }
    }
}
