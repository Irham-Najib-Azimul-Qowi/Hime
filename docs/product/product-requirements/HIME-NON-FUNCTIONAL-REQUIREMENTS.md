# Hime Non-Functional Requirements Specification (Calibrated)

- **Document ID:** HIME-NON-FUNCTIONAL-REQUIREMENTS
- **Task ID:** TASK-02-04 / TASK-02-04B / TASK-02-04D (Product Requirements & Experience Architecture)
- **Phase:** Phase 02 — Nusa Reverse Engineering & Architecture Discovery (Product Specification)
- **Authority:** Lead Systems Architect & Senior Performance Engineer
- **Status:** **CALIBRATED SPECIFICATION — PENDING OWNER REVIEW**
- **Date:** 2026-11-08

---

## 1. Executive Summary & Benchmark Environment

Dokumen ini mendefinisikan kebutuhan non-fungsional (*Non-Functional Requirements / NFR*) terkalibrasi untuk **Hime**.

Sesuai arahan evaluasi Project Owner (TASK-02-04B), dokumen ini membedakan secara jujur dan transparan antara:
1. **Target yang Diusulkan (`[PROPOSED TARGET]`):** Sasaran performa dan batas sumber daya yang belum diuji secara empiris pada build biner Hime yang lengkap.
2. **Ambang Batas Penerimaan (*Acceptance Threshold*):** Nilai toleransi maksimum sebelum sebuah modul dianggap gagal memenuhi kriteria kualitas rekayasa.
3. **Metode Pengukuran (*Measurement Methodology*):** Prosedur pengujian objektif yang dapat direplikasi pada fase verifikasi.

### Lingkungan Pengujian Benchmark Resmi (*Benchmark Environment*)
Seluruh target dan batas toleransi dikalibrasi terhadap profil perangkat keras host pengembang riil yang diverifikasi pada TASK-01-01:
- **Model Prosesor:** Intel Core i5-8250U (4 Cores / 8 Threads, Base Clock 1.60 GHz, Max Turbo 3.40 GHz).
- **Kapasitas RAM Fisik:** ~20.8 GB DDR4 (Ruang memori fisik bebas riil saat beban kerja pengguna aktif: **~3.7 GB free physical RAM**).
- **Pengolah Grafis:** Intel UHD Graphics 620 (Integrated iGPU, dynamic shared memory).
- **Penyimpanan:** Solid State Drive (NTFS, waktu akses acak rendah).
- **Layar Host:** Resolusi 1366 × 768 px @ 60 Hz.
- **Sistem Operasi:** Windows 11 Home Single Language x64 (Build 26100).

---

## 2. Resource Budget per Component & Stack Footprint

Hime beroperasi sebagai sistem multi-proses (*multi-process architecture*) yang terdiri dari 4 komponen independen. Anggaran memori dan utilisasi CPU dibagi secara tegas per-komponen:

### A. Rincian Anggaran Sumber Daya per Komponen

```text
+--------------------------------------------------------------------------------------------------------------------------+
| TABEL ANGGARAN SUMBER DAYA MULTI-PROSES HIME (RESOURCE BUDGET BY COMPONENT)                                              |
+------------------------------------+-------------------+-------------------+-------------------+-------------------------+
| Komponen Sistem                    | Idle Memory (MB)  | Active Memory(MB) | Peak Memory (MB)  | CPU Utilization         |
|                                    | [Target / Thresh] | [Target / Thresh] | [Target / Thresh] | [Target / Threshold]    |
+------------------------------------+-------------------+-------------------+-------------------+-------------------------+
| 1. Hime Desktop (Tauri + Webview2) | <= 80 / 110 MB    | <= 150 / 200 MB   | <= 220 / 280 MB   | Idle: <=0.5% / Act: <=5%|
| 2. Hime Core (Rust Daemon)         | <= 20 / 35 MB     | <= 45 / 70 MB     | <= 70 / 100 MB    | Idle: <=0.1% / Act: <=3%|
| 3. Hermes Agent (Python via uv)    | <= 65 / 90 MB     | <= 180 / 260 MB   | <= 260 / 350 MB   | Idle:  0.0% / Act: <=15%|
| 4. 9Router Service (Node.js)       | <= 50 / 75 MB     | <= 95 / 140 MB    | <= 140 / 190 MB   | Idle:  0.0% / Act: <=4% |
+------------------------------------+-------------------+-------------------+-------------------+-------------------------+
| TOTAL SELURUH STACK MULTI-PROSES   | <= 215 / 310 MB   | <= 470 / 670 MB   | <= 690 / 920 MB   | Idle: <=1.0% / Act:<=25%|
+------------------------------------+-------------------+-------------------+-------------------+-------------------------+
```

*Keterangan:* `Target` adalah proposed target rekayasa; `Thresh` adalah ambang batas toleransi maksimum (*Acceptance Threshold*).

### B. Spesifikasi Teknis Komponen

#### 1. Hime Desktop (Tauri 2 + Webview2 Frontend)
- **Tanggung Jawab:** Antarmuka visual 3-Tier Canvas, rendering Markdown, animasi fluid, event input.
- **Idle Memory:** Proposed Target: $\le 80\text{ MB}$; Acceptance Threshold: $\le 110\text{ MB}$.
- **Active Memory:** Proposed Target: $\le 150\text{ MB}$; Acceptance Threshold: $\le 200\text{ MB}$.
- **Peak Memory (Rendering Diff/Tabel Besar):** Proposed Target: $\le 220\text{ MB}$; Acceptance Threshold: $\le 280\text{ MB}$.
- **CPU Utilization:** Idle $\le 0.5\%$; Aktif $\le 5.0\%$ (spikes saat transisi kanvas).
- **Metode Pengukuran:** Memantau Private Working Set dari proses `hime_desktop.exe` dan anak proses `msedgewebview2.exe` via Windows Performance Counter `Process(*)\Working Set - Private`.

#### 2. Hime Core (Rust Orchestrator & Native PAL)
- **Tanggung Jawab:** Window procedure subclassing, direct composition hook, named pipe IPC bridge, file watcher, SQLite WAL manager, Job Object supervisor.
- **Idle Memory:** Proposed Target: $\le 20\text{ MB}$; Acceptance Threshold: $\le 35\text{ MB}$.
- **Active Memory:** Proposed Target: $\le 45\text{ MB}$; Acceptance Threshold: $\le 70\text{ MB}$.
- **Peak Memory (Pengindeksan FTS5 Batch):** Proposed Target: $\le 70\text{ MB}$; Acceptance Threshold: $\le 100\text{ MB}$.
- **CPU Utilization:** Idle $\le 0.1\%$; Aktif $\le 3.0\%$.
- **Metode Pengukuran:** Profiling binary Rust menggunakan heap profiler `dhat` / Windows Performance Recorder (WPR).

#### 3. Hermes Agent (Python Runtime via `uv`)
- **Tanggung Jawab:** 10-Phase Reasoning Loop, tool execution dispatch, context packing.
- **Idle Memory (Dormant IPC Wait):** Proposed Target: $\le 65\text{ MB}$; Acceptance Threshold: $\le 90\text{ MB}$.
- **Active Memory (Reasoning Loop):** Proposed Target: $\le 180\text{ MB}$; Acceptance Threshold: $\le 260\text{ MB}$.
- **Peak Memory (Context Window Maksimum):** Proposed Target: $\le 260\text{ MB}$; Acceptance Threshold: $\le 350\text{ MB}$.
- **CPU Utilization:** Idle $0.0\%$; Aktif $\le 15.0\%$ (dibatasi 1 logical core max).
- **Metode Pengukuran:** Perekaman memori virtual environment Python `python.exe` saat mengeksekusi sub-proses tool.

#### 4. 9Router Service (Node.js Background Gateway)
- **Tanggung Jawab:** SSE token streaming relay, adapter multi-provider, rotasi akun, failover provider.
- **Idle Memory (Socket Listening):** Proposed Target: $\le 50\text{ MB}$; Acceptance Threshold: $\le 75\text{ MB}$.
- **Active Memory (Streaming Relay):** Proposed Target: $\le 95\text{ MB}$; Acceptance Threshold: $\le 140\text{ MB}$.
- **Peak Memory (Multi-Chunk Buffer):** Proposed Target: $\le 140\text{ MB}$; Acceptance Threshold: $\le 190\text{ MB}$.
- **CPU Utilization:** Idle $0.0\%$; Aktif $\le 4.0\%$.
- **Metode Pengukuran:** Node.js `process.memoryUsage().rss` dan tracking PID via `Get-Process`.

#### 5. Total Multi-Process Footprint Evaluation
- **Klarifikasi Angka 235 MB Idle & 600 MB Active:**
  - Angka **$\le 215\text{ MB}$ (Target) / $\le 310\text{ MB}$ (Ambang Batas)** adalah proposed target untuk **seluruh stack gabungan (Hime Desktop + Core + Hermes + 9Router)** saat kondisi idle.
  - Angka **$\le 470\text{ MB}$ (Target) / $\le 670\text{ MB}$ (Ambang Batas)** adalah proposed target untuk **seluruh stack gabungan saat streaming inferensi dan operasi berkas aktif**.
  - Puncak pemakaian ekstrem (*Peak Memory*) dibatasi pada **$\le 690\text{ MB}$ (Target) / $\le 920\text{ MB}$ (Ambang Batas)**.
- **Evaluasi Kelayakan Host:** Dengan memori fisik bebas rata-rata **~3.7 GB**, pemakaian total stack puncak $\le 920\text{ MB}$ menyisakan ruang aman $> 2.7\text{ GB}$ untuk aplikasi pengguna lain (browser, IDE). Klaim kelayakan ini berstatus **`[PROPOSED TARGET — REQUIRES PHASE 06/07 BENCHMARK VALIDATION]`**.

### C. Kebijakan Remediasi & Eskalasi Kegagalan Benchmark (Benchmark Failure Policy)
Apabila hasil pengujian benchmark empiris pada fase implementasi (Phase 06/07) gagal memenuhi proposed target atau melampaui ambang batas penerimaan (*acceptance threshold*), tim rekayasa wajib menjalankan 5 langkah remediasi disiplin berikut:
1. **Profiling Komponen yang Melampaui Anggaran:** Mengisolasi proses spesifik (Hime Desktop WebView2, Hime Core Rust, Hermes Python, atau 9Router Node.js) menggunakan tool profiling mendalam (Windows Performance Recorder/Analyzer, dhat heap profiler, memory-profiler Python, dan Node.js heap snapshot v8).
2. **Identifikasi Penyebab Dominan:** Menganalisis apakah konsumsi berlebih dipicu oleh overhead arsitektur runtime (misal arsitektur multi-proses WebView2), kebocoran alokasi memori (*memory leak*), buffer streaming chunk yang tidak segera di-flush, atau alokasi cache halaman SQLite FTS5 yang terlalu agresif.
3. **Optimasi Rekayasa Non-Destruktif:** Menerapkan optimasi kode yang tidak mengorbankan keamanan dan stabilitas:
   - Penyetelan siklus Garbage Collection dan pemangkasan working set via `SetProcessWorkingSetSize` saat transisi ke mode idle.
   - Penyesuaian batas page cache basis data SQLite.
   - Pemuatan modul secara malas (*lazy loading*) untuk dependensi yang jarang dipanggil.
   - *Catatan:* Dilarang keras menonaktifkan DACL security, mematikan audit logging, atau mengorbankan verifikasi integritas file demi mengejar angka RAM.
4. **Pengujian Ulang Terstandarisasi:** Mengulang eksekusi suite benchmark otomatis pada beban kerja idle, aktif, dan peak selama minimum 30 menit untuk memverifikasi efektivitas optimasi.
5. **Eskalasi Formal ke Project Owner:** Jika setelah seluruh optimasi teknis diterapkan konsumsi memori tetap melampaui batas toleransi (*acceptance threshold*), tim rekayasa wajib menyusun laporan eskalasi formal disertai bukti telemetri profiling untuk meminta revisi target atau penyesuaian batasan spesifikasi minimum secara resmi dari Project Owner.

---

## 3. Performance & Latency Specifications

### NFR-PERF-01: Global Omnibox Activation Latency
- **Kategori:** Responsivitas Antarmuka Pengguna.
- **Proposed Target:** Waktu jeda dari penekanan tombol `Alt + Space` hingga kursor teks aktif di Omnibox adalah $\le \mathbf{100\text{ ms}}$ (Target Ideal: $\le 60\text{ ms}$).
- **Acceptance Threshold:** $\le \mathbf{140\text{ ms}}$.
- **Rasional Rekayasa:** Latensi $> 140\text{ ms}$ menghasilkan persepsi *input lag* yang mengganggu aliran pengetikan pengguna.
- **Metode Pengukuran:** Mengukur selisih waktu dari pemanggilan hook keyboard Win32 (`WH_KEYBOARD_LL`) hingga pemanggilan event `requestAnimationFrame` pertama yang me-render input field di Webview2 menggunakan `QueryPerformanceCounter` berpresisi sub-milidetik.

### NFR-PERF-02: Streaming Token Latency (Time-to-First-Token)
- **Kategori:** Kecepatan Aliran Inferensi AI.
- **Proposed Target:** $\le \mathbf{800\text{ ms}}$ dari penekanan `Enter` pada prompt hingga karakter teks pertama muncul di antarmuka (di luar latensi jaringan eksternal provider cloud).
- **Acceptance Threshold:** $\le \mathbf{1200\text{ ms}}$ (menggunakan mock gateway loopback).
- **Metode Pengukuran:** Pengujian otomatis dengan mock server lokal 9Router pada port 20128 yang langsung membalas SSE stream tanpa penundaan jaringan eksternal.

### NFR-PERF-03: Animation Fluidity & Frame Rate
- **Kategori:** Kualitas Grafis & Render.
- **Proposed Target:** Animasi morphing kanvas 3-Tier (34 px $\rightarrow$ 212 px $\rightarrow$ 540 px) berjalan pada $\mathbf{60\text{ FPS}}$ stabil dengan frame time $\le 16.6\text{ ms}$ pada iGPU Intel UHD Graphics 620.
- **Acceptance Threshold:** $100\%$ frame di atas $\mathbf{45\text{ FPS}}$ tanpa *dropped frame* berurutan $> 2$ frame berturut-turut.
- **Metode Pengukuran:** Profiling rendering menggunakan Webview2 DevTools Performance Timeline / GPUView selama 50 siklus transisi buka-tutup berturut-turut.

### NFR-PERF-04: Cold & Warm Application Startup Time
- **Kategori:** Kecepatan Boot Sistem.
- **Proposed Target:**
  - *Cold Startup (Pertama kali dijalankan pasca boot OS):* $\le \mathbf{2.5\text{ detik}}$ hingga kapsul Tier 1 tampil di layar atas.
  - *Warm Startup (Dijalankan kembali setelah ditutup):* $\le \mathbf{800\text{ ms}}$.
- **Acceptance Threshold:** Cold Startup $\le \mathbf{4.0\text{ detik}}$; Warm Startup $\le \mathbf{1.5\text{ detik}}$.
- **Metode Pengukuran:** Skrip automated launch yang mengukur waktu proses dari pemanggilan biner `hime.exe` hingga jendela `NotchWindow` menerima pesan `WM_PAINT` pertama.

### NFR-PERF-05: Concurrent Task Limitations (Resource Protection)
- **Kategori:** Manajemen Beban Host.
- **Proposed Target:** Pada rilis MVP, sistem membatasi eksekusi agen pada **1 tugas aktif pada satu waktu (*Single Active Task at a Time*)** dengan antrean terurut (*FIFO Task Queue*).
- **Rasional Rekayasa:** Menjalankan dua agen penalaran multi-step secara simultan pada Intel i5-8250U (4 Cores / 8 Threads) berisiko menyebabkan *CPU throttling* dan lonjakan RAM melebihi batas toleransi.
- **Acceptance Threshold:** Jika tugas kedua dimasukkan saat tugas pertama masih berjalan, sistem wajib menahan tugas kedua di antrean atau meminta konfirmasi pengguna untuk membatalkan tugas pertama.

---

## 4. Security & Policy Enforcement Requirements (Phase 01 Realignment)

Kebutuhan keamanan dikalibrasi ulang untuk menanggulangi temuan risiko Phase 01 (`TASK-01-04-SECURITY-RUNTIME-AUDIT.md`):

### NFR-SEC-01: 9Router Vulnerability Mitigation & Credential Protection
- **Status Risiko Teridentifikasi (Phase 01):**
  1. *Localhost Authentication Bypass:* Proses lokal apa pun dapat mengirim request ke port 20128 jika tidak dilindungi token sesi dinamis.
  2. *Potensi 0.0.0.0 Binding:* Kesalahan konfigurasi dapat mengekspos 9Router ke jaringan LAN.
  3. *Plaintext Provider Credentials:* Basis data SQLite 9Router (`%APPDATA%\9router\db\data.sqlite` pada Windows / `~/.9router/db/data.sqlite` pada Linux; sebelumnya disebut secara kolokial `9router.db`) menyimpan kredensial penyedia model dalam teks polos (CWE-312 `[VERIFIED STATIC]`).
  4. *Default Password:* Upstream 9Router memiliki fallback password bawaan `"123456"` pada kode sumber (`[VERIFIED STATIC — SOURCE FALLBACK]`).
- **Existing Verified Controls (`[VERIFIED STATIC/RUNTIME]`):**
  - Bind interface dipaksa ke `127.0.0.1` melalui environment variable `HOST=127.0.0.1` dan file konfigurasi.
- **Planned Controls for Hime (MVP) (`[PLANNED CONTROL]`):**
  - **Injeksi Token CSPRNG:** Saat inisialisasi pertama kali, Hime Core Process Supervisor menginjeksi token acak 32 karakter dari generator CSPRNG via variabel lingkungan `INITIAL_PASSWORD` saat meluncurkan 9Router, meniadakan ketergantungan pada fallback password `"123456"`.
  - **Hardened ACL pada Folder Data 9Router:** Direktori data (`%APPDATA%\9router\` pada Windows / `~/.9router/` pada Linux) diproteksi menggunakan Windows ACL / POSIX permission `0700` agar hanya dapat dibaca oleh user pemilik.
- **Upstream Changes Required / Maintained Fork Consideration:**
  - *Kebutuhan:* Upstream 9Router saat ini belum mendukung enkripsi database native untuk basis data SQLite `%APPDATA%\9router\db\data.sqlite`.
  - *Keputusan Strategis:* Jika upstream tidak menerima patch enkripsi kolom kredensial SQLite, Hime akan mengelola **Maintained Internal Patch / Wrapper** yang mengenkripsi blob kredensial menggunakan DPAPI sebelum dimasukkan ke `data.sqlite`, atau me-restart 9Router dengan in-memory credentials injection.
- **Proposed Residual Risk:** Jika malware berhasil mengeksekusi kode di bawah akun pengguna aktif yang sama, malware dapat membaca memori proses 9Router atau membaca file jika ACL ditembus. Risiko ini dimitigasi dengan isolasi hak akses akun non-administrator.

### NFR-SEC-02: Centralized Hime Core Policy Enforcement Gate
- **Prinsip Arsitektur:** **Hermes Agent DILARANG memiliki jalur langsung (*direct execution path*) ke sistem operasi untuk mengeksekusi alat atau tindakan sistem tingkat OS tanpa melewati pengawasan Hime Core.**
- **Klarifikasi Sub-Proses:** Sub-proses standar internal interpreter Python diizinkan untuk keperluan operasional runtime internal (seperti inspeksi path virtualenv atau metadata modul). Namun, seluruh eksekusi tindakan alat (*tools*) yang diminta oleh model AI serta manipulasi file/sistem operasi DILARANG menggunakan eksekusi lokal langsung; wajib dialihkan secara penuh melalui pesan IPC ke Hime Core.
- **Enforcement Architecture:**
  - Seluruh pemanggilan alat (*tool calls*) dari Hermes Agent dikirim melalui saluran abstraksi IPC yang terautentikasi (Named Pipe di Windows / Unix Domain Socket di Linux via `IpcTransport` trait) ke Hime Core.
  - Hime Core bertindak sebagai **Policy Enforcement Point (PEP)** dan **Policy Decision Point (PDP)**.
- **Kualifikasi Approval Modal:**
  - Modal persetujuan (*Approval Modal*) adalah **antarmuka otorisasi (*user consent UI*)**, BUKAN batas keamanan (*security boundary*) itu sendiri.
  - **Batas keamanan substantif berada pada Hime Core IPC validator** yang memblokir eksekusi alat jika token izin tidak valid, kedaluwarsa, atau ditolak.
  - Jeda waktu 500 ms adalah **fitur ergonomi anti-slip** (mencegah klik tak sengaja), bukan mekanisme kriptografi atau kontrol keamanan substantif.

---

## 5. File Operations & Undo Reliability

### NFR-DATA-01: Storage Move Integrity & Non-Atomic Cross-Volume Handling
- **Kategori:** Integritas Penyimpanan Berkas.
- **Ketentuan Rekayasa & Batasan Atomisitas:**
  - **Same-Volume Move:** Menggunakan Win32 `MoveFileExW` (`MOVEFILE_REPLACE_EXISTING`) / POSIX `rename()`. Atomisitas hanya terjamin pada filesystem ber-journal (NTFS, ReFS, ext4), namun tidak dapat diklaim universal (bisa gagal karena file lock, permission failure, network shares SMB, reparse points OneDrive, atau volume FAT32/exFAT).
  - **Cross-Volume Move (Misal Drive C: ke Drive X:, atau Laptop ke Android):** Operasi lintas volume **TIDAK DAPAT DIKLAIM ATOMIK SECARA KESELURUHAN**.
    - Prosedur Wajib: (1) Salin berkas sumber ke berkas sementara `.hime_tmp` di volume tujuan, (2) Hitung dan cocokkan checksum SHA-256 berkas sumber vs tujuan, (3) Lakukan rename atomik dari `.hime_tmp` ke nama berkas final di volume tujuan, (4) Panggil `FlushFileBuffers` / `fsync` untuk menjamin durability, (5) Hapus berkas sumber HANYA SETELAH langkah 2, 3, dan 4 terbukti 100% valid.
    - Berkas asli tidak boleh dihapus jika terjadi kegagalan di tengah jalan (*Zero Source Loss*).
- **Acceptance Criteria untuk Kondisi Ekstrem:**
  1. *Failed Copy / Disk Full:* Sistem membatalkan operasi, menghapus file `.hime_tmp`, dan mempertahankan berkas sumber utuh.
  2. *Hash Mismatch:* Sistem melempar alert integritas, membatalkan operasi, dan tidak menghapus berkas sumber.
  3. *File Locked / In Use:* Operasi ditolak secara anggun (*graceful failure*) dengan pesan galat ramah.
  4. *Source Modified during Copy:* Pengecekan hash akhir mendeteksi ketidakcocokan, operasi dibatalkan.
  5. *Undo after Destination Modified:* Jika berkas di tujuan telah diedit pengguna setelah dipindahkan, sistem menolak Undo otomatis dan meminta konfirmasi manual pengguna untuk mencegah penimpaan data baru.

---

## 6. Matriks Ringkasan Non-Functional Requirements Terkalibrasi

| ID NFR | Domain Kualitas | Parameter Metrik Kunci | Proposed Target | Acceptance Threshold | Metode Pengukuran |
|:---:|:---|:---|:---:|:---:|:---|
| **NFR-PERF-01** | Responsivitas | Latensi Aktivasi Omnibox | $\le 100\text{ ms}$ | $\le 140\text{ ms}$ | QPC Sub-ms Timer Hook |
| **NFR-PERF-02** | Kecepatan AI | Time-to-First-Token (Mock) | $\le 800\text{ ms}$ | $\le 1200\text{ ms}$ | SSE Mock Gateway Benchmark |
| **NFR-PERF-03** | Kualitas Visual| Stabilitas Animasi Morphing | $60\text{ FPS}$ ($\le 16.6\text{ ms}$) | $\ge 45\text{ FPS}$ (0 dropped $>2$) | DevTools Performance Timeline |
| **NFR-PERF-04** | Booting | Waktu Cold Startup Aplikasi | $\le 2.5\text{ detik}$ | $\le 4.0\text{ detik}$ | Automated Process Launch Timer |
| **NFR-PERF-05** | Skalabilitas | Batasan Tugas Simultan MVP | $1\text{ Tugas Aktif}$ | $1\text{ Tugas Aktif (FIFO Queue)}$ | Concurrency Semaphore Audit |
| **NFR-RES-01** | Sumber Daya | RAM Idle Seluruh Stack | $\le 215\text{ MB Total}$ | $\le 310\text{ MB Total}$ | Windows Perfmon Working Set |
| **NFR-RES-02** | Sumber Daya | RAM Aktif Seluruh Stack | $\le 470\text{ MB Total}$ | $\le 670\text{ MB Total}$ | Stress-test Active Streaming |
| **NFR-RES-03** | Sumber Daya | RAM Puncak Seluruh Stack | $\le 690\text{ MB Total}$ | $\le 920\text{ MB Total}$ | Batch Peak Workload Monitor |
| **NFR-SEC-01** | Keamanan | Mitigasi Kredensial 9Router (Planned) | $0\text{ Plaintext Secret}$ | Enkripsi DPAPI/libsecret wrapper + ACL $0700$ | Hex Byte Disk & Socket Audit |
| **NFR-SEC-02** | Keamanan | Enforce Kebijakan Hime Core | $100\text{ Gatewayed Tools}$ | $0\text{ Direct Tool Bypass Path}$ | Cross-Platform IPC Channel Test |
| **NFR-DATA-01** | Integritas | Penanganan Lintas-Volume | Salin $\rightarrow$ Hash $\rightarrow$ Del | $0\text{ Source Loss on Failure}$ | Disk Full & Kill Injection Test |
| **NFR-DATA-02** | Integritas | Rollback Jurnal Undo 30 Hari| Validasi Hash Sebelum Undo | Blokir Undo jika Destinasi Berubah | Undo Collision & Integrity Test |
