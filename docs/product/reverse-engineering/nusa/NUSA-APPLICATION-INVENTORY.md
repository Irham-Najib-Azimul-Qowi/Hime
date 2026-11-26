# Nusa Application Inventory & Binary Metadata

- **Document ID:** NUSA-APPLICATION-INVENTORY
- **Task ID:** TASK-02-01 (Phase 02 — Nusa Reverse Engineering)
- **Target Application:** Nusa Dynamic Island for Windows
- **Version:** 1.2.0.0
- **Authority:** Senior Reverse Engineering Specialist & Windows Systems Analyst
- **Status:** `[VERIFIED STATIC]` **CANONICAL INVENTORY**
- **Date:** 2026-11-08

---

## 1. Executive Summary

Dokumen ini menyajikan inventarisasi lengkap, faktual, dan berbasis bukti (*evidence-based*) terhadap paket aplikasi dan biner executable **Nusa Dynamic Island v1.2.0.0**. Seluruh data diperoleh melalui inspeksi Portable Executable (PE32+), dekompilasi setup script Inno Setup (`install_script.iss`), analisis bagian biner (`.text`, `.rdata`, `.rsrc`), serta verifikasi jalur penyimpanan lokal pada sistem host Windows 11.

Berdasarkan analisis biner statis dan dekompilasi metadata, Nusa terkonfirmasi merupakan **aplikasi desktop native murni (single-binary standalone)** yang ditulis dalam bahasa pemrograman **Rust** dan dikompilasi ke dalam arsitektur x86-64 tanpa runtime web (Bukan Electron/Tauri) dan tanpa runtime framework .NET/WPF.

---

## 2. File Metadata & Executable Identity

### A. Target File Artifacts

| Atribut File | Installer Executable (`NusaSetup.exe`) | Main Binary Executable (`nusa.exe`) | Packaging Archive (`NusaSetup.zip`) | Klasifikasi Bukti |
|:---|:---|:---|:---|:---:|
| **Lokasi Relatif** | `RE/Nusa/00-source-materials/NusaSetup.exe` | `RE/Nusa/00-source-materials/extracted/{app}/nusa.exe` | `RE/Nusa/00-source-materials/NusaSetup.zip` | `[VERIFIED STATIC]` |
| **Ukuran File** | 2,945,252 bytes (~2.81 MB / 2,877 KiB) | 1,265,664 bytes (~1.20 MB / 1,236 KiB) | 2,430,268 bytes (~2.32 MB) | `[VERIFIED STATIC]` |
| **SHA-256 Hash** | `BC4133305B28B929ECA42CC32833DC4FD34002CE35676D27761942419F4D4B02` | `28D8A7CE05705667366FFC23B94D024B3EB4F3B3F6335EC1C63A3C0898495EC2` | `3590642776D0184423C4D2BDDF92D48D6FF03C9439406BA41FEC2DFD933CF115` | `[VERIFIED STATIC]` |
| **MD5 Hash** | `799D79E62DF5A77B13BC50BCE2A418E0` | `8220A6158CF8F53B470295D31D56F523` | `B777098FFDF3921EEEC6113A7F187CE1` | `[VERIFIED STATIC]` |
| **Format File** | Inno Setup Self-Extracting Stub (PE32) | Native Windows Executable (PE32+) | Standard ZIP Archive (Deflate) | `[VERIFIED STATIC]` |
| **Timestamp File** | 2026-11-01 20:29:00 WIB | 2026-11-01 20:25:32 WIB | 2026-11-01 20:30:14 WIB | `[VERIFIED STATIC]` |

### B. Portable Executable (PE) Version Information

Ekstraksi resource `RT_VERSION` pada `nusa.exe` dan `NusaSetup.exe`:

```text
ProductName         : Nusa
ProductVersion      : 1.2.0.0
ProductVersionRaw   : 1.2.0.0
FileDescription     : Nusa (Installer: Nusa Setup)
FileVersion         : 1.2.0.0
FileVersionRaw      : 1.2.0.0
CompanyName         : Rama
LegalCopyright      : Copyright © 2026 Rama
OriginalFilename    : nusa.exe
InternalName        : nusa
Comments            : A living notch for Windows (Installer: This installation was built with Inno Setup)
Subsystem           : Windows GUI (Subsystem 2)
Machine Architecture: AMD64 / x86-64 (0x8664)
Characteristics     : 0x0022 (EXECUTABLE_IMAGE | LARGE_ADDRESS_AWARE)
Entry Point RVA     : 0x000D09D0
ImageBase           : 0x0000000140000000
Linker Version      : 14.38 (MSVC 2022 Linker / LLVM backend)
Digital Signature   : Unsigned / NotSigned (Tidak bertanda tangan Authenticode)
```

---

## 3. Directory Structure & File Organization

Nusa v1.2.0.0 memisahkan secara ketat direktori biner aplikasi (read-only) dengan direktori state pengguna (writable).

```text
Windows Filesystem Organization:
├── %LOCALAPPDATA%\Programs\Nusa\               <-- [Direktori Biner Aplikasi (Instalasi Standar)]
│   ├── nusa.exe                                <-- Biner executable utama (1,265,664 bytes)
│   └── unins000.exe                            <-- Uninstaller Inno Setup (~1.3 MB saat terpasang)
│
└── %LOCALAPPDATA%\Nusa\                       <-- [Direktori Data Pengguna & Konfigurasi Persisten]
    ├── settings.txt                            <-- Berkas konfigurasi utama (Plaintext Key-Value)
    ├── shelf.txt                               <-- Metadata item file/link pada panel Shelf
    ├── clipboard.txt                           <-- Riwayat clipboard tersimpan (5 item Free, 50 Pro)
    ├── focus.txt                               <-- Konfigurasi siklus timer Pomodoro
    ├── water.txt                               <-- Konfigurasi dan log konsumsi air harian
    ├── tasks.txt (atau todo.txt)               <-- Daftar tugas harian To-Do list
    ├── glass.log                               <-- Log diagnostik rendering DWM/DirectComposition
    └── Shelf\                                  <-- Folder staging penyimpanan berkas fisik shelf
```

- **Verifikasi Host Windows 11:** Folder `C:\Users\Satoshi\AppData\Local\Nusa\Shelf\` telah terverifikasi keberadaannya secara statis di diska mesin host (`[VERIFIED STATIC]`), membuktikan bahwa aplikasi dirancang menyimpan data secara terpusat pada `%LOCALAPPDATA%\Nusa\`.

---

## 4. Configuration & User Data Files Schema

Seluruh konfigurasi dan data pengguna disimpan dalam format teks polos (*plaintext format*) berbasis baris kunci-nilai (*key-value*) atau serialisasi delimit:

### A. `settings.txt` (Konfigurasi Utama Aplikasi)
Daftar flag pengaturan terkonfirmasi dari string biner offset `0x000E04D8`:
```text
sound=1|0                     # Efek suara notifikasi/chime
lyrics=1|0                    # Pengambilan dan penayangan lirik LRCLIB
todo_start=1|0                # Suara saat tugas to-do dimulai
todo_end=1|0                  # Suara saat tugas to-do selesai
snooze=5|10|15                # Durasi tunda pengingat air/to-do (menit)
week_monday=1|0               # Awal minggu kalender (0=Minggu, 1=Senin)
focus=25                      # Durasi Pomodoro focus (menit)
short_break=5                 # Durasi istirahat pendek (menit)
long_break=15                 # Durasi istirahat panjang (menit)
hud_volume=1|0                # OSD popup perubahan volume
hud_brightness=1|0            # OSD popup perubahan kecerahan layar
hud_caps=1|0                  # OSD popup tombol Caps Lock
hud_language=1|0              # OSD popup pergantian bahasa input
popup_battery=1|0             # Popup peringatan baterai laptop
popup_wifi=1|0                # Popup status koneksi Wi-Fi
popup_bluetooth=1|0           # Popup status koneksi headset Bluetooth
open_click=1|0                # Buka island via Klik (1) atau Hover (0)
page_icons=1|0                # Tampilkan ikon halaman saat hover dot pagination
fast_animation=1|0            # Kecepatan animasi (0=Normal, 1=Fast)
shelf_clear=1|0               # Bersihkan isi shelf saat Nusa dimulai ulang
calendar_week=1|0             # Tampilan kalender mode mingguan
hide_fullscreen=1|0           # Sembunyikan notch saat aplikasi layar penuh aktif
prayer=1|0                    # Aktifkan fitur jadwal shalat
prayer_city=<Nama_Kota>       # Kota terpilih untuk kalkulasi hisab
prayer_reminder=1|0           # Notifikasi waktu shalat
prayer_early=1|0              # Peringatan 10 menit sebelum waktu shalat
prayer_sound=1|0              # Suara pengingat saat adzan
prayer_pause_media=1|0        # Jeda musik/video otomatis saat masuk waktu shalat
notch_style=classic|floating  # 0=Joined to top edge, 1=Floats below top edge
glass=1|0                     # Efek kaca buram (acrylic) di belakang notch
status_ring=1|0               # Ring status di sebelah jam
```

### B. `water.txt` (Log & Konfigurasi Hidrasi)
Format string terverifikasi pada offset `0x000E25E1`:
```ini
bottle=600                    # Kapasitas botol air (ml)
goal=2000                     # Target konsumsi harian (ml)
start=08:00                   # Jam mulai jam aktif
end=20:00                     # Jam selesai jam aktif
day=2026-11-08                # Tanggal tracking hari ini
taken=1200                    # Total volume air yang sudah diminum (ml)
inb=2                         # Jumlah isi ulang botol (refill count)
fired=0                       # Status pemicuan notifikasi
hours=12                      # Rentang jam aktif
```

---

## 5. Visual Assets & Embedded Resources

Pemeriksaan bagian `.rsrc` pada biner `nusa.exe` dan `NusaSetup.exe`:

1. **Embedded Icons (Ikon Aplikasi):**
   - Resource Type: `RT_ICON` (Type 3) dan `RT_GROUP_ICON` (Type 14).
   - Resolusi Ikon Tersemat: `256x256` (PNG compressed 32-bit RGBA), `48x48`, `32x32`, `16x16` (ICO uncompressed).
   - Visual Ikon: Simbol kapsul Dynamic Island minimalis berwarna hitam pekat dengan aksen gradasi halus.
2. **Installer Graphics (Inno Setup):**
   - `WizardImage0.bmp`: Banner vertikal kiri wizard (164x314 px, 24-bit BMP).
   - `WizardSmallImage0.bmp`: Ikon pojok kanan atas wizard (55x55 px, 24-bit BMP).
3. **Rendering Aset Vektor Internal:**
   - Nusa **tidak menyimpan berkas SVG atau PNG eksternal** untuk elemen UI di dalam direktori instalasi.
   - Seluruh ikon UI (ikon play, pause, next, volume, water bottle, clock, checkmark, prayer arch, close) digambar secara langsung melalui kode geometri vektor Direct2D (`ID2D1PathGeometry`).

---

## 6. Binary Dependencies & Legitimate Windows APIs

Analisis Import Address Table (IAT) membuktikan bahwa `nusa.exe` beroperasi dengan dependensi sistem Windows native murni:

| Dynamic Link Library (DLL) | Fungsi / API Diimpor Utama | Tujuan Fungsional dalam Nusa |
|:---|:---|:---|
| **`dcomp.dll`** | `DCompositionCreateDevice` | Composition visual tree hardware-accelerated, per-pixel transparency, integrasi DWM. |
| **`d2d1.dll`** | `D2D1CreateFactory`, `D2D1MakeRotateMatrix` | Rendering bentuk kapsul (squircle), ikon vektor, progress bar, audio visualizer. |
| **`d3d11.dll`** | `D3D11CreateDevice` | Perangkat grafis DirectX 11 sebagai akselerator backend Direct2D. |
| **`dwrite.dll`** | `DWriteCreateFactory` | Tipografi Segoe UI, layout teks, rendering lirik lagu, kalkulasi metrik font. |
| **`dwmapi.dll`** | `DwmSetWindowAttribute`, `DwmExtendFrameIntoClientArea` | Efek kaca buram (*acrylic glass blur*), drop shadow, mitigasi border DWM. |
| **`user32.dll`** | `CreateWindowExW`, `SetWindowLongW`, `Shell_NotifyIconW`, `SetWinEventHook` | Window management, extended styles, taskbar tray icon, fullscreen hook. |
| **`kernel32.dll`** | `CreateMutexW`, `CreateFileW`, `GetDpiForMonitor` | Mutex single instance, I/O berkas plaintext, DPI awareness query. |
| **`shell32.dll`** | `SHQueryUserNotificationState`, `ShellExecuteW` | Deteksi aplikasi fullscreen/presentasi, pembukaan link browser. |
| **`winmm.dll`** | `PlaySoundW` | Pemutaran suara lonceng (*chimes*) notifikasi built-in. |
| **WinRT APIs** | `GSMTC`, `UserNotificationListener`, `HttpClient` | Kontrol media Spotify/YouTube, listener notifikasi, pengunduh lirik LRCLIB. |

---

## 7. Installation & Uninstallation Mechanisms

Berdasarkan naskah Inno Setup resmi (`install_script.iss`):

1. **Mekanisme Pemasangan (Setup Flow):**
   - **Tingkat Hak Akses:** `PrivilegesRequired=lowest` (Tanpa permintaan elevasi UAC administrator).
   - **Lokasi Sasaran:** Makro `{autopf}\Nusa` mengevaluasi ke `%LOCALAPPDATA%\Programs\Nusa`.
   - **Pendaftaran Auto-Start:** Menulis entri string ke registry pengguna:
     `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` $\rightarrow$ `"Nusa" = """{app}\nusa.exe"""`.
   - **Pencegahan Multi-Instans:** Memeriksa mutex sistem named `NusaNotch.SingleInstance`.
2. **Mekanisme Pencopotan (Uninstallation Flow):**
   - Menjalankan `unins000.exe` yang menghapus biner di `%LOCALAPPDATA%\Programs\Nusa` dan entri registry Run.
   - **Dialog Retensi Data Pengguna:** Menampilkan konfirmasi interaktif:
     `"Also remove your Nusa data (to-do list, water log, shelf, settings)?"`
   - Jika pengguna memilih "No", seluruh data di `%LOCALAPPDATA%\Nusa\` dipertahankan.

---

## 8. Evidence Classification & Provenance Matrix

Klasifikasi bukti pada inventarisasi ini mengacu pada skema tata kelola bukti:
- `VERIFIED STATIC`: Dibuktikan langsung melalui inspeksi berkas executable, header PE, metadata, skema konfigurasi, tabel impor (IAT), atau artefak statis pada diska.
- `VERIFIED RUNTIME`: Dibuktikan melalui pengujian eksekusi proses aktif pada lingkungan sistem operasi yang terpantau.
- `INFERRED`: Kesimpulan analitis logis berdasarkan bukti tidak langsung.
- `UNKNOWN`: Informasi yang belum dapat dipastikan dari bukti yang tersedia.
- `NOT TESTED`: Perilaku yang belum diuji secara empiris dalam sesi audit terkontrol.

| Parameter Analisis | Nilai Teramati | Klasifikasi Bukti | Sumber Bukti Primer |
|:---|:---|:---:|:---|
| **Bahasa Pemrograman** | Rust (Native MSVC/LLVM x64) | `[VERIFIED STATIC]` | String `rustc/48a229...`, `library\std\sys\pal\windows`. |
| **Versi Aplikasi** | 1.2.0.0 | `[VERIFIED STATIC]` | Header PE `RT_VERSION` dan `User-Agent: Nusa/1.2.0`. |
| **Ukuran Biner Utama** | 1,265,664 bytes | `[VERIFIED STATIC]` | Pemeriksaan ukuran berkas fisik pada diska. |
| **Mesin Grafis** | DirectComposition + Direct2D | `[VERIFIED STATIC]` | Tabel IAT `dcomp.dll`, `d2d1.dll`, `dwrite.dll`. |
| **Lokasi Instalasi** | `%LOCALAPPDATA%\Programs\Nusa\` | `[VERIFIED STATIC]` | Direktif Inno Setup `{autopf}\Nusa` pada `install_script.iss`. |
| **Lokasi Data Pengguna**| `%LOCALAPPDATA%\Nusa\` | `[VERIFIED STATIC]` | Jejak folder fisik di host `C:\Users\Satoshi\AppData\Local\Nusa`. |
| **Penyedia Lirik Musik**| LRCLIB API (`https://lrclib.net`) | `[VERIFIED STATIC]` | URL endpoint pada biner string offset `0x000E1228`. |
| **Mekanisme Lisensi** | Freeware Proprietary (Free vs Pro) | `[VERIFIED STATIC]` | String `Nusa Pro`, `Free: 5 items`, `Copyright © 2026 Rama`. |
