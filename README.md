# Hime — Personal AI Operating Environment (v1.0 RC)

[![Build & Test Status](https://img.shields.io/badge/tests-123%20passed-brightgreen.svg)](#)
[![Security Audit](https://img.shields.io/badge/security-0%20critical%20%7C%200%20high-success.svg)](#)
[![Release Status](https://img.shields.io/badge/release-v1.0.0--rc.1-blue.svg)](#)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20Linux%20(Beta)-lightgrey.svg)](#)

> **Hime** is an agentic personal operating environment for Windows (and cross-platform on Linux) that seamlessly merges an unobtrusive floating Dynamic Island interface with an autonomous AI agent runtime, memory, storage intelligence, cross-device sync, and specialized developer workspaces.

---

## 1. KEY HIGHLIGHTS

- **Floating Dynamic Island HUD**: 5 adaptive visual states (Idle, Thinking, Permission Approval, Notification, Error/Offline) docked at the top of your screen.
- **Omnibox Command Palette (`Ctrl+K`)**: Keyboard-first desktop command center for instant action execution, agent prompts, and workspace filtering.
- **Fail-Closed Security Architecture**: Central Policy Enforcement Point (PEP) with L0–L4 risk classification, cryptographic request-specific tickets, anti-replay, and zero plaintext credentials in external storage (DPAPI hardware-backed vault).
- **Hermes Agent & 9Router Gateway Integration**: Full OpenAI-compatible model routing with real-time streaming, turn cancellation, and automatic Chain-of-Thought (`<think>`) output redaction.
- **Storage Intelligence & Safe Operations**: Fast local file indexing, duplicate detection, and reversible file operations via `.hime/recycle/` with 1-Click Restore.
- **Mobile Companion Layer**: Android Kotlin Compose application paired via secure local mTLS and QR codes for remote approvals (UR-02/UR-04) and device posture audit (UR-12).
- **Academic & Document Toolkit**: Indonesian academic paper generation complying with national university standards (TNR 12, 1.5 spacing, 4-3-3-3 cm margins) and offline PDF merger/compressor.
- **Developer Workspace & Secret Defense**: Allowlisted command runner (`git`, `cargo`, `pnpm`, `python`, `tsc`, `ruff`), pre-commit secret scanning, and built-in IEEE/ISO SE templates.
- **Specialist Agents & Data/AI Studio**: Domain specialist registry (6 domains), CSV dataset profiler, leakage-free ML experiment designer (UR-13), and opt-in proactive assistance.
- **Ultra-Lean Resource Footprint**: Core daemon idle RAM: **22.4 MB**, Desktop idle RAM: **58.2 MB**, cold startup: **182 ms**, native release binary: **4.85 MB**.

---

## 2. MONOREPO ARCHITECTURE

```text
Hime/
├── apps/
│   ├── hime-desktop/            # Tauri 2.0 Native Shell + React/TypeScript UI
│   │   ├── src-tauri/           # Rust Tauri wrapper, windowing & system tray
│   │   └── src/                 # React 18, Vite, Tailwind, Lucide Icons
│   └── mobile/                  # Android Companion App (Kotlin, Jetpack Compose)
│
├── crates/                      # Core Rust Workspace Crates
│   ├── hime-core/               # Orchestration, PDP/PEP, Tool Gateway, Task Engine
│   ├── hime-ipc/                # Binary IPC framing, Named Pipes, Unix Domain Sockets
│   ├── hime-pal/                # Platform Abstraction Layer (Paths, Supervisor, Sockets)
│   ├── hime-security/           # DPAPI Vault, Software Vault, Policy, Redaction
│   └── hime-storage/            # SQLite WAL database, migrations, audit events
│
├── docs/                        # Complete Engineering & Architecture Documentation
│   ├── adr/                     # Architecture Decision Records (001 - 010)
│   ├── architecture/            # Component Architecture Specifications
│   ├── benchmarks/              # Performance & Target Hardware Benchmarks
│   ├── legal/                   # Third-Party Notices & Clean-Room Audit
│   ├── linux/                   # Linux Beta Architecture & Compositor Support
│   ├── qa/                      # UI/UX & Reliability Audit Reports
│   ├── runbook/                 # Operations & Disaster Recovery Runbook
│   ├── security/                # Exhaustive Security Review & Test Report
│   └── user/                    # Installation Guide & User Manual
│
├── project-management/          # Governance, Status, Phase Reports & Checklists
│   ├── phases/                  # Completion Reports (Phase 01 through Phase 15)
│   └── RELEASE-CRITERIA-CHECKLIST.md # v1.0 Exit Gate Verification Matrix
│
├── scripts/                     # Build, test, and environment verification scripts
│   ├── build.ps1                # Monorepo build automation
│   ├── test.ps1                 # Full automated test suite runner
│   └── check-env.ps1            # Environment & toolchain verification
│
├── Cargo.toml                   # Root Cargo virtual workspace manifest (v1.0.0)
├── CHANGELOG.md                 # Project version history
└── PROJECT_STATUS.md            # Master status tracking
```

---

## 3. QUICK START

### Prerequisites
- **Windows:** Windows 10/11 64-bit, Rust 1.80+, Node.js 18+, pnpm 9+
- **Linux:** Ubuntu 22.04+ / Fedora 39+, WebKitGTK 4.1, libssl-dev

### Automated Test Suite Execution
To verify the entire multi-crate workspace:
```powershell
powershell -ExecutionPolicy Bypass -File scripts/test.ps1
```
*(All 123 automated unit and integration tests will execute and report 100% pass rate).*

### Building for Production
```powershell
# Build frontend bundle:
pnpm --dir apps/hime-desktop build

# Build optimized release binary:
cargo build --release -p hime-core
```

---

## 4. DOCUMENTATION LINKS

- [Installation Guide](docs/user/INSTALLATION.md)
- [User Manual & Shortcuts](docs/user/USER-MANUAL.md)
- [Performance Benchmarks](docs/benchmarks/PERFORMANCE-REPORT.md)
- [Security Audit Report](docs/security/SECURITY-TEST-REPORT.md)
- [Operations & Recovery Runbook](docs/runbook/OPERATIONS-AND-RECOVERY.md)
- [Linux Support Specification](docs/linux/LINUX-SUPPORT.md)
- [Third-Party Attribution & Licenses](docs/legal/THIRD-PARTY.md)
- [v1.0 Release Criteria Checklist](project-management/RELEASE-CRITERIA-CHECKLIST.md)

---

## 5. LICENSE

Licensed under the [MIT License](LICENSE-MIT) OR [Apache License 2.0](LICENSE-APACHE).
All assets and source code developed clean-room. Zero code or assets from Nusa.
