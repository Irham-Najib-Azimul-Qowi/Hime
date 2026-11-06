# 05 — UI/UX Reverse Engineering
## Proyek: Hime Assistant
### Target: Nusa Dynamic Island

---

## 1. Filosofi Desain & Konsep Visual

Nusa mengadaptasi konsep Dynamic Island (yang dipopulerkan di perangkat seluler modern) ke dalam lingkungan desktop Windows. 

Karakteristik visual utamanya:
- **Bentuk Kapsul Organik (Continuous Pill/Capsule):** Menggunakan geometri sudut membulat tinggi (*squircle / continuous curvature*).
- **Dark Mode OLED Glass:** Dominasi warna hitam pekat (*true black / dark graphite*) dengan aksen *frosted glass* (efek kaca buram / acrylic) di lapisan belakang saat jendela berekspansi.
- **Tipografi Native Modern:** Memanfaatkan font keluarga `Segoe UI` standar Windows dengan hierarki visual yang jelas.
- **Interaksi Mikro (Micro-interactions):** Transisi animasi pegas (*spring animation*) yang halus saat transisi dari mode kapsul mini ke mode panel ekspansi.

---

## 2. Dimensi, Posisi, dan Geometri Jendela

Berdasarkan pengujian biner dan inspeksi langsung struktur kanvas DirectComposition:

```text
+-------------------------------------------------------------------------+  Top = 0 (Batas Atas Layar)
|                                                                         |
|                          [   Kapsul Idle   ]                            |  Idle: ~200 x 34 px
|                                                                         |
|            +-----------------------------------------------+            |
|            |                                               |            |
|            |             Panel Ekspansi Penuh              |            |  Expanded: Max 740 x 212 px
|            |         (Media / To-do / Shelf / dll)         |            |
|            |                                               |            |
|            +-----------------------------------------------+            |
|                                                                         |
+-------------------------------------------------------------------------+  Bottom = 212 px
<------------------------------- 740 px ---------------------------------->
```

### A. Dua Gaya Penempatan (Notch Placement Styles)
String biner pada offset `0x000E30E8` membuktikan adanya 2 pilihan gaya penempatan fisik:
1. **Classic (Joined to the top edge):**
   - Kapsul menempel langsung ke bezel atas layar (mirip notch laptop MacBook).
   - Radius sudut hanya melengkung di sisi bawah, sementara sisi atas rata sejajar garis tepi layar monitor.
2. **Floating (Floats just below the top edge):**
   - Kapsul mengapung beberapa piksel di bawah tepi atas layar (margin ~8–12 px), membentuk pil oval penuh (*pill shape*) independen (mirip Dynamic Island iPhone).

### B. Dimensi Fisik (Physical Dimensions)
- **Status Idle (Minimal Notch):**
  - Lebar: ~180 – 240 px (bergantung pada konten status, misal durasi lagu atau waktu Pomodoro).
  - Tinggi: ~32 – 36 px.
  - Border Radius: $\frac{\text{Tinggi}}{2} \approx 16\text{–}18\text{ px}$ (Full Pill Capsule).
- **Status Expanded (Panel Terbuka):**
  - Lebar: Hingga 740 px (lebar kanvas maksimum).
  - Tinggi: Hingga 212 px.
  - Border Radius: ~24 – 28 px pada keempat sudut luar panel.
- **Pilihan Skala Ukuran (Notch Size Scaling):**
  - `Auto` (Mengikuti skala DPI Windows yang dilaporkan oleh `GetDpiForMonitor`)
  - `100%` (Ukuran standar)
  - `115%` (Medium)
  - `130%` (Large)
  - `150%` (Extra Large — cocok untuk layar 4K/UHD)

---

## 3. Palet Warna & Desain Visual (Color System)

| Elemen UI | Nilai Hex / Komposisi | Keterangan |
| :--- | :--- | :--- |
| **Latar Belakang Kapsul** | `#000000` s/d `#0C0C0C` (Alpha 92% – 96%) | Hitam solid dengan sedikit transparansi untuk kesan menyatu dengan monitor. |
| **Frosted Glass Backdrop** | Acrylic DWM Blur + Glass Tint | Efek buram lembut di belakang island saat terbuka untuk memisahkan UI dari wallpaper/jendela lain. |
| **Teks Primer** | `#FFFFFF` (Solid White) | Judul lagu, waktu, nama task, angka persentase. |
| **Teks Sekunder** | `#8E8E93` – `#A0A0A0` (Muted Gray) | Nama artis, label status, tanggal, countdown. |
| **Aksen Media & Water** | `#2997FF` / `#0A84FF` (Vibrant Blue) | Indikator progress air minum dan kontrol pemutar musik. |
| **Aksen Pomodoro & Baterai Rendah** | `#FF453A` / `#FF9F0A` (Coral Red & Orange) | Status waktu fokus dan peringatan baterai laptop hampir habis. |
| **Aksen Sukses & To-do Selesai** | `#30D158` (Apple Green) | Checkbox to-do selesai, baterai terisi penuh (*Fully Charged*). |

---

## 4. Sistem Animasi & Interaksi Mikro

Struktur biner pada offset `0x000E2AA8` (`animating`, `dirty`, `peek`, `fast_animation`) mengungkap sistem state machine animasi Nusa:

1. **Trigger Interaksi:**
   - **Mode Hover:** Meletakkan kursor di atas kapsul memicu status `peek` (membesar sedikit sebagai tanda responsif) sebelum membuka panel penuh.
   - **Mode Click:** Memerlukan klik mouse kiri untuk berekspansi (`open_click` toggle di settings).
2. **Kurva Easing & Kecepatan:**
   - Mendukung dua preset: `Normal` dan `Fast`.
   - Menggunakan interpolasi kurva Direct2D (*cubic bezier / spring physics*) untuk perubahan lebar dan tinggi secara bersamaan, sehingga sudut membulat tidak mengalami distorsi selama berekspansi.
3. **Indikator Halaman (Page Dots Navigation):**
   - Saat terbuka, bagian bawah panel menampilkan titik navigasi (*page dots*).
   - Pengguna dapat beralih antar halaman fitur (Media, Today, Widgets, To-do, Water, Shelf).
   - Fitur `page_icons`: Menampilkan ikon mini fitur saat kursor diarahkan ke titik halaman bersangkutan.

---

## 5. Fitur Maskot Interaktif (`Nusa Mascot`)

Ditemukan referensi fitur unik pada string biner offset `0x000DC0D0` dan `0x000E15F6`:
- **Nama Fitur:** `Nusa Mascot` ("A little friend that lives in the notch").
- **Konsep:** Karakter/maskot virtual mini bergaya retro/pixel yang menghuni notch ketika idle. Maskot memiliki animasi tidur, berkedip, atau bereaksi ketika lagu diputar.
- **Implementasi:** Di-render langsung menggunakan bitmap Direct2D kecil atau geometri vektor prosedural.

---

## 6. Komparasi Karakteristik UI Nusa vs Rencana Hime Assistant

| Parameter Desain | Implementasi Nusa | Rekomendasi Adaptasi untuk Hime Assistant |
| :--- | :--- | :--- |
| **Bentuk Dasar** | Kapsul pil hitam di tengah atas layar | **Pertahankan.** Menjadi identitas utama UI floating island Hime. |
| **Kanvas Maksimum** | 740 × 212 px (Horizontal oriented) | **Perluas.** Hime butuh membaca respons AI, sehingga kanvas perlu mendukung ekspansi vertikal (misal hingga 740 × 480 px). |
| **Elemen Interaksi Utama** | Media controls, to-do, widget statis | **Gantikan dengan Chat & Action Interface:** Input bar teks, status eksekusi Hermes Agent, streaming bubble. |
| **Navigasi Halaman** | Dots horizontal | **Sederhanakan:** Tab mode status (Idle / Running / Result / Input). |
| **Gaya Jendela** | DirectComposition borderless popup | **Gunakan pendekatan arsitektur yang sama** untuk menjamin performa 60 FPS dan efisiensi memori. |

