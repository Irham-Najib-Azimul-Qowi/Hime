# Phase Specification: Phase 06 — Engineering Foundation

- **Phase ID:** PHASE-06
- **Phase Name:** Engineering Foundation & Workspace Bootstrap
- **Target Milestone:** Milestone 6 (Core Rust Plumbing & Foundation)
- **Status:** **SPECIFIED — PENDING PREREQUISITES**
- **Lead / Owner:** System Architect & Technical Lead
- **Execution Role:** Senior Software Engineer & Technical Execution Agent
- **Date:** 2026-11-09

---

## 1. Overview & Objectives

### Objective
Fase ini berfokus pada pembangunan fondasi rekayasa teknis pertama (*initial core engineering foundation*) untuk ekosistem Hime. Pada fase ini, baris kode implementasi pertama akan ditulis untuk menginisialisasi repositori monorepo Rust multi-crate, menyusun kerangka (*skeleton*) Hime Desktop dan Hime Core, membangun modul transpor IPC biner `HIME`, mengimplementasikan modul supervisor proses berbasis Windows Job Object, membangun modul secret vault berbasis Windows DPAPI, serta menyiapkan harness pengujian otomatis.

Fase ini **TIDAK mencakup seluruh fitur produk Hime** (seperti Dynamic Island widget cards, orkestrasi penalaran AI lengkap, atau otomatisasi browser). Fase ini semata-mata membangun fondasi *plumbing* teknis yang dapat dikompilasi secara bersih, diuji keandalannya secara deterministik, dan dijadikan pijakan kokoh untuk Phase 07 (Desktop Presentation) dan Phase 08 (AI Runtime Integration).

### Lingkup Kerja Utama:
1. **Inisialisasi Monorepo Rust Multi-Crate (Cargo Workspace):**
   - Root virtual `Cargo.toml` yang menaungi 6 target crate:
     * `apps/hime-desktop`: Shell antarmuka berbasis Tauri 2.0 (Rust backend + React 19).
     * `crates/hime-core`: Kernel modular monolith (Tokio async runtime, message bus, policy engine).
     * `crates/hime-ipc`: Protokol transpor IPC, framing biner `HIME`, serializer JSON-RPC 2.0.
     * `crates/hime-pal`: Platform Abstraction Layer (Win32 HWND subclassing, DirectComposition hooks).
     * `crates/hime-storage`: Database SQLite WAL connection pool, FTS5 virtual tables.
     * `crates/hime-security`: Secret vault berbasis DPAPI (`CryptProtectData`), token CSPRNG.
2. **Implementasi Skeleton Hime Desktop:**
   - Bootstrap proyek Tauri 2.0 menggunakan template resmi (`tauri init`).
   - Kerangka minimal frontend React 19 + TypeScript + Vite + Tailwind CSS.
   - Pendaftaran command Tauri dasar untuk komunikasi bridge frontend-backend.
3. **Implementasi Skeleton Hime Core:**
   - Inisialisasi runtime Tokio multithreaded (`#[tokio::main]`).
   - Deklarasi trait modular monolith (`Engine`, `Supervisor`, `PolicyEnforcer`, `StorageEngine`).
   - In-memory event bus terstruktur menggunakan Tokio `mpsc` / `broadcast` channels.
4. **Implementasi Modul Transpor IPC (`hime-ipc`):**
   - Serializer dan deserializer framing biner `HIME` (12-byte header).
   - Validasi batas payload maksimum $16\text{ MB}$ sebelum alokasi heap (*pre-allocation check*).
   - Server dan client transpor Windows Named Pipe (`\\.\pipe\hime-hermes`) dengan konfigurasi DACL eksklusif Current User SID.
   - Fallback transpor Linux Unix Domain Socket (`mode 0600`).
   - Handshake authentication validator berbasis CSPRNG 256-bit token dan monotonic sequence nonces ($N_{\text{msg}} = N_{\text{last}} + 1$).
5. **Implementasi Process Supervisor (`hime-supervisor`):**
   - Manajemen subproses via Win32 Job Object (`HimeSupervisorJob`) dengan batas `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`.
   - Jaminan pemusnahan seluruh child processes seketika saat proses induk dihentikan paksa (*Zero Orphan Processes*).
   - Generator token acak CSPRNG untuk inisialisasi password `INITIAL_PASSWORD` 9Router.
6. **Implementasi Secret Management Foundation:**
   - Wrapper aman terhadap API Windows DPAPI (`CryptProtectData` dan `CryptUnprotectData`) dengan proteksi lingkup `CurrentUser`.
   - Abstraksi trait `SecretVault` untuk kompatibilitas lintas-platform masa depan (Linux Secret Service / libsecret).
7. **Implementasi Logging Terstruktur & Observability:**
   - Inisialisasi subscriber `tracing` dan `tracing-subscriber` Rust.
   - Filter redaksi data sensitif (regex filter terhadap API key, bearer token, dan password) untuk mencegah kebocoran secret pada log sistem (**SEC-09**).
8. **Automated Test Harness:**
   - Unit tests untuk seluruh crate logic.
   - Contract tests untuk framing biner `HIME` dan pesan JSON-RPC 2.0.
   - Integration test supervisor: peluncuran subproses dummy dan verifikasi terminasi otomatis saat parent dimatikan (`taskkill /F`).

---

## 2. Prerequisites & Entry Criteria

- [ ] **Toolchain Gate (TG-01):** Compiler Rust $\ge 1.80.0$, Cargo, dan MSVC C++ Build Tools 2022 telah terpasang dan diverifikasi aktif pada host oleh Project Owner (`cargo --version` lulus).
- [ ] **Phase 04 Gate:** Dokumen spesifikasi implementasi Phase 04 telah disahkan oleh System Architect & Technical Lead.
- [ ] **Phase 05 Gate:** Skrip pengembangan dan pemeriksaan lingkungan Phase 05 selesai diverifikasi (`scripts/check-env.ps1` keluar dengan kode 0).
- [ ] **Security Blockers Retained:** IB-01 (Kredensial 9Router) dan IB-02 (Localhost Bypass 9Router) tetap berstatus OPEN dan diikatkan pada gerbang keamanan implementasi selanjutnya.

---

## 3. Work Breakdown Structure (Tasks)

| Task ID | Judul Task | Prioritas | Tanggung Jawab | Status | Deliverables Utama |
|:---|:---|:---:|:---:|:---:|:---|
| **TASK-06-01** | Cargo Workspace & Monorepo Multi-Crate Bootstrap | CRITICAL | Senior Systems Engineer | BACKLOG | `Cargo.toml`, 6 crate manifests |
| **TASK-06-02** | Structured Logging & Secret Redaction Engine (SEC-09) | HIGH | Senior Software Engineer | BACKLOG | `crates/hime-core/src/logging.rs` |
| **TASK-06-03** | Secret Vault Foundation & Windows DPAPI Wrapper | CRITICAL | Senior Security Engineer | BACKLOG | `crates/hime-security/src/dpapi.rs` |
| **TASK-06-04** | Process Supervisor & Windows Job Object Implementation | CRITICAL | Senior Systems Engineer | BACKLOG | `crates/hime-core/src/supervisor/` |
| **TASK-06-05** | IPC Binary Framing `HIME` & Transport Library | CRITICAL | Senior Systems Engineer | BACKLOG | `crates/hime-ipc/src/` |
| **TASK-06-06** | Hime Desktop Skeleton & Tauri 2.0 Initialization | HIGH | Frontend / Desktop Lead | BACKLOG | `apps/hime-desktop/` |
| **TASK-06-07** | Integration Adapter Interfaces & Trait Contracts | HIGH | Senior Software Engineer | BACKLOG | `crates/hime-core/src/adapters/` |
| **TASK-06-08** | Foundation Test Harness & Subprocess Termination Audit | HIGH | QA & Security Engineer | BACKLOG | `tests/foundation_integration.rs` |

---

## 4. Exit Criteria & Definition of Done

- [ ] Seluruh 8 task rekayasa fondasi selesai diimplementasikan.
- [ ] `cargo check --workspace` dan `cargo build --workspace` lulus dengan 0 compiler errors dan 0 warnings.
- [ ] `cargo test --workspace` lulus dengan 100% test pass rate.
- [ ] Uji deterministik process supervisor membuktikan bahwa subproses tiruan mati secara instan saat proses induk di-kill mendadak via `taskkill /F /PID <pid>` (*Zero Orphan Verification*).
- [ ] Uji roundtrip IPC membuktikan frame biner `HIME` berhasil dikirim dan diterima melalui Windows Named Pipe terotentikasi, serta menolak frame $> 16\text{ MB}$ dan payload replay secara benar.
- [ ] Uji secret vault membuktikan enkripsi dan dekripsi data melalui Windows DPAPI berhasil tanpa menulis plaintext secret ke diska.
- [ ] Laporan Akseptansi Phase 06 diterbitkan dan disahkan oleh System Architect & Technical Lead.
