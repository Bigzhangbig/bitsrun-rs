use crate::client::SrunClient;
use crate::config;
use crate::monitor::start_hardware_monitor;

use std::fs;
use std::time::Duration;
use anyhow::Context;
use anyhow::Result;
use log::{info, debug};
use owo_colors::OwoColorize;
use owo_colors::Stream::Stdout;
use tokio::signal::ctrl_c;

#[derive(serde::Deserialize)]
pub struct SrunDaemon {
    pub username: String,
    pub password: String,
    pub dm: bool,
    pub poll_interval: u64,
}

impl SrunDaemon {
    pub async fn run(config: Option<String>) -> Result<()> {
        let finalized_cfg = config::validate_config_file(&config)?;
        let daemon_cfg_str = fs::read_to_string(&finalized_cfg).with_context(|| {
            format!(
                "failed to read config file `{}`",
                &finalized_cfg.if_supports_color(Stdout, |t: &String| t.underline())
            )
        })?;

        let daemon: SrunDaemon = serde_json::from_str(&daemon_cfg_str)?;
        let http_client = reqwest::Client::builder().build()?;
        let srun = SrunClient::new(
            daemon.username.clone(),
            daemon.password.clone(),
            Some(http_client),
            None,
            Some(daemon.dm),
        ).await?;

        let poll_interval = if daemon.poll_interval == 0 { 3600 } else { daemon.poll_interval };
        let mut srun_ticker = tokio::time::interval(Duration::from_secs(poll_interval));
        let mut hardware_events = start_hardware_monitor();

        info!("Starting smart daemon for {} (interval={}s)", daemon.username, poll_interval);

        loop {
            tokio::select! {
                _ = srun_ticker.tick() => {
                    debug!("Scheduled keep-alive check...");
                    let _ = srun.ensure_online().await;
                }
                _ = hardware_events.recv() => {
                    info!("Hardware event detected, ensuring connectivity...");
                    let _ = srun.ensure_online().await;
                }
                _ = ctrl_c() => {
                    info!("Gracefully exiting...");
                    break;
                }
            }
        }
        Ok(())
    }
}
