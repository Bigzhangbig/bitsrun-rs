use std::io::Write;

use anyhow::Context;
use anyhow::Result;
use chrono::Local;
use clap::Parser;
use enable_ansi_support::enable_ansi_support;
use owo_colors::OwoColorize;
use owo_colors::Stream::Stderr;
use owo_colors::Stream::Stdout;

use bitsrun::cli;
use bitsrun::client;
use bitsrun::daemon;
use bitsrun::tables;
use bitsrun::user;

use cli::{Arguments, ClientArgs, Commands, StatusArgs};
use client::{get_login_state, SrunClient};
use daemon::SrunDaemon;
use tables::{print_config_paths, print_login_state};

#[tokio::main]
async fn main() {
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
    if std::env::var("RUST_LOG").is_err() {
        if cfg!(debug_assertions) {
            std::env::set_var("RUST_LOG", "debug");
        } else {
            std::env::set_var("RUST_LOG", "info");
        }
    }
    // Initialize logger with custom format including timestamp
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format(|buf, record| {
            let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
            let level = record.level();
            let level_style = buf.default_level_style(level);
            writeln!(
                buf,
                "[{} {} {}] {}",
                timestamp,
                level_style.value(level),
                record.target(),
                record.args()
            )
        })
        .init();

    // disable ansi colors on non-supported windows terminals
    if enable_ansi_support().is_err() {
        owo_colors::set_override(false);
    }

    let args = Arguments::parse();

    // reusable http client without proxy and with timeout
    let http_client = reqwest::Client::builder()
        .no_proxy()
        .connect_timeout(std::time::Duration::from_millis(400))
        .timeout(std::time::Duration::from_millis(400))
        .build()?;

    // commands
    match &args.command {
        // check login status
        Some(Commands::Status(status_args)) => {
            srun_status(http_client, status_args, args.verbose).await?
        }

        // login or logout
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
            SrunDaemon::run(config_path).await?;
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
    // only verbose on args.verbose = true and not outputting json
    let login_state = get_login_state(&http_client, verbose).await?;

    // output json
    if status_args.json & !verbose {
        let raw_json = serde_json::to_string(&login_state)?;
        println!("{}", raw_json);
        return Ok(());
    }

    // output human readable
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
