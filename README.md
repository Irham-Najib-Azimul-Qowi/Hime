# Hime — Pure-Rust Modular AI Desktop Ecosystem

> **Hime** is a Windows personal AI desktop ecosystem built around **ADR-017: Modular Two-Subsystem Native Rust Architecture**, pairing a pure-Rust Direct2D Dynamic Island shell (`hime-island-native.exe`) with a modular AI & intelligence runtime (`hime-core.exe`).

---

## 1. Architecture Overview (ADR-017 Phase 4)

Hime separates the always-on desktop interface from the AI runtime across two independent Rust subsystems communicating over a Win32 Named Pipe (`\\.\pipe\hime-ipc`):

| Subsystem | Location | Binary | Role |
| :--- | :--- | :--- | :--- |
| **Subsystem 1: Hime Island Native** | `island/native/` | `hime-island-native.exe` (`1.21 MB`) | Pure-Rust Win32 + Direct2D + DirectWrite + DirectComposition shell (zero WebView2). Renders 3-edge Dynamic Island Notch, Floating Dock, Dual Edge Overlays, Drag & Drop File Shelf, Native Settings, AI Drawer, Module Manager, and embedded global hotkeys. Runs standalone in non-AI mode at **`12.15 MB` Working Set / `10.08 MB` Private RAM**. |
| **Subsystem 2: Hime Core (`hime-ai`)** | `core/` | `hime-core.exe` (`9.76 MB`) | Optional modular Rust AI runtime (`hime-core`, `hime-ipc`, `hime-modsdk`, `hime-pal`, `hime-security`, `hime-storage`) launched when `hime-ai-enabled` is `"true"` (or `-WithAi`). Provides Policy Enforcement (PEP/PDP), DPAPI-encrypted vault, SQLite WAL + FTS5 memory, dynamic `SKILL.md` runtime, MCP Host Client, lyrics/tasks/routines/screen-time/proactive/dev/document engines, and adaptive AI adapters. |
| **External Gateway (Optional)** | External (`127.0.0.1:20128`) | `9router` | Optional external OpenAI-compatible router alongside direct Gemini native and Anthropic/OpenAI-compatible providers. |

> The legacy Tauri 2.0 / WebView2 / React + Bun shell (`island/src`, `island/src-tauri`, `hime_hotkey.exe`) has been archived under [`Archive/legacy-webview2-island/`](Archive/legacy-webview2-island/README.md).

---

## 2. Repository Structure

```text
Hime/
├── island/
│   ├── native/                  # Subsystem 1: Pure-Rust Direct2D + DirectComposition shell
│   └── scripts/                 # Taskbar recovery and desktop inspection utilities
│
├── core/
│   ├── Cargo.toml               # Subsystem 2: Rust virtual workspace manifest
│   └── crates/
│       ├── hime-core/           # AI adapters, orchestration, PDP/PEP, productivity, docs, dev tools
│       ├── hime-ipc/            # Binary-framed Named Pipe / UDS JSON-RPC transport
│       ├── hime-modsdk/         # Module & skill SDK contracts
│       ├── hime-pal/            # Platform Abstraction Layer
│       ├── hime-security/       # DPAPI / software vault, redaction, policy engine
│       └── hime-storage/        # SQLite WAL persistence, FTS5 memory, audit log
│
├── scripts/                     # Pure-Rust CLI, lifecycle, brand, hygiene & E2E gates
│   ├── hime.ps1                 # Unified CLI (doctor, setup, dev, build, test, check, clean)
│   ├── start-hime.ps1           # Launch hime-core + hime-island-native + supervisor
│   ├── hime-supervisor.ps1      # Background self-healing watchdog (HimeQuit aware)
│   ├── stop-hime.ps1            # Synchronized HimeQuit shutdown & taskbar restoration
│   ├── setup-shortcuts.ps1      # Desktop, Start Menu & Autostart shortcuts
│   ├── check-brand.ps1          # Brand compliance gate
│   ├── check-hygiene.ps1        # Code hygiene gate
│   ├── verify-e2e.ps1           # Automated 4-tier pure-Rust verification suite
│   └── verify-desktop-live.ps1  # Live Win32 HWND, process & HimeQuit validation
│
├── tools/                       # Silent VBS launcher & helper utilities
├── modules/                     # Declarative module manifests (eyes, keybinds)
├── contracts/                   # JSON schemas for module manifests
├── design/                      # Brand SVG marks, design tokens, eye geometry
├── mobile/                      # Android Kotlin Compose companion app
├── docs/                        # Architecture specs & ADRs (ADR-004 through ADR-017)
└── Archive/
    └── legacy-webview2-island/  # Archived WebView2 / Tauri / Bun shell (ADR-017 Phase 4)
```

---

## 3. Quick Start

### Prerequisites
- **OS:** Windows 10 / 11 (64-bit)
- **Toolchain:** Rust (`rustc` + `cargo`, `stable-x86_64-pc-windows-msvc` with Visual Studio C++ Build Tools), Git
- *(No Node.js, Bun, or WebView2 build tools are required for the active pure-Rust desktop shell.)*

### Verify Toolchain & Quality Gates
```powershell
# Check Rust toolchain prerequisites
.\scripts\hime.ps1 doctor

# Run brand compliance, code hygiene, and all Rust unit/integration tests
.\scripts\hime.ps1 check

# Or run the full 4-tier E2E verification suite
powershell -ExecutionPolicy Bypass -File scripts/verify-e2e.ps1
```

### Build & Run
```powershell
# Build optimized release binaries (core/target/release/hime-core.exe & island/native/target/release/hime-island-native.exe)
.\scripts\hime.ps1 build

# Start Hime Native Shell (pure shell-only default; pass -WithAi to include hime-core + optional 9Router)
powershell -ExecutionPolicy Bypass -File scripts/start-hime.ps1

# Validate live desktop processes, RAM (<100 MB non-AI), top-center Notch HWND, and HimeQuit shutdown
powershell -ExecutionPolicy Bypass -File scripts/verify-desktop-live.ps1 -Detailed

# Cleanly stop Hime and restore Windows taskbar
powershell -ExecutionPolicy Bypass -File scripts/stop-hime.ps1
```

---

## 4. Verified Baseline (ADR-017 Phase 4)

- **Automated Test Suite (`scripts/verify-e2e.ps1 -Detailed`):** **342 passed, 0 failed** (`245` `core` + `97` `island/native` [`91` lib + `6` binary]), `check-brand.ps1` & `check-hygiene.ps1` `0` violations.
- **Live Desktop Footprint (`scripts/verify-desktop-live.ps1 -Detailed`):** Non-AI full shell (`-NoAi`, Dock + Notch + Overlays): **`12.15 MB` Working Set / `10.08 MB` Private RAM** (`12/12` checks PASS); AI-enabled (`-WithAi`): `hime-island-native` **`13.49 MB` Working Set** (`11/11` checks PASS).

---

## 5. Documentation & ADRs

- [ADR-017: Modular Two-Subsystem Native Rust Architecture](docs/adr/017-modular-two-subsystem-native-architecture.md)
- [ADR-016: Dual Rust Workspace Architecture](docs/adr/016-dual-rust-workspace-architecture.md)
- [Documentation Index](docs/INDEX.md)
- [Project Status](PROJECT_STATUS.md)

---

## 6. License

Licensed under [LICENSE](LICENSE) and [NOTICE](NOTICE). See `Archive/legacy-webview2-island/` for archived WebView2 shell history.
