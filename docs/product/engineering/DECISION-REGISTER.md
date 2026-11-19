# Hime Assistant — Consolidated Technical & Security Decision Register

- **Document ID:** DECISION-REGISTER
- **Phase:** Phase 01 — Discovery & Technical Validation
- **Authority:** System Architect & Security Specialist
- **Status:** **CANONICAL DECISION TRACKER**
- **Date:** 2026-11-08

---

## 1. Governance Overview & Status Definitions

Dokumen ini melacak seluruh keputusan arsitektural, teknis, tata kelola, dan keamanan dalam ekosistem Hime Assistant. Dokumen ini secara tegas membedakan keputusan yang telah disetujui secara formal dari proposal yang masih membutuhkan tinjauan atau keputusan persetujuan dari **Project Owner**.

### Definisi Status Keputusan:

| Status | Definisi Tata Kelola | Wewenang Pengubah Status |
|:---|:---|:---:|
| **APPROVED** | Keputusan telah disetujui secara formal oleh Project Owner dan menjadi dasar implementasi. | Project Owner |
| **PROPOSED** | Proposal teknis yang dirumuskan oleh tim/agen, menunggu evaluasi atau formalisasi ADR. | AI Agent / Architect |
| **PENDING OWNER DECISION** | Keputusan kritis (terutama terkait risiko residu dan keamanan) yang wajib diputuskan oleh Project Owner. | Project Owner |
| **BLOCKED** | Keputusan atau alur kerja terhambat oleh ketergantungan eksternal, risiko keamanan belum termitigasi, atau ketiadaan keputusan. | Project Owner / Security Architect |
| **DEFERRED** | Pembahasan keputusan ditunda secara sengaja ke fase pengembangan berikutnya yang lebih relevan. | Project Owner |

---

## 2. Decision Traceability Matrix

| Decision ID | Judul Keputusan | Domain | Status Keputusan | Memblokir Implementasi? | Target Fase |
|:---:|:---|:---:|:---:|:---:|:---:|
| **DEC-SEC-01** | Penanganan Kredensial Teks Polos 9Router SQLite (RSK-03) | Security | **PENDING OWNER DECISION** | TIDAK (Phase 01–04)<br>YA (Phase 06 Multi-User) | Phase 05/06 |
| **DEC-SEC-02** | Pembatasan Otorisasi Luas Token Mesin CLI 9Router (RSK-08) | Security | **PENDING OWNER DECISION** | TIDAK | Phase 05/06 |
| **DEC-SEC-03** | Remediasi Bypass Autentikasi Loopback LLM API 9Router (RSK-10) | Security | **BLOCKED & PENDING OWNER DECISION** | **YA (IMPLEMENTATION BLOCKER)** | Phase 05/06 |
| **DEC-SEC-04** | Kualifikasi Teknis & Risiko Residu Akses Memori Satu Pengguna | Security | **PENDING OWNER DECISION** | TIDAK | Phase 05 |
| **DEC-SEC-05** | Implementasi Active Runtime Socket Verification IPv4/IPv6 (RSK-12) | Security | **PROPOSED (PLANNED CONTROL)** | TIDAK | Phase 06 |
| **DEC-ARCH-01** | Clean-Room Reimplementation Nusa & Batasan Aset Proprietary | Architecture | **APPROVED** | TIDAK | Phase 02/07 |
| **DEC-ARCH-02** | Topologi Dual-Control Plane & Runtime Decoupling | Architecture | **APPROVED** | TIDAK | Phase 05/06 |
| **DEC-ARCH-03** | Arsitektur Hime Core: Modular Monolith dengan Ports & Adapters | Architecture | **PROPOSED (PENDING ADR)** | TIDAK | Phase 05 |
| **DEC-ARCH-04** | Strategi Inferensi AI Awal: Cloud AI vs Local Model | Architecture | **APPROVED** | TIDAK | Phase 01/08 |
| **DEC-ARCH-05** | Protokol IPC: Windows Named Pipes dengan PipeSecurity DACL | Architecture | **PROPOSED (PENDING ADR)** | TIDAK | Phase 05/07 |
| **DEC-TOOL-01** | Instalasi Toolchain Rust & Visual Studio Build Tools | Toolchain | **APPROVED** | YA (Hanya untuk Phase 06+) | Phase 06 |
| **DEC-DATA-01** | Strategi Migrasi Skema Basis Data Percakapan SQLite | Data | **PROPOSED (PENDING ADR)** | TIDAK | Phase 06/10 |

---

## 3. Detailed Decision Records

---

### DEC-SEC-01: Penanganan Kredensial Teks Polos 9Router SQLite (RSK-03)
- **Decision ID:** DEC-SEC-01
- **Domain:** Security / Data Persistence
- **Konteks:** 9Router v0.5.112 menyimpan API key dan token OAuth penyedia AI dalam bentuk teks polos JSON di berkas SQLite `%APPDATA%\9router\db\data.sqlite` (CWE-312).
- **Opsi yang Tersedia:**
  1. *Opsi A (Defense-in-Depth OS):* Mengandalkan pengerasan izin direktori NTFS ACL (`icacls /inheritance:r`) dan enkripsi diska EFS (`cipher /e`) pada Windows 11 Pro, dipadukan dengan Master DPAPI Vault pada Hime Core. Menerima risiko residu terhadap proses lain dalam akun pengguna yang sama.
  2. *Opsi B (Upstream Storage Patch):* Mengembangkan patch pada 9Router untuk mendukung penyimpanan *in-memory* atau enkripsi kolom SQL sebelum disimpan ke diska.
- **Trade-off Teknis:** Opsi A tidak memerlukan modifikasi kode 9Router namun menyisakan celah terhadap infostealer di akun yang sama. Opsi B memberikan keamanan penuh tetapi membutuhkan pemeliharaan fork/patch 9Router.
- **Dampak Keamanan:** HIGH (potensi kebocoran API key jika host terinfeksi malware).
- **Rekomendasi Tim:** Terapkan Opsi A sebagai baseline di Phase 06, evaluasi kelayakan Opsi B pada perancangan arsitektur sistem Phase 05.
- **Status:** **PENDING OWNER DECISION** (Usulan Risiko Residu: Keputusan di tangan Project Owner).
- **Memblokir Implementasi:** TIDAK untuk Phase 01–05; YA sebelum rilis produksi multi-user.

---

### DEC-SEC-02: Pembatasan Otorisasi Luas Token Mesin CLI 9Router (RSK-08)
- **Decision ID:** DEC-SEC-02
- **Domain:** Security / IPC & Authorization
- **Konteks:** Token `x-9r-cli-token` diturunkan dari `auth/cli-secret` dan `MachineGuid`, meloloskan akses ke seluruh rute sensitif `/api/*` (termasuk ekspor database dan shutdown) tanpa masa kedaluwarsa atau pembatasan granular.
- **Opsi yang Tersedia:**
  1. *Opsi A (Isolasi File & Penolakan Penggunaan Eksternal):* Kunci direktori `auth/` dengan NTFS ACL ketat; Hime Core mengelola daur hidup 9Router via sinyal proses OS langsung dan **TIDAK MENGGUNAKAN** rute administrasi token CLI untuk operasional normal.
  2. *Opsi B (Upstream Token Revocation Patch):* Menambahkan middleware rotasi token dan pemeriksaan hak akses granular pada 9Router.
- **Trade-off Teknis:** Opsi A sepenuhnya menghindari penggunaan token yang berisiko; Opsi B memerlukan maintenance overhead pada repositori eksternal.
- **Dampak Keamanan:** HIGH jika token jatuh ke proses lokal tak berizin.
- **Rekomendasi Tim:** Terapkan Opsi A (Hime Supervisor tidak bergantung pada token CLI).
- **Status:** **PENDING OWNER DECISION**.
- **Memblokir Implementasi:** TIDAK.

---

### DEC-SEC-03: Remediasi Bypass Autentikasi Loopback LLM API 9Router (RSK-10)
- **Decision ID:** DEC-SEC-03
- **Domain:** Security / API Gateway Authorization
- **Konteks:** Pada 9Router v0.5.112, `src/dashboardGuard.js:161-165` mengecualikan seluruh request loopback via `if (isLocalRequest(request)) return true;`. Setiap proses lokal dapat langsung memanggil endpoint inferensi `/v1/chat/completions` tanpa otentikasi, memotong Hime Core dan menguras kuota pengguna. Loopback bukan autentikasi, dan token virtual tidak diperiksa.
- **Opsi yang Tersedia:**
  1. *Opsi A (Upstream PR / Maintained Fork):* Memodifikasi `src/dashboardGuard.js` pada 9Router untuk menghapus pengecualian loopback dan mewajibkan otentikasi header untuk semua request tanpa terkecuali.
  2. *Opsi B (Isolasi Akun Service OS & Firewall WFP):* Menjalankan 9Router di bawah akun service lokal Windows terpisah, dipadukan dengan aturan Windows Filtering Platform (WFP) yang hanya mengizinkan PID Hime Core untuk membuka port 20128.
  3. *Opsi C (Reverse Proxy Internal Terotentikasi):* Hime Core bertindak sebagai reverse proxy lokal ketat, namun 9Router tetap harus diikat ke port acak yang diisolasi.
- **Trade-off Teknis:** Opsi A adalah solusi paling bersih secara arsitektur namun membutuhkan fork/patch. Opsi B rumit dalam instalasi desktop konsumen.
- **Dampak Keamanan:** **CRITICAL/HIGH** (Eksfiltrasi inferensi tak berbayar, bypass kontrol kebijakan Hime).
- **Rekomendasi Tim:** Opsi A (Maintained fork minimal untuk 9Router di Phase 05/06).
- **Status:** **BLOCKED & PENDING OWNER DECISION**.
- **Memblokir Implementasi:** **YA (IMPLEMENTATION BLOCKER)** sebelum Hime dapat mengandalkan 9Router untuk operasional komersial, multi-tenant, atau kuota sensitif.

---

### DEC-SEC-04: Kualifikasi Teknis & Risiko Residu Akses Memori Satu Pengguna
- **Decision ID:** DEC-SEC-04
- **Domain:** Security / Windows Process Isolation
- **Konteks:** Evaluasi apakah malware yang berjalan dalam konteks pengguna Windows yang sama (`XATOSHI\Satoshi` Medium Integrity) dapat mengekstrak secret Hime dari memori proses.
- **Kualifikasi Teknis Terkonfirmasi:** Membaca memori proses (`PROCESS_VM_READ` 0x0010 dan `PROCESS_QUERY_INFORMATION` 0x0400) dievaluasi oleh Windows Security Reference Monitor terhadap DACL objek proses dan token pemanggil. Hime Core dapat memperketat DACL objek prosesnya sendiri via `SetKernelObjectSecurity` dan mengaktifkan mitigasi proses `PROCESS_MITIGATION_DYNAMIC_CODE_POLICY`. Namun, DPAPI, NTFS ACL, dan zeroization sendiri tidak memberikan isolasi proses yang absolut.
- **Opsi yang Tersedia:**
  1. *Opsi A (Standard Process Hardening):* Menerapkan DACL proses restriktif, mitigasi exploit Windows, dan memory zeroization via crate `zeroize`. Menerima risiko residu terhadap malware canggih yang berhasil melewati DACL.
  2. *Opsi B (AppContainer Sandbox / Virtualization):* Mengisolasi Hime Desktop dan komponen ke dalam AppContainer sandbox Windows.
- **Trade-off Teknis:** Opsi B menambah kompleksitas interaksi file lokal dan instalasi desktop.
- **Dampak Keamanan:** MEDIUM/HIGH.
- **Rekomendasi Tim:** Opsi A untuk desktop v1.0, dengan dokumentasi risiko residu transparan.
- **Status:** **PENDING OWNER DECISION**.
- **Memblokir Implementasi:** TIDAK.

---

### DEC-SEC-05: Implementasi Active Runtime Socket Verification IPv4/IPv6 (RSK-12)
- **Decision ID:** DEC-SEC-05
- **Domain:** Security / Network Verification
- **Konteks:** 9Router memiliki kode bawaan `DEFAULT_HOST = "0.0.0.0"` (`cli/cli.js:110`). Mitigasi mewajibkan parameter `--host 127.0.0.1` disertai verifikasi aktif socket listening oleh Hime Process Supervisor.
- **Opsi yang Tersedia:**
  1. *Opsi A (Active Dual-Stack Socket Verification + Job Object Kill):* Supervisor memeriksa tabel TCP IPv4 (`AF_INET`) dan IPv6 (`AF_INET6`) via Win32 `GetExtendedTcpTable`, mencocokkan port, dan memastikan `dwOwningPid` cocok dengan child PID 9Router. Tolak startup (*fail-closed*) jika terikat pada `0.0.0.0`, LAN IP, atau PID mismatch.
  2. *Opsi B (Asumsi Parameter CLI):* Hanya mengandalkan argumen `--host 127.0.0.1` tanpa verifikasi runtime.
- **Trade-off Teknis:** Opsi A memerlukan penulisan modul inspeksi socket Win32 pada Rust supervisor (~150 baris kode), namun menjamin 100% ketiadaan paparan LAN tak sengaja.
- **Dampak Keamanan:** HIGH jika port 20128 bocor ke LAN (Wi-Fi/Ethernet publik).
- **Rekomendasi Tim:** Opsi A (Wajib diimplementasikan pada Phase 06).
- **Status:** **PROPOSED (PLANNED CONTROL)** — Status efektivitas saat ini: **`[NOT YET VERIFIED]`** hingga Phase 06.
- **Memblokir Implementasi:** TIDAK.

---

### DEC-ARCH-01: Clean-Room Reimplementation Nusa & Batasan Aset Proprietary
- **Decision ID:** DEC-ARCH-01
- **Domain:** Legal / Architecture
- **Konteks:** Nusa Dynamic Island v1.2.0.0 adalah aplikasi freeware proprietary berhak cipta Rama tanpa kode sumber terbuka.
- **Keputusan:** Hime Desktop adalah reimplementasi bersih (*clean-room reimplementation*) menggunakan Tauri 2.0 (Rust) + Web Frontend. Seluruh aset biner asli Nusa diisolasi di `RE/Nusa/` sebagai bahan referensi riset dan tidak didistribusikan ke dalam produk rilis Hime.
- **Status:** **APPROVED**.
- **Memblokir Implementasi:** TIDAK.

---

### DEC-ARCH-02: Topologi Dual-Control Plane & Runtime Decoupling
- **Decision ID:** DEC-ARCH-02
- **Domain:** Architecture / Process Management
- **Konteks:** Cara pengintegrasian 9Router dan Hermes Agent ke dalam ekosistem Hime.
- **Keputusan:** Menetapkan topologi **Dual-Control Plane**: Hime Core mengontrol daur hidup proses melalui Windows Job Objects dan Named Pipes (Control Plane 1), sementara inferensi token dialirkan langsung antara Hermes dan 9Router via loopback HTTP `/v1` (Control Plane 2). Runtime Rust, Python, dan Node.js tetap terpisah dalam proses independen.
- **Status:** **APPROVED**.
- **Memblokir Implementasi:** TIDAK.

---

### DEC-ARCH-03: Arsitektur Hime Core: Modular Monolith dengan Ports & Adapters
- **Decision ID:** DEC-ARCH-03
- **Domain:** Architecture / System Design
- **Konteks:** Menentukan struktur internal Hime Core (Rust) untuk menjaga maintainability dan mencegah kompleksitas prematur.
- **Proposal:** Mengadopsi pola **Modular Monolith** dengan batasan domain yang jelas (Session, Policy, Memory, Tool, Supervisor) menggunakan prinsip **Clean Architecture / Ports and Adapters**. Setiap subsistem eksternal diakses melalui trait Rust.
- **Status:** **PROPOSED (PENDING FORMAL ADR APPROVAL DI PHASE 05)**.
- **Memblokir Implementasi:** TIDAK.

---

### DEC-ARCH-04: Strategi Inferensi AI Awal: Cloud AI vs Local Model
- **Decision ID:** DEC-ARCH-04
- **Domain:** Architecture / AI Infrastructure
- **Konteks:** Kapasitas perangkat keras host (Intel Core i5-8250U, iGPU Intel UHD 620, free RAM ~3.71 GB) tidak memungkinkan eksekusi model bahasa besar lokal secara layak.
- **Keputusan:** Ekosistem Hime menetapkan model **Cloud AI** (via OpenAI, Anthropic, DeepSeek, Google Gemini) sebagai pendekatan inferensi awal. Inferensi lokal ditunda hingga fase lanjutan dan hanya untuk perangkat dengan GPU diskret.
- **Status:** **APPROVED**.
- **Memblokir Implementasi:** TIDAK.

---

### DEC-ARCH-05: Protokol IPC: Windows Named Pipes dengan PipeSecurity DACL
- **Decision ID:** DEC-ARCH-05
- **Domain:** Architecture / IPC
- **Konteks:** Menentukan media komunikasi antar-proses lokal berkecepatan tinggi antara Hime Desktop, Hime Core, dan Hermes Agent.
- **Proposal:** Menggunakan **Windows Named Pipes** dengan `SECURITY_ATTRIBUTES` terkonfigurasi DACL eksplisit hanya untuk SID pengguna aktif, dilengkapi token handshake 256-bit sekali pakai. Protokol berbasis framing panjang biner dengan payload JSON/Protobuf.
- **Status:** **PROPOSED (PENDING FORMAL ADR APPROVAL DI PHASE 05)**.
- **Memblokir Implementasi:** TIDAK.

---

### DEC-TOOL-01: Instalasi Toolchain Rust & Visual Studio Build Tools
- **Decision ID:** DEC-TOOL-01
- **Domain:** Development Toolchain
- **Konteks:** Host belum memiliki compiler Rust (`rustc`/`cargo`) dan linker MSVC C++.
- **Keputusan:** Ketiadaan Rust compiler ditegaskan **bukan penghambat (non-blocker) untuk Phase 01–05**. Pengguna akan menginstal Rust via `rustup-init.exe` bersama Visual Studio C++ Build Tools sebelum implementasi kode desktop pada Phase 06 dimulai.
- **Status:** **APPROVED**.
- **Memblokir Implementasi:** YA (Hanya saat memasuki Phase 06).

---

### DEC-DATA-01: Strategi Migrasi Skema Basis Data Percakapan SQLite
- **Decision ID:** DEC-DATA-01
- **Domain:** Data Architecture
- **Konteks:** Pemilihan mekanisme migrasi skema SQL untuk basis data percakapan dan memori Hime Core.
- **Proposal:** Menggunakan pustaka migrasi berbasis Rust terkompilasi (misal: `refinery` atau `rusqlite_migration`) dengan berkas SQL berversi (`V1__initial.sql`, `V2__fts5_index.sql`) yang tersemat (*embedded*) di dalam biner Hime Core.
- **Status:** **PROPOSED (PENDING FORMAL ADR APPROVAL DI PHASE 05)**.
- **Memblokir Implementasi:** TIDAK.
