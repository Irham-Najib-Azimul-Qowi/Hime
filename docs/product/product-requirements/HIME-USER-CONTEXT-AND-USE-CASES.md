# Hime User Context & Use Cases Specification

- **Document ID:** HIME-USER-CONTEXT-AND-USE-CASES
- **Task ID:** TASK-02-04 / TASK-02-04D (Product Requirements & Experience Architecture)
- **Phase:** Phase 02 — Nusa Reverse Engineering & Architecture Discovery (Product Specification)
- **Authority:** Principal Product Manager & Senior Human-Computer Interaction (HCI) Specialist
- **Status:** **APPROVED SPECIFICATION**
- **Date:** 2026-11-08

---

## 1. Executive Summary

Dokumen ini memetakan konteks kehidupan digital dan skenario penggunaan nyata (*use cases*) dari pengguna sasaran **Hime** di lingkungan kerja desktop Windows dan Linux.

Penyusunan use case ini mencakup 11 domain aktivitas digital:
1. Produktivitas Pribadi (*Personal Productivity*)
2. Manajemen Berkas & Penyimpanan (*File & Storage Management*)
3. Pembuatan & Penyuntingan Dokumen (*Document Creation & Editing*)
4. Riset & Kerja Akademik (*Research & Academic Work*)
5. Rekayasa Perangkat Lunak (*Software Engineering*)
6. Otomatisasi Peramban (*Browser Automation*)
7. Manajemen Perangkat (*Device Management & Cross-Device*)
8. Manajemen Tugas & Jadwal (*Task & Schedule Management*)
9. Komunikasi & Notifikasi (*Communication & Notification*)
10. Alur Kerja Berbantuan AI (*AI-Assisted Workflows*)
11. Organisasi Data & Manajemen Pengetahuan Pribadi (*PKM*)

Setiap skenario dirinci dalam struktur operasional baku: **User Goal**, **Trigger**, **Context**, **Hime Behavior**, **Required Capability**, **Expected Outcome**, **Permission Requirement**, **Failure Handling**, dan **MVP Classification**.

---

## 2. Comprehensive Use Case Catalog

### UC-01: Quick Ambient Command & Instant Prompting (Personal Productivity)
- **User Goal:** Mengirimkan pertanyaan ringkas atau instruksi cepat kepada AI tanpa membuka jendela aplikasi penuh atau beralih dari dokumen yang sedang dikerjakan.
- **Trigger:** Menekan tombol pintasan global `Alt + Space` atau klik kiri langsung pada kapsul Dynamic Island (sesuai DIR-01 Click-to-expand sebagai default).
- **Context:** Pengguna sedang membaca file PDF atau mengedit dokumen di editor teks latar belakang.
- **Hime Behavior:**
  1. Hime membuka **Omnibox Prompt Bar** di tengah atas layar monitor dengan transisi spring mulus.
  2. Fokus kursor otomatis berada pada kolom input teks.
  3. Pengguna mengetik: *"Konversi 150 USD ke IDR dengan kurs terkini"*.
  4. Hime merutekan pertanyaan ke 9Router loopback, menerima streaming token jawaban, dan menampilkannya pada panel ringkas Tier 2 (~212 px).
  5. Pengguna menekan `Esc`; Hime seketika menutup kembali ke status idle tanpa mengganggu jendela editor.
- **Required Capability:** Global hotkey registration, Omnibox UI, streaming token relay, keyboard dismiss.
- **Expected Outcome:** Pengguna memperoleh jawaban dalam < 2 detik tanpa kehilangan konteks kerja utama.
- **Permission Requirement:** Safe (Otomatis / Tidak memerlukan izin khusus).
- **Failure Handling:** Jika koneksi 9Router terputus, Hime menampilkan banner pill: *"Koneksi gateway terputus, klik untuk memeriksa status 9Router"*.
- **MVP Classification:** **MVP (Must Have)**.

---

### UC-02: Intelligent Download Conflict Resolution & Auto-Sorting (File & Storage)
- **User Goal:** Mengunduh berkas dari web tanpa menyebabkan folder `Downloads` menumpuk ratusan berkas tidak terorganisir dan mencegah penimpaan file duplikat tanpa sadar.
- **Trigger:** Berkas baru terdeteksi di folder `Downloads` (via File System Watcher) atau berkas diseret ke kapsul Hime.
- **Context:** Pengguna baru saja mengunduh berkas bernama `laporan_keuangan_2026.pdf`, di mana berkas dengan nama yang sama sudah ada di folder arsip dokumen.
- **Hime Behavior:**
  1. File System Watcher mendeteksi file baru dan menghitung checksum SHA-256 serta menganalisis metadata.
  2. Hime mendeteksi keberadaan file dengan nama serupa:
     - Jika hash identik: Hime menandai sebagai duplikat pasti (*exact content match*).
     - Jika hash berbeda: Hime menandai sebagai revisi versi baru (*content difference detected*).
  3. Kapsul Hime berekspansi ke Tier 2 menampilkan kartu notifikasi interaktif:
     *"Ditemukan berkas serupa di Dokumen/Keuangan/ (Ukuran berbeda). Tindakan yang disarankan: Simpan sebagai 'laporan_keuangan_2026_v2.pdf' atau Timpa?"*
  4. Pengguna memilih `[ Buat Versi Baru (v2) ]`.
  5. Hime memindahkan berkas secara aman ke folder tujuan yang sesuai secara otomatis.
- **Required Capability:** File System Watcher, cryptographic hash calculation, conflict resolution engine, safe file move with undo buffer.
- **Expected Outcome:** Berkas tersimpan rapi pada folder semantik yang relevan, konflik terselesaikan, dan riwayat pemindahan dicatat pada database SQLite.
- **Permission Requirement:** Low-Medium (Konfirmasi pengguna satu kali melalui kartu pop-up).
- **Failure Handling:** Jika folder tujuan terkunci atau izin baca/tulis ditolak, Hime mempertahankan berkas di folder Downloads asli dan menampilkan pesan galat yang informatif.
- **MVP Classification:** **MVP (Must Have)**.

---

### UC-03: Multi-Document AI Synthesis & Literature Extraction (Research & Academic Work)
- **User Goal:** Mengekstrak metodologi, temuan utama, dan perbandingan dari tiga berkas paper penelitian PDF yang berbeda.
- **Trigger:** Pengguna memilih 3 file PDF di File Explorer dan menyeretnya (*drag-and-drop*) langsung ke Dynamic Island Hime.
- **Context:** Pengguna sedang menyusun bab tinjauan pustaka tesis atau laporan analisis industri.
- **Hime Behavior:**
  1. Kapsul mendeteksi seretan berkas, menampilkan banner visual *"Drop untuk ekstraksi konteks dokumen"*, dan menerima dokumen via OLE `IDropTarget`.
  2. Hime Core mengekstrak teks dan metadata dokumen menggunakan parser lokal.
  3. Hime berekspansi ke **Tier 3 (Expanded Chat Canvas, ~540 px)**.
  4. Omnibox menampilkan badge dokumen terlampir: `[3 Berkas Tersemat: PaperA.pdf, PaperB.pdf, PaperC.pdf]`.
  5. Pengguna mengetik: *"Bandingkan metodologi dan kelemahan dari ketiga paper ini dalam bentuk tabel Markdown"*.
  6. Hermes Agent memproses konteks melalui 9Router, mengalirkan tabel sintesis, dan menyediakan tombol `[ Salin Markdown ]` serta `[ Simpan sebagai Catatan ]`.
- **Required Capability:** OLE drag-and-drop ingestion, local document parser, multi-document context manager, Markdown renderer dengan copy-to-clipboard.
- **Expected Outcome:** Pengguna memperoleh tabel komparasi mendalam dalam hitungan detik tanpa membuka berkas PDF satu per satu.
- **Permission Requirement:** Safe (Hanya membaca berkas yang secara sadar di-drop oleh pengguna).
- **Failure Handling:** Jika file PDF terenkripsi password, Hime menampilkan input field dialog untuk memasukkan password dokumen lokal.
- **MVP Classification:** **MVP (Must Have)**.

---

### UC-04: Automated Git & Test Runner Execution with Approval (Software Engineering)
- **User Goal:** Meminta agen AI menjalankan audit pengujian unit, menganalisis kegagalan pengujian, dan membuat branch git baru untuk perbaikan.
- **Trigger:** Perintah pengguna melalui Omnibox: *"Jalankan cargo test, jika ada kegagalan perbaiki kode di modul parser"*.
- **Context:** Pengguna sedang berada di direktori repositori proyek kode.
- **Hime Behavior:**
  1. Hime membuka **Agent Activity Stream** pada panel Tier 2/3.
  2. Hermes Agent merencanakan tindakan (*Action Plan*):
     - Langkah 1: `shell_exec("cargo test")`
     - Langkah 2: Evaluasi keluaran stderr.
  3. Hime mendeteksi bahwa perintah mengeksekusi shell sistem operasi (kategori tindakan berisiko sedang SEC-06).
  4. Hime menampilkan **Interactive Action Approval Modal**:
     *"Hermes Agent meminta izin mengeksekusi perintah shell: `cargo test` di direktori `X:/folder_ai/Hime`"*.
     Pilihan: `[ Izinkan Sekali ]`, `[ Izinkan Selama Sesi Ini ]`, `[ Tolak ]`.
  5. Pengguna menekan `[ Izinkan Sekali ]`.
  6. Hermes Agent mengeksekusi perintah, menangkap log output secara real-time di Activity Stream, dan melaporkan ringkasan hasil uji (misal: 14 lulus, 0 gagal).
- **Required Capability:** Process supervisor, sandboxed shell execution, Agent Activity Stream, Human-in-the-Loop Approval Modal, timeout guard.
- **Expected Outcome:** Pengujian dijalankan dengan transparansi operasional penuh tanpa mengeksekusi perintah arbitrer secara liar.
- **Permission Requirement:** High (Wajib persetujuan eksplisit pengguna sebelum eksekusi).
- **Failure Handling:** Jika perintah melebihi batas waktu timeout (misal 60 detik tanpa respon), Hime secara otomatis mengirimkan sinyal SIGKILL/TerminateProcess dan mengembalikan status ke pengguna.
- **MVP Classification:** **MVP (Must Have)**.

---

### UC-05: Web Research & Information Gathering via Browser Automation (Browser Automation)
- **User Goal:** Mengumpulkan harga, spesifikasi teknis, dan ulasan produk dari tiga situs web e-commerce tanpa membuka tab peramban secara manual.
- **Trigger:** Perintah teks di Omnibox: *"Cari harga termurah untuk ThinkPad T14 Gen 4 di Tokopedia dan Shopee, lalu buat perbandingannya"*.
- **Context:** Pengguna sedang merencanakan pengadaan perangkat keras kantor.
- **Hime Behavior:**
  1. Hermes Agent memanggil tool browser automation (Puppeteer / Playwright headless bridge).
  2. Agent Activity Stream menampilkan tahapan:
     - `Navigating to target URLs...`
     - `Extracting pricing DOM selectors...`
     - `Parsing structured comparison...`
  3. Hime menyaring anti-bot / CAPTCHA (sesuai aturan rekayasa: jika terhadang CAPTCHA, agen berhenti dan mengalihkan jendela browser terlihat agar diselesaikan pengguna).
  4. Menghasilkan kartu ringkasan perbandingan harga dengan tautan sumber bare URL.
- **Required Capability:** Browser automation driver, DOM parser, CAPTCHA detection & fallback modal, network fetch manager.
- **Expected Outcome:** Ringkasan harga terstruktur siap saji di canvas Hime.
- **Permission Requirement:** Medium (Memerlukan izin untuk membuka koneksi internet outbound).
- **Failure Handling:** Jika situs memblokir bot, Hime membuka jendela Chromium interaktif dan meminta pengguna: *"Silakan selesaikan verifikasi Cloudflare pada jendela peramban untuk melanjutkan"*.
- **MVP Classification:** **Phase 2 (Post-MVP)**.

---

### UC-06: Cross-Device Storage Cleanup & Android Photo Staging (Device Management)
- **User Goal:** Memeriksa kapasitas memori smartphone Android dari laptop, menemukan file foto/video duplikat atau tangkapan layar lama, dan memindahkannya ke hard disk laptop untuk mengosongkan ruang HP.
- **Trigger:** Perangkat Android terhubung via USB kabel (MTP/ADB) atau Wi-Fi P2P local loopback, atau perintah manual dari tab Device Manager.
- **Context:** Memori internal Android pengguna hampir penuh (sisa < 2 GB).
- **Hime Behavior:**
  1. Hime Device Manager mendeteksi perangkat Android via ADB / Local Companion Daemon.
  2. Menganalisis direktori `DCIM/Screenshots` dan `WhatsApp/Media`.
  3. Mengidentifikasi 450 tangkapan layar berumur > 3 bulan yang memakan ruang 1.8 GB.
  4. Hime memunculkan kartu rekomendasi di Dynamic Island:
     *"Memori HP kritis (Sisa 1.8 GB). Ditemukan 450 tangkapan layar lama. Pindahkan ke Arsip Laptop `D:/Arsip_HP/`?"*
  5. Pengguna menekan `[ Pindahkan & Kosongkan Ruang ]`.
  6. Hime menyalin berkas secara aman, memvalidasi hash integritas, lalu menghapus berkas sumber pada HP setelah transfer terbukti 100% valid.
- **Required Capability:** Android Bridge Protocol (ADB / Companion Daemon), cross-device file transfer, integrity validation, batch safe-delete.
- **Expected Outcome:** Ruang memori HP bertambah 1.8 GB, seluruh foto tersimpan aman di laptop dengan struktur folder tahun/bulan.
- **Permission Requirement:** High (Memerlukan konfirmasi izin transfer & penghapusan berkas pada perangkat eksternal).
- **Failure Handling:** Jika koneksi USB/Wi-Fi terputus di tengah proses transfer, Hime menghentikan operasi secara aman tanpa menghapus berkas yang belum selesai ditransfer (prinsip zero data loss).
- **MVP Classification:** **Phase 2 (Post-MVP)**.

---

### UC-07: Context-Aware Daily Agenda & Task Check-In (Task & Schedule Management)
- **User Goal:** Memulai hari kerja dengan ringkasan terpadu mengenai komitmen penting dan tenggat waktu tugas harian (dengan opsi utilitas gaya hidup seperti jadwal shalat dan hidrasi sebagai plugin opsional).
- **Trigger:** Komputer pertama kali dinyalakan di pagi hari (logon startup) atau klik pada panel Today di Dynamic Island.
- **Context:** Jam 08:30 WIB, hari kerja biasa.
- **Hime Behavior:**
  1. Dynamic Island bertransisi halus menampilkan kartu **Today Morning Briefing**.
  2. Menampilkan 3 item prioritas utama hari ini (dari to-do list lokal `FR-PROD-01`).
  3. Jika plugin opsional gaya hidup diaktifkan (`FR-PROD-02`, `FR-PROD-03`), kartu menampilkan hitung mundur shalat terdekat dan indikator pengingat minum air.
  4. Pengguna dapat langsung mencentang tugas yang selesai dari panel kompak tanpa membuka aplikasi planner terpisah.
- **Required Capability:** Local task engine (SQLite `FR-PROD-01`), audio chime notification; opsional: prayer time calculator & hydration reminder plugin interfaces.
- **Expected Outcome:** Pengguna memiliki kesadaran mental yang jernih terhadap prioritas hari tanpa overwhelmed oleh notifikasi kalender yang berisik.
- **Permission Requirement:** Safe (Akses data lokal).
- **Failure Handling:** Jika plugin eksternal gagal diinisialisasi, Hime tetap menampilkan to-do list lokal secara utuh tanpa hambatan.
- **MVP Classification:** **MVP (Local Tasks Checklist)** / **Phase 2 (Optional Lifestyle Plugins: Hydration & Prayer)**.

---

### UC-08: Intelligent Document Drafting & Code Snippet Refactoring (Document Creation)
- **User Goal:** Menulis draf surat resmi, proposal teknis, atau merefaktor modul fungsi Python yang berantakan langsung dari desktop.
- **Trigger:** Mengetik di Omnibox: *"Buat draf memo pengajuan upgrade server host"*.
- **Context:** Pengguna sedang menyiapkan proposal infrastruktur rekayasa.
- **Hime Behavior:**
  1. Hime membuka **Tier 3 Expanded Workspace Canvas**.
  2. Hermes Agent menghasilkan draf dokumen terstruktur dengan format standar institusi.
  3. Hime menyediakan kontrol editor mini di dalam panel: tombol format Markdown, tombol `[ Ekspor ke .docx / .pdf ]`, dan tombol `[ Salin ]`.
  4. Pengguna dapat memberikan instruksi revisi lanjutan secara percakapan (*conversational refinement*): *"Ubah bahasanya menjadi lebih formal dan tambahkan estimasi biaya"*.
- **Required Capability:** Streaming markdown generator, multi-turn conversational memory, file export utility.
- **Expected Outcome:** Draf selesai dalam hitungan menit dan siap dikirim.
- **Permission Requirement:** Safe (Generasi teks lokal/cloud via 9Router).
- **Failure Handling:** Jika token rate limit tercapai pada salah satu provider 9Router, 9Router secara transparan melakukan failover ke provider cadangan (misal dari Anthropic ke OpenAI atau Gemini) tanpa interupsi obrolan.
- **MVP Classification:** **MVP (Must Have)**.

---

### UC-09: Unobtrusive System Health & Hardware HUD Alerts (Communication & Notification)
- **User Goal:** Memantau perubahan status volume suara, tingkat kecerahan layar, status koneksi gateway, atau peringatan baterai laptop tanpa terganggu oleh dialog popup OSD Windows yang kaku.
- **Trigger:** Tombol fisik volume keyboard ditekan, atau baterai laptop mencapai 15%.
- **Context:** Pengguna sedang menonton video atau mempresentasikan slide layar penuh.
- **Hime Behavior:**
  1. Hime menangkap event notifikasi perangkat keras melalui adapter CoreAudio/Power Management.
  2. Kapsul Dynamic Island bertransisi mulus menjadi bar horizontal HUD elegan (~220 px) menampilkan ikon dan slider volume atau status baterai.
  3. Menampilkan status selama ~2.5 detik lalu menyusut kembali menjadi kapsul idle tanpa mencuri fokus jendela presentasi.
- **Required Capability:** OS Hardware Event Listeners, non-activating window overlay, DWM direct composition.
- **Expected Outcome:** Notifikasi tersampaikan dengan estetika modern yang tidak mengaburkan materi layar utama.
- **Permission Requirement:** Safe (Event sistem lokal).
- **Failure Handling:** Jika API hardware tidak merespons, Hime mengabaikan event tanpa melempar exception fatal.
- **MVP Classification:** **MVP (Must Have)**.

---

### UC-10: Multi-Step Agentic Problem Solving Loop (AI-Assisted Workflows)
- **User Goal:** Meminta agen menyelesaikan tugas multi-tahap kompleks yang membutuhkan perencanaan, pemanggilan beberapa tools, pengujian, dan verifikasi mandiri.
- **Trigger:** Perintah Omnibox: *"Audit seluruh file .md di folder docs/, periksa apakah ada broken link, dan buat laporan rekapitulasinya"*.
- **Context:** Pemeliharaan dokumentasi proyek repositori besar.
- **Hime Behavior:**
  1. Hermes Agent menginisiasi **10-Phase Reasoning Loop**:
     - *Phase 1:* Pemahaman instruksi & formulasi hipotesis.
     - *Phase 2:* Rencana tindakan berurutan.
     - *Phase 3:* Pemanggilan tool `fs_list_files("docs/", "*.md")`.
     - *Phase 4:* Pemanggilan tool `fs_read_file()` dan regex link extractor.
     - *Phase 5:* Validasi tautan lokal pada filesystem.
     - *Phase 6:* Kompilasi laporan broken link.
  2. Selama proses berjalan, kapsul Dynamic Island menampilkan animasi detak halus (*subtle pulse*) dengan badge progres: `[Auditing 14/32 files...]`.
  3. Setelah selesai, Hime menyajikan laporan ringkasan pada canvas Tier 3 dengan pilihan tombol `[ Simpan Laporan ]`.
- **Required Capability:** Hermes Agent Reasoning Loop, background task supervisor, multi-tool orchestration, progress reporting stream.
- **Expected Outcome:** Tugas verifikasi selesai secara otomatis, akurat, dan dapat diverifikasi ulang oleh pengguna.
- **Permission Requirement:** Medium (Izin pembacaan berkas lokal dalam cakupan folder kerja).
- **Failure Handling:** Pengguna dapat menekan tombol `[ Berhenti / Cancel ]` di Dynamic Island kapan saja untuk menghentikan loop agen seketika.
- **MVP Classification:** **MVP (Must Have)**.

---

### UC-11: Personal Knowledge Graph & Context Memory Recall (PKM)
- **User Goal:** Mengingat kembali keputusan teknis, catatan percakapan, atau lokasi file yang pernah dibahas minggu lalu tanpa mencari manual di ribuan file.
- **Trigger:** Pertanyaan di Omnibox: *"Di mana saya menyimpan catatan konfigurasi proxy 9Router dan apa kesimpulan diskusi kemarin?"*.
- **Context:** Pengguna lupa path absolut file catatan konfigurasi.
- **Hime Behavior:**
  1. Hime Core mencari riwayat percakapan dan indeks dokumen menggunakan mesin pencari lokal SQLite FTS5 (Full-Text Search) dan embedding semantik.
  2. Menemukan entri yang relevan di `project-management/engineering/ENGINEERING-FOUNDATION.md`.
  3. Hime menjawab:
     *"Catatan konfigurasi proxy 9Router berada di `project-management/engineering/ENGINEERING-FOUNDATION.md#L45`. Diskusi kemarin menetapkan topologi Dual-Control Plane pada port 20128 dengan bind loopback 127.0.0.1."*
  4. Menyediakan tombol pintas `[ Buka File ]` untuk langsung membuka file di editor default.
- **Required Capability:** SQLite FTS5 indexer, local memory recall manager, file launcher adapter.
- **Expected Outcome:** Informasi ditemukan kembali dalam waktu < 1 detik secara lokal dan privat.
- **Permission Requirement:** Safe (Pencarian memori basis data lokal).
- **Failure Handling:** Jika tidak ditemukan kecocokan persis, Hime memberikan saran pencarian kata kunci terdekat tanpa mengarang fakta (*no hallucination*).
- **MVP Classification:** **Phase 2 (Post-MVP)**.

---

## 3. Matriks Klasifikasi Use Case & Fase Peluncuran

Tabel berikut merangkum pemetaan seluruh use case terhadap fase peluncuran produk:

| ID Use Case | Nama Use Case | Domain Aktivitas | Kompleksitas Teknis | Risiko Keamanan | Klasifikasi Fase |
|:---:|:---|:---|:---:|:---:|:---:|
| **UC-01** | Quick Ambient Command & Instant Prompting | Personal Productivity | Sedang | Rendah | **MVP (Must Have)** |
| **UC-02** | Download Conflict Resolution & Auto-Sorting | File & Storage | Sedang-Tinggi | Menengah | **MVP (Must Have)** |
| **UC-03** | Multi-Document AI Synthesis & Extraction | Research & Academic | Sedang | Rendah | **MVP (Must Have)** |
| **UC-04** | Automated Git & Test Runner with Approval | Software Engineering | Tinggi | Tinggi | **MVP (Must Have)** |
| **UC-05** | Web Research via Browser Automation | Browser Automation | Tinggi | Menengah | **Phase 2 (Post-MVP)** |
| **UC-06** | Cross-Device Storage Cleanup & Android Sync| Device Management | Sangat Tinggi | Tinggi | **Phase 2 (Post-MVP)** |
| **UC-07** | Context-Aware Daily Agenda & Task Check-In | Task & Schedule | Rendah-Sedang | Rendah | **MVP (Tasks) / Phase 2 (Lifestyle)** |
| **UC-08** | Intelligent Document Drafting & Refactoring | Document Creation | Sedang | Rendah | **MVP (Must Have)** |
| **UC-09** | Unobtrusive System Health & Hardware HUD | Communication & Notif | Sedang | Rendah | **MVP (Must Have)** |
| **UC-10** | Multi-Step Agentic Problem Solving Loop | AI Workflows | Sangat Tinggi | Tinggi | **MVP (Must Have)** |
| **UC-11** | Knowledge Graph & Context Memory Recall | PKM & Memory | Tinggi | Rendah | **Phase 2 (Post-MVP)** |
