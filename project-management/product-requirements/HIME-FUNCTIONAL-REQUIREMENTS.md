# Hime Functional Requirements Specification

- **Document ID:** HIME-FUNCTIONAL-REQUIREMENTS
- **Task ID:** TASK-02-04 / TASK-02-04D (Product Requirements & Experience Architecture)
- **Phase:** Phase 02 — Nusa Reverse Engineering & Architecture Discovery (Product Specification)
- **Authority:** Lead Systems Architect & Senior Software Quality Engineer
- **Status:** **APPROVED SPECIFICATION**
- **Date:** 2026-11-08

---

## 1. Executive Summary

Dokumen ini mendokumentasikan katalog kebutuhan fungsional (*Functional Requirements*) untuk **Hime**, mencakup total **33 Kebutuhan Fungsional Unik** (24 Must Have, 5 Should Have, 4 Could Have, serta 3 inisiatif Won't Have).

Seluruh kebutuhan dikelompokkan ke dalam 13 domain fungsional terstruktur:
1. `FR-CORE`: Core AI Interaction
2. `FR-AGENT`: Agent Management & Orchestration
3. `FR-FILE`: File & Storage Intelligence
4. `FR-DOC`: Document Workspace & Generation
5. `FR-BROWSER`: Browser Automation
6. `FR-DEVICE`: Device Integration (Desktop & Mobile)
7. `FR-PROD`: Personal Productivity & Utilities
8. `FR-DEV`: Developer Workspace & Code Tools
9. `FR-MEM`: Memory, Context & Knowledge Management
10. `FR-NOTIF`: Notification & Hardware HUD Alerts
11. `FR-SEC`: Security, Secrets & Human-in-the-Loop Permissions
12. `FR-SYS`: System Integration, Windowing & Process Supervision
13. `FR-XPLAT`: Cross-Platform Support (Windows & Linux)

Setiap kebutuhan diberi atribut lengkap: **ID**, **Nama**, **Deskripsi**, **User Value**, **Acceptance Criteria**, **Prioritas (MoSCoW)**, **Dependensi**, **Risiko Rekayasa**, dan **Klasifikasi MVP**.

---

## 2. Functional Requirements Catalog

### Domain 1: FR-CORE (Core AI Interaction)

#### FR-CORE-01: Global Omnibox Activation & Dynamic Island Expansion
- **Nama:** Pemanggilan Omnibox Instan via Pintasan Global & Interaksi Kapsul
- **Deskripsi:** Sistem harus menyediakan kolom prompt Omnibox yang dapat dibuka dan ditutup dari jendela aplikasi mana pun melalui tombol shortcut global terstandarisasi (`Alt + Space` default). Sesuai arahan Project Owner (DIR-01), **Mode Klik (*Click-to-Expand*) ditetapkan sebagai default behavior pembukaan antarmuka**, sedangkan Mode Hover menjadi preferensi yang dapat dikonfigurasi pengguna di pengaturan.
- **User Value:** Pengguna dapat mengetikkan instruksi atau pertanyaan secara instan tanpa memindahkan tangan dari keyboard atau beralih jendela, bebas dari pembukaan tak sengaja saat kursor melintas.
- **Acceptance Criteria:**
  1. Menekan `Alt + Space` membuka Omnibox dalam waktu $\le 100\text{ ms}$ (Target) / $\le 140\text{ ms}$ (Ambang batas).
  2. Fokus kursor input otomatis berada pada text field Omnibox.
  3. Klik kiri pada kapsul idle Tier 1 membuka panel ekspansi Tier 2/3 (Default).
  4. Menekan `Esc` seketika menutup Omnibox dan mengembalikan fokus ke jendela aktif sebelumnya tanpa mencuri kursor ketik.
- **Prioritas (MoSCoW):** **MUST HAVE**
- **Dependencies:** `FR-SYS-01` (Global Hotkey Register), `FR-SYS-03` (Window Focus Manager).
- **Risk:** Rendah.
- **MVP Classification:** **MVP**

#### FR-CORE-02: Multi-Provider Streaming Token Relay
- **Nama:** Streaming Token Percakapan Real-Time melalui 9Router
- **Deskripsi:** Sistem harus mengalirkan respons model bahasa kata-per-kata (*streaming token*) secara real-time dari 9Router loopback gateway ke antarmuka pengguna tanpa jeda buffering yang terasa.
- **User Value:** Pengguna segera melihat respons awal tanpa menunggu seluruh paragraf selesai dihasilkan.
- **Acceptance Criteria:**
  1. Waktu hingga kemunculan token pertama (*Time-to-First-Token / TTFT*) $\le 800\text{ ms}$ pada koneksi normal.
  2. Rendering teks Markdown (bold, italic, list, code block) terformat otomatis saat streaming berlangsung.
  3. Menyediakan tombol `[ Stop Streaming ]` yang langsung memutus request jaringan jika ditekan.
- **Prioritas (MoSCoW):** **MUST HAVE**
- **Dependencies:** 9Router Service (`http://127.0.0.1:20128/v1`), EventSource / SSE Parser.
- **Risk:** Menengah (Stabilitas koneksi jaringan ke penyedia model pihak ketiga).
- **MVP Classification:** **MVP**

#### FR-CORE-03: Multi-Turn Conversation Threading
- **Nama:** Pengelolaan Riwayat Percakapan Multi-Turn
- **Deskripsi:** Sistem harus menyimpan dan mengelola riwayat obrolan dalam format percakapan bertahap (*multi-turn dialog*) dengan isolasi konteks per-sesi.
- **User Value:** Pengguna dapat melakukan penyempurnaan instruksi secara interaktif berdasarkan jawaban AI sebelumnya.
- **Acceptance Criteria:**
  1. Konteks percakapan tersimpan secara persisten pada basis data SQLite lokal.
  2. Pengguna dapat membuat sesi percakapan baru (*New Chat*), melihat riwayat sesi lama, dan menghapus sesi.
  3. Sistem membatasi jendela token konteks (*sliding window context*) agar tidak melebihi kapasitas context window model LLM aktif.
- **Prioritas (MoSCoW):** **MUST HAVE**
- **Dependencies:** SQLite Storage Engine, `FR-MEM-01`.
- **Risk:** Rendah.
- **MVP Classification:** **MVP**

---

### Domain 2: FR-AGENT (Agent Management & Orchestration)

#### FR-AGENT-01: Agent Activity Stream
- **Nama:** Visualisasi Transparan Alur Aktivitas Agen Hermes
- **Deskripsi:** Sistem harus menampilkan status operasional, rencana tindakan (*action plan*), pemanggilan alat (*tool invocations*), dan progres kerja Hermes Agent secara real-time tanpa menampilkan pemikiran internal privat (*private chain-of-thought*).
- **User Value:** Pengguna memahami apa yang sedang dikerjakan agen secara transparan tanpa kebingungan atau kekhawatiran.
- **Acceptance Criteria:**
  1. Setiap tindakan agen menampilkan badge nama alat dan parameter ringkas (misal: `[Tool: fs_read_file] "config.toml"`).
  2. Indikator progres visual menampilkan tahapan saat ini (misal: "Langkah 2 dari 5: Mengekstrak dependensi...").
  3. Keluaran stderr / stdout hasil eksekusi alat diringkas dan dapat di-expand oleh pengguna.
- **Prioritas (MoSCoW):** **MUST HAVE**
- **Dependencies:** Hermes Agent Named Pipe IPC protocol, Hime Core Orchestrator.
- **Risk:** Menengah (Sinkronisasi status asynchronous antar proses).
- **MVP Classification:** **MVP**

#### FR-AGENT-02: Task Lifecycle Control (Cancel & Retry)
- **Nama:** Kendali Siklus Hidup Tugas Agen (Batal & Ulang)
- **Deskripsi:** Sistem harus memberikan kendali penuh kepada pengguna untuk membatalkan eksekusi tugas agen yang sedang berjalan atau mengulang kembali tugas yang gagal.
- **User Value:** Pengguna tidak merasa terjebak jika agen melakukan tindakan yang keliru atau loop berkepanjangan.
- **Acceptance Criteria:**
  1. Menekan tombol `[ Batal ]` mengirimkan sinyal interupsi ke Hermes Agent dalam waktu < 200 ms.
  2. Sub-proses shell yang diluncurkan oleh agen diterminasi secara bersih tanpa meninggalkan proses yatim (*orphan processes*).
  3. Opsi `[ Ulangi ]` memungkinkan pengguna mengedit instruksi sebelum mengirimkan kembali.
- **Prioritas (MoSCoW):** **MUST HAVE**
- **Dependencies:** `FR-AGENT-01`, `FR-SYS-02` (Process Supervisor).
- **Risk:** Menengah.
- **MVP Classification:** **MVP**

#### FR-AGENT-03: Background Long-Running Task Supervisor
- **Nama:** Pengawasan Tugas Latar Belakang Jangka Panjang
- **Deskripsi:** Sistem harus mampu mengeksekusi tugas agen yang membutuhkan waktu lama (misal: riset web mendalam atau crawling 50 berkas) di latar belakang sambil memperbarui status ringkas pada Dynamic Island.
- **User Value:** Pengguna dapat terus bekerja pada aplikasi lain sementara tugas berat diselesaikan AI secara mandiri.
- **Acceptance Criteria:**
  1. Kapsul idle menampilkan badge progres ringkas saat tugas latar belakang berjalan.
  2. Menampilkan notifikasi visual dan audio halus saat tugas latar belakang selesai.
  3. Membatasi alokasi CPU sub-proses agar tidak membuat sistem host mengalami perlambatan (*freezing*).
- **Prioritas (MoSCoW):** **SHOULD HAVE**
- **Dependencies:** `FR-AGENT-01`, `FR-SYS-02`.
- **Risk:** Tinggi (Manajemen utilisasi CPU/RAM pada laptop pengguna).
- **MVP Classification:** **Phase 2**

---

### Domain 3: FR-FILE (File & Storage Intelligence)

#### FR-FILE-01: Drag-and-Drop File Context Ingestion
- **Nama:** Penyeretan Berkas Langsung sebagai Konteks Instruksi AI
- **Deskripsi:** Sistem harus mendeteksi penyeretan berkas (*drag-and-drop*) dari file manager sistem operasi ke Dynamic Island Hime dan mengekstrak isinya sebagai konteks masukan AI.
- **User Value:** Pengguna tidak perlu mencari path berkas secara manual; cukup seret berkas dokumen untuk dianalisis AI.
- **Acceptance Criteria:**
  1. Menyeret berkas ke atas kapsul memicu tampilan visual *"Drop untuk ekstraksi konteks"*.
  2. Mendukung berkas teks, Markdown, PDF, Python, Rust, JSON, YAML, dan gambar (PNG, JPG).
  3. Berkas yang disematkan muncul sebagai badge chip interaktif pada Omnibox.
- **Prioritas (MoSCoW):** **MUST HAVE**
- **Dependencies:** OLE COM `IDropTarget` (Windows) / X11/Wayland DnD (Linux), Local File Parser.
- **Risk:** Menengah (Penanganan berkas biner berukuran sangat besar).
- **MVP Classification:** **MVP**

#### FR-FILE-02: 5-Dimensional Duplicate Detection
- **Nama:** Deteksi Duplikasi Berkas Berbasis Verifikasi 5 Dimensi
- **Deskripsi:** Sistem harus mampu memindai dan mengidentifikasi relasi berkas berdasarkan 5 dimensi: (1) Ukuran berkas, (2) Hash konten kriptografis SHA-256, (3) Nama berkas, (4) Metadata EXIF/atribut, dan (5) Kesamaan semantik dokumen. Hash SHA-256 adalah indikator definitif kesamaan konten bit-per-bit. Kesamaan semantik HANYA digunakan untuk menemukan berkas yang serupa (mirip), BUKAN untuk menyatakan duplikat secara definitif.
- **User Value:** Pengguna dapat mengidentifikasi duplikat sejati tanpa salah menghapus berkas berbeda yang memiliki nama yang sama.
- **Acceptance Criteria:**
  1. Mengklasifikasikan relasi berkas ke dalam 5 kategori: Exact Duplicate (Confidence 100%), Probable Duplicate (70-90%), Similar Content (50-70%), Name Collision (0% kesamaan konten), dan Unrelated File.
  2. Dua file hanya diklasifikasikan sebagai *Exact Duplicate* jika ukuran dan hash SHA-256 identik 100%.
  3. Berkas dengan nama sama namun hash berbeda diklasifikasikan sebagai *Name Collision* atau *Similar Content* dan dilarang ditimpa secara otomatis.
  4. Menampilkan pratinjau komparasi visual sebelum tindakan pembersihan.
- **Prioritas (MoSCoW):** **MUST HAVE**
- **Dependencies:** Hashing Engine, SQLite Document Index.
- **Risk:** Rendah.
- **MVP Classification:** **MVP**

#### FR-FILE-03: Download Conflict Resolution & Intelligent Save Location
- **Nama:** Resolusi Konflik Unduhan & Penentuan Lokasi Simpan Cerdas
- **Deskripsi:** Sistem harus memonitor folder `Downloads`, mendeteksi konflik penamaan atau berkas duplikat baru, menganalisis ekstensi/isi berkas, dan menyarankan lokasi penyimpanan terorganisir.
- **User Value:** Mencegah penumpukan ribuan berkas acak di folder Downloads dan menghindari penimpaan data penting secara tidak sengaja.
- **Acceptance Criteria:**
  1. Saat berkas baru masuk ke `Downloads`, Hime memeriksa konflik penamaan dengan koleksi yang sudah ada.
  2. Menampilkan rekomendasi lokasi simpan semantik (misal: PDF tagihan $\rightarrow$ `Dokumen/Finansial/2026/`).
  3. Pengguna dapat memilih: `[ Simpan ke Lokasi Saran ]`, `[ Buat Versi Baru (v2) ]`, atau `[ Biarkan di Downloads ]`.
- **Prioritas (MoSCoW):** **MUST HAVE**
- **Dependencies:** File System Watcher, Metadata Classifier, SQLite Storage Index.
- **Risk:** Menengah.
- **MVP Classification:** **MVP**

#### FR-FILE-04: Reliable Safe File Move Lifecycle & 30-Day Undo Journal
- **Nama:** Siklus Hidup Pemindahan Berkas Aman & Jurnal Pembatalan 30 Hari
- **Deskripsi:** Setiap operasi pemindahan berkas wajib mengikuti siklus hidup aman: (1) Same-volume move dijamin atomik melalui OS atomic rename, (2) Cross-volume move tidak diklaim atomik secara keseluruhan melainkan menggunakan prosedur multi-tahap (salin ke `.hime_tmp` $\rightarrow$ verifikasi hash SHA-256 $\rightarrow$ rename atomik $\rightarrow$ hapus berkas asal hanya setelah verifikasi valid). Berkas asal tidak boleh hilang jika operasi gagal. Seluruh pemindahan dicatat pada tabel SQLite `storage_journal` dengan kemampuan pembatalan (*Undo*) selama 30 hari.
- **User Value:** Jaminan zero data loss tanpa rasa cemas kehilangan berkas berharga saat penataan storage.
- **Acceptance Criteria:**
  1. Same-volume move memanfaatkan atomic rename native OS (`MoveFileExW` / `rename()`).
  2. Cross-volume move mempertahankan berkas asal utuh jika terjadi kegagalan (disk full, hash mismatch, process crash).
  3. Menampilkan banner toast dengan tombol `[ Batalkan / Undo ]` selama 30 detik pasca operasi.
  4. Entri jurnal Undo tersimpan selama 30 hari di basis data lokal `storage_journal`.
  5. Jika berkas di destinasi telah dimodifikasi pengguna sebelum Undo dijalankan, sistem menolak Undo otomatis dan meminta konfirmasi eksplisit pengguna untuk mencegah penimpaan data baru.
- **Prioritas (MoSCoW):** **MUST HAVE**
- **Dependencies:** SQLite Journal Table, Hash Verification Engine, Atomic Filesystem Operations.
- **Risk:** Rendah.
- **MVP Classification:** **MVP**

---

### Domain 4: FR-DOC (Document Workspace & Generation)

#### FR-DOC-01: Markdown Document Editor & Preview
- **Nama:** Editor & Pratinjau Dokumen Markdown Terpadu
- **Deskripsi:** Sistem harus menyediakan panel pratinjau dan penyuntingan dokumen Markdown dengan rendering tabel, syntax highlighting blok kode, dan tipografi rapi pada Tier 3 canvas.
- **User Value:** Pengguna dapat langsung meninjau dan menyunting draf tulisan yang dihasilkan AI tanpa membuka editor eksternal.
- **Acceptance Criteria:**
  1. Mendukung sintaksis GitHub Flavored Markdown (GFM) lengkap.
  2. Blok kode dilengkapi dengan syntax highlighting otomatis dan tombol `[ Salin Kode ]`.
  3. Dokumen dapat disimpan langsung ke file `.md` lokal.
- **Prioritas (MoSCoW):** **MUST HAVE**
- **Dependencies:** Markdown Parser & Renderer Component.
- **Risk:** Rendah.
- **MVP Classification:** **MVP**

#### FR-DOC-02: Document Multi-Format Exporter
- **Nama:** Ekspor Dokumen ke Format Populer (.pdf, .docx, .html)
- **Deskripsi:** Sistem harus mampu mengekspor dokumen Markdown yang dibuat AI ke dalam format PDF standar atau dokumen Office Word (`.docx`).
- **User Value:** Dokumen hasil kerja AI langsung siap dibagikan kepada rekan kerja atau dosen tanpa konversi manual tambahan.
- **Acceptance Criteria:**
  1. Ekspor menghasilkan layout yang rapi dengan margin dan font standar.
  2. Menghasilkan file dalam waktu $\le 3\text{ detik}$.
- **Prioritas (MoSCoW):** **COULD HAVE**
- **Dependencies:** Local Document Export Engine.
- **Risk:** Menengah (Ukuran dependensi rendering PDF).
- **MVP Classification:** **Phase 3**

---

### Domain 5: FR-BROWSER (Browser Automation)

#### FR-BROWSER-01: Headless Web Scraping & Research Fetcher
- **Nama:** Pengambilan Data Web & Ekstraksi Teks Otomatis
- **Deskripsi:** Sistem harus mampu membuka URL web publik, mengekstrak konten artikel utama, membersihkan iklan dan elemen DOM yang tidak relevan, lalu meneruskannya ke AI sebagai konteks riset.
- **User Value:** Pengguna dapat meminta riset dari sumber web terkini tanpa membuka banyak tab peramban secara manual.
- **Acceptance Criteria:**
  1. Mampu mengekstrak teks bersih dari halaman artikel/blog dalam waktu $\le 4\text{ detik}$.
  2. Menyaring elemen script, cookie banners, dan iklan.
  3. Tautan referensi yang dikutip disajikan dalam bentuk bare URL yang valid.
- **Prioritas (MoSCoW):** **SHOULD HAVE**
- **Dependencies:** HTTP Fetcher / Readability Engine.
- **Risk:** Menengah.
- **MVP Classification:** **Phase 2**

#### FR-BROWSER-02: Interactive Browser Automation with CAPTCHA Fallback
- **Nama:** Otomatisasi Peramban Interaktif dengan Fallback CAPTCHA
- **Deskripsi:** Sistem harus mampu mengotomatisasi interaksi web kompleks (mengisi formulir, navigasi multi-halaman). Jika sistem mendeteksi rintangan CAPTCHA / Cloudflare Turnstile, sistem wajib berhenti secara otomatis dan membuka jendela browser terlihat agar diselesaikan oleh pengguna.
- **User Value:** Tugas otomatisasi web berjalan lancar tanpa melanggar prinsip anti-bot bypass.
- **Acceptance Criteria:**
  1. Deteksi CAPTCHA langsung menghentikan agen dan memunculkan jendela Chromium interaktif.
  2. Menampilkan pesan panduan: *"Silakan selesaikan verifikasi pada jendela peramban untuk melanjutkan"*.
  3. Melanjutkan otomatisasi setelah pengguna menyelesaikan verifikasi secara manual.
- **Prioritas (MoSCoW):** **COULD HAVE**
- **Dependencies:** Headless Browser Engine (Playwright / Puppeteer Bridge).
- **Risk:** Tinggi (Kompleksitas instalasi browser binaries).
- **MVP Classification:** **Phase 3**

---

### Domain 6: FR-DEVICE (Device Integration & Cross-Device Sync)

#### FR-DEVICE-01: Android Storage Inspector via Local Link
- **Nama:** Inspeksi & Manajemen Memori Android dari Desktop
- **Deskripsi:** Sistem harus mampu mendeteksi perangkat Android yang terhubung via kabel USB (ADB) atau Wi-Fi P2P Companion App, membaca statistik memori, dan mengidentifikasi folder media yang memakan ruang besar.
- **User Value:** Pengguna dapat mengelola dan merapikan penyimpanan smartphone langsung dari laptop tanpa navigasi manual di layar HP yang kecil.
- **Acceptance Criteria:**
  1. Mendeteksi kapasitas sisa penyimpanan internal dan kartu SD eksternal Android.
  2. Mengidentifikasi jumlah berkas pada folder `Screenshots`, `Camera`, dan `WhatsApp Media`.
  3. Menampilkan kartu ringkasan visual status penyimpanan HP di panel Hime.
- **Prioritas (MoSCoW):** **SHOULD HAVE**
- **Dependencies:** Android ADB Bridge / Hime Companion Daemon.
- **Risk:** Tinggi (Variasi driver USB dan kompatibilitas vendor Android).
- **MVP Classification:** **Phase 2**

#### FR-DEVICE-02: Safe Cross-Device Media Backup & Archiving
- **Nama:** Pencadangan & Pengarsipan Media Lintas Perangkat yang Aman
- **Deskripsi:** Sistem harus mampu memindahkan berkas foto/dokumen terpilih dari perangkat Android ke hard disk laptop, memvalidasi integritas hash, dan menghapus berkas asli pada HP hanya setelah konfirmasi izin eksplisit pengguna.
- **User Value:** Mengosongkan ruang memori HP dengan cepat dan aman tanpa risiko foto penting hilang.
- **Acceptance Criteria:**
  1. Seluruh berkas yang ditransfer divalidasi checksum SHA-256 sebelum penghapusan sumber.
  2. Wajib menampilkan modal otorisasi izin sebelum menghapus berkas pada HP.
  3. Mempertahankan metadata EXIF tanggal dan lokasi pengambilan foto asli.
- **Prioritas (MoSCoW):** **SHOULD HAVE**
- **Dependencies:** `FR-DEVICE-01`, `FR-SEC-01`.
- **Risk:** Tinggi (Risiko kehilangan data jika transfer gagal di tengah jalan).
- **MVP Classification:** **Phase 2**

---

### Domain 7: FR-PROD (Personal Productivity & Utilities)

#### FR-PROD-01: Local Task Checklist & Daily Goals
- **Nama:** Pengelolaan Daftar Tugas Harian & Pelacakan Target
- **Deskripsi:** Sistem harus menyediakan panel checklist tugas ringkas dengan persistensi lokal SQLite, kemampuan penambahan tugas instan via teks alami, dan indikator penyelesaian visual/audio.
- **User Value:** Pengguna dapat mencatat dan mencentang to-do harian tanpa membuka aplikasi manajemen proyek yang berat.
- **Acceptance Criteria:**
  1. Menambah tugas baru via input teks singkat: *"Kirim laporan audit sebelum jam 4 sore"*.
  2. Mencentang tugas memicu animasi centang dan audio chime halus.
  3. Data to-do tersimpan persisten di basis data lokal `hime_tasks.db`.
- **Prioritas (MoSCoW):** **MUST HAVE**
- **Dependencies:** SQLite Task Store, Audio Chime Utility.
- **Risk:** Rendah.
- **MVP Classification:** **MVP**

#### FR-PROD-02: Ambient Hydration & Break Reminder (Optional Lifestyle Utility)
- **Nama:** Pengingat Ambient Hidrasi & Istirahat Kerja (Fitur Opsional)
- **Deskripsi:** Sistem dapat menyediakan pelacak konsumsi air minum ambient melalui cincin progres visual melingkar pada kapsul idle, dengan tombol catat cepat (*Quick Log*) dan tunda (*Snooze*).
- **User Value:** Menjaga kebiasaan minum air pengguna saat fokus bekerja di depan layar.
- **Acceptance Criteria:**
  1. Cincin progres hijau terisi proporsional dengan target harian (misal: 2.0 Liter).
  2. Tombol satu-klik `[ Minum 250ml ]` memperbarui cincin secara instan.
  3. Dinonaktifkan secara default pada instalasi awal dan dapat diaktifkan melalui menu pengaturan.
- **Prioritas (MoSCoW):** **COULD HAVE**
- **Dependencies:** Local State Manager.
- **Risk:** Rendah.
- **MVP Classification:** **Phase 2 (Optional Utility)**

#### FR-PROD-03: Context-Aware Prayer Time Schedule (Optional Regional Plugin)
- **Nama:** Jadwal Shalat Berbasis Lokasi & Hitung Mundur Waktu (Plugin Opsional)
- **Deskripsi:** Sistem dapat menampilkan hitung mundur waktu shalat berikutnya berdasarkan koordinat kota pengguna menggunakan perhitungan astronomis offline atau API Kemenag RI, disajikan di kartu Today.
- **User Value:** Memberikan pengingat ibadah tepat waktu bagi pengguna Muslim di Indonesia tanpa aplikasi tambahan.
- **Acceptance Criteria:**
  1. Menghitung 5 waktu shalat harian secara offline berdasarkan lintang/bujur kota pilihan.
  2. Menampilkan hitung mundur format jam:menit menuju waktu shalat berikutnya.
  3. Berupa plugin modular yang dapat diaktifkan secara opsional di menu pengaturan.
- **Prioritas (MoSCoW):** **COULD HAVE**
- **Dependencies:** Offline Prayer Calculation Math, City Coordinate Register.
- **Risk:** Rendah.
- **MVP Classification:** **Phase 2 (Optional Plugin)**
  1. Menghitung 5 waktu shalat harian secara offline berdasarkan lintang/bujur kota pilihan.
  2. Menampilkan hitung mundur format jam:menit menuju waktu shalat berikutnya.
  3. Menyediakan opsi notifikasi adzan halus saat waktu tiba.
- **Prioritas (MoSCoW):** **SHOULD HAVE**
- **Dependencies:** Offline Prayer Calculation Math, City Coordinate Register.
- **Risk:** Rendah.
- **MVP Classification:** **MVP**

---

### Domain 8: FR-DEV (Developer Workspace & Code Tools)

#### FR-DEV-01: Sandboxed Terminal Command Execution
- **Nama:** Eksekusi Perintah Terminal Terawasi dalam Sandbox
- **Deskripsi:** Sistem harus mampu mengeksekusi perintah shell (PowerShell / Bash / CMD) atas instruksi agen AI, dengan pembatasan direktori kerja, penangkapan stdout/stderr secara real-time, dan batas waktu eksekusi (*timeout*).
- **User Value:** Agen dapat menjalankan perintah build, lint, test, atau instalasi paket secara mandiri dengan pengawasan ketat.
- **Acceptance Criteria:**
  1. Perintah dijalankan di bawah hak akses pengguna standar (non-administrator).
  2. Seluruh baris stdout/stderr dialirkan langsung ke Agent Activity Stream.
  3. Perintah yang berjalan melampaui timeout (default 60 detik) otomatis dihentikan paksa.
- **Prioritas (MoSCoW):** **MUST HAVE**
- **Dependencies:** `FR-SEC-01` (Human Approval Modal), `FR-SYS-02` (Process Supervisor).
- **Risk:** Tinggi (Potensi eksekusi perintah destruktif jika tanpa izin pengguna).
- **MVP Classification:** **MVP**

#### FR-DEV-02: Code Syntax Diff & File Patching Engine
- **Nama:** Visualisasi Perbedaan Kode (Diff) & Penambalan Berkas Aman
- **Deskripsi:** Saat agen AI mengusulkan perubahan kode sumber pada repositori pengguna, sistem harus menampilkan visualisasi diff (sebelum vs sesudah) dengan highlight warna merah/hijau, dan hanya menerapkan patch setelah disetujui pengguna.
- **User Value:** Pengembang dapat meninjau perubahan baris per baris sebelum file fisik diubah di hard disk.
- **Acceptance Criteria:**
  1. Menampilkan diff side-by-side atau inline dengan penomoran baris yang akurat.
  2. Tombol `[ Terapkan Patch ]` melakukan modifikasi file secara atomik.
  3. Menolak patch jika konteks baris file asli telah berubah di disk.
- **Prioritas (MoSCoW):** **MUST HAVE**
- **Dependencies:** Diff Parser & Renderer, Atomic File Replacer.
- **Risk:** Menengah.
- **MVP Classification:** **MVP**

---

### Domain 9: FR-MEM (Memory, Context & Knowledge Management)

#### FR-MEM-01: Local SQLite FTS5 Conversation & Knowledge Index
- **Nama:** Indeks Pencarian Teks Penuh (FTS5) Obrolan & Dokumen
- **Deskripsi:** Sistem harus mengindeks seluruh percakapan lampau, catatan ringkasan, dan metadata berkas ke dalam virtual table SQLite FTS5 lokal untuk pencarian semantik dan penarikan konteks berkecepatan tinggi.
- **User Value:** Pengguna dan agen AI dapat menemukan kembali informasi lampau dalam hitungan milidetik secara privat di komputer sendiri.
- **Acceptance Criteria:**
  1. Kueri penelusuran mengembalikan hasil dalam waktu $\le 50\text{ ms}$ untuk 10.000 entri catatan.
  2. Mendukung pencarian frasa (*token phrase matching*) dan prefix search.
  3. Basis data tersimpan lokal pada folder profil data pengguna dengan hak akses ACL terbatas.
- **Prioritas (MoSCoW):** **MUST HAVE**
- **Dependencies:** SQLite3 dengan ekstensi FTS5 aktif (`[VERIFIED RUNTIME]` pada audit host).
- **Risk:** Rendah.
- **MVP Classification:** **MVP**

#### FR-MEM-02: Sliding Window & Token Context Distillation
- **Nama:** Distilasi Konteks & Manajemen Jendela Token Cerdas
- **Deskripsi:** Sistem harus memantau akumulasi token percakapan, melakukan ringkasan otomatis (*context summarization*) pada pesan-pesan lama ketika mendekati batas kuota model, dan mempertahankan fakta-fakta kunci (*core facts*).
- **User Value:** Percakapan panjang tidak crash akibat melebihi context window dan biaya token inferensi tetap hemat.
- **Acceptance Criteria:**
  1. Mencegah error `context_length_exceeded` secara proaktif.
  2. Menyimpan fakta penting (nama variabel, path proyek, preferensi pengguna) ke dalam memori kerja permanen (*System Prompt Scratchpad*).
- **Prioritas (MoSCoW):** **MUST HAVE**
- **Dependencies:** Tokenizer Estimator, 9Router Proxy.
- **Risk:** Menengah.
- **MVP Classification:** **MVP**

---

### Domain 10: FR-NOTIF (Notification & Hardware HUD Alerts)

#### FR-NOTIF-01: Transient Hardware OSD Interception (Volume & Power)
- **Nama:** Intersepsi Tampilan OSD Perangkat Keras ke Dynamic Island
- **Deskripsi:** Sistem harus mendengarkan perubahan volume master audio dan status daya baterai laptop dari sistem operasi, menampilkan indikator visual HUD elegan pada Dynamic Island selama ~2.5 detik, lalu menyusut kembali secara otomatis.
- **User Value:** Menggantikan kotak OSD Windows native yang kaku dengan antarmuka Dynamic Island modern yang tidak mengalihkan perhatian.
- **Acceptance Criteria:**
  1. Perubahan volume fisik memicu visualisasi slider HUD dalam waktu $\le 50\text{ ms}$.
  2. Menampilkan HUD tanpa mencuri fokus pengetikan dari jendela aktif (`WS_EX_NOACTIVATE`).
  3. Otomatis menutup (*auto-dismiss*) setelah 2.5 detik tanpa interaksi lanjutan.
- **Prioritas (MoSCoW):** **MUST HAVE**
- **Dependencies:** CoreAudio API / PulseAudio/PipeWire Listener, Non-Activating Overlay.
- **Risk:** Rendah.
- **MVP Classification:** **MVP**

#### FR-NOTIF-02: Actionable Task Notification Toasts
- **Nama:** Notifikasi Interaktif untuk Penyelesaian Tugas & Permintaan Izin
- **Deskripsi:** Sistem harus menyajikan pemberitahuan interaktif ketika tugas latar belakang selesai atau saat agen membutuhkan masukan pengguna, dilengkapi dengan tombol tindakan langsung di dalam notifikasi.
- **User Value:** Pengguna dapat merespons kebutuhan agen seketika dari sudut layar desktop.
- **Acceptance Criteria:**
  1. Notifikasi memuat tombol aksi: `[ Lihat Hasil ]`, `[ Setujui ]`, `[ Abaikan ]`.
  2. Klik pada notifikasi membawa jendela kanvas Hime ke latar depan.
- **Prioritas (MoSCoW):** **MUST HAVE**
- **Dependencies:** System Tray / Native Toast Bridge.
- **Risk:** Rendah.
- **MVP Classification:** **MVP**

---

### Domain 11: FR-SEC (Security, Secrets & Human-in-the-Loop Permissions)

#### FR-SEC-01: Centralized Policy Enforcement & Interactive Action Approval Modal
- **Nama:** Penegakan Kebijakan Terpusat di Hime Core & Dialog Otorisasi Interaktif
- **Deskripsi:** Hime Core harus bertindak sebagai satu-satunya pusat penegakan kebijakan (*Policy Enforcement Point & Policy Decision Point*). Hermes Agent dilarang memiliki jalur eksekusi alat mandiri yang dapat mem-bypass Hime Core. Seluruh aksi alat dievaluasi terhadap 8 izin granular (Read, Write, Modify, Delete, Execute, Network, Ext Transfer, Device) dan aksi berisiko tinggi (Execute shell, Delete file, Network, Device) wajib dicegat dan memerlukan otorisasi eksplisit pengguna. Approval modal adalah antarmuka persetujuan (bukan batas keamanan kriptografis), dan jeda 500 ms adalah kontrol ergonomi anti-slip.
- **User Value:** Pengguna memiliki kendali mutlak dan perlindungan terhadap eksekusi perintah berbahaya oleh agen.
- **Acceptance Criteria:**
  1. Hime Core memblokir setiap tool execution yang tidak membawa tiket otorisasi valid dari saluran Named Pipe IPC.
  2. Modal menampilkan rincian tindakan lengkap: Jenis izin, nama alat, perintah spesifik yang akan dijalankan, dan target path.
  3. Pilihan tombol: `[ Izinkan Sekali ]` (TTL: 60s), `[ Izinkan Selama Sesi Ini ]` (TTL: 15 menit), `[ Tolak / Batal ]`.
  4. Tombol izinkan dinonaktifkan selama 500 ms pertama sebagai kontrol ergonomi anti-slip.
  5. Memilih `Tolak` mengembalikan error izin terstruktur ke agen dan mencegah pengulangan perintah yang sama (*anti-looping guard*).
  6. Setiap tindakan dicatat secara permanen di tabel SQLite `tool_audit_log`.
- **Prioritas (MoSCoW):** **MUST HAVE**
- **Dependencies:** Hime Core Security Policy Engine, Named Pipe IPC Validator, `tool_audit_log` Table.
- **Risk:** Sangat Tinggi (Komponen pertahanan keamanan utama sistem).
- **MVP Classification:** **MVP**

#### FR-SEC-02: Hardware-Backed Credential Encryption & 9Router Hardening
- **Nama:** Enkripsi Kredensial Berbasis OS & Mitigasi Keamanan 9Router
- **Deskripsi:** Sistem dilarang keras menyimpan kunci API, token sesi, atau password dalam format teks polos (`.txt` / `.json`). Seluruh rahasia wajib dienkripsi menggunakan Windows DPAPI (`CryptProtectData`) pada Windows, atau FreeDesktop Secret Service API (`libsecret`) pada Linux. Termasuk memitigasi kerentanan database SQLite 9Router (`%APPDATA%\9router\db\data.sqlite` pada Windows / `~/.9router/db/data.sqlite` pada Linux) yang menyimpan kredensial penyedia model dalam teks polos melalui pembungkus enkripsi DPAPI Hime Core atau internal maintained patch jika upstream tidak mengadopsi enkripsi (diklasifikasikan sebagai kontrol mitigasi terencana Hime Core `[PLANNED CONTROL]`, bukan kontrol native 9Router).
- **User Value:** Kunci API dan data kredensial AI berbayar pengguna terlindungi dari pencurian malware lokal.
- **Acceptance Criteria:**
  1. Database kredensial menyimpan ciphertext blob terenkripsi DPAPI / Secret Service.
  2. Direktori data 9Router (`%APPDATA%\9router\` pada Windows / `~/.9router/` pada Linux) diisolasi dengan Windows ACL / Linux permission `0700` (hanya dapat diakses akun user pemilik).
  3. Fallback password bawaan 9Router `"123456"` pada kode sumber dimitigasi dengan injeksi token acak 32 karakter dari generator CSPRNG via environment variable `INITIAL_PASSWORD` saat proses diluncurkan.
  4. Dekripsi hanya dapat dilakukan pada konteks user logon yang sama di mesin lokal.
- **Prioritas (MoSCoW):** **MUST HAVE**
- **Dependencies:** OS Cryptographic Native Libraries, CSPRNG Token Generator.
- **Risk:** Tinggi (Mitigasi kerentanan backend eksternal 9Router).
- **MVP Classification:** **MVP**

#### FR-SEC-03: Localhost-Only IPC Transport Isolation
- **Nama:** Isolasi Transport Komunikasi IPC Antar-Proses ke Loopback
- **Deskripsi:** Seluruh saluran komunikasi antara Hime Desktop, Hime Core, Hermes Agent, dan 9Router harus dibatasi secara ketat pada antarmuka loopback lokal (`127.0.0.1` / Named Pipes `\\.\pipe\hime-*` / Unix Domain Sockets) dan dilarang mengikat ke antarmuka publik (`0.0.0.0`).
- **User Value:** Mencegah peretas pada jaringan Wi-Fi lokal yang sama mengakses gateway inferensi atau mengontrol agen AI dari jauh.
- **Acceptance Criteria:**
  1. 9Router dipaksa mengikat pada `127.0.0.1:20128`.
  2. Saluran IPC antar-proses dikonfigurasi terisolasi loopback per-platform: pada Windows menggunakan Named Pipe (`\\.\pipe\hime-*`) dengan DACL menolak akses jaringan (*Network Access Denied*) dan verifikasi client PID; pada Linux menggunakan Unix Domain Socket (`$XDG_RUNTIME_DIR/hime/*.sock`) dengan izin mode `0600` dan verifikasi peer credential `SO_PEERCRED`.
  3. Upaya koneksi dari IP eksternal ditolak seketika pada level socket OS.
- **Prioritas (MoSCoW):** **MUST HAVE**
- **Dependencies:** Socket Configuration, OS Security Descriptors.
- **Risk:** Sangat Tinggi.
- **MVP Classification:** **MVP**

---

### Domain 12: FR-SYS (System Integration, Windowing & Process Supervision)

#### FR-SYS-01: 3-Tier Adaptive Elastic Window Controller
- **Nama:** Pengendali Jendela Kanvas Elastis 3-Tier
- **Deskripsi:** Shell antarmuka harus mampu bertransformasi ukuran dan posisinya secara dinamis tanpa kedipan (*zero-flicker*) di antara 3 tingkat dimensi: Tier 1 (~34 px Idle Capsule), Tier 2 (~212 px HUD / Quick Action), dan Tier 3 (480–600 px Rich Chat / Workspace), dengan penanganan per-pixel click-through transparency pada area kanvas kosong.
- **User Value:** Antarmuka menyesuaikan bentuk secara elegan mengikuti bobot muatan konten tanpa memakan ruang desktop berlebih.
- **Acceptance Criteria:**
  1. Transisi ukuran jendela berjalan mulus ($\le 240\text{ ms}$) menggunakan interpolasi GPU DirectComposition / Wayland compositor.
  2. Area kanvas transparan di luar poligon aktif meneruskan klik mouse ke jendela latar belakang via `WM_NCHITTEST` (`HTTRANSPARENT`).
  3. Jendela memiliki extended style `WS_EX_TOPMOST` dan `WS_EX_TOOLWINDOW` agar tidak mengotori Taskbar dan dialog Alt+Tab.
- **Prioritas (MoSCoW):** **MUST HAVE**
- **Dependencies:** Tauri 2 Window API, Windows Subclassing WindowProc / Linux Layer Shell.
- **Risk:** Tinggi (Interaksi grafis native windowing).
- **MVP Classification:** **MVP**

#### FR-SYS-02: Managed Multi-Process Supervisor (Hime Core Lifecycle)
- **Nama:** Pengawas Siklus Hidup Proses Mandiri
- **Deskripsi:** Hime Core harus bertindak sebagai supervisor proses yang mengelola peluncuran (*spawning*), pemantauan kesehatan (*health-check*), dan terminasi bersih (*graceful shutdown*) terhadap proses backend eksternal (Hermes Agent Python runtime dan 9Router Node.js gateway).
- **User Value:** Pengguna tidak perlu menjalankan script terminal atau command prompt secara manual; sistem menyala dan mati secara terpadu dalam satu klik.
- **Acceptance Criteria:**
  1. Saat Hime dibuka, Hime Core meluncurkan 9Router dan Hermes Agent secara otomatis di latar belakang.
  2. Mengaitkan seluruh proses anak ke Windows Job Object (`JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`) sehingga jika Hime crash atau keluar, seluruh proses anak mati secara otomatis tanpa meninggalkan proses zombie.
  3. Menginjeksi token acak 32 karakter dari generator CSPRNG via environment variable `INITIAL_PASSWORD` saat proses diluncurkan, meniadakan fallback default password `"123456"`.
- **Prioritas (MoSCoW):** **MUST HAVE**
- **Dependencies:** Rust `std::process::Command`, Windows Job Objects API / Linux cgroups/prctl.
- **Risk:** Sangat Tinggi (Integritas siklus hidup multi-proses).
- **MVP Classification:** **MVP**

#### FR-SYS-03: System Tray Background Integration
- **Nama:** Integrasi Menu Konteks Baki Sistem Operasi
- **Deskripsi:** Sistem harus mendaftarkan ikon notifikasi pada system tray Windows / status notifier Linux, menyediakan menu konteks cepat untuk pengaturan, toggle autostart, pemilihan monitor, dan penutupan bersih aplikasi.
- **User Value:** Aplikasi tetap dapat berjalan di latar belakang dan mudah diakses kembali saat jendela desktop ditutup.
- **Acceptance Criteria:**
  1. Klik kanan pada ikon tray menampilkan menu konteks native Win32 / Freedesktop.
  2. Menyediakan opsi: `Buka Hime`, `Pengaturan...`, `Mulai saat Boot`, `Status Gateway 9Router`, `Keluar Bersih`.
  3. Mendukung registrasi ulang ikon secara otomatis jika `explorer.exe` atau desktop shell mengalami crash (*TaskbarCreated* message).
- **Prioritas (MoSCoW):** **MUST HAVE**
- **Dependencies:** Tauri System Tray Plugin.
- **Risk:** Rendah.
- **MVP Classification:** **MVP**

---

### Domain 13: FR-XPLAT (Cross-Platform Support)

#### FR-XPLAT-01: Modular Platform Abstraction Layer (PAL)
- **Nama:** Lapisan Abstraksi Platform Antarmuka OS
- **Deskripsi:** Hime Core dan Hime Desktop harus mengisolasi seluruh panggilan sistem operasi native ke dalam antarmuka Rust trait / TypeScript interface `PlatformAdapter` yang membedakan implementasi Windows dan Linux secara bersih tanpa mencemari logika bisnis utama.
- **User Value:** Kode domain Hime bersifat portabel dan dapat dikompilasi secara konsisten di Windows maupun Linux.
- **Acceptance Criteria:**
  1. Hime Core tidak memuat pemanggilan `winapi` atau `windows-rs` di luar modul adapter `platform/windows/`.
  2. Implementasi Linux diisolasi pada modul `platform/linux/`.
  3. Seluruh unit test logika bisnis dapat berjalan di lingkungan simulasi mock tanpa ketergantungan API host tertentu.
- **Prioritas (MoSCoW):** **MUST HAVE**
- **Dependencies:** Rust Traits Architecture, Cargo Feature Flags (`windows`, `linux`).
- **Risk:** Menengah.
- **MVP Classification:** **MVP**

#### FR-XPLAT-02: Linux Display Server Detection & Adaptation (X11 vs Wayland)
- **Nama:** Deteksi & Adaptasi Server Tampilan Linux (X11 vs Wayland)
- **Deskripsi:** Pada sistem operasi Linux, Hime harus mendeteksi jenis display server yang aktif (`XDG_SESSION_TYPE`) dan memilih mekanisme penempatan jendela overlay yang sesuai: X11 Window Manager hints (`_NET_WM_WINDOW_TYPE_DOCK`) atau Wayland `wlr-layer-shell-unstable-v1` protocol.
- **User Value:** Pengguna Linux di GNOME maupun KDE Plasma mendapatkan pengalaman Dynamic Island yang konsisten dengan batasan compositing masing-masing.
- **Acceptance Criteria:**
  1. Mendeteksi sesi X11 atau Wayland secara otomatis saat startup.
  2. Pada Wayland, menggunakan layer-shell protocol untuk memposisikan overlay di tepi atas layar.
  3. Jika compositing Wayland tidak mendukung overlay mengapung bebas, menyediakan opsi fallback menjadi jendela utilitas terpusat (*Floating Utility Mode*).
- **Prioritas (MoSCoW):** **SHOULD HAVE**
- **Dependencies:** Wayland Protocols Crate, X11 RB Crate.
- **Risk:** Sangat Tinggi (Fragmentasi ekosistem Wayland di Linux).
- **MVP Classification:** **Phase 2**

---

## 3. Matriks Rekapitulasi MoSCoW & Rencana Peluncuran

Katalog spesifikasi Hime mendefinisikan total **33 Kebutuhan Fungsional Unik** (terbagi atas 24 Must Have, 5 Should Have, dan 4 Could Have, ditambah 3 inisiatif Won't Have). Tabel berikut menyajikan rekapitulasi prioritas seluruh kebutuhan fungsional Hime:

| Kategori Prioritas | Total Kebutuhan | Daftar Requirement ID | Target Peluncuran |
|:---|:---:|:---|:---:|
| **MUST HAVE** (Kritis untuk Kelayakan Produk) | **24 Kebutuhan** | `FR-CORE-01`, `FR-CORE-02`, `FR-CORE-03`, `FR-AGENT-01`, `FR-AGENT-02`, `FR-FILE-01`, `FR-FILE-02`, `FR-FILE-03`, `FR-FILE-04`, `FR-DOC-01`, `FR-PROD-01`, `FR-DEV-01`, `FR-DEV-02`, `FR-MEM-01`, `FR-MEM-02`, `FR-NOTIF-01`, `FR-NOTIF-02`, `FR-SEC-01`, `FR-SEC-02`, `FR-SEC-03`, `FR-SYS-01`, `FR-SYS-02`, `FR-SYS-03`, `FR-XPLAT-01` | **MVP (Fase 1)** |
| **SHOULD HAVE** (Penting, Tidak Menahan MVP Awal) | **5 Kebutuhan** | `FR-AGENT-03`, `FR-BROWSER-01`, `FR-DEVICE-01`, `FR-DEVICE-02`, `FR-XPLAT-02` | **Phase 2 (Post-MVP)** |
| **COULD HAVE** (Fitur Lanjutan Bernilai Tinggi) | **4 Kebutuhan** | `FR-DOC-02`, `FR-BROWSER-02`, `FR-PROD-02`, `FR-PROD-03` | **Phase 2 (Opsional) / Phase 3** |
| **TOTAL KEBUTUHAN AKTIF** | **33 Kebutuhan** | **24 Must + 5 Should + 4 Could** | **Fase 1 s/d Fase 3** |
| **WON'T HAVE** (Di Luar Cakupan Siklus Saat Ini) | **3 Inisiatif** | Autonomous System Takeover, Public P2P Cloud, Local 70B LLM Native Hosting | **Future Vision** |
