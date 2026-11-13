# Nusa Rendering & Window Architecture Reverse Engineering Analysis

- **Document ID:** NUSA-RENDERING-ARCHITECTURE
- **Task ID:** TASK-02-02 (Phase 02 — Nusa Reverse Engineering)
- **Target Application:** Nusa Dynamic Island for Windows (v1.2.0.0)
- **Authority:** Principal Graphics & Windows Windowing Systems Specialist
- **Status:** **CANONICAL RENDERING SPECIFICATION**
- **Date:** 2026-11-08

---

## 1. Executive Summary

Dokumen ini menyajikan analisis arsitektur terhadap pipeline grafis, akselerasi perangkat keras GPU, sistem komposisi jendela Desktop Window Manager (DWM), dan tipografi dari **Nusa Dynamic Island v1.2.0.0**.

Berdasarkan analisis biner statis terhadap tabel impor PE32+ (`dcomp.dll`, `d2d1.dll`, `d3d11.dll`, `dwrite.dll`, `dwmapi.dll`) dan konstanta pada segmen `.rdata`, arsitektur rendering Nusa mengadopsi API Windows modern:
- **Direct3D 11:** Menyediakan perangkat grafis GPU (`D3D11CreateDevice` ada dalam IAT).
- **Direct2D:** Menggambar elemen grafis antarmuka vektor (`D2D1CreateFactory` ada dalam IAT).
- **DirectComposition:** Menyediakan perangkat komposisi visual GPU (`DCompositionCreateDevice` ada dalam IAT).
- **DirectWrite:** Merender tipografi teks font `Segoe UI` (`DWriteCreateFactory` ada dalam IAT).
- **DWM Integration:** Mengatur atribut jendela backdrop blur (`DwmSetWindowAttribute` ada dalam IAT).

> **Catatan Kalibrasi Bukti Arsitektural:**
> Rantai pipeline terpadu (`D3D11 → DXGI Swapchain → Direct2D → DirectComposition → DWM`) merupakan **`[INFERRED]`** berdasarkan pola standar arsitektur komposisi DirectX Windows. Pengujian ini tidak melibatkan *GPU frame debugger* atau *runtime graphics trace* aktif (seperti PIX atau RenderDoc). Kapabilitas API yang tersedia dibedakan secara tegas dari performa tolok ukur (*benchmark*) riil.

---

## 2. Rendering Pipeline Hierarchy & Data Flow

Diagram berikut memetakan hierarki pipeline rendering grafis Nusa, mulai dari inisialisasi perangkat hingga output monitor.

> **Klasifikasi Bukti Komponen Pipeline:**
> - `[VERIFIED STATIC]`: Keberadaan simbol impor pada Import Address Table (IAT).
> - `[METADATA / STRINGS]`: Didukung oleh string literal dalam segmen `.rdata`.
> - `[INFERRED]`: Alur keterhubungan antar-komponen disimpulkan dari arsitektur standar Windows DirectX/DirectComposition; **bukan hasil tracing frame aktif (PIX/RenderDoc)**.

```mermaid
graph TD
    subgraph "Hardware & Driver Layer"
        GPU[GPU Hardware: Intel / NVIDIA / AMD]
        DXGI[DXGI Infrastructure: dxgi.dll<br><b>[VERIFIED STATIC]</b>]
    end

    subgraph "DirectX Core Device Layer"
        D3DDevice["Direct3D 11 Device & DeviceContext<br>D3D11CreateDevice()<br><b>[VERIFIED STATIC]</b>"]
        SwapChain["DXGI Swap Chain Surface<br>DXGI_FORMAT_B8G8R8A8_UNORM<br><b>[INFERRED PIPELINE LINKAGE]</b>"]
    end

    subgraph "2D Vector & Typography Engine"
        D2DFactory["Direct2D Factory & RenderTarget<br>D2D1CreateFactory()<br><b>[VERIFIED STATIC]</b>"]
        DWriteFactory["DirectWrite Factory & TextFormat<br>DWriteCreateFactory('Segoe UI')<br><b>[VERIFIED STATIC / STRINGS]</b>"]
        D2DGeometry["Procedural Vector Drawing<br>(Squircle, Pills, Rings, Icons)<br><b>[INFERRED PIPELINE USAGE]</b>"]
    end

    subgraph "Composition & Window Integration Layer"
        DCompDevice["DirectComposition Device<br>DCompositionCreateDevice()<br><b>[VERIFIED STATIC]</b>"]
        DCompVisual["DirectComposition Visual Tree<br>IDCompositionVisual Target<br><b>[INFERRED]</b>"]
        HWND_Notch["Win32 HWND 'NotchWindow'<br>Style: WS_POPUP (0x94000000)<br>ExStyle: WS_EX_NOREDIRECTIONBITMAP (0x08200088)<br><b>[VERIFIED STATIC APIS / INFERRED STYLE]</b>"]
        DWMBlur["DWM System Backdrop Blur<br>DwmSetWindowAttribute()<br><b>[VERIFIED STATIC]</b>"]
    end

    subgraph "Display Compositor"
        DWMCompositor[Desktop Window Manager Compositor]
        Screen["Monitor Output Framebuffer<br>(Target Display Refresh Rate [INFERRED])"]
    end

    %% Pipeline Flow
    GPU --> D3DDevice
    D3DDevice --> DXGI
    DXGI --> SwapChain
    SwapChain --> D2DFactory
    DWriteFactory --> D2DFactory
    D2DGeometry --> D2DFactory
    D2DFactory -->|"Render 2D Frame (dirty=1) [INFERRED]"| DCompVisual
    DCompDevice --> DCompVisual
    DCompVisual -->|"Bind Directly to HWND [INFERRED]"| HWND_Notch
    DWMBlur -->|"Apply Frosted Glass [INFERRED]"| HWND_Notch
    HWND_Notch --> DWMCompositor
    DWMCompositor --> Screen
```

---

## 3. DirectComposition, GDI Redirection, & API Capability vs Performance Reality

### A. Rincian Bukti Komponen Grafis
Untuk menjamin akurasi rekayasa balik, bukti teknis dipisahkan berdasarkan sumbernya:

| Komponen Grafis | Import Terverifikasi IAT (`[VERIFIED STATIC]`) | Bukti Metadata / Strings (`[STRINGS]`) | Keterhubungan Antar-Komponen (`[INFERRED]`) |
|:---|:---|:---|:---|
| **Direct3D 11** | `d3d11.dll`: `D3D11CreateDevice` (offset `0x108D80`) | N/A | Diindikasikan sebagai backend pembuat DXGI device untuk Direct2D. |
| **DXGI** | `dxgi.dll`: dipanggil via D3D11 device | N/A | Permukaan swapchain dihubungkan ke DirectComposition visual. |
| **Direct2D** | `d2d1.dll`: `D2D1CreateFactory` (`0x108CBC`), `D2D1MakeRotateMatrix` (`0x108E12`) | N/A | Menggambar elemen UI pada DXGI surface / render target Direct2D. |
| **DirectWrite** | `dwrite.dll`: `DWriteCreateFactory` (`0x108C8A`) | String `"Segoe UI"` (`0x000DFB68`), `"Segoe Fluent Icons"` | Menyediakan format teks dan layout untuk di-render oleh Direct2D. |
| **DirectComposition**| `dcomp.dll`: `DCompositionCreateDevice` (`0x108CA0`) | N/A | Mengikat visual komposisi DirectX langsung ke target visual HWND. |
| **DWM Integration** | `dwmapi.dll`: `DwmSetWindowAttribute` (`0x108DFA`) | String `"glass"` (`0x000E05B0`) | Mengaktifkan atribut blur/backdrop pada jendela `NotchWindow`. |

> **Catatan Status Pipeline Aktif:**
> Rantai pipa `D3D11 → DXGI Swapchain → Direct2D → DirectComposition → DWM` **merupakan kesimpulan inferensi (`[INFERRED]`)** berdasarkan pola standar API komposisi native Windows. Analisis ini tidak melakukan tracing frame GPU runtime aktif menggunakan alat debugger grafis.

### B. Extended Style `WS_EX_NOREDIRECTIONBITMAP` (`0x08000000`): Kapabilitas API vs Realitas Performa

- **Kapabilitas API:**
  - `WS_EX_NOREDIRECTIONBITMAP` adalah bendera extended window style Win32 yang secara spesifik menginstruksikan DWM untuk **meniadakan alokasi GDI redirection bitmap surface**.
  - Pada aplikasi Win32 transparan tradisional (`WS_EX_LAYERED`), DWM mengalokasikan bitmap memori sistem per-jendela untuk compositing CPU-ke-GPU. Bendera ini menonaktifkan surface redirection tersebut, mensyaratkan aplikasi menyediakan swapchain DirectX sendiri (via DirectComposition).
- **Koreksi Klaim Performa (API Capability vs Benchmark):**
  - *Bukan Jaminan Nol Overhead:* Peniadaan GDI redirection bitmap hanya meniadakan alokasi bitmap DWM tertentu; ini **tidak secara otomatis menghilangkan seluruh overhead GDI aplikasi** jika aplikasi masih memanggil API GDI konvensional (misal penanganan font atau kursor).
  - *Bukan Jaminan Otomatis 60 FPS:* Keberadaan API DirectComposition dan bendera ini **bukan jaminan performa 60 FPS tanpa beban GPU**. Frame rate nyata sangat bergantung pada kompleksitas penggambaran vektor Direct2D, alokasi memori tekstur, resolusi layar, sinkronisasi VSync kompositor DWM, dan beban GPU saat runtime.
  - Kapabilitas arsitektural API tidak boleh disamakan dengan hasil tolok ukur (*benchmark*) riil yang terverifikasi.

---

## 4. Direct2D & DirectWrite Vector Typography Pipeline

### A. Direct2D Vector Engine (`d2d1.dll`) `[VERIFIED STATIC]`

- **API Diimpor:** `D2D1CreateFactory` (`0x108CBC`), `D2D1MakeRotateMatrix` (`0x108E12`).
- **Mekanisme Menggambar UI Prosedural:**
  - Karena tidak ada berkas gambar PNG/SVG tersemat di `.rsrc` untuk kontrol UI, Nusa menggambar seluruh visualnya secara prosedural:
    - **Geometri Kapsul / Squircle:** Menggunakan `ID2D1PathGeometry` dengan kurva Bézier kontinu pada keempat sudut panel.
    - **Indikator Progres Air & Baterai:** Menggunakan busur lingkaran Direct2D (`ID2D1GeometrySink::AddArc`).
    - **Visualizer Audio & Ikon:** Menggunakan primitif poligon dan garis vektor.
  - Matriks rotasi (`D2D1MakeRotateMatrix`) digunakan untuk memutar ikon jarum jam, fase bulan, atau animasi loader.

### B. DirectWrite Typography Engine (`dwrite.dll`) `[VERIFIED STATIC]`

- **API Diimpor:** `DWriteCreateFactory` (`0x108C8A`).
- **Font Utama:** `Segoe UI` (Terkonfirmasi dari string biner offset `0x000DFB68`).
- **Karakteristik Teks:**
  - Layout teks dihitung menggunakan antarmuka `IDWriteTextLayout`.
  - Rendering teks menggunakan subpixel ClearType anti-aliasing yang terintegrasi langsung dengan target render Direct2D.
  - Memastikan seluruh teks judul lagu, waktu Pomodoro, nama task, dan angka persentase tetap tajam pada semua faktor penskalaan monitor (*Per-Monitor DPI scaling*).

---

## 5. DWM Acrylic Blur & Backdrop Integration

- **API Diimpor:** `DwmSetWindowAttribute` (`dwmapi.dll` offset `0x108DFA`).
- **Fungsi Visual:**
  - Nusa mengaktifkan kebijakan backdrop DWM Windows (`DWMWA_SYSTEMBACKDROP_TYPE` / `ACCENT_ENABLE_ACRYLICBLURBEHIND`).
  - Ketika panel Dynamic Island berekspansi (`glass=1` pada `settings.txt`), DWM secara otomatis menerapkan efek *frosted glass* halus yang memburamkan wallpaper desktop dan jendela aplikasi di belakang notch.
  - Efek blur ini diproses langsung oleh shader DWM Windows GPU tanpa membebani thread aplikasi Nusa.

---

## 6. Window Geometry, Transparency, & Hit-Testing

### A. Geometri Kanvas 740 × 212 px `[INFERRED FROM ARTIFACTS]`

- Kanvas rendering fisik jendela `NotchWindow` berukuran tetap **740 × 212 px** yang menempel pada batas atas monitor (`Top = 0` atau margin floating ~8–12 px).
- Pada mode kapsul idle, visual hanya menempati area kecil di tengah atas (~200 × 34 px).

### B. Mekanisme Hit-Testing Per-Piksel (`WM_NCHITTEST`) `[INFERRED]`

Untuk memastikan kanvas besar 740 × 212 px tidak memblokir interaksi mouse pada aplikasi lain saat island dalam status idle:

```text
Cursor Position Hit-Testing Logic:
+-------------------------------------------------------------------------+ Top = 0
| [HTTRANSPARENT]        +-----------------------+        [HTTRANSPARENT] |
| (Klik lolos ke bawah)  |   [HTCLIENT] Kapsul   |  (Klik lolos ke bawah) |
|                        +-----------------------+                        |
|                                                                         |
|            +-----------------------------------------------+            |
|            |    Area Panel Ekspansi (Saat Terbuka: HTCLIENT)  |          |
|            |    (Saat Idle: HTTRANSPARENT - Tembus Klik)   |            |
|            +-----------------------------------------------+            |
|                                                                         |
+-------------------------------------------------------------------------+ Bottom = 212 px
<------------------------------- 740 px ---------------------------------->
```

1. Windows mengirimkan pesan `WM_NCHITTEST` setiap kali kursor mouse bergerak di atas koordinat kanvas.
2. Window Procedure memeriksa apakah titik kursor berada di dalam batas poligon visual yang sedang aktif (*active visual bounding box*).
3. Jika kursor berada di dalam kapsul: mengembalikan `HTCLIENT` (mouse event diproses oleh Nusa).
4. Jika kursor berada di luar kapsul (area transparan): mengembalikan `HTTRANSPARENT` (Windows langsung meneruskan klik mouse ke jendela aplikasi atau desktop di bawah kanvas Nusa).

---

## 7. DPI Scaling & Multi-Monitor Adaptation

### A. Per-Monitor DPI Scaling (`GetDpiForMonitor`) `[VERIFIED STATIC]`

- Biner mengimpor `GetDpiForMonitor` dari `api-ms-win-shcore-scaling-l1-1-1.dll`.
- Saat jendela notch dipindahkan antar-monitor dengan skala DPI berbeda (misal: Monitor 1 pada 100% DPI $\leftrightarrow$ Monitor 2 pada 150% DPI):
  1. Windows mengirimkan pesan `WM_DPICHANGED`.
  2. Nusa membaca DPI baru dari `HIWORD(wParam)` / `LOWORD(wParam)`.
  3. Memperbarui resolusi render Direct2D via `ID2D1RenderTarget::SetDpi`.
  4. Seluruh elemen vektor dan teks dihitung ulang dimensinya secara proporsional tanpa mengalami artefak buram (*no bitmap stretching blur*).

---

## 8. Animation Scheduling & Event-Driven Rendering Lifecycle

Struktur biner pada offset `0x000E2AA8` membuktikan FSM status animasi berbasis 4 flag variabel:

```mermaid
stateDiagram-v2
    [*] --> Idle: Inisialisasi Window & RenderTarget

    state Idle {
        [*] --> Sleeping: dirty = 0, animating = 0
        Sleeping --> DirtyState: Event Tiba (dirty = 1)
    }

    Idle --> AnimatingState: Pointer Hover / Click / HUD Event

    state AnimatingState {
        animating: 1
        dirty: 1
        StepInterpolation: Hitung Interpolasi Frame Direct2D
        PresentFrame: SwapChain Present (Target Display Refresh Rate [INFERRED])
    }

    AnimatingState --> AnimatingState: Belum Mencapai Target Dimensi
    AnimatingState --> Idle: Target Tercapai (animating = 0, dirty = 0)
```

- **Pola Event-Driven Repainting:**
  - Nusa **tidak menggunakan** loop rendering tak terhingga (*infinite while loop*) seperti pada game engine.
  - Jika tidak ada animasi yang berjalan (`animating = 0`) dan tidak ada data baru yang berubah (`dirty = 0`), render loop berhenti total.
  - Pola ini menjadi kunci arsitektur mengapa Nusa mampu mempertahankan konsumsi CPU yang sangat rendah saat status idle.

---

## 9. Architectural Translation Map for Hime Desktop (Tauri 2.0 / Rust)

| Komponen Rendering Nusa | Pustaka Nusa (C++/Rust Native) | Strategi Reimplementasi Hime Desktop (Tauri 2.0) | Crate / Teknologi Pengganti di Hime |
|:---|:---|:---|:---|
| **Window Container** | Win32 `CreateWindowExW` | Tauri Window Builder via Rust | `tauri::window::WindowBuilder` |
| **Window Frame & Styles** | `WS_POPUP \| WS_EX_TOPMOST \| WS_EX_TOOLWINDOW` | Konfigurasi deklaratif Tauri: `transparent(true)`, `decorations(false)`, `always_on_top(true)`, `skip_taskbar(true)` | `tauri.conf.json` / `tauri::window` |
| **Zero-Redirection Compositing** | `WS_EX_NOREDIRECTIONBITMAP` | Injeksi ExStyle Win32 pada window handle HWND post-initialization | Crate `windows::Win32::UI::WindowsAndMessaging` |
| **Rendering Engine** | Direct2D + Direct3D 11 hardcoded | **Webview2 (Edge Chromium GPU Pipeline)** | HTML5, CSS3, Tailwind CSS, GPU Compositor Webview2 |
| **Animasi Spring & Morphing** | Rumus interpolasi Direct2D C++ | **CSS Transitions / Framer Motion / Web Animations API** | CSS `transition: all 260ms cubic-bezier(0.16, 1, 0.3, 1)` |
| **Tipografi Teks** | DirectWrite (`dwrite.dll`) | Web Typography Engine (Chromium subpixel font rendering) | Font CSS `@font-face` (`Segoe UI Variable`, `Inter`) |
| **Frosted Glass Blur** | DWM Backdrop Policy (`dwmapi.dll`) | CSS `backdrop-filter: blur(24px) saturate(180%)` + DWM Window Backdrop | CSS Backdrop Filter & Crate `windows::Win32::Graphics::DwmApi` |
| **Hit-Testing Per-Piksel** | Prosedur Win32 `WM_NCHITTEST` | Window procedure subclassing di Rust atau Tauri ignore-cursor-events API | `tauri::Window::set_ignore_cursor_events()` |
