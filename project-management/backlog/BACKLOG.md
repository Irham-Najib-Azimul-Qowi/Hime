# Project Backlog
**Project:** Hime Assistant  
**Last Updated:** 2026-11-06  

---

## 1. Backlog Summary by Phase

| Phase | Phase Name | Status | Total Items | High Priority |
|:---:|:---|:---:|:---:|:---:|
| **PHASE 0** | Workspace Infrastructure & Governance | **IN PROGRESS** | 2 | 2 |
| **PHASE 1** | Reverse Engineering & Research | **COMPLETED** | 4 | 4 |
| **PHASE 2** | System Architecture & Technical Specifications | **READY** | 4 | 4 |
| **PHASE 3** | Environment & Dependency Bootstrap | **PLANNED** | 3 | 2 |
| **PHASE 4** | Hime Desktop Core Shell (Tauri 2.0 Rust) | **PLANNED** | 5 | 3 |
| **PHASE 5** | Hermes Agent & 9Router Service Integration | **PLANNED** | 4 | 3 |
| **PHASE 6** | Dynamic Island UI & Interaction System | **PLANNED** | 6 | 4 |
| **PHASE 7** | Hime Mobile (Android) & Cross-Device Sync | **PLANNED** | 4 | 1 |

---

## 2. Granular Backlog Items

### Phase 0: Workspace Infrastructure & Governance
- [x] **TASK-00-01:** Initial Workspace Setup, Environment Audit, AGENTS.md, Governance, Skill Curation. *(Current)*
- [ ] **TASK-00-02:** Workspace Git Initialization, Root `.gitignore`, and Pre-Commit Hygiene.

### Phase 2: System Architecture & Technical Specifications
- [ ] **TASK-02-01:** System Architecture Master Blueprint (Hime UI ↔ Hermes Agent ↔ 9Router Dual-Control Plane).
- [ ] **TASK-02-02:** IPC & Security Protocol Specification (Named Pipe, One-Time Session Tickets, DPAPI Credential Storage).
- [ ] **TASK-02-03:** Dynamic Island Morphology & State Machine Specification (Compact, Thinking, Tool HUD, Drawer).
- [ ] **TASK-02-04:** 9Router Health Monitor & Fallback Notification Contract.

### Phase 3: Environment & Dependency Bootstrap
- [ ] **TASK-03-01:** Rust & Cargo Toolchain Installation on Windows Host (`rustup`, MSVC C++ Build Tools).
- [ ] **TASK-03-02:** Tauri 2.0 CLI & Prerequisites Verification.
- [ ] **TASK-03-03:** Local 9Router Service Bootstrap & Initial Password Generation Script.

### Phase 4: Hime Desktop Core Shell (Tauri 2.0 Rust)
- [ ] **TASK-04-01:** Tauri 2.0 Project Scaffold with Win32 Transparent DirectComposition Window.
- [ ] **TASK-04-02:** Floating Island Window Management (`WS_EX_TOPMOST`, `WS_EX_TOOLWINDOW`, Multi-Monitor DPI awareness).
- [ ] **TASK-04-03:** Background Process Supervisor for Hermes Agent and 9Router in Rust.
- [ ] **TASK-04-04:** Local Named Pipe Broker Client in Rust.

### Phase 5: Hermes Agent & 9Router Integration
- [ ] **TASK-05-01:** Hermes Agent Virtual Environment Setup (`uv`) and Gateway Daemon Launch.
- [ ] **TASK-05-02:** End-to-End Inference Path: Hermes → 9Router loopback (`127.0.0.1:20128/v1`).
- [ ] **TASK-05-03:** Streaming Tool Execution & Thinking Token Relay to Tauri Frontend.

### Phase 6: Dynamic Island UI & Interaction System
- [ ] **TASK-06-01:** Physics-Based Dynamic Island Animations (using `animate` and `emil-design-eng` standards).
- [ ] **TASK-06-02:** Real-Time Agent Activity Visualizer (Thinking Glow, Tool Call Progress, Token Counter).
- [ ] **TASK-06-03:** Tool Execution Approval Modal (YOLO vs. Guardian Confirmation).

