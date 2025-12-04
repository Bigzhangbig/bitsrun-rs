# bitsrun

[![GitHub Workflow Status (CI)](https://img.shields.io/github/actions/workflow/status/spencerwooo/bitsrun-rs/ci.yml?logo=github&label=ci&labelColor=%23223227)](https://github.com/spencerwooo/bitsrun-rs/actions/workflows/ci.yml)
[![GitHub Workflow Status (Release)](https://img.shields.io/github/actions/workflow/status/spencerwooo/bitsrun-rs/release.yml?logo=github&label=release&labelColor=%23223227)](https://github.com/spencerwooo/bitsrun-rs/actions/workflows/release.yml)
[![GitHub release](https://img.shields.io/github/v/release/spencerwooo/bitsrun-rs?logo=github&labelColor=%23223227)](https://github.com/spencerwooo/bitsrun-rs/releases/latest)
[![Crates.io](https://img.shields.io/crates/d/bitsrun?logo=rust&labelColor=%23223227&color=%23dec867)](https://crates.io/crates/bitsrun)

🌐 北京理工大学校园网（10.0.0.55）无头登录登出命令行工具，使用 Rust 编写。

[English](README.md) | 简体中文

![CleanShot 2023-12-04 at 16 47 26@2x](https://github.com/spencerwooo/bitsrun-rs/assets/32114380/23343ba1-961c-41aa-b4b6-c09da93fb699)

## 安装

#### 一键安装（Linux / macOS，推荐）

- `curl -fsSL https://cdn.jsdelivr.net/gh/spencerwooo/bitsrun-rs@main/install.sh | sh -`

#### Ubuntu / Debian（推荐用于 `systemd` 支持）

- 从 [Releases](https://github.com/spencerwooo/bitsrun-rs/releases/latest) 下载最新的 `.deb` 安装包。
- `sudo apt install </path/to/file>.deb`

**如果需要 `bitsrun.service` systemd 服务：**

- 编辑 `/lib/systemd/system/bitsrun.service` 指定配置文件的绝对路径
- 然后使用 `sudo systemctl start bitsrun` 启动服务

#### Cargo

- `cargo install bitsrun`

#### 下载二进制文件

- 从 [Releases](https://github.com/spencerwooo/bitsrun-rs/releases/latest) 下载最新的二进制文件。
- 解压文件：`tar -xvf <file>.tar.gz`
- 将二进制文件移动到 `$PATH`，例如：`mv <file>/bitsrun ~/.local/bin/`

#### Windows 服务模式

Windows 用户可以将 bitsrun 作为 Windows 服务运行，以实现开机自动登录和保持在线。详细步骤请参见 [Windows 服务安装指南](#windows-服务安装)。

## 使用方法

登录或登出校园网：

```console
$ bitsrun login -u <用户名> -p <密码>
bitsrun: <ip> (<用户名>) logged in

$ bitsrun logout -u <用户名>
bitsrun: <ip> logged out
```

检查设备登录状态：

```console
$ bitsrun status
bitsrun: <ip> (<用户名>) is online
┌────────────────┬───────────────┬───────────────┬─────────┐
│ Traffic Used   │ Online Time   │ User Balance  │ Wallet  │
├────────────────┼───────────────┼───────────────┼─────────┤
│ 188.10 GiB     │ 2 months      │ 10.00         │ 0.00    │
└────────────────┴───────────────┴───────────────┴─────────┘
```

保持会话活跃，使用 `bitsrun keep-alive`：

```console
$ bitsrun keep-alive
 INFO  bitsrun::daemon > starting daemon (<用户名>) with polling interval=3600s
 INFO  bitsrun::daemon > <ip> (<用户名>): login success,
 ...
 ^C INFO  bitsrun::daemon > <用户名>: gracefully exiting
```

> [!NOTE]
> 使用可用的系统服务管理器将 `bitsrun keep-alive` 作为守护进程运行。（例如，Linux 使用 `systemd`，macOS 使用 `launchd`，Windows 使用 Windows 服务）。

## 可用命令

```console
$ bitsrun --help
A headless login and logout CLI for 10.0.0.55 at BIT

Usage: bitsrun [OPTIONS] [COMMAND]

Commands:
  login         登录校园网
  logout        登出校园网
  status        检查设备登录状态
  config-paths  列出所有可能的配置文件路径
  keep-alive    定期向服务器发送登录请求以保持会话活跃
  help          打印此消息或给定子命令的帮助信息

Options:
  -v, --verbose  详细输出
  -h, --help     打印帮助信息
  -V, --version  打印版本信息
```

> [!TIP]
> 使用环境变量 `NO_COLOR=true` 禁用彩色输出。

## 配置和凭据

要保存您的凭据和配置，请在可用的配置路径下创建配置文件 `bit-user.json`：

```json
{
  "username": "<用户名>",
  "password": "<密码>",
  "dm": true,
  "poll_interval": 3600
}
```

- **`dm` 用于指定当前设备是否为哑终端，需要通过备用端点登出。如果您使用的设备是哑终端，请设置为 `true`（不要加引号！）。**
- `poll_interval` 是一个可选字段，用于指定轮询登录请求的间隔（以秒为单位）。默认值为 `3600` 秒（1 小时）。仅由 `bitsrun keep-alive` 使用。

可以使用以下命令列出可用的配置文件路径：

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
> 配置文件位置取决于操作系统。运行该命令以检查您系统上接受的位置。

**在 Linux 和 macOS 上，将此文件的权限设置为 `600`，否则 `bitsrun` 将拒绝读取它。**

```console
$ chmod 600 <path/to/bit-user.json>
```

## Windows 服务安装

Windows 用户可以将 bitsrun 作为系统服务运行，以实现开机自动启动和后台保持在线。

### 方法一：原生 Windows 服务（推荐）

从 0.5.0 版本开始，bitsrun 支持使用 `windows-service` crate 实现的原生 Windows 服务。这是推荐的方法，因为它能更好地与 Windows 服务控制管理器 (SCM) 集成。

#### 前提条件

1. 从 [Releases](https://github.com/spencerwooo/bitsrun-rs/releases/latest) 下载 Windows 版本的 bitsrun 可执行文件
2. 将可执行文件放置在一个永久位置（例如 `C:\Program Files\bitsrun\bitsrun.exe`）
3. 创建配置文件 `bit-user.json` 并放置在合适的位置（例如 `C:\Program Files\bitsrun\bit-user.json`）

#### 安装步骤

1. 以管理员身份打开命令提示符或 PowerShell

2. 使用内置的 `sc` 命令安装服务：

```powershell
# 进入 bitsrun 目录
cd "C:\Program Files\bitsrun"

# 使用原生 Windows 服务模式安装服务
sc create bitsrun binPath= "C:\Program Files\bitsrun\bitsrun.exe windows-service" start= auto
sc description bitsrun "BIT Campus Network Auto Login Service"
```

3. 启动服务：

```powershell
sc start bitsrun
```

#### 服务管理

```powershell
# 查看服务状态
sc query bitsrun

# 停止服务
sc stop bitsrun

# 重启服务
sc stop bitsrun
sc start bitsrun

# 删除服务
sc delete bitsrun
```

> [!NOTE]
> 原生 Windows 服务模式使用 `windows-service` 命令，可直接与 Windows 服务控制管理器 (SCM) 集成。服务会自动从默认配置路径读取配置，或者您可以将 `bit-user.json` 放在可执行文件的同一目录中。

### 方法二：使用 NSSM（备选方案）

NSSM（Non-Sucking Service Manager）是一个简单易用的 Windows 服务管理工具，可作为备选方法使用。

#### 前提条件

与方法一相同。

#### 安装步骤

1. 下载 [NSSM](https://nssm.cc/download)
2. 以管理员身份打开命令提示符或 PowerShell
3. 运行以下命令安装服务：

```powershell
# 进入 NSSM 所在目录
cd C:\path\to\nssm\win64

# 安装服务（会打开 GUI 配置界面）
.\nssm.exe install bitsrun
```

4. 在 NSSM GUI 中配置：
   - **Path（路径）**: `C:\Program Files\bitsrun\bitsrun.exe`
   - **Startup directory（启动目录）**: `C:\Program Files\bitsrun`
   - **Arguments（参数）**: `keep-alive --config C:\Program Files\bitsrun\bit-user.json`
   - **Service name（服务名称）**: `bitsrun`

5. 点击 "Install service（安装服务）" 按钮

6. 启动服务：

```powershell
.\nssm.exe start bitsrun
```

#### 服务管理命令

```powershell
# 启动服务
nssm start bitsrun

# 停止服务
nssm stop bitsrun

# 删除服务
nssm remove bitsrun confirm
```

### 方法三：使用 sc 命令配合 keep-alive（旧方案）

也可以使用 Windows 内置的 `sc` 命令配合 `keep-alive` 子命令创建服务：

```powershell
# 以管理员身份运行
sc create bitsrun binPath= "C:\Program Files\bitsrun\bitsrun.exe keep-alive --config C:\Program Files\bitsrun\bit-user.json" start= auto
sc description bitsrun "BIT Campus Network Auto Login Service"
sc start bitsrun
```

> [!NOTE]
> 此方法使用 keep-alive 模式运行 bitsrun，但没有使用原生 Windows 服务集成。推荐使用方法一（原生 Windows 服务）以获得更好的 Windows SCM 集成。

### 附加说明

可以使用 Windows 服务管理器（`services.msc`）或命令行查看服务状态：

```powershell
sc query bitsrun
```

您还可以在 Windows 事件查看器的"Windows 日志" > "应用程序"中查看服务日志以进行故障排查。

> [!IMPORTANT]
> 确保配置文件 `bit-user.json` 中包含正确的用户名和密码，并且路径使用绝对路径。在 Windows 上不需要设置文件权限为 600。

## 项目结构

本项目包含以下主要源文件：

| 文件 | 用途 |
|------|---------|
| `src/main.rs` | 应用程序的入口点，处理命令行界面初始化并协调执行流程 |
| `src/cli.rs` | 使用 clap 定义命令行参数、子命令及其配置 |
| `src/client.rs` | 核心 SRUN 客户端实现，包含登录/登出逻辑和门户通信 |
| `src/config.rs` | 配置文件处理，包括路径枚举和验证 |
| `src/daemon.rs` | 守护进程模式实现，通过定期登录请求保持会话活跃 |
| `src/tables.rs` | 用于显示状态表和配置路径的美化打印工具 |
| `src/user.rs` | 从配置文件或交互式提示管理用户凭据 |
| `src/xencode.rs` | SRUN 门户认证的加密算法实现 |

## 相关项目

- [`zu1k/srun`](https://github.com/zu1k/srun) - Srun 认证系统登录工具（Rust）
- [`Mmx233/BitSrunLoginGo`](https://github.com/Mmx233/BitSrunLoginGo) - 深澜校园网登录脚本 Go 语言版（Go）
- [`vouv/srun`](https://github.com/vouv/srun) - BIT 校园网高效客户端（Go）
- [`BITNP/bitsrun`](https://github.com/BITNP/bitsrun) - 10.0.0.55 无头登录/登出脚本（Python）

## 许可证

[MIT](./LICENSE)
