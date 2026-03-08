use crate::client::SrunClient;
use crate::config;
use crate::monitor::start_hardware_monitor;
use crate::monitor::HardwareEvent;

use anyhow::Context;
use anyhow::Result;
use log::{debug, info, warn};
use owo_colors::OwoColorize;
use owo_colors::Stream::Stdout;
use std::fs;
use std::pin::Pin;
use std::time::Duration;
use tokio::signal::ctrl_c;
use tokio::time::Sleep;

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
        let http_client = reqwest::Client::builder()
            .no_proxy()
            .connect_timeout(Duration::from_secs(3))
            .timeout(Duration::from_secs(5))
            .build()?;

        let mut srun = SrunClient::new(
            daemon.username.clone(),
            daemon.password.clone(),
            Some(http_client.clone()),
            None,
            Some(daemon.dm),
        )
        .await?;

        let poll_interval = if daemon.poll_interval == 0 {
            3600
        } else {
            daemon.poll_interval
        };
        let mut srun_ticker = tokio::time::interval(Duration::from_secs(poll_interval));
        let mut hardware_events = start_hardware_monitor();

        info!(
            "Starting smart daemon for {} (interval={}s)",
            daemon.username, poll_interval
        );

        let mut debounce_timer: Option<Pin<Box<Sleep>>> = None;

        loop {
            tokio::select! {
                _ = srun_ticker.tick() => {
                    debug!("Scheduled keep-alive check...");
                    let _ = srun.ensure_online().await;
                }
                event = hardware_events.recv() => {
                    if let Some(HardwareEvent::Refresh) = event {
                        debug!("Hardware event received, debouncing...");
                        debounce_timer = Some(Box::pin(tokio::time::sleep(Duration::from_millis(500))));
                    }
                }
                _ = async {
                    if let Some(ref mut timer) = debounce_timer {
                        timer.as_mut().await;
                    } else {
                        std::future::pending::<()>().await;
                    }
                }, if debounce_timer.is_some() => {
                    debounce_timer = None;
                    info!("Hardware event stabilized, refreshing client context...");
                    
                    // Re-create the http_client to clear all connection pools/cache
                    let new_http_client = reqwest::Client::builder()
                        .no_proxy()
                        .connect_timeout(Duration::from_secs(3))
                        .timeout(Duration::from_secs(5))
                        .build()
                        .unwrap_or(http_client.clone());
                    
                    // Re-instantiate srun client to pick up the most accurate IP and ac_id for the current interface
                    match SrunClient::new(
                        daemon.username.clone(),
                        daemon.password.clone(),
                        Some(new_http_client),
                        None,
                        Some(daemon.dm),
                    ).await {
                        Ok(new_srun) => {
                            info!("Network discovery successful, applying new context.");
                            srun = new_srun;
                            let _ = srun.ensure_online().await;
                        }
                        Err(e) => {
                            warn!("Network discovery failed: {}. This is expected during interface switching. Retrying later...", e);
                            // Set a short debounce timer to retry discovery shortly
                            debounce_timer = Some(Box::pin(tokio::time::sleep(Duration::from_secs(2))));
                        }
                    }
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
