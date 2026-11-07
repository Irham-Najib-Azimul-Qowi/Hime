# Hime Assistant — Engineering Foundation Reference Document

- **Document ID:** ENGINEERING-FOUNDATION
- **Phase:** Phase 01 — Discovery & Technical Validation
- **Authority:** Senior Engineering Infrastructure Architect & System Architect
- **Status:** **CANONICAL ENGINEERING BASELINE**
- **Date:** 2026-11-08

---

## 1. Executive Vision & Foundational Context

Hime Assistant adalah ekosistem asisten kecerdasan buatan (*personal AI ecosystem*) yang beroperasi lintas perangkat desktop (Windows 11) dan mobile (Android). Ekosistem ini menggabungkan antarmuka *Dynamic Island* melayang (*floating HUD*) yang terinspirasi dari estetika Nusa dengan *runtime* agen AI otonom yang tangguh (Hermes Agent) serta gateway perutean model AI lokal multi-penyedia (9Router).

Dokumen ini berfungsi sebagai **rujukan tunggal kanonikal (*single source of truth*)** yang mengonsolidasikan seluruh temuan penemuan (*discovery findings*), batasan lingkungan host, profil aset, arsitektur sistem, baseline keamanan (SEC-01 s/d SEC-10), dan standar rekayasa yang telah divalidasi sepanjang pelaksanaan Phase 01.

---

## 2. Environment Baseline & Hardware Capacity

Seluruh parameter teknis lingkungan host Windows 11 telah divalidasi secara empiris pada TASK-01-01.

### A. Kapasitas Perangkat Keras Host

| Komponen Perangkat Keras | Spesifikasi Terverifikasi | Klasifikasi Bukti | Implikasi Rekayasa |
|:---|:---|:---:|:---|
| **Sistem Operasi** | Windows 11 Pro 64-bit (Build 26100.2033) | `[VERIFIED]` | Target platform desktop utama; mendukung DPAPI, Job Objects, Named Pipes, DirectComposition, NTFS ACL, EFS. |
| **Prosesor (CPU)** | Intel Core i5-8250U (4 Cores, 8 Threads, 1.60 GHz base, up to 3.40 GHz boost) | `[VERIFIED]` | Kapasitas komputasi menengah. **Tidak memadai untuk inferensi LLM lokal secara mandiri**. |
| **Memori Fisik (RAM)** | 20,774 MB (~20.8 GB) DDR4 terpasang | `[VERIFIED]` | Cukup untuk menjalankan tumpukan multi-proses Hime + toolchain pengembangan. |
| **Ruang Bebas RAM Terukur** | **~3.71 GB** bebas pada saat audit runtime | `[VERIFIED]` | **Batasan Nyata:** Alokasi memori Hime harus sangat efisien; dilarang terjadi memory leak antar-proses. |
| **Unit Pengolah Grafis (GPU)** | Intel UHD Graphics 620 (VRAM Terbagi, Direct3D 12 Feature Level 12_1) | `[VERIFIED]` | Mendukung akselerasi DirectComposition & WebView2. Tidak mendukung akselerasi komputasi AI kuantisasi (CUDA/ROCm). |
| **Penyimpanan Diska** | Drive X: ~62.0 GB bebas (Lokasi Repo); Drive C: ~41.0 GB bebas | `[VERIFIED]` | Memadai untuk runtime, cache model cloud, dan SQLite database. |

### B. Kesiapan Perangkat Lunak & Toolchain Pengembangan

| Perkakas / Runtime | Versi Terverifikasi | Status Kesiapan | Peran dalam Ekosistem Hime |
|:---|:---|:---:|:---|
| **Git & GitHub CLI** | `git` v2.55.0.windows.3, `gh` v2.102.0 | `[VERIFIED]` | Pengendali versi utama & otomatisasi GitHub. |
| **Node.js, npm, pnpm** | `node` v24.18.0, `npm` 11.16.0, `pnpm` 12.14.0 | `[VERIFIED]` | Runtime resmi untuk menjalankan 9Router AI Gateway & tooling web. |
| **Python & Astral uv** | `python` 3.13.16, `uv` 0.12.16 | `[VERIFIED]` | Runtime resmi untuk Hermes Agent; eksekusi virtualenv ultra-cepat. |
| **SQLite3 & Modul FTS5** | SQLite v3.50.4 (FTS5 terverifikasi aktif via Python `sqlite3`) | `[VERIFIED]` | Penyimpanan percakapan lokal, pengindeksan teks penuh (*full-text search*). |
| **WebView2 Runtime** | v157.0.4341.42 (Built-in Windows 11) | `[VERIFIED]` | Mesin perender UI untuk Tauri 2.0 shell pada Hime Desktop. |
| **PowerShell** | PS 5.1 (Built-in) & PS 7.5.0 Core | `[VERIFIED]` | Shell otomatisasi scripting diagnostik dan manajemen proses. |
| **Rust Compiler & Cargo** | `rustc` / `cargo` (Belum Terpasang) | `[MISSING / PREREQUISITE]` | **Non-blocker untuk Phase 01–05**; Prasyarat mutlak sebelum Phase 06 (Hime Core/Tauri). |
| **Visual Studio Build Tools**| MSVC C++ x64 linker (Belum Terpasang) | `[MISSING / PREREQUISITE]` | Diinstal bersamaan dengan toolchain Rust via `rustup-init.exe`. |

### C. Kualifikasi Realistis: Nilai Terukur vs Target Estimasi

Untuk menjaga integritas rekayasa (*evidence-first*), batas antara bukti empiris dan usulan target ditetapkan secara tegas:
1. **Strategi Inferensi AI:** Model inferensi **Cloud AI** (OpenAI, Anthropic, DeepSeek, Google Gemini) ditetapkan sebagai strategi inferensi awal. Penggunaan model lokal (misal: Llama 8B via Ollama/GGUF) **TIDAK VIABLE** pada perangkat keras i5-8250U tanpa GPU diskret.
2. **Anggaran Alokasi Memori (Target Desain vs Benchmark):**
   - Target alokasi Hime Desktop Shell (Tauri 2.0 Rust idle): `<100 MB` RAM.
   - Target alokasi 9Router background daemon: `~150–300 MB` RAM.
   - Target alokasi Hermes Agent worker: `~200–400 MB` RAM.
   - Angka-angka di atas adalah **target desain arsitektur (*proposed budget*)**, bukan hasil uji beban empiris (*benchmark*). Uji memori aktual akan diukur pada Phase 06/07.
3. **Kinerja UI & Latensi IPC (Target Desain vs Benchmark):**
   - Target visual render DirectComposition: `60 FPS` tanpa jitter.
   - Target round-trip IPC Named Pipe: `<10 ms`.
   - Kapabilitas Windows DirectComposition dan Named Pipe telah terverifikasi tersedia, namun ketercapaian 60 FPS dan <10ms latensi adalah **target performa arsitektur**, bukan benchmark yang telah terbukti pada tahap penemuan ini.

---

## 3. Asset & Dependency Baseline

Konsolidasi aset dan audit dependensi dari TASK-01-02 dan TASK-01-03:

### A. Repositori Aset & Provenance Komponen

```text
Ekosistem Hime (Monorepo Workspace X:\folder_ai\Hime)
├── Nusa Dynamic Island v1.2.0.0 (Proprietary Reference — Clean-Room Reimplementation)
├── Hermes Agent (External Submodule/Vendor — commit c5f347eb — MIT License)
├── 9Router AI Gateway (External Submodule/Vendor — commit 1fe586bb — MIT License)
├── Curated Agent Skills (7 Pustaka Terbuka di .agent/skills — MIT License)
└── Hime Desktop & Hime Core (New Native Implementation in Rust + Tauri 2.0)
```

1. **Aplikasi Referensi Nusa Dynamic Island v1.2.0.0:**
   - Hak Cipta & Lisensi: Freeware proprietary karya Rama. Kode sumber **TIDAK TERSEDIA**.
   - Batasan Hukum & Rekayasa: Hime Desktop adalah **reimplementasi mandiri (*clean-room reimplementation*)**, bukan *fork* atau pembongkaran biner ilegal. Pengetahuan arsitektur (DirectComposition, window flags `WS_EX_LAYERED`, morphology states) diekstraksi dari riset reverse engineering di `RE/Nusa/`. Biner asli Nusa tidak didistribusikan ulang.
2. **Hermes Agent (Nous Research):**
   - Commit: `c5f347eb` (v0.7.0+). Lisensi: **MIT License** resmi.
   - Provenance: Repositori open-source resmi Nous Research.
   - Status Dependensi: 46 dependensi langsung di `pyproject.toml`. Terdapat `356 locked/resolved packages (direct + transitive)` pada `uv.lock`. Jumlah transitif murni berstatus **`NOT SEPARATELY COUNTED`** karena keterbatasan ekstraksi graph tanpa instalasi. Seluruh lisensi dependensi langsung terverifikasi kompatibel (MIT/Apache-2.0/BSD/PSF). Lisensi paket transitif belum diaudit individual (*UNVERIFIED*).
3. **9Router AI Gateway (decolua):**
   - Commit: `1fe586bb` (v0.5.112). Lisensi: **MIT License** resmi.
   - Provenance: Repositori open-source Next.js 16 monorepo.
   - Status Dependensi: 34 dependensi langsung teridentifikasi dari manifes (`package.json` dan `cli/package.json`). **Tidak memiliki lockfile** (`package-lock.json` atau `pnpm-lock.yaml`). Dependensi transitif belum terkunci dan belum diaudit (*UNVERIFIED*).
4. **Curated Agent Skills (7 Pustaka Terbuka):**
   - Lokasi: `.agent/skills/` (`animate`, `emil-design-eng`, `figcraft-design`, `ponytail`, `rea`, `systematic-debugging`, `verification-before-completion`).
   - Sifat: 100% instruksi rekayasa prosedural berbasis Markdown (SKILL.md) tanpa biner eksekusi. Lisensi kompatibel.
5. **Planned Dependencies Hime (9 Komponen Rencana Phase 06+):**
   - Tauri 2.0 (`tauri`), Tokio (`tokio`), Rusqlite (`rusqlite`), Ring (`ring`), Windows-sys (`windows-sys`), Serde (`serde`), Tracing (`tracing`), Zeroize (`zeroize`), Hyper (`hyper`). Seluruhnya berlisensi permisif (MIT / Apache-2.0).
6. **Kewajiban Distribusi & Lisensi Pihak Ketiga:**
   - Seluruh pemberitahuan hak cipta (*copyright notices*) dan lisensi MIT untuk Hermes Agent dan 9Router wajib disertakan dalam distribusi biner Hime (direktori `legal/` atau dialog About).
   - Tidak ada dependensi langsung berlisensi copyleft ketat (GPL/AGPL) yang teridentifikasi dalam ekosistem Hime.

---

## 4. System Architecture & Component Responsibilities

Arsitektur Hime Assistant menerapkan pola **Dual-Control Plane** dan pemisahan proses yang ketat (*Process Isolation*).

### A. Data Flow Diagram & Trust Boundaries

```text
[ External AI Providers / Web APIs ]
             ▲
             │ HTTPS (TLS 1.3)
             ▼
      +--------------+
      |   9Router    |  <-- [Trust Boundary 4: Local AI Gateway (UNTRUSTED API BOUNDARY)]
      +--------------+      (Runs under Node.js 24; port 20128; RSK-10 & RSK-12 apply)
             ▲
             │ Localhost HTTP (127.0.0.1:20128/v1 - OpenAI Compatible API)
             ▼
      +--------------+
      | Hermes Agent |  <-- [Trust Boundary 3: Autonomous Reasoning Runtime (SEMI-TRUSTED)]
      +--------------+      (Runs under Python 3.13; tool execution mediated by Core)
             ▲
             │ Authenticated Named Pipes (\\.\pipe\hime-hermes-{uuid})
             ▼
================== [ Trust Boundary 2: Central Policy Enforcement Gate ] ==================
|                                   HIME CORE                                             |
|  - Process Supervisor (Win32 Job Objects)     - Secret Vault (DPAPI + Entropy)          |
|  - Tool Permission Enforcement Gate           - Conversational Storage (SQLite + FTS5)  |
|  - Runtime Socket Verifier (IPv4/IPv6)        - Path Sandbox & Canonicalization         |
===========================================================================================
             ▲
             │ Authenticated Named Pipes / Tauri IPC
             ▼
      +--------------+
      | Hime Desktop |  <-- [Trust Boundary 1: User Interface / Presentation Shell]
      +--------------+      (Tauri 2.0 Rust + WebView2; DirectComposition HUD)
             ▲
             │ Direct User Interaction (Mouse / Keyboard / Hotkeys)
             ▼
       [ End User / Operator (TRUSTED AUTHORITY) ]
```

### B. Tanggung Jawab & Batas-Batas Komponen

| Komponen | Lingkungan Runtime | Tanggung Jawab Utama | Batasan Keamanan & Operasional |
|:---|:---|:---|:---|
| **Hime Desktop** | Tauri 2.0 (Rust Shell + WebView2) | - Merender antarmuka *Dynamic Island HUD* melayang.<br>- Menangani interaksi mouse/keyboard, hover morphing, dan klik-tembus (*hit-test transparency*).<br>- Menyajikan dialog konfirmasi persetujuan manusia (*Approval Modal*) yang non-bypassable. | - Semi-trusted boundary.<br>- Tidak menyimpan secret master.<br>- Bergantung pada Hime Core untuk seluruh eksekusi logika berat dan data sensitif. |
| **Hime Core** | Native Daemon (Rust) | - **Pusat Kebijakan & Otoritas Keamanan (Central Policy Gate)**.<br>- Menjalankan supervisi proses anak (Supervisor via Win32 Job Objects).<br>- Menjalankan Master Secret Vault (Windows DPAPI + Entropy).<br>- Memverifikasi kepatuhan listening socket (SEC-04).<br>- Membatasi akses sistem berkas (Sandboxing & Path Allowlist).<br>- Mengelola database percakapan & indeks pencarian SQLite FTS5. | - Trusted Core boundary.<br>- Berjalan pada konteks Medium Integrity `XATOSHI\Satoshi`.<br>- Menolak eksekusi shell destruktif tanpa konfirmasi pengguna. |
| **Hermes Agent** | Sub-proses Terpisah (Python 3.13 + Astral uv) | - Melakukan penalaran otonom (*autonomous reasoning*), perencanaan tugas (*task planning*), dan pemanggilan tool (*function calling*).<br>- Memproses prompt pengguna dan respons model AI. | - Semi-trusted boundary.<br>- **Dilarang memiliki akses eksekusi OS langsung (*raw shell/terminal execution*)**.<br>- Seluruh pemanggilan tool wajib dialihkan (*proxied*) ke Hime Core via Named Pipe. `HERMES_YOLO_MODE` dimatikan secara permanen. |
| **9Router** | Sub-proses Terpisah (Node.js 24 Background Daemon) | - Berfungsi sebagai AI Provider Gateway multi-penyedia.<br>- Melakukan perutean model pintar (*smart routing*), fallback otomatis (*combo models*), dan kompatibilitas OpenAI REST API. | - Untrusted boundary.<br>- Berjalan pada port dinamis loopback (default 20128).<br>- Terikat batasan RSK-10 (bypass auth localhost) dan RSK-12 (default 0.0.0.0).<br>- Dilarang mengekspos port ke LAN. |
| **Hime Tool Gateway** | Modul Logika Internal Hime Core (Rust) | - Mengatur izin dan memediasi eksekusi seluruh tool (filesystem, shell, web browser, device sync).<br>- Menerapkan *path allowlist* ketat dan pembersihan parameter input. | - Enforcer tingkat sistem.<br>- Memblokir path traversal (`canonicalize`), menolak akses direktori sensitif (`.ssh`, browser cookies, system32). |

### C. Prinsip Isolasi Proses & Daur Hidup Mandiri

1. **Larangan Merger Runtime Prematur:**
   - Runtime Python (Hermes), Node.js (9Router), dan Rust (Hime Core/Desktop) **TIDAK BOLEH DIGABUNGKAN** ke dalam satu proses tunggal demi alasan simplifikasi arsitektur.
   - Setiap komponen mempertahankan runtime, tumpukan dependensi, daur hidup (*lifecycle*), dan ruang memori (*address space*) mandiri.
2. **Topologi Dual-Control Plane:**
   - *Control Plane 1 (Manajemen & Supervisi):* Hime Core mengontrol 9Router dan Hermes melalui sinyal proses OS, Windows Job Objects, variabel lingkungan saat spawn, dan IPC Named Pipe internal.
   - *Control Plane 2 (Data Plane / Inferensi):* Hermes Agent mengirimkan payload inferensi langsung ke 9Router melalui localhost HTTP endpoint (`http://127.0.0.1:20128/v1`), menjaga latensi transmisi token tetap minimal.
3. **Arah Desain Hime Core:**
   - **Modular Monolith:** Hime Core dibangun sebagai modular monolith pada tahap awal untuk menghindari kompleksitas prematur microservices.
   - **Clean Architecture / Ports & Adapters:** Seluruh batas I/O eksternal (IPC broker, SQLite storage, supervisor process, DPAPI vault) diisolasi melalui traits/interfaces.
   - **Domain-Driven Design (DDD):** Batas domain dipisahkan tegas antara Session Management, Policy Enforcement, Memory/Context, dan Tool Execution.
   - **Single Hermes Runtime:** Hime menggunakan satu runtime Hermes Agent terpadu dengan orkestrasi specialist skills dan workflow sebelum mengevaluasi runtime multi-agent.

---

## 5. Security Foundation (SEC-01 through SEC-10)

Konsolidasi 10 kontrol keamanan rekayasa wajib untuk memitigasi risiko-risiko terkonfirmasi:

### A. Matriks 10 Kontrol Keamanan Wajib

| ID Kontrol | Nama Kontrol Keamanan | Mekanisme Teknis Rekayasa | Ancaman yang Dimitigasi | Status Implementasi |
|:---:|:---|:---|:---|:---:|
| **SEC-01** | **NTFS ACL Hardening & EFS Encryption** | Menetapkan ACL direktori restriktif (`icacls "%APPDATA%\9router" /inheritance:r /grant:r "%USERNAME%":(OI)(CI)F`) dan enkripsi diska offline EFS (`cipher /e`). | Akses lintas akun OS pada mesin multi-user, ekstraksi fisik diska offline. | **PLANNED** (Phase 06) |
| **SEC-02** | **DPAPI + Optional Entropy Secret Vault** | Enkripsi rahasia master at-rest menggunakan Windows DPAPI (`CryptProtectData`) dengan garam aplikasi *optionalEntropy*. | Pencurian kredensial offline, dump file cadangan, scraping database oleh akun lain. | **PLANNED** (Phase 06/09) |
| **SEC-03** | **Process Supervisor Environment Injection** | Injeksi token CSPRNG 32-karakter acak via `std::process::Command::env("INITIAL_PASSWORD", &token)` langsung ke environment block 9Router saat spawn (zero shell history, zero CLI args). | Pengambilalihan dashboard 9Router via password bawaan `123456` (RSK-04, CWE-1188). | **PLANNED** (Phase 06) |
| **SEC-04** | **Strict Loopback Binding & Active Runtime Socket Verification** | Menjalankan 9Router dengan `--host 127.0.0.1` (`[RECOMMENDATION]`). Supervisor aktif memeriksa socket TCP IPv4/IPv6 via Win32 `GetExtendedTcpTable`, mencocokkan port, dan memastikan `dwOwningPid` cocok dengan PID anak 9Router (`[PLANNED CONTROL]`). *Fail-closed kill* jika melanggar. | Paparan port 20128 ke seluruh jaringan LAN (RSK-12, CWE-1327). | **PLANNED / NOT YET VERIFIED** (Phase 06) |
| **SEC-04 (B)**| **Localhost Auth Patch / OS Process Isolation** | Menghapus short-circuit loopback auth `if (isLocalRequest) return true;` pada 9Router via upstream PR / maintained fork, ATAU isolasi akun service Windows + aturan firewall WFP. | Akses inferensi tanpa autentikasi oleh proses lokal lain (RSK-10, CWE-306). | **IMPLEMENTATION BLOCKER** (Phase 05/06) |
| **SEC-05** | **Authenticated & DACL-Protected Named Pipes IPC** | Komunikasi lokal via Windows Named Pipes dengan `PipeSecurity` DACL khusus SID pengguna aktif, dilarang `SECURITY_DELEGATION`/`IMPERSONATION`, disertai token handshake 256-bit sekali pakai. | Pembajakan IPC lokal, injeksi perintah sintetis oleh aplikasi tak berizin (CWE-285). | **PLANNED** (Phase 07) |
| **SEC-06** | **Central Policy & Tool Permission Gate** | Intersepsi seluruh pemanggilan tool Hermes di Hime Core. Mewajibkan dialog konfirmasi visual manusia (*Human Confirmation Modal*) pada Hime Desktop untuk aksi destruktif. `HERMES_YOLO_MODE` nonaktif permanen. | Eskalasi eksekusi perintah berbahaya akibat *Indirect Prompt Injection* (RSK-11, CWE-78). | **PLANNED** (Phase 08) |
| **SEC-07** | **Filesystem Sandboxing & Path Canonicalization** | Resolusi canonical path (`canonicalize`), verifikasi batasan workspace, dan deny-list absolut untuk folder sensitif (`.ssh`, `.gnupg`, cookies browser, `%WINDIR%`). Penulisan berkas atomik (*atomic replace*). | Penembusan direktori / Zip Slip (`..\..`), penghancuran berkas sistem (CWE-22). | **PLANNED** (Phase 09) |
| **SEC-08** | **Windows Job Objects Process Supervision** | Pengikatan seluruh proses anak (9Router, Hermes) ke Win32 Job Object dengan flag `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE` serta kuota batas memori (9Router 512MB, Hermes 1024MB). | Proses liar (*orphan/zombie daemons*), kebocoran memori ekstrim, serangan DoS host. | **PLANNED** (Phase 06) |
| **SEC-09** | **Structured Logging & Secret Redaction Layer** | Audit logging terstruktur `AUDIT.jsonl` dengan layer filter regex otomatis untuk menyamarkan API key (`Bearer [REDACTED]`, `sk-[REDACTED]`, token CLI) sebelum ditulis ke diska. | Kebocoran rahasia dan kredensial ke dalam berkas log (CWE-532). | **PLANNED** (Phase 06) |
| **SEC-10** | **Authenticode Code Signing & Update Verification** | Penandatanganan biner Windows dengan sertifikat digital Authenticode, validasi manifest pembaruan HTTPS bertanda tangan Ed25519 dan hash SHA-256, serta rollback otomatis. | Penggantian biner atau update palsu oleh penyerang jaringan (Supply Chain Tampering). | **PLANNED** (Phase 15) |

### B. Status Risiko Keamanan Utama & Implementation Blocker

1. **RSK-10 (Bypass Autentikasi Loopback LLM API — IMPLEMENTATION BLOCKER):**
   - *Fakta Kode:* `src/dashboardGuard.js:161-165` meloloskan seluruh request loopback tanpa validasi API key.
   - *Dampak:* Proses lokal lain dapat langsung memanggil port 9Router dan menghabiskan kuota inferensi. Loopback binding bukan autentikasi; token virtual diabaikan.
   - *Klasifikasi:* **`IMPLEMENTATION BLOCKER & PROPOSED RESIDUAL RISK — OWNER DECISION REQUIRED`**. Hime dilarang menggunakan 9Router untuk operasional multi-tenant atau berbayar sebelum keputusan mitigasi dipilih oleh Project Owner (Opsi Upstream Fork vs Opsi Akun Service WFP).
2. **RSK-12 (Default Host Binding 0.0.0.0 & Active Runtime Socket Verification):**
   - *Fakta Kode:* `cli/cli.js:110` menetapkan `DEFAULT_HOST = "0.0.0.0"` (`[VERIFIED]`).
   - *Status Mitigasi:* Parameter startup `--host 127.0.0.1` (`[RECOMMENDATION]`); verifikasi runtime socket aktif IPv4/IPv6 & `dwOwningPid` matching (`[PLANNED CONTROL]`). Efektivitas mitigasi berstatus **`[NOT YET VERIFIED]`** hingga diuji pada Phase 06.
3. **RSK-03 (Plaintext Credentials in 9Router SQLite):**
   - *Fakta Kode:* Kredensial penyedia AI disimpan dalam teks polos JSON di `providerConnections.data` pada berkas `%APPDATA%\9router\db\data.sqlite` (CWE-312, `[VERIFIED]`).
   - *Status:* Dimitigasi melalui NTFS ACL & EFS; berstatus **`PROPOSED RESIDUAL RISK — OWNER DECISION REQUIRED`** terhadap potensi pembacaan oleh malware pada sesi pengguna yang sama.
4. **RSK-08 (9Router CLI Machine Token Broad Authorization):**
   - *Fakta Kode:* Token `x-9r-cli-token` diturunkan dari `auth/cli-secret` dan `MachineGuid`, meloloskan pemanggilan ke seluruh endpoint administrasi sensitif `/api/*` (`[VERIFIED]`).
   - *Status:* Dimitigasi melalui kontrol proses internal; berstatus **`PROPOSED RESIDUAL RISK — OWNER DECISION REQUIRED`**.
5. **Kualifikasi Teknis Akses Memori Satu Pengguna (*Same-User Process Memory Access*):**
   - *Fakta Teknis:* Malware pada akun pengguna yang sama (`XATOSHI\Satoshi` Medium Integrity) adalah ancaman serius. Namun, membaca memori proses lain (`PROCESS_VM_READ` 0x0010 dan `PROCESS_QUERY_INFORMATION` 0x0400) dievaluasi oleh Windows SRM berdasarkan Security Descriptor (DACL) objek proses dan token pemanggil. Hime Core dapat memperketat DACL objek prosesnya sendiri (`SetKernelObjectSecurity`) dan mengaktifkan mitigasi proses (`PROCESS_MITIGATION_DYNAMIC_CODE_POLICY`). Kendati demikian, DPAPI, NTFS ACL, dan zeroization sendiri **TIDAK memberikan isolasi proses yang lengkap**.
   - *Status:* **`PROPOSED RESIDUAL RISK — OWNER DECISION REQUIRED`**.
6. **Prinsip Otoritas Tata Kelola:** AI Agent **DILARANG KERAS** menyatakan suatu risiko residu telah diterima (*accepted*) secara sepihak tanpa keputusan formal dari Project Owner.

---

## 6. Architecture & Implementation Constraints (10 Non-Negotiables)

1. **Komponen Eksternal Tidak Dibangun Ulang:** Hermes Agent dan 9Router adalah dependensi eksternal berlisensi MIT. Keduanya diaudit, dikonfigurasi, dan diintegrasikan — bukan dibangun ulang dari nol.
2. **Reimplementasi Bersih Nusa (*Clean-Room Reimplementation*):** Kode sumber Nusa tidak tersedia. Hime Desktop wajib dibangun ulang dari nol (*clean-room reimplementation*) menggunakan Tauri 2.0 (Rust) + Web tanpa menduplikasi biner proprietary.
3. **Hime Core Dibangun Mandiri Dari Nol:** Lapisan orkestrasi, pengawasan, dan penegakan kebijakan (Hime Core) dibangun secara mandiri dalam bahasa Rust untuk menjamin keandalan memori, keamanan tipe, dan kinerja tingkat kernel.
4. **Hermes Tanpa Akses Sistem Langsung:** Hermes Agent dilarang memiliki akses eksekusi shell, terminal, atau manipulasi sistem berkas tanpa mediasi dan persetujuan dari Hime Core.
5. **Kepatuhan Penanganan Kredensial & Manajemen 9Router:** Penanganan kredensial, variabel lingkungan, dan port 9Router wajib mematuhi seluruh batasan keamanan SEC-01 s/d SEC-04.
6. **Larangan Manipulasi Basis Data 9Router Langsung:** Dilarang melakukan operasi tulis langsung (*direct database write*) ke SQLite 9Router tanpa desain integrasi yang telah disetujui melalui ADR dan diuji secara komprehensif.
7. **Preservasi Daur Hidup & Versi Mandiri Komponen:** Daur hidup, tumpukan dependensi, dan nomor versi setiap komponen (Hime Core, Desktop, Hermes, 9Router) wajib dikelola secara independen.
8. **Nol Rahasia di Kode Sumber & Log:** Dilarang keras menempatkan API key, password, token, atau secondary entropy secara statis (*hardcoded*) di dalam kode sumber, templat konfigurasi, atau berkas log.
9. **Larangan Operasi Destruktif Tanpa Otorisasi:** Dilarang mengeksekusi operasi penghapusan berkas, modifikasi sistem, atau pengunduhan biner eksternal tanpa kebijakan eksplisit dan otorisasi manusia.
10. **Penegakan Keamanan Tingkat Sistem:** Kontrol keamanan wajib ditegakkan pada lapisan sistem (Hime Core, OS DACL, Job Objects, Firewall), bukan hanya mengandalkan instruksi teks prompt (*system prompt instructions*).

---

## 7. Engineering Standards & Quality Framework

Tumpukan standar rekayasa yang disepakati untuk menjamin maintainability dan keandalan sistem:

1. **Prinsip Desain Perangkat Lunak:** Penerapan SOLID, KISS, dan DRY secara proporsional. Hindari abstraksi spekulatif (*no speculative abstractions*) dan hindari penambahan pustaka yang tidak memiliki kebutuhan mendesak (*YAGNI*).
2. **Modular Monolith & Ports and Adapters:** Hime Core dibangun sebagai modular monolith dengan domain model terisolasi (DDD) dan ports/adapters untuk setiap integrasi I/O.
3. **Test-Driven Development (TDD) untuk Logika Kritis:** Modul otentikasi, validasi socket, inspeksi token, enkripsi vault, dan sandboxing path wajib dikembangkan dengan pendekatan TDD dan pengujian unit otomatis.
4. **Hierarki Pengujian Komprehensif:**
   - *Unit Tests:* Menguji fungsi individual secara terisolasi (mock I/O).
   - *Integration Tests:* Menguji interaksi lintas komponen (Named Pipe, Job Objects, Supervisor).
   - *Contract Tests:* Menguji kepatuhan antarmuka REST OpenAI 9Router dan protokol IPC.
   - *Security Tests:* Menguji penolakan traversal path, kebocoran port, dan sanitasi log.
5. **Trunk-Based Development & Semantic Versioning:** Pengembangan menggunakan branch `main` dengan conventional commit (`feat:`, `fix:`, `docs:`, `security:`, `refactor:`).
6. **Architecture Decision Records (ADR):** Setiap keputusan arsitektur signifikan wajib dicatat dalam format ADR di `docs/adr/`.
7. **Pencatatan Log Terstruktur & Jejak Audit:** Seluruh subsistem menggunakan structured logging (JSON Lines) dengan korelasi ID (`trace_id`) untuk penelusuran forensik.
8. **Migrasi Basis Data Berversi:** Skema basis data percakapan Hime Core dikelola melalui migrasi SQL berversi yang deterministik.
9. **Kriteria Selesai (*Definition of Done*):** Suatu task hanya dinyatakan selesai jika kode telah ditinjau (*peer/self-review*), pengujian lulus 100% dengan bukti keluaran, dokumentasi diperbarui, galat whitespace nihil (`git diff --check`), dan seluruh kriteria penerimaan terpenuhi.
