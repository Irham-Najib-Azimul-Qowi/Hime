# Hime Assistant — Dependency-Aware Implementation Roadmap (Phases 02–15)

- **Document ID:** IMPLEMENTATION-ROADMAP
- **Phase:** Phase 01 — Discovery & Technical Validation
- **Authority:** Senior Engineering Infrastructure Architect & System Architect
- **Status:** **APPROVED BASELINE ROADMAP**
- **Date:** 2026-11-08

---

## 1. Executive Summary & Hardware Realism Framework

Peta jalan implementasi ini memetakan urutan pengembangan teknis ekosistem Hime Assistant dari Phase 02 hingga Phase 15. Peta jalan disusun berdasarkan analisis ketergantungan antar-komponen (*dependency graph*), mitigasi risiko terkonfirmasi, dan batasan perangkat keras empiris yang terukur pada mesin host:

- **Batas Komputasi & Memori Riil:** Host ditenagai CPU Intel Core i5-8250U (4C/8T) dengan sisa memori fisik bebas terukur sebesar **~3.71 GB** dan iGPU Intel UHD Graphics 620.
- **Strategi Inferensi Cloud-First:** Pengembangan difokuskan pada orkestrasi model cloud multi-penyedia melalui 9Router. Inferensi LLM lokal tidak dimasukkan ke dalam jalur kritis (*critical path*) tahap awal.
- **Pengembangan Bertahap (*Staged Concurrency*):** Komponen inti (Hime Core dan Process Supervisor) diselesaikan sebelum lapisan presentasi visual (Hime Desktop) dan integrasi agen otonom (Hermes Agent) dihubungkan.

---

## 2. Alignment with Official 16-Phase Governance

Peta jalan ini mematuhi penomoran dan pembagian domain 16 fase resmi yang tercatat pada `PROJECT_STATUS.md` dan *Hime Engineering Master Plan*:

```text
Fase Tata Kelola & Riset:
├── Phase 00: Project Governance (COMPLETED)
├── Phase 01: Discovery & Technical Validation (CLOSING / READY FOR ACCEPTANCE)
├── Phase 02: Nusa Reverse Engineering (RESEARCH COMPLETED — PENDING FORMAL ACCEPTANCE)
└── Phase 03: Hermes & 9Router Integration Audit (RESEARCH COMPLETED — PENDING FORMAL ACCEPTANCE)

Fase Spesifikasi & Desain Arsitektur:
├── Phase 04: Product & System Specification (Dynamic Island Morphology & UX Specs)
└── Phase 05: Architecture & Security Design (ADR Formulation, Threat Modeling, IPC Specs)

Fase Rekayasa Inti & Integrasi Desktop:
├── Phase 06: Engineering Foundation (Rust Toolchain, Hime Core Bootstrap, Supervisor, SEC-01..04)
├── Phase 07: Hime Desktop (Tauri 2.0 Rust Shell, DirectComposition HUD, Named Pipes)
├── Phase 08: AI Runtime Integration (Hermes Python, 9Router Node.js, Streaming Token Relay)
├── Phase 09: Hime Intelligence Core (Context Distillation, Semantic Routing, Master Vault)
└── Phase 10: Storage Intelligence (SQLite FTS5 Local Vector/Index, Conversation History)

Fase Ekosistem Lintas Perangkat & Produktivitas:
├── Phase 11: Hime Mobile & Device Layer (Android Native Kotlin, Foreground Service, Remote Sync)
├── Phase 12: Productivity & Documents (Workspace Document Parsers, Task Extraction)
├── Phase 13: Browser & Developer Workspace (CDP Automation Sandbox, Local Dev Tools)
├── Phase 14: Specialist Agents & Advanced Skills (Sub-agent Workflows, Dynamic Skill Loader)
└── Phase 15: Hardening, Release & Maintenance (Authenticode Signing, Installer, Auto-Update)
```

---

## 3. Detailed Workstream Specifications (11 Core Workstreams)

---

### WS-1: Architecture & Technical Design (Phases 04 & 05)
- **Objective:** Menyusun spesifikasi fungsional produk, interaksi state HUD Dynamic Island, formalisasi Architecture Decision Records (ADR), dan desain kontrak IPC detail.
- **Prerequisites:** Formal acceptance Phase 01, 02, dan 03.
- **Expected Deliverables:**
  * Dokumen Spesifikasi Produk & State Morphology Dynamic Island (`docs/master-plan/PRODUCT_SPEC.md`).
  * Serangkaian ADR kanonikal di `docs/adr/` (ADR-001 Modular Monolith, ADR-002 Named Pipes IPC, ADR-003 Dual-Control Plane, ADR-004 Storage Migration).
  * Spesifikasi Skema Protokol IPC Named Pipe (framing, pesan protobuf/json, skema handshake).
- **Technical Dependencies:** Hasil reverse engineering Nusa (`RE/Nusa/`) dan 9Router/Hermes (`RE/9ROUTER/`, `RE/Hermes/`).
- **Security Requirements:** Validasi desain mitigasi RSK-10, batasan DACL Named Pipe, dan arsitektur token handshake.
- **Acceptance Criteria:** Seluruh ADR disetujui Project Owner; kontrak IPC terdefinisi tanpa ambiguitas tipe.
- **Blocking Decisions:** DEC-SEC-03 (Pilihan strategi remedi RSK-10), DEC-ARCH-03 (Pola Modular Monolith).
- **Suggested Order:** Urutan ke-1 (Fondasi mutlak sebelum menulis kode).

---

### WS-2: Hime Core Foundation (Phase 06)
- **Objective:** Membangun *kernel* logika Hime Core dalam bahasa Rust sebagai *modular monolith* yang aman dan efisien.
- **Prerequisites:** Rust compiler (`rustc`/`cargo`) & MSVC C++ Build Tools terpasang; WS-1 disetujui.
- **Expected Deliverables:**
  * Crate `hime-core` (Workspace Rust root).
  * Struktur modul domain: `domain/`, `ports/`, `adapters/`, `supervisor/`, `vault/`.
  * Logging terstruktur dengan filter redaksi data sensitif (**SEC-09**).
- **Technical Dependencies:** Rust crates: `tokio`, `tracing`, `serde`, `thiserror`.
- **Security Requirements:** **SEC-09** (Redaksi regex token/kunci API), pengerasan DACL proses (**DEC-SEC-04**).
- **Acceptance Criteria:** `cargo build` dan `cargo test` lulus 100%; unit test modular monolith lolos.
- **Blocking Decisions:** DEC-TOOL-01 (Pengguna menginstal Rust toolchain).
- **Suggested Order:** Urutan ke-2 (Paralel dengan awal WS-3).

---

### WS-3: Process Supervisor & Runtime Management (Phase 06)
- **Objective:** Mengembangkan modul *Process Supervisor* di Hime Core untuk mengelola peluncuran, pemantauan kesehatan, kuota sumber daya, dan terminasi proses anak secara deterministik.
- **Prerequisites:** WS-2 (Hime Core skeleton).
- **Expected Deliverables:**
  * Modul `supervisor` pada `hime-core`.
  * Integrasi Win32 Job Object dengan flag `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE` (**SEC-08**).
  * Generator token acak CSPRNG 32-karakter dan injeksi `INITIAL_PASSWORD` via environment block (**SEC-03**).
  * Modul verifikasi socket runtime aktif IPv4/IPv6 (**SEC-04** / RSK-12).
- **Technical Dependencies:** Crates: `windows-sys` (API Job Object, GetExtendedTcpTable, TerminateProcess), `ring` (CSPRNG).
- **Security Requirements:** **SEC-03**, **SEC-04**, **SEC-08**. Kegagalan socket verifier wajib *fail-closed*.
- **Acceptance Criteria:** Supervisor mampu mematikan seluruh sub-proses seketika saat proses induk di-kill; socket verifier mendeteksi pelanggaran `0.0.0.0` dalam <500ms dan mematikan child process.
- **Blocking Decisions:** DEC-SEC-05 (Implementasi Active Runtime Socket Verification).
- **Suggested Order:** Urutan ke-3.

---

### WS-4: 9Router Integration & Security Remediation (Phases 06 & 08)
- **Objective:** Mengintegrasikan gateway model 9Router sebagai background daemon terkelola, menerapkan remedi RSK-10, mengunci direktori data, dan menyiapkan rute REST loopback.
- **Prerequisites:** WS-3 (Supervisor siap); Node.js v24 terpasang.
- **Expected Deliverables:**
  * Skrip peluncuran dan pembungkus daemon 9Router di bawah Supervisor.
  * Penerapan pengerasan izin NTFS ACL pada `%APPDATA%\9router` (**SEC-01**).
  * Penerapan patch penghapusan short-circuit loopback auth pada `src/dashboardGuard.js` (sesuai keputusan DEC-SEC-03).
  * Modul HTTP client di Hime Core untuk komunikasi loopback aman.
- **Technical Dependencies:** Node.js v24, npm/pnpm, 9Router source code (`RE/9ROUTER/00-source-materials/`).
- **Security Requirements:** **SEC-01**, **SEC-03**, **SEC-04**, mitigasi RSK-03, RSK-08, RSK-10, RSK-12.
- **Acceptance Criteria:** 9Router hanya mendengarkan pada `127.0.0.1:20128`; akses endpoint tanpa otentikasi ditolak; kredensial dilindungi NTFS ACL.
- **Blocking Decisions:** DEC-SEC-01 (Opsi SQLite storage), DEC-SEC-03 (Persetujuan patch RSK-10).
- **Suggested Order:** Urutan ke-4.

---

### WS-5: Hime Desktop & Dynamic Island HUD (Phase 07)
- **Objective:** Membangun antarmuka pengguna desktop melayang (*Dynamic Island HUD*) menggunakan Tauri 2.0 (Rust shell + WebView2) dengan perenderan DirectComposition dan klik-tembus transparan.
- **Prerequisites:** WS-2 (Hime Core); Tauri CLI v2 terpasang.
- **Expected Deliverables:**
  * Crate `hime-desktop` (Tauri 2.0 app).
  * Jendela borderless transparan dengan window styles Win32 (`WS_EX_LAYERED`, `WS_EX_TOOLWINDOW`, `WS_EX_TOPMOST`, `WS_EX_NOACTIVATE`).
  * Penanganan multi-monitor DPI scaling (`WM_DPICHANGED`) memitigasi RSK-05.
  * Antarmuka visual Dynamic Island (React/Tailwind) dengan animasi halus 60 FPS dan dialog persetujuan tindakan destruktif (*Approval Modal*).
  * Klien IPC Named Pipe terotentikasi (**SEC-05**).
- **Technical Dependencies:** Tauri 2.0, WebView2 Runtime (v157+ terpasang), DirectComposition API.
- **Security Requirements:** **SEC-05** (Named Pipe security token), isolasi konteks WebView2 (`tauri-plugin-single-instance`).
- **Acceptance Criteria:** Island melayang di bagian atas layar; transisi state (collapsed, expanded, thinking, toast) berjalan mulus tanpa visual artifact; memori idle <100 MB.
- **Blocking Decisions:** DEC-ARCH-01 (Batas clean-room Nusa).
- **Suggested Order:** Urutan ke-5.

---

### WS-6: Hermes Agent Integration (Phase 08)
- **Objective:** Mengintegrasikan runtime penalaran otonom Hermes Agent (Python 3.13) di bawah pengawasan Hime Supervisor, memediasi pemanggilan tool, dan mengalirkan token streaming ke Desktop HUD.
- **Prerequisites:** WS-3 (Supervisor); WS-4 (9Router); Python 3.13 & uv terpasang.
- **Expected Deliverables:**
  * Modul jembatan sub-proses Hermes di Hime Core via Named Pipe.
  * Konfigurasi profil Hermes yang menonaktifkan `HERMES_YOLO_MODE` secara permanen.
  * Adapter interceptor tool: Hermes mengalihkan seluruh pemanggilan terminal/filesystem ke Hime Core (**SEC-06**).
  * Pipeline streaming relay token inferensi (9Router -> Hermes -> Core -> Desktop HUD).
- **Technical Dependencies:** Python 3.13, Astral uv 0.12, Hermes Agent (`RE/Hermes/00-source-materials/hermes-agent`).
- **Security Requirements:** **SEC-06** (Central Policy Gate), **SEC-08** (Memory quota 1024MB), isolasi env var `local_env_policy.py`.
- **Acceptance Criteria:** Hermes menerima tugas pengguna, melakukan penalaran, memanggil tool via Hime Core, dan seluruh token teralirkan real-time ke UI; eksekusi perintah berbahaya dicegat oleh konfirmasi pengguna.
- **Blocking Decisions:** Keputusan persetujuan integrasi tool mediation.
- **Suggested Order:** Urutan ke-6.

---

### WS-7: Storage Intelligence & Conversation Persistence (Phases 06 & 10)
- **Objective:** Menyediakan penyimpanan percakapan lokal, konteks memori, dan pengindeksan teks penuh berbasis SQLite + FTS5 terenkripsi.
- **Prerequisites:** WS-2 (Hime Core).
- **Expected Deliverables:**
  * Modul `storage` di Hime Core menggunakan `rusqlite`.
  * Sistem migrasi skema SQL berversi (**DEC-DATA-01**).
  * Virtual table SQLite FTS5 untuk pencarian riwayat interaksi secepat kilat.
  * Master Secret Vault menggunakan Windows DPAPI (**SEC-02**).
- **Technical Dependencies:** Crate `rusqlite` (fitur `bundled` atau sistem v3.50.4 dengan ekstensi FTS5).
- **Security Requirements:** **SEC-01** (NTFS ACL pada database file), **SEC-02** (DPAPI CurrentUser + entropy).
- **Acceptance Criteria:** Percakapan tersimpan deterministik; pencarian FTS5 mengembalikan hasil dalam <15ms; file database terproteksi dari akun lain.
- **Blocking Decisions:** DEC-DATA-01 (Pustaka migrasi basis data).
- **Suggested Order:** Urutan ke-7.

---

### WS-8: Browser & Workspace Tool Workflows (Phase 13)
- **Objective:** Menyediakan kemampuan otomasi browser dan inspeksi dokumen secara aman tanpa mengekspos host ke risiko navigasi web tak terpercaya.
- **Prerequisites:** WS-6 (Hermes siap).
- **Expected Deliverables:**
  * Modul browser sandbox Chromium (CDP wrapper) dengan flag isolasi (`--disable-web-security=false`, block `file://`).
  * Sandbox sistem berkas workspace dengan *path allowlist* dan anti-traversal (**SEC-07**).
  * Penolak unduhan runtime dinamis 9Router (**RSK-09**).
- **Technical Dependencies:** Chromium/Playwright-core, Crate `tempfile`.
- **Security Requirements:** **SEC-07** (Path canonicalization, hardline deny-list `.ssh`, `AppData`, `system32`).
- **Acceptance Criteria:** Operasi baca/tulis terbatas hanya pada direktori proyek yang diizinkan; upaya traversal `..\..` ditolak dengan galat `AccessDenied`.
- **Blocking Decisions:** Tidak ada.
- **Suggested Order:** Urutan ke-8.

---

### WS-9: Productivity & Task Management (Phase 12)
- **Objective:** Mengintegrasikan kemampuan manajemen tugas personal, ekstraksi to-do dari percakapan, kalender, dan ringkasan dokumen pengguna.
- **Prerequisites:** WS-6 (Hermes), WS-7 (Storage).
- **Expected Deliverables:**
  * Parser dokumen lokal (Markdown, PDF, Plaintext) non-destruktif.
  * Modul pelacak tugas (*Task Engine*) tersinkronisasi dengan SQLite.
- **Technical Dependencies:** Parser teks Rust/Python ringan.
- **Security Requirements:** Pembacaan dokumen hanya dengan persetujuan path eksplisit dari pengguna.
- **Acceptance Criteria:** Hime mampu mengekstrak to-do list dari konteks obrolan dan menyimpannya ke database lokal.
- **Blocking Decisions:** Tidak ada.
- **Suggested Order:** Urutan ke-9.

---

### WS-10: Android Device Bridge & Remote Sync (Phase 11)
- **Objective:** Membangun lapisan jembatan perangkat bergerak (*Hime Mobile*) menggunakan Android Native (Kotlin) dengan Foreground Service untuk sinkronisasi konteks dan kontrol jarak jauh.
- **Prerequisites:** WS-2 (Hime Core IPC & Network Broker siap); Android SDK terpasang.
- **Expected Deliverables:**
  * Proyek Android Native (Kotlin) di `apps/mobile/`.
  * Implementasi Android Foreground Service dengan notifikasi persisten memitigasi **RSK-06** (Doze mode eviction).
  * Saluran komunikasi terenkripsi TLS lokal / WebSocket terotentikasi antara Android dan Hime Core.
- **Technical Dependencies:** Kotlin, Android Jetpack, OkHttp, WebSockets.
- **Security Requirements:** Handshake mutual authentication dengan pinning sertifikat lokal; tidak ada transmisi teks polos di LAN.
- **Acceptance Criteria:** Perangkat Android dapat menampilkan status Dynamic Island secara remote dan mengirim prompt suara/teks ke desktop tanpa terputus di background.
- **Blocking Decisions:** Tidak ada.
- **Suggested Order:** Urutan ke-10.

---

### WS-11: Testing, Hardening, Packaging & Release (Phases 14 & 15)
- **Objective:** Menjalankan audit keamanan penetrasi final, optimasi performa memori, penandatanganan biner Authenticode, pembuatan installer Windows MSI/EXE, dan mekanisme update deterministik.
- **Prerequisites:** Seluruh workstream sebelumnya selesai.
- **Expected Deliverables:**
  * Paket installer Windows (`.msi` / `.exe`) via WiX / NSIS terintegrasi Tauri bundler.
  * Penandatanganan biner Authenticode Code Signing (**SEC-10**).
  * Sistem pembaruan otomatis HTTPS bertanda tangan Ed25519 dengan rollback otomatis.
  * Laporan uji beban RAM dan benchmark visual 60 FPS.
- **Technical Dependencies:** Windows SDK `signtool.exe`, Tauri bundler.
- **Security Requirements:** **SEC-10**, validasi hash SHA-256 rilis, proteksi rantai pasok.
- **Acceptance Criteria:** Installer terpasang bersih tanpa peringatan SmartScreen mencurigakan; update client memverifikasi signature sebelum unpacking; rollback berhasil jika simulasi update rusak.
- **Blocking Decisions:** Pengadaan sertifikat penandatanganan kode Authenticode.
- **Suggested Order:** Urutan ke-11 (Tahap Final).

---

## 4. Critical Path & Dependency Order Graph

```text
[Phase 01 Acceptance] (Sekarang)
         │
         ▼
[Phase 02 & 03 Formal Acceptance]
         │
         ▼
[WS-1: Specs & Architecture Design (Phases 04–05)] <── [DEC-SEC-03 Blocker Resolved]
         │
         ├─────────────────────────────────────────┐
         ▼                                         ▼
[Toolchain Rust Installed (DEC-TOOL-01)]    [9Router Upstream Patch Prepared]
         │                                         │
         ▼                                         │
[WS-2: Hime Core Foundation (Phase 06)]            │
         │                                         │
         ▼                                         │
[WS-3: Process Supervisor (Phase 06)] ─────────────┤
         │                                         ▼
         ├──────────────────────────────> [WS-4: 9Router Integration (Phase 06)]
         ▼                                         │
[WS-5: Hime Desktop & HUD (Phase 07)]              │
         │                                         ▼
         └──────────────────────────────> [WS-6: Hermes Agent Integration (Phase 08)]
                                                   │
                                                   ▼
                                          [WS-7: Storage & Memory (Phase 09/10)]
                                                   │
                                                   ▼
                                          [WS-8: Browser & Workspaces (Phase 13)]
                                                   │
                                                   ▼
                                          [WS-9: Productivity Engine (Phase 12)]
                                                   │
                                                   ▼
                                          [WS-10: Android Device Bridge (Phase 11)]
                                                   │
                                                   ▼
                                          [WS-11: Hardening & Release (Phase 14–15)]
```

---

## 5. Summary of Blocking Decisions Before Execution

| Workstream | Keputusan Penghambat (*Blocking Decision*) | Tindakan Wajib yang Dibutuhkan |
|:---|:---|:---|
| **WS-1 / WS-4** | **DEC-SEC-03 (Remediasi RSK-10)** | Project Owner memilih opsi remedi loopback auth bypass (Fork vs WFP) sebelum integrasi 9Router diimplementasikan. |
| **WS-2 s/d WS-5**| **DEC-TOOL-01 (Rust Compiler Installation)** | Pengguna menginstal Rust (`rustup-init.exe`) dan Visual Studio Build Tools sebelum Phase 06 dimulai. |
| **WS-1** | **DEC-ARCH-03 (Modular Monolith Pattern)** | Formalisasi ADR-001 pada Phase 05 disetujui Project Owner. |
| **WS-11** | **Authenticode Certificate Acquisition** | Pengadaan kredensial penandatanganan biner sebelum distribusi biner rilis. |
