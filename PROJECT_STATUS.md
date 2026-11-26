# PROJECT STATUS — HIME ECOSYSTEM (ADR-017 PHASE 4)

**Updated:** 2026-11-26  
**Architecture Baseline:** [ADR-017: Modular Two-Subsystem Native Rust Architecture](docs/adr/017-modular-two-subsystem-native-architecture.md)  
**Active Phase:** **Phase 4 — Cutover & WebView2 Archival (`DONE` — `[VERIFIED]`)**

---

## 1. CURRENT ARCHITECTURE SNAPSHOT

- **Product Identity:** **Hime Island** (Pure-Rust Desktop Shell) & **Hime** (Modular AI Runtime)
- **Active Subsystems:**
  1. **Subsystem 1 — `hime-island-native` (`island/native/`):** Pure-Rust Win32 + Direct2D + DirectWrite + DirectComposition shell (`hime-island-native.exe`, `1.21 MB` / `1,263,616` bytes release).
     - Zero WebView2 processes, zero Chromium/Bun runtime dependency.
     - Software Direct2D rendering (`D2D1_RENDER_TARGET_TYPE_SOFTWARE`), shared thread-local factories, cached brushes/text formats/bitmaps/regions, lazy Settings & AI Drawer surfaces, dirty-check rendering, and working-set trimming (`12.15 MB` Working Set / `10.08 MB` Private RAM in full non-AI Windows UI replacement mode with Dock, Notch, and Edge Overlays enabled).
     - Absorbs global hotkey handling (`island/native/src/hotkey.rs`, replacing legacy `hime_hotkey.exe`).
     - Supports Zero-AI shell-only mode by default (`hime-ai-enabled: "false"`), launching Subsystem 2 only when AI is enabled.
  2. **Subsystem 2 — `hime-core` (`core/`):** Rust workspace (`hime-core`, `hime-ipc`, `hime-modsdk`, `hime-pal`, `hime-security`, `hime-storage`) compiling `hime-core.exe` (`9.76 MB` / `10,231,808` bytes release).
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
| **Phase 3** | Native Rust Shell (`island/native/`) | `DONE` *(VERIFIED)* | Direct2D/DirectComposition Notch, Dock, Overlays, Settings, AI Drawer, embedded hotkeys, full `ipc_client.rs` & `main.rs` wiring, `<15 MB` RAM optimization (`97/97` tests pass) |
| **Phase 4** | Cutover & Archive WebView2 (`Archive/legacy-webview2-island/`, `scripts/`, docs) | `DONE` *(VERIFIED)* | Legacy WebView2 shell archived; scripts & quality gates migrated to 100% pure-Rust native-first with non-AI shell-only default (`342/342` total tests pass) |

---

## 3. PURE-RUST TOOLING & QUALITY GATES

- **Unified CLI (`scripts/hime.ps1`):** Dynamic `$repoRoot` resolution; supports `doctor`, `setup`, `dev`, `build`, `test`, `check`, `restore-taskbar`, and `clean` targeting `core/Cargo.toml` and `island/native/Cargo.toml` without Bun or Tauri.
- **Lifecycle & Watchdog Scripts:** `scripts/start-hime.ps1`, `scripts/hime-supervisor.ps1`, `scripts/stop-hime.ps1`, and `scripts/setup-shortcuts.ps1` updated to dynamic `$repoRoot`, pure-Rust binaries (`hime-island-native.exe`, plus optional `hime-core.exe` when `hime-ai-enabled` is `"true"` or `-WithAi`), and fast `HimeQuit` shutdown.
- **Automated Gates:**
  - `scripts/check-brand.ps1`: Audits `island/native/Cargo.toml`, `island/native/src`, `core/crates`, `modules`, `contracts`, `design`, `scripts`, `tools`.
  - `scripts/check-hygiene.ps1`: Audits unanchored `TODO`/`FIXME`, production `todo!()`/`unimplemented!()`, and stray artifacts across `core/crates` and `island/native/src`.
  - `scripts/verify-e2e.ps1`: 4-tier pure-Rust verification (Brand Gate, Hygiene Gate, `core/Cargo.toml` tests, `island/native/Cargo.toml` tests).
  - `scripts/verify-desktop-live.ps1`: Live desktop verification for `hime-island-native`, `hime-supervisor`, `hime.active`, `< 100 MB` non-AI Working Set RAM, top-center `HimeNotch` HWND, `< 2000ms` `HimeQuit` shutdown, and `Shell_TrayWnd` taskbar restoration (supporting `-NoAi` default and `-WithAi`).

---

## 4. VERIFICATION STATUS (`[VERIFIED]` — 2026-11-26)

- **Release Binaries:**
  - `island/native/target/release/hime-island-native.exe`: `1,263,616` bytes (`1.21 MB`)
  - `core/target/release/hime-core.exe`: `10,231,808` bytes (`9.76 MB`)
- **Automated Quality & E2E Suite (`scripts/verify-e2e.ps1 -Detailed` — ALL PASSED in `80.71s`):**
  - `scripts/check-brand.ps1`: PASSED (`0` violations)
  - `scripts/check-hygiene.ps1`: PASSED (`0` violations)
  - `core/Cargo.toml` (`cargo test --workspace`): **245 passed, 0 failed, 0 ignored**
  - `island/native/Cargo.toml` (`cargo test`): **97 passed, 0 failed, 0 ignored** (`91` lib + `6` binary)
  - **Total Automated Rust Tests:** **342 passed, 0 failed**
- **Live Desktop Validation & RAM Footprint (`[VERIFIED]`):**
  - **Non-AI Full Shell Mode (`verify-desktop-live.ps1 -Detailed -NoAi`):** **12/12 checkpoints PASSED** (`0` `hime-core` processes, `HimeNotch` top-center HWND `540x320`, `HimeQuit` shutdown `746ms < 2000ms`, `Shell_TrayWnd visible: True`); `start-hime.ps1 -NoAi` post-trim footprint with Dock + Notch + Overlays active: **`12.15 MB` Working Set / `10.08 MB` Private RAM**.
  - **AI-Enabled Mode (`verify-desktop-live.ps1 -Detailed -WithAi`):** **11/11 checkpoints PASSED** (`hime-core` + `hime-island-native` Working Set **`13.49 MB`**, `HimeQuit` shutdown `872ms < 2000ms`, `Shell_TrayWnd visible: True`).
