//! bitsrun - 北京理工大学校园网登录客户端
//!
//! 本文件是程序的入口点，负责：
//! - 初始化命令行参数解析
//! - 根据子命令分发到相应的处理函数
//! - 协调各模块完成登录、登出、状态查询和守护进程等功能
//! - 统一的错误处理和输出格式化
//!
//! bitsrun - BIT Campus Network Login Client
//!
//! This is the main entry point of the application, responsible for:
//! - Initializing command-line argument parsing
//! - Dispatching to appropriate handlers based on subcommands
//! - Coordinating modules to perform login, logout, status check, and daemon functions
//! - Unified error handling and output formatting

mod cli;
mod client;
mod config;
mod daemon;
mod tables;
mod user;
mod xencode;

#[cfg(windows)]
mod windows_service;
#[cfg(windows)]
mod service_logger;

use anyhow::Context;
use anyhow::Result;
use clap::Parser;
use cli::ClientArgs;
use cli::StatusArgs;
use enable_ansi_support::enable_ansi_support;
use owo_colors::OwoColorize;
use owo_colors::Stream::Stderr;
use owo_colors::Stream::Stdout;

use cli::Arguments;
use cli::Commands;
use client::get_login_state;
use client::SrunClient;
use daemon::SrunDaemon;
use tables::print_config_paths;
use tables::print_login_state;

#[tokio::main]
async fn main() {
    #[cfg(windows)]
    {
        // Try to start as a Windows service first; if not launched by SCM, this will error
        if windows_service::run_windows_service().is_ok() {
            return;
        }
    }
    if let Err(err) = cli().await {
        eprintln!(
            "{} {}: {}",
            "bitsrun".if_supports_color(Stderr, |t| t.bright_red()),
            "(error)".if_supports_color(Stderr, |t| t.dimmed()),
            err
        );
        std::process::exit(1);
    }
}

async fn cli() -> Result<()> {
    // 在不支持 ANSI 颜色的 Windows 终端上禁用颜色输出
    // Disable ANSI colors on non-supported Windows terminals
    if enable_ansi_support().is_err() {
        owo_colors::set_override(false);
    }

    let args = Arguments::parse();

    // 创建可复用的 HTTP 客户端
    // Create reusable HTTP client
    let http_client = reqwest::Client::new();

    // 根据子命令分发到相应的处理函数
    // Dispatch to appropriate handlers based on subcommand
    match &args.command {
        // 查询登录状态
        // Check login status
        Some(Commands::Status(status_args)) => {
            srun_status(http_client, status_args, args.verbose).await?
        }

        // 登录或登出
        // Login or logout
        Some(Commands::Login(client_args)) | Some(Commands::Logout(client_args)) => {
            let bit_user = user::finalize_bit_user(
                &client_args.username,
                &client_args.password,
                client_args.dm,
                &client_args.config,
                matches!(args.command, Some(Commands::Login(_))),
            )
            .with_context(|| "unable to parse user credentials")?;

            let srun_client = SrunClient::new(
                bit_user.username,
                bit_user.password,
                Some(http_client),
                client_args.ip,
                Some(bit_user.dm),
            )
            .await?;

            match &args.command {
                Some(Commands::Login(_)) => {
                    srun_login(&srun_client, client_args, args.verbose).await?
                }
                Some(Commands::Logout(_)) => {
                    srun_logout(&srun_client, client_args, args.verbose).await?
                }
                _ => {}
            };
        }

        Some(Commands::KeepAlive(daemon_args)) => {
            let config_path = daemon_args.config.to_owned();
            let daemon = SrunDaemon::new(config_path)?;
            daemon.start(http_client).await?;
        }

        #[cfg(windows)]
        Some(Commands::WindowsService) => {
            // 运行 Windows 服务
            // Run Windows service
            windows_service::run_windows_service()
                .map_err(|e| anyhow::anyhow!("Windows service error: {}", e))?;
        }

        Some(Commands::ConfigPaths) => print_config_paths(),

        None => {}
    }

    Ok(())
}

async fn srun_status(
    http_client: reqwest::Client,
    status_args: &StatusArgs,
    verbose: bool,
) -> Result<()> {
    // 当 args.verbose = true 且不输出 JSON 时才启用详细输出
    // Only verbose on args.verbose = true and not outputting JSON
    let login_state = get_login_state(&http_client, verbose).await?;

    // 如果指定了 --json，输出 JSON 格式
    // Output JSON if --json flag is specified
    if status_args.json & !verbose {
        let raw_json = serde_json::to_string(&login_state)?;
        println!("{}", raw_json);
        return Ok(());
    }

    // 输出人类可读的格式
    // Output human-readable format
    match login_state.error.as_str() {
        "ok" => {
            println!(
                "{} {} {} is online",
                "bitsrun:".if_supports_color(Stdout, |t| t.bright_green()),
                &login_state
                    .online_ip
                    .to_string()
                    .if_supports_color(Stdout, |t| t.underline()),
                format!("({})", login_state.user_name.clone().unwrap_or_default())
                    .if_supports_color(Stdout, |t| t.dimmed())
            );

            // print status table
            print_login_state(login_state);
        }
        _ => {
            println!(
                "{} {} is offline",
                "bitsrun:".if_supports_color(Stdout, |t| t.blue()),
                login_state
                    .online_ip
                    .to_string()
                    .if_supports_color(Stdout, |t| t.underline())
            );
        }
    };
    Ok(())
}

async fn srun_login(
    srun_client: &SrunClient,
    client_args: &ClientArgs,
    verbose: bool,
) -> Result<()> {
    let resp = srun_client.login(client_args.force, verbose).await?;
    match resp.error.as_str() {
        "ok" => println!(
            "{} {} {} logged in",
            "bitsrun:".if_supports_color(Stdout, |t| t.bright_green()),
            resp.online_ip
                .to_string()
                .if_supports_color(Stdout, |t| t.underline()),
            format!("({})", resp.username.clone().unwrap_or_default())
                .if_supports_color(Stdout, |t| t.dimmed())
        ),
        _ => println!(
            "{} failed to login, {} {}",
            "bitsrun:".if_supports_color(Stdout, |t| t.red()),
            resp.error,
            format!("({})", resp.error_msg).if_supports_color(Stdout, |t| t.dimmed())
        ),
    };
    Ok(())
}

async fn srun_logout(
    srun_client: &SrunClient,
    client_args: &ClientArgs,
    verbose: bool,
) -> Result<()> {
    let resp = srun_client.logout(client_args.force, verbose).await?;
    match resp.error.as_str() {
        "ok" | "logout_ok" => println!(
            "{} {} logged out",
            "bitsrun:".if_supports_color(Stdout, |t| t.green()),
            resp.online_ip
                .to_string()
                .if_supports_color(Stdout, |t| t.underline())
        ),
        _ => println!(
            "{} failed to logout, {} {}",
            "bitsrun:".if_supports_color(Stdout, |t| t.red()),
            resp.error,
            format!("({})", resp.error_msg).if_supports_color(Stdout, |t| t.dimmed())
        ),
    };
    Ok(())
}
