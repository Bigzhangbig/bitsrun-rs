# Windows 服务设置

[English](windows-service.md) | 简体中文

本文档介绍如何在 Windows 上将 `bitsrun` 作为原生 Windows 服务运行。

## 概述

- 服务名称：`Bitsrun`
- 服务类型：独立进程（由 SCM 管理）
- 行为：由 SCM 启动时在服务上下文中运行 `keep-alive` 守护进程；停止/关机时能优雅退出并上报状态
- 日志：
  - 事件日志：`应用程序` 日志，来源为 `Bitsrun`
  - 本地日志文件：与可执行文件同目录，文件名 `bitsrun_service.log`
- 配置文件位置：优先从可执行文件所在目录寻找 `bit-user.json`

## 前置条件

- 已安装 `bitsrun.exe`，例如放在 `C:\Program Files\Bitsrun\bitsrun.exe`
- 在上述目录放置配置文件 `bit-user.json`（与可执行文件同目录），内容示例：

```json
{
  "username": "<username>",
  "password": "<password>",
  "dm": false,
  "poll_interval": 3600
}
```

## 安装服务

以管理员身份打开 PowerShell 或 CMD，执行：

```powershell
sc.exe create Bitsrun binPath= "C:\Program Files\Bitsrun\bitsrun.exe" DisplayName= "Bitsrun" start= auto
sc.exe description Bitsrun "Bitsrun keep-alive service for BIT gateway"
```

说明：
- `binPath` 指向 `bitsrun.exe` 的绝对路径
- `start= auto` 表示系统启动时自动启动服务
- 创建后可在“服务”管理器中看到 `Bitsrun`

## 启动与管理

```powershell
sc.exe start Bitsrun
sc.exe stop Bitsrun
sc.exe query Bitsrun
```

如需删除服务：

```powershell
sc.exe stop Bitsrun
sc.exe delete Bitsrun
```

## 日志与排错

- 文件日志：`bitsrun_service.log` 位于 `bitsrun.exe` 同目录，记录关键事件（启动、守护进程初始化、停止等）
- 事件日志：打开“事件查看器”→“Windows 日志”→“应用程序”，筛选来源为 `Bitsrun`
- 初次安装时，事件日志来源注册需要管理员权限；若事件日志无记录，请确认以管理员身份创建服务并启动一次

## 与 CLI 的关系

- 手动在终端运行 `bitsrun`（非 SCM 启动）时，程序按 CLI 方式工作（`login`/`logout`/`status`/`keep-alive` 等）
- 由 SCM 启动时，程序作为服务运行，内部自动进入守护模式，无需额外参数

## 参考与兼容性

- 参考改动：`PR_windows_service_minimal.md`，以及源码中的服务入口 `src/windows_service.rs:267` 与主入口调度 `src/main.rs:53`
- 非 Windows 平台不受影响；所有服务相关代码使用条件编译 `cfg(windows)`
