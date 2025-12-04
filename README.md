# bitsrun

[![GitHub Workflow Status (CI)](https://img.shields.io/github/actions/workflow/status/spencerwooo/bitsrun-rs/ci.yml?logo=github&label=ci&labelColor=%23223227)](https://github.com/spencerwooo/bitsrun-rs/actions/workflows/ci.yml)
[![GitHub Workflow Status (Release)](https://img.shields.io/github/actions/workflow/status/spencerwooo/bitsrun-rs/release.yml?logo=github&label=release&labelColor=%23223227)](https://github.com/spencerwooo/bitsrun-rs/actions/workflows/release.yml)
[![GitHub release](https://img.shields.io/github/v/release/spencerwooo/bitsrun-rs?logo=github&labelColor=%23223227)](https://github.com/spencerwooo/bitsrun-rs/releases/latest)
[![Crates.io](https://img.shields.io/crates/d/bitsrun?logo=rust&labelColor=%23223227&color=%23dec867)](https://crates.io/crates/bitsrun)

🌐 A headless login and logout CLI for gateway (10.0.0.55) at BIT, now in Rust.

English | [简体中文](README_CN.md)

![CleanShot 2023-12-04 at 16 47 26@2x](https://github.com/spencerwooo/bitsrun-rs/assets/32114380/23343ba1-961c-41aa-b4b6-c09da93fb699)

## Install

#### One-line install (Linux / macOS, recommended)

- `curl -fsSL https://cdn.jsdelivr.net/gh/spencerwooo/bitsrun-rs@main/install.sh | sh -`

#### Ubuntu / Debian (recommended for `systemd` support)

- Download the latest `.deb` package from [Releases](https://github.com/spencerwooo/bitsrun-rs/releases/latest).
- `sudo apt install </path/to/file>.deb`

**If `bitsrun.service` systemd service required:**

- Edit `/lib/systemd/system/bitsrun.service` to specify absolute config path
- Then start service with `sudo systemctl start bitsrun`

#### Cargo

- `cargo install bitsrun`

#### Download binary

- Download the latest binary from [Releases](https://github.com/spencerwooo/bitsrun-rs/releases/latest).
- Uncompress file: `tar -xvf <file>.tar.gz`
- Move binary to `$PATH`, such as: `mv <file>/bitsrun ~/.local/bin/`

#### Windows Service

Windows users can run bitsrun as a Windows service for automatic login at startup and keeping the session alive. See [Windows Service Installation](#windows-service-installation) for detailed instructions.

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

## Windows Service Installation

Windows users can run bitsrun as a system service for automatic startup at boot and keeping the session alive in the background.

### Method 1: Native Windows Service (Recommended)

Starting from version 0.5.0, bitsrun has native Windows service support using the `windows-service` crate. This is the recommended method as it provides better integration with Windows Service Control Manager (SCM).

#### Prerequisites

1. Download the Windows version of bitsrun executable from [Releases](https://github.com/spencerwooo/bitsrun-rs/releases/latest)
2. Place the executable in a permanent location (e.g., `C:\Program Files\bitsrun\bitsrun.exe`)
3. Create a config file `bit-user.json` and place it in an appropriate location (e.g., `C:\Program Files\bitsrun\bit-user.json`)

#### Installation Steps

1. Open Command Prompt or PowerShell as Administrator

2. Install the service using the built-in `sc` command:

```powershell
# Navigate to bitsrun directory
cd "C:\Program Files\bitsrun"

# Install service using native Windows service mode
sc create bitsrun binPath= "C:\Program Files\bitsrun\bitsrun.exe windows-service" start= auto
sc description bitsrun "BIT Campus Network Auto Login Service"
```

3. Start the service:

```powershell
sc start bitsrun
```

#### Service Management

```powershell
# Check service status
sc query bitsrun

# Stop service
sc stop bitsrun

# Restart service
sc stop bitsrun
sc start bitsrun

# Remove service
sc delete bitsrun
```

> [!NOTE]
> The native Windows service mode uses the `windows-service` command which integrates directly with Windows Service Control Manager (SCM). The service will automatically read configuration from default config paths or you can place `bit-user.json` in the same directory as the executable.

### Method 2: Using NSSM (Alternative)

NSSM (Non-Sucking Service Manager) is an easy-to-use Windows service management tool that can be used as an alternative method.

### Method 2: Using NSSM (Alternative)

NSSM (Non-Sucking Service Manager) is an easy-to-use Windows service management tool that can be used as an alternative method.

#### Prerequisites

Same as Method 1 above.

#### Installation Steps

1. Download [NSSM](https://nssm.cc/download)
2. Open Command Prompt or PowerShell as Administrator
3. Run the following command to install the service:

```powershell
# Navigate to NSSM directory
cd C:\path\to\nssm\win64

# Install service (will open GUI configuration interface)
.\nssm.exe install bitsrun
```

4. Configure in NSSM GUI:
   - **Path**: `C:\Program Files\bitsrun\bitsrun.exe`
   - **Startup directory**: `C:\Program Files\bitsrun`
   - **Arguments**: `keep-alive --config C:\Program Files\bitsrun\bit-user.json`
   - **Service name**: `bitsrun`

5. Click "Install service" button

6. Start the service:

```powershell
.\nssm.exe start bitsrun
```

#### Service Management Commands

```powershell
# Start service
nssm start bitsrun

# Stop service
nssm stop bitsrun

# Remove service
nssm remove bitsrun confirm
```

### Method 3: Using sc Command with keep-alive (Legacy)

You can also use Windows built-in `sc` command to create a service with the `keep-alive` subcommand:

### Method 3: Using sc Command with keep-alive (Legacy)

You can also use Windows built-in `sc` command to create a service with the `keep-alive` subcommand:

```powershell
# Run as Administrator
sc create bitsrun binPath= "C:\Program Files\bitsrun\bitsrun.exe keep-alive --config C:\Program Files\bitsrun\bit-user.json" start= auto
sc description bitsrun "BIT Campus Network Auto Login Service"
sc start bitsrun
```

> [!NOTE]
> This method runs bitsrun in keep-alive mode but doesn't use native Windows service integration. Method 1 (Native Windows Service) is recommended for better integration with Windows SCM.

### Additional Notes
### Additional Notes

You can check the service status using Windows Service Manager (`services.msc`) or command line:

```powershell
sc query bitsrun
```

You can also view service logs in Windows Event Viewer under "Windows Logs" > "Application" for troubleshooting.

> [!IMPORTANT]
> Ensure that the config file `bit-user.json` contains the correct username and password, and use absolute paths. On Windows, there is no need to set file permissions to 600.

## Project Structure

This project consists of the following main source files:

| File | Purpose |
|------|---------|
| `src/main.rs` | Entry point of the application, handles command-line interface initialization and orchestrates the execution flow |
| `src/cli.rs` | Defines command-line arguments, subcommands, and their configurations using clap |
| `src/client.rs` | Core SRUN client implementation with login/logout logic and portal communication |
| `src/config.rs` | Configuration file handling, including path enumeration and validation |
| `src/daemon.rs` | Daemon mode implementation for keeping sessions alive with periodic login requests |
| `src/tables.rs` | Pretty-printing utilities for displaying status tables and configuration paths |
| `src/user.rs` | User credential management from config files or interactive prompts |
| `src/xencode.rs` | Encryption algorithm implementation for SRUN portal authentication |

## Related

- [`zu1k/srun`](https://github.com/zu1k/srun) - Srun authentication system login tools. (Rust)
- [`Mmx233/BitSrunLoginGo`](https://github.com/Mmx233/BitSrunLoginGo) - 深澜校园网登录脚本 Go 语言版 (Go)
- [`vouv/srun`](https://github.com/vouv/srun) - An efficient client for BIT campus network. (Go)
- [`BITNP/bitsrun`](https://github.com/BITNP/bitsrun) - A headless login / logout script for 10.0.0.55 at BIT. (Python)

## License

[MIT](./LICENSE)
