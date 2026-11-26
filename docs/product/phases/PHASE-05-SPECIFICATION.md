# Phase Specification: Phase 05 — Development Environment & Implementation Readiness

- **Phase ID:** PHASE-05
- **Phase Name:** Development Environment & Implementation Readiness
- **Target Milestone:** Milestone 5 (Host Readiness & Toolchain Verification)
- **Status:** **SPECIFIED — READY FOR EXECUTION**
- **Lead / Owner:** System Architect & Technical Lead
- **Execution Role:** Senior Software Engineer & Technical Execution Agent
- **Date:** 2026-11-09

---

## 1. Overview & Objectives

### Objective
Fase ini berfokus pada verifikasi komprehensif lingkungan pengembangan host, penyiapan skrip otomatisasi alur kerja pengembangan (*development workflow scripts*), konfigurasi lokal, serta penetapan prosedur verifikasi kesiapan (*readiness verification*) sebelum implementasi kode sumber Phase 06 dapat dimulai.

Prinsip mutlak fase ini adalah **ZERO AUTOMATIC SOFTWARE INSTALLATION**: Agen AI dilarang melakukan instalasi software, compiler, atau paket sistem secara otomatis atau diam-diam. Seluruh kebutuhan toolchain yang belum tersedia didokumentasikan dengan panduan instalasi resmi untuk dipasang secara mandiri oleh Project Owner.

### Lingkup Kerja Utama:
1. **Audit & Verifikasi Kesiapan Toolchain Host Windows 11:**
   - Mengkaji status aktual perangkat lunak dan kompilator yang terpasang pada host.
   - Mengklasifikasikan komponen secara transparan:
     * *Already Available `[VERIFIED]`*: Node.js v24.2.0, Python 3.13, uv 0.12.0, Git v2.55.0, WebView2 Runtime v157.0.
     * *Not Available `[MISSING]`*: Rust compiler (`rustc 1.80+`), package manager `cargo`, MSVC C++ Build Tools 2022 (v143), Windows 10/11 SDK, Tauri CLI v2 (`cargo-tauri`).
     * *Owner Action Required*: Pengguna mengunduh dan memasang `rustup-init.exe` bersama Visual Studio Build Tools 2022.
     * *Can Be Prepared Without Installation*: Skrip alur kerja, file konfigurasi template, desain manifes workspace, dan skrip verifikasi lingkungan.
2. **Pengembangan Skrip Otomatisasi Alur Kerja (Development Scripts):**
   - `scripts/check-env.ps1`: Skrip diagnostik untuk memverifikasi versi seluruh compiler, runtime, dan path environment tanpa mengubah konfigurasi sistem.
   - `scripts/build.ps1`: Skrip wrapper kompilasi workspace monorepo multi-crate.
   - `scripts/test.ps1`: Skrip eksekusi test suite berjenjang (unit test, contract test, integration test).
   - `scripts/dev.ps1`: Skrip peluncuran terpadu lingkungan pengembangan lokal (Tauri dev server + Hime Core debug binary).
3. **Konfigurasi Lingkungan Lokal (Local Development Configuration):**
   - Penetapan port TCP loopback lokal (`127.0.0.1:20128` untuk 9Router, port debugger).
   - Penetapan nama transpor Named Pipe Windows (`\\.\pipe\hime-hermes-dev`) dan Linux UDS (`$XDG_RUNTIME_DIR/hime/dev.sock`).
   - Penetapan struktur path direktori data lokal terisolasi (`%LOCALAPPDATA%\Hime\dev\`).
   - Template variabel lingkungan lokal (`.env.example`) dengan redaksi secret.
4. **Strategi Cargo Workspace & Konfigurasi Profil Kompilasi:**
   - Desain virtual manifest `Cargo.toml` root.
   - Konfigurasi profil kompilasi: `dev` (fast compilation, debug symbols, overflow checks) vs `release` (LTO, opt-level 3, stripped symbols).
5. **Security Baseline untuk Lingkungan Pengembangan:**
   - Penguatan `.gitignore` untuk mencegah kebocoran file kredensial lokal (`*.db`, `*.sqlite`, `.9router/`, `*.key`, `*.pem`, `target/`).
   - Larangan hardcoding token atau password default pada skrip pengujian.

---

## 2. Prerequisites & Entry Criteria

- [x] Phase 04 (Engineering Planning & Implementation Specification) telah dispesifikasikan.
- [x] Kebutuhan perangkat keras empiris host telah diverifikasi (Intel Core i5-8250U, ~3.7 GB RAM bebas, Intel UHD 620).
- [x] Dokumen arsitektur dan batasan rekayasa ([`HIME-IMPLEMENTATION-CONSTRAINTS.md`](file:///x:/folder_ai/Hime/project-management/system-architecture/HIME-IMPLEMENTATION-CONSTRAINTS.md)) telah dipublikasikan.
- [x] Dua Implementation Blockers (IB-01 & IB-02) dipertahankan sebagai syarat keamanan aktif.

---

## 3. Work Breakdown Structure (Tasks)

| Task ID | Judul Task | Prioritas | Tanggung Jawab | Status | Deliverables Utama |
|:---|:---|:---:|:---:|:---:|:---|
| **TASK-05-01** | Host Toolchain Audit & Readiness Verification Procedure | CRITICAL | Senior Infrastructure Engineer | BACKLOG | `docs/environment/HOST-TOOLCHAIN-AUDIT-REPORT.md`, `scripts/check-env.ps1` |
| **TASK-05-02** | Development Scripts & Build/Test Workflow Automation | HIGH | Senior Software Engineer | BACKLOG | `scripts/build.ps1`, `scripts/test.ps1`, `scripts/dev.ps1` |
| **TASK-05-03** | Local Development Configuration & Sandboxing Setup | HIGH | Senior Systems Engineer | BACKLOG | `docs/environment/LOCAL-DEVELOPMENT-CONFIG.md`, `.env.example` |
| **TASK-05-04** | Cargo Workspace Virtual Manifest & Profile Architecture | HIGH | Lead Systems Architect | BACKLOG | `docs/environment/CARGO-WORKSPACE-ARCHITECTURE.md` |
| **TASK-05-05** | Development Security Baseline & Secrets Git-Exclusion | HIGH | Senior Security Architect | BACKLOG | `docs/security/DEVELOPMENT-SECURITY-BASELINE.md`, `.gitignore` verification |
| **TASK-05-06** | Phase 05 Verification & Implementation Gate Sign-off | CRITICAL | Technical Governance Lead | BACKLOG | `project-management/phases/PHASE-05-ACCEPTANCE.md` |

---

## 4. Exit Criteria & Definition of Done

- [ ] Skrip diagnostik `scripts/check-env.ps1` selesai dibuat dan mampu mendeteksi ketersediaan seluruh toolchain secara non-destruktif.
- [ ] Panduan instalasi manual toolchain yang hilang (Rust 1.80+ MSVC, Visual Studio Build Tools 2022) disajikan secara jelas dan terverifikasi untuk pengguna.
- [ ] Seluruh skrip pengembangan (`build.ps1`, `test.ps1`, `dev.ps1`) selesai disiapkan dan diuji sintaksisnya.
- [ ] Template konfigurasi lokal (`.env.example`) dan aturan `.gitignore` diverifikasi mencegah kebocoran secret.
- [ ] Project Owner telah menyelesaikan instalasi toolchain yang hilang, dibuktikan dengan luaran sukses dari `scripts/check-env.ps1`.
- [ ] Laporan Akseptansi Phase 05 disahkan oleh System Architect & Technical Lead sebelum memulai Phase 06.
