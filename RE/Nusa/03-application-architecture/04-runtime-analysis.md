# 04 — Runtime Behavior Analysis
## Proyek: Hime Assistant
### Target: Nusa Dynamic Island (`nusa.exe`)

---

## 1. Metodologi Observasi Runtime

Observasi runtime dilakukan secara non-destruktif pada lingkungan pengguna lokal (Windows 10/11 x64).

Pemeriksaan mencakup:
- Eksekusi proses biner dan pengukuran konsumsi memori/thread.
- Inspeksi struktur jendela Win32 (Window handle, window class, koordinat, style, dan extended style) menggunakan Win32 P/Invoke dan CTypes.
- Inspeksi siklus hidup (*lifecycle*), interaksi tray icon, dan mekanisme pencegahan instans ganda (*single instance*).

---

## 2. Profil Proses & Sumber Daya (Process Metrics)

| Parameter | Nilai Terukur | Keterangan & Analisis |
| :--- | :--- | :--- |
| **Nama Proses** | `nusa.exe` | Single process (Bukan multi-proses seperti Electron). |
| **Konsumsi Memori (Working Set)** | **~58 – 61 MB** | **Sangat hemat.** Rata-rata aplikasi Electron mengonsumsi 200–400 MB. Nusa mempertahankan jejak memori minimalis. |
| **Konsumsi CPU Idle** | **0.0% – 0.1%** | Rendering DirectComposition berbasis event (*dirty flag*), tidak me-render ulang jika frame tidak berubah. |
| **Jumlah Thread** | 12 – 14 thread | - Main GUI thread (Window message loop)<br>- DirectComposition composition thread<br>- Media session watcher (GSMTC WinRT listener)<br>- Windows Event Hook listener thread (`SetWinEventHook`)<br>- Network watcher thread (`NetworkInformation`)<br>- Timer worker threads |
| **Mekanisme Mutex** | `Local\NusaNotch.SingleInstance` | Mencegah instans ganda. Jika biner dijalankan untuk kedua kalinya, ia mendeteksi mutex dan keluar secara bersih setelah mengaktifkan jendela yang sudah ada. |

---

## 3. Arsitektur Jendela Utama (`NotchWindow`)

Pengukuran langsung terhadap HWND jendela pulau dinamis Nusa menghasilkan data teknis berikut:

```text
Window Handle (HWND) : 0x003106A6 (Dinamis per-launch)
Window Class Name    : NotchWindow
Window Title         : Notch
Window Coordinates   : Left=313, Top=0, Right=1053, Bottom=212
Window Dimension     : Width=740 px, Height=212 px
Window Style         : 0x94000000 (WS_POPUP | WS_VISIBLE | WS_CLIPSIBLINGS)
Extended Style       : 0x08200088
```

### Analisis Extended Window Styles (ExStyle):

1. **`WS_EX_TOPMOST` (`0x00000008`) [CONFIRMED]**
   - Jendela Dynamic Island diatur secara permanen agar selalu berada di lapisan teratas (*always-on-top*), mengapung di atas semua jendela aplikasi desktop lainnya.
2. **`WS_EX_TOOLWINDOW` (`0x00000080`) [CONFIRMED]**
   - Jendela diperlakukan sebagai utility/tool window: **tidak memunculkan ikon di Taskbar Windows dan tidak muncul dalam daftar switcher Alt+Tab**.
3. **`WS_EX_NOREDIRECTIONBITMAP` (`0x08000000`) [CONFIRMED]**
   - **Karakteristik teknis tingkat tinggi:** Bendera ini menginstruksikan DWM Windows bahwa jendela tidak menggunakan GDI bitmap redirection surface. Nusa merender visual langsung ke DirectX/DirectComposition Swapchain surface, menghasilkan performa rendering nol-overhead dan latensi komposisi terendah.
4. **`WS_EX_COMPOSITED` (`0x00200000`) [CONFIRMED]**
   - Mengaktifkan double-buffering hirarkis dari Windows composition pipeline.

### Analisis Dimensi & Penentuan Posisi:
- **`Top = 0`**: Jendela menempel tepat pada batas atas layar monitor utama (*docked to screen top*).
- **Horizontal Centering**: Pada resolusi layar pengujian (1366 px lebar):
  $$\text{Margin Kiri} = \frac{1366 - 740}{2} = 313\text{ px}$$
  Nusa secara otomatis menghitung titik tengah layar monitor yang aktif dan menempatkan window frame tepat di tengah atas.
- **Ukuran Kanvas (740×212 px)**: Area jendela seluas 740×212 px adalah batas maksimum kanvas rendering DirectComposition. Ketika island dalam status *idle* (pil kecil), hanya area pil di tengah atas yang tampak buram/berisi visual, sedangkan sisa kanvas 740×212 px bersifat transparan per-pixel dan click-through. Ketika island diklik atau di-hover, pil berekspansi secara mulus di dalam kanvas tersebut.

---

## 4. Jendela Konfigurasi (`NusaSettings`)

Ketika aplikasi dipanggil dengan flag `--settings` atau pengguna memilih menu "Settings..." dari Tray Icon:

```text
Window Class Name  : NusaSettings
Window Title       : Nusa Settings
Window Dimensions  : Width=654 px, Height=556 px (Centering pada layar: 356, 106)
Window Style       : 0x94020000 (WS_POPUP | WS_VISIBLE | WS_CLIPSIBLINGS)
Caption / Frame    : Tidak ada (Borderless custom window)
ExStyle            : 0x00240000
```

- **Desain Jendela Pengaturan:** Jendela pengaturan Nusa bukan dialog standar Windows Form atau WinUI, melainkan jendela kustom tanpa bingkai (*frameless*) yang juga di-render langsung menggunakan Direct2D dengan sudut membulat dan tombol navigasi tab di sisi kiri.

---

## 5. Integrasi System Tray & Siklus Hidup Aplikasi

- **Ikon Tray Windows:** Didaftarkan menggunakan API `Shell_NotifyIconW` dengan pesan callback kustom.
- **Struktur Menu Konteks Tray Icon:**
  1. `Settings...` (Membuka jendela pengaturan `NusaSettings`)
  2. `Start with Windows` (Toggle registry Run `HKCU`)
  3. `Nusa Mascot` (Toggle maskot animasi mini di notch)
  4. `Minimal notch` (Toggle ukuran pil minimalis ketika idle)
  5. `Notch size` (Submenu: Auto / 100%, 115%, 130%, 150%)
  6. `Display` (Submenu pemilihan layar pada konfigurasi multi-monitor)
  7. `About Nusa` (Dialog kredit dan versi)
  8. `Exit` (Menutup proses `nusa.exe` secara bersih)

---

## 6. Penanganan Aplikasi Layar Penuh (Fullscreen Detection)

Nusa memiliki mekanisme deteksi aplikasi layar penuh (*fullscreen mitigation*):
- Menggunakan `SHQueryUserNotificationState`: Mendeteksi saat pengguna sedang menjalankan game layar penuh, memutar video fullscreen (VLC, YouTube), atau mode presentasi PowerPoint (`QUNS_BUSY`, `QUNS_RUNNING_D3D_FULL_SCREEN`, `QUNS_PRESENTATION_MODE`).
- Menggunakan `SetWinEventHook`: Mendengarkan event pergantian jendela aktif (`EVENT_SYSTEM_FOREGROUND`).
- Perilaku: Jika opsi `hide_fullscreen` aktif (default), jendela `NotchWindow` disembunyikan sementara (`ShowWindow(SW_HIDE)`) agar tidak menutupi HUD game atau interface presentasi.

---

## 7. Rangkuman Perilaku Runtime

| Fitur Runtime | Observasi Lapangan | Status |
| :--- | :--- | :--- |
| **Footprint Memori** | ~58 MB (Sangat rendah dan stabil) | CONFIRMED |
| **Tipe Window** | Borderless WS_POPUP, Always-On-Top | CONFIRMED |
| **Akselerasi Grafis** | DirectComposition + Direct2D | CONFIRMED |
| **Perilaku Alt+Tab** | Tersembunyi (WS_EX_TOOLWINDOW) | CONFIRMED |
| **Multi-Monitor** | Mendukung pemindahan notch ke monitor lain | CONFIRMED |
| **Deteksi Fullscreen** | Otomatis sembunyi saat game/video fullscreen | CONFIRMED |
| **Tray Integration** | Menu lengkap via Shell_NotifyIconW | CONFIRMED |

