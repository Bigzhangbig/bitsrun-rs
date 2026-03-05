# bitsrun

[English](README.md)  |  简体中文

[![GitHub Workflow Status (CI)](https://img.shields.io/github/actions/workflow/status/Bigzhangbig/bitsrun-rs/ci.yml?logo=github&label=ci&labelColor=%23223227)](https://github.com/Bigzhangbig/bitsrun-rs/actions/workflows/ci.yml)
[![GitHub Workflow Status (Release)](https://img.shields.io/github/actions/workflow/status/Bigzhangbig/bitsrun-rs/release.yml?logo=github&label=release&labelColor=%23223227)](https://github.com/Bigzhangbig/bitsrun-rs/actions/workflows/release.yml)
[![GitHub release](https://img.shields.io/github/v/release/Bigzhangbig/bitsrun-rs?logo=github&labelColor=%23223227)](https://github.com/Bigzhangbig/bitsrun-rs/releases/latest)

🌐 针对北京理工大学 (BIT) 网关 (10.0.0.55) 的无头登录与注销命令行工具，现已支持 Rust。

### ✨ Fork 版本亮点

本分支 (`Bigzhangbig/bitsrun-rs`) 在原项目基础上引入了多项增强功能，**由 Gemini CLI (AI) 辅助开发完成**：

- **🚀 原生 macOS Wi-Fi 监控**：在 macOS 上，`keep-alive` 守护进程利用 `SystemConfiguration` 和 `IOKit` 框架实时监控网络变化和系统电源事件。
  - **即时重连**：在连接到新 Wi-Fi 或从休眠中唤醒后，立即自动触发登录尝试。
  - **漫游支持**：即使 SSID 相同（如在不同教学楼间移动），也能探测到物理 AP 的切换（BSSID 变更），确保会话持续活跃而无需等待轮询间隔。
  - **零延迟与低功耗**：使用系统事件回调而非恒定的定时轮询。
- **🛡️ 健壮的协议对齐**：改进了 IP 探测与自动校正逻辑。如果网关探测到的 IP 与本地不一致，客户端会自动对齐并重新认证，确保登录成功。

![CleanShot 2023-12-04 at 16 47 26@2x](https://github.com/spencerwooo/bitsrun-rs/assets/32114380/23343ba1-961c-41aa-b4b6-c09da93fb699)

> [!IMPORTANT]
> **AI 辅助开发**：本分支中的大量逻辑（尤其是硬件监控和协议优化部分）是由 **Gemini CLI** 生成或重构的。虽经测试，用户仍应了解这些改动由 AI 驱动。

## 免责声明

**本软件仅供学习和研究使用，严禁用于任何非法用途。**

- **风险自担**：开发者（包括 AI 助手）不对因使用本软件导致的任何账户封禁、网络中断、数据丢失或法律纠纷承担责任。
- **不保证性**：由于校园网网关协议可能随时更新，本软件不保证在所有时间、所有环境下均能正常工作。
- **AI 生成代码**：本项目部分核心功能由 AI 生成，虽经人工验证，但仍可能存在边界情况下的异常行为，请根据实际情况谨慎使用。

## 安装

#### 一键安装 (Linux / macOS, 推荐)

- `curl -fsSL https://cdn.jsdelivr.net/gh/Bigzhangbig/bitsrun-rs@main/install.sh | sh -`

#### Ubuntu / Debian (推荐用于 `systemd` 支持)

- 从 [Releases](https://github.com/Bigzhangbig/bitsrun-rs/releases/latest) 下载最新的 `.deb` 安装包。
- `sudo apt install </path/to/file>.deb`

**如果需要 `bitsrun.service` systemd 服务：**

- 编辑 `/lib/systemd/system/bitsrun.service` 以指定绝对配置路径
- 然后使用 `sudo systemctl start bitsrun` 启动服务

#### 下载二进制文件

- 从 [Releases](https://github.com/Bigzhangbig/bitsrun-rs/releases/latest) 下载最新的二进制文件。
- 解压文件：`tar -xvf <file>.tar.gz`
- 将二进制文件移动到 `$PATH`，例如：`mv <file>/bitsrun ~/.local/bin/`

> [!NOTE]
> `cargo install bitsrun` 仍将安装由原作者维护的官方版本。如需使用本 Fork 分支的功能，请使用上述安装方式。

## 使用方法

要登录或注销校园网，只需：

```console
$ bitsrun login -u <用户名> -p <密码>
bitsrun: <ip> (<用户名>) 已登录

$ bitsrun logout -u <用户名>
bitsrun: <ip> 已注销
```

检查设备登录状态：

```console
$ bitsrun status
bitsrun: <ip> (<username>) is online
┌────────────────┬───────────────┬───────────────┬─────────┐
│ Traffic Used   │ Online Time   │ User Balance  │ Wallet  │
├────────────────┼───────────────┼───────────────┼─────────┤
│ 188.10 GiB     │ 2 months      │ 10.00         │ 0.00    │
└────────────────┴───────────────┴───────────────┴─────────┘
```

使用 `bitsrun keep-alive` 保持会话活跃：

```console
$ bitsrun keep-alive
 INFO  bitsrun::daemon > starting daemon (<username>) with polling interval=3600s
 INFO  bitsrun::daemon > <ip> (<username>): login success,
 ...
 ^C INFO  bitsrun::daemon > <username>: gracefully exiting
```

> [!NOTE]
> 使用可用的系统服务管理器（如 Linux 的 `systemd`、macOS 的 `launchd` 或 Windows 服务）将 `bitsrun keep-alive` 作为守护进程运行。

### 🍏 macOS 开机自启动配置 (LaunchAgent)

为了在您登录 macOS 时自动启动 `bitsrun keep-alive`：

1. 查找您的 `bitsrun` 二进制文件路径：
```bash
which bitsrun
# 示例输出：/Users/harvey/.local/bin/bitsrun
```

2. 在 `~/Library/LaunchAgents/com.bigzhangbig.bitsrun.plist` 创建一个 plist 文件（请将 `/path/to/bitsrun` 和 `你的用户名` 替换为实际信息）：

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.bigzhangbig.bitsrun</string>
    <key>ProgramArguments</key>
    <array>
        <string>/path/to/bitsrun</string>
        <string>keep-alive</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>/Users/你的用户名/Library/Logs/bitsrun.log</string>
    <key>StandardErrorPath</key>
    <string>/Users/你的用户名/Library/Logs/bitsrun.log</string>
</dict>
</plist>
```

3. 加载并启动服务：
```bash
launchctl load ~/Library/LaunchAgents/com.bigzhangbig.bitsrun.plist
```

3. 查看运行日志：
```bash
tail -f ~/Library/Logs/bitsrun.log
```

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

> [!TIP]
> 使用环境变量 `NO_COLOR=true` 禁用彩色输出。

## 配置与凭据

要保存您的凭据和配置，请在可用的配置路径下创建配置文件 `bit-user.json`：

```json
{
  "username": "<用户名>",
  "password": "<密码>",
  "dm": true,
  "poll_interval": 3600
}
```

- **`dm` 用于指定当前设备是否为哑终端，需要通过替代端点进行注销。如果当前设备是哑终端，请设置为 `true`（不要加引号！）。**
- `poll_interval` 是一个可选字段，用于指定轮询登录请求的间隔（以秒为单位）。默认值为 `3600` 秒（1 小时）。仅供 `bitsrun keep-alive` 使用。

可以使用以下命令列出可用的配置文件路径：

```console
$ bitsrun config-paths
bitsrun: list of possible config paths
┌──────────┬─────────────────────────────────────────────────────────────┐
│ 优先级   │ 可能的配置路径                                              │
├──────────┼─────────────────────────────────────────────────────────────┤
│ 1        │ /Users/spencerwoo/.config/bit-user.json                     │
│ 2        │ /Users/spencerwoo/.config/bitsrun/bit-user.json             │
│ 3        │ /Users/spencerwoo/Library/Preferences/bitsrun/bit-user.json │
│ 4        │ bit-user.json                                               │
└──────────┴─────────────────────────────────────────────────────────────┘
```

> [!NOTE]
> 配置文件位置取决于操作系统。运行该命令以检查系统上接受的位置。

**在 Linux 和 macOS 上将此文件的权限设置为 `600`，否则 `bitsrun` 将拒绝读取它。**

```console
$ chmod 600 <path/to/bit-user.json>
```

## 相关项目

- [`zu1k/srun`](https://github.com/zu1k/srun) - Srun 认证系统登录工具 (Rust)
- [`Mmx233/BitSrunLoginGo`](https://github.com/Mmx233/BitSrunLoginGo) - 深澜校园网登录脚本 Go 语言版 (Go)
- [`vouv/srun`](https://github.com/vouv/srun) - 针对 BIT 校园网的高效客户端 (Go)
- [`BITNP/bitsrun`](https://github.com/BITNP/bitsrun) - 针对 BIT 网关 (10.0.0.55) 的无头登录/注销脚本 (Python)

## 许可证

[MIT](./LICENSE)
