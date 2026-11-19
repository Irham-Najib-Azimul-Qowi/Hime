# TASK-02-04C — Final Consistency Correction & Acceptance Readiness Report

**Proyek:** Hime — Personal AI Desktop Environment
**Fase:** 02 — Nusa Reverse Engineering & Architecture Discovery (Product Specification)
**Task ID:** TASK-02-04C (Koreksi Konsistensi Akhir & Kesiapan Penerimaan)
**Status Tata Kelola:** `COMPLETE — PENDING OWNER REVIEW`
**Kebijakan Eksekusi:** ZERO COMMIT | ZERO PUSH | NO HIME IMPLEMENTATION
**Tanggal Penyelesaian:** 2026-11-08
**Dokumen Terkait:** Seluruh 13 Dokumen Spesifikasi Produk di [`project-management/product-requirements/`](file:///x:/folder_ai/Hime/project-management/product-requirements/)

---

## 1. Executive Summary & Ringkasan Eksekutif

Laporan ini mendokumentasikan penyelesaian formal **TASK-02-04C — Final Consistency Correction & Acceptance Readiness**, yang merupakan tindak lanjut terarah berdasarkan evaluasi kritis Project Owner terhadap TASK-02-04B.

Tujuan utama dari task ini adalah menyelesaikan anomali administratif penghitungan kebutuhan, mengkalibrasi akurasi bukti keamanan 9Router berdasarkan artefak audit Phase 01 yang sah, memperdalam spesifikasi teknis keamanan IPC Named Pipe, mengkualifikasikan batas nyata atomisitas sistem berkas (*filesystem atomicity boundaries*), menetapkan kebijakan penanganan kegagalan benchmark performa, serta memastikan keselarasan 100% lintas dokumen spesifikasi produk Hime.

Seluruh revisi dilakukan secara presisi tanpa mengubah keputusan produk yang telah disetujui, tanpa melakukan reverse engineering tambahan, dan mematuhi batas tata kelola ketat (*zero commit, zero push, no source code implementation*).

---

## 2. Rekonsiliasi Jumlah Kebutuhan Fungsional (Requirement Count Reconciliation)

### A. Temuan Anomali Penghitungan
Pada laporan TASK-02-04B sebelumnya, terdapat inkonsistensi administratif di mana teks laporan menyebut "32 total requirements", namun rincian kategori MoSCoW mencantumkan 24 Must Have, 5 Should Have, dan 4 Could Have ($24 + 5 + 4 = 33$).

### B. Hasil Audit Forensik Dokumen Aktual
Pemeriksaan menyeluruh berbasis script parser terhadap file aktual [`HIME-FUNCTIONAL-REQUIREMENTS.md`](file:///x:/folder_ai/Hime/project-management/product-requirements/HIME-FUNCTIONAL-REQUIREMENTS.md) mengonfirmasi:
- **Total Heading Kebutuhan (`#### FR-`):** **Tepat 33 Kebutuhan Unik**.
- **Duplikasi ID:** **0 ID Duplikat** (Nol duplikasi terdeteksi).
- **ID yang Hilang:** **0 ID Hilang** (Seluruh 33 ID terdefinisi lengkap dari FR-CORE hingga FR-XPLAT).
- **Rincian Klasifikasi MoSCoW Aktual:**
  1. **MUST HAVE (Kritis untuk Kelayakan Produk / MVP):** **24 Kebutuhan**
     `FR-CORE-01`, `FR-CORE-02`, `FR-CORE-03`, `FR-AGENT-01`, `FR-AGENT-02`, `FR-FILE-01`, `FR-FILE-02`, `FR-FILE-03`, `FR-FILE-04`, `FR-DOC-01`, `FR-PROD-01`, `FR-DEV-01`, `FR-DEV-02`, `FR-MEM-01`, `FR-MEM-02`, `FR-NOTIF-01`, `FR-NOTIF-02`, `FR-SEC-01`, `FR-SEC-02`, `FR-SEC-03`, `FR-SYS-01`, `FR-SYS-02`, `FR-SYS-03`, `FR-XPLAT-01`.
  2. **SHOULD HAVE (Penting, Tidak Menahan MVP Awal / Phase 2):** **5 Kebutuhan**
     `FR-AGENT-03`, `FR-BROWSER-01`, `FR-DEVICE-01`, `FR-DEVICE-02`, `FR-XPLAT-02`.
  3. **COULD HAVE (Fitur Lanjutan / Plugin Opsional Phase 2 & 3):** **4 Kebutuhan**
     `FR-DOC-02`, `FR-BROWSER-02`, `FR-PROD-02`, `FR-PROD-03`.
  4. **WON'T HAVE (Batasan Inisiatif Di Luar Cakupan):** **3 Inisiatif**
     *Autonomous System Takeover*, *Public P2P Cloud*, *Local 70B LLM Native Hosting*.
- **Total Rekonsiliasi:** **$24 + 5 + 4 = 33$ Kebutuhan Fungsional Aktif**.

Seluruh dokumen yang sebelumnya merujuk angka 28 atau 32 telah diperbaiki dan disinkronkan menjadi angka definitif **33 Kebutuhan Fungsional**.

---

## 3. Akurasi & Klasifikasi Bukti Keamanan 9Router (Phase 01 Alignment)

Berdasarkan audit ulang terhadap dokumen Phase 01 ([`TASK-01-04-SECURITY-RUNTIME-AUDIT.md`](file:///x:/folder_ai/Hime/project-management/reviews/TASK-01-04-SECURITY-RUNTIME-AUDIT.md), [`SECURITY-BASELINE.md`](file:///x:/folder_ai/Hime/project-management/security/SECURITY-BASELINE.md), [`THREAT-MODEL.md`](file:///x:/folder_ai/Hime/project-management/security/THREAT-MODEL.md)) serta pemeriksaan kode sumber statis 9Router pada `RE/9ROUTER/00-source-materials/9router/`, temuan keamanan dikalibrasi secara ketat:

### A. Lokasi dan Nama Berkas Basis Data Kanonikal
- **Temuan Bukti Kode Sumber (`src/lib/dataDir.js` & `src/lib/db/paths.js`):**
  - Pada Windows (`win32`): `DATA_DIR` secara default mengarah ke `%APPDATA%\9router`. Subdirektori basis data adalah `db/` dengan berkas fisik **`data.sqlite`**. Jalur kanonikal Windows:
    **`%APPDATA%\9router\db\data.sqlite`** (`[VERIFIED STATIC]`).
  - Pada Linux/macOS: `DATA_DIR` mengarah ke `~/.9router`, dengan berkas fisik **`~/.9router/db/data.sqlite`** (`[VERIFIED STATIC]`).
- **Koreksi Tata Kelola:** Penyebutan kolokial seperti `9router.db` atau direktori generik `.9router/` pada draf sebelumnya telah dikoreksi di seluruh dokumen menjadi jalur resmi **`%APPDATA%\9router\db\data.sqlite`**.

### B. Matriks Klasifikasi Bukti Keamanan 9Router

| Dimensi Keamanan | Bukti Teknis & Lokasi Kode Sumber | Klasifikasi Bukti | Status Pengujian Runtime | Mitigasi Terencana Hime (`[PLANNED CONTROL]`) |
|:---|:---|:---:|:---:|:---|
| **Penyimpanan Kredensial Penyedia (Plaintext)** | `src/lib/db/repos/connectionsRepo.js` menyimpan JSON stringify di kolom `data` tabel `providerConnections`. | **`[VERIFIED STATIC]`** (CWE-312) | Terkonfirmasi pada audit statis Phase 01. | Pembungkus DPAPI / Secret Service Hime Core; isolasi Windows ACL `0700` pada direktori `%APPDATA%\9router\`. |
| **Bypass Autentikasi Loopback (Localhost Bypass)** | `src/dashboardGuard.js:161-165`: `if (isLocalRequest(request)) return true;` meloloskan request loopback tanpa periksa API key. | **`[VERIFIED STATIC]`** | Terkonfirmasi pada audit kode Phase 01. | Token sesi dinamis (*Session Bearer Token*) per-sesi; evaluasi internal fork wrapper pada Phase 05 jika upstream menolak patch. |
| **Fallback Password Dashboard Bawaan** | `src/app/api/auth/login/route.js:58`: `process.env.INITIAL_PASSWORD || "123456"`. | **`[VERIFIED STATIC — SOURCE FALLBACK]`** | **`[NOT TESTED RUNTIME]`** (Belum ada live deployment Hime). | Hime Process Supervisor menginjeksi token acak 32 karakter dari generator CSPRNG via env var `INITIAL_PASSWORD` saat peluncuran proses anak. |
| **Pembuatan Token Mesin CLI (`x-9r-cli-token`)** | `src/shared/utils/machineId.js:49-54`: Diturunkan dari `SHA256(machineId + "9r-cli-auth" + cliSecret)[0..16]`. | **`[VERIFIED STATIC]`** | Terkonfirmasi pada audit kode Phase 01. | Hime Core tidak menggunakan CLI wrapper untuk kontrol proses; mengandalkan job object sinyal internal. |
| **Hak Istimewa Token CLI** | `src/dashboardGuard.js:145-149` meloloskan akses ke `/api/settings/database` (ekspor DB penuh) dan `/api/shutdown`. | **`[VERIFIED STATIC]`** | Terkonfirmasi pada audit kode Phase 01. | File secret di `%APPDATA%\9router\auth\cli-secret` dikunci dengan ACL pengguna lokal saja. |
| **Default Host Binding `0.0.0.0`** | `cli/cli.js:110`: `defaultHost = "0.0.0.0"`. | **`[VERIFIED STATIC]`** | Terkonfirmasi pada audit kode Phase 01. | Supervisor mengeksekusi dengan argumen strictly `--host 127.0.0.1` + verifikasi socket aktif via Win32 `GetExtendedTcpTable` (`AF_INET`/`AF_INET6`). |

---

## 4. Spesifikasi Keamanan IPC Named Pipe (Hermes ↔ Hime Core)

Spesifikasi IPC Named Pipe pada [`HIME-AGENT-EXPERIENCE-SPECIFICATION.md`](file:///x:/folder_ai/Hime/project-management/product-requirements/HIME-AGENT-EXPERIENCE-SPECIFICATION.md) telah diperdalam dengan rincian arsitektural:

1. **Pembedaan Konseptual Tegas:**
   - **Autentikasi (*Authentication*):** Membuktikan bahwa proses yang terhubung ke Named Pipe benar-benar proses anak resmi Hermes Agent.
   - **Otorisasi (*Authorization*):** Menentukan apakah aksi alat yang diminta diizinkan berdasarkan hak akses 8 izin granular dan konfirmasi pengguna.
   - **Isolasi Proses (*Process Isolation*):** Memisahkan memori dan lingkungan runtime antara Hime Core dan sub-proses Python Hermes.
   - **Penegakan Kebijakan (*Policy Enforcement*):** Hime Core bertindak sebagai satu-satunya pusat penegakan kebijakan (*Policy Decision Point & Policy Enforcement Point*).
2. **Kualifikasi Batasan DACL Named Pipe:**
   - Named Pipe `\\.\pipe\hime-hermes` dikonfigurasi dengan DACL `Current User SID` dan `Network Access Denied`.
   - **Peringatan Keamanan Kritis:** DACL berbasis User SID **TIDAK MENGISOLASI ancaman dari proses lain yang berjalan di bawah akun pengguna yang sama (Standard User, Medium Integrity Level)**. Proses lokal liar dari user yang sama tetap dapat mencoba membuka koneksi ke named pipe jika mengetahui namanya.
3. **Verifikasi Identitas Proses Klien (Client PID Verification):**
   - Hime Core memanggil Win32 `GetNamedPipeClientProcessId(pipe_handle, &client_pid)` segera setelah koneksi diterima.
   - `client_pid` dicocokkan secara deterministik dengan child PID Hermes Agent yang disimpan oleh Process Supervisor saat pemanggilan `CreateProcessW`. Jika PID tidak cocok, koneksi seketika diputus (`DisconnectNamedPipe`).
4. **Autentikasi Handshake & Token Efemeral 256-Bit:**
   - Process Supervisor menyuntikkan token CSPRNG 256-bit ke environment block Hermes saat launch.
   - Hermes wajib mengirimkan handshake terstruktur dalam $\le 5000\text{ ms}$; kegagalan token atau timeout memicu pemutusan pipa (*fail-closed*).
5. **Ketiadaan Jalur Eksekusi Alat Alternatif (*No Direct Bypass Path*):**
   - Hermes Agent dilarang keras memiliki jalur eksekusi alat mandiri (`subprocess.run`, `os.system`). Seluruh pemanggilan alat dialihkan melalui envelope IPC JSON ke Hime Core.
6. **Perlindungan Replay & Korelasi Permintaan:**
   - Setiap pesan membawa `request_id`, `correlation_id` (UUIDv4), `nonce` (counter sekuensial monotonik), dan verifikasi *timestamp drift* ($\le \pm 5\text{ detik}$).
7. **Siklus Hidup & Penanganan Kegagalan IPC:**
   - Status terdefinisi: `UNCONNECTED` $\rightarrow$ `LISTENING` $\rightarrow$ `HANDSHAKING` $\rightarrow$ `AUTHENTICATED_IDLE` $\rightarrow$ `EXECUTING_TOOL` $\rightarrow$ `TERMINATING`.
   - Gangguan koneksi (`ERROR_BROKEN_PIPE`), crash anak, atau timeout (30s) membatalkan transaksi tool secara deterministik dan memicu restart terkontrol via Process Supervisor.
8. **Status Mitigasi Tingkat OS Tambahan:**
   - Peningkatan isolasi proses tingkat OS lanjutan (seperti Windows AppContainer, Windows Restricted Token, Service Account khusus, atau aturan firewall WFP) diklasifikasikan sebagai **`[DESIGN REQUIREMENT / TECHNICAL DEPENDENCY]`** untuk diimplementasikan pada Fase 05/06, BUKAN sebagai kontrol yang telah terverifikasi saat ini.

---

## 5. Kualifikasi Atomisitas Operasi Sistem Berkas & Undo Reliability

Klaim keandalan operasi berkas pada [`HIME-FILE-AND-STORAGE-EXPERIENCE.md`](file:///x:/folder_ai/Hime/project-management/product-requirements/HIME-FILE-AND-STORAGE-EXPERIENCE.md) telah disempurnakan:

1. **Batasan Realistis Operasi Same-Volume Move:**
   - Menggunakan Win32 `MoveFileExW(MOVEFILE_REPLACE_EXISTING)` atau POSIX `rename()`.
   - **Kualifikasi Atomisitas:** Hanya atomik pada level metadata entri direktori pada sistem berkas lokal ber-journal (NTFS, ReFS, ext4).
   - **Kondisi di Mana Atomisitas Gugur:**
     * *Filesystem Non-Journaled (FAT32, exFAT):* Pemutusan daya saat penulisan kluster dapat menyebabkan korupsi direktori atau inkonsistensi entri berkas.
     * *Kunci Berkas (*File Lock*):* Menghasilkan error `ERROR_SHARING_VIOLATION` jika berkas dibuka eksklusif oleh proses lain.
     * *Kegagalan Izin (*Permission Failure*):* Menghasilkan `ERROR_ACCESS_DENIED` jika ACL melarang penghapusan target penimpaan.
     * *Volume Jaringan & Cloud Reparse Points:* Pada SMB shares atau OneDrive Files On-Demand, operasi dapat berubah menjadi salin-dan-hapus di latar belakang tanpa jaminan atomik lokal.
     * *Kegagalan Hardware Media Simpan:* Bad sector atau kerusakan kontroler SSD.
   - **Prinsip Hime:** Hime Core tidak pernah berasumsi bahwa same-volume move selalu atomik secara universal; sistem memeriksa kode hasil operasi dan memverifikasi integritas metadata.
2. **Kualifikasi Operasi Cross-Volume Move (Non-Atomik per Definisi):**
   - Pemindahan antar-volume fisik/logis **secara mendasar tidak dapat bersifat atomik**.
   - Wajib menerapkan Prosedur Multi-Tahap Terverifikasi: (1) Pre-flight disk space check $\rightarrow$ (2) Salin stream ke temporary `.hime_tmp` $\rightarrow$ (3) Verifikasi checksum SHA-256 berkas sumber vs tujuan $\rightarrow$ (4) Atomic rename `.hime_tmp` ke nama final $\rightarrow$ (5) Hapus berkas sumber **hanya setelah verifikasi Tahap 3 dan 4 terbukti sukses 100%**.
   - **Prinsip Utama:** *Zero source file loss before verified destination confirmation*. Berkas sumber tidak pernah disentuh atau dihapus sebelum salinan target valid.
3. **Protokol Pemulihan Pasca-Crash (Crash Recovery):**
   - Tabel SQLite `storage_journal` mencatat state transaksional (`STAGE_COPYING`, `STAGE_VERIFYING`, `STAGE_COMMITTING`, `SUCCESS`).
   - Crash saat `STAGE_COPYING` / `STAGE_VERIFYING`: Berkas sumber 100% utuh. Berkas sisa `.hime_tmp` di volume tujuan dibersihkan saat startup, status dicatat `ABORTED_CLEANED`.
   - Crash saat `STAGE_COMMITTING` (Target final ada, sumber belum terhapus): Kedua berkas ada dan hash-nya identik. Sistem tidak menggandakan data, menyelesaikan penghapusan berkas sumber tertunda, dan menandai status `SUCCESS`.
4. **Proteksi Jurnal Pembatalan (Undo Reliability):**
   - Jika berkas di destinasi telah dimodifikasi atau dihapus oleh pengguna di luar Hime, sistem **DILARANG melakukan Undo otomatis**. Sistem wajib menampilkan dialog konfirmasi eksplisit untuk mencegah penimpaan data baru pengguna secara tidak sengaja.

---

## 6. Kualifikasi Target Sumber Daya & Kebijakan Remediasi Benchmark

Pada [`HIME-NON-FUNCTIONAL-REQUIREMENTS.md`](file:///x:/folder_ai/Hime/project-management/product-requirements/HIME-NON-FUNCTIONAL-REQUIREMENTS.md), status target konsumsi sumber daya ditegaskan kembali:

1. **Status Target Konsumsi RAM Multi-Proses:**
   - Seluruh angka resource footprint dikualifikasikan sebagai **`[PROPOSED TARGET — REQUIRES PHASE 06/07 BENCHMARK VALIDATION]`**.
   - Proposed Target Gabungan: Idle $\le \mathbf{215\text{ MB}}$ (Ambang batas: $310\text{ MB}$), Active $\le \mathbf{470\text{ MB}}$ (Ambang batas: $670\text{ MB}$), Peak $\le \mathbf{690\text{ MB}}$ (Ambang batas: $920\text{ MB}$).
   - Ruang Bebas Host: Menyisakan $> 2.7\text{ GB}$ memori bebas pada laptop pengguna (Core i5-8250U, ~3.7 GB RAM bebas saat baseline).
2. **Kebijakan Remediasi 5-Tahap Kegagalan Benchmark (Benchmark Failure Policy):**
   Apabila benchmark empiris pada fase implementasi melampaui ambang batas penerimaan (*acceptance threshold*), tim rekayasa wajib menjalankan protokol:
   - **Tahap 1 (Profiling Komponen):** Mengisolasi proses spesifik menggunakan Windows Performance Recorder, dhat heap profiler, memory-profiler Python, dan Node.js heap snapshot v8.
   - **Tahap 2 (Identifikasi Penyebab Dominan):** Membedakan apakah lonjakan dipicu oleh overhead multi-proses WebView2, memory leak, buffer chunk streaming, atau alokasi cache SQLite FTS5.
   - **Tahap 3 (Optimasi Rekayasa Non-Destruktif):** Menerapkan penyetelan GC, pemangkasan working set via `SetProcessWorkingSetSize` saat idle, batas cache halaman SQLite, dan lazy-loading modul. *Dilarang keras menonaktifkan keamanan DACL atau verifikasi hash demi mengejar angka RAM.*
   - **Tahap 4 (Pengujian Ulang Terstandarisasi):** Menjalankan ulang suite benchmark otomatis minimum 30 menit pada beban idle, aktif, dan peak.
   - **Tahap 5 (Eskalasi Formal ke Project Owner):** Jika setelah optimasi konsumsi tetap melampaui batas toleransi, menyusun laporan telemetri profiling formal untuk meminta revisi target resmi dari Project Owner.

---

## 7. Hasil Audit Konsistensi Lintas Dokumen (Cross-Document Consistency Matrix)

Pemeriksaan konsistensi komprehensif dilakukan di antara seluruh artefak dokumentasi spesifikasi dan tata kelola:

| Berkas Dokumen | Status Rekonsiliasi FR (33 ID) | Status Keamanan 9Router & IPC | Status Atomisitas & Undo | Status Target RAM / Host |
|:---|:---:|:---:|:---:|:---:|
| [`HIME-FUNCTIONAL-REQUIREMENTS.md`](file:///x:/folder_ai/Hime/project-management/product-requirements/HIME-FUNCTIONAL-REQUIREMENTS.md) | **33 Kebutuhan** (24 Must, 5 Should, 4 Could) | `%APPDATA%\9router\db\data.sqlite`, CSPRNG Initial Password | Same vs cross volume, zero source loss, Undo 30 hari | Sesuai NFR |
| [`HIME-NON-FUNCTIONAL-REQUIREMENTS.md`](file:///x:/folder_ai/Hime/project-management/product-requirements/HIME-NON-FUNCTIONAL-REQUIREMENTS.md) | Sesuai | `%APPDATA%\9router\db\data.sqlite`, mitigasi plaintext DB, 9Router ACL 0700 | Same-volume qualified, non-atomic cross-volume | `[PROPOSED TARGET]` $\le 215/310\text{MB}$ Idle, $\le 470/670\text{MB}$ Active, 5-Step Policy |
| [`HIME-AGENT-EXPERIENCE-SPECIFICATION.md`](file:///x:/folder_ai/Hime/project-management/product-requirements/HIME-AGENT-EXPERIENCE-SPECIFICATION.md) | Sesuai | Named Pipe DACL + User SID, Client PID check, CSPRNG token, Core PEP/PDP, No bypass | N/A | Sesuai NFR |
| [`HIME-FILE-AND-STORAGE-EXPERIENCE.md`](file:///x:/folder_ai/Hime/project-management/product-requirements/HIME-FILE-AND-STORAGE-EXPERIENCE.md) | Sesuai | N/A | Batasan MoveFileExW/POSIX rename, 5-step cross-volume, journal state recovery, Undo pre-check | Sesuai NFR |
| [`HIME-MVP-AND-PRODUCT-ROADMAP.md`](file:///x:/folder_ai/Hime/project-management/product-requirements/HIME-MVP-AND-PRODUCT-ROADMAP.md) | Sesuai 12 Pilar Inti (FR Must Have) | Sentralisasi Hime Core, mitigasi kredensial 9Router | Safe file move, Undo 30 hari | Alokasi RAM $\le 310\text{MB}$ Idle, single task limit |
| [`HIME-PRODUCT-RISKS-AND-OPEN-DECISIONS.md`](file:///x:/folder_ai/Hime/project-management/product-requirements/HIME-PRODUCT-RISKS-AND-OPEN-DECISIONS.md) | Sesuai | `%APPDATA%\9router\db\data.sqlite` (CWE-312), loopback bypass, source fallback "123456" | Multi-step file move, Undo 30 hari | RSK-01 host memory limit, Proposed targets |
| [`HIME-PRODUCT-MODULE-ARCHITECTURE.md`](file:///x:/folder_ai/Hime/project-management/product-requirements/HIME-PRODUCT-MODULE-ARCHITECTURE.md) | Sesuai (14 Modul) | `%APPDATA%\9router\db\data.sqlite` DPAPI wrapper, Core PEP/PDP, anti-slip 500ms | Modul 05 Storage Manager safe lifecycle | Sesuai NFR |
| [`HIME-PRODUCT-VISION.md`](file:///x:/folder_ai/Hime/project-management/product-requirements/HIME-PRODUCT-VISION.md) | Sesuai | Hime Core Policy Enforcement Point, Approval Modal UI, jeda anti-slip 500ms | Safe storage management | Non-intrusive ubiquitous presence |
| [`TASK-02-04-COMPLETION-REPORT.md`](file:///x:/folder_ai/Hime/project-management/product-requirements/TASK-02-04-COMPLETION-REPORT.md) | **33 Kebutuhan** (24 Must, 5 Should, 4 Could) | `%APPDATA%\9router\db\data.sqlite`, Hime Core PEP/PDP | Same vs cross volume, zero source loss, Undo 30 hari | Indeks 14 Deliverables lengkap |
| [`TASK-02-04B-REVIEW-AND-CALIBRATION-REPORT.md`](file:///x:/folder_ai/Hime/project-management/product-requirements/TASK-02-04B-REVIEW-AND-CALIBRATION-REPORT.md) | **33 Kebutuhan** (24 Must, 5 Should, 4 Could) | `%APPDATA%\9router\db\data.sqlite` (CWE-312), fallback "123456" dikualifikasikan | Same vs cross volume move, AC-FAIL-01 s/d 07 | `[PROPOSED TARGET]` 215/470/690 MB terkalibrasi |
| [`PROJECT_STATUS.md`](file:///x:/folder_ai/Hime/PROJECT_STATUS.md) | Sesuai Phase 02 Status | Sesuai | Sesuai | Sesuai |

---

## 8. Inventaris Berkas Deliverables Lengkap (14 Dokumen Spesifikasi Produk)

Direktori [`project-management/product-requirements/`](file:///x:/folder_ai/Hime/project-management/product-requirements/) kini secara resmi memuat paket lengkap 14 artefak spesifikasi produk Hime:

1. [`HIME-PRODUCT-VISION.md`](file:///x:/folder_ai/Hime/project-management/product-requirements/HIME-PRODUCT-VISION.md) — Visi, filosofi produk, 6 prinsip UX, diferensiasi pasar, dan batas tanggung jawab produk.
2. [`HIME-USER-CONTEXT-AND-USE-CASES.md`](file:///x:/folder_ai/Hime/project-management/product-requirements/HIME-USER-CONTEXT-AND-USE-CASES.md) — 11 domain use cases konkret dengan trigger, context, perilaku, izin, dan failure handling.
3. [`HIME-FUNCTIONAL-REQUIREMENTS.md`](file:///x:/folder_ai/Hime/project-management/product-requirements/HIME-FUNCTIONAL-REQUIREMENTS.md) — 33 Functional Requirements unik (24 Must Have, 5 Should Have, 4 Could Have) dan 3 inisiatif Won't Have.
4. [`HIME-NON-FUNCTIONAL-REQUIREMENTS.md`](file:///x:/folder_ai/Hime/project-management/product-requirements/HIME-NON-FUNCTIONAL-REQUIREMENTS.md) — 18 NFR terukur (pemisahan anggaran 4 proses, target vs ambang batas, 5-Step Failure Remediation Policy).
5. [`HIME-PRODUCT-MODULE-ARCHITECTURE.md`](file:///x:/folder_ai/Hime/project-management/product-requirements/HIME-PRODUCT-MODULE-ARCHITECTURE.md) — Arsitektur 14 modul produk, keterkaitan Hime Core, dan dependensi sistem operasi.
6. [`HIME-UX-AND-INTERACTION-ARCHITECTURE.md`](file:///x:/folder_ai/Hime/project-management/product-requirements/HIME-UX-AND-INTERACTION-ARCHITECTURE.md) — Model Kanvas Elastis 3-Tier, aturan transisi, interaksi keyboard-first, dan Click-to-expand default (DIR-01).
7. [`HIME-AGENT-EXPERIENCE-SPECIFICATION.md`](file:///x:/folder_ai/Hime/project-management/product-requirements/HIME-AGENT-EXPERIENCE-SPECIFICATION.md) — Spesifikasi AX Hermes, ketiadaan private CoT, Named Pipe DACL & Client PID verification, Hime Core PEP/PDP.
8. [`HIME-FILE-AND-STORAGE-EXPERIENCE.md`](file:///x:/folder_ai/Hime/project-management/product-requirements/HIME-FILE-AND-STORAGE-EXPERIENCE.md) — Deteksi duplikat 5 dimensi, kualifikasi batasan atomisitas filesystem, safe move lifecycle, dan Undo 30 hari.
9. [`HIME-CROSS-PLATFORM-EXPERIENCE-ARCHITECTURE.md`](file:///x:/folder_ai/Hime/project-management/product-requirements/HIME-CROSS-PLATFORM-EXPERIENCE-ARCHITECTURE.md) — Strategi Windows-first Linux-second, PAL adapter, Wayland/X11, dan fallback anggun DIR-03.
10. [`HIME-MVP-AND-PRODUCT-ROADMAP.md`](file:///x:/folder_ai/Hime/project-management/product-requirements/HIME-MVP-AND-PRODUCT-ROADMAP.md) — Lingkup MVP 12 pilar fondasi, kriteria pembobotan, dan peta jalan produk 4 fase.
11. [`HIME-PRODUCT-RISKS-AND-OPEN-DECISIONS.md`](file:///x:/folder_ai/Hime/project-management/product-requirements/HIME-PRODUCT-RISKS-AND-OPEN-DECISIONS.md) — Risiko 9Router Phase 01, klasifikasi kontrol keamanan, penerapan DIR-01 s/d DIR-04, register OQ-05 s/d 07.
12. [`TASK-02-04-COMPLETION-REPORT.md`](file:///x:/folder_ai/Hime/project-management/product-requirements/TASK-02-04-COMPLETION-REPORT.md) — Laporan Sintesis Eksekutif Induk Penyelesaian TASK-02-04.
13. [`TASK-02-04B-REVIEW-AND-CALIBRATION-REPORT.md`](file:///x:/folder_ai/Hime/project-management/product-requirements/TASK-02-04B-REVIEW-AND-CALIBRATION-REPORT.md) — Laporan Sintesis Peninjauan Arsitektur & Kalibrasi Metrik TASK-02-04B.
14. [`TASK-02-04C-FINAL-CONSISTENCY-REPORT.md`](file:///x:/folder_ai/Hime/project-management/product-requirements/TASK-02-04C-FINAL-CONSISTENCY-REPORT.md) — Laporan Konsistensi Akhir & Kesiapan Penerimaan TASK-02-04C (Dokumen Ini).

---

## 9. Keterbatasan yang Tersisa & Persiapan Fase Berikutnya

1. **Keterbatasan yang Didokumentasikan (*Documented Limitations*):**
   - **Enkripsi Kredensial 9Router Upstream:** Tergantung pada keputusan Phase 05 (OQ-05), apakah mengajukan upstream PR ke repositori `decolua/9router` atau mengimplementasikan *maintained internal fork wrapper* pada Hime Core.
   - **Validasi Kinerja Empiris:** Seluruh proposed target konsumsi memori dan latensi belum melalui benchmark nyata pada build biner Hime; validasi empiris wajib dilaksanakan pada Phase 06/07.
   - **Isolasi OS Tingkat Lanjut:** Penerapan AppContainer atau Windows Restricted Token untuk Hermes Agent diklasifikasikan sebagai *design requirement* yang membutuhkan pengujian kompatibilitas interpreter Python pada Phase 05/06.
2. **Kesiapan Menuju Fase Berikutnya:**
   Paket spesifikasi produk Hime kini berstatus matang, koheren, dan bebas kontradiksi, siap menjadi dasar perancangan arsitektur sistem dan desain keamanan (Phase 05).

---

## 10. Kepatuhan Tata Kelola & Batasan Henti (Stop Condition)

1. **Verifikasi Kebijakan Eksekusi:**
   - [x] **Zero Implementation Code:** Tidak ada kode sumber implementasi aplikasi Hime (Rust/WebView/JS) yang ditulis.
   - [x] **Zero Dependencies:** Tidak ada instalasi dependensi atau paket baru.
   - [x] **Zero External Modification:** Kode Hermes Agent dan 9Router tidak diubah.
   - [x] **Zero Additional Reverse Engineering:** Tidak melakukan bedah biner atau reverse engineering baru.
   - [x] **Zero Commit:** Tidak ada perintah `git commit` yang dieksekusi.
   - [x] **Zero Push:** Tidak ada pengiriman ke remote origin GitHub.
   - [x] **Zero Unauthorized Acceptance:** Status penerimaan formal tetap berada di bawah wewenang Project Owner.
2. **Kondisi Berhenti (*Stop Condition*):**
   - Pekerjaan pada **TASK-02-04C** dinyatakan **SELESAI PENUH** (*Complete*).
   - Status tata kelola ditetapkan sebagai:
     **`COMPLETE — PENDING OWNER REVIEW`**
   - Pengerjaan dihentikan penuh di titik ini. Sistem **TIDAK AKAN** memulai TASK-02-05 secara otomatis sebelum menerima evaluasi dan instruksi resmi dari Project Owner.

---
*Laporan ini disusun secara otomatis dan diverifikasi sesuai pedoman tata kelola proyek Hime.*
