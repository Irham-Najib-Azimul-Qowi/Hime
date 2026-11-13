# Nusa vs Hime Desktop Feature Gap & Architectural Analysis

- **Document ID:** NUSA-FEATURE-GAP-ANALYSIS
- **Task ID:** TASK-02-01 / TASK-02-02A (Phase 02 — Nusa Reverse Engineering)
- **Target Comparison:** Nusa Dynamic Island v1.2.0.0 vs Planned Hime Desktop
- **Authority:** Chief Software Architect & AI Product Strategist
- **Status:** **CORRECTED — STRATEGIC ARCHITECTURAL AUDIT**
- **Date:** 2026-11-08

---

## 1. Executive Summary

Dokumen ini menyajikan analisis kesenjangan fitur (*feature gap analysis*) dan matriks evaluasi strategis antara **Nusa Dynamic Island v1.2.0.0** (aplikasi referensi) dan **Hime Desktop** (aplikasi asisten AI yang akan dibangun secara *clean-room*).

Meskipun Nusa menyediakan referensi visual dan ergonomi desktop yang luar biasa untuk konsep Dynamic Island pada Windows, fungsi Nusa secara mendasar terbatas pada utilitas widget statis (pemutar musik, pelacak air, to-do list sederhana). Hime Desktop, di sisi lain, merupakan antarmuka komputasi cerdas berbasis agen (*AI-native agent command center*) yang ditenagai oleh **Hermes Agent** dan **9Router**.

Sesuai arahan tata kelola rekayasa TASK-02-02A:
1. Seluruh klaim komparasi diberi kualifikasi bukti yang jelas.
2. Rekomendasi antarmuka pemantauan AI difokuskan secara eksklusif pada **Agent Activity Stream** — yang mengutamakan transparansi aksi, alat, dan hasil yang dapat dipahami pengguna, **bukan** mengekspos private chain-of-thought atau penalaran internal mentah model.

Dokumen ini memetakan seluruh kapabilitas ke dalam **4 Kuadran Strategis Rekayasa**:
1. **Kuadran 1 — Adopsi Langsung (Clean-Room Replication):** Pola visual dan sistem Windows yang dipertahankan.
2. **Kuadran 2 — Tingkatkan & Perluas (Needs Major Re-engineering):** Fitur yang konsep dasarnya bagus namun harus dirombak total untuk AI.
3. **Kuadran 3 — Abaikan / Buang (Omit / Drop):** Fitur Nusa yang tidak relevan, spesifik regional, atau memiliki risiko keamanan.
4. **Kuadran 4 — Diferensiator Inti Hime (New AI Capabilities):** Kapabilitas baru yang tidak ada pada Nusa dan menjadi keunggulan kompetitif Hime.

---

## 2. Strategic Quadrant Analysis

```mermaid
quadrantChart
    title Matriks Strategi Adopsi Fitur: Nusa vs Hime Desktop
    x-axis Nilai Relevansi bagi Asisten AI --> Tinggi
    y-axis Kesiapan Desain Nusa --> Tinggi
    quadrant-1 Kuadran 1: Adopsi Langsung (Clean-Room)
    quadrant-2 Kuadran 2: Tingkatkan & Rombak untuk AI
    quadrant-3 Kuadran 3: Abaikan / Buang
    quadrant-4 Kuadran 4: Diferensiator Baru Hime

    "Geometri Kapsul Floating": [0.85, 0.95]
    "Topmost & Toolwindow Shell": [0.90, 0.90]
    "Animasi Spring DWM": [0.80, 0.85]
    "Fullscreen Auto-hide": [0.85, 0.80]
    "System Tray Management": [0.95, 0.90]
    "Hardware HUD Popups": [0.80, 0.75]

    "File Shelf -> AI File Ingest": [0.90, 0.55]
    "Kanvas 212px -> Elastic Vertical": [0.95, 0.40]
    "Teks Statis -> Markdown Streaming": [0.98, 0.35]
    "Plaintext -> SQLite DPAPI": [0.90, 0.30]
    "Settings UI -> Modern Web Settings": [0.75, 0.50]

    "Jadwal Shalat Kemenag": [0.10, 0.85]
    "LRCLIB Scraping": [0.15, 0.75]
    "Plaintext Clipboard Log": [0.25, 0.60]
    "Inno Setup Installer": [0.20, 0.70]
    "Pixel Mascot Standalone": [0.30, 0.65]

    "Agent Activity Stream": [0.98, 0.15]
    "Omnibox Prompt (Alt+Space)": [0.95, 0.20]
    "Action Approval Modal": [0.96, 0.10]
    "Tool Execution HUD": [0.92, 0.18]
    "9Router Model Gateway Pill": [0.88, 0.12]
```

---

## 3. Quadrant 1 — Adopt Directly (Clean-Room Replication)

Fitur-fitur berikut telah divalidasi memiliki nilai ergonomi dan performa tinggi pada Nusa, dan akan direimplementasikan secara mandiri (*clean-room*) pada Hime Desktop:

1. **Geometri Kapsul Mengapung (*Floating Island Geometry*):**
   - Kapsul idle berbentuk pil simetris (~200 × 34 px) di tengah atas layar monitor dengan radius sudut penuh.
   - Pilihan penempatan `Floating` (mengapung dengan margin atas 8–12 px) sebagai identitas visual utama Hime.
2. **Karakteristik Jendela Shell Windows (`WS_EX_TOPMOST | WS_EX_TOOLWINDOW`):**
   - Jendela selalu berada di lapisan teratas (*always-on-top*) namun tidak memunculkan tombol di Taskbar dan tersembunyi dari daftar `Alt + Tab`.
   - Menggunakan konfigurasi borderless transparan DirectComposition/Tauri untuk rendering dengan latensi rendah.
3. **Sistem Animasi Pegas (*Spring Physics Micro-interactions*):**
   - Morfologi transisi organik dari kapsul kecil ke panel lebar tanpa distorsi sudut membulat.
4. **Deteksi Aplikasi Layar Penuh (*Fullscreen Suppression*):**
   - Memanfaatkan `SHQueryUserNotificationState` untuk menyembunyikan notch secara otomatis saat game 3D layar penuh atau presentasi PowerPoint sedang aktif.
5. **Hardware HUD Popups Pattern:**
   - Animasi pil responsif saat terjadi perubahan status perangkat keras (volume suara, kecerahan layar, status daya baterai).
6. **Integrasi System Tray & Siklus Hidup Latar Belakang:**
   - Ikon baki sistem Windows yang persisten (`Shell_NotifyIconW`), opsi auto-start registry, dan penanganan pemulihan crash `TaskbarCreated`.

---

## 4. Quadrant 2 — Improve & Enhance (Needs Major Re-engineering)

Fitur-fitur berikut ada pada Nusa, namun memiliki keterbatasan fundamental yang wajib dirombak dan diperluas agar memenuhi standar asisten AI modern:

| Komponen Fitur | Implementasi Nusa v1.2.0.0 | Kebutuhan & Desain Ulang Hime Desktop | Justifikasi Teknis |
|:---|:---|:---|:---|
| **Dimensi Kanvas** | Kanvas kaku berdimensi tetap **740 × 212 px** (hanya berorientasi horizontal). | **Kanvas Elastis Dinamis (Adaptive Elastic Canvas):** Lebar 740 px dengan tinggi adaptif (Idle: 34 px $\rightarrow$ Compact: 212 px $\rightarrow$ AI Chat: 480–600 px) dilengkapi kemampuan *scrolling*. | Jawaban LLM dan pemantauan aktivitas memerlukan ruang vertikal yang memadai agar nyaman dibaca. |
| **Metode Input Pengguna** | Hanya klik mouse, drag-and-drop, dan shortcut keyboard standar; **tidak memiliki bar input teks**. | **Omnibox Prompt Bar Terpadu:** Input teks interaktif yang dapat dipanggil instan melalui shortcut global (`Alt + Space`) dengan riwayat perintah dan autocomplete. | Sebagai asisten AI, kemampuan pengguna memberikan instruksi teks kapan saja adalah interaksi paling mendasar. |
| **Presentasi Konten** | Teks sebaris statis (hanya judul lagu atau nama tugas). | **Rich Streaming Markdown Engine:** Menampilkan teks jawaban AI yang mengalir (*token streaming*), blok kode bersintaks (*syntax highlighting*), tabel, dan tombol salin. | Standar mutlak untuk interaksi percakapan dengan AI model modern. |
| **File Staging (Shelf)** | Panel OLE drag-and-drop pasif yang hanya menampung berkas fisik di `%LOCALAPPDATA%\Nusa\Shelf\`. | **AI Context Ingestion Shelf:** Menyeret berkas ke notch langsung mengubah berkas menjadi konteks prompt untuk Hermes Agent (membaca isi berkas, menganalisis data, atau meringkas dokumen). | Mentransformasi fitur utilitas penyimpanan pasif menjadi pemicu aksi kognitif cerdas. |
| **Arsitektur Penyimpanan**| Berkas teks mentah (`.txt`) tak terstruktur di `%LOCALAPPDATA%`, tanpa proteksi integritas atau enkripsi. | **SQLite Lokal Terenkripsi (Windows DPAPI):** Penyimpanan terstruktur untuk riwayat percakapan, preferensi, dan rahasia API dengan jaminan integritas ACID. | Mencegah korupsi data saat mati lampu mendadak dan melindungi privasi pengguna dari proses lain. |
| **Antarmuka Pengaturan**| Jendela kustom kaku 654×556 px tanpa aksesibilitas keyboard atau pembaca layar. | **Web-based Settings Interface (Tauri 2.0):** Antarmuka pengaturan modern dengan skema validasi tipe, aksesibilitas penuh, dan dukungan tema. | Memudahkan ekspansi konfigurasi gateway 9Router, model LLM, dan izin agen. |

---

## 5. Quadrant 3 — Omit / Drop (Do NOT Adopt)

Fitur-fitur Nusa berikut **TIDAK AKAN DIADOPSI** ke dalam arsitektur inti Hime Desktop:

1. **Jadwal Shalat Kemenag RI Tertanam di Core:**
   - *Alasan:* Nusa menyematkan formula hisab spesifik Kementerian Agama Indonesia dan daftar 38 provinsi langsung ke dalam biner aplikasi. Hime adalah ekosistem AI desktop universal. Menyematkan logika regional ke dalam core sistem melanggar prinsip desain modular. Jika fitur ini dibutuhkan, ia dapat disediakan sebagai *optional community plugin*.
2. **Scraping REST API LRCLIB:**
   - *Alasan:* Mengambil lirik dari API publik pihak ketiga tidak memiliki relevansi terhadap kapabilitas asisten AI dan menimbulkan ketergantungan jaringan eksternal yang rapuh.
3. **Penyimpanan Riwayat Clipboard Plaintext Berisiko:**
   - *Alasan:* Menyimpan 50 item clipboard dalam teks polos pada berkas `clipboard.txt` menimbulkan celah keamanan kebocoran kredensial (RSK-03 / SEC-02).
4. **Installer Monolitik Inno Setup:**
   - *Alasan:* Hime Desktop akan menggunakan sistem bundling modern Tauri 2.0 (NSIS/MSI) yang terintegrasi dengan pipeline CI/CD dan penandatanganan kode digital.

---

## 6. Quadrant 4 — Hime Differentiators / New AI Capabilities

Fitur-fitur revolusioner berikut **sama sekali tidak ada pada Nusa**, dan menjadi diferensiator utama serta inti nilai tambah dari ekosistem Hime Desktop:

### A. Hermes Agent Activity Stream
- Dynamic Island berfungsi sebagai pusat transparansi operasional agen dengan menyajikan **Agent Activity Stream**:
  - **Status Pekerjaan Agen:** Menampilkan status terkini secara ringkas (`Membaca Konteks...`, `Menyiapkan Rencana...`, `Menjalankan Perintah...`, `Selesai`).
  - **Ringkasan Rencana Tindakan:** Daftar langkah terstruktur yang akan atau sedang diambil agen.
  - **Tool yang Sedang Digunakan:** Menampilkan nama tool aktif (misal: `shell_exec`, `web_search`, `read_file`).
  - **Status Eksekusi Tool:** Indikator visual progres eksekusi alat secara real-time.
  - **Hasil Tindakan:** Ringkasan hasil eksekusi (misal: `File config.json berhasil diperbarui`, `3 commit ditemukan`).
  - **Alasan Keputusan Ringkas:** Penjelasan singkat yang mudah dipahami pengguna mengapa tindakan tersebut diambil.
  - **Permintaan Persetujuan (*Approval Gate*):** Tombol aksi konfirmasi saat tindakan sensitif memerlukan otorisasi pengguna.
  - *Catatan Tata Kelola:* **Fitur ini BUKAN untuk mengekspos private chain-of-thought atau internal reasoning mentah model AI.** Tujuannya murni transparansi aktivitas dan kendali pengguna.

### B. Real-Time Tool Execution HUD
- Saat Hermes Agent memanggil perkakas sistem (*tool calling*):
  - Kapsul island menampilkan kartu ringkas transparan:
    `[Tool: execute_shell] Menjalankan 'git status' pada repo...`
    `[Tool: read_file] Membaca 'config.toml' (2.4 KB)...`
  - Memberikan visibilitas instan kepada pengguna tanpa membuka terminal latar belakang.

### C. Interactive Action Approval Modal (Security Gate)
- Sesuai dengan prinsip tata kelola keamanan SEC-06:
  - Tindakan kritis atau berisiko tinggi (misal: menghapus berkas, menjalankan perintah shell destruktif, mengakses kredensial) memicu modal persetujuan interaktif pada island:
    `Hermes meminta izin eksekusi: 'rmdir /s /q build'`
    `[ Izinkan Sekali ]   [ Tolak ]   [ Lihat Detail ]`
  - Eksekusi diblokir sepenuhnya hingga persetujuan eksplisit diberikan.

### D. 9Router Model Gateway Status Pill
- Dynamic Island menampilkan indikator kesehatan gateway inferensi lokal:
  - Latensi inferensi (misal: `18 ms`), model yang aktif digunakan, dan indikator koneksi loopback `127.0.0.1:20128`.

### E. Global Hotkey Omnibox & Context Memory Bus
- Shortcut keyboard global instan (`Alt + Space`) yang memusatkan fokus keyboard ke island dari aplikasi Windows mana pun.
- Integrasi *Context Memory Bus* yang secara cerdas mengingat riwayat sesi percakapan sebelumnya dan konteks jendela aplikasi aktif pengguna.

---

## 7. Comparative Feature Matrix Table

| Parameter / Fitur | Nusa Dynamic Island v1.2.0.0 | Bukti Sumber Nusa | Hime Desktop (Clean-Room Reimplementation) | Status Keputusan |
|:---|:---:|:---:|:---:|:---:|
| **Bentuk Visual Dasar** | Kapsul Pil Mengapung | `[INFERRED]` | Kapsul Pil Mengapung | **ADOPT (Q1)** |
| **Always-On-Top & ToolWindow** | Ya (`WS_EX_TOPMOST \| TOOLWINDOW`) | `[INFERRED FROM RE]` | Ya (`WS_EX_TOPMOST \| TOOLWINDOW`) | **ADOPT (Q1)** |
| **DWM Acrylic Frosted Glass** | Ya (DirectComposition) | `[VERIFIED STATIC]` (IAT) | Ya (Tauri / DWM Backdrop Policy) | **ADOPT (Q1)** |
| **Fullscreen Auto-Hide** | Ya (`SHQueryUserNotificationState`) | `[VERIFIED STATIC]` (IAT) | Ya (`SHQueryUserNotificationState`) | **ADOPT (Q1)** |
| **Hardware Volume/Battery HUD**| Ya (Core Audio & Power API) | `[VERIFIED STATIC]` (IAT) | Ya (Core Audio & Power API) | **ADOPT (Q1)** |
| **System Tray & Auto-Start** | Ya (`Shell_NotifyIconW` + Run HKCU)| `[VERIFIED STATIC]` (IAT) | Ya (Tauri Tray + Registry Manager) | **ADOPT (Q1)** |
| **Dimensi Kanvas Jendela** | 740 × 212 px (Statis) | `[INFERRED FROM RE]` | 740 × (34 s/d 600 px) (Elastis Adaptif) | **ENHANCE (Q2)** |
| **Input Bar Teks Pengguna** | Tidak Ada | `[VERIFIED STATIC]` | **Omnibox Interaktif (`Alt + Space`)** | **ENHANCE (Q2)** |
| **Engine Rendering Jawaban** | Label Teks Statis Sebaris | `[VERIFIED STATIC]` | **Rich Streaming Markdown + Syntax Highlight**| **ENHANCE (Q2)** |
| **File Shelf Staging** | Penampungan Berkas Pasif | `[VERIFIED STATIC]` | **AI Context Ingestion Shelf** | **ENHANCE (Q2)** |
| **Penyimpanan State & Riwayat**| Plaintext Files (`.txt`) | `[VERIFIED STATIC]` | **SQLite Lokal Terenkripsi DPAPI** | **ENHANCE (Q2)** |
| **Antarmuka Pengaturan** | Jendela Direct2D Kustom Kaku | `[VERIFIED STATIC]` | **Web UI Settings Modern di Tauri** | **ENHANCE (Q2)** |
| **Jadwal Shalat Kemenag RI** | Ada (Hardcoded di Core) | `[VERIFIED STATIC]` | Dihilangkan dari Core (Plugin Opsional) | **OMIT (Q3)** |
| **LRCLIB Online Scraping** | Ada (HTTP GET lrclib.net) | `[VERIFIED STATIC]` | Dihilangkan dari Core | **OMIT (Q3)** |
| **Plaintext Clipboard Storage**| Ada (5 s/d 50 item teks bebas) | `[VERIFIED STATIC]` | Dihilangkan (Diganti Enkripsi Memori) | **OMIT (Q3)** |
| **Agent Activity Stream** | Tidak Ada | N/A | **Tersedia (Aktivitas, Rencana, & Tool HUD)**| **NEW (Q4)** |
| **Tool Execution HUD** | Tidak Ada | N/A | **Tersedia (Real-Time Action Cards)** | **NEW (Q4)** |
| **Action Approval Modal** | Tidak Ada | N/A | **Tersedia (Security Enforcement Gate)** | **NEW (Q4)** |
| **9Router Gateway Monitor** | Tidak Ada | N/A | **Tersedia (Status Pill & Latency)** | **NEW (Q4)** |
| **Context Memory Bus** | Tidak Ada | N/A | **Tersedia (Persistent Agent Context)** | **NEW (Q4)** |
