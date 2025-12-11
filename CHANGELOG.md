# Changelog

All notable changes to this project will be documented in this file.

## 0.5.1 — 2025-12-12

- Added: Native Windows service support integrated with SCM (#4)
- Added: Daemon mode (keep-alive) for persistent login
- Changed: Release workflow improved to handle forks safely; uploads gated to upstream only
- Changed: Debian package now installs systemd unit files; adjusted restart parameters
- Changed: Refactored duplicated code patterns; updated dependencies
- Fixed: Disabled ANSI colors on unsupported Windows terminals; added ANSI support gating
- Fixed: Corrected balance-related field types
- CI: Avoid writing cache on forks; upgraded `actions/checkout` to v4
- Docs: Added Chinese README and Windows service installation guidance; expanded documentation and screenshots

