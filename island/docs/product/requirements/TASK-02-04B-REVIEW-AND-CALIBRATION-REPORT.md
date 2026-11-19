# TASK-02-04B — Product Architecture Review & Specification Calibration Report

**Proyek:** Hime — Personal AI Desktop Environment
**Fase:** 02 — Nusa Reverse Engineering & Architecture Discovery
**Task ID:** TASK-02-04B (Tindak Lanjut & Kalibrasi TASK-02-04)
**Status Tata Kelola:** `COMPLETE — PENDING OWNER REVIEW`
**Kebijakan Eksekusi:** ZERO COMMIT | ZERO PUSH | NO HIME IMPLEMENTATION
**Tanggal Penyelesaian:** 2026-11-08
**Dokumen Terkait:** `HIME-PRODUCT-REQUIREMENTS-INDEX.md`, Seluruh Deliverable Dokumen Produk di `project-management/product-requirements/`

---

## 1. Executive Summary & Ringkasan Eksekutif

TASK-02-04B merupakan task peninjauan arsitektur dan kalibrasi spesifikasi teknis produk Hime berdasarkan evaluasi kritis Project Owner terhadap hasil deliverable TASK-02-04.

Tujuan utama dari task ini adalah melakukan penyempurnaan, standardisasi batas teknis, eliminasi asumsi yang belum teruji secara empiris, penguatan model keamanan multi-proses terpusat, pengukuhan keandalan operasi sistem berkas, serta sinkronisasi direktif arsitektur Project Owner ke dalam seluruh dokumen spesifikasi produk Hime.

Seluruh revisi dilaksanakan dengan prinsip integritas bukti (*evidence-first engineering*), konsistensi lintas dokumen (*cross-document consistency*), serta tata kelola ketat (*zero commit, zero push, no source code implementation*).

---

## 2. Rincian Temuan Evaluasi & Tindakan Kalibrasi

### A. Anggaran RAM & Kalibrasi Metrik Kinerja (NFR)
- **Kondisi Sebelumnya:** Anggaran RAM mencantumkan angka target 235 MB idle dan 600 MB active seolah-olah merupakan hasil pengukuran empiris yang telah divalidasi, tanpa pemisahan batas konsumsi antar-komponen dalam tumpukan multi-proses.
- **Tindakan Kalibrasi:**
  1. Menegaskan klasifikasi target sebagai **`[PROPOSED TARGET]`**, bukan klaim pengukuran (*measured data*).
  2. Memecah anggaran resource secara rinci ke dalam empat komponen arsitektur independen ditambah total jejak sistem:
     - **Hime Desktop (Tauri Shell & WebView2):** Idle $\le 85\text{ MB}$ (ambang: 120 MB), Active $\le 160\text{ MB}$ (ambang: 220 MB), Peak $\le 220\text{ MB}$ (ambang: 300 MB), CPU $\le 2.0\%$ / $15.0\%$.
     - **Hime Core (Rust Logic & Supervisor):** Idle $\le 30\text{ MB}$ (ambang: 45 MB), Active $\le 60\text{ MB}$ (ambang: 90 MB), Peak $\le 90\text{ MB}$ (ambang: 120 MB), CPU $\le 0.5\%$ / $8.0\%$.
     - **Hermes Agent (Python Runtime):** Idle $\le 65\text{ MB}$ (ambang: 95 MB), Active $\le 175\text{ MB}$ (ambang: 240 MB), Peak $\le 260\text{ MB}$ (ambang: 350 MB), CPU $\le 0.5\%$ / $25.0\%$.
     - **9Router (Node.js Gateway):** Idle $\le 35\text{ MB}$ (ambang: 50 MB), Active $\le 75\text{ MB}$ (ambang: 120 MB), Peak $\le 120\text{ MB}$ (ambang: 150 MB), CPU $\le 0.5\%$ / $10.0\%$.
     - **Total Multi-Process Footprint:** Proposed Idle $\le 215\text{ MB}$ (ambang batas: $310\text{ MB}$), Proposed Active $\le 470\text{ MB}$ (ambang batas: $670\text{ MB}$), Proposed Peak $\le 690\text{ MB}$ (ambang batas: $920\text{ MB}$).
  3. Mengalibrasi metrik latensi Omnibox ($\le 100\text{ ms}$ input to cursor, $\le 150\text{ ms}$ window expansion), frame rate DirectComposition 60 FPS (dengan ambang batas penerimaan $\ge 45\text{ FPS}$ pada Intel UHD 620), startup time (Cold Boot $\le 2.5\text{ s}$ / ambang $4.0\text{ s}$; Warm Boot $\le 800\text{ ms}$ / ambang $1.5\text{ s}$), serta menetapkan batas konkuren MVP menjadi **tepat 1 tugas aktif** (*single active agent task*).

### B. Konsistensi Arsitektur Keamanan & Mitigasi 9Router
- **Kondisi Sebelumnya:** Beberapa klaim mitigasi 9Router belum selaras dengan temuan audit Phase 01 (`TASK-01-04-SECURITY-RUNTIME-AUDIT.md`), mekanisme otorisasi agen belum menegaskan pusat penegakan kebijakan, dan fungsi approval modal belum diposisikan secara presisi.
- **Tindakan Kalibrasi:**
  1. Menegaskan temuan risiko 9Router dari Phase 01:
     - Kredensial penyedia model tersimpan dalam format teks polos (*plaintext*) pada database SQLite `%APPDATA%\9router\db\data.sqlite` (`[VERIFIED STATIC]`).
     - Risiko bypass autentikasi jika interface terikat ke non-loopback atau diakses proses lokal tak terotorisasi (`[VERIFIED STATIC]` pada `src/dashboardGuard.js`).
     - Fallback default password `"123456"` pada kode sumber `src/app/api/auth/login/route.js` (`[VERIFIED STATIC — SOURCE FALLBACK]`, `[NOT TESTED RUNTIME]`).
     - Keterbatasan kuota token pada autentikasi CLI pihak ketiga.
  2. Mengklasifikasikan kontrol keamanan menjadi: *Verified Controls*, *Planned Controls*, *Upstream Changes / Maintained Internal Fork Wrapper*, dan *Residual Risks*.
  3. Menetapkan **Hime Core sebagai satu-satunya Policy Enforcement Point (PEP) dan Policy Decision Point (PDP)**. Hermes Agent dilarang keras memiliki jalur eksekusi alat langsung ke OS; seluruh invocations dicegat oleh Hime Core melalui Named Pipe IPC terotentikasi.
  4. Mendefinisikan **8 Izin Granular Alat**: Read, Write, Modify, Delete, Execute, Network, Ext Transfer, Device.
  5. Memposisikan **Action Approval Modal sebagai antarmuka persetujuan (*user consent UI*)**, bukan batas kriptografis independen.
  6. Mengkualifikasikan delay tombol 500 ms sebagai **kontrol ergonomi anti-slip (*anti-slip affordance*)**, bukan kontrol keamanan substantif.

### C. Keandalan Operasi Sistem Berkas & Jurnal Pembatalan (Undo)
- **Kondisi Sebelumnya:** Siklus pemindahan berkas belum membedakan operasi dalam satu volume (*same-volume*) dengan lintas volume (*cross-volume*), serta belum merinci mekanisme pembatalan setelah destinasi diubah.
- **Tindakan Kalibrasi:**
  1. Membedakan siklus hidup berkas secara formal:
     - *Same-Volume Move:* Menggunakan atomic rename OS (`MoveFileExW` dengan `MOVEFILE_REPLACE_EXISTING` / `rename()`). Operasi ini bersifat atomik di tingkat filesystem.
     - *Cross-Volume Move:* Non-atomik secara keseluruhan. Wajib menggunakan prosedur multi-langkah terverifikasi: Copy ke file temporary `.hime_tmp` pada volume tujuan $\rightarrow$ Verifikasi checksum SHA-256 identik $\rightarrow$ Atomic rename dari temporary ke final name $\rightarrow$ Penghapusan file sumber *hanya setelah* verifikasi integritas sukses 100%. Prinsip: *Zero source file loss before guaranteed destination verification*.
  2. Merinci mekanisme jurnal pembatalan 30 hari:
     - Pre-flight check integritas file destinasi sebelum rollback.
     - Jika file destinasi telah diubah atau dihapus oleh pengguna di luar Hime, otomatisasi penimpaan diblokir dan dialog konfirmasi konflik dimunculkan.
  3. Menambahkan kriteria penerimaan formal untuk 7 kasus kegagalan ekstrim:
     - `AC-FAIL-01`: Kegagalan penyalinan lintas-volume.
     - `AC-FAIL-02`: Ketidakcocokan hash SHA-256.
     - `AC-FAIL-03`: Proses mati mendadak (*process crash/power loss*).
     - `AC-FAIL-04`: Disk destinasi penuh (*out of disk space*).
     - `AC-FAIL-05`: Berkas sumber terkunci oleh aplikasi lain (*sharing violation*).
     - `AC-FAIL-06`: Berkas sumber berubah di tengah proses transfer.
     - `AC-FAIL-07`: Pembatalan (*undo*) setelah berkas destinasi dimodifikasi.

### D. Penyempurnaan Mesin Deteksi Duplikat
- **Kondisi Sebelumnya:** Konsep deteksi duplikat berbasis semantik belum dibedakan secara tegas dari pencocokan kriptografis identik.
- **Tindakan Kalibrasi:**
  1. Menetapkan 5 dimensi deteksi: Ukuran berkas (*Size*), Hash konten kriptografis SHA-256, Kesamaan nama berkas (*Filename similarity*), Analisis metadata & waktu, serta Kesamaan semantik teks/vektor.
  2. Menegaskan prinsip batas teknis: **SHA-256 adalah bukti definitif identitas konten**, sedangkan kesamaan semantik **hanya mendeteksi kemiripan konten atau varian draf**, BUKAN duplikat pasti (*NOT definite duplicates*).
  3. Membagi hasil deteksi ke dalam 5 kategori terpisah: *Exact Duplicate*, *Probable Duplicate*, *Similar Content*, *Name Collision*, dan *Unrelated File*, lengkap dengan tingkat keyakinan (*confidence level*) dan aksi yang diizinkan / dilarang.

### E. Penajaman Ruang Lingkup MVP & Peta Jalan Produk
- **Kondisi Sebelumnya:** Utilitas gaya hidup (pengingat hidrasi dan jadwal shalat) masih tercampur di dalam cakupan inti MVP, sementara batas integrasi penyimpanan Android memerlukan penegasan.
- **Tindakan Kalibrasi:**
  1. Memprioritaskan **12 Pilar Fondasi Inti MVP**: Minimal Idle Pill & Omnibox, 3-Tier Canvas, Multi-Provider Token Relay, Agent Activity Stream & Task Interruption, Human-in-the-Loop Security Gate, Sandboxed Terminal Runner & Diff, Ingesti Drag-and-Drop & SHA-256, Manajemen Unduhan & Safe File Move, Local Task Checklist, Transien HUD, Memori SQLite FTS5, dan Multi-Process Supervisor.
  2. Mereklasifikasi utilitas gaya hidup (`FR-PROD-02` Hidrasi dan `FR-PROD-03` Jadwal Shalat) dari Must/Should Have menjadi **COULD HAVE / Plugin Opsional Fase 2**.
  3. Menegaskan kembali manajemen penyimpanan perangkat Android (`FR-DEVICE-01`, `FR-DEVICE-02`) sebagai bagian resmi roadmap **Phase 2 (Post-MVP)**.

### F. Klarifikasi Batas Lintas Platform (Windows vs Linux)
- **Kondisi Sebelumnya:** Hubungan arsitektur antara platform Windows dan Linux memerlukan batas yang lebih eksplisit untuk mencegah kompromi desain antar-OS.
- **Tindakan Kalibrasi:**
  1. Menegaskan batas hierarkis: Hime Core sepenuhnya bebas dependensi OS platform (*platform-independent*), sedangkan Hime Desktop mengisolasi detail windowing native ke dalam Platform Abstraction Layer (PAL).
  2. Menegaskan tidak adanya pemaksaan paritas palsu (*no forced parity*); fitur windowing disesuaikan dengan kapabilitas native compositor masing-masing OS.
  3. Menerapkan arahan fallback anggun (*graceful fallback*) pada lingkungan Wayland yang membatasi floating overlay bebas menjadi *Floating Utility Window Mode*.

---

## 3. Penerapan Direktif Project Owner (DIR-01 s/d DIR-04)

Seluruh pertanyaan terbuka (*open questions*) yang sebelumnya ditandai OQ-01 hingga OQ-04 telah resmi diselesaikan melalui arahan definitif Project Owner dan diintegrasikan ke seluruh dokumen spesifikasi:

| Kode Direktif | Dokumen Terkait | Arahan Project Owner | Status Implementasi Spesifikasi |
|:---|:---|:---|:---:|
| **DIR-01** (ex OQ-01) | `HIME-PRODUCT-MODULE-ARCHITECTURE.md`<br>`HIME-UX-AND-INTERACTION-ARCHITECTURE.md`<br>`HIME-FUNCTIONAL-REQUIREMENTS.md` | **Click-to-expand ditetapkan sebagai interaksi default** untuk membuka ekspansi panel Dynamic Island. Opsi hover intip (*dwell 250 ms*) dipertahankan sebagai fitur opsional yang dapat diaktifkan melalui menu Settings. | **TERAPKAN LENGKAP** |
| **DIR-02** (ex OQ-02) | `HIME-DEVICE-ECOSYSTEM-AND-MOBILE.md`<br>`HIME-PRODUCT-RISKS-AND-OPEN-DECISIONS.md` | **Integrasi USB via ADB Client Engine ditetapkan sebagai mekanisme awal** untuk integrasi perangkat Android pada Fase 2. Arsitektur protokol tetap disiapkan agar kompatibel dengan Local Wi-Fi Companion Daemon di masa mendatang. | **TERAPKAN LENGKAP** |
| **DIR-03** (ex OQ-03) | `HIME-CROSS-PLATFORM-EXPERIENCE-ARCHITECTURE.md`<br>`HIME-PRODUCT-RISKS-AND-OPEN-DECISIONS.md` | **Menyediakan graceful fallback menjadi floating utility window** jika lingkungan GNOME Wayland atau compositor Linux tertentu membatasi protokol layer-shell atau penempatan overlay mengapung bebas. | **TERAPKAN LENGKAP** |
| **DIR-04** (ex OQ-04) | `HIME-FUNCTIONAL-REQUIREMENTS.md`<br>`HIME-MVP-AND-PRODUCT-ROADMAP.md`<br>`HIME-PRODUCT-MODULE-ARCHITECTURE.md` | **SQLite FTS5 (Full-Text Search) ditetapkan sebagai mekanisme memori utama MVP**, dengan lapisan abstraksi yang siap diperluas ke local vector embeddings pada Fase 2 tanpa memecah skema data dasar. | **TERAPKAN LENGKAP** |

---

## 4. Matriks Rekapitulasi Penyesuaian Kebutuhan & Lingkup

Penyesuaian prioritas MoSCoW dan klasifikasi fase peluncuran kebutuhan fungsional disajikan dalam tabel berikut:

| Kategori Prioritas | Total Awal (TASK-02-04) | Total Terkalibrasi (TASK-02-04B) | Rincian Kebutuhan ID | Status Peluncuran |
|:---|:---:|:---:|:---|:---:|
| **MUST HAVE** | 21 Kebutuhan | **24 Kebutuhan** | `FR-CORE-01`, `FR-CORE-02`, `FR-CORE-03`, `FR-AGENT-01`, `FR-AGENT-02`, `FR-FILE-01`, `FR-FILE-02`, `FR-FILE-03`, `FR-FILE-04`, `FR-DOC-01`, `FR-PROD-01`, `FR-DEV-01`, `FR-DEV-02`, `FR-MEM-01`, `FR-MEM-02`, `FR-NOTIF-01`, `FR-NOTIF-02`, `FR-SEC-01`, `FR-SEC-02`, `FR-SEC-03`, `FR-SYS-01`, `FR-SYS-02`, `FR-SYS-03`, `FR-XPLAT-01` | **MVP (Fase 1)** |
| **SHOULD HAVE** | 5 Kebutuhan | **5 Kebutuhan** | `FR-AGENT-03`, `FR-BROWSER-01`, `FR-DEVICE-01`, `FR-DEVICE-02`, `FR-XPLAT-02` | **Phase 2 (Post-MVP)** |
| **COULD HAVE** | 2 Kebutuhan | **4 Kebutuhan** | `FR-DOC-02`, `FR-BROWSER-02`, `FR-PROD-02`, `FR-PROD-03` | **Phase 2 (Opsional) / Phase 3** |
| **TOTAL KEBUTUHAN AKTIF** | **28 Kebutuhan** | **33 Kebutuhan** | **24 Must + 5 Should + 4 Could** | **Fase 1 s/d Fase 3** |
| **WON'T HAVE** | 3 Inisiatif | **3 Inisiatif** | Autonomous System Takeover, Public P2P Cloud, Local 70B LLM Native Hosting | **Future Vision** |

*Catatan Reklasifikasi Khusus:*
- `FR-PROD-02` (Ambient Hydration) direklasifikasi dari Must Have ke **Could Have (Plugin Opsional Fase 2)**.
- `FR-PROD-03` (Prayer Time Schedule) direklasifikasi dari Should Have ke **Could Have (Plugin Opsional Fase 2)**.
- `FR-FILE-04` (Safe File Move Lifecycle & Undo) dan `FR-SEC-03` (Localhost-Only Transport) ditegaskan statusnya dalam **Must Have (MVP)** dengan kriteria penerimaan terkalibrasi.

---

## 5. Register Keputusan Terbuka (Open Decisions) untuk Fase Berikutnya

Dengan diselesaikannya OQ-01 hingga OQ-04 menjadi direktif resmi (DIR-01 s/d DIR-04), keputusan terbuka yang tersisa ditangguhkan secara terencana untuk Fase 05 (Implementation & Hardening):

| ID Keputusan | Topik Keputusan | Deskripsi & Batas Waktu | Dampak Arsitektur |
|:---:|:---|:---|:---|
| **OQ-05** | Mekanisme Enkripsi Kredensial 9Router | Menentukan apakah perbaikan plaintext credential pada `9router.db` dilakukan melalui kontribusi upstream PR ke repositori `decolua/9router` atau dipelihara sebagai internal fork wrapper pada Hime Core.<br>*Target: Fase 05.* | Keamanan database kredensial dan kemudahan pemeliharaan dependensi eksternal. |
| **OQ-06** | Engine Ekstraksi Teks Multi-Format | Memilih library Rust native (`pdf-extract`, `calamine`) versus subprocess CLI (`pandoc`, `pdftotext`) untuk ingesti dokumen format non-plain text.<br>*Target: Fase 05.* | Ukuran biner kompilasi Hime Core dan kecepatan pengindeksan dokumen lokal. |
| **OQ-07** | Pemilihan Model Vector Embedding Lokal | Mengevaluasi model embedding lokal (misal: `bge-small-en-v1.5` atau `all-MiniLM-L6-v2` via ONNX runtime) versus FTS5 murni untuk performa pada perangkat berdaya rendah.<br>*Target: Fase 05 (Persiapan Phase 2).* | Penggunaan RAM tambahan (~80 MB) dan akurasi temu kembali memori kontekstual. |

---

## 6. Register Risiko Sisa (Residual Risks Register)

Risiko sisa yang diketahui dan telah terdokumentasi secara transparan meliputi:

```text
+----------------------------------------------------------------------------------------------------+
| REGISTER RISIKO SISA (RESIDUAL RISKS REGISTER) — HIME DESKTOP:                                     |
+----------------------------------------------------------------------------------------------------+
| 1. RISK-9ROUTER-PLAINTEXT-DB:                                                                      |
|    Tingkat: TINGGI (Dampak Keamanan)                                                               |
|    Deskripsi: Kredensial API eksternal yang diinput pengguna ke 9Router tersimpan dalam plaintext   |
|    pada database SQLite (%APPDATA%\9router\db\data.sqlite pada Windows).                           |
|    Mitigasi Hime: Direktori data diisolasi dengan hak akses ACL OS 0700 (hanya user).               |
|    Rencana Fase 05: Menyiapkan patch enkripsi DPAPI terintegrasi di Hime Core.                    |
|                                                                                                    |
| 2. RISK-RESOURCE-HEADROOM-HOST:                                                                    |
|    Tingkat: SEDANG (Performa)                                                                      |
|    Deskripsi: Mesin host pengembangan (Core i5-8250U, RAM bebas ~3.7 GB) memiliki batas ketat.    |
|    Mitigasi Hime: Total budget stack multi-proses dibatasi <= 215 MB idle dan <= 470 MB active,   |
|    menyisakan headroom > 2.7 GB untuk aplikasi kerja pengguna lainnya.                             |
|                                                                                                    |
| 3. RISK-LINUX-WAYLAND-FRAGMENTATION:                                                               |
|    Tingkat: SEDANG (Kompatibilitas Tampilan)                                                       |
|    Deskripsi: Compositor GNOME Mutter Wayland menolak protokol wlr-layer-shell pihak ketiga.       |
|    Mitigasi Hime: Fallback anggun (DIR-03) ke jendela utilitas mengapung terpusat (Floating Window).|
+----------------------------------------------------------------------------------------------------+
```

---

## 7. Hasil Audit Konsistensi Lintas Dokumen (Cross-Document Consistency Audit)

Audit substantif menyeluruh dilakukan terhadap seluruh 13 dokumen produk di `project-management/product-requirements/` untuk memverifikasi keterpaduan data:

1. **Konsistensi Anggaran RAM:** Nilai usulan target $\le 215\text{ MB}$ (Idle), $\le 470\text{ MB}$ (Active), dan ambang batas penerimaan $310\text{ MB}$ / $670\text{ MB}$ konsisten di seluruh dokumen `HIME-NON-FUNCTIONAL-REQUIREMENTS.md`, `HIME-PRODUCT-RISKS-AND-OPEN-DECISIONS.md`, dan `HIME-MVP-AND-PRODUCT-ROADMAP.md`.
2. **Konsistensi Kebijakan Keamanan:** Peran Hime Core sebagai satu-satunya Policy Enforcement Point (PEP/PDP), evaluasi 8 izin granular, kualifikasi Action Approval Modal sebagai antarmuka persetujuan, dan jeda 500 ms sebagai kontrol ergonomi anti-slip selaras 100% pada `HIME-AGENT-EXPERIENCE-SPECIFICATION.md`, `HIME-FUNCTIONAL-REQUIREMENTS.md`, `HIME-PRODUCT-MODULE-ARCHITECTURE.md`, dan `HIME-PRODUCT-VISION.md`.
3. **Konsistensi Operasi Berkas & Deteksi Duplikat:** Pembedaan pemindahan same-volume (atomik) vs cross-volume (multi-tahap aman dengan zero source loss) serta 5 dimensi deteksi duplikat (SHA-256 definitif vs kesamaan semantik) konsisten antara `HIME-FILE-AND-STORAGE-EXPERIENCE.md` dan `HIME-FUNCTIONAL-REQUIREMENTS.md`.
4. **Konsistensi Interaksi Default (DIR-01):** Penetapan Click-to-expand sebagai mekanisme default pembukaan Tier 2 panel terintegrasi di `HIME-PRODUCT-VISION.md`, `HIME-PRODUCT-MODULE-ARCHITECTURE.md`, `HIME-USER-CONTEXT-AND-USE-CASES.md`, dan `HIME-UX-AND-INTERACTION-ARCHITECTURE.md`.
5. **Konsistensi Reklasifikasi Kebutuhan:** Pemindahan hidrasi dan jadwal shalat ke Plugin Opsional Fase 2 tercermin sinkron pada tabel ringkasan MoSCoW, inventaris use case, dan modul produk.

---

## 8. Inventaris Dokumen yang Dikalibrasi & Dibuat

Berikut adalah daftar lengkap artefak dokumentasi yang telah dikalibrasi dan dihasilkan dalam TASK-02-04B:

| Nama Berkas Dokumen | Status Tindakan | Ringkasan Kalibrasi Substantif |
|:---|:---:|:---|
| `HIME-NON-FUNCTIONAL-REQUIREMENTS.md` | **Dikalibrasi** | Pemisahan resource budget per 4 komponen, penegasan proposed targets vs ambang batas, re-evaluasi 235/600 MB target, single task limit MVP. |
| `HIME-FILE-AND-STORAGE-EXPERIENCE.md` | **Dikalibrasi** | Siklus same-volume vs cross-volume move, 5 dimensi duplikat (SHA-256 vs semantik), detail undo 30 hari, kriteria kegagalan AC-FAIL-01 s/d 07. |
| `HIME-AGENT-EXPERIENCE-SPECIFICATION.md` | **Dikalibrasi** | Hime Core PEP/PDP terpusat, 8 izin granular, modal persetujuan sebagai UI, jeda 500 ms sebagai kontrol ergonomi anti-slip. |
| `HIME-MVP-AND-PRODUCT-ROADMAP.md` | **Dikalibrasi** | Prioritisasi 12 pilar fondasi inti, reklasifikasi hidrasi/shalat ke Phase 2 plugin, penegasan Android storage management di Phase 2. |
| `HIME-PRODUCT-RISKS-AND-OPEN-DECISIONS.md` | **Dikalibrasi** | Integrasi risiko 9Router dari Phase 01, klasifikasi kontrol keamanan terperinci, penerapan direktif DIR-01 s/d DIR-04, inventaris OQ-05 s/d 07. |
| `HIME-CROSS-PLATFORM-EXPERIENCE-ARCHITECTURE.md` | **Dikalibrasi** | Penegasan batas Core (independen) vs Desktop (PAL), penegasan no forced parity, penerapan fallback anggun Wayland (DIR-03). |
| `HIME-FUNCTIONAL-REQUIREMENTS.md` | **Dikalibrasi** | Pembaruan FR-CORE-01, FR-FILE-02, FR-FILE-04, FR-PROD-02, FR-PROD-03, FR-SEC-01, FR-SEC-02, FR-SEC-03, serta rekonsiliasi total 33 kebutuhan pada tabel MoSCoW Section 3. |
| `HIME-PRODUCT-MODULE-ARCHITECTURE.md` | **Dikalibrasi** | Modul 01 (Click default DIR-01), Modul 11 (Task MVP vs Lifestyle plugin), Modul 14 (Hime Core PEP/PDP, DPAPI wrapper), sinkronisasi tabel Section 4. |
| `HIME-PRODUCT-VISION.md` | **Dikalibrasi** | Penyelarasan ketersediaan instan kapsul (DIR-01 Click default), penguatan prinsip Human-in-the-Loop PEP/PDP, jeda ergonomi 500 ms. |
| `HIME-USER-CONTEXT-AND-USE-CASES.md` | **Dikalibrasi** | UC-01 (Click default trigger DIR-01), UC-07 (Task MVP vs Phase 2 lifestyle plugin), sinkronisasi matriks Section 3. |
| `HIME-UX-AND-INTERACTION-ARCHITECTURE.md` | **Dikalibrasi** | Diagram alir Tier 1 ke Tier 2 (Click default DIR-01), Decision Matrix Section 3, kualifikasi modal otorisasi & 500 ms jeda ergonomi anti-slip. |
| `TASK-02-04-COMPLETION-REPORT.md` | **Dikalibrasi** | Pembaruan indeks paket spesifikasi, rekapitulasi 14 dokumen arsitektur produk dan rekonsiliasi 33 kebutuhan fungsional. |
| `TASK-02-04B-REVIEW-AND-CALIBRATION-REPORT.md` | **Dibuat Baru** | Laporan resmi komprehensif peninjauan arsitektur, kalibrasi metrik, dan penerapan direktif Project Owner (dokumen ini). |

---

## 9. Pernyataan Kepatuhan Tata Kelola & Stop Condition

1. **Kebijakan Eksekusi Terpenuhi:**
   - [x] Zero Commit: Tidak ada perintah `git commit` yang dieksekusi.
   - [x] Zero Push: Tidak ada perintah `git push` yang dieksekusi.
   - [x] No Implementation Code: Tidak ada pembuatan kode biner, backend logic Rust, atau frontend UI.
   - [x] Evidence & Precision Integrity: Seluruh klaim target performa dikualifikasikan sebagai `[PROPOSED TARGET]`, risiko keamanan diselaraskan dengan audit Phase 01.
2. **Kondisi Berhenti (*Stop Condition*):**
   - Task TASK-02-04B dinyatakan **SELESAI DENGAN KUALIFIKASI PENUH** (*Complete*).
   - Seluruh deliverables siap ditinjau oleh Project Owner.
   - Sistem **TIDAK AKAN** melangkah ke TASK-02-05 secara mandiri sebelum menerima keputusan penerimaan resmi dari Project Owner.

---
*Laporan ini disusun secara otomatis dan diverifikasi sesuai pedoman tata kelola proyek Hime.*
