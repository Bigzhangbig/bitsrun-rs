# bitsrun

[![GitHub Workflow Status (CI)](https://img.shields.io/github/actions/workflow/status/Bigzhangbig/bitsrun-rs/ci.yml?logo=github&label=ci&labelColor=%23223227)](https://github.com/Bigzhangbig/bitsrun-rs/actions/workflows/ci.yml)
[![GitHub Workflow Status (Release)](https://img.shields.io/github/actions/workflow/status/Bigzhangbig/bitsrun-rs/release.yml?logo=github&label=release&labelColor=%23223227)](https://github.com/Bigzhangbig/bitsrun-rs/actions/workflows/release.yml)
[![GitHub release](https://img.shields.io/github/v/release/Bigzhangbig/bitsrun-rs?logo=github&labelColor=%23223227)](https://github.com/Bigzhangbig/bitsrun-rs/releases/latest)
[![Crates.io](https://img.shields.io/crates/d/bitsrun?logo=rust&labelColor=%23223227&color=%23dec867)](https://crates.io/crates/bitsrun)

🌐 A headless login and logout CLI for gateway (10.0.0.55) at BIT, now in Rust.

### ✨ Fork Highlights

This fork (`Bigzhangbig/bitsrun-rs`) introduces several enhancements over the original project, **developed with the assistance of Gemini CLI (AI)**:

- **🚀 Native macOS Wi-Fi Monitoring**: On macOS, the `keep-alive` daemon now utilizes the `SystemConfiguration` and `IOKit` frameworks to monitor network changes and system power events in real-time.
  - **Instant Re-login**: Automatically triggers a login attempt the moment you connect to a new Wi-Fi network or wake your Mac from sleep.
  - **Roaming Support**: Detects physical AP switching (BSSID change) even if the SSID remains the same (e.g., moving between buildings with `BIT-Web`), ensuring your session stays active without waiting for the polling interval.
  - **Zero-Latency & Low Power**: Uses event-driven system callbacks instead of constant polling.
- **🛡️ Robust Protocol Alignment**: Improved IP detection and auto-correction. If the gateway's detected IP differs from the local one, the client automatically aligns and re-authenticates to ensure success.

![CleanShot 2023-12-04 at 16 47 26@2x](https://github.com/spencerwooo/bitsrun-rs/assets/32114380/23343ba1-961c-41aa-b4b6-c09da93fb699)

> [!IMPORTANT]
> **AI-Assisted Development**: Significant portions of the logic in this fork, especially the hardware monitoring and protocol refinements, were generated or refactored using **Gemini CLI**. While extensively tested, users should be aware of the AI-driven nature of these changes.

## Disclaimer

**本软件仅供学习和研究使用，严禁用于任何非法用途。**

- **风险自担**：开发者（包括 AI 助手）不对因使用本软件导致的任何账户封禁、网络中断、数据丢失或法律纠纷承担责任。
- **不保证性**：由于校园网网关协议可能随时更新，本软件不保证在所有时间、所有环境下均能正常工作。
- **AI 生成代码**：本项目部分核心功能由 AI 生成，虽经人工验证，但仍可能存在边界情况下的异常行为，请根据实际情况谨慎使用。

## Install

#### One-line install (Linux / macOS, recommended)

- `curl -fsSL https://cdn.jsdelivr.net/gh/Bigzhangbig/bitsrun-rs@main/install.sh | sh -`

#### Ubuntu / Debian (recommended for `systemd` support)

- Download the latest `.deb` package from [Releases](https://github.com/Bigzhangbig/bitsrun-rs/releases/latest).
- `sudo apt install </path/to/file>.deb`

**If `bitsrun.service` systemd service required:**

- Edit `/lib/systemd/system/bitsrun.service` to specify absolute config path
- Then start service with `sudo systemctl start bitsrun`

#### Download binary

- Download the latest binary from [Releases](https://github.com/Bigzhangbig/bitsrun-rs/releases/latest).
- Uncompress file: `tar -xvf <file>.tar.gz`
- Move binary to `$PATH`, such as: `mv <file>/bitsrun ~/.local/bin/`

> [!NOTE]
> `cargo install bitsrun` 仍将安装由原作者维护的官方版本。如需使用本 Fork 分支的功能，请使用上述安装方式。

## Usage

To log into or out of the campus network, simply:

```console
$ bitsrun login -u <username> -p <password>
bitsrun: <ip> (<username>) logged in

$ bitsrun logout -u <username>
bitsrun: <ip> logged out
```

To check device login status:

```console
$ bitsrun status
bitsrun: <ip> (<username>) is online
┌────────────────┬───────────────┬───────────────┬─────────┐
│ Traffic Used   │ Online Time   │ User Balance  │ Wallet  │
├────────────────┼───────────────┼───────────────┼─────────┤
│ 188.10 GiB     │ 2 months      │ 10.00         │ 0.00    │
└────────────────┴───────────────┴───────────────┴─────────┘
```

To keep the session alive, use `bitsrun keep-alive`:

```console
$ bitsrun keep-alive
 INFO  bitsrun::daemon > starting daemon (<username>) with polling interval=3600s
 INFO  bitsrun::daemon > <ip> (<username>): login success,
 ...
 ^C INFO  bitsrun::daemon > <username>: gracefully exiting
```

> [!NOTE]
> Use available system service managers to run `bitsrun keep-alive` as a daemon. (e.g., `systemd` for Linux, `launchd` for macOS, and Windows Service for Windows).

## Available commands

```console
$ bitsrun --help
A headless login and logout CLI for 10.0.0.55 at BIT

Usage: bitsrun [OPTIONS] [COMMAND]

Commands:
  login         Login to the campus network
  logout        Logout from the campus network
  status        Check device login status
  config-paths  List all possible config file paths
  keep-alive    Poll the server with login requests to keep the session alive
  help          Print this message or the help of the given subcommand(s)

Options:
  -v, --verbose  Verbose output
  -h, --help     Print help
  -V, --version  Print version
```

> [!TIP]
> Use environment variable `NO_COLOR=true` to disable colored output.

## Config and credentials

To save your credentials and configurations, create config file `bit-user.json` under an available config path as:

```json
{
  "username": "<username>",
  "password": "<password>",
  "dm": true,
  "poll_interval": 3600
}
```

- **`dm` is for specifying whether the current device is a dumb terminal, and requires logging out through the alternative endpoint. Set to `true` (no quotes!) if the device you are working with is a dumb terminal.**
- `poll_interval` is an optional field for specifying the interval (in seconds) of polling login requests. Default is `3600` seconds (1 hour). Used by `bitsrun keep-alive` only.

Available config file paths can be listed with:

```console
$ bitsrun config-paths
bitsrun: list of possible config paths
┌──────────┬─────────────────────────────────────────────────────────────┐
│ Priority │ Possible Config Path                                        │
├──────────┼─────────────────────────────────────────────────────────────┤
│ 1        │ /Users/spencerwoo/.config/bit-user.json                     │
│ 2        │ /Users/spencerwoo/.config/bitsrun/bit-user.json             │
│ 3        │ /Users/spencerwoo/Library/Preferences/bitsrun/bit-user.json │
│ 4        │ bit-user.json                                               │
└──────────┴─────────────────────────────────────────────────────────────┘
```

> [!NOTE]
> The config file location is OS-dependent. Run the command to check the accepted locations on your system.

**Set permissions of this file to `600` on Linux and macOS, or `bitsrun` will refuse to read it.**

```console
$ chmod 600 <path/to/bit-user.json>
```

## Related

- [`zu1k/srun`](https://github.com/zu1k/srun) - Srun authentication system login tools. (Rust)
- [`Mmx233/BitSrunLoginGo`](https://github.com/Mmx233/BitSrunLoginGo) - 深澜校园网登录脚本 Go 语言版 (Go)
- [`vouv/srun`](https://github.com/vouv/srun) - An efficient client for BIT campus network. (Go)
- [`BITNP/bitsrun`](https://github.com/BITNP/bitsrun) - A headless login / logout script for 10.0.0.55 at BIT. (Python)

## License

[MIT](./LICENSE)
