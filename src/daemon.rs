//! 守护进程模块
//!
//! 本模块实现了 keep-alive 守护进程功能：
//! - 从配置文件读取用户凭据和轮询间隔
//! - 定期向认证服务器发送登录请求以保持会话活跃
//! - 支持优雅退出（Ctrl+C）
//! - 提供日志记录功能，便于监控运行状态
//!
//! Daemon Module
//!
//! This module implements the keep-alive daemon functionality:
//! - Reads user credentials and polling interval from config file
//! - Periodically sends login requests to authentication server to keep session alive
//! - Supports graceful shutdown (Ctrl+C)
//! - Provides logging for monitoring runtime status

use crate::client::SrunClient;
use crate::config;

use anyhow::Result;
use log::info;
use log::warn;

use reqwest::Client;
use serde::Deserialize;

use tokio::signal::ctrl_c;
use tokio::time::Duration;

#[derive(Debug, Deserialize)]
pub struct SrunDaemon {
    username: String,
    password: String,
    dm: bool,
    // polls every 1 hour by default
    poll_interval: Option<u64>,
}

impl SrunDaemon {
    pub fn new(config_path: Option<String>) -> Result<SrunDaemon> {
        // in daemon mode, bitsrun must be able to read all required fields from the config file,
        // including `username`, `password`, and `dm`.
        config::read_config_file::<SrunDaemon>(&config_path)
    }

    pub async fn start(&self, http_client: Client) -> Result<()> {
        // 默认将日志级别设置为 INFO
        // Set logger to INFO level by default
        pretty_env_logger::formatted_builder()
            .filter_level(log::LevelFilter::Info)
            .init();

        // 默认每小时轮询一次
        // Set default polling interval to every 1 hour
        let poll_interval = self.poll_interval.unwrap_or(3600);

        // 如果轮询间隔过短，发出警告
        // Warn if polling interval is too short
        if poll_interval < 60 * 10 {
            warn!("polling interval is too short, please set it to at least 10 minutes (600s)");
        }

        // 启动守护进程
        // Start daemon
        let mut srun_ticker = tokio::time::interval(Duration::from_secs(poll_interval));
        let srun = SrunClient::new(
            self.username.clone(),
            self.password.clone(),
            Some(http_client),
            None,
            Some(self.dm),
        )
        .await?;

        info!(
            "starting daemon ({}) with polling interval={}s",
            self.username, poll_interval,
        );

        loop {
            let tick = srun_ticker.tick();
            let login = srun.login(true, false);

            tokio::select! {
                _ = tick => {
                    match login.await {
                        Ok(resp) => {
                            match resp.error.as_str() {
                                "ok" => {
                                    info!("{} ({}): login success, {}", resp.client_ip, self.username, resp.suc_msg.unwrap_or_default());
                                }
                                _ => {
                                    warn!("{} ({}): login failed, {}", resp.client_ip, self.username, resp.error);
                                }
                            }
                        }
                        Err(e) => {
                            warn!("{}: login failed: {}", self.username, e);
                        }
                    }
                }
                _ = ctrl_c() => {
                    info!("{}: gracefully exiting", self.username);
                    break;
                }
            }
        }

        Ok(())
    }

    /// 启动守护进程，支持外部关闭信号
    /// Start daemon with external shutdown signal support
    #[cfg(windows)]
    pub async fn start_with_shutdown<F>(
        &self,
        http_client: Client,
        shutdown_signal: F,
    ) -> Result<()>
    where
        F: std::future::Future<Output = ()>,
    {
        // 默认将日志级别设置为 INFO
        // Set logger to INFO level by default
        pretty_env_logger::formatted_builder()
            .filter_level(log::LevelFilter::Info)
            .init();

        // 默认每小时轮询一次
        // Set default polling interval to every 1 hour
        let poll_interval = self.poll_interval.unwrap_or(3600);

        // 如果轮询间隔过短，发出警告
        // Warn if polling interval is too short
        if poll_interval < 60 * 10 {
            warn!("polling interval is too short, please set it to at least 10 minutes (600s)");
        }

        // 启动守护进程
        // Start daemon
        let mut srun_ticker = tokio::time::interval(Duration::from_secs(poll_interval));
        let srun = SrunClient::new(
            self.username.clone(),
            self.password.clone(),
            Some(http_client),
            None,
            Some(self.dm),
        )
        .await?;

        info!(
            "starting daemon ({}) with polling interval={}s",
            self.username, poll_interval,
        );

        tokio::pin!(shutdown_signal);

        loop {
            let tick = srun_ticker.tick();
            let login = srun.login(true, false);

            tokio::select! {
                _ = tick => {
                    match login.await {
                        Ok(resp) => {
                            match resp.error.as_str() {
                                "ok" => {
                                    info!("{} ({}): login success, {}", resp.client_ip, self.username, resp.suc_msg.unwrap_or_default());
                                }
                                _ => {
                                    warn!("{} ({}): login failed, {}", resp.client_ip, self.username, resp.error);
                                }
                            }
                        }
                        Err(e) => {
                            warn!("{}: login failed: {}", self.username, e);
                        }
                    }
                }
                _ = &mut shutdown_signal => {
                    info!("{}: gracefully exiting", self.username);
                    break;
                }
            }
        }

        Ok(())
    }
}
