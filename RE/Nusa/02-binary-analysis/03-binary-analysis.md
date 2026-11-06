# 03 — Application Binary Analysis
## Proyek: Hime Assistant
### Target: `nusa.exe` (1,265,664 byte)

---

## 1. Ringkasan Eksekutif Analisis Biner

Analisis terhadap biner utama `nusa.exe` menghasilkan kesimpulan yang sangat tegas dan definitif:

> **Aplikasi Nusa Dynamic Island BUKAN aplikasi berbasis web (Electron/Tauri), BUKAN aplikasi .NET/WPF/WinUI, BUKAN Flutter, dan BUKAN Qt. Nusa adalah aplikasi NATIVE STANDALONE murni yang ditulis dalam bahasa pemrograman RUST dan dikompilasi menggunakan LLVM/MSVC menjadi biner x64 tanpa dependensi pihak ketiga.**

Aplikasi ini menggunakan akselerasi perangkat keras langsung melalui **DirectComposition**, **Direct2D**, dan **DirectWrite** yang diikat langsung ke subsistem Windows melalui crate resmi Microsoft `windows-rs`.

---

## 2. Struktur Portable Executable (PE32+)

Pemeriksaan rinci terhadap PE header biner `nusa.exe` menunjukkan konfigurasi berikut:

```text
Format Biner          : PE32+ (64-bit AMD64)
Arsitektur Target     : x86-64 / AMD64 (Machine 0x8664)
Subsystem             : Windows GUI (2)
ImageBase             : 0x140000000
Jumlah Bagian (Sect)  : 7 Seksi
TimeDateStamp         : 1793539732 (Kompilasi November 2026)
Characteristics       : 0x0022 (EXECUTABLE_IMAGE | LARGE_ADDRESS_AWARE)
Entry Point RVA       : 0x000D09D0
CLR Header (COM+)     : VA=0x0, Size=0x0 (Bukan .NET)
```

### Layout Seksi Biner (Section Headers)

| Seksi | Virtual Address | Virtual Size | Raw Offset | Raw Size | Karakteristik | Fungsi |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **`.text`** | `0x00001000` | 888,832 byte | `0x00000400` | 888,832 byte | `0x60000020` | Kode biner eksekusi utama |
| **`.rdata`** | `0x000DA000` | 196,132 byte | `0x000D9400` | 196,608 byte | `0x40000040` | Data konstan, tabel IAT, dan string |
| **`.data`** | `0x0010A000` | 8,104 byte | `0x00109400` | 3,584 byte | `0xC0000040` | Variabel global mutable |
| **`.pdata`** | `0x0010C000` | 19,584 byte | `0x0010A200` | 19,968 byte | `0x40000040` | Exception handling unwind data |
| **`.fptable`** | `0x00111000` | 256 byte | `0x0010F000` | 512 byte | `0xC0000040` | Function pointer table (Control Flow Guard) |
| **`.rsrc`** | `0x00112000` | 150,272 byte | `0x0010F200` | 150,528 byte | `0x40000040` | Resource icon dan version info |
| **`.reloc`** | `0x00137000` | 4,420 byte | `0x00133E00` | 4,608 byte | `0x42000040` | Relokasi dasar (ASLR) |

---

## 3. Identifikasi Compiler & Toolchain (Rust)

Bukti konkret bahwa `nusa.exe` dikompilasi menggunakan bahasa pemrograman **Rust**:

1. **Jejak Kompilator (`rustc` Commit Signature):**
   Ditemukan puluhan referensi internal ke jalur kompilasi standar Rust:
   `rustc/48a229ceaefd4985c50990b14116b6d856af0985` (commit resmi `rustc` pada toolchain stabil Rust 1.80/1.81).
2. **Library Standar Rust:**
   Ditemukan modul internal:
   - `library\std\src\sync\mpmc\` (Multi-Producer Multi-Consumer channel)
   - `library\std\src\sync\once_lock.rs`
   - `library\std\src\sys\pal\windows\`
   - `library\panic_abort\` (Strategi penanganan panic: abort)
   - `src\main.rs` (Entry point proyek pengembang)
3. **Crate Eksternal yang Digunakan:**
   Pemeriksaan prefix `cargo\registry\src\index.crates.io-*` mengungkap bahwa aplikasi ini memiliki dependensi pihak ketiga yang sangat minimal:
   - **`windows-0.58.0`** (Microsoft Official Windows crate)
   - **`windows-core-0.58.0`**
   - **`windows-strings-0.1.0`**
   Tidak ada crate UI berat seperti `tauri`, `egui`, `iced`, atau `slint`. UI dibangun langsung di atas API Windows native!

---

## 4. Arsitektur Grafis & Rendering Pipeline

Pemeriksaan daftar Dynamic Link Libraries (DLL) yang diimpor mengungkap arsitektur grafis canggih yang digunakan Nusa:

### A. DirectComposition (`dcomp.dll`)
- **API Diimpor:** `DCompositionCreateDevice`
- **Tujuan:** DirectComposition adalah API komposisi grafis Windows modern (diperkenalkan sejak Windows 8 dan disempurnakan di Windows 10/11) yang memungkinkan rendering visual berlapis (*visual tree*) dengan transparansi per-pixel, efek animasi hardware-accelerated, dan penggabungan jendela tanpa latensi GDI.
- **Tipe Jendela:** Jendela menggunakan style `WS_EX_NOREDIRECTIONBITMAP` (`0x08000000`), membuktikan jendela ini mengikat langsung (*bind*) visual target DirectComposition ke DXGI Swapchain, bukan menggunakan GDI redirection bitmap biasa.

### B. Direct2D & Direct3D 11 (`d2d1.dll`, `d3d11.dll`)
- **API Diimpor:** `D2D1CreateFactory`, `D2D1MakeRotateMatrix`, `D3D11CreateDevice`
- **Tujuan:** Menggambar seluruh elemen grafis Dynamic Island (bentuk pil/kapsul dengan sudut membulat/rounded corner, ikon vektor, progress bar, audio visualizer bars, dan teks) langsung pada kartu grafis (GPU).
- **Hasil:** Animasi ekspansi, perubahan ukuran, dan rotasi berlangsung dengan frame rate 60–120+ FPS tanpa membebani CPU.

### C. DirectWrite (`dwrite.dll`)
- **API Diimpor:** `DWriteCreateFactory`
- **Tujuan:** Tipografi teks yang tajam pada semua skala DPI Windows.
- **Font Utama:** `Segoe UI` (dikonfirmasi dari string biner offset `0x000DFB68`).

### D. Desktop Window Manager (`dwmapi.dll`)
- **API Diimpor:** `DwmSetWindowAttribute`
- **Tujuan:** Mengontrol atribut blur/frosted glass dan transparansi backdrop di belakang island yang sedang terbuka (`Appearance: Frosted glass behind the open notch`).

---

## 5. Integrasi Windows Runtime (WinRT) & API Sistem

Nusa mengintegrasikan fitur-fitur desktop tingkat tinggi melalui WinRT dan Win32 APIs:

| Modul WinRT / Win32 | Fungsi dalam Aplikasi Nusa |
| :--- | :--- |
| **`Windows.Media.Control.GlobalSystemMediaTransportControlsSessionManager`** (GSMTC) | Memantau media yang sedang diputar di Windows (Spotify, Chrome, YouTube, Apple Music), membaca judul lagu, nama artis, thumbnail/album art, dan mengirim perintah play/pause/skip. |
| **`Windows.Storage.Streams.DataReader`** | Membaca thumbnail/album art binary stream dari GSMTC. |
| **`Windows.Web.Http.HttpClient`** | Mengambil lirik lagu secara asinkron dari API basis data lirik [LRCLIB](https://lrclib.net) (`https://lrclib.net/api/search?track_name=...&artist_name=...`). |
| **`Windows.UI.Notifications.Management.UserNotificationListener`** | Menangkap notifikasi Windows dari aplikasi lain untuk ditampilkan pada Dynamic Island (Notification Pill). |
| **`Windows.Networking.Connectivity.NetworkInformation`** | Mendeteksi status sambungan internet (Online/Offline) dan perubahan adapter jaringan. |
| **`Windows.Devices.Bluetooth.BluetoothDevice`** & `cfgmgr32.dll` | Mendeteksi koneksi perangkat Bluetooth (headset, earphone) dan menampilkan popup baterai saat terhubung. |
| **`RegisterPowerSettingNotification`** | Memantau status baterai laptop (Low battery, Critical, Fully charged, Charger plugged/unplugged). |
| **`SetWinEventHook`** & `SHQueryUserNotificationState` | Mendeteksi aplikasi layar penuh (games, presentasi, video full-screen) untuk menyembunyikan notch secara otomatis (`hide_fullscreen`). |
| **`RegisterDragDrop`**, `DoDragDrop`, `DragQueryFileW` (OLE32) | Implementasi fitur **File Shelf**: pengguna dapat men-drag file dari File Explorer ke notch, menyimpannya sementara, dan men-drag-nya keluar kapan saja. |
| **`GetClipboardData`**, `ExcludeClipboardContentFromMonitorProcessing` | Implementasi fitur **Clipboard History**: menyimpan riwayat copy-paste lokal pengguna. |
| **`PlaySoundW`** (`winmm.dll`) | Memainkan nada dering/chime pengingat (Pomodoro break, Water reminder, To-do alert). |

---

## 6. Analisis Asset & Resource (`.rsrc`)

Pemeriksaan direktori resource `RT_ICON`, `RT_GROUP_ICON`, dan `RT_VERSION`:
- **Ikon Aplikasi:** Terdapat 15 varian resolusi ikon mulai dari 16×16 hingga 256×256 px dalam format kompresi PNG multi-layer.
- **Asset Gambar Eksternal:** **Tidak ada file PNG, JPG, SVG, atau file font terpisah yang dibundle.**
- **Mekanisme Render Simbol/Ikon:** Seluruh ikon UI (ikon play, pause, speaker, to-do checkbox, botol minum, bulan, lonceng, jam) di-render secara prosedural melalui perintah vektor Direct2D (`ID2D1Geometry` / `ID2D1PathGeometry`).

---

## 7. Eliminasi Hipotesis yang Terbukti Salah

| Hipotesis Awal | Hasil Pemeriksaan | Bukti Konkret |
| :--- | :--- | :--- |
| **Electron** | **TIDAK TERBUKTI (FALSE)** | Tidak ada `resources/app.asar`, `node.dll`, `v8`, Chromium content modules, atau file HTML/JS. |
| **Tauri** | **TIDAK TERBUKTI (FALSE)** | Tidak ada `WebView2Loader.dll`, tidak ada dependensi `wry`/`tao`, dan tidak ada bundel frontend web. |
| **.NET / WPF / WinUI** | **TIDAK TERBUKTI (FALSE)** | CLR Header bernilai nol (`0x0`). Tidak ada metadata .NET atau runtime CoreCLR. |
| **Qt / QML** | **TIDAK TERBUKTI (FALSE)** | Tidak ada ketergantungan pada `Qt6Core.dll` atau engine QML. |
| **Flutter Desktop** | **TIDAK TERBUKTI (FALSE)** | Tidak ada `flutter_windows.dll` atau direktori `data/flutter_assets`. |
| **Native Rust + DComp** | **TERBUKTI (CONFIRMED)** | Jejak `rustc`, crate `windows-rs`, import `dcomp.dll`, `d2d1.dll`, `dwrite.dll`. |

