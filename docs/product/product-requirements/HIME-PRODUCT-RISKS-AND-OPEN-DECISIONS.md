# Hime Product Risks & Open Decisions Register (Calibrated)

- **Document ID:** HIME-PRODUCT-RISKS-AND-OPEN-DECISIONS
- **Task ID:** TASK-02-04 / TASK-02-04B / TASK-02-04D (Product Requirements & Experience Architecture)
- **Phase:** Phase 02 — Nusa Reverse Engineering & Architecture Discovery (Product Specification)
- **Authority:** Lead Systems Architect & Senior Risk Governance Specialist
- **Status:** **CALIBRATED SPECIFICATION — PENDING OWNER REVIEW**
- **Date:** 2026-11-08

---

## 1. Executive Summary

Dokumen ini mendokumentasikan pemetaan risiko keamanan dan arsitektur produk (*Product & Security Risks*), ketergantungan teknis (*Technical Dependencies*), arahan keputusan awal dari Project Owner (*Project Owner Directives*), usulan keputusan rekayasa (*Proposed Decisions*), serta register risiko residual (*Residual Risks*) untuk **Hime**.

Berdasarkan evaluasi tata kelola rekayasa (TASK-02-04B), dokumen ini menyajikan analisis keamanan menyeluruh terhadap komponen backend eksternal (terutama gateway **9Router** dan runtime **Hermes Agent**), menyelaraskan temuan risiko dengan hasil audit Phase 01 (`TASK-01-04-SECURITY-RUNTIME-AUDIT.md`), serta merumuskan strategi penanganan melalui upstream patch atau maintained internal fork.

---

## 2. Arahan Desain dari Project Owner (Project Owner Initial Directives)

Sesuai evaluasi Project Owner pada TASK-02-04B, empat pertanyaan terbuka sebelumnya telah memperoleh arahan desain awal. Arahan ini dicatat sebagai **pedoman desain resmi yang menunggu pengesahan formal task (*Project Owner Directives Awaiting Formal Task Acceptance*)**:

| ID Arahan | Domain Arahan | Keputusan Desain Project Owner | Implikasi Rekayasa pada Hime |
|:---:|:---|:---|:---|
| **DIR-01 (ex OQ-01)** | **Pemicu Ekspansi Kanvas** | **Mode Klik (*Click-to-Expand*) ditetapkan sebagai DEFAULT behavior.** Mode Hover menjadi opsi konfigurasi pengguna di menu pengaturan. | Mengeliminasi secara total risiko pemicu ekspansi tidak sengaja (*accidental expansion*) saat pengguna melintaskan kursor menuju tab peramban di tepi atas layar. |
| **DIR-02 (ex OQ-02)** | **Integrasi Perangkat Android** | **Protokol ADB USB kabel ditetapkan sebagai tahap awal integrasi Android (Phase 2).** Arsitektur tetap dirancang agar Wi-Fi Companion dapat ditambahkan kemudian. | Menghindari kompleksitas pengembangan aplikasi mobile native di awal; memanfaatkan Android Debug Bridge standar yang stabil dan cepat untuk transfer file media. |
| **DIR-03 (ex OQ-03)** | **Dukungan GNOME Wayland di Linux** | **GNOME Wayland menggunakan *Graceful Fallback* ke mode jendela utilitas mengapung (*Floating Utility Window Mode*).** | Tidak memaksakan pengguna memasang ekstensi shell GNOME pihak ketiga; Hime tetap dapat berjalan mulus sebagai jendela utilitas terpusat di layar atas. |
| **DIR-04 (ex OQ-04)** | **Mesin Memori Percakapan** | **SQLite FTS5 ditetapkan untuk MVP, dilengkapi dengan lapisan abstraksi (*Storage Abstraction Layer*).** | Sangat hemat memori RAM host (~3.7 GB free); modul local vector embedding dapat ditambahkan secara modular pada Phase 2/3 tanpa merombak skema basis data inti. |

---

## 3. Analisis Kritis Risiko Keamanan 9Router & Hermes Agent

Sesuai temuan audit keamanan Phase 01 (`TASK-01-04-SECURITY-RUNTIME-AUDIT.md`), integrasi 9Router dan Hermes Agent membawa sejumlah kerentanan arsitektural yang **TIDAK BOLEH DIANGGAP SELESAI HANYA KARENA BERJALAN DI LOOPBACK**:

### A. Evaluasi Risiko Keamanan 9Router

```text
+----------------------------------------------------------------------------------------------------+
| 5 KERENTANAN UTAMA BACKEND 9ROUTER (HASIL AUDIT PHASE 01 TASK-01-04 & RE/9ROUTER):                 |
+----------------------------------------------------------------------------------------------------+
| 1. LOCALHOST AUTHENTICATION BYPASS & SNOOPING [VERIFIED STATIC]:                                   |
|    Kode src/dashboardGuard.js:161-165 meloloskan request loopback (isLocalRequest) tanpa validasi  |
|    API key atau CLI token. Proses lokal lain dapat mengakses port 20128 tanpa autentikasi ketat.    |
|                                                                                                    |
| 2. POTENSI 0.0.0.0 BINDING PADA UPSTREAM [VERIFIED STATIC]:                                        |
|    Kode cli/cli.js:110 secara default mengikat ke 0.0.0.0 jika environment variable HOST tidak     |
|    didefinisikan eksplisit, berpotensi mengekspos gateway ke subnet LAN / Wi-Fi publik.            |
|                                                                                                    |
| 3. PLAINTEXT PROVIDER CREDENTIALS PADA DATA.SQLITE [VERIFIED STATIC]:                              |
|    Basis data SQLite (%APPDATA%\9router\db\data.sqlite pada Windows / ~/.9router/db/data.sqlite    |
|    pada Linux; sebelumnya secara kolokial disebut 9router.db) menyimpan API key OpenAI/Anthropic/  |
|    Google dalam bentuk teks polos (plaintext JSON, CWE-312) di tabel providerConnections.          |
|                                                                                                    |
| 4. DEFAULT INITIAL PASSWORD FALLBACK PADA KODE SUMBER [VERIFIED STATIC - SOURCE FALLBACK]:         |
|    Kode src/app/api/auth/login/route.js:58 membaca process.env.INITIAL_PASSWORD || "123456". Nilai|
|    "123456" adalah fallback statis jika env var tidak disetel dan hash belum tersimpan. Belum      |
|    pernah diuji pada live runtime Hime [NOT TESTED RUNTIME] karena belum ada deployment aktif.      |
|                                                                                                    |
| 5. HERMES AUTONOMOUS TOOL EXECUTION PRIVILEGES:                                                    |
|    Hermes Agent memiliki pustaka alat bawaan yang dapat memanggil shell_exec dan fs_write tanpa    |
|    konfirmasi jika tidak dicegat secara terpusat oleh gerbang keamanan Hime Core.                  |
+----------------------------------------------------------------------------------------------------+
```

### B. Matriks Kontrol Keamanan, Kebutuhan Upstream Patch, & Risiko Residual

| Kerentanan Teridentifikasi | Kontrol yang Telah Terverifikasi | Kontrol yang Direncanakan Hime (`[PLANNED]`) | Kebutuhan Upstream Patch / Fork | Hambatan Implementasi (*Blockers*) | Risiko Residual yang Diusulkan (*Residual Risk*) |
|:---|:---|:---|:---|:---|:---|
| **1. Localhost Auth Bypass** | `[VERIFIED STATIC]` pada `src/dashboardGuard.js`. Bind dipaksa ke `127.0.0.1`. | Token otorisasi dinamis (*Session Bearer Token*) yang dirotasi per-sesi oleh Hime Core. | Upstream 9Router perlu mendukung strict header token verification pada rute `/v1/*`. | **BLOCKER JIKA UPSTREAM MENOLAK:** Memerlukan maintained internal patch / proxy interceptor. | Proses lokal lain dengan hak user sama dapat mencoba membaca memori RAM untuk mencuri token sesi. |
| **2. Potensi 0.0.0.0 Binding** | `[VERIFIED STATIC]` pada `cli/cli.js:110`. | Hime Core memaksakan `--host 127.0.0.1` dan memverifikasi socket aktif via Win32 `GetExtendedTcpTable` sebelum streaming data; kill jika `0.0.0.0`. | Tidak butuh patch upstream; diproteksi oleh supervisor Hime Core. | Tidak ada. | Risiko kesalahan konfigurasi manual oleh pengguna jika mengedit file konfigurasi di luar GUI. |
| **3. Plaintext Credentials (`data.sqlite`)** | `[VERIFIED STATIC]` (CWE-312) pada `%APPDATA%\9router\db\data.sqlite` / `~/.9router/db/data.sqlite`. | Hime Core menerapkan Windows ACL / Linux permission `0700` ketat pada direktori data `%APPDATA%\9router\` dan merencanakan wrapper enkripsi DPAPI/libsecret. | **Kebutuhan Upstream:** Fitur enkripsi kolom database kredensial via DPAPI/libsecret. | **KONTROL TERENCANA:** Hime Core membungkus kredensial dalam DPAPI blob sebelum di-insert. | Malware lokal berhak akses user yang sama dapat membaca file jika ACL berhasil ditembus. |
| **4. Default Password "123456"** | `[VERIFIED STATIC — SOURCE FALLBACK]`, `[NOT TESTED RUNTIME]`. | Hime Core Process Supervisor menginjeksi string CSPRNG acak 32-karakter via `INITIAL_PASSWORD` saat spawn proses anak. | Tidak butuh patch upstream; didukung via parameter inisialisasi lingkungan. | Tidak ada. | Kehilangan file konfigurasi mengharuskan reset password lokal. |
| **5. Hermes Autonomous Tool Privileges** | Saluran IPC loopback terisolasi (Windows Named Pipe / Linux Unix Domain Socket). | **Hime Core Centralized Policy Enforcement Gate:** Seluruh tool calls dicegat; pemisahan tegas internal subprocess vs delegated tool execution; aksi berisiko tinggi wajib Action Approval Modal. | Hermes Agent IPC adapter harus mengarahkan seluruh eksekusi tool ke Hime Core via `IpcTransport` (dilarang memiliki bypass path mandiri untuk eksekusi tool). | **BLOCKER ARSITEKTURAL:** Perlu memastikan modul Hermes tool runner tidak memiliki *bypass path*. | Pengguna secara sadar menyetujui perintah berbahaya karena kurang teliti membaca rincian perintah. |

---

## 4. Keputusan yang Telah Disetujui (Confirmed Decisions Summary)

1. **DEC-01:** Strategi platform: *Windows-first, cross-platform by design* (Linux target resmi kedua).
2. **DEC-02:** Hime Core 100% platform-independent tanpa dependensi langsung Windows API (isolasi via PAL).
3. **DEC-03:** Arsitektur 4 komponen mandiri: Hime Desktop, Hime Core, Hermes Agent, dan 9Router.
4. **DEC-04:** Hime Desktop adalah reimplementasi bersih (*clean-room reimplementation*) menggunakan Tauri 2.0. Nusa hanya referensi inspirasi UX.
5. **DEC-05:** 9Router beroperasi sebagai Managed Background Service dengan topologi Dual-Control Plane terikat loopback `127.0.0.1:20128`.
6. **DEC-06:** Pemikiran internal privat (*private chain-of-thought*) Hermes Agent dilarang ditampilkan ke pengguna; digantikan oleh Agent Activity Stream.

---

## 5. Pertanyaan Terbuka & Keputusan yang Ditunda (Remaining Open Decisions)

Daftar pertanyaan yang masih memerlukan klarifikasi rekayasa lanjutan pada fase arsitektur sistem (Phase 05):

| ID Keputusan | Topik Pertanyaan Terbuka | Pilihan Opsi yang Tersedia | Status Rekayasa |
|:---:|:---|:---|:---:|
| **OQ-05** | **Strategi Penanganan Patch 9Router Database:** Apakah mengajukan Pull Request ke upstream `decolua/9router` untuk enkripsi kredensial, atau memelihara *maintained internal fork* khusus Hime? | Pilihan A: Mengajukan PR ke upstream repository.<br>Pilihan B: Memelihara internal patch wrapper di Hime Core (Rekomendasi Tim). | **OPEN DECISION (Phase 05)** |
| **OQ-06** | **Mekanisme Pengecekan Integritas Eksekusi Shell:** Apakah membatasi perintah shell pada daftar whitelist ketat (*Strict Whitelist*), atau menggunakan evaluasi dinamis oleh Hime Core Security Policy Engine? | Pilihan A: Strict command whitelist (Sangat aman, kurang fleksibel).<br>Pilihan B: Dynamic Policy Engine dengan Approval Gate (Fleksibel, aman terawasi). | **OPEN DECISION (Phase 05)** |
| **OQ-07** | **Penyimpanan Berkas Undo Lintas-Volume:** Apakah berkas yang di-undo dari volume lain disimpan sementara di folder `.hime_recycle` lokal di volume tersebut, atau langsung dihapus setelah salin balik? | Pilihan A: Langsung hapus sumber setelah verifikasi hash valid (Sederhana).<br>Pilihan B: Staging di folder recycle lokal volume (Lebih aman, butuh kuota ruang). | **OPEN DECISION (Phase 05)** |

---

## 6. Register Risiko Produk & Strategi Mitigasi Terkalibrasi

| ID Risiko | Deskripsi Risiko Produk | Dampak | Probabilitas | Tingkat Keparahan | Strategi Mitigasi Rekayasa Terkalibrasi |
|:---:|:---|:---:|:---:|:---:|:---|
| **RSK-01** | **Exhaustion Sumber Daya Host pada Laptop Pengguna:** Beban gabungan Hime, Hermes, dan 9Router dapat menghabiskan sisa RAM fisik (~3.7 GB). | Tinggi | Sedang | **HIGH** | Anggaran memori total stack dibatasi $\le 310\text{ MB}$ (Idle) dan $\le 670\text{ MB}$ (Aktif); eksekusi agen dibatasi 1 tugas aktif di MVP; Job Objects supervisor. |
| **RSK-02** | **Fragmentasi Compositing Wayland di Linux:** Protokol `wlr-layer-shell` tidak didukung oleh GNOME Wayland default. | Sedang | Tinggi | **MEDIUM** | Menerapkan arahan DIR-03: fallback anggun otomatis ke *Floating Utility Window Mode* pada sesi GNOME Wayland. |
| **RSK-03** | **Risiko Eksekusi Perintah Destruktif oleh Hermes:** Hermes Agent menjalankan perintah shell terminal yang merusak sistem. | Sangat Tinggi | Rendah | **HIGH** | Sentralisasi penegakan kebijakan di Hime Core (PEP); Action Approval Modal wajib untuk seluruh perintah shell (SEC-06); jalankan sebagai non-admin. |
| **RSK-04** | **Ketidakstabilan & Rate Limit Provider 9Router:** API cloud pihak ketiga mengalami downtime atau kuota habis. | Sedang | Tinggi | **MEDIUM** | Mengaktifkan multi-provider two-tier automatic failover di 9Router; rotasi akun terotomasi; banner status gateway transparan di UI. |
| **RSK-05** | **Kehilangan Data pada Pemindahan Berkas:** Kesalahan operasi pemindahan berkas unduhan merusak dokumen asli. | Sangat Tinggi | Rendah | **HIGH** | Menolak klaim atomik lintas-volume; wajib prosedur multi-tahap (Salin $\rightarrow$ Hash Check $\rightarrow$ Atomic Rename $\rightarrow$ Hapus Asal); jurnal Undo 30 hari. |
| **RSK-06** | **Pencurian Fokus Ketik saat Dynamic Island Terbuka:** Island mencuri kursor ketik (*caret*) dari editor kerja pengguna. | Tinggi | Sedang | **MEDIUM** | Konfigurasi `WS_EX_NOACTIVATE` dan respons pesan `WM_MOUSEACTIVATE` (`MA_NOACTIVATE`) pada window procedure Win32. |
