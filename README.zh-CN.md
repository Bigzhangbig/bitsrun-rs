# bitsrun

[English](README.md) | 简体中文

[![GitHub Workflow Status (CI)](https://img.shields.io/github/actions/workflow/status/spencerwooo/bitsrun-rs/ci.yml?logo=github&label=ci&labelColor=%23223227)](https://github.com/spencerwooo/bitsrun-rs/actions/workflows/ci.yml)
[![GitHub Workflow Status (Release)](https://img.shields.io/github/actions/workflow/status/spencerwooo/bitsrun-rs/release.yml?logo=github&label=release&labelColor=%23223227)](https://github.com/spencerwooo/bitsrun-rs/actions/workflows/release.yml)
[![GitHub release](https://img.shields.io/github/v/release/spencerwooo/bitsrun-rs?logo=github&labelColor=%23223227)](https://github.com/spencerwooo/bitsrun-rs/releases/latest)
[![Crates.io](https://img.shields.io/crates/d/bitsrun?logo=rust&labelColor=%23223227&color=%23dec867)](https://crates.io/crates/bitsrun)

🌐 北京理工大学（BIT）校园网网关（10.0.0.55）的无界面登录/登出 CLI（Rust 版）。

如果你需要 Windows 服务方式运行守护进程，请阅读《Windows 服务设置》文档：[`docs/windows-service.zh-CN.md`](docs/windows-service.zh-CN.md)。

## 安装

- 一行安装（Linux / macOS，推荐）：`curl -fsSL https://cdn.jsdelivr.net/gh/spencerwooo/bitsrun-rs@main/install.sh | sh -`
- Ubuntu / Debian（推荐，支持 `systemd`）：
  - 从 [Releases](https://github.com/spencerwooo/bitsrun-rs/releases/latest) 下载最新 `.deb`
  - `sudo apt install </path/to/file>.deb`
  - 如需 `bitsrun.service`：编辑 `/lib/systemd/system/bitsrun.service` 指定绝对配置路径，然后 `sudo systemctl start bitsrun`
- Cargo：`cargo install bitsrun`
- 直接下载二进制：从 [Releases](https://github.com/spencerwooo/bitsrun-rs/releases/latest) 下载，`tar -xvf <file>.tar.gz` 解压后将 `bitsrun` 移动到 `PATH`

## 使用

登录或登出：

```console
$ bitsrun login -u <username> -p <password>
bitsrun: <ip> (<username>) logged in

$ bitsrun logout -u <username>
bitsrun: <ip> logged out
```

查询设备登录状态：

```console
$ bitsrun status
bitsrun: <ip> (<username>) is online
┌────────────────┬───────────────┬───────────────┬─────────┐
│ Traffic Used   │ Online Time   │ User Balance  │ Wallet  │
├────────────────┼───────────────┼───────────────┼─────────┤
│ 188.10 GiB     │ 2 months      │ 10.00         │ 0.00    │
└────────────────┴───────────────┴───────────────┴─────────┘
```

保持会话存活：

```console
$ bitsrun keep-alive
 INFO  bitsrun::daemon > starting daemon (<username>) with polling interval=3600s
 INFO  bitsrun::daemon > <ip> (<username>): login success,
 ...
 ^C INFO  bitsrun::daemon > <username>: gracefully exiting
```

> 使用系统可用的服务管理器在后台运行 `bitsrun keep-alive`（例如 Linux 的 `systemd`、macOS 的 `launchd`，以及 Windows 的 Windows 服务）。Windows 服务详细见 [`docs/windows-service.zh-CN.md`](docs/windows-service.zh-CN.md)。

## 可用命令

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

提示：设置环境变量 `NO_COLOR=true` 可禁用彩色输出。

## Windows 服务（仅限 Windows）

当由 Windows 服务控制管理器（SCM）启动时，`bitsrun` 会在服务上下文中运行 `keep-alive` 守护进程，并向 SCM 上报状态。日志同时写入 Windows 事件日志（来源：`Bitsrun`）以及可执行文件同目录的本地日志文件 `bitsrun_service.log`。

快速设置：

```powershell
sc.exe create Bitsrun binPath= "C:\Program Files\Bitsrun\bitsrun.exe" DisplayName= "Bitsrun" start= auto
sc.exe description Bitsrun "Bitsrun keep-alive service for BIT gateway"
```

- 将 `bit-user.json` 放在 `bitsrun.exe` 同目录，服务将优先读取该路径的配置
- 管理服务：`sc.exe start Bitsrun`、`sc.exe stop Bitsrun`、`sc.exe query Bitsrun`
- 删除服务：`sc.exe stop Bitsrun` 后执行 `sc.exe delete Bitsrun`
- 查看日志：
  - 文件日志：与 `bitsrun.exe` 同目录的 `bitsrun_service.log`
  - 事件日志：事件查看器 → Windows 日志 → 应用程序 → 来源 `Bitsrun`

> 注意：创建与启动服务需要管理员权限，以正确注册事件日志来源。
>
> 详细指南：[`docs/windows-service.zh-CN.md`](docs/windows-service.zh-CN.md)

## 配置与凭据

将你的配置保存到 `bit-user.json`（OS 依赖的配置路径可通过 `bitsrun config-paths` 查看）。示例：

```json
{
  "username": "<username>",
  "password": "<password>",
  "dm": true,
  "poll_interval": 3600
}
```

- `dm`：若当前设备属于“哑终端”，登出需要使用备用端点，如果你使用的设备是**普通终端**，请设为 `true`
- `poll_interval`：守护进程轮询登录请求的间隔（秒），默认为 `3600`

Linux / macOS 上需将此文件权限设为 `600`，否则 `bitsrun` 将拒绝读取：

```console
$ chmod 600 <path/to/bit-user.json>
```

## 相关项目

- [`zu1k/srun`](https://github.com/zu1k/srun) - 深澜认证系统登录工具（Rust）
- [`Mmx233/BitSrunLoginGo`](https://github.com/Mmx233/BitSrunLoginGo) - 深澜校园网登录脚本（Go）
- [`vouv/srun`](https://github.com/vouv/srun) - 高效的 BIT 校园网客户端（Go）
- [`BITNP/bitsrun`](https://github.com/BITNP/bitsrun) - Python 版无界面登录/登出脚本

## 许可证

[MIT](./LICENSE)
