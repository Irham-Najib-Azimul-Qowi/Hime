# PROJECT STATUS — HIME ECOSYSTEM (ADR-017 PHASE 4)

**Updated:** 2026-11-26  
**Architecture Baseline:** [ADR-017: Modular Two-Subsystem Native Rust Architecture](docs/adr/017-modular-two-subsystem-native-architecture.md)  
**Active Phase:** **Phase 4 — Cutover & WebView2 Archival (`DONE` — `[VERIFIED]`)**

---

## 1. CURRENT ARCHITECTURE SNAPSHOT

- **Product Identity:** **Hime Island** (Pure-Rust Desktop Shell) & **Hime** (Modular AI Runtime)
- **Active Subsystems:**
  1. **Subsystem 1 — `hime-island-native` (`island/native/`):** Pure-Rust Win32 + Direct2D + DirectWrite + DirectComposition shell (`hime-island-native.exe`, `3.74 MB` release).
     - Zero WebView2 processes, zero Chromium/Bun runtime dependency.
     - Absorbs global hotkey handling (`island/native/src/hotkey.rs`, replacing legacy `hime_hotkey.exe`).
     - Includes 3-edge Dynamic Island Notch, Floating Dock, Dual Edge Overlays, File Shelf, Native Settings UI, AI Drawer, Module Manager, and live Win32 OS telemetry.
  2. **Subsystem 2 — `hime-core` (`core/`):** Rust workspace (`hime-core`, `hime-ipc`, `hime-modsdk`, `hime-pal`, `hime-security`, `hime-storage`) compiling `hime-core.exe` (`9.76 MB` release).
     - Connected via Win32 Named Pipe (`\\.\pipe\hime-ipc`).
     - Supports adaptive AI providers (OpenAI-compatible / optional external 9Router on `127.0.0.1:20128`, Google Gemini native, Anthropic Messages), FTS5 self-learning memory, `SKILL.md` dynamic skills, MCP host client, lyrics, tasks, routines, screen-time, proactive engine, developer checks, and document tools.
- **Legacy Archival (`Archive/legacy-webview2-island/`):**
  - Previous Tauri 2.0 / WebView2 / React / Vite / Bun shell files (`island/src`, `island/src-tauri`, `island/public`, HTML entrypoints, `package.json`, `bun.lock`, `tsconfig*.json`, `vite.config.ts`) archived via `git mv`.
  - `island/native/` and `island/scripts/` remain active in `island/`.

---

## 2. ADR-017 PHASE STATUS

| Phase | Scope | Status | Notes |
| :--- | :--- | :---: | :--- |
| **Phase 1** | Core AI Upgrade (`core/`) | `DONE` | Multi-provider adapters, MCP host, `SKILL.md` loader, FTS5 memory, IPC handlers |
| **Phase 2** | IPC & Module Installer Contract (`hime-ipc`, `hime-modsdk`, `module_mgr.rs`) | `DONE` | Versioned Named Pipe JSON-RPC contract & module management |
| **Phase 3** | Native Rust Shell (`island/native/`) | `DONE` *(VERIFIED)* | Direct2D/DirectComposition Notch, Dock, Overlays, Settings, AI Drawer, embedded hotkeys, full `ipc_client.rs` & `main.rs` wiring (`96/96` tests pass) |
| **Phase 4** | Cutover & Archive WebView2 (`Archive/legacy-webview2-island/`, `scripts/`, docs) | `DONE` *(VERIFIED)* | Legacy WebView2 shell archived; scripts & quality gates migrated to 100% pure-Rust native-first (`341/341` total tests pass, `11/11` live desktop checks pass) |

---

## 3. PURE-RUST TOOLING & QUALITY GATES

- **Unified CLI (`scripts/hime.ps1`):** Dynamic `$repoRoot` resolution; supports `doctor`, `setup`, `dev`, `build`, `test`, `check`, `restore-taskbar`, and `clean` targeting `core/Cargo.toml` and `island/native/Cargo.toml` without Bun or Tauri.
- **Lifecycle & Watchdog Scripts:** `scripts/start-hime.ps1`, `scripts/hime-supervisor.ps1`, `scripts/stop-hime.ps1`, and `scripts/setup-shortcuts.ps1` updated to dynamic `$repoRoot` and pure-Rust binaries (`hime-core.exe`, `hime-island-native.exe`).
- **Automated Gates:**
  - `scripts/check-brand.ps1`: Audits `island/native/Cargo.toml`, `island/native/src`, `core/crates`, `modules`, `contracts`, `design`, `scripts`, `tools`.
  - `scripts/check-hygiene.ps1`: Audits unanchored `TODO`/`FIXME`, production `todo!()`/`unimplemented!()`, and stray artifacts across `core/crates` and `island/native/src`.
  - `scripts/verify-e2e.ps1`: 4-tier pure-Rust verification (Brand Gate, Hygiene Gate, `core/Cargo.toml` tests, `island/native/Cargo.toml` tests).
  - `scripts/verify-desktop-live.ps1`: Live desktop verification for `hime-core`, `hime-island-native`, `hime-supervisor`, `hime.active`, top-center Notch HWND, `< 2000ms` `HimeQuit` shutdown, and taskbar restoration (without `hime_hotkey.exe`).

---

## 4. VERIFICATION STATUS (`[VERIFIED]` — 2026-11-26)

- **Release Binaries:**
  - `core/target/release/hime-core.exe`: `10,231,808` bytes (`9.76 MB`)
  - `island/native/target/release/hime-island-native.exe`: `3,918,109` bytes (`3.74 MB`)
- **Automated Quality & E2E Suite (`scripts/verify-e2e.ps1 -Detailed` — ALL PASSED in `116.57s`):**
  - `scripts/check-brand.ps1`: PASSED (`0` violations)
  - `scripts/check-hygiene.ps1`: PASSED (`0` violations)
  - `core/Cargo.toml` (`cargo test --workspace`): **245 passed, 0 failed, 0 ignored**
  - `island/native/Cargo.toml` (`cargo test`): **96 passed, 0 failed, 0 ignored** (`90` lib + `6` binary)
  - **Total Automated Rust Tests:** **341 passed, 0 failed**
- **Live Desktop Validation (`scripts/verify-desktop-live.ps1 -Detailed`):**
  - **11/11 checkpoints PASSED**: `hime-core`, `hime-island-native`, `hime-supervisor`, `hime.active`, visible top-center `HimeNotch` HWND (`540x320`), synchronized `HimeQuit` shutdown (`189ms < 2000ms`), `hime.active` cleanup, and `Shell_TrayWnd` taskbar restoration.
