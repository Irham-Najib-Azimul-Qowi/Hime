# Modifications to Upstream Bloom (GPL-3.0 §5 Compliance)

**Base Project**: Bloom (`https://github.com/little-systems/bloom.git`)  
**Base Version**: `v3.10.2`  
**Base Commit**: `0ab9f2f08b75cfeffd3040330e312c2244d7726a` (2026-11-11)  
**License**: GNU General Public License v3.0 (`GPL-3.0`)  
**Derivative Project**: Hime Island (Desktop Shell for the Hime Personal AI Ecosystem)  
**Date of Modifications**: November 2026  

---

## 1. Summary of Modifications

In compliance with GNU GPL-3.0 Section 5(a), this document catalogues the prominent modifications made to Bloom to create the Hime Island desktop shell.

### 1.1 Monorepo Restructuring & Isolation
- The Bloom shell is hosted within the Hime monorepo under `island/`.
- Frontend source resides in `island/src/` with additive Hime extensions in `island/src/hime/`.
- Tauri Rust backend resides in `island/src-tauri/` with additive Hime extensions in `island/src-tauri/src/hime/`.
- Root management script `scripts/hime.ps1` orchestrates building and testing across the monorepo.

### 1.2 Brand Neutralization & Identity
- Application product name updated to **Hime Island** (`local.hime.island`) in `tauri.conf.json` and `package.json`.
- Neutralized user-facing dialog titles and file names (e.g., "Export Hime Island Settings", "Import Hime Island Settings", `hime-settings.json`).
- Updated internal IPC bridge client identity to `hime-shell-bridge`.
- Purged unreferenced legacy Bloom graphics from `island/public/` (reducing binary payload by ~6.6MB).
- Replaced mascot and application iconography with the Hime two-eyes mark derived from `design/brand/hime-mark.svg`.

### 1.3 Upstream Network Neutralization (ADR-015)
- Neutralized upstream auto-updater checks (`tauri-plugin-updater`) to prevent unauthorized network calls or overwrites from upstream Bloom binaries.
- Disabled remote announcement / release popup dialogs.

### 1.4 Hime Core Inter-Process Communication Bridge (ADR-013, ADR-014)
- Added `island/src-tauri/src/hime/` module providing a Win32 Local Named Pipe (`\\.\pipe\hime-ipc`) bridge.
- Decoupled from the Hime Core AI kernel (`hime-core.exe`): Hime Core runs as an independent process. If Core is unavailable, the Island shell remains 100% operational in a standalone state.
- Added session token handshake, periodic ping latency measurement, and event streaming.

### 1.5 Synchronized Clean Shutdown Coordination
- Integrated Win32 Named Event `HimeQuit` across tray menu, global shortcut (`Ctrl + Alt + Q`), in-app quit commands, and PowerShell supervisor.
- Added persistence discrimination logic in taskbar restoration: checks `GetSystemMetrics(SM_SHUTTINGDOWN)` to preserve `%APPDATA%\ai.hime.desktop\hime.active` across system reboot/shutdown while removing it on explicit user termination.

### 1.6 User Interface & Settings Enhancements
- Enhanced `island/src/settings/AboutTab.tsx` with dedicated "License & Open Source Notices" disclosing GPL-3.0 licensing, upstream attribution to Sehajveer Singh and contributors, key open source components, and pointers to legal documentation.
- Integrated additive Hime module slots, widgets, and bubble notifications in `island/src/hime/`.

---

## 2. Integrity and Access to Source Code

Under the terms of GPL-3.0:
1. The complete machine-readable source code of Hime Island (including all modifications and additive shell components) is provided alongside distributed binaries.
2. The legal notices in `LICENSE` and `NOTICE` are preserved and prominently displayed in the application UI.
