# 06 — Feature and Internal Mechanism Analysis
## Proyek: Hime Assistant
### Target: Nusa Dynamic Island

---

## 1. Inventarisasi Fitur yang Terkonfirmasi

Berdasarkan analisis statis kode biner, inspeksi skema string konfigurasi, dan pengujian runtime, seluruh fitur Nusa versi 1.2.0.0 berhasil dipetakan secara akurat.

Berikut adalah daftar modul fitur lengkap beserta mekanisme internalnya:

---

## 2. Rincian Modul Fitur

### A. Media Player & Live Lyrics
- **Deskripsi:** Menampilkan musik yang sedang diputar di Windows, kontrol media (Play, Pause, Next, Previous), dan sinkronisasi lirik berjalan secara real-time.
- **Mekanisme Internal:**
  1. Menggunakan WinRT `Windows.Media.Control.GlobalSystemMediaTransportControlsSessionManager` (GSMTC) untuk terhubung ke sesi pemutar media aktif (Spotify, YouTube di browser, Apple Music, VLC).
  2. Mengambil judul lagu, nama artis, dan cover art (`Windows.Storage.Streams.DataReader`).
  3. Mengambil lirik lagu secara online melalui REST API [LRCLIB](https://lrclib.net) (`Windows.Web.Http.HttpClient`):
     ```text
     GET https://lrclib.net/api/search?track_name=<title>&artist_name=<artist>
     ```
  4. Lirik di-render sinkron dengan stempel waktu pemutaran audio.

### B. File Shelf & Clipboard Manager
- **Deskripsi:** Area penampungan sementara (*staging shelf*) untuk file dan riwayat teks salinan clipboard.
- **Mekanisme Internal:**
  - **File Shelf:** Menggunakan OLE COM interface (`IDropTarget`, `DoDragDrop`, `RegisterDragDrop`, `DragQueryFileW`). Pengguna dapat men-drag file apa pun dari File Explorer ke notch. File disimpan secara fisik di folder:
    `%LOCALAPPDATA%\Nusa\Shelf\`
    dan daftarnya dicatat dalam `shelf.txt`. Terdapat opsi pembersihan otomatis saat Nusa restart (`shelf_clear`).
  - **Clipboard Shelf:** Memantau clipboard Windows (`GetClipboardData`). Menyimpan teks yang baru disalin ke dalam `clipboard.txt` (maksimal 5 item pada versi Free).
  - **Perlindungan Privasi:** Menggunakan format terdaftar `ExcludeClipboardContentFromMonitorProcessing` dan `CanIncludeInClipboardHistory` agar teks internal tidak mengotori riwayat clipboard eksternal.

### C. To-Do List & Task Management
- **Deskripsi:** Daftar tugas harian cepat yang dapat diceklis langsung dari Dynamic Island.
- **Penyimpanan:** Disimpan dalam format teks sederhana:
  `%LOCALAPPDATA%\Nusa\tasks.txt` (atau `todo.txt`).
- **Mekanisme:** Mendukung pembuatan tugas (`Add Task`), edit tugas (`Edit Task`), penanda tugas selesai (`Task Done Today`), serta suara pengingat saat waktu tugas dimulai atau berakhir (`todo_start`, `todo_end`).

### D. Water Tracker & Pengingat Hidrasi
- **Deskripsi:** Pelacak konsumsi air harian dan pengingat minum teratur.
- **Penyimpanan:** Disimpan dalam format key-value pada file:
  `%LOCALAPPDATA%\Nusa\water.txt`.
- **Skema Data Terkonfirmasi:**
  ```text
  bottle=<kapasitas_botol_ml>
  goal=<target_harian_ml>
  start=<jam_mulai_aktif>
  end=<jam_selesai_aktif>
  day=<tanggal_hari_ini>
  taken=<jumlah_ml_sudah_diminum>
  inb=<jumlah_isi_ulang_botol>
  fired=<status_notifikasi>
  hours=<rentang_jam>
  ```
- **Aksi Pengguna:** Tombol "Drank" (mencatat konsumsi), "Refilled" (mencatat pengisian ulang botol), dan "Snooze" (menunda pengingat).

### E. Pomodoro Focus Timer
- **Deskripsi:** Timer teknik Pomodoro untuk meningkatkan produktivitas belajar atau kerja.
- **Penyimpanan:** `%LOCALAPPDATA%\Nusa\focus.txt`.
- **Konfigurasi Siklus:**
  - `focus` (Durasi kerja, default 25 menit)
  - `short_break` (Istirahat pendek, default 5 menit)
  - `long_break` (Istirahat panjang setelah 4 siklus)
  - Mengeluarkan suara chime saat transisi status melalui API `PlaySoundW`.

### F. Waktu Shalat (Prayer Times — Metode Kemenag RI)
- **Deskripsi:** Menghitung dan menampilkan jadwal shalat 5 waktu untuk wilayah Indonesia dengan notifikasi dan otomatisasi media.
- **Mekanisme Teknis:**
  - **Basis Wilayah:** Memuat daftar kota dan kabupaten dari 38 provinsi di Indonesia (Banda Aceh, Medan, Padang, Jakarta, Bandung, Surabaya, Makassar, Jayapura, dll.).
  - **Metode Perhitungan:** Formula hisab resmi Kementerian Agama Republik Indonesia (Kemenag RI).
  - **Fitur Otomatisasi:**
    - Notifikasi 10 menit sebelum waktu shalat (`prayer_early`).
    - Peringatan suara azan / chime saat waktu masuk (`prayer_sound`).
    - **Otomatis Pause Media:** Mengirim sinyal jeda media ke Spotify/YouTube saat azan tiba (`prayer_pause_media`)!
    - Koreksi menit manual per waktu shalat (`prayer_adj_subuh`, `prayer_adj_dzuhur`, dll.).

### G. HUD & Popups Indikator Sistem
- **Deskripsi:** Menampilkan animasi pil responsif ketika terjadi perubahan status perangkat keras Windows tanpa memunculkan OSD bawaan Windows yang kaku.
- **Indikator yang Didukung:**
  1. Perubahan Volume suara (`hud_volume`)
  2. Perubahan Brightness layar (`hud_brightness`)
  3. Status tombol Caps Lock (`hud_caps`)
  4. Pergantian Input Bahasa Keyboard (`hud_language`)
  5. Status Baterai Laptop (`popup_battery`): Peringatan baterai lemah (*Low battery*), kritis (*Plug in now*), dan penuh (*You can unplug*).
  6. Status Wi-Fi dan Bluetooth Headset (`popup_wifi`, `popup_bluetooth`).

### H. Widgets & Glanceable Info
- **Deskripsi:** Halaman widget cepat yang menampilkan ringkasan informasi harian:
  - **World Clock:** Menampilkan waktu di 2 kota pilihan dunia (Tokyo, London, New York, Mecca, dll.).
  - **Fase Bulan (Moon Phase):** Perhitungan fase bulan astronomis (New Moon, Waxing Crescent, Full Moon, dll.).
  - **Countdown Acara:** Hitung mundur ke hari besar (Tahun Baru, Idul Fitri, Hari Kemerdekaan, Natal).
  - **Today Page:** Ringkasan jumlah tugas selesai hari ini dan progres minum air.

---

## 3. Matriks Komponen & Potensi Integrasi untuk Hime Assistant

| Fitur Nusa | Relevansi untuk Hime Assistant | Rekomendasi Adaptasi |
| :--- | :--- | :--- |
| **Media & Lyrics** | Rendah (Bukan fokus AI assistant) | Dapat dinonaktifkan atau dijadikan widget sekunder agar area kanvas fokus ke Hermes Agent. |
| **File Shelf** | **SANGAT TINGGI** | **Dapat diubah menjadi File Input untuk Hermes!** Pengguna dapat men-drop file dokumen/kode ke island, lalu Hermes langsung membaca dan menganalisis file tersebut. |
| **Clipboard History** | Sedang | Berguna untuk memilih teks konteks yang akan dikirim ke Hermes. |
| **HUD / Notifications** | **SANGAT TINGGI** | Menjadi pondasi visual untuk **Notification Pill** saat Hermes selesai menjalankan tugas di background. |
| **To-do / Focus** | Sedang | Dapat diintegrasikan dengan to-do generator dari Hermes Agent. |
| **Island Expand Animation**| **KRITIS / INTI** | Mekanisme animasi ekspansi kapsul menjadi canvas percakapan dan status eksekusi Hermes. |

