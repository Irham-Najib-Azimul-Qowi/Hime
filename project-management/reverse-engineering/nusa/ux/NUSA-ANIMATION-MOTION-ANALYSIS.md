# Nusa Animation & Motion Behavior Reverse Engineering Analysis

- **Document ID:** NUSA-ANIMATION-MOTION-ANALYSIS
- **Task ID:** TASK-02-03 / TASK-02-03B (Phase 02 — Nusa Reverse Engineering)
- **Target Application:** Nusa Dynamic Island for Windows (v1.2.0.0 by Rama)
- **Authority:** Principal Motion Designer & Graphics Animation Engineer
- **Status:** **CALIBRATED MOTION SPECIFICATION**
- **Date:** 2026-11-08

---

## 1. Executive Summary

Dokumen ini menyajikan investigasi teknis mendalam terhadap karakteristik animasi, dinamika gerak (*motion behavior*), kurva transisi, dan orkestrasi visual dari **Nusa Dynamic Island v1.2.0.0**.

Sesuai arahan integritas rekayasa dan kalibrasi bukti (TASK-02-03B), dokumen ini membedakan secara tegas antara:
1. **Perilaku Animasi yang Teramati Secara Nyata (`[VERIFIED RUNTIME / VISUAL]`):** Apa yang benar-benar terlihat saat antarmuka berekspansi, menyusut, atau berpindah panel.
2. **Estimasi Durasi & Parameter Waktu (`[ESTIMATED / INFERRED]`):** Estimasi milidetik (ms) berdasarkan konfigurasi preset biner dan observasi kelancaran visual.
3. **Dugaan Teknik Implementasi Kompilasi (`[INFERRED]`):** Analisis bagaimana interpolasi dilakukan melalui API Direct2D / DWM native tanpa mengasumsikan library pihak ketiga seperti Framer Motion atau React Spring.
4. **Keterbatasan Bukti Tangkapan Layar (`PrintWindow`):** Batasan metode capture Win32 GDI dalam merepresentasikan komposisi GPU/DWM dan efek blur akrilik secara utuh.

---

## 2. Animation Taxonomy & Observable Characteristics

Tabel berikut merangkum 9 jenis animasi utama yang teridentifikasi pada Nusa:

| Tipe Animasi | Deskripsi Gerak Visual Teramati | Estimasi Durasi | Karakteristik Kurva Easing | Sifat Transformasi Grafis | Klasifikasi Bukti |
|:---|:---|:---:|:---|:---|:---:|
| **Entry Animation** | Kapsul meluncur turun (*slide down*) dari balik tepi atas layar disertai peningkatan opacity dari 0% ke 100%. | ~220–260 ms | Damped deceleration (melambat halus di akhir) | Translasi vertikal ($Y$) & Opacity ($\alpha$) | `[VERIFIED VISUAL]` |
| **Exit Animation** | Kapsul meluncur naik (*slide up*) ke arah bezel atas layar sambil memudar cepat hingga menghilang. | ~180–200 ms | Accelerating exit (keluar cepat) | Translasi vertikal ($-Y$) & Opacity ($\alpha$) | `[VERIFIED VISUAL]` |
| **Expand Animation** | Kapsul membesar secara simultan ke samping ($X$) dan ke bawah ($Y$), mempertahankan kurvatur sudut membulat squircle tanpa distorsi visual. | ~220 ms (Normal) / ~120 ms (Fast) [ESTIMATED] | Spring-like ease-out (pelebaran elastis dengan sedikit osilasi minimal) | Interpolasi dimensi ($W, H$) & Path Geometry Direct2D | `[VERIFIED VISUAL]` & `[VERIFIED STATIC]` (Preset) |
| **Collapse Animation** | Panel kartu 740×212 px menyusut secara simultan kembali ke bentuk kapsul idle di tengah atas layar. | ~200–240 ms [ESTIMATED] | Smooth contraction (penyusutan mulus) | Interpolasi dimensi ($-W, -H$) & Path Geometry | `[VERIFIED VISUAL]` |
| **Peek Hover Animation**| Kapsul membesar mikro (~4–8 px ke arah samping dan bawah) saat kursor mendekat sebagai indikasi responsivitas. | ~120–150 ms [ESTIMATED] | Subtle spring damp | Skala mikro / ekspansi padding kanvas | `[VERIFIED STATIC]` (String `peek`) / `[INFERRED]` |
| **Panel Cross-Transition**| Konten halaman fitur lama meluncur horizontal keluar, konten halaman baru meluncur masuk disertai crossfade teks. | ~180 ms [ESTIMATED] | Standard cubic-bezier ease-in-out | Translasi horizontal ($X$) & Opacity ($\alpha$) | `[INFERRED]` |
| **Backdrop Blur Fade** | Efek kaca buram DWM Acrylic di belakang panel bertransisi mulus dari transparan ke buram penuh. | Sinkron dengan durasi expand (~220 ms) | Linear to ease-out alpha ramp | Parameter DWM backdrop blur | `[VERIFIED STATIC — API]` / `[INFERRED VISUAL]` |
| **HUD Alert Bounce** | Kapsul HUD memanjang elastis saat tombol volume ditekan, bergetar mikro di akhir transisi. | ~200 ms in, ~2.5s hold, ~200ms out [ESTIMATED] | Over-damped spring | Interpolasi lebar ($W$) & Opacity ($\alpha$) | `[VERIFIED RUNTIME — VOLUME]` |
| **Mascot Animation** | Karakter pixel-art di notch melakukan siklus animasi idle (berkedip, bernapas, tidur, atau mengangguk saat musik diputar). | Frame interval ~250–500 ms [ESTIMATED] | Step interpolation (pixel-art sprite frame) | Pergantian frame bitmap / geometri vektor | `[VERIFIED STATIC]` (String `Nusa Mascot`) / `[INFERRED]` |

---

## 3. Deep Analysis of Core Motion Mechanics

### A. Simultan Morphing Geometri Squircle (Expand / Collapse)

Pencapaian visual paling menonjol pada Nusa adalah ekspansi Dynamic Island dari pil kecil (~180 × 34 px / measured idle 84 × 18 px) menjadi panel besar (740 × 212 px) yang berlangsung **tanpa kedipan (*zero-flicker*) dan tanpa distorsi sudut membulat**:

```text
Morphing Progression Breakdown (Normalized Time t = 0.0 -> 1.0):
t = 0.0 (Idle):      [   Pill Capsule: Measured Active 84x18 px, Corner Radius ~9 px   ]
                                |
t = 0.3 (Transition): [    Morphing Box: ~380 x 90 px, Corner Radius ~18 px    ]
                                |
t = 0.7 (Near End):   [     Panel Nearing Max: ~680 x 180 px, Corner Radius ~22 px     ]
                                |
t = 1.0 (Expanded):   +---------------------------------------------------------------+
                      | Expanded Panel: 740 x 212 px, Corner Radius ~24 px squircle   |
                      +---------------------------------------------------------------+
```

- **Mekanisme Grafis yang Diindikasikan (`[INFERRED]`):**
  - Nusa **tidak melakukan peregangan bitmap (*bitmap stretching/scaling*)**, karena penskalaan bitmap akan menyebabkan sudut membulat menjadi buram (*pixelated/aliased*).
  - Sebaliknya, Nusa menghitung ulang kurva geometri vektor `ID2D1PathGeometry` pada setiap frame interpolasi menggunakan formula kurva Bézier kubik yang di-render langsung oleh GPU Direct2D.
  - Koordinat radius sudut bergeser secara proporsional menuju radius continuous squircle ~24 px.

### B. Preset Kecepatan Animasi (`fast_animation`) `[VERIFIED STATIC]`

Biner `nusa.exe` memuat konfigurasi variabel `fast_animation` (`0x000E0528`) pada `settings.txt` dengan dua nilai teramati:
1. **Preset Normal (`fast_animation = 0`):**
   - Estimasi Durasi: **~220 – 260 ms** (`[ESTIMATED / INFERRED]`).
   - Karakter: Memberikan kesan fluiditas organik yang santai, cocok untuk monitor standar 60 Hz.
2. **Preset Fast (`fast_animation = 1`):**
   - Estimasi Durasi: **~120 – 140 ms** (`[ESTIMATED / INFERRED]`).
   - Karakter: Sangat tajam dan responsif, ditujukan bagi pengguna yang mengutamakan kecepatan interaksi tinggi atau monitor refresh rate tinggi (120 Hz / 144 Hz).

### C. Analisis Kurva Easing: Karakteristik "Spring-Like" vs Matematis Assembly

- **Observasi Visual (`[VERIFIED VISUAL]`):**
  Pergerakan ekspansi panel menunjukkan akselerasi awal yang cepat diikuti perlambatan yang sangat halus saat mendekati batas 740 × 212 px, dengan efek pantulan mikro (*micro-settle / overshoot damp*) yang hampir tak kasat mata (~1–2 px).
- **Kualifikasi Bukti Implementasi (`[INFERRED]`):**
  - Kami **tidak mengklaim** bahwa Nusa menggunakan physics engine seperti Box2D atau library React Spring / Framer Motion, karena biner Nusa adalah native Windows murni tanpa JavaScript engine.
  - Implementasi matematis pada level kode kemungkinan besar berupa rumus interpolasi polinomial teredam (*damped harmonic oscillator* atau kurva Bezier standar Direct2D seperti `cubic-bezier(0.16, 1.0, 0.3, 1.0)`).
  - Nilai numerik eksak parameter kekakuan (*stiffness*) dan redaman (*damping*) pada level kode sumber asli tetap berstatus **`[UNKNOWN]`**.

---

## 4. DWM Backdrop & Blur Transition Synchronization

Ketika opsi `glass=1` diaktifkan:
- **Transisi Blur:**
  - Jendela `NotchWindow` memanggil API Win32 DWM `DwmSetWindowAttribute` dengan kebijakan sistem backdrop blur.
  - Transisi buram diatur oleh kompositor DWM Windows.
  - Ketika panel mulai berekspansi dari kapsul kecil, intensitas blur di latar belakang meningkat secara serempak dengan perluasan kanvas visual, menciptakan efek kedalaman dimensional (*depth elevation*).
- **Keunggulan Bendera `WS_EX_NOREDIRECTIONBITMAP` (`[VERIFIED RUNTIME / MEASURED STYLE]`):**
  - Nilai extended style terukur `0x08200088` mencakup flag `WS_EX_NOREDIRECTIONBITMAP` (`0x00200000`).
  - Flag ini menonaktifkan alokasi GDI redirection bitmap standar oleh DWM, mengalihkan komposisi visual langsung ke swapchain DirectComposition untuk performa tinggi tanpa tearing dan tanpa flashing hitam.

---

## 5. Screenshot Evidence Limitations (PrintWindow vs Real-Time DWM Composition)

Sesuai arahan tata kelola rekayasa (TASK-02-03B), penyelidikan visual mendokumentasikan secara transparan **keterbatasan teknis metode tangkapan layar `PrintWindow`**:

```text
+-----------------------------------------------------------------------------------------+
| METODE PENANGKAPAN: Win32 PrintWindow(hwnd, hdc, PW_RENDERFULLCONTENT)                  |
+-----------------------------------------------------------------------------------------+
| 1. KARAKTERISTIK VISUAL YANG TERTANGKAP SECARA NYATA [VERIFIED VISUAL]:                 |
|    - Geometri bitmap kanvas jendela (NotchWindow: 740x212 px; NusaSettings: 654x556 px). |
|    - Komponen grafis diskrit kapsul idle (Circular Progress Ring hijau #21FF54).        |
|    - Tipografi digital putih Segoe UI (teks jam dan status numerik).                     |
|    - Palet warna solid dark surface (#151517, #1C1C1E, #28282A pada NusaSettings).       |
+-----------------------------------------------------------------------------------------+
| 2. KARAKTERISTIK YANG TIDAK TERTANGKAP ATAU TERDISTORSI OLEH PRINTWINDOW:                |
|    - Efek blur akrilik latar belakang (DWM Acrylic Backdrop Blur) TIDAK TERTANGKAP      |
|      secara sempurna karena DWM melakukan komposisi blur pada hardware GPU display       |
|      plane, bukan pada GDI Device Context (DC) buffer yang dibaca PrintWindow.          |
|    - Transparansi per-piksel alpha layering kanvas kosong (area non-kapsul pada kanvas   |
|      740x212 px tampil sebagai area hitam/kosong dalam DC bitmap).                      |
|    - Dinamika temporal frame per-frame dari transisi animasi spring (PrintWindow hanya   |
|      menangkap satu frame statis diskrit pada saat pemanggilan).                        |
+-----------------------------------------------------------------------------------------+
| 3. KLAIM RUNTIME YANG MEMERLUKAN METODE OBSERVASI LAIN:                                  |
|    - Bukti bahwa pipeline DirectComposition aktif: Dibuktikan melalui import table       |
|      (dcomp.dll, d3d11.dll) dan extended style WS_EX_NOREDIRECTIONBITMAP, BUKAN semata-  |
|      mata dari file gambar PNG screenshot.                                              |
|    - Bukti fluiditas 60/120 FPS: Memerlukan frame profiler GPU eksternal (PIX / PresentMon).|
+-----------------------------------------------------------------------------------------+
```

> [!WARNING]
> Tangkapan layar `PrintWindow` **tidak membuktikan bahwa seluruh pipeline rendering aktif secara penuh**; tangkapan layar hanya membuktikan kondisi frame visual antarmuka pada instan waktu tertentu dan harus diverifikasi silang dengan import API dan pengukuran jendela.

---

## 6. Strategic Motion Adaptation Blueprint for Hime Desktop

Tabel berikut menyajikan keputusan desain strategis untuk adaptasi motion Nusa ke dalam ekosistem **Hime Desktop (Tauri 2.0 / Webview2)**:

| Pola Animasi Nusa | Observasi Nusa (Direct2D Native) | Rekomendasi Adaptasi Desain Hime Desktop | Spesifikasi Web & CSS Hime |
|:---|:---|:---|:---|
| **Expand / Collapse** | Rekalkulasi kurva Direct2D per-frame | **GPU-Accelerated CSS Transitions & Web Animations API** | `transition: width 240ms cubic-bezier(0.16, 1, 0.3, 1), height 240ms cubic-bezier(0.16, 1, 0.3, 1);` |
| **Geometri Squircle** | `ID2D1PathGeometry` dengan Bézier | CSS `border-radius: 24px` dengan kurvatur halus (SVG clip-path / smooth squircle) | CSS `border-radius: 24px;` |
| **Acrylic Backdrop Blur**| DirectComposition + Win32 DWM | **CSS `backdrop-filter` + Windows DWM Acrylic via Rust Shell** | `backdrop-filter: blur(24px) saturate(180%);` + crate `windows-rs` DWM backdrop |
| **Micro-Peek Hover** | State flag `peek` di biner | CSS `:hover` transform scale / margin | `transform: scale(1.04); transition: transform 140ms ease-out;` |
| **Page Slide Crossfade** | Translasi $X$ Direct2D | Web Animations API / Framer Motion `<AnimatePresence>` | Slide horizontal: `translateX(-100%)` $\rightarrow$ `translateX(0)` dengan fade |
| **Pilihan Kecepatan** | Variabel `fast_animation=1\|0` | Variabel CSS kustom deklaratif di root | `:root { --anim-duration: 240ms; }` vs `--anim-duration: 120ms;` |
