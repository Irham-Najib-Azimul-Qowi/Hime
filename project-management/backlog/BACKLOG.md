# Project Backlog
**Project:** Hime Assistant
**Last Updated:** 2026-11-06 (Post Final Governance Review)

---

## 1. Backlog Summary by Phase

| Phase | Phase Name | Status | Total Items | High Priority |
|:---:|:---|:---:|:---:|:---:|
| **PHASE 00** | Project Governance | **COMPLETED** | 2 | 2 |
| **PHASE 01** | Discovery & Environment Audit | **IN PROGRESS** | 5 | 5 |
| **PHASE 02** | Nusa Reverse Engineering | **RESEARCH COMPLETED (PENDING ACCEPTANCE)** | 1 | 1 |
| **PHASE 03** | Hermes & 9Router Integration Audit | **RESEARCH COMPLETED (PENDING ACCEPTANCE)** | 2 | 2 |
| **PHASE 04** | Product & System Specification | **PLANNED** | 3 | 3 |
| **PHASE 05** | Architecture & Security Design | **PLANNED** | 5 | 5 |
| **PHASE 06** | Engineering Foundation | **PLANNED** | 3 | 2 |
| **PHASE 07** | Hime Desktop | **PLANNED** | 5 | 3 |
| **PHASE 08** | AI Runtime Integration | **PLANNED** | 4 | 3 |
| **PHASE 09** | Hime Intelligence Core | **PLANNED** | 2 | 1 |
| **PHASE 10** | Storage Intelligence | **PLANNED** | 2 | 1 |
| **PHASE 11** | Hime Mobile & Device Layer | **PLANNED** | 4 | 1 |

---

## 2. Granular Backlog Items

### Phase 00: Project Governance *(COMPLETED)*
- [x] **TASK-00-01:** Initial Workspace Setup, Environment Audit, AGENTS.md, Governance, Skill Curation. *(COMPLETED)*
- [x] **TASK-00-02:** Workspace Git Initialization, Root `.gitignore`, Toolchain Verification, and Repository Protection. *(ACCEPTED)*

### Phase 01: Discovery & Environment Audit *(IN PROGRESS)*
- [x] **TASK-01-01:** Environment Baseline Validation — Verify runtime versions (Node.js, Python, uv, Git required; JDK, Android SDK future; Rust Phase 06–07; Docker optional) and workspace integrity against `ENVIRONMENT_AUDIT.md`. *(ACCEPTED)*
- [x] **TASK-01-02:** Existing Project & Research Asset Inventory — Catalog all RE outputs, documentation, and research artifacts for completeness and accuracy. *(ACCEPTED)*
- [-] **TASK-01-03:** Dependency and License Audit — Audit all external dependencies (Hermes, 9Router, agent skills) for license compatibility and version pinning. *(IN PROGRESS)*
- [ ] **TASK-01-04:** Security & Runtime Environment Audit — Audit local security posture: file permissions, credential exposure, network bindings, and process isolation.
- [ ] **TASK-01-05:** Discovery Consolidation & Phase Acceptance — Consolidate all Phase 01 findings into a formal Discovery Report and verify Phase Acceptance criteria.

### Phase 02: Nusa Reverse Engineering *(RESEARCH COMPLETED — PENDING ACCEPTANCE)*
- [x] **TASK-02-01:** Full reverse engineering of Nusa Dynamic Island (`nusetup.exe`) — 8 reports in `RE/Nusa/`.

### Phase 03: Hermes & 9Router Integration Audit *(RESEARCH COMPLETED — PENDING ACCEPTANCE)*
- [x] **TASK-03-01:** Hermes Agent reverse engineering — 5 reports in `RE/Hermes/reports/`.
- [x] **TASK-03-02:** 9Router reverse engineering — 14 reports in `RE/9ROUTER/`.

### Phase 04: Product & System Specification *(PLANNED)*
- [ ] **TASK-04-01:** Dynamic Island Product Requirements & User Experience Specification (Product requirements, user stories, interaction flows, notifications, voice/text triggers).
- [ ] **TASK-04-02:** System Behavior & State Morphology Specification (Dynamic Island functional states: Idle, Listening, Thinking Glow, Tool Execution HUD, Drawer expansion).
- [ ] **TASK-04-03:** High-Level System Contract & Non-Functional Requirements Specification (Performance: latency <50ms, memory <100MB, CPU idle <1%; functional boundary contracts; acceptance criteria).

### Phase 05: Architecture & Security Design *(PLANNED)*
- [ ] **TASK-05-01:** System Architecture Master Blueprint (Hime UI ↔ Hermes Agent ↔ 9Router Dual-Control Plane Topology).
- [ ] **TASK-05-02:** Component Architecture & Process Supervision Model (Tauri Rust backend, WebView frontend, Python daemon, Node.js gateway process boundaries).
- [ ] **TASK-05-03:** IPC & Communication Protocol Implementation Design (Win32 Named Pipe client/server, one-time session tickets, loopback SSE).
- [ ] **TASK-05-04:** Security Architecture & Threat Model (Windows DPAPI credential vault, 9Router credential isolation, loopback binding policy, process security).
- [ ] **TASK-05-05:** 9Router Health Monitor, Failover, & Fallback Notification Contract.

### Phase 06: Engineering Foundation
- [ ] **TASK-06-01:** Rust & Cargo Toolchain Installation on Windows Host (`rustup`, MSVC C++ Build Tools).
- [ ] **TASK-06-02:** Tauri 2.0 CLI & Prerequisites Verification.
- [ ] **TASK-06-03:** Local 9Router Service Bootstrap & CSPRNG Initial Password Generation via Child Process Environment.

### Phase 07: Hime Desktop
- [ ] **TASK-07-01:** Tauri 2.0 Project Scaffold with Win32 Transparent DirectComposition Window.
- [ ] **TASK-07-02:** Floating Island Window Management (`WS_EX_TOPMOST`, `WS_EX_TOOLWINDOW`, Multi-Monitor DPI awareness).
- [ ] **TASK-07-03:** Background Process Supervisor for Hermes Agent and 9Router in Rust.
- [ ] **TASK-07-04:** Local Named Pipe Broker Client in Rust.
- [ ] **TASK-07-05:** Dynamic Island UI & Interaction System (Physics-Based Animations, Real-Time Agent Visualizer).

### Phase 08: AI Runtime Integration
- [ ] **TASK-08-01:** Hermes Agent Virtual Environment Setup (`uv`) and Gateway Daemon Launch.
- [ ] **TASK-08-02:** End-to-End Inference Path: Hermes → 9Router loopback (`127.0.0.1:20128/v1`).
- [ ] **TASK-08-03:** Streaming Tool Execution & Thinking Token Relay to Tauri Frontend.
- [ ] **TASK-08-04:** Tool Execution Approval Modal (YOLO vs. Guardian Confirmation).

### Phase 11: Hime Mobile & Device Layer
- [ ] **TASK-11-01:** Android Project Scaffold & Foreground Service Architecture.
- [ ] **TASK-11-02:** Cross-Device Sync Protocol Design.
- [ ] **TASK-11-03:** Remote Execution Session with WakeLock.
- [ ] **TASK-11-04:** Mobile UI Implementation.
