# Project Backlog — Hime × Bloom Rebuild

**Project:** Hime Assistant (Bloom Shell Rebuild)  
**Last Updated:** 2026-11-16 (Stage H1 Execution)  
**Host Framework:** Bloom v3.10.2 (Tauri v2 + Rust + React 19 / TypeScript)  
**Backend:** Hime Core (Standalone Rust Workspace `core/`)

---

## 1. Roadmap & Stage Status Overview

| Stage / Phase | Description | Status | Priority | Target Deliverables |
|:---:|:---|:---:|:---:|:---|
| **Stage H0** | Bloom Study & Host Import, Safety Harness, Neutralize Updaters | **`COMPLETED`** | P0 | Baseline metrics, safety scripts, ADR 013-015 |
| **Stage H1** | Legacy Harvest & Repo Restructure, Core Standalone Monorepo | **`IN PROGRESS`** | P0 | `HARVEST-MANIFEST.md`, `core/` workspace (131 tests), `OLD-UI-LESSONS.md` |
| **Stage H2** | Bridge IPC & Dynamic Island Vertical Slice | **`NEXT`** | P0 | Win32 Named Pipe bridge, notch status dot, honest offline state, VC-01 |
| **Phase 08R**| AI Runtime Re-Integration (Hermes + 9Router + DPAPI Vault) | `PENDING` | P0 | Live activity bubbles in notch, multi-turn chat panel, IB-01/IB-02 gates |
| **Phase 09R**| Intelligence Core, Tool Gateway & Island Translation | `PENDING` | P1 | Ctrl+Shift+T selection translation, L0–L4 tool policies, task supervisor |
| **Phase 10R**| Storage Intelligence & Safe File Operations | `PENDING` | P1 | Shelf integration, safe staging, recycle bin rollback |
| **Phase 11R**| Mobile & Cross-Device Layer | `PENDING` | P1 | Android Companion app, mTLS pairing, remote ticket approvals |
| **Phase 12R**| Productivity & Documents | `PENDING` | P2 | Study schedule, Kanban, document merger & academic reports |
| **Phase 13R**| Browser & Developer Workspace | `PENDING` | P2 | On-demand `hime-workspace` window, Git ops, safe command runner |
| **Phase 14R**| Specialist Agents & Multi-Model Routing | `PENDING` | P2 | Specialist agent registry, dataset profiler, proactive nudges |
| **Phase 15R**| Hardening, Benchmarking & v1.0 Release Candidate | `PENDING` | P0 | Final handover report, packaging installer, zero regression verification |

---

## 2. Granular Task Tracking

### Stage H0: Bloom Study & Host Import *(COMPLETED)*
- [x] **TASK-H0.1:** Bun toolchain installation & PATH verification.
- [x] **TASK-H0.2:** Bloom clone & pin to release tag `v3.10.2`.
- [x] **TASK-H0.3:** License audit (GPL-3.0 compliance & IPC boundary verification).
- [x] **TASK-H0.4:** Taskbar safety harness (`restore-taskbar.ps1`, `run-safe.ps1`).
- [x] **TASK-H0.5:** Neutralize upstream updater & announcements (ADR-015, TP-001..TP-003).
- [x] **TASK-H0.6:** Pristine host build & Rust quality gates.
- [x] **TASK-H0.7:** Source study & conventions documentation.
- [x] **TASK-H0.8:** Baseline metrics & visual acceptance protocol.
- [x] **TASK-H0.9:** Touchpoint candidate mapping.
- [x] **TASK-H0.10:** ADR documentation (ADR-013, 014, 015).

### Stage H1: Legacy Harvest & Restructure *(CURRENT)*
- [x] **TASK-H1.1:** Legacy repository inventory (`OLD-INVENTORY.md`).
- [x] **TASK-H1.2:** Harvest & migration manifest (`HARVEST-MANIFEST.md`).
- [x] **TASK-H1.3:** Copy & verify KEEP/ADAPT files with SHA-256 hash matching (97/97 crate files match).
- [x] **TASK-H1.4:** Standalone Core workspace build & test verification (131 tests passed, bridge client integration test added).
- [x] **TASK-H1.5:** Old UI architectural lesson extraction (`OLD-UI-LESSONS.md` & `types-draft.md`).
- [x] **TASK-H1.6:** Document reorganization (`PROJECT_STATUS.md`, `BACKLOG.md`, `RISK_REGISTER.md`, `AGENTS.md`).
- [x] **TASK-H1.7:** Utility scripts (`check-env.ps1`, `build-core.ps1`, `dev-hime.ps1`).
- [x] **TASK-H1.8:** Local CI workflow job (`ci.yml` hime-core job).
- [ ] **TASK-H1.9:** Deletion of failed UI and obsolete build caches in legacy repository per checklist 5.1.

### Stage H2: Bridge & Island Vertical Slice *(UPCOMING)*
- [ ] **TASK-H2.1:** Path dependency / client setup for `hime-ipc` in `src-tauri`.
- [ ] **TASK-H2.2:** Core background service & reconnect supervisor in `src-tauri/src/hime/services.rs`.
- [ ] **TASK-H2.3:** Tauri commands registration (`hime_core_status`, `hime_ping`, `hime_handshake`).
- [ ] **TASK-H2.4:** Frontend API and event hooks in `src/hime/api/` and `src/hime/events/`.
- [ ] **TASK-H2.5:** Island morphing slice: Status dot and minimal indicator on Bloom notch.
- [ ] **TASK-H2.6:** Honest offline state handling when Core is stopped.
- [ ] **TASK-H2.7:** Visual verification & Owner acceptance milestone VC-01.
