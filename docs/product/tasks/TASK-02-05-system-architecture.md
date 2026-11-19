# Task Specification: TASK-02-05 — Hime System Architecture & Technical Design

- **Task ID:** TASK-02-05
- **Phase:** Phase 02 — Nusa Reverse Engineering & Architecture Discovery (System Architecture Blueprint)
- **Title:** Hime System Architecture & Technical Design
- **Priority:** CRITICAL
- **Owner:** Principal Software Architect, Staff Systems Engineer & Senior Technical Architect
- **Reviewer:** Project Owner / User
- **Status:** **IN PROGRESS**
- **Dependencies:**
  - TASK-01-01 s/d TASK-01-05 — Environment, Asset, Dependency, Security Audit & Consolidation (Phase 01 Acceptance)
  - TASK-02-01 s/d TASK-02-03B — Nusa Discovery, Binary, Subsystems, UX & Interaction Reverse Engineering (Accepted)
  - TASK-02-04 / 04B / 04C / 04D — Hime Product Requirements & Experience Architecture (Complete — Pending Owner Review)
- **Commit Reference:** Pending Task Acceptance Approval by Project Owner (Zero Commit / Zero Push Discipline)

---

## 1. Description & Context

Merancang cetak biru arsitektur sistem (*System Architecture Blueprint*) yang menyeluruh, terukur, konkret, dan dapat diimplementasikan secara langsung untuk ekosistem **Hime**.

Hime dirancang sebagai personal AI desktop environment mandiri (*Windows-first, cross-platform by design*) yang beroperasi melalui 4 proses independen:
1. **Hime Desktop:** Shell desktop presentation native (Tauri 2.0, React 19, TypeScript, Tailwind CSS, Motion) dengan Kanvas Elastis 3-Tier Dynamic Island, tembus-klik `WM_NCHITTEST`, dan keyboard shortcut `Alt + Space`.
2. **Hime Core:** Pusat orkestrasi sistem berbasis modular monolith Rust, bertindak sebagai *Policy Decision Point (PDP)* dan *Policy Enforcement Point (PEP)*, pengawas proses (*process supervisor*), pengelola state bus, dan pengelola basis data lokal SQLite FTS5.
3. **Hermes Agent:** Runtime penalaran otonom eksternal berbasis Python (Nous Research), di mana seluruh pemanggilan alat (*tool calls*) dan tindakan sistem OS dialihkan via saluran IPC ke Hime Core.
4. **9Router:** Gateway inferensi multi-provider eksternal berbasis Node.js (decolua) yang beroperasi sebagai managed background service pada loopback `127.0.0.1:20128`.

---

## 2. Acceptance Criteria & Verification Matrix

- [ ] **AC-01 (System Context & C4 Container Architecture):** Dokumen [`HIME-C4-ARCHITECTURE.md`](file:///x:/folder_ai/Hime/project-management/system-architecture/HIME-C4-ARCHITECTURE.md) mendefinisikan Context, Container, Component, dan Deployment Diagram dengan batas kepercayaan (*trust boundaries*), aliran data, dan kegagalan.
- [ ] **AC-02 (Hime Core Internal Architecture):** Dokumen [`HIME-CORE-INTERNAL-ARCHITECTURE.md`](file:///x:/folder_ai/Hime/project-management/system-architecture/HIME-CORE-INTERNAL-ARCHITECTURE.md) merumuskan arsitektur modular monolith 14 modul domain dengan trait/interface, input/output, dependencies, error models, dan boundaries.
- [ ] **AC-03 (IPC & Communication Architecture):** Dokumen [`HIME-IPC-AND-COMMUNICATION-ARCHITECTURE.md`](file:///x:/folder_ai/Hime/project-management/system-architecture/HIME-IPC-AND-COMMUNICATION-ARCHITECTURE.md) merancang Windows Named Pipe (DACL + PID verify), Linux Unix Domain Socket (mode `0600` + `SO_PEERCRED`), trait `IpcTransport`, framing custom, JSON-RPC 2.0 schemas, timeout 30s, token 256-bit CSPRNG, dan replay protection.
- [ ] **AC-04 (Agent Orchestration Architecture):** Dokumen [`HIME-AGENT-ORCHESTRATION-ARCHITECTURE.md`](file:///x:/folder_ai/Hime/project-management/system-architecture/HIME-AGENT-ORCHESTRATION-ARCHITECTURE.md) merinci alur reasoning loop, siklus tugas (*task lifecycle*), Activity Stream transparan, larangan private CoT, dan single Hermes process runtime MVP.
- [ ] **AC-05 (Tool & Service Architecture):** Dokumen [`HIME-TOOL-AND-SERVICE-ARCHITECTURE.md`](file:///x:/folder_ai/Hime/project-management/system-architecture/HIME-TOOL-AND-SERVICE-ARCHITECTURE.md) membedakan secara eksplisit Agent vs Skill vs Tool vs Service vs Integration, skema kontrak tool, tingkatan risiko, izin, audit log, serta pembedaan sub-proses internal vs tool execution.
- [ ] **AC-06 (Data Architecture):** Dokumen [`HIME-DATA-ARCHITECTURE.md`](file:///x:/folder_ai/Hime/project-management/system-architecture/HIME-DATA-ARCHITECTURE.md) menetapkan arsitektur SQLite (WAL mode, FTS5), batas kepemilikan schema multi-domain, migrasi, jurnal undo, dan proteksi rahasia DPAPI/Secret Service.
- [ ] **AC-07 (Security Architecture):** Dokumen [`HIME-SECURITY-ARCHITECTURE.md`](file:///x:/folder_ai/Hime/project-management/system-architecture/HIME-SECURITY-ARCHITECTURE.md) merancang sentralisasi PEP/PDP, Action Approval Modal dengan delay ergonomi 500 ms, pemisahan plaintext 9Router vs kontrol terencana, isolasi proses, dan mitigasi prompt injection.
- [ ] **AC-08 (Filesystem & Storage Architecture):** Dokumen [`HIME-SYSTEM-ARCHITECTURE.md`](file:///x:/folder_ai/Hime/project-management/system-architecture/HIME-SYSTEM-ARCHITECTURE.md) mengintegrasikan 5 dimensi jaminan filesystem, primitif Win32/Linux, variasi filesystem (FAT32/exFAT vs NTFS/ext4), pengujian empiris, Zero Source File Loss, dan jurnal undo 30 hari.
- [ ] **AC-09 (Desktop Application Architecture):** Dokumen [`HIME-DESKTOP-APPLICATION-ARCHITECTURE.md`](file:///x:/folder_ai/Hime/project-management/system-architecture/HIME-DESKTOP-APPLICATION-ARCHITECTURE.md) merancang Tauri 2, React 19/TS, dynamic island, windowing Win32 `WM_NCHITTEST` / Linux layer-shell, focus protection, dan hotkey hook.
- [ ] **AC-10 (Runtime & Resource Management):** Dokumen [`HIME-RUNTIME-AND-RESOURCE-ARCHITECTURE.md`](file:///x:/folder_ai/Hime/project-management/system-architecture/HIME-RUNTIME-AND-RESOURCE-ARCHITECTURE.md) mengalokasikan anggaran memori/CPU berbasis hardware i5-8250U (~3.7GB RAM bebas), urutan startup/shutdown, Job Objects/prctl supervision, dan penanganan crash.
- [ ] **AC-11 (Cross-Platform Implementation Architecture):** Dokumen [`HIME-CROSS-PLATFORM-IMPLEMENTATION-ARCHITECTURE.md`](file:///x:/folder_ai/Hime/project-management/system-architecture/HIME-CROSS-PLATFORM-IMPLEMENTATION-ARCHITECTURE.md) mendefinisikan Platform Abstraction Layer (PAL), adapter Windows & Linux, fallback mode GNOME Wayland (DIR-03), dan strategi packaging.
- [ ] **AC-12 (Integration Architecture):** Dokumen [`HIME-INTEGRATION-ARCHITECTURE.md`](file:///x:/folder_ai/Hime/project-management/system-architecture/HIME-INTEGRATION-ARCHITECTURE.md) merinci adapter Hermes, 9Router Dual-Control Plane, Android Device Bridge via USB ADB, GitHub, Google Drive, dan headless browser.
- [ ] **AC-13 (Architecture Decision Records):** Dokumen [`HIME-ARCHITECTURE-DECISION-RECORDS.md`](file:///x:/folder_ai/Hime/project-management/system-architecture/HIME-ARCHITECTURE-DECISION-RECORDS.md) memuat 12 ADR lengkap dengan konteks, alternatif, keputusan, konsekuensi, dan reversibilitas.
- [ ] **AC-14 (Technical Implementation Roadmap & Traceability Matrix):** Dokumen [`HIME-TECHNICAL-IMPLEMENTATION-ROADMAP.md`](file:///x:/folder_ai/Hime/project-management/system-architecture/HIME-TECHNICAL-IMPLEMENTATION-ROADMAP.md) memetakan seluruh 33 FR dan 18 NFR ke komponen, kontrak, task implementasi, dan metode verifikasi.
- [ ] **AC-15 (Task Completion Report):** Dokumen [`TASK-02-05-COMPLETION-REPORT.md`](file:///x:/folder_ai/Hime/project-management/system-architecture/TASK-02-05-COMPLETION-REPORT.md) merangkum eksekutif closure task.
- [ ] **AC-16 (Zero Implementation Code, Zero Commit, Zero Push):** Disiplin mutlak dipatuhi.

---

## 3. Deliverables & Documentation Index

| Deliverable ID | Berkas Dokumen | Deskripsi Konten | Status |
|:---:|:---|:---|:---:|
| **DEL-02-46** | [`project-management/tasks/TASK-02-05-system-architecture.md`](file:///x:/folder_ai/Hime/project-management/tasks/TASK-02-05-system-architecture.md) | Dokumen Spesifikasi Formal Tata Kelola Task Ini | **DRAFT** |
| **DEL-02-47** | [`project-management/system-architecture/HIME-SYSTEM-ARCHITECTURE.md`](file:///x:/folder_ai/Hime/project-management/system-architecture/HIME-SYSTEM-ARCHITECTURE.md) | Master Blueprint Arsitektur Sistem Hime | **PLANNED** |
| **DEL-02-48** | [`project-management/system-architecture/HIME-C4-ARCHITECTURE.md`](file:///x:/folder_ai/Hime/project-management/system-architecture/HIME-C4-ARCHITECTURE.md) | C4 Context, Container, Component, & Deployment Diagrams | **PLANNED** |
| **DEL-02-49** | [`project-management/system-architecture/HIME-CORE-INTERNAL-ARCHITECTURE.md`](file:///x:/folder_ai/Hime/project-management/system-architecture/HIME-CORE-INTERNAL-ARCHITECTURE.md) | Arsitektur Modular Monolith Hime Core (14 Domain Modules) | **PLANNED** |
| **DEL-02-50** | [`project-management/system-architecture/HIME-IPC-AND-COMMUNICATION-ARCHITECTURE.md`](file:///x:/folder_ai/Hime/project-management/system-architecture/HIME-IPC-AND-COMMUNICATION-ARCHITECTURE.md) | Spesifikasi Protokol IPC, Wire Framing, & Schema JSON-RPC | **PLANNED** |
| **DEL-02-51** | [`project-management/system-architecture/HIME-AGENT-ORCHESTRATION-ARCHITECTURE.md`](file:///x:/folder_ai/Hime/project-management/system-architecture/HIME-AGENT-ORCHESTRATION-ARCHITECTURE.md) | Arsitektur Orkestrasi Agen, Reasoning Loop, & Activity Stream | **PLANNED** |
| **DEL-02-52** | [`project-management/system-architecture/HIME-TOOL-AND-SERVICE-ARCHITECTURE.md`](file:///x:/folder_ai/Hime/project-management/system-architecture/HIME-TOOL-AND-SERVICE-ARCHITECTURE.md) | Arsitektur Tool, Service, Skill, Kontrak Schema, & Risk Tiers | **PLANNED** |
| **DEL-02-53** | [`project-management/system-architecture/HIME-DATA-ARCHITECTURE.md`](file:///x:/folder_ai/Hime/project-management/system-architecture/HIME-DATA-ARCHITECTURE.md) | Arsitektur Data SQLite WAL, FTS5, Migrasi, & Domain Schemas | **PLANNED** |
| **DEL-02-54** | [`project-management/system-architecture/HIME-SECURITY-ARCHITECTURE.md`](file:///x:/folder_ai/Hime/project-management/system-architecture/HIME-SECURITY-ARCHITECTURE.md) | Arsitektur Keamanan, PEP/PDP, Threat Model, & 9Router State | **PLANNED** |
| **DEL-02-55** | [`project-management/system-architecture/HIME-DESKTOP-APPLICATION-ARCHITECTURE.md`](file:///x:/folder_ai/Hime/project-management/system-architecture/HIME-DESKTOP-APPLICATION-ARCHITECTURE.md) | Arsitektur Desktop Shell Tauri 2, 3-Tier Morphology, & Windowing | **PLANNED** |
| **DEL-02-56** | [`project-management/system-architecture/HIME-RUNTIME-AND-RESOURCE-ARCHITECTURE.md`](file:///x:/folder_ai/Hime/project-management/system-architecture/HIME-RUNTIME-AND-RESOURCE-ARCHITECTURE.md) | Arsitektur Runtime, Supervisi Proses, & Anggaran Resource Host | **PLANNED** |
| **DEL-02-57** | [`project-management/system-architecture/HIME-CROSS-PLATFORM-IMPLEMENTATION-ARCHITECTURE.md`](file:///x:/folder_ai/Hime/project-management/system-architecture/HIME-CROSS-PLATFORM-IMPLEMENTATION-ARCHITECTURE.md) | Arsitektur PAL, Adapter Windows & Linux, serta Packaging | **PLANNED** |
| **DEL-02-58** | [`project-management/system-architecture/HIME-INTEGRATION-ARCHITECTURE.md`](file:///x:/folder_ai/Hime/project-management/system-architecture/HIME-INTEGRATION-ARCHITECTURE.md) | Arsitektur Integrasi Hermes, 9Router, Android ADB, & Layanan Luar | **PLANNED** |
| **DEL-02-59** | [`project-management/system-architecture/HIME-ARCHITECTURE-DECISION-RECORDS.md`](file:///x:/folder_ai/Hime/project-management/system-architecture/HIME-ARCHITECTURE-DECISION-RECORDS.md) | 12 Architecture Decision Records (ADR-001 s/d ADR-012) | **PLANNED** |
| **DEL-02-60** | [`project-management/system-architecture/HIME-TECHNICAL-IMPLEMENTATION-ROADMAP.md`](file:///x:/folder_ai/Hime/project-management/system-architecture/HIME-TECHNICAL-IMPLEMENTATION-ROADMAP.md) | Matriks Ketertelusuran (Traceability), Fase Implementasi, & Gaps | **PLANNED** |
| **DEL-02-61** | [`project-management/system-architecture/TASK-02-05-COMPLETION-REPORT.md`](file:///x:/folder_ai/Hime/project-management/system-architecture/TASK-02-05-COMPLETION-REPORT.md) | Laporan Sintesis Eksekutif Akhir TASK-02-05 | **PLANNED** |
