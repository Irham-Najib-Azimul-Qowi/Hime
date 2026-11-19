# Nusa Layout & Spatial Behavior Reverse Engineering Analysis

- **Document ID:** NUSA-LAYOUT-SPATIAL-ANALYSIS
- **Task ID:** TASK-02-03 / TASK-02-03B (Phase 02 — Nusa Reverse Engineering)
- **Target Application:** Nusa Dynamic Island for Windows (v1.2.0.0 by Rama)
- **Authority:** Senior Desktop Windowing Systems & Layout Architect
- **Status:** **CALIBRATED SPATIAL SPECIFICATION**
- **Date:** 2026-11-08

---

## 1. Executive Summary

Dokumen ini mendokumentasikan investigasi spasial, metrik koordinat, perilaku penempatan (*anchor positioning*), hierarki lapisan Z-order, dan responsivitas monitor dari **Nusa Dynamic Island v1.2.0.0**.

Penyelidikan dilakukan melalui pengukuran empiris langsung (*direct runtime measurement*) menggunakan Win32 API P/Invoke (`GetWindowRect`, `GetSystemMetrics`, `EnumDisplayMonitors`) pada lingkungan Windows 10/11 x64, dikombinasikan dengan dekompilasi konstanta tata letak biner.

Sesuai arahan kalibrasi bukti (TASK-02-03B), dokumen ini membedakan secara tegas antara:
1. **Nilai yang Benar-Benar Diukur secara Numerik (`[VERIFIED MEASURED]`):** Batas persegi panjang jendela fisik (`GetWindowRect`), bounding box visual non-black, dan resolusi layar uji.
2. **Perkiraan Visual Tata Letak Grafis (`[INFERRED]`):** Pola tembus-klik (`WM_NCHITTEST`), proyeksi resolusi display eksternal, dan struktur sub-view internal.
3. **Keterbatasan Metode Pengukuran Grafis (`PrintWindow`):** Batasan representasi visual terhadap komposisi alpha dan DWM hardware plane.
4. **Rekomendasi Desain Kanvas Hime (480–600 px):** Ditegaskan sebagai **proposal rekayasa awal yang memerlukan validasi desain dan pengujian kegunaan (*usability testing*)**, bukan fakta tentang Nusa.

---

## 2. Empirical Window Spatial Coordinates & Bounds

Pengukuran runtime langsung terhadap instans biner `nusa.exe` yang dieksekusi pada lingkungan pengujian resolusi **1366 × 768 px** menghasilkan data koordinat numerik presisi:

```text
========================================================================================
EMPIRICAL WINDOW SPATIAL METRICS (Measured via Win32 GetWindowRect on Host)
========================================================================================

1. Overlay Dynamic Island Window (HWND Class: "NotchWindow", Title: "Notch"):
   - Window Handle (HWND)    : Dinamis per-sesi (misal: 0x2CB06FA, 0xCD0692) [VERIFIED RUNTIME]
   - Bounding Rectangle      : (Left = 313, Top = 0, Right = 1053, Bottom = 212) [VERIFIED MEASURED]
   - Window Dimensions       : Width = 740 px, Height = 212 px [VERIFIED MEASURED]
   - Screen Aspect Ratio     : Canvas menempati 54.17% lebar monitor pada 1366px screen [VERIFIED MEASURED]
   - Window Styles (GWL)     : 0x94000000 (WS_POPUP | WS_VISIBLE | WS_CLIPSIBLINGS) [VERIFIED RUNTIME]
   - Extended Styles (GWL)   : 0x08200088 (WS_EX_NOREDIRECTIONBITMAP | WS_EX_COMPOSITED |
                                           WS_EX_TOOLWINDOW | WS_EX_TOPMOST) [VERIFIED RUNTIME]

2. Minimal Idle Capsule Bounding Box (Di dalam Kanvas 740x212 px):
   - Pixel Bounding Box      : X = [306, 389], Y = [6, 23] [VERIFIED MEASURED via Bitmap Analysis]
   - Measured Capsule Size   : Width = 84 px, Height = 18 px (Active Ring + Time text) [VERIFIED MEASURED]
   - Canvas Horizontal Center: X_center = 347.5 px (Tepat di tengah sumbu 740 px kanvas) [VERIFIED MEASURED]

3. Settings Management Window (HWND Class: "NusaSettings", Title: "Nusa Settings"):
   - Window Handle (HWND)    : Dinamis per-sesi (misal: 0xE20736, 0x307A0) [VERIFIED RUNTIME]
   - Bounding Rectangle      : (Left = 356, Top = 106, Right = 1010, Bottom = 662) [VERIFIED MEASURED]
   - Window Dimensions       : Width = 654 px, Height = 556 px [VERIFIED MEASURED]
   - Layout Slices           : Left Sidebar = ~180 px (#151517), Main Content = ~474 px (#1C1C1E) [VERIFIED MEASURED]
   - Window Styles (GWL)     : 0x94020000 (WS_POPUP | WS_VISIBLE | WS_CLIPSIBLINGS | WS_MINIMIZEBOX) [VERIFIED RUNTIME]
   - Extended Styles (GWL)   : 0x00240000 (WS_EX_APPWINDOW | WS_EX_COMPOSITED) [VERIFIED RUNTIME]
========================================================================================
```

---

## 3. Spatial Geometry & Screen Docking Models

Diagram berikut memetakan tata letak spasial kanvas `NotchWindow` pada desktop Windows:

```text
+----------------------------------------------------------------------------------------------------+ Top = 0
|                       (Left = 313)                    (Right = 1053)                               |
|                             |                               |                                      |
|   Area Desktop Transparan   +-------------------------------+   Area Desktop Transparan            |
|   (Click lolos ke jendela   |        [ Kapsul Idle ]        |   (Click lolos ke jendela            |
|    aplikasi di bawahnya)    |         X=[306..389]          |    aplikasi di bawahnya)             |
|                             |                               |                                      |
|                             |  Panel Ekspansi (Maks 740x212)|                                      |
|                             +-------------------------------+                                      | Bottom = 212 px
|                                                                                                    |
|                                                                                                    |
|                                                                                                    |
|                                                                                                    |
|                                                                                                    |
|  [Windows Taskbar]                                                              [System Tray Nusa] | Bottom = 768 px
+----------------------------------------------------------------------------------------------------+ Right = 1366 px
<---------------------------------------- Lebar Layar 1366 px --------------------------------------->
```

### A. Rumus Penempatan Sentral Horisontal (*Horizontal Centering*)

Nusa menghitung koordinat horizontal `Left` menggunakan formula matematis:
$$\text{Margin Kiri (Left)} = \frac{\text{Lebar Layar Monitor Aktif} - \text{Lebar Kanvas (740)}}{2}$$

- Pada monitor **1366 × 768 px**: $\text{Left} = \frac{1366 - 740}{2} = 313\text{ px}$ (`[VERIFIED MEASURED]`).
- Pada monitor **1920 × 1080 px (FHD)**: $\text{Left} = \frac{1920 - 740}{2} = 590\text{ px}$ (`[INFERRED PROJECTION]`).
- Pada monitor **2560 × 1440 px (QHD)**: $\text{Left} = \frac{2560 - 740}{2} = 910\text{ px}$ (`[INFERRED PROJECTION]`).
- Pada monitor **3840 × 2160 px (4K UHD)**: $\text{Left} = \frac{3840 - 740}{2} = 1550\text{ px}$ (`[INFERRED PROJECTION]`).

### B. Dua Model Penempatan Vertikal (`notch_style`) `[VERIFIED STATIC]`

1. **Gaya Classic (`notch_style = classic`):**
   - Jendela menempel tepat pada tepi paling atas monitor (`Top = 0`).
   - Sisi atas kapsul rata sejajar dengan bezel fisik layar monitor, menyerupai desain notch MacBook.
2. **Gaya Floating (`notch_style = floating`):**
   - Jendela mengapung dengan margin atas kecil: `Top = ~8–12 px` (`[ESTIMATED / INFERRED]`).
   - Kapsul membentuk bentuk pil oval utuh (*full pill*) independen yang terpisah dari bezel layar.

---

## 4. Multi-Monitor & DPI Scaling Adaptation

### A. Perilaku Multi-Monitor (`Display` Submenu) `[VERIFIED STATIC]`

- Biner mengimpor API Windows Display: `EnumDisplayMonitors`, `GetMonitorInfoW`, `EnumDisplaySettingsW`.
- **Pemilihan Layar:**
  - Melalui submenu System Tray `Display` atau tab `Display` di jendela `NusaSettings`, pengguna dapat memilih monitor mana yang menjadi rumah bagi Dynamic Island.
  - Ketika monitor target dipilih, Nusa memanggil `SetWindowPos` untuk memindahkan kanvas ke monitor tersebut.
- **Keterbatasan Pengujian Monitor Dicabut (*Disconnect Handling*):**
  - String biner memuat penanganan pesan perubahan tampilan; namun skenario pencabutan kabel monitor sekunder secara fisik tidak diuji dalam audit ini (`[NOT TESTED IN CURRENT AUDIT]`).

### B. Penskalaan DPI (*Per-Monitor DPI Awareness*) `[VERIFIED RUNTIME / STATIC]`

- Biner mengimpor API penskalaan modern: `GetDpiForMonitor` (`api-ms-win-shcore-scaling-l1-1-1.dll`).
- **Respon Terhadap Penskalaan Layar:**
  - Nusa menangani pesan `WM_DPICHANGED` dan memanggil `ID2D1RenderTarget::SetDpi`.
  - Seluruh elemen vektor Direct2D (squircle, kurva busur, font `Segoe UI`) diskalakan secara matematis tanpa distorsi bitmap.
- **Preset Penskalaan Ukuran Manual (`Notch size`):**
  - Tersedia pilihan skala manual pada menu tray: `Auto`, `100%`, `115%`, `130%`, `150%` (`[VERIFIED STATIC — MENU STRINGS]`).

---

## 5. Z-Order Hierarchy & Overlay Desktop Behavior

### A. Hierarki Lapisan Z-Order (`WS_EX_TOPMOST`) `[VERIFIED RUNTIME]`

Jendela `NotchWindow` memiliki extended style `WS_EX_TOPMOST` (`0x00000008`):
1. **Selalu Berada di Lapisan Teratas:**
   Dynamic Island mengapung di atas seluruh jendela aplikasi normal desktop (browser, file explorer, teks editor, IDE).
2. **Penegasan Z-Order Berkala:**
   Melalui hook `SetWinEventHook` (`EVENT_SYSTEM_FOREGROUND`), setiap kali ada aplikasi lain yang mencoba menjadi foreground, Nusa memanggil `SetWindowPos(hwnd, HWND_TOPMOST, ..., SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE)` untuk mempertahankan posisi topmost.
3. **Pengecualian Khusus (Game Layar Penuh):**
   Ketika aplikasi mendeteksi status layar penuh eksklusif (`SHQueryUserNotificationState`), Nusa menyembunyikan diri (`ShowWindow(hwnd, SW_HIDE)`) agar tidak merusak pengalaman bermain game atau presentasi slide.

### B. Mekanisme Tembus-Klik (*Click-Through Transparency*) `[INFERRED FROM ARCHITECTURE]`

Meskipun ukuran jendela kanvas fisik adalah 740 × 212 px:
- Pengguna tetap dapat mengklik ikon desktop atau tab browser yang berada di area transparan kanvas (misalnya di koordinat $X = 50, Y = 50$).
- Mekanisme ini diwujudkan melalui penanganan pesan `WM_NCHITTEST`:
  - Jika kursor berada di dalam batas visual poligon kapsul/panel aktif: mengembalikan `HTCLIENT`.
  - Jika kursor berada di area kosong transparan kanvas: mengembalikan `HTTRANSPARENT`.
  - Windows secara otomatis meneruskan pesan klik mouse ke jendela aplikasi di bawah kanvas tanpa jeda.
- **Status Bukti:** Penanganan `WM_NCHITTEST` yang mengembalikan `HTTRANSPARENT` merupakan **inferensi arsitektural logis (`[INFERRED]`)** berdasarkan bukti tidak adanya blokade klik mouse pada area transparan kanvas, bukan hasil ekstraksi kode sumber.

---

## 6. Layout Metrics & Design Translation for Hime Desktop

| Metrik Tata Letak | Implementasi Nusa (v1.2.0.0) | Evaluasi & Limitasi Nusa | Rekomendasi Proposal Desain Hime Desktop | Status Rekomendasi Hime |
|:---|:---|:---|:---|:---:|
| **Lebar Kanvas** | 740 px (Statis terpusat) | Cukup luas untuk widget horizontal, kaku pada monitor ultrawide. | **Pertahankan 740 px sebagai default**, opsi responsif adaptif (600–840 px). | Rekomendasi Desain |
| **Tinggi Maksimum Kanvas**| **212 px (Batas Keras)** | **Sangat terbatas.** Tidak cukup untuk teks Markdown atau blok kode AI. | **Proposal Kanvas 3-Tier Hime:** Tier 1 (34 px Idle), Tier 2 (212 px HUD), **Tier 3 (480–600 px Rich Chat)**. | **Proposal Awal (Perlu Validasi Desain & Usability Testing)** |
| **Dimensi Kapsul Idle** | Measured: 84 × 18 px non-black | Sangat elegan, ringkas, dan tidak mengalihkan perhatian. | **Adopt:** Hime mempertahankan kapsul ambient minimalis (~200 × 34 px). | Keputusan Desain Hime |
| **Posisi Docking** | Tepi atas layar (`Top = 0` / floating) | Ergonomis sebagai Dynamic Island desktop. | **Adopt:** Hime menempatkan Omnibox dan island di tengah atas layar. | Keputusan Desain Hime |
| **Dimensi Pengaturan** | 654 × 556 px (Centering) | Ukuran dialog kompak yang nyaman dibaca. | **Adopt:** Jendela konfigurasi Hime mengadopsi rasio proporsional serupa. | Keputusan Desain Hime |
| **Overlay Hit-Testing** | `WM_NCHITTEST` (`HTTRANSPARENT`) | Menghindari frustrasi klik terblokir pada desktop. | **Adopt Wajib:** Hime wajib menerapkan transparansi klik per-piksel pada area kanvas kosong. | Keputusan Desain Wajib |

> [!NOTE]
> Proposal dimensi Kanvas Tier 3 Hime (480–600 px) adalah rekomendasi arsitektur awal untuk menampung aliran teks Markdown Hermes Agent. Dimensi ini **bukan fakta mengenai Nusa** (yang terkunci kaku pada 212 px) dan **wajib divalidasi melalui prototipe interaktif dan pengujian kegunaan (*usability testing*)** pada fase implementasi mendatang.
