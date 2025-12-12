//! Windows 服务模块
//!
//! 本模块实现了 Windows 服务功能：
//! - 与 Windows 服务控制管理器 (SCM) 交互
//! - 注册服务控制处理程序以响应控制事件
//! - 管理服务状态并及时通知 SCM
//! - 在服务上下文中运行 keep-alive 守护进程
//!
//! Windows Service Module
//!
//! This module implements Windows service functionality:
//! - Interacts with Windows Service Control Manager (SCM)
//! - Registers service control handler to respond to control events
//! - Manages service state and notifies SCM promptly
//! - Runs keep-alive daemon in service context

#[cfg(windows)]
use std::ffi::OsString;
#[cfg(windows)]
use std::sync::mpsc;
#[cfg(windows)]
use std::time::Duration;
#[cfg(windows)]
use windows_service::{
    define_windows_service,
    service::{
        ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus,
        ServiceType,
    },
    service_control_handler::{self, ServiceControlHandlerResult},
    service_dispatcher,
};

#[cfg(windows)]
use crate::daemon::SrunDaemon;
#[cfg(windows)]
use log::{error, info};

// 定义 Windows 服务名称
// Define Windows service name
const SERVICE_NAME: &str = "Bitsrun";

// 定义服务类型
// Define service type
const SERVICE_TYPE: ServiceType = ServiceType::OWN_PROCESS;

#[cfg(windows)]
define_windows_service!(ffi_service_main, service_main);

/// Windows 服务入口点
/// 此函数由 Windows 服务控制管理器调用
///
/// Windows service entry point
/// This function is called by the Windows Service Control Manager
#[cfg(windows)]
fn service_main(arguments: Vec<OsString>) {
    if let Err(e) = run_service(arguments) {
        error!("Service error: {}", e);
    }
}

/// 运行 Windows 服务的主要逻辑
/// Main logic for running the Windows service
#[cfg(windows)]
fn run_service(_arguments: Vec<OsString>) -> windows_service::Result<()> {
    // 创建一个通道用于接收停止信号
    // Create a channel for receiving stop signals
    let (shutdown_tx, shutdown_rx) = mpsc::channel();

    // 定义服务控制事件处理程序
    // Define service control event handler
    let event_handler = move |control_event| -> ServiceControlHandlerResult {
        match control_event {
            // 处理停止请求
            // Handle stop request
            ServiceControl::Stop => {
                info!("Received stop signal from SCM");
                shutdown_tx.send(()).ok();
                ServiceControlHandlerResult::NoError
            }
            // 处理关闭请求（系统关闭时）
            // Handle shutdown request (during system shutdown)
            ServiceControl::Shutdown => {
                info!("Received shutdown signal from SCM");
                shutdown_tx.send(()).ok();
                ServiceControlHandlerResult::NoError
            }
            // 返回服务状态查询
            // Handle status query
            ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
            // 其他控制命令暂不处理
            // Other control commands are not handled
            _ => ServiceControlHandlerResult::NotImplemented,
        }
    };

    // 注册服务控制处理程序
    // Register the service control handler
    let status_handle = service_control_handler::register(SERVICE_NAME, event_handler)?;

    // 通知 SCM 服务正在启动
    // Notify SCM that the service is starting
    status_handle.set_service_status(ServiceStatus {
        service_type: SERVICE_TYPE,
        current_state: ServiceState::StartPending,
        controls_accepted: ServiceControlAccept::empty(),
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 1,
        wait_hint: Duration::from_secs(10),
        process_id: None,
    })?;

    #[cfg(windows)]
    {
        let _ = winlog2::init(SERVICE_NAME);
    }

    // 从可执行文件所在目录尝试读取配置文件
    // Try reading config file from the executable directory
    let config_path = match std::env::current_exe() {
        Ok(mut exe_path) => {
            exe_path.pop();
            exe_path.push("bit-user.json");
            let cfg = exe_path.to_string_lossy().to_string();
            match std::fs::metadata(&cfg) {
                Ok(meta) if meta.is_file() => Some(cfg),
                _ => None,
            }
        }
        Err(_) => None,
    };

    let daemon = match SrunDaemon::new(config_path) {
        Ok(d) => d,
        Err(e) => {
            error!("Failed to create daemon: {}", e);
            // 通知 SCM 服务启动失败
            // Notify SCM that service start failed
            status_handle.set_service_status(ServiceStatus {
                service_type: SERVICE_TYPE,
                current_state: ServiceState::Stopped,
                controls_accepted: ServiceControlAccept::empty(),
                exit_code: ServiceExitCode::ServiceSpecific(1),
                checkpoint: 0,
                wait_hint: Duration::default(),
                process_id: None,
            })?;
            return Ok(());
        }
    };

    // 尽快通知 SCM 服务状态为 Running
    // Notify SCM as soon as possible that service is Running
    status_handle.set_service_status(ServiceStatus {
        service_type: SERVICE_TYPE,
        current_state: ServiceState::Running,
        controls_accepted: ServiceControlAccept::STOP | ServiceControlAccept::SHUTDOWN,
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: None,
    })?;

    info!("Service started successfully");

    // 创建 tokio 运行时以运行异步代码
    // Create tokio runtime to run async code
    let runtime = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            error!("Failed to create tokio runtime: {}", e);
            // 通知 SCM 服务启动失败
            // Notify SCM that service start failed
            status_handle.set_service_status(ServiceStatus {
                service_type: SERVICE_TYPE,
                current_state: ServiceState::Stopped,
                controls_accepted: ServiceControlAccept::empty(),
                exit_code: ServiceExitCode::ServiceSpecific(2),
                checkpoint: 0,
                wait_hint: Duration::default(),
                process_id: None,
            })?;
            return Ok(());
        }
    };

    // 在主循环中运行业务逻辑
    // Run business logic in the main loop
    runtime.block_on(async {
        let http_client = reqwest::Client::new();

        // 创建一个用于监听停止信号的 future
        // Create a future to listen for stop signal
        let shutdown_future = async {
            // 在异步任务中等待停止信号
            // Wait for stop signal in async task
            match tokio::task::spawn_blocking(move || shutdown_rx.recv()).await {
                Ok(Ok(())) => info!("Received shutdown signal"),
                Ok(Err(e)) => error!("Error receiving shutdown signal: {}", e),
                Err(e) => error!("Task join error: {}", e),
            }
        };

        // 创建守护进程任务
        // Create daemon task
        let daemon_future = daemon.start_with_shutdown(http_client, shutdown_future);

        // 等待守护进程完成或收到停止信号
        // Wait for daemon to complete or receive stop signal
        if let Err(e) = daemon_future.await {
            error!("Daemon error: {}", e);
        }
    });

    info!("Service stopping");

    // 退出循环前通知 SCM 服务状态为 Stopped
    // Notify SCM that service is Stopped before exiting the loop
    status_handle.set_service_status(ServiceStatus {
        service_type: SERVICE_TYPE,
        current_state: ServiceState::Stopped,
        controls_accepted: ServiceControlAccept::empty(),
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: None,
    })?;

    info!("Service stopped");

    Ok(())
}

/// 启动 Windows 服务调度器
/// 此函数应该从 main 函数调用
///
/// Start the Windows service dispatcher
/// This function should be called from the main function
#[cfg(windows)]
pub fn run_windows_service() -> windows_service::Result<()> {
    // 将服务分发给服务控制管理器
    // Dispatch the service to the Service Control Manager
    service_dispatcher::start(SERVICE_NAME, ffi_service_main)?;
    Ok(())
}
