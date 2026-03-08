use std::net::IpAddr;
use std::time::Duration;

use crate::xencode::fkbase64;
use crate::xencode::xencode;
use anyhow::bail;
use anyhow::Context;
use anyhow::Result;
use hmac::Hmac;
use hmac::Mac;
use md5::Digest;
use md5::Md5;
use owo_colors::OwoColorize;
use owo_colors::Stream::Stdout;
use reqwest::Client;

use log::{debug, info};
use serde::Deserialize;
use serde::Serialize;
use serde_json::json;
use sha1::Sha1;

/// Constants used for the /srun_portal endpoint
pub const SRUN_PORTAL: &str = "http://10.0.0.55";
pub const SRUN_TYPE: &str = "1";
pub const SRUN_N: &str = "200";

/// An arbitrary HTTP URL for srun to redirect
pub const CAPTIVE_PORTAL_TEST: &str = "http://www.bit.edu.cn";

/// The response from the `/rad_user_info` endpoint
///
/// This response is used to determine if the device is logged in or not, and if it is logged in,
/// what the current login state is (i.e., IP address, user balance, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SrunLoginState {
    // always present
    pub error: String,
    pub online_ip: IpAddr,

    // present when logged in
    #[serde(rename = "ServerFlag", skip_serializing_if = "Option::is_none")]
    pub server_flag: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub add_time: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all_bytes: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bytes_in: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bytes_out: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checkout_date: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keepalive_time: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub products_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub real_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remain_bytes: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remain_seconds: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sum_bytes: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sum_seconds: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sysver: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_balance: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_charge: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_mac: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wallet_balance: Option<f64>,

    // present when logged out
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_ip: Option<IpAddr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_msg: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub res: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub srun_ver: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub st: Option<i64>,
}

/// Get the login state of the current device
pub async fn get_login_state(client: &Client, verbose: bool) -> Result<SrunLoginState> {
    // call /rad_user_info with callback=jsonp to get the login state
    let params = [("callback", "jsonp")];
    let url = format!("{}/cgi-bin/rad_user_info", SRUN_PORTAL);

    // get the response and extract the json
    let resp = client
        .get(&url)
        .query(&params)
        .send()
        .await
        .with_context(|| "failed to get login state")?;
    let raw_text = resp.text().await?;

    if verbose {
        println!(
            "{} status response from portal:\n{}",
            "bitsrun:".if_supports_color(Stdout, |t| t.blue()),
            raw_text.if_supports_color(Stdout, |t| t.dimmed())
        );
    }

    // valid json starts at index 6 and ends at the second to last character
    if raw_text.len() < 8 {
        bail!("login status response too short: `{}`", raw_text)
    }
    let raw_json = &raw_text[6..raw_text.len() - 1];
    let parsed_json = match serde_json::from_str::<SrunLoginState>(raw_json) {
        Ok(json) => json,
        Err(err) => bail!("failed to parse login status response: {}", err),
    };

    Ok(parsed_json)
}

/// Get the ac_id of the current device by visiting a URL
async fn get_acid_by_url(client: &Client, url: &str) -> Result<String> {
    let resp = client.get(url).send().await.with_context(|| {
        format!(
            "failed to get ac_id from `{}`",
            url.if_supports_color(Stdout, |t| t.underline())
        )
    })?;
    let redirect_url = resp.url().to_string();
    let parsed_url = url::Url::parse(&redirect_url).with_context(|| {
        format!(
            "failed to parse url `{}`",
            redirect_url.if_supports_color(Stdout, |t| t.underline())
        )
    })?;

    let mut query = parsed_url.query_pairs().into_owned();
    let ac_id = query.find(|(key, _)| key == "ac_id").with_context(|| {
        format!(
            "failed to get ac_id from `{}`",
            redirect_url.if_supports_color(Stdout, |t| t.underline())
        )
    })?;
    Ok(ac_id.1)
}

/// Check if the current connection is truly online or intercepted.
/// Returns Ok(None) if truly online (204 received).
/// Returns Ok(Some(ac_id)) if intercepted by captive portal.
/// Returns Err if network is not reachable at all.
pub(crate) async fn check_connectivity(client: &Client) -> Result<Option<String>> {
    // Use a domestic connectivity check endpoint for better stability in China
    let target = "http://connect.rom.miui.com/generate_204";

    let resp = client
        .get(target)
        .timeout(Duration::from_secs(3))
        .send()
        .await?;

    if resp.status().as_u16() == 204 {
        // Truly online
        return Ok(None);
    }

    // If not 204, we are likely intercepted. Try to get ac_id from redirect or body.
    let final_url = resp.url().to_string();
    if let Ok(parsed_url) = url::Url::parse(&final_url) {
        if let Some((_, ac_id)) = parsed_url.query_pairs().find(|(k, _)| k == "ac_id") {
            return Ok(Some(ac_id.into_owned()));
        }
    }

    // Fallback: search for ac_id in the response body (for WISPr XML like responses)
    let body = resp.text().await.unwrap_or_default();
    if let Some(pos) = body.find("index_") {
        let sub = &body[pos..];
        if let Some(ac_pos) = sub.find("ac_id=") {
            let ac_val = sub[ac_pos + 6..]
                .split(|c: char| !c.is_ascii_digit())
                .next()
                .unwrap_or("43");
            return Ok(Some(ac_val.to_string()));
        }
    }

    // Default to a sensible ac_id if intercepted but can't find specific id
    Ok(Some("43".to_string()))
}

/// Get the ac_id of the current device
async fn get_acid(client: &Client) -> Result<String> {
    // Try to visit `CAPTIVE_PORTAL_TEST`.
    // If not logged in, it will be redirected to `SRUN_PORTAL` with ac_id.
    // Otherwise, we fall back to visit `SRUN_PORTAL` directly.
    // https://en.wikipedia.org/wiki/Captive_portal#Detection
    //
    // Because of ITC's double authentication mechanism, visiting `SRUN_PORTAL` directly is not preferred.
    // https://itc.bit.edu.cn/fwzn/zxbl/f2c0c8e939ce4e9cace880d5403fe4b5.htm
    get_acid_by_url(client, CAPTIVE_PORTAL_TEST)
        .await
        .or(get_acid_by_url(client, SRUN_PORTAL).await)
}

/// SRUN portal response type when calling login/logout
///
/// Note that fields that are not used are omitted
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SrunPortalResponse {
    // present only when logging in and succeeds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suc_msg: Option<String>,

    // always present on logins and logouts
    pub client_ip: IpAddr,
    pub online_ip: IpAddr,
    pub error: String,
    pub error_msg: String,
    pub res: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SrunChallenge {
    // the only useful field that must be present
    pub challenge: String,
    pub client_ip: Option<IpAddr>,
}

/// SRUN client
#[derive(Debug)]
pub struct SrunClient {
    // reusable http client
    pub http_client: Client,

    // srun login info, username is student id
    pub username: String,
    pub password: String,

    // srun portal info
    pub ip: IpAddr,
    pub ac_id: String,
    pub dm: bool, // whether the device is authenticated with its mac address
    pub login_state: SrunLoginState,
}

impl SrunClient {
    /// Create a new SRUN client, where the http client will be reused if provided
    ///
    /// # Arguments
    ///
    /// * `username` - The username of the SRUN account (student id)
    /// * `password` - The password of the SRUN account
    /// * `ip` - The IP address (`online_ip` from the login portal if not specified)
    /// * `dm` - Whether the device is authenticated through the campus login portal with its mac
    ///   address (important for dumb terminals!!!)
    /// * `http_client` - The http client to be used (a new one will be created if not specified)
    pub async fn new(
        username: String,
        password: String,
        http_client: Option<Client>,
        ip: Option<IpAddr>,
        dm: Option<bool>,
    ) -> Result<SrunClient> {
        let http_client = http_client.unwrap_or_else(|| {
            Client::builder()
                .no_proxy()
                .connect_timeout(Duration::from_millis(400))
                .timeout(Duration::from_millis(400))
                .build()
                .unwrap_or_default()
        });

        // Use a 1s timeout for initial discovery to allow for multiple micro-retries or slow responses
        let ac_id = tokio::time::timeout(Duration::from_secs(1), get_acid(&http_client))
            .await
            .context("Discovery timeout (ac_id)")??;

        let login_state =
            tokio::time::timeout(Duration::from_secs(1), get_login_state(&http_client, false))
                .await
                .context("Discovery timeout (login_state)")??;

        let ip = ip.unwrap_or(login_state.online_ip);
        let dm = dm.unwrap_or(false);
        Ok(SrunClient {
            http_client,
            username,
            password,
            ip,
            ac_id,
            dm,
            login_state,
        })
    }

    /// Login to the SRUN portal
    pub async fn login(&self, force: bool, verbose: bool) -> Result<SrunPortalResponse> {
        // check if already logged in
        if (self.login_state.error == "ok") & !force {
            bail!(
                "{} already logged in",
                self.login_state
                    .online_ip
                    .to_string()
                    .if_supports_color(Stdout, |t| t.underline())
            )
        }

        // construct checksum and crypto encodings
        let (token, real_ip) = self.get_challenge(verbose).await?;
        let real_ip_str = real_ip.to_string();

        let chksum_data = json!({
            "username": self.username.clone(),
            "password": self.password.clone(),
            "ip": real_ip_str,
            "acid": self.ac_id.clone(),
            "enc_ver": String::from("srun_bx1"),
        });

        let json_chksum_data = serde_json::to_string(&chksum_data)?;
        let encoded_data = xencode(json_chksum_data.as_str(), token.as_str());
        let info = format!("{}{}", "{SRBX1}", fkbase64(encoded_data));

        // construct param payload
        let mac = Hmac::<Md5>::new_from_slice(token.as_bytes())?;
        let hmd5 = format!("{:x}", mac.finalize().into_bytes());

        let chksum = {
            let chk = format!(
                "{0}{1}{0}{2}{0}{3}{0}{4}{0}{5}{0}{6}{0}{7}",
                &token,
                &self.username,
                &hmd5,
                &self.ac_id,
                &real_ip_str,
                &SRUN_N,
                &SRUN_TYPE,
                &info
            );
            let mut hasher = Sha1::new();
            hasher.update(chk);
            format!("{:x}", hasher.finalize())
        };

        // construct request body
        let password_encoded = format!("{}{}", "{MD5}", hmd5);
        let params = [
            ("callback", "jsonp"),
            ("action", "login"),
            ("username", self.username.as_str()),
            ("password", password_encoded.as_str()),
            ("chksum", chksum.as_str()),
            ("info", info.as_str()),
            ("ac_id", self.ac_id.as_str()),
            ("ip", real_ip_str.as_str()),
            ("type", SRUN_TYPE),
            ("n", SRUN_N),
        ];
        let url = format!("{}/cgi-bin/srun_portal", SRUN_PORTAL);

        debug!("Portal Request: {}?{:?}", url, params);

        // send login request
        let resp = self
            .http_client
            .get(&url)
            .query(&params)
            .send()
            .await
            .with_context(|| "failed to send request when logging in")?;
        let raw_text = resp.text().await?;
        debug!("Portal Output: {}", raw_text);

        if verbose {
            println!(
                "{} login response from portal:\n{}",
                "bitsrun:".if_supports_color(Stdout, |t| t.blue()),
                raw_text.if_supports_color(Stdout, |t| t.dimmed())
            );
        }

        if raw_text.len() < 8 {
            bail!("login response too short: `{}`", raw_text)
        }
        let raw_json = &raw_text[6..raw_text.len() - 1];
        serde_json::from_str::<SrunPortalResponse>(raw_json)
            .with_context(|| format!("failed to parse malformed login response:\n  {}", raw_json))
    }

    /// Logout of the SRUN portal
    pub async fn logout(&self, force: bool, verbose: bool) -> Result<SrunPortalResponse> {
        // check if already logged out
        if (self.login_state.error == "not_online_error") & !force {
            bail!(
                "{} already logged out",
                self.ip
                    .to_string()
                    .if_supports_color(Stdout, |t| t.underline())
            )
        }

        // check if username match
        let logged_in_username = self.login_state.user_name.clone().unwrap_or_default();
        if logged_in_username != self.username {
            println!(
                "{} logged in user {} does not match yourself {}, logging out anyway",
                "warning:".if_supports_color(Stdout, |t| t.yellow()),
                format!("({})", logged_in_username).dimmed(),
                format!("({})", self.username).dimmed()
            );
        }

        // check if ip match
        let logged_in_ip = self.login_state.online_ip;
        if logged_in_ip != self.ip {
            println!(
                "{} logged in ip (`{}`) does not match `{}`, things may not work as expected",
                "warning:".if_supports_color(Stdout, |t| t.yellow()),
                logged_in_ip
                    .to_string()
                    .if_supports_color(Stdout, |t| t.underline()),
                self.ip
                    .to_string()
                    .if_supports_color(Stdout, |t| t.underline())
            );
        }

        // perform logout action
        let url = {
            // dumb terminals use a different endpoint (dm logout)
            match self.dm {
                true => format!("{}/cgi-bin/rad_user_dm", SRUN_PORTAL),
                false => format!("{}/cgi-bin/srun_portal", SRUN_PORTAL),
            }
        };

        let ip_str = self.ip.to_string();
        let mut params = vec![
            ("callback", String::from("jsonp")),
            ("ip", self.ip.to_string()),
            ("username", logged_in_username.clone()),
        ];

        if self.dm {
            use chrono::Utc;
            let timestamp = Utc::now().timestamp().to_string();
            let unbind = String::from("1");

            let sign = {
                let mut hasher = Sha1::new();
                let sn = format!(
                    "{0}{1}{2}{3}{0}",
                    timestamp, logged_in_username, ip_str, unbind
                );

                hasher.update(sn);
                format!("{:x}", hasher.finalize())
            };

            params.push(("time", timestamp));
            params.push(("unbind", unbind));
            params.push(("sign", sign));
        } else {
            params.push(("action", String::from("logout")));
            params.push(("ac_id", self.ac_id.clone()));
        }

        let resp = self
            .http_client
            .get(&url)
            .query(&params)
            .send()
            .await
            .with_context(|| "failed to send request when logging out")?;
        let raw_text = resp.text().await?;

        if verbose {
            println!(
                "{} logout response from portal:\n{}",
                "bitsrun:".if_supports_color(Stdout, |t| t.blue()),
                raw_text.if_supports_color(Stdout, |t| t.dimmed())
            );
        }

        if raw_text.len() < 8 {
            bail!("login response too short: `{}`", raw_text)
        }
        let raw_json = &raw_text[6..raw_text.len() - 1];
        serde_json::from_str::<SrunPortalResponse>(raw_json)
            .with_context(|| format!("failed to parse malformed logout response:\n  {}", raw_json))
    }

    /// Ensure the client is online by checking connectivity and performing login if needed.
    pub async fn ensure_online(&self) -> Result<()> {
        match check_connectivity(&self.http_client).await {
            Ok(None) => {
                debug!("Client is already online.");
                return Ok(());
            }
            Ok(Some(ac_id)) => {
                info!(
                    "Client intercepted (ac_id={}), initiating smart login...",
                    ac_id
                );
            }
            Err(e) => {
                info!(
                    "Network unreachable ({}), waiting for interface to be ready...",
                    e
                );
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        }

        // Attempt login retries every 0.5s for 5s total
        for i in 1..=10 {
            match self.login(true, false).await {
                Ok(resp) if resp.error == "ok" || resp.error == "ip_already_online_error" => {
                    tokio::time::sleep(Duration::from_millis(500)).await;
                    if check_connectivity(&self.http_client).await.is_ok() {
                        info!("Smart login success (attempt {}).", i);
                        return Ok(());
                    }
                }
                _ => {
                    debug!("Login attempt {} failed, retrying in 500ms...", i);
                }
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
        bail!("Failed to ensure online after 10 attempts (5s)")
    }

    async fn get_challenge(&self, _verbose: bool) -> Result<(String, IpAddr)> {
        let mut request_ip = self.ip;
        let mut challenge = String::new();

        // Try up to 2 times to align with gateway's detected IP
        for attempt in 1..=2 {
            let ip_str = request_ip.to_string();
            let params = [
                ("callback", "jsonp"),
                ("username", self.username.as_str()),
                ("ip", &ip_str),
            ];
            let url = format!("{}/cgi-bin/get_challenge", SRUN_PORTAL);

            debug!(
                "Challenge Request (attempt {}): {}?{:?}",
                attempt, url, params
            );

            let resp = self
                .http_client
                .get(&url)
                .query(&params)
                .send()
                .await
                .with_context(|| "failed to get challenge")?;
            let raw_text = resp.text().await?;
            debug!("Challenge Output (attempt {}): {}", attempt, raw_text);

            if raw_text.len() < 8 {
                bail!("challenge response too short: `{}`", raw_text)
            }
            let raw_json = &raw_text[6..raw_text.len() - 1];
            let parsed_json =
                serde_json::from_str::<SrunChallenge>(raw_json).with_context(|| {
                    format!(
                        "failed to parse malformed get_challenge response:\n  {}",
                        raw_json
                    )
                })?;

            challenge = parsed_json.challenge;

            if let Some(detected_ip) = parsed_json.client_ip {
                if detected_ip != request_ip {
                    info!("IP Mismatch! Requested: {}, Gateway saw: {}. Re-fetching challenge with correct IP...", request_ip, detected_ip);
                    request_ip = detected_ip;
                    continue; // Fetch again with the correct IP
                }
            }

            // If we are here, either IP matches or gateway didn't return client_ip
            return Ok((challenge, request_ip));
        }

        Ok((challenge, request_ip))
    }
}
