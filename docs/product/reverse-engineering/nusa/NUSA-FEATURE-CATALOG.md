# Nusa Feature Catalog & Internal Mechanisms

- **Document ID:** NUSA-FEATURE-CATALOG
- **Task ID:** TASK-02-01 / TASK-02-02A (Phase 02 — Nusa Reverse Engineering)
- **Target Application:** Nusa Dynamic Island for Windows (v1.2.0.0)
- **Authority:** Senior Systems Analyst & Feature Architecture Specialist
- **Status:** **CORRECTED — EVIDENCE-QUALIFIED CATALOG**
- **Date:** 2026-11-08

---

## 1. Executive Summary

Dokumen ini mendokumentasikan katalog terperinci dari 10 modul fitur yang teridentifikasi pada biner **Nusa Dynamic Island v1.2.0.0**.

Sesuai instruksi tata kelola bukti TASK-02-02A, setiap modul dianalisis dengan memisahkan secara ketat bukti statis (`[VERIFIED STATIC]`), pengujian runtime riil (`[VERIFIED RUNTIME]`), kesimpulan logis (`[INFERRED]`), dan hal yang belum diuji secara empiris dalam sesi saat ini (`[NOT TESTED]`).

Seluruh rekomendasi adaptasi fitur untuk **Hime Desktop** diselaraskan dengan arsitektur **Agent Activity Stream** untuk pelaporan progres, aksi perkakas (*tools*), dan konfirmasi persetujuan tindakan tanpa menampilkan private chain-of-thought internal model AI.

---

## 2. Feature Modules Breakdown

### Modul 01: Media Player & Live Lyrics

- **Klasifikasi Bukti:**
  - Keberadaan API GSMTC & URL LRCLIB: `[VERIFIED STATIC]` (Strings `.rdata` offset `0x000E083C`, `0x000E0E78`).
  - Penayangan lirik sinkron & transisi cover art runtime: `[INFERRED]` (Belum diuji ulang dengan sesi audio aktif di sesi ini / `[NOT TESTED]`).
- **Deskripsi Fungsional:**
  Mendeteksi sesi pemutaran media audio/video aktif di Windows (Spotify, browser, media player). Menampilkan judul lagu, artis, artwork cover, linimasa progres, kontrol pemutaran, serta permintaan lirik sinkron via LRCLIB.
- **Metode Akses:**
  Otomatis aktif pada kapsul saat media diputar, atau melalui tab media pada panel ekspansi.
- **Model Input Pengguna `[INFERRED]`:**
  Klik tombol Play/Pause, Next, Previous, scrubbing progress bar, dan toggle lirik (`lyrics=1|0`).
- **Output & Presentasi Visual `[INFERRED]`:**
  Album art, teks judul (font tebal), artis (font abu-abu), slider linimasa, serta deretan lirik sinkron.
- **Mekanisme Internal Terverifikasi `[VERIFIED STATIC]`:**
  - Panggilan WinRT `Windows.Media.Control.GlobalSystemMediaTransportControlsSessionManager` (GSMTC).
  - Permintaan HTTP GET ke endpoint:
    `https://lrclib.net/api/search?track_name=<title>&artist_name=<artist>`
- **Batasan Teknis:**
  Ketergantungan jaringan eksternal ke `lrclib.net`; gagal menampilkan lirik jika koneksi terputus atau metadata tidak cocok.
- **Relevansi untuk Hime Desktop:**
  - *Relevansi:* **Rendah (Opsional)**.
  - *Rekomendasi:* Bukan fitur inti asisten AI. Dynamic Island Hime diprioritaskan untuk **Agent Activity Stream** dan status eksekusi Hermes Agent; integrasi media GSMTC dapat dijadikan plugin latar belakang opsional.

---

### Modul 02: File Shelf (Temporary Drag & Drop Staging)

- **Klasifikasi Bukti:**
  - Antarmuka OLE COM (`IDropTarget`, `RegisterDragDrop`) & Folder `%LOCALAPPDATA%\Nusa\Shelf\`: `[VERIFIED STATIC]`.
  - Operasi drag-and-drop file interaktif runtime: `[INFERRED]` / `[NOT TESTED in this session]`.
- **Deskripsi Fungsional:**
  Area penyimpanan sementara (*staging area*) untuk berkas yang diseret (*drag-and-drop*) oleh pengguna dari File Explorer atau browser.
- **Metode Akses:**
  Menjatuhkan berkas ke atas kapsul Dynamic Island atau membuka tab Shelf.
- **Model Input Pengguna `[INFERRED]`:**
  Drop berkas ke island, drag berkas keluar dari shelf ke aplikasi lain, hapus item, dan hapus semua (*clear all*).
- **Mekanisme Internal Terverifikasi `[VERIFIED STATIC]`:**
  - Pendaftaran antarmuka OLE COM `IDropTarget` melalui `RegisterDragDrop` pada jendela `NotchWindow`.
  - Pembacaan format `CF_HDROP` via `DragQueryFileW`.
  - Berkas fisik ditampung di `%LOCALAPPDATA%\Nusa\Shelf\`.
  - Metadata dicatat pada file teks `%LOCALAPPDATA%\Nusa\shelf.txt`.
- **Batasan Teknis:**
  Tidak ada verifikasi integritas berkas jika berkas asli dipindahkan secara eksternal.
- **Relevansi untuk Hime Desktop:**
  - *Relevansi:* **SANGAT TINGGI (FITUR STRATEGIS INTI)**.
  - *Rekomendasi:* Ditransformasikan menjadi **"AI Context File Ingestion"**! Pengguna men-drop berkas dokumen, kode, atau gambar langsung ke island; Hime Desktop meneruskan jalur berkas ini ke Hermes Agent sebagai konteks analisis tugas.

---

### Modul 03: Clipboard History & Manager

- **Klasifikasi Bukti:**
  - Format clipboard privasi & berkas `clipboard.txt`: `[VERIFIED STATIC]`.
  - Penangkapan event `WM_CLIPBOARDUPDATE` runtime: `[INFERRED]` / `[NOT TESTED in this session]`.
- **Deskripsi Fungsional:**
  Memantau clipboard Windows dan menyimpan riwayat cuplikan teks yang disalin (5 item versi Free, hingga 50 versi Pro).
- **Mekanisme Internal Terverifikasi `[VERIFIED STATIC]`:**
  - Impor `AddClipboardFormatListener`, `GetClipboardData`.
  - Format terdaftar khusus privasi: `ExcludeClipboardContentFromMonitorProcessing` dan `CanIncludeInClipboardHistory`.
  - Penyimpanan teks polos ke `%LOCALAPPDATA%\Nusa\clipboard.txt`.
- **Batasan Teknis:**
  Penyimpanan teks mentah tanpa enkripsi berisiko membocorkan data sensitif (password/token).
- **Relevansi untuk Hime Desktop:**
  - *Relevansi:* **Sedang (Perlu Re-engineering)**.
  - *Rekomendasi:* Diperlukan untuk cuplikan konteks teks cepat prompt Hermes, namun wajib menggunakan enkripsi memori/DPAPI sesuai standar keamanan SEC-02.

---

### Modul 04: To-Do List & Task Management

- **Klasifikasi Bukti:**
  - Berkas `tasks.txt` / `todo.txt`, konfigurasi suara, dan API `PlaySoundW`: `[VERIFIED STATIC]`.
  - Manajemen tugas interaktif runtime: `[INFERRED]` / `[NOT TESTED in this session]`.
- **Deskripsi Fungsional:**
  Daftar tugas harian ringkas dengan status centang selesai dan chime pengingat.
- **Mekanisme Internal Terverifikasi `[VERIFIED STATIC]`:**
  - Berkas persisten `%LOCALAPPDATA%\Nusa\tasks.txt`.
  - Pemutaran suara lonceng via `winmm.dll` (`PlaySoundW`) saat tugas dimulai (`todo_start`) atau selesai (`todo_end`).
- **Batasan Teknis:**
  Hanya format teks flat lokal tanpa sinkronisasi cloud.
- **Relevansi untuk Hime Desktop:**
  - *Relevansi:* **Sedang**.
  - *Rekomendasi:* Diadaptasi sebagai tampilan **Agent Action Plan** multi-langkah yang dirumuskan oleh Hermes Agent saat menyelesaikan tugas berurutan.

---

### Modul 05: Water Tracker & Hydration Reminder

- **Klasifikasi Bukti:**
  - Skema berkas `water.txt` (`bottle=`, `goal=`, dll.): `[VERIFIED STATIC]`.
  - Siklus timer notifikasi berkala runtime: `[INFERRED]` / `[NOT TESTED in this session]`.
- **Deskripsi Fungsional:**
  Pelacak asupan cairan harian dengan pop-up berkala dan perhitungan target volume minum.
- **Mekanisme Internal Terverifikasi `[VERIFIED STATIC]`:**
  - Penyimpanan kunci-nilai di `%LOCALAPPDATA%\Nusa\water.txt`.
  - Parameter: `bottle`, `goal`, `start`, `end`, `day`, `taken`, `inb`, `fired`, `hours`.
- **Relevansi untuk Hime Desktop:**
  - *Relevansi:* **Rendah (Widget Sekunder)**.
  - *Rekomendasi:* Bukan bagian dari sistem inti agen AI. Dapat dijadikan contoh modul widget sekunder pada fase perluasan widget ekosistem Hime di masa mendatang.

---

### Modul 06: Pomodoro Focus Timer

- **Klasifikasi Bukti:**
  - Konfigurasi `focus.txt` (`focus`, `short_break`, `long_break`): `[VERIFIED STATIC]`.
  - Siklus hitung mundur & perpindahan sesi runtime: `[INFERRED]` / `[NOT TESTED in this session]`.
- **Deskripsi Fungsional:**
  Timer fokus berbasis teknik Pomodoro (25 menit kerja, 5 menit istirahat).
- **Mekanisme Internal Terverifikasi `[VERIFIED STATIC]`:**
  - Berkas persisten `%LOCALAPPDATA%\Nusa\focus.txt`.
  - Transisi suara via `PlaySoundW`.
- **Relevansi untuk Hime Desktop:**
  - *Relevansi:* **Sedang**.
  - *Rekomendasi:* Terintegrasi dengan mode fokus Hime, di mana agen dapat secara otomatis menunda notifikasi non-kritis selama sesi fokus aktif.

---

### Modul 07: Prayer Times (Jadwal Shalat — Metode Kemenag RI)

- **Klasifikasi Bukti:**
  - Daftar 38 provinsi/kota Indonesia, formula hisab, flag `prayer_pause_media`: `[VERIFIED STATIC]`.
  - Eksekusi pause media GSMTC otomatis saat waktu shalat: `[INFERRED]` / `[NOT TESTED in this session]`.
- **Deskripsi Fungsional:**
  Menghitung jadwal shalat 5 waktu di Indonesia berdasarkan formula Kemenag RI, memberikan notifikasi, dan mengirim sinyal jeda media.
- **Mekanisme Internal Terverifikasi `[VERIFIED STATIC]`:**
  - Basis data koordinat lintang/bujur kota-kota di Indonesia dalam biner statis.
  - Perhitungan matematis astronomis posisi matahari.
  - Integrasi flag jeda media (`prayer_pause_media=1`).
- **Batasan Teknis:**
  Terbatas secara eksklusif untuk wilayah yurisdiksi Indonesia.
- **Relevansi untuk Hime Desktop:**
  - *Relevansi:* **Rendah (Spesifik Regional)**.
  - *Rekomendasi:* **TIDAK DIADOPSI** ke dalam core Hime Desktop. Jika diperlukan, dibangun sebagai plugin komunitas terpisah.

---

### Modul 08: System HUD & Hardware Popups

- **Klasifikasi Bukti:**
  - Flag konfigurasi `hud_volume`, `hud_brightness`, `hud_caps`, dll. & impor Core Audio `IAudioEndpointVolume`: `[VERIFIED STATIC]`.
  - Animasi visual morphing pop-up hardware runtime: `[INFERRED]` / `[NOT TESTED in this session]`.
- **Deskripsi Fungsional:**
  Menampilkan animasi pil responsif saat terjadi perubahan status perangkat keras Windows (volume, kecerahan layar, Caps Lock, status baterai laptop).
- **Mekanisme Internal Terverifikasi `[VERIFIED STATIC]`:**
  - Core Audio API (`IAudioEndpointVolumeCallback`).
  - Power Setting Notification (`GUID_ACDC_POWER_SOURCE`, `GUID_BATTERY_PERCENTAGE_REMAINING`).
  - Pemantauan Caps Lock dan keyboard input language.
- **Relevansi untuk Hime Desktop:**
  - *Relevansi:* **SANGAT TINGGI (POLA INTERAKSI KRITIS)**.
  - *Rekomendasi:* Diadaptasi menjadi **"Agent Status HUD Pill"** dalam **Agent Activity Stream** untuk mengabarkan saat agen sedang mengeksekusi perintah shell, membaca berkas, atau memanggil model inferensi.

---

### Modul 09: Widgets & Glanceable Info

- **Klasifikasi Bukti:**
  - String biner `world_clock`, `moon_phase`, `countdown`, `today`: `[VERIFIED STATIC]`.
  - Rendering kartu widget runtime: `[INFERRED]` / `[NOT TESTED in this session]`.
- **Deskripsi Fungsional:**
  Widget ringkas: World Clock (2 kota dunia), Moon Phase astronomis, Event Countdown, dan Today Summary.
- **Relevansi untuk Hime Desktop:**
  - *Relevansi:* **Sedang**.
  - *Rekomendasi:* Dievolusikan menjadi **"Specialist Agent Quick Cards"** untuk ringkasan rapat, agenda harian, atau status koneksi gateway 9Router.

---

### Modul 10: System Tray & Application Controls

- **Klasifikasi Bukti:**
  - Panggilan `Shell_NotifyIconW`, pesan `TaskbarCreated`, registry Run, kelas `NusaSettings`: `[VERIFIED STATIC]`.
  - Perilaku klik tray menu runtime: `[INFERRED]` / `[NOT TESTED in this session]`.
- **Deskripsi Fungsional:**
  Kontrol latar belakang via baki sistem (System Tray), jendela pengaturan khusus (`NusaSettings` 654×556 px), pemilihan monitor, dan toggle auto-start.
- **Mekanisme Internal Terverifikasi `[VERIFIED STATIC]`:**
  - `Shell_NotifyIconW` dengan struktur `NOTIFYICONDATAW` (`NOTIFYICON_VERSION_4`).
  - Penanganan crash shell via `RegisterWindowMessageW("TaskbarCreated")`.
  - Auto-start via `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`.
- **Relevansi untuk Hime Desktop:**
  - *Relevansi:* **SANGAT TINGGI (KOMPONEN INFRASTRUKTUR STANDAR)**.
  - *Rekomendasi:* Diadopsi penuh menggunakan Tauri 2.0 Tray API di backend Rust Hime.

---

## 3. Comprehensive Feature Summary & Strategic Mapping

| No | Modul Fitur Nusa | Klasifikasi Bukti | Status Penggunaan Nusa | Rencana Adopsi Hime Desktop | Prioritas Implementasi Hime |
|:---:|:---|:---:|:---:|:---|:---:|
| 1 | **Media Player & Lyrics** | `[VERIFIED STATIC]` (API) / `[INFERRED]` (UI) | Fitur Utama | Widget sekunder / plugin opsional | Rendah (Phase 06+) |
| 2 | **File Shelf** | `[VERIFIED STATIC]` (OLE/Folder) / `[INFERRED]` (UI) | Fitur Sekunder | **AI Context Ingestion Shelf** | **KRITIS (Phase 03)** |
| 3 | **Clipboard History** | `[VERIFIED STATIC]` (Format) / `[INFERRED]` (UI) | Fitur Sekunder | AI Prompt Context Snippets (Encrypted) | Sedang (Phase 04) |
| 4 | **To-Do List** | `[VERIFIED STATIC]` (IO/Audio) / `[INFERRED]` (UI) | Fitur Sekunder | Agent Action Plan Execution Steps | Sedang (Phase 04) |
| 5 | **Water Tracker** | `[VERIFIED STATIC]` (Skema IO) / `[INFERRED]` (UI) | Fitur Tambahan | Widget komunitas opsional | Rendah (Phase 07+) |
| 6 | **Pomodoro Timer** | `[VERIFIED STATIC]` (Skema IO) / `[INFERRED]` (UI) | Fitur Tambahan | Mode Fokus Produktivitas Terintegrasi | Sedang (Phase 05) |
| 7 | **Prayer Times (Kemenag)**| `[VERIFIED STATIC]` (Formula/Kota) | Fitur Regional | **TIDAK DIADOPSI** ke Core Hime | Diabaikan / Plugin Luar |
| 8 | **System HUD Popups** | `[VERIFIED STATIC]` (CoreAudio/GUID) | Fitur Utama | **Agent Status HUD Pill** | **KRITIS (Phase 03)** |
| 9 | **Glanceable Widgets** | `[VERIFIED STATIC]` (Strings) / `[INFERRED]` (UI) | Fitur Tambahan | Specialist Agent Quick Cards | Sedang (Phase 05) |
| 10| **System Tray & Lifecycle**| `[VERIFIED STATIC]` (Shell API/Registry) | Komponen Inti | **Tauri 2.0 System Tray & Lifecycle** | **KRITIS (Phase 03)** |
