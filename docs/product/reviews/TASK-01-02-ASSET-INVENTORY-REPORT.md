# LAPORAN TEKNIS AUDIT PENEMUAN: TASK-01-02
## Existing Project & Research Asset Inventory

- **Dokumen:** Laporan Komprehensif Inventarisasi Aset Proyek & Riset Hime Assistant
- **Fase:** Phase 01 — Discovery & Environment Audit
- **Task ID:** TASK-01-02
- **Target Objek:** Seluruh Aset Repositori, Aplikasi Referensi Nusa, Hermes Agent, 9Router, Dokumentasi, Lisensi, dan Pustaka Skill
- **Auditor:** Senior Engineering Infrastructure Architect & Repository Specialist
- **Tanggal Pelaksanaan:** 2026-11-07
- **Status Tugas:** `[VERIFIED]` **ACCEPTED**

---

## 1. Executive Summary

Sesuai dengan mandat tata kelola pada [`AGENTS.md`](file:///x:/folder_ai/Hime/AGENTS.md) dan arahan *Qualified Acceptance* TASK-01-01, pelaksanaan **TASK-01-02 (Existing Project & Research Asset Inventory)** difokuskan untuk menginventarisasi secara faktual, mendalam, dan non-destruktif seluruh aset fisik dan intelektual yang telah tersedia di dalam ruang kerja Hime (`X:\folder_ai\Hime`).

Audit ini mengonfirmasi bahwa proyek Hime Assistant berdiri di atas fondasi riset yang sangat solid dan kaya:
1. **Pemisahan Sumber Tertutup vs Mandiri:** Aplikasi referensi **Nusa Dynamic Island v1.2.0.0** terkonfirmasi merupakan aplikasi *closed-source* (freeware proprietary karya Rama). Sumber kode tidak tersedia, sehingga seluruh arsitektur dan perilaku desktop diekstraksi melalui dekompilasi biner statis dan observasi runtime. Hime Desktop tidak menduplikasi biner Nusa, melainkan melakukan reimplementasi mandiri (*clean-room reimplementation*) menggunakan Tauri 2.0 (Rust) + Web.
2. **Kesiapan Komponen Eksternal (*External Dependencies*):**
   - **Hermes Agent (Nous Research):** Kode sumber lengkap tersedia pada commit `c5f347eb`, berlisensi resmi **MIT**, siap dioperasikan menggunakan Python 3.13 dan Astral `uv`.
   - **9Router AI Gateway (decolua):** Kode sumber lengkap monorepo Next.js 16 / React 19 tersedia pada commit `1fe586bb` (v0.5.112), berlisensi resmi **MIT**, siap dioperasikan menggunakan Node.js v24.
3. **Kekayaan Laporan Riset Intelektual:** Ditemukan dan diverifikasi **27 dokumen riset teknis utama** (8 laporan Nusa, 5 laporan Hermes, 14 laporan 9Router) yang telah merinci seluruh aspek integrasi, API, arsitektur, dan mitigasi keamanan.
4. **Pustaka Skill Terkurasi:** Tersedia **7 pustaka skill AI** berstandar terbuka di `.agent/skills/`, seluruhnya berlisensi MIT.
5. **Kesiapan Fase Berikutnya:** Tidak ada hambatan teknis (*blockers*) untuk melanjutkan ke **TASK-01-03 (Dependency and License Audit)** dan penyusunan spesifikasi produk pada Phase 04.

---

## 2. Repository Inventory & Peta Struktur Workspace

Pemeriksaan menyeluruh terhadap root workspace `X:\folder_ai\Hime` menghasilkan pemetaan struktur pohon direktori kanonikal sebagai berikut:

```text
Hime/ (X:\folder_ai\Hime)
├── .agent/                      <- Pustaka Curated Agent Skills kanonikal (7 skills, format SKILL.md)
├── .agents                      <- Windows Directory Junction ke .agent/ (kompatibilitas runner agen)
├── .git/                        <- Repositori Git root (branch main, baseline commit 8cf0070)
├── Archive/                     <- Arsip artefak lama & transisi (dijaga oleh .gitkeep)
├── docs/                        <- Dokumentasi resmi proyek, governance, ADR, security, QA
│   ├── adr/                     <- Architecture Decision Records (ADR_TEMPLATE.md)
│   ├── architecture/            <- Blueprint arsitektur teknis masa depan (.gitkeep)
│   ├── governance/              <- 10 Dokumen kebijakan tata kelola rekayasa
│   ├── master-plan/             <- Master roadmap & vision (.gitkeep)
│   ├── reports/                 <- Laporan audit infrastruktur (REPOSITORY_PREFLIGHT_AUDIT.md)
│   ├── security/                <- Kebijakan & template audit keamanan (SECURITY_REVIEW_TEMPLATE.md)
│   ├── skills/                  <- Indeks pustaka skill (SKILL_REGISTRY.md)
│   ├── standards/               <- Standar desain & coding (.gitkeep)
│   └── testing/                 <- Strategi & template pengujian (QA_REPORT_TEMPLATE.md)
├── Documentation/               <- Dokumentasi pendukung historis (.gitkeep)
├── project-management/          <- Sistem pelacakan proyek, task, risiko, keputusan
│   ├── backlog/                 <- Backlog kanonikal (BACKLOG.md)
│   ├── decisions/               <- Log keputusan arsitektur & tata kelola (DECISION_LOG.md)
│   ├── inventory/               <- Indeks aset kanonikal (ASSET-REGISTER.md)
│   ├── phases/                  <- Template & berkas acceptance fase (PHASE-00-ACCEPTANCE.md)
│   ├── reviews/                 <- Laporan review formal (TASK-00-02, TASK-01-01, TASK-01-02)
│   ├── risks/                   <- Register risiko komprehensif RSK-01 s/d RSK-08 (RISK_REGISTER.md)
│   └── tasks/                   <- Spesifikasi rinci task TASK-00-01, 00-02, 01-01, 01-02
├── Projects/                    <- Ruang kerja kode sumber implementasi masa depan (.gitkeep)
├── RE/                          <- Arsip Riset Rekayasa Terbalik (Reverse Engineering)
│   ├── 00-source-materials      <- Sub-direktori materi sumber mentah (diabaikan dari Git root)
│   ├── 9ROUTER/                 <- 14 Laporan riset gateway 9Router + repo 9router v0.5.112
│   ├── Hermes/                  <- 5 Laporan riset Hermes Agent + repo hermes-agent c5f347eb
│   ├── Nusa/                    <- 8 Laporan riset Nusa Dynamic Island + installer & ekstraksi
│   ├── Shared/                  <- Direktori riset lintas komponen (architecture, diagrams, reports)
│   ├── README.md                <- Panduan arsip riset
│   └── research-index.md        <- Indeks target riset TRG-01 s/d TRG-04
├── Resources/                   <- Desain grafis, aset visual, mockups (.gitkeep)
├── scripts/                     <- Script build, packaging, dan automasi (.gitkeep)
├── tools/                       <- Perkakas pendukung pengembang (.gitkeep)
├── .gitignore                   <- Aturan filter Git (3.7 KB, isolasi binary, nested git, DB, secrets)
├── AGENTS.md                    <- Aturan permanen coding agent Hime (authoritative)
├── CHANGELOG.md                 <- Log perubahan versi proyek
├── PROJECT_STATUS.md            <- Status fase rekayasa resmi (16 fase Master Plan)
└── README.md                    <- Gambaran umum proyek Hime Assistant
```

### Penjelasan Fungsi Aktual Direktori Utama Berdasarkan Bukti:
1. **`.agent/` (Curated Skills):** Berisi 7 pustaka skill aktif berformat Markdown (`SKILL.md`) yang memandu agen AI dalam animasi UI (`animate`), rekayasa desain (`emil-design-eng`), efisiensi kode minimalis (`ponytail`), metodologi RE (`rea`), debugging sistematis (`systematic-debugging`), gerbang verifikasi bukti (`verification-before-completion`), dan panduan aksesibilitas web (`web-design-guidelines`).
2. **`docs/` (Governance & Standards):** Pusat kebijakan tata kelola rekayasa Hime. Mengatur siklus hidup perubahan kode, standar peninjauan, kebijakan keamanan kredensial, aturan agen otonom, dan template operasional.
3. **`project-management/` (Execution Tracking):** Sistem pelacakan status proyek yang mengisolasi backlog, risiko (RSK-01 s/d RSK-08), keputusan arsitektural (DEC-01 s/d DEC-08), dan laporan formal penutupan task/fase.
4. **`RE/` (Reverse Engineering Knowledge Base):** Menyimpan seluruh basis pengetahuan empiris yang diperoleh dari pembedahan biner dan kode sumber tiga komponen pilar: Nusa, Hermes, dan 9Router.
5. **`Projects/`, `scripts/`, `tools/`, `Resources/`:** Dipersiapkan sebagai struktur direktori bersih (*clean-slate*) untuk tahap implementasi kode produk (Phase 06 ke atas), saat ini dijaga oleh berkas `.gitkeep` agar strukturnya terlacak di Git tanpa membawa sampah biner.

---

## 3. Nusa Application Asset Inventory

Nusa Dynamic Island merupakan aplikasi desktop Windows berorientasi visual yang menjadi acuan utama antarmuka mengambang (*floating Dynamic Island HUD*) Hime.

### A. Klasifikasi Bukti Aset Nusa
- **`[VERIFIED]` Ketersediaan Biner Installer & Ekstraksi:**
  - Installer Asli: `RE/Nusa/00-source-materials/NusaSetup.exe` (2,945,252 bytes, SHA-256: `BC4133305B28B929ECA42CC32833DC4FD34002CE35676D27761942419F4D4B02`).
  - Arsip Distribusi: `RE/Nusa/00-source-materials/NusaSetup.zip` (2,430,268 bytes).
  - Skrip Konfigurasi Installer: `install_script.iss` (Dekompresi via `innounp 2.70.1`, membuktikan Inno Setup v6.7.0 Unicode dengan kompresi solid LZMA2).
  - Biner Aplikasi Statis: `nusa.exe` (1,265,664 bytes, PE32+ x86-64 GUI).
  - Tabel String Ekstraksi: `rdata_strings.txt` (49,682 bytes dump ASCII/Unicode seksi `.rdata`).
  - Versi Aplikasi Terverifikasi: `1.2.0.0` (AppId: `{A03D4881-45E3-4B16-B844-936040C558E5}`).
  - Hak Akses Pemasangan: `PrivilegesRequired=lowest` (Dipasang di `%LOCALAPPDATA%\Programs\Nusa` tanpa prompt UAC Administrator).
- **`[VERIFIED]` Ketiadaan Kode Sumber (*Source Code Unavailable*):**
  - Tidak ditemukan repositori kode sumber Nusa di seluruh workspace atau rilis publik. Nusa didistribusikan sebagai freeware biner tertutup oleh Rama (`https://rama2555.github.io/nusa/`).
  - **Kesimpulan Arsitektur:** Hime Desktop wajib melakukan **reimplementasi mandiri (*clean-room reimplementation*)** menggunakan Tauri 2.0 (Rust) + Web, bukan melakukan modifikasi biner (*binary patching*) atau forking.
- **`[VERIFIED]` Runtime & Tumpukan Grafis Nusa:**
  - Bahasa Pemrograman: **Rust (100% Native)** — Ditemukan jejak `rustc/48a229ceaefd4985c50990b14116b6d856af0985` dan modul `std::sys::pal::windows`.
  - Windows API Binding: `windows-rs v0.58.0` (`windows-core`, `windows-strings`).
  - Compositor & Rendering: **DirectComposition** (`dcomp.dll`), **Direct2D** (`d2d1.dll`), **DirectWrite** (`dwrite.dll`), dan DWM (`dwmapi.dll`).
  - Jendela Utama: Window class `NotchWindow`, extended styles `WS_EX_NOREDIRECTIONBITMAP`, `WS_EX_TOPMOST`, `WS_EX_TOOLWINDOW`, `WS_EX_LAYERED`.
  - Konsumsi Sumber Daya: Memori aktif ~58–61 MB, CPU idle 0.0%–0.1%.
- **`[INFERRED]` Struktur Data Penyimpanan Lokal Nusa:**
  - File penyimpanan lokal di `%LOCALAPPDATA%\Nusa\`: `tasks.txt`, `water.txt`, `focus.txt`, `shelf.txt`, `clipboard.txt`. Format penyimpanan berupa teks polos / key-value sederhana.
- **`[UNKNOWN]` Algoritma Khusus:**
  - Implementasi internal perhitungan hisab jadwal shalat Kemenag RI tertanam dalam bentuk instruksi mesin x86-64 di seksi `.text`, namun formula input/output astronomisnya standar dan dapat digantikan dengan crate astronomi Rust terbuka (`praytimes` atau API Kemenag).

---

## 4. Nusa Reverse Engineering Research Inventory

Seluruh hasil riset pembedahan Nusa tersimpan rapi di `RE/Nusa/` dengan 8 dokumen teknis lengkap:
1. `01-initial-inspection/01-initial-reconnaissance.md`: Header PE, hash SHA-256, signature digital.
2. `01-initial-inspection/02-installer-analysis.md`: Dekompilasi konfigurasi Inno Setup `.iss`, Registry run keys, privilege lowest.
3. `02-binary-analysis/03-binary-analysis.md`: Analisis seksi biner Rust, eliminasi hipotesis Electron/WPF, crate `windows-rs`.
4. `03-application-architecture/04-runtime-analysis.md`: Window handle `NotchWindow`, DirectComposition swapchain, multi-monitor DPI.
5. `04-ui-and-interaction-analysis/05-ui-ux-analysis.md`: Dimensi kapsul idle (180–240 × 34 px) vs panel expanded (740 × 212 px), tema OLED, frosted glass acrylic.
6. `05-dependencies-and-frameworks/06-feature-analysis.md`: Pemetaan seluruh modul fungsional Nusa.
7. `06-modification-feasibility/07-modification-feasibility.md`: Perbandingan opsi binary patching vs Tauri 2.0 reimplementation.
8. `reports/FINAL-REVERSE-ENGINEERING-REPORT.md`: Laporan komprehensif master reverse engineering Nusa (19.4 KB).

### Matriks Fitur Nusa (Nusa Feature Matrix)

| Feature | Evidence | Current Understanding | Confidence | Follow-up for Hime |
|:---|:---|:---|:---:|:---|
| **Dynamic Island Floating Window** | HWND `NotchWindow`, seksi `.rdata` `dcomp.dll`, `dwmapi.dll` | Jendela transparan mengambang di atas layar (`WS_EX_TOPMOST`, `WS_EX_TOOLWINDOW`) dengan DirectComposition hardware acceleration. | `HIGH` `[VERIFIED]` | Reimplementasi via Tauri 2.0 Rust native window plugins di Phase 07. |
| **Media Controller & Live Lyrics** | WinRT GSMTC session strings, endpoint `lrclib.net/api/search` | Membaca metadata lagu Windows via GSMTC dan mengambil lirik tersinkronisasi waktu via REST API LRCLIB. | `HIGH` `[VERIFIED]` | Integrasikan WinRT media session di Hime Core / Desktop. |
| **File Shelf Staging** | OLE DragDrop COM interfaces (`IDropTarget`, `DoDragDrop`), folder `%LOCALAPPDATA%\Nusa\Shelf` | Penampungan sementara file dari File Explorer dengan drag-and-drop. | `HIGH` `[VERIFIED]` | Implementasikan OLE DragDrop pada drawer antarmuka Hime. |
| **Clipboard History** | API `GetClipboardData`, format `ExcludeClipboardContentFromMonitorProcessing` | Menyimpan 5 item teks salinan terbaru dengan perlindungan agar tidak bocor ke history eksternal. | `HIGH` `[VERIFIED]` | Reimplementasi dengan enkripsi lokal dan batas kuota. |
| **Water Tracker & Hidrasi** | File skema `water.txt` (`bottle=`, `goal=`, `taken=`), tombol Drank/Refill | Pelacak konsumsi air harian dengan timer interval pengingat minum. | `HIGH` `[VERIFIED]` | Masukkan ke dalam modul produktivitas Hime (Phase 12). |
| **Pomodoro Focus Timer** | File skema `focus.txt`, suara transisi chime `PlaySoundW` | Siklus kerja 25 menit / istirahat 5 menit dengan notifikasi audio. | `HIGH` `[VERIFIED]` | Hubungkan dengan status "Focus/DND" pada Hermes Agent. |
| **Prayer Times (Jadwal Shalat)** | Daftar kota 38 provinsi di `.rdata`, fitur jeda media otomatis | Menghitung waktu shalat Kemenag RI dan otomatis menjeda musik/video saat azan. | `HIGH` `[VERIFIED]` | Sediakan modul jadwal shalat lokal di Hime Core. |
| **Hardware HUD Indicators** | String event `hud_volume`, `hud_brightness`, `hud_caps`, `popup_battery` | Visualisasi animasi kapsul saat volume, brightness, atau tombol sistem berubah. | `HIGH` `[VERIFIED]` | Integrasikan event hook Windows di backend Rust Tauri. |
| **Widgets & Glanceable Info** | String layout World Clock, Moon Phase, Event Countdown, Today summary | Panel widget ringkasan harian yang dapat diakses dengan swipe/klik. | `MEDIUM` `[OBSERVED]` | Bangun sebagai Dynamic Island Drawer widget di frontend web. |

---

## 5. Hermes Agent Inventory (Nous Research)

Hermes Agent merupakan agen AI otonom yang menjadi otak penalaran (*reasoning runtime*) bagi Hime Assistant.

### A. Identitas Repositori & Runtime
- **Lokasi Repositori:** `RE/Hermes/00-source-materials/hermes-agent/` (Git repository terisolasi, diabaikan oleh root `.gitignore`).
- **Versi & Commit Terverifikasi:** Commit `c5f347eb` (`chore(catalog): review disclosure for rentahuman`), branch `main`.
- **Lisensi:** **MIT License**, Copyright (c) 2025 Nous Research (Berkas: `LICENSE`).
- **Runtime Environment:** Python `>=3.11,<3.15` (dikelola via Astral `uv`, file `uv.lock` berukuran 1.25 MB). Host Windows 11 telah terverifikasi memiliki Python 3.13.16 dan `uv` 0.12.16.

### B. Arsitektur Kode & Siklus Hidup Agen
1. **Core Orchestrator (`agent/ai_agent.py`):**
   - Kelas `AIAgent` berbasis 14 mixins modular: manajemen status percakapan, registry perkakas, perizinan keamanan, integrasi memori, penyedia LLM, dll.
2. **Mesin Status Loop Penalaran 10 Fase (`conversation_loop.py`):**
   - Siklus penalaran terstruktur:
     `PROMPT_PREPARATION` → `CONTEXT_ASSEMBLY` → `MODEL_INFERENCE` → `STREAMING_TOKEN_PROCESSING` → `TOOL_CALL_EXTRACTION` → `APPROVAL_GATING` → `TOOL_EXECUTION` → `OBSERVATION_INJECTION` → `TRAJECTORY_COMPRESSION` → `TERMINATION_EVALUATION`.
3. **Sistem Perizinan Perkakas (*Approval Gating*):**
   - Mendukung 3 tingkat keamanan: `YOLO` (otomatis jalan tanpa konfirmasi), `Smart Guardian` (verifikasi keamanan menggunakan model LLM kecil evaluator), dan `Manual Gate` (modal dialog konfirmasi persetujuan pengguna manusia).
4. **Persistensi Memori & FTS5 (`hermes_state*.py`):**
   - Basis data memori SQLite mode WAL (*Write-Ahead Logging*).
   - Indeks pencarian semantik teks penuh menggunakan SQLite FTS5 (`hermes_state_fts.py`).
   - Algoritma kompresi trajektori percakapan (`trajectory_compressor.py`).
5. **Transport Komunikasi & Antarmuka IPC (`gateway/`):**
   - **Win32 Named Pipe:** `\\.\pipe\hermes-gateway-<hash>` (`gateway/windows_pipe.py`).
   - **WebSocket Gateway:** Endpoint streaming `/api/ws` dengan envelope pesan JSON terstruktur.
   - **FastAPI / Uvicorn Server:** REST API lokal untuk administrasi sesi dan perkakas.
   - **Tiket Autentikasi:** Mekanisme handshake tiket sesi berumur pendek (TTL 30 detik).

### C. Pembagian Tanggung Jawab: Hermes Agent vs Hime Core

| Kemampuan / Fitur | Disediakan oleh Hermes Agent | Harus Disediakan oleh Hime Core |
|:---|:---:|:---:|
| **Inference Call ke LLM** | Ya (via `OPENAI_BASE_URL`) | Tidak (Hime hanya mengonfigurasi endpoint 9Router) |
| **Reasoning & Tool Execution Loop** | Ya (10-Phase State Machine) | Tidak (Hermes mengeksekusi tools di sandboxnya) |
| **ConPTY Terminal Execution** | Ya (Subprocess tree management) | Tidak |
| **Memory FTS5 & SQLite State** | Ya (Manajemen riwayat obrolan) | Tidak (Hermes mengelola databasenya sendiri) |
| **Process Lifecycle Supervision** | Tidak | **Ya (Hime Rust Supervisor memantau daemon Hermes)** |
| **Dynamic Island HUD UI** | Tidak | **Ya (Tauri 2.0 Webview merender glow & status)** |
| **Windows DPAPI Credential Vault** | Tidak | **Ya (Hime mengamankan API key di vault DPAPI Windows)** |
| **Hardware & Media Integration (GSMTC)** | Tidak | **Ya (Hime menangkap event volume/lagu dari Windows)** |
| **Approval Modal UI** | Tidak | **Ya (Hime menampilkan pop-up konfirmasi YOLO/Guardian)** |

---

## 6. 9Router AI Gateway Inventory (decolua)

9Router adalah gateway multi-provider AI yang berperan sebagai proxy model lokal, failover otomatis, dan perute lalu lintas inferensi Hermes Agent.

### A. Identitas Repositori & Runtime
- **Lokasi Repositori:** `RE/9ROUTER/00-source-materials/9router/` (Git repository terisolasi, diabaikan oleh root `.gitignore`).
- **Versi & Commit Terverifikasi:** Commit `1fe586bb` (`# v0.5.112 (2026-10-28)`), branch `master`.
- **Lisensi:** **MIT License**, Copyright (c) 2024-2026 decolua and contributors (Berkas: `LICENSE`).
- **Runtime Environment:** Node.js `>=18.0.0` (Host Windows 11 telah terverifikasi memiliki Node.js v24.18.0, npm 11.16.0, dan pnpm 12.14.0).
- **Port Operasional:** Port `20127` (Web Dashboard) dan Port `20128` (AI Reverse Proxy Gateway).

### B. Pemetaan 14 Kelompok Riset 9Router
Seluruh 14 topik riset yang dipersyaratkan terbukti lengkap dan terdokumentasi di `RE/9ROUTER/`:
1. `01-INITIAL-RECONNAISSANCE.md`: Metadata repositori, monorepo Next.js + CLI, env contract, pragma SQLite WAL.
2. `02-SOURCE-CODE-ARCHITECTURE.md`: Arsitektur heksagonal, pemisahan lapisan `src/` (Dashboard) vs `open-sse/` (Engine Proxy).
3. `03-CORE-ROUTING-ENGINE.md`: Siklus hidup request 12 fase, format intermediate pivot, seleksi akun.
4. `04-PROVIDER-ADAPTER-ARCHITECTURE.md`: `BaseExecutor`, `DefaultExecutor`, adapter 140+ penyedia LLM (OpenAI, Anthropic, Gemini, Nous, dll.).
5. `05-MODEL-DISCOVERY-AND-MANAGEMENT.md`: Endpoint `/v1/models`, dynamic resolvers, basis data kapabilitas `capabilities.js`.
6. `06-SMART-ROUTING-COMBO-AND-FALLBACK.md`: Two-tier fallback routing, multi-account rotation, empty stream failover (`emptyFailover.js`).
7. `07-API-GATEWAY-AND-PROTOCOL.md`: Paritas 100% dengan OpenAI REST/SSE API (`/v1/chat/completions`), streaming token deltas & tool calls.
8. `08-DASHBOARD-AND-UI-ARCHITECTURE.md`: Antarmuka Next.js 16 / React 19 UI, Zustand state store, Recharts visualizer.
9. `09-AUTHENTICATION-AND-SECURITY.md`: Analisis autentikasi dashboard bcrypt, risiko penyimpanan token plaintext dalam SQLite.
10. `10-DATA-STORAGE-AND-STATE.md`: Skema tabel SQLite (`9router.db`), pragma WAL, in-memory state caching.
11. `11-PERFORMANCE-AND-RELIABILITY.md`: Connection pooling via `undici`, streaming backpressure, tuning timeout TTFT 200 detik.
12. `12-HERMES-INTEGRATION-ANALYSIS.md`: Verifikasi kompatibilitas 100% jalur inferensi Hermes ↔ 9Router (`127.0.0.1:20128/v1`).
13. `13-HIME-INTEGRATION-FEASIBILITY.md`: Desain topologi **Dual-Control Plane** dan visualisasi failover pada Dynamic Island.
14. `14-FINAL-REVERSE-ENGINEERING-REPORT.md`: Sintesis arsitektural master dan jawaban atas 12 pertanyaan fundamental.

### C. Catatan Keamanan Kritis: Risiko RSK-08 (`x-9r-cli-token`)
Pemeriksaan source code pada `src/dashboardGuard.js:16-20,236` dan `src/shared/utils/machineId.js:49-54` mengonfirmasi:
- Token mesin internal `x-9r-cli-token` dibentuk dari `SHA256(rawMachineId + "9r-cli-auth" + cliSecret)[0..16]`.
- File rahasia `cli-secret` disimpan di `%APPDATA%\9router\auth\cli-secret` dalam bentuk plaintext tanpa ACL Windows.
- **Peringatan Tata Kelola & Keamanan:** Token ini memberikan otorisasi penuh (*broad authorization*) ke seluruh rute `/api/*` (termasuk reset password, ekstraksi database, dan mematikan server). Token ini **TIDAK** boleh dianggap sebagai API publik yang aman. Hime akan memperlakukannya sebagai risiko tinggi (RSK-08) dan menerapkan mitigasi isolasi proses anak serta proteksi ACL Windows pada Phase 05–06.

---

## 7. Research & Documentation Inventory

Pemeriksaan komprehensif terhadap seluruh dokumentasi di repositori Hime:

| Kategori Dokumen | Status Dokumen | Jalur Berkas Relatif | Evaluasi & Rekomendasi Sumber Kebenaran |
|:---|:---:|:---|:---|
| **Tata Kelola Agen** | ACTIVE | `AGENTS.md` | Otoritas tertinggi aturan permanen AI agent. Menjadi sumber kebenaran utama. |
| **Status Proyek** | ACTIVE | `PROJECT_STATUS.md` | Sumber kebenaran resmi status fase berdasarkan 16 fase Master Plan. |
| **Master Plan & Roadmap** | ACTIVE (Implicit) | `AGENTS.md` & `PROJECT_STATUS.md` | Folder `docs/master-plan/` saat ini hanya berisi `.gitkeep`. Master plan 16 fase tertanam di `AGENTS.md` & `PROJECT_STATUS.md`. Disarankan menerbitkan berkas formal `docs/master-plan/ENGINEERING_MASTER_PLAN.md` pada Phase 04. |
| **Kebijakan Governance** | ACTIVE | `docs/governance/` (10 berkas) | Lengkap, konsisten, dan terverifikasi di Phase 00. |
| **Pustaka Skill Registry** | ACTIVE | `docs/skills/SKILL_REGISTRY.md` | Sumber kebenaran kanonikal kurasi skill AI di `.agent/skills/`. |
| **Acceptance Phase 00** | DUPLICATE / HIERARCHICAL | `project-management/phases/PHASE-00-ACCEPTANCE.md`<br>vs<br>`project-management/reviews/PHASE-00-ACCEPTANCE.md` | **Temuan Duplikasi:** Versi di `reviews/` memuat revisi wording final untuk risiko RSK-08. **Rekomendasi:** Tetapkan `reviews/PHASE-00-ACCEPTANCE.md` sebagai *Single Source of Truth* untuk acceptance Phase 00. |
| **Risk Register** | ACTIVE | `project-management/risks/RISK_REGISTER.md` | Memuat RSK-01 hingga RSK-08. Konsisten dan mutakhir. |
| **Decision Log** | ACTIVE | `project-management/decisions/DECISION_LOG.md` | Memuat keputusan DEC-01 hingga DEC-08. Konsisten. |
| **Backlog Proyek** | ACTIVE | `project-management/backlog/BACKLOG.md` | Terperinci per fase. Konsisten dengan status `ACCEPTED` pada TASK-01-01. |
| **Spesifikasi Task** | ACTIVE | `project-management/tasks/` | Memuat TASK-00-01, TASK-00-02, TASK-01-01, dan template. |
| **Laporan Review Formal**| ACTIVE | `project-management/reviews/` | Memuat laporan TASK-00-02, TASK-01-01, dan TASK-01-02. |
| **Riset Nusa** | ACTIVE | `RE/Nusa/` (8 laporan) | Lengkap, tidak ada konflik, konsisten merekomendasikan reimplementasi Tauri 2.0. |
| **Riset Hermes** | ACTIVE | `RE/Hermes/reports/` (5 laporan) | Lengkap, konsisten merekomendasikan Named Pipe IPC. |
| **Riset 9Router** | ACTIVE | `RE/9ROUTER/` (14 laporan) | Lengkap, konsisten merekomendasikan topologi Dual-Control Plane. |
| **Spesifikasi Arsitektur**| PLANNED | `docs/architecture/` (.gitkeep) | Belum tersedia (dijadwalkan resmi untuk dikerjakan pada Phase 05). |
| **Spesifikasi Produk** | PLANNED | `docs/master-plan/` (.gitkeep) | Belum tersedia (dijadwalkan resmi untuk dikerjakan pada Phase 04). |
| **ADR (Decision Records)** | PLANNED | `docs/adr/` (ADR_TEMPLATE.md) | Belum ada rekaman individual; keputusan saat ini dicatat di `DECISION_LOG.md`. Formal ADR akan dimulai pada Phase 05. |

---

## 8. Third-Party Dependency & License Inventory

Audit inventarisasi dependensi dan lisensi terhadap seluruh komponen eksternal:

| Komponen / Library | Versi Terverifikasi | Sumber / Asal | Lisensi Terverifikasi | Penggunaan Aktual | Potensi Risiko Lisensi & Kewajiban |
|:---|:---|:---|:---:|:---|:---|
| **Nusa Dynamic Island** | `1.2.0.0` | Rama (`rama2555.github.io/nusa`) | **Proprietary / Closed Source** | Referensi desain & perilaku antarmuka | **Risiko:** Penggunaan kode biner dilarang. **Kewajiban:** Reimplementasi *clean-room* penuh via Tauri 2.0 tanpa menyalin aset biner/kode milik Rama. |
| **Hermes Agent** | Commit `c5f347eb` | `NousResearch/hermes-agent` | **MIT License** | Subproses runtime agen penalaran | **Sangat Aman:** Lisensi MIT sangat permisif. Wajib menyertakan pemberitahuan hak cipta Nous Research saat distribusi. |
| **9Router** | `v0.5.112` (commit `1fe586bb`) | `decolua/9router` | **MIT License** | Subproses AI gateway & fallback proxy | **Sangat Aman:** Lisensi MIT sangat permisif. Wajib menyertakan pemberitahuan hak cipta decolua saat distribusi. |
| **Tauri 2.0 (Target Framework)** | v2.x (Phase 06) | Tauri Programme | **MIT / Apache 2.0** | Shell aplikasi desktop native Hime | **Sangat Aman:** Dual license permisif standar industri Rust. |
| **Curated Agent Skills (7 skills)** | Latest | GitHub Open Community | **MIT License** | Pedoman dan prosedur rekayasa agen AI | **Sangat Aman:** 100% berkas Markdown instruksional tanpa kode tersembunyi. |

---

## 9. Pemetaan Hubungan Aset (Asset Relationship Mapping)

Diagram di bawah ini menggambarkan arsitektur hubungan antar seluruh aset fisik, dependensi eksternal, dan dokumen rekayasa Hime Assistant:

```mermaid
flowchart TD
    subgraph References ["ASET REFERENSI (CLEAN-ROOM SOURCE)"]
        NUSA["Nusa Dynamic Island (v1.2.0.0)<br>Proprietary Closed-Source Executable<br>[REBUILD INDEPENDENTLY]"]
        NUSA_RE["Nusa RE Suite (8 Laporan di RE/Nusa/)<br>[RETAIN AS DESIGN BLUEPRINT]"]
        NUSA -.->|Dianalisis & Diekstraksi| NUSA_RE
    end

    subgraph HimeEcosystem ["EKOSISTEM HIME ASSISTANT (YANG AKAN DIBANGUN)"]
        HIME_DESKTOP["Hime Desktop (Tauri 2.0 Rust + Web)<br>Window Management, DirectComposition Island, Tray HUD<br>[BUILD CLEAN-ROOM]"]
        HIME_CORE["Hime Core Orchestration Layer<br>Process Supervisor, DPAPI Vault, Named Pipe Broker<br>[BUILD FROM SCRATCH]"]
        HIME_DESKTOP <-->|Tauri IPC / State| HIME_CORE
    end

    subgraph ExternalEngines ["KOMPONEN EKSTERNAL (INTEGRATE AS-IS)"]
        HERMES["Hermes Agent (Python Daemon)<br>NousResearch commit c5f347eb<br>[INTEGRATE, DO NOT REBUILD]"]
        NINEROUTER["9Router AI Gateway (Node.js Service)<br>decolua v0.5.112 commit 1fe586bb<br>[INTEGRATE, DO NOT REBUILD]"]
        HERMES_RE["Hermes RE Suite (5 Laporan)<br>[RETAIN]"]
        NINEROUTER_RE["9Router RE Suite (14 Laporan)<br>[RETAIN]"]
    end

    subgraph ExternalCloud ["LAYANAN CLOUD EKSTERNAL"]
        LLM_PROVIDERS["Penyedia Model AI Cloud<br>(OpenAI, Anthropic, Gemini, Nous, DeepSeek, Groq)"]
    end

    %% Hubungan Desain
    NUSA_RE ==>|Mengarahkan Desain UI/UX| HIME_DESKTOP

    %% Hubungan IPC
    HIME_CORE <-->|Win32 Named Pipe (Session Tickets)| HERMES
    HIME_CORE -.->|Process Supervision & CSPRNG Password| NINEROUTER

    %% Hubungan Inferensi
    HERMES -->|Inference HTTP: localhost:20128/v1| NINEROUTER
    NINEROUTER -->|WAN HTTPS Proxying| LLM_PROVIDERS

    %% Hubungan Persistensi
    HERMES -->|State WAL & FTS5| HERMES_DB[(Hermes SQLite DB)]
    NINEROUTER -->|Accounts & Logs| NINEROUTER_DB[(9Router SQLite DB)]
    HIME_CORE -->|Master Keys & Tokens| DPAPI_VAULT[(Windows DPAPI Vault)]

    classDef retain fill:#e1f5fe,stroke:#0288d1,stroke-width:2px;
    classDef integrate fill:#e8f5e9,stroke:#388e3c,stroke-width:2px;
    classDef rebuild fill:#fff3e0,stroke:#f57c00,stroke-width:2px;
    classDef build fill:#f3e5f5,stroke:#7b1fa2,stroke-width:2px;

    class NUSA_RE,HERMES_RE,NINEROUTER_RE retain;
    class HERMES,NINEROUTER integrate;
    class NUSA,HIME_DESKTOP rebuild;
    class HIME_CORE build;
```

### Klasifikasi Perlakuan Komponen:
1. **Dipertahankan (*Retained*):** Seluruh 27 laporan riset `RE/`, 10 dokumen governance, dan 7 curated skills.
2. **Diintegrasikan (*Integrated as-is*):** Hermes Agent (Python daemon via `uv`) dan 9Router (Node.js daemon via child process).
3. **Dibangun Ulang (*Rebuilt Independently / Clean-Room*):** Pengalaman Dynamic Island Nusa menggunakan Tauri 2.0 (Rust) + Webview.
4. **Dibangun dari Nol (*Built from Scratch*):** Hime Core (Supervisor proses, Named Pipe client broker, DPAPI Credential Vault).
5. **Belum Diputuskan (*Undecided / Planned*):** Mekanisme sinkronisasi Android Mobile Layer (Phase 11) dan spesialisasi sub-agen (Phase 14).

---

## 10. Consolidation & Gap Analysis

Berdasarkan inventarisasi menyeluruh di atas, dilakukan evaluasi terhadap 10 pertanyaan kritis tata kelola:

1. **Aset apa yang sudah siap digunakan?**
   - Repositori Git root, aturan `AGENTS.md`, dokumen tata kelola rekayasa, 7 curated skills, seluruh materi riset reverse engineering di `RE/`, source code Hermes Agent, dan source code 9Router.
2. **Aset apa yang perlu diverifikasi lebih lanjut?**
   - Mekanisme sandboxing dan ACL token CLI 9Router (`x-9r-cli-token`) sebelum integrasi daemon pada Phase 06.
   - Pustaka WinRT audio session pada Rust (`windows-rs`) untuk menangkap status media Windows tanpa mengorbankan stabilitas shell.
3. **Aset apa yang belum tersedia?**
   - Dokumen formal spesifikasi produk PRD (`docs/master-plan/`), dokumen blueprint arsitektur sistem (`docs/architecture/`), compiler Rust & MSVC Build Tools pada host Windows (Phase 06).
4. **Penelitian apa yang masih memiliki pertanyaan terbuka?**
   - Formulasi presisi kurva animasi DirectComposition Tauri 2.0 agar mencapai paritas 60 FPS setara Nusa native.
   - Penanganan perilaku Doze Mode Android untuk koneksi remote execution Hermes di masa mendatang (Phase 11).
5. **Apakah ada pekerjaan yang sudah pernah dilakukan tetapi belum terdokumentasi?**
   - Tidak ada. Semua pengujian biner Nusa, Hermes, dan 9Router telah terpetakan dalam 27 laporan resmi.
6. **Apakah ada konflik antar-dokumen?**
   - Terdapat redudansi kecil antara draft awal `project-management/phases/PHASE-00-ACCEPTANCE.md` dan dokumen final `project-management/reviews/PHASE-00-ACCEPTANCE.md`. Dokumen di `reviews/` telah ditetapkan sebagai satu-satunya sumber kebenaran (*Single Source of Truth*).
7. **Apakah ada dependency atau aset yang berpotensi menimbulkan risiko lisensi?**
   - Biner Nusa berstatus proprietary. Risiko dieliminasi 100% dengan prinsip *zero binary reuse* (reimplementasi clean-room). Dependensi Hermes dan 9Router berlisensi MIT (sangat aman).
8. **Apakah ada duplikasi fungsi antara Hermes, Hime Core, dan 9Router?**
   - Ditemukan potensi tumpang tindih: Hermes memiliki adapter LLM langsung (`agent/providers/`), sementara 9Router juga merupakan gateway penyedia LLM.
   - **Keputusan:** Hermes akan diinstruksikan hanya memanggil 9Router (`OPENAI_BASE_URL=http://127.0.0.1:20128/v1`), menyerahkan seluruh urusan rotasi akun, fallback model, dan kuota provider kepada 9Router.
9. **Apakah ada komponen yang sebaiknya tidak dibangun ulang?**
   - **Hermes Agent** dan **9Router** TIDAK BOLEH dibangun ulang dari nol. Membangun ulang reasoning loop atau gateway 140+ provider adalah pemborosan waktu yang masif. Cukup diaudit, dikonfigurasi, dan disupervisi.
10. **Apakah ada kebutuhan penelitian tambahan sebelum masuk ke Product Specification?**
    - Tidak ada. Seluruh hasil penemuan teknis sudah sangat memadai untuk merumuskan PRD dan Use Case pada Phase 04.

### Tabel Analisis Kesenjangan (Gap Analysis Matrix)

| ID | Gap / Finding | Impact | Priority | Recommended Action |
|:---:|:---|:---|:---:|:---|
| **GAP-01** | **Ketiadaan Compiler Rust & MSVC Build Tools di Host** | Memblokir proses kompilasi kode Tauri 2.0 desktop shell | MEDIUM (Deferred) | Pasang `rustup-init.exe` dan MSVC C++ Build Tools pada **Phase 06 (TASK-06-01)**. Tidak memblokir Phase 01–05. |
| **GAP-02** | **Risiko Keamanan Token CLI 9Router (RSK-08)** | Potensi pembacaan token mesin oleh proses lokal lain | HIGH | Rancang Windows NTFS ACL hardening dan sandbox proses anak pada **Phase 05 (TASK-05-04)**. |
| **GAP-03** | **Duplikasi Dokumen Acceptance Phase 00** | Kebingungan referensi formal antara folder `phases/` vs `reviews/` | LOW | Tetapkan `project-management/reviews/PHASE-00-ACCEPTANCE.md` sebagai *Single Source of Truth* di seluruh dokumen pelacak. |
| **GAP-04** | **Belum Adanya Dokumen Spesifikasi PRD Resmi di `docs/master-plan/`** | Visi produk dan use case masih tersebar di `AGENTS.md` dan laporan riset | HIGH | Susun dokumen PRD resmi pada **Phase 04 (TASK-04-01 & TASK-04-02)**. |
| **GAP-05** | **Duplikasi Pemanggilan Provider LLM (Hermes vs 9Router)** | Potensi redundansi logika pemanggilan AI | MEDIUM | Tetapkan konfigurasi wajib `OPENAI_BASE_URL` Hermes mengarah ke loopback 9Router (`127.0.0.1:20128/v1`). |

---

## 11. Open Questions (Pertanyaan Terbuka)

1. **Format Sinkronisasi Data Antar-Perangkat (Desktop ↔ Mobile):** Apakah sinkronisasi to-do, riwayat, dan memori agen antara Hime Desktop (Windows) dan Hime Mobile (Android) akan menggunakan SQLite direct sync via CRDT / peer-to-peer WebSocket, atau melalui broker cloud terenkripsi end-to-end? (Akan dirumuskan pada Phase 05 / Phase 11).
2. **Mekanisme Otentikasi Web Dashboard 9Router bagi Pengguna Akhir:** Apakah dashboard web 9Router (port 20127) akan dibuka untuk diakses langsung oleh pengguna Hime melalui browser untuk manajemen akun AI, atau seluruh manajemen provider akan dibungkus (*wrapped*) secara transparan di dalam UI Hime Desktop? (Akan diputuskan pada Phase 04 / Phase 05).

---

## 12. Rekomendasi Langkah Rekayasa (Recommendations)

1. **Selesaikan Phase 01 Secara Berurutan:** Lanjutkan ke **TASK-01-03 (Dependency and License Audit)** untuk mengunci versi dependensi dan memverifikasi kompatibilitas pustaka pihak ketiga secara formal.
2. **Pertahankan Kebijakan Isolasi Repositori:** Jangan pernah memasukkan sub-repositori `hermes-agent` atau `9router` ke dalam pelacakan Git root Hime. Biarkan keduanya tetap terisolasi di `RE/` sebagai bahan referensi dan runtime.
3. **Gunakan Asset Register Sebagai Referensi Utama:** Perbarui [`project-management/inventory/ASSET-REGISTER.md`](file:///x:/folder_ai/Hime/project-management/inventory/ASSET-REGISTER.md) setiap kali ada penambahan artefak penting baru.

---

## 13. Referensi Bukti (Evidence References)

- **Repositori & Git:**
  - Git Root Log: `85ec774 docs(discovery): accept task 01-01 environment baseline`
  - Aturan Filter: [`.gitignore`](file:///x:/folder_ai/Hime/.gitignore)
- **Aset Nusa:**
  - Biner Asli: `RE/Nusa/00-source-materials/NusaSetup.exe`
  - Inno Setup Script: `RE/Nusa/00-source-materials/extracted/install_script.iss`
  - Laporan Master: [`RE/Nusa/reports/FINAL-REVERSE-ENGINEERING-REPORT.md`](file:///x:/folder_ai/Hime/RE/Nusa/reports/FINAL-REVERSE-ENGINEERING-REPORT.md)
- **Aset Hermes:**
  - Repositori: `RE/Hermes/00-source-materials/hermes-agent/` (commit `c5f347eb`)
  - Lisensi: `RE/Hermes/00-source-materials/hermes-agent/LICENSE` (MIT)
  - Laporan Master: [`RE/Hermes/reports/REPORT-05 - REVERSE ENGINEERING EXECUTIVE SUMMARY.md`](file:///x:/folder_ai/Hime/RE/Hermes/reports/REPORT-05%20-%20REVERSE%20ENGINEERING%20EXECUTIVE%20SUMMARY.md)
- **Aset 9Router:**
  - Repositori: `RE/9ROUTER/00-source-materials/9router/` (commit `1fe586bb`, v0.5.112)
  - Lisensi: `RE/9ROUTER/00-source-materials/9router/LICENSE` (MIT)
  - Laporan Master: [`RE/9ROUTER/14-FINAL-REVERSE-ENGINEERING-REPORT.md`](file:///x:/folder_ai/Hime/RE/9ROUTER/14-FINAL-REVERSE-ENGINEERING-REPORT.md)
- **Pustaka Skill:**
  - Indeks Kanonikal: [`docs/skills/SKILL_REGISTRY.md`](file:///x:/folder_ai/Hime/docs/skills/SKILL_REGISTRY.md)
  - Lokasi Fisik: `.agent/skills/` (7 skills tervalidasi)
- **Indeks Aset Baru:**
  - Berkas Indeks: [`project-management/inventory/ASSET-REGISTER.md`](file:///x:/folder_ai/Hime/project-management/inventory/ASSET-REGISTER.md)

---

## 14. Status Akhir Tugas (Final Task Status)

- **Task ID:** TASK-01-02
- **Hasil Audit:** 100% Selesai dan Tervalidasi Berbasis Bukti
- **Vonis Rekomendasi:** **`[VERIFIED]` ACCEPTED**
- **Penetapan Indeks:** `project-management/inventory/ASSET-REGISTER.md` resmi ditetapkan sebagai indeks aset utama proyek Hime.
- **Tindak Lanjut:** Melanjutkan audit lisensi formal ke TASK-01-03.
