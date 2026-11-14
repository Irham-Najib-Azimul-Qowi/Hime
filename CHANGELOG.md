# Changelog

All notable changes to the Hime project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [1.0.0-rc.1] - 2026-11-14

### Added
- **Core Orchestration & Policy Enforcement (`hime-core`)**:
  - Central Policy Enforcement Point (PEP) and Policy Decision Point (PDP) governing all actions.
  - Policy risk classification (L0: Read-Only, L1: Safe Create, L2: Reversible Modify, L3: External Network/Commit with Approval, L4: Destructive Blocked).
  - Single-use, time-limited cryptographic approval tickets with anti-replay protection.
  - Multi-step Task Engine with state machine transitions, crash recovery, and rolling in-memory event buffer.
  - Dynamic Memory Context Engine with strict token budgeting and keyword search.
  - Real-time text translation pipeline with Island HUD pill display and privacy opt-in gates.
- **AI Runtime & Model Routing**:
  - Integration with Hermes Agent (NousResearch) and 9Router Gateway (decolua).
  - Hime Loopback Provider Adapter (`127.0.0.1:20129`) decrypting credentials on-the-fly from DPAPI, fully resolving upstream blocker **IB-01**.
  - Chain-of-Thought (`<think>`) and API key output redaction across UI, logs, and external transports (SEC-09).
- **Storage Intelligence & Safe Operations (`hime-storage`)**:
  - Embedded SQLite database with Write-Ahead Logging (WAL) and hash-chained audit events for tamper detection.
  - Background local file indexing, duplicate and redundant file analysis.
  - Reversible file operations via `.hime/recycle/` with 1-Click Restore and audit journaling.
- **Mobile Companion Layer (`apps/mobile/`, Android Kotlin Compose)**:
  - Mutual TLS (mTLS) with embedded self-signed local Certificate Authority (CA).
  - Time-limited QR code pairing with single-use session tokens.
  - Remote permission approvals (UR-02/UR-04) and device posture audit (battery, charging, security lock UR-12).
  - Resumable chunked file transfers with SHA-256 integrity verification.
- **Productivity & Documents Engine**:
  - Interactive Kanban task board with study schedule and strict/balanced/santai accountability modes.
  - Indonesian Academic Report Generator complying with national standards (Times New Roman 12pt, 1.5 line spacing, 4-3-3-3 cm margins).
  - Offline PDF toolkit: merge, split, and compress multi-page PDF documents.
  - Artcraft local UI/UX component discovery indexer.
- **Browser & Developer Workspace**:
  - Web research browser with strict Server-Side Request Forgery (SSRF) defense, blocking loopback, private subnets, and cloud metadata (`169.254.169.254`).
  - Anti-prompt injection defense encapsulating DOM text into `<untrusted_web_content>` tags.
  - Quarantined download manager with SHA-256 hashing.
  - Allowlisted command runner (`git`, `cargo`, `pnpm`, `python`, `tsc`, `ruff`) rejecting raw shell strings.
  - Pre-commit secret scanner detecting AWS keys, GitHub PATs, and private keys.
  - Built-in IEEE/ISO software engineering templates (SRS, SAD, Test Plan, Code Review).
- **Specialist Agents & Data/AI Studio**:
  - Specialist agent registry managing 6 domain experts (Software Engineer, Data & AI, Research, Document, Productivity, Storage).
  - Zero-dependency CSV dataset profiler and leakage-free ML experiment designer (UR-13).
  - Safe, opt-in proactive assistance with cooldown rate limiting and quick mute controls.
  - Automated specialist evaluation benchmark harness.
- **Cross-Platform Linux PAL (`hime-pal`, `hime-ipc`, `hime-security`)**:
  - XDG Base Directory specification compliance (`~/.config/hime`, `~/.local/share/hime`, `~/.cache/hime`, `$XDG_RUNTIME_DIR/hime`).
  - Unix Domain Socket transport with strict `0600` permissions.
  - Process group supervisor with coordinated signal propagation.
  - Cross-platform `SoftwareEncryptedVault` using SHA-256 CTR keystream with automatic memory zeroization.
- **Desktop User Interface (`apps/hime-desktop`, Tauri 2.0)**:
  - Floating Dynamic Island HUD with 5 adaptive visual states.
  - Full-keyboard Omnibox Command Palette (`Ctrl+K`).
  - 9 integrated workstation views with `Alt+1` through `Alt+9` quick navigation.
  - High-contrast dark mode (14.8:1), DPI scaling (100%–200%), and `prefers-reduced-motion` compliance.
- **Packaging & Hardening**:
  - Release build producing a 4.85 MB native `hime-core.exe` binary.
  - Tauri NSIS and MSI per-user installer bundling without administrative elevation.
  - Comprehensive operational runbooks, user manuals, and security test reports.

---

## [0.1.0] - 2026-11-06

### Added
- Initial project scaffolding and reverse engineering audit phase.
- Core crate architecture foundations (`hime-core`, `hime-ipc`, `hime-pal`, `hime-storage`, `hime-security`).
- Initial Tauri desktop application shell.
