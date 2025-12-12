# Windows Service Setup

English | [简体中文](windows-service.zh-CN.md)

This document describes how to run `bitsrun` as a native Windows Service (SCM-managed), mirroring the behavior of the minimal branch.

## 概述

- Service name: `Bitsrun`
- Type: Own process managed by SCM
- Behavior: When launched by SCM, runs the `keep-alive` daemon in service context; reports status; handles stop/shutdown
- Logging:
  - Event Log: Source `Bitsrun`, under “Application”
  - Local file: `bitsrun_service.log` next to the executable
- Config file: Prefer `bit-user.json` from the executable directory

```json
{
  "username": "<username>",
  "password": "<password>",
  "dm": false,
  "poll_interval": 3600
}
```

Run as Administrator in PowerShell:

```powershell
sc.exe create Bitsrun binPath= "C:\Program Files\Bitsrun\bitsrun.exe" DisplayName= "Bitsrun" start= auto
sc.exe description Bitsrun "Bitsrun keep-alive service for BIT gateway"
```

Notes:
- `binPath` must be an absolute path to `bitsrun.exe`
- `start= auto` starts the service on boot

Manage:

```powershell
sc.exe start Bitsrun
sc.exe stop Bitsrun
sc.exe query Bitsrun
```

```powershell
sc.exe stop Bitsrun
sc.exe delete Bitsrun
```

Logs and troubleshooting:
- File: `bitsrun_service.log` beside `bitsrun.exe`
- Event Log: Event Viewer → Windows Logs → Application → Source `Bitsrun`
- The first install/start requires Administrator privileges to properly register the Event Log source

CLI vs Service:
- CLI mode: When started manually from a terminal, `bitsrun` works as usual
- Service mode: When launched by SCM, it runs the `keep-alive` daemon automatically

References and Compatibility:
- Reference changes: `PR_windows_service_minimal.md`, service entry `src/windows_service.rs:267`, main dispatcher `src/main.rs:53`
- Non-Windows platforms unaffected (`cfg(windows)` guarded)
