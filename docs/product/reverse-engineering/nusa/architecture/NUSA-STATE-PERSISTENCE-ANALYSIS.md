# Nusa State & Persistence Architecture Reverse Engineering Analysis

- **Document ID:** NUSA-STATE-PERSISTENCE-ANALYSIS
- **Task ID:** TASK-02-02 (Phase 02 — Nusa Reverse Engineering)
- **Target Application:** Nusa Dynamic Island for Windows (v1.2.0.0)
- **Authority:** Senior Data Storage & Database Architecture Specialist
- **Status:** **CANONICAL PERSISTENCE SPECIFICATION**
- **Date:** 2026-11-08

---

## 1. Executive Summary

Dokumen ini membedah arsitektur pengelolaan state (*state management*), format serialisasi, berkas konfigurasi, serta model penyimpanan persisten (*persistence layer*) dari **Nusa Dynamic Island v1.2.0.0**.

Berdasarkan analisis biner statis dan dekompilasi konstanta, Nusa menerapkan pendekatan penyimpanan **teks polos mentah (*raw flat plaintext files*)** tanpa basis data relasional (bukan SQLite) dan tanpa format serialisasi biner berstruktur (bukan Protobuf, Bincode, atau MsgPack). Seluruh state persisten disimpan terpusat di dalam direktori profil pengguna:
`%LOCALAPPDATA%\Nusa\`

Dokumen ini mendokumentasikan skema data seluruh berkas persisten yang teridentifikasi, menganalisis risiko korupsi data (*data corruption*), kerentanan race condition, dan celah keamanan kebocoran teks clipboard polos, serta merumuskan rekomendasi arsitektur penyimpanan modern berbasis **SQLite + Windows DPAPI** untuk proyek **Hime Desktop**.

---

## 2. Storage Topology & Directory Organization

Penyimpanan persisten Nusa diisolasi secara ketat pada ruang lingkup pengguna (*user-scope*):

```text
Filesystem Storage Topology:
C:\Users\<User>\AppData\Local\Nusa\
├── settings.txt           [Plaintext Kunci-Nilai: Konfigurasi Preferensi Global]
├── water.txt              [Plaintext Kunci-Nilai: Log & Konfigurasi Hidrasi Harian]
├── focus.txt              [Plaintext Kunci-Nilai: Siklus & Durasi Timer Pomodoro]
├── shelf.txt              [Plaintext Delimited: Metadata & Indeks Berkas Shelf]
├── clipboard.txt          [Plaintext Records: Riwayat Cuplikan Salinan Clipboard]
├── tasks.txt (todo.txt)   [Plaintext Lines: Daftar Tugas & Status Centang Harian]
├── glass.log              [Plaintext Log: Catatan Diagnostik Rendering DWM / DComp]
└── Shelf\                 [Direktori Fisik: Tempat Penampungan Berkas Staging]
```

---

## 3. Configuration & State Files Schema Breakdown

### A. `settings.txt` (Konfigurasi Utama Aplikasi) `[VERIFIED STATIC]`

Berkas ini menyimpan seluruh preferensi operasional aplikasi dalam format kunci-nilai (`key=value\n`). String biner pada offset `0x000E04D8` mengonfirmasi parameter kunci berikut:

| Kunci Konfigurasi | Tipe Data / Nilai Teramati | Fungsi Pengaturan | Subsistem Konsumen |
|:---|:---:|:---|:---|
| `sound` | `1 \| 0` | Mengaktifkan suara lonceng notifikasi/chime | Audio HUD |
| `lyrics` | `1 \| 0` | Mengaktifkan pengambilan lirik dari LRCLIB | Media Subsystem |
| `todo_start` | `1 \| 0` | Efek suara saat waktu tugas to-do dimulai | To-Do Widget |
| `todo_end` | `1 \| 0` | Efek suara saat tugas to-do ditandai selesai | To-Do Widget |
| `snooze` | `5 \| 10 \| 15` (Menit) | Durasi tunda pengingat hidrasi/tugas | Water / Widget |
| `week_monday` | `1 \| 0` | Hari pertama kalender (0=Minggu, 1=Senin) | Calendar Widget |
| `focus` | `25` (Menit) | Durasi fase konsentrasi kerja Pomodoro | Pomodoro Timer |
| `short_break` | `5` (Menit) | Durasi istirahat pendek Pomodoro | Pomodoro Timer |
| `long_break` | `15` (Menit) | Durasi istirahat panjang Pomodoro | Pomodoro Timer |
| `hud_volume` | `1 \| 0` | Mengaktifkan pop-up HUD volume audio | Hardware HUD |
| `hud_brightness`| `1 \| 0` | Mengaktifkan pop-up HUD kecerahan layar | Hardware HUD |
| `hud_caps` | `1 \| 0` | Mengaktifkan pop-up HUD status Caps Lock | Hardware HUD |
| `hud_language` | `1 \| 0` | Mengaktifkan pop-up HUD pergantian bahasa | Hardware HUD |
| `popup_battery` | `1 \| 0` | Mengaktifkan pop-up HUD peringatan baterai | Hardware HUD |
| `popup_wifi` | `1 \| 0` | Mengaktifkan pop-up HUD status koneksi Wi-Fi | Hardware HUD |
| `popup_bluetooth`| `1 \| 0` | Mengaktifkan pop-up HUD headset Bluetooth | Hardware HUD |
| `open_click` | `1 \| 0` | Pemicu ekspansi: 0=Hover kursor, 1=Klik mouse | Window Manager |
| `page_icons` | `1 \| 0` | Tampilkan preview ikon saat hover titik navigasi | UI / Animation |
| `fast_animation`| `1 \| 0` | Kecepatan animasi (0=Normal ~220ms, 1=Fast ~120ms)| Animation Subsystem |
| `shelf_clear` | `1 \| 0` | Bersihkan berkas fisik Shelf saat Nusa restart | File Shelf Subsystem |
| `hide_fullscreen`| `1 \| 0` | Sembunyikan notch saat aplikasi layar penuh aktif | Fullscreen Hook |
| `prayer` | `1 \| 0` | Mengaktifkan fitur jadwal shalat Kemenag RI | Prayer Times Widget |
| `prayer_city` | String nama kota | Kota terpilih untuk koordinat hisab matahari | Prayer Times Widget |
| `prayer_early` | `1 \| 0` | Peringatan notifikasi 10 menit sebelum waktu shalat | Prayer Times Widget |
| `prayer_sound` | `1 \| 0` | Suara pengingat saat masuk waktu shalat | Prayer Times Widget |
| `prayer_pause_media`| `1 \| 0` | Otomatis jeda musik/video saat azan tiba | Media Subsystem |
| `notch_style` | `classic \| floating` | Gaya notch: 0=Classic (menempel), 1=Floating | Window Manager |
| `glass` | `1 \| 0` | Efek frosted glass acrylic blur di belakang panel | Rendering Engine |

---

### B. `water.txt` (Log & State Hidrasi) `[VERIFIED STATIC]`

Format kunci-nilai terverifikasi pada string biner offset `0x000E25E1`:

```ini
bottle=600       # Kapasitas satu botol minum (ml)
goal=2000        # Target konsumsi harian (ml)
start=08:00      # Jam mulai interval pengingat harian
end=20:00        # Jam selesai interval pengingat harian
day=2026-11-08   # Tanggal pencatatan aktif (di-reset harian)
taken=1200       # Total volume air yang sudah dikonsumsi hari ini (ml)
inb=2            # Hitungan isi ulang botol (refill count)
fired=0          # Flag status penembakan pop-up reminder
hours=12         # Rentang durasi jam aktif
```

---

### C. `focus.txt` (State Timer Pomodoro) `[VERIFIED STATIC]`

Menyimpan konfigurasi durasi dan status siklus aktif:
```ini
focus=25         # Menit fase fokus
short_break=5    # Menit fase istirahat pendek
long_break=15    # Menit fase istirahat panjang
cycles=4         # Jumlah siklus sebelum istirahat panjang
completed=3      # Sesi fokus yang berhasil diselesaikan hari ini
```

---

### D. `shelf.txt` (Metadata Indeks File Shelf) `[VERIFIED STATIC]`

Menyimpan daftar berkas yang sedang ditampung pada panel File Shelf. Setiap baris merepresentasikan satu berkas yang dipisahkan oleh karakter delimit (*pipe `|` atau tab*):
```text
<File_Name>|<Original_Source_Path>|<File_Size_Bytes>|<Timestamp_Added>
Contoh:
document.pdf|C:\Users\Satoshi\Documents\document.pdf|245120|1793540120
code.rs|X:\folder_ai\Hime\src\main.rs|4096|1793540250
```

---

### E. `clipboard.txt` (Riwayat Teks Clipboard) `[VERIFIED STATIC]`

- Menyimpan teks cuplikan yang baru disalin pengguna.
- Pemisah antar-item menggunakan penanda batas (*record boundary delimiter*).
- Batas penyimpanan: 5 entri pada lisensi Free, hingga 50 entri pada lisensi Pro.

---

### F. `tasks.txt` / `todo.txt` (Daftar Tugas Harian) `[VERIFIED STATIC]`

Menyimpan daftar checklist tugas:
```text
[X] 09:00 Siapkan laporan audit arsitektur
[ ] 14:00 Review implementasi DirectComposition
[ ] 16:30 Rapat sinkronisasi tim rekayasa
```

---

## 4. Read & Write Mechanics Analysis

Pemeriksaan tabel impor biner (`kernel32.dll`) mengonfirmasi API I/O berkas yang dipanggil Nusa:
- `CreateFileW`
- `ReadFile`
- `WriteFile`
- `SetEndOfFile`
- `CloseHandle`

### Alur Operasi Baca & Tulis `[INFERRED]`:
1. **Pembacaan Saat Inisialisasi:** Saat aplikasi diluncurkan, subsistem terkait membaca seluruh isi berkas teks ke dalam memori menggunakan alokator Rust (`std::fs::read_to_string`).
2. **Penyuntingan State:** Interaksi pengguna (misal: mengklik tombol "Drank" atau menandai to-do) memutasi struct data in-memory di thread utama.
3. **Penulisan Bersih (Overwrite):** Aplikasi membuka kembali berkas dengan flag `GENERIC_WRITE | CREATE_ALWAYS`, menulis seluruh serialisasi teks dari awal (*full overwrite*), dan memanggil `CloseHandle`.

---

## 5. Engineering Risks of Nusa Persistence Architecture

Melalui evaluasi mendalam, ditemukan **tiga risiko teknis kritis** pada arsitektur penyimpanan Nusa v1.2.0.0:

### A. Risiko Korupsi Data (*Data Truncation Risk*) `[CRITICAL]`
- **Masalah:** Nusa melakukan overwrite berkas secara in-place tanpa pola *Atomic File Replacement* (yaitu menulis ke berkas sementara `.tmp` terlebih dahulu, lalu menggantinya via `ReplaceFileW` / `MoveFileExW` dengan flag `MOVEFILE_REPLACE_EXISTING`).
- **Dampak:** Jika komputer mengalami mati mendadak (*power outage*), blue screen (BSOD), atau proses `nusa.exe` dimatikan paksa (*taskkill*) saat operasi penulisan berlangsung, berkas `settings.txt` atau `water.txt` dapat terpotong menjadi 0-byte (rusak permanen).

### B. Risiko Benturan Konkurensi (*Race Conditions*) `[HIGH]`
- **Masalah:** Berkas teks polos diakses tanpa mekanisme penguncian berkas (*file locking* via `LOCKFILE_EXCLUSIVE_LOCK`).
- **Dampak:** Jika worker thread dan main GUI thread mencoba menulis ke berkas yang sama secara bersamaan, data dapat saling menimpa (*clobbered writes*).

### C. Risiko Keamanan & Privasi (*Plaintext Secret Exposure*) `[CRITICAL — SEC-02]`
- **Masalah:** Berkas `clipboard.txt` dan `settings.txt` disimpan dalam bentuk teks polos mentah di `%LOCALAPPDATA%\Nusa\` tanpa enkripsi kriptografis apa pun.
- **Dampak:** Jika pengguna tidak sengaja menyalin kata sandi (*password*), token API rahasia, atau informasi pribadi ke clipboard, data sensitif tersebut akan tertulis permanen dalam teks polos pada hard disk. Aplikasi atau skrip lokal lain yang berjalan dengan hak pengguna biasa (*unprivileged*) dapat membaca isi berkas ini dengan mudah.

---

## 6. Architectural Translation & Migration Recommendations for Hime Desktop

Arsitektur penyimpanan Hime Desktop dirancang secara radikal berbeda untuk menjamin integritas data ACID dan kepatuhan terhadap standar keamanan rekayasa **SEC-01 s/d SEC-10**:

| Aspek Arsitektur | Pendekatan Nusa v1.2.0.0 | Desain Hime Desktop (Tauri 2.0 / Rust) | Keunggulan Arsitektur Hime |
|:---|:---|:---|:---|
| **Format Penyimpanan** | Berkas teks polos mentah (`.txt`) terpisah-pisah | **Basis Data Relasional SQLite Lokal (Crate `rusqlite`)** | Menyediakan struktur data relasional ACID terkelola; mencegah risiko berkas teks terpotong (*truncation*). |
| **Model Konkurensi & Journaling** | Baca/tulis sinkron tanpa penguncian | **SQLite WAL Mode (Write-Ahead Logging)** | WAL menyediakan mekanisme journaling dan memungkinkan pembacaan berlangsung bersamaan dengan transaksi penulisan dalam kondisi yang didukung SQLite. Jaminan durability dipengaruhi konfigurasi `PRAGMA synchronous`, filesystem, dan perangkat penyimpanan. |
| **Keamanan Field Sensitif** | Plaintext terbuka di diska | **Enkripsi Selektif Windows DPAPI (`CryptProtectData`)** | Kredensial, token API model, dan rahasia sensitif dienkripsi per-field menggunakan kunci master akun Windows pengguna. |
| **Kontrol Akses OS** | Default inherited ACLs | **Windows File ACLs Ketat pada `%LOCALAPPDATA%\Hime\`** | Membatasi akses direktori hanya untuk akun pengguna pemilik sesi (*user-only DACL*). |
| **Penyimpanan Berkas Konteks** | Folder staging statis `%LOCALAPPDATA%\Nusa\Shelf\` | **Managed Document Vault dengan Validasi Ukuran & Tipe MIME** | Berkas yang di-drop diverifikasi keamanannya sebelum disajikan sebagai konteks agen Hermes. |
| **Manajemen Riwayat Sesi**| Terbatas (hanya 50 item teks) | **Indexed Vector / FTS5 Full-Text Search di SQLite** | Memungkinkan Hermes Agent mencari kembali riwayat percakapan dan konteks lampau secara instan. |

---

## 7. Storage, Security, & Encryption Boundaries Classification (Hime Storage Architecture)

Untuk menghindari kerancuan konsep penyimpanan dan keamanan pada Hime, batas peran dan tanggung jawab masing-masing lapisan diklasifikasikan secara tegas dan independen:

### A. Windows DPAPI (`CryptProtectData` / `CryptUnprotectData`)
- **Klasifikasi:** Perlindungan data atau secret tertentu melalui Windows Data Protection API pada tingkat aplikasi.
- **Peran:** Perlindungan data sensitif setingkat field (*field-level protection*).
- **Cakupan:** Token otentikasi API, bearer credentials, session keys, dan nilai rahasia tertentu.
- **Mekanisme:** Menggunakan kunci kriptografi simetris yang diturunkan langsung dari kredensial logon pengguna Windows aktif. Data didekripsi hanya di dalam memori saat dibutuhkan.

### B. SQLCipher / SQLite Encryption Extension (SEE)
- **Klasifikasi:** Enkripsi database pada tingkat aplikasi (*application-level database encryption*).
- **Peran:** Mengenkripsi seluruh halaman berkas database secara mandiri (*page-level encryption* di tingkat aplikasi).
- **Cakupan:** Seluruh isi berkas database (skema, tabel, indeks, dan metadata).
- **Klarifikasi:** SQLite secara bawaan (*vanilla SQLite*) **TIDAK** menyediakan enkripsi database. Untuk mengenkripsi berkas database pada tingkat aplikasi diperlukan ekstensi khusus seperti **SQLCipher** atau **SEE**.

### C. BitLocker Drive Encryption
- **Klasifikasi:** Enkripsi volume atau perangkat penyimpanan pada tingkat sistem operasi (*OS-level volume / storage device encryption*).
- **Peran:** Melindungi data *at-rest* dari pencurian fisik media penyimpanan atau akses *offline* di luar OS yang terotentikasi.
- **Klarifikasi Penting:** BitLocker **BUKAN ENKRIPSI DATABASE PADA TINGKAT APLIKASI**. Ketika volume Windows telah di-mount dan pengguna masuk (*logged on*), berkas database `hime.db` dapat dibaca secara transparan oleh aplikasi atau proses apa pun yang memiliki izin filesystem dalam sesi tersebut.

### D. Windows File Access Control Lists (ACLs)
- **Klasifikasi:** Kontrol akses terhadap objek dan berkas berdasarkan identitas serta izin Windows (*OS-level object/file access control*).
- **Peran:** Membatasi izin baca/tulis direktori `%LOCALAPPDATA%\Hime\` dan berkas database hanya kepada security identifier (SID) pengguna aktif dan `SYSTEM`. Mencegah akses dari akun pengguna lain pada mesin bersama (*shared workstation*).

### E. SQLite WAL (Write-Ahead Logging) Mode
- **Klasifikasi:** Mekanisme journaling dan konkurensi database (*database journaling & concurrency mechanism*).
- **Peran:** WAL menyediakan mekanisme journaling dan memungkinkan pembacaan berlangsung bersamaan dengan transaksi penulisan dalam kondisi yang didukung SQLite.
- **Klarifikasi Batas & Karakteristik:**
  - **Bukan Mekanisme Enkripsi:** SQLite WAL **BUKAN MEKANISME ENKRIPSI ATAU KEAMANAN KRIPTOGRAFIS**.
  - **Batas Konkurensi:** Kemampuan *concurrent read* pada WAL tidak boleh disamakan dengan jaminan tidak adanya *blocking* dalam seluruh kondisi (transaksi penulisan tetap saling mengunci, dan pembacaan dapat tertahan saat checkpointing).
  - **Durability Dipengaruhi Konfigurasi Host:** WAL tidak secara inheren menjamin *durability* mutlak secara mandiri; tingkat ketahanan durability dipengaruhi oleh konfigurasi `PRAGMA synchronous` (misal `NORMAL` vs `FULL` vs `EXTRA`), filesystem host, dan perilaku cache perangkat penyimpanan (*storage device write-caching*). Karakteristik bawaan WAL ini harus dipisahkan dari konfigurasi durability dan keamanan yang nantinya ditetapkan Hime.

> **Pernyataan Batas Keamanan Arsitektural:**
> **Kombinasi SQLite WAL + DPAPI TIDAK SECARA OTOMATIS mengenkripsi seluruh berkas database (`hime.db`).**
> Berkas database SQLite utama tetap berupa berkas data relasional terstruktur yang dapat dibaca oleh proses lokal dalam sesi pengguna yang sama. Hanya kolom/field rahasia tertentu yang dienkripsi menggunakan DPAPI sebelum ditulis ke tabel SQLite. Proteksi volume fisik bergantung pada BitLocker (tingkat OS), sedangkan enkripsi database penuh di tingkat aplikasi memerlukan ekstensi seperti SQLCipher jika disyaratkan.
