# Phase Specification: Phase 04 — Engineering Planning & Implementation Specification

- **Phase ID:** PHASE-04
- **Phase Name:** Engineering Planning & Implementation Specification
- **Target Milestone:** Milestone 4 (Architecture-to-Implementation Translation)
- **Status:** **SPECIFIED — READY FOR EXECUTION**
- **Lead / Owner:** System Architect & Technical Lead
- **Execution Role:** Senior Software Engineer & Technical Execution Agent
- **Date:** 2026-11-09

---

## 1. Overview & Objectives

### Objective
Fase ini berfokus pada persiapan rekayasa teknis mendalam yang menerjemahkan cetak biru arsitektur Hime ([`TASK-02-05`](file:///x:/folder_ai/Hime/project-management/system-architecture/HIME-SYSTEM-ARCHITECTURE.md) dan [`TASK-02-05A`](file:///x:/folder_ai/Hime/project-management/system-architecture/TASK-02-05A-ACCEPTANCE-REPORT.md)) menjadi spesifikasi implementasi terinci yang siap dieksekusi langsung pada kode sumber.

Fase ini menjembatani dokumen arsitektur konseptual dengan repositori nyata, menetapkan kepemilikan modul, batas toleransi komponen, kontrak antarmuka API & IPC biner, strategi integrasi subproses eksternal (Hermes & 9Router), strategi keamanan data at-rest dan in-transit, serta *Definition of Done (DoD)* untuk setiap modul sebelum baris kode pertama ditulis.

### Lingkup Kerja Utama:
1. **Finalisasi Batas Tanggung Jawab Komponen (Bounded Contexts):**
   - Hime Desktop (Presentation, Native Windowing, Win32 DirectComposition, HUD, React 19 / Motion).
   - Hime Core (Engine Modular Monolith Rust, Tokio Async, PDP/PEP Centralized Enforcement, Task Engine, Context Manager, Storage Service, Process Supervisor).
   - Hermes Agent (NousResearch Python Runtime, 10-Phase Reasoning Loop, shimmed tool execution via IPC, zero direct OS bypass).
   - 9Router AI Gateway (Node.js Service, Managed Background Service pada `127.0.0.1:20128`, Dual-Control Plane, multi-provider load balancing & fallback).
2. **Struktur Repositori & Kepemilikan Modul (Cargo Workspace Layout):**
   - `apps/hime-desktop`: Tauri 2.0 shell + React 19 frontend.
   - `crates/hime-core`: Modular monolith kernel (domain logic, state machine, task engine).
   - `crates/hime-ipc`: Framing biner `HIME`, serializer JSON-RPC 2.0, transport Named Pipe / UDS.
   - `crates/hime-pal`: Platform Abstraction Layer (Win32 DirectComposition, window subclassing, Linux fallback).
   - `crates/hime-storage`: SQLite WAL connection manager, FTS5 migration, safe move engine, 30-day undo.
   - `crates/hime-security`: Secret vault (DPAPI), token CSPRNG, policy engine & validation rules.
3. **Definisi Kontrak Antarkomponen (API & IPC Schemas):**
   - Framing biner `HIME` (12-byte header: Magic `0x48494D45`, Version `0x01`, MsgType, Length `u32` Big-Endian).
   - Payload JSON-RPC 2.0 terstandarisasi untuk pemanggilan tool (`tool_call.request`, `tool_call.response`), token streaming (`stream.chunk`), notifikasi status (`status.update`), dan pembatalan (`cancel.request`).
   - Aturan anti-replay: Token CSPRNG 256-bit ephemeral, strictly monotonic sequence nonces ($N_{\text{msg}} = N_{\text{last}} + 1$), dan drift waktu $\le \pm 5\text{ detik}$.
4. **Penyematan Versi Teknologi & Dependensi (Dependency Pinning):**
   - Rust 2021 edition (compiler $\ge 1.80.0$).
   - Tauri v2.0 stable.
   - React 19, TypeScript 5.5+, Vite 5+, Tailwind CSS 3.4+, Motion (Framer Motion) 11+.
   - Python 3.11/3.13, uv 0.12+.
   - Node.js v24 LTS.
   - SQLite 3.45+ (bundled via `rusqlite` dengan feature `bundled`, `modern_sqlite`, `fts5`, `vfs`).
5. **Strategi Integrasi Hermes & 9Router:**
   - Kepatuhan mutlak ADR-006: Nol modifikasi pada kode sumber repositori upstream.
   - Manajemen lifecycle subproses via Windows Job Object (`JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`).
   - Healthcheck loopback HTTP dan verifikasi kepemilikan PID socket TCP 20128 (`GetExtendedTcpTable`).
6. **Strategi Konfigurasi & Secret Vault:**
   - Proteksi kredensial at-rest via Windows DPAPI (`CryptProtectData`) dan Linux Secret Service / libsecret.
   - Mitigasi IB-01: Penyimpanan kredensial berbayar tidak pernah ditulis plaintext ke SQLite 9Router.
7. **Strategi Penyimpanan Data & 5-Dimensi Integritas Sistem Berkas:**
   - SQLite WAL single-writer dedicated thread + read-only connection pool.
   - Same-volume atomic rename (`MoveFileExW` / `renameat2`) dan cross-volume copy-hash-verify-delete.
   - Direktori reversible safety trash 30 hari (`storage_journal`).
8. **Strategi Pengujian & QA Matrix:**
   - Unit test per-crate, contract tests untuk IPC schemas, integration test multi-proses, security test (STRIDE audit).
9. **Observability, Tracing & Error Handling:**
   - Structured logging via crate `tracing`, redaksi otomatis token & API key via regex filter (SEC-09).
   - Fail-closed behavior pada seluruh validasi keamanan.
10. **Definition of Done (DoD) & Implementation Plan Berdasarkan Ketergantungan.**

---

## 2. Prerequisites & Entry Criteria

- [x] Phase 00 (Project Governance) telah diterima resmi (*Accepted*).
- [x] Phase 01 (Discovery & Technical Validation) telah diterima resmi (*Accepted*).
- [x] Phase 02 (Nusa Reverse Engineering & UX Discovery) telah menyelesaikan seluruh 7 task dan berstatus `READY FOR OWNER ACCEPTANCE`.
- [x] Phase 03 (Dependencies Audit, Product Requirements & System Architecture) telah menyelesaikan 51 dokumen dan berstatus `READY FOR OWNER ACCEPTANCE`.
- [x] Cetak biru arsitektur ([`TASK-02-05`](file:///x:/folder_ai/Hime/project-management/system-architecture/HIME-SYSTEM-ARCHITECTURE.md)) dan 12 ADR disahkan.
- [x] Register 16 Batasan Rekayasa ([`HIME-IMPLEMENTATION-CONSTRAINTS.md`](file:///x:/folder_ai/Hime/project-management/system-architecture/HIME-IMPLEMENTATION-CONSTRAINTS.md)) aktif dan terikat pada seluruh fase.
- [x] Dua Implementation Blockers (IB-01 & IB-02) dipertahankan sebagai Security Gates aktif.

---

## 3. Work Breakdown Structure (Tasks)

| Task ID | Judul Task | Prioritas | Tanggung Jawab | Status | Deliverables Utama |
|:---|:---|:---:|:---:|:---:|:---|
| **TASK-04-01** | Component Responsibility & Module Ownership Matrix | HIGH | Senior Software Engineer | BACKLOG | `docs/architecture/COMPONENT-BOUNDARIES-AND-OWNERSHIP.md` |
| **TASK-04-02** | IPC Binary Framing & JSON-RPC Protocol Specification | CRITICAL | Senior Systems Engineer | BACKLOG | `docs/architecture/IPC-BINARY-PROTOCOL-SPECIFICATION.md` |
| **TASK-04-03** | Subprocess Supervisor & Lifecycle Management Spec | HIGH | Senior Systems Engineer | BACKLOG | `docs/architecture/SUBPROCESS-LIFECYCLE-SPECIFICATION.md` |
| **TASK-04-04** | Secret Management & Credential Injection Design (IB-01) | CRITICAL | Senior Security Architect | BACKLOG | `docs/security/SECRET-MANAGEMENT-AND-INJECTION-SPEC.md` |
| **TASK-04-05** | 9Router Localhost Guard & PID Verification Spec (IB-02) | CRITICAL | Senior Security Architect | BACKLOG | `docs/security/LOCALHOST-GUARD-AND-PID-VERIFICATION-SPEC.md` |
| **TASK-04-06** | SQLite WAL & 5D Filesystem Operations Specification | HIGH | Senior Storage Architect | BACKLOG | `docs/architecture/STORAGE-AND-SAFE-FILESYSTEM-SPEC.md` |
| **TASK-04-07** | Observability, Tracing & Error Handling Specification | MEDIUM | Senior Software Engineer | BACKLOG | `docs/architecture/OBSERVABILITY-AND-ERROR-HANDLING-SPEC.md` |
| **TASK-04-08** | Testing Strategy, QA Matrix & Definition of Done | HIGH | QA & Governance Lead | BACKLOG | `docs/testing/TESTING-STRATEGY-AND-DOD.md` |

---

## 4. Exit Criteria & Definition of Done

- [ ] Seluruh 8 task spesifikasi implementasi (TASK-04-01 s/d TASK-04-08) diselesaikan tanpa ambiguitas tipe atau protokol.
- [ ] Batas kepemilikan crate dan modul monorepo terpetakan 100% tanpa tumpang-tindih tanggung jawab.
- [ ] Kontrak skema pesan IPC biner `HIME` dan payload JSON-RPC 2.0 terdefinisi lengkap dengan validator panjang pre-allocation $\le 16\text{ MB}$.
- [ ] Desain mitigasi teknis untuk IB-01 (in-memory injection/proxy) dan IB-02 (socket owner PID check & reverse proxy) terspesifikasi secara detail.
- [ ] Matriks pengujian (Unit, Contract, Integration, Security, Benchmark) dan kriteria Definition of Done (DoD) formal ditetapkan untuk seluruh komponen.
- [ ] Laporan Akseptansi Phase 04 diterbitkan dan disahkan oleh System Architect & Technical Lead.
