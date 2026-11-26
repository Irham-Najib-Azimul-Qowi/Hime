# Nusa UI Component Inventory & Visual Structure Analysis

- **Document ID:** NUSA-UI-COMPONENT-INVENTORY
- **Task ID:** TASK-02-03 / TASK-02-03B (Phase 02 — Nusa Reverse Engineering)
- **Target Application:** Nusa Dynamic Island for Windows (v1.2.0.0 by Rama)
- **Authority:** Senior UI/UX Reverse Engineer & Design Systems Specialist
- **Status:** **CALIBRATED UX SPECIFICATION**
- **Date:** 2026-11-08

---

## 1. Executive Summary

Dokumen ini menyajikan inventarisasi lengkap komponen antarmuka pengguna (*User Interface*), struktur visual, hierarki kontrol, dan status kemunculan dari **Nusa Dynamic Island v1.2.0.0**.

Sesuai arahan kalibrasi bukti rekayasa balik (TASK-02-03B), dokumen ini menerapkan diferensiasi ketat antara:
1. **Komponen yang Diamati & Diukur Langsung pada Runtime (`[VERIFIED RUNTIME / MEASURED]`):** Struktur jendela Win32 yang aktif, koordinat numerik empiris, dan frame visual yang berhasil ditangkap.
2. **Karakteristik Visual yang Tertangkap Tangkapan Layar (`[VERIFIED VISUAL]`):** Elemen visual yang terverifikasi dalam bitmap hasil capture.
3. **Komponen yang Disimpulkan dari Artefak Statis (`[VERIFIED STATIC — STRINGS/IAT]` / `[INFERRED]`):** Komponen kartu sub-view (Media Player, Synced Lyrics, Today, Widgets, To-Do, Water Tracker, File Shelf, Clipboard Drawer) yang diidentifikasi melalui string biner `.rdata`, konfigurasi, dan import table Win32/WinRT, namun **tidak diamati secara langsung dalam sesi pemutaran media / interaksi penuh pada pengujian runtime terbatas**.

Nusa mengorganisasikan antarmukanya ke dalam tiga lapisan struktural utama:
1. **Overlay Jendela Mengapung Utama (`NotchWindow`):** Kanvas DirectComposition borderless berukuran terukur 740 × 212 px di batas atas layar yang menampung Dynamic Island.
2. **Jendela Pengaturan Mandiri (`NusaSettings`):** Jendela frameless berukuran terukur 654 × 556 px untuk konfigurasi sistem dan preferensi pengguna.
3. **Komponen Baki Sistem (*System Tray Area*):** Ikon baki Windows dengan menu konteks Win32 untuk kontrol latar belakang.

---

## 2. High-Level UI Structural Architecture & Provenance Tree

Struktur antarmuka Nusa tersusun atas hierarki komponen berikut beserta sumber bukti (*evidence provenance*):

```text
NUSA APPLICATION UI TREE & EVIDENCE PROVENANCE
├── NotchWindow (HWND Class: "NotchWindow", ExStyle: 0x08200088, Canvas: 740x212 px) [VERIFIED RUNTIME / MEASURED]
│   ├── Minimal Idle Capsule (Compact Pill ~84-240 x 18-36 px; Bounding Box: 84x18 px) [VERIFIED RUNTIME / MEASURED]
│   │   ├── Circular Progress Ring (Water / Battery gauge in #21FF54) [VERIFIED VISUAL via PrintWindow]
│   │   ├── Digital Status Typography (Time in Segoe UI White) [VERIFIED VISUAL via PrintWindow]
│   │   └── Interactive Mascot Slot (Pixel-art companion slot) [VERIFIED STATIC — String "Nusa Mascot"]
│   │
│   ├── Expanded Panel Container (Squircle Card max 740x212 px) [VERIFIED RUNTIME / MEASURED CANVAS BOUNDS]
│   │   ├── Acrylic Frosted Glass Backdrop (DWM backdrop blur) [VERIFIED STATIC — API/Strings] / [INFERRED VISUAL]
│   │   ├── Active Feature Card Surface [INFERRED ARCHITECTURAL CONTAINER]
│   │   │   ├── Media Player View (Cover Art, Title, Artist, Seekbar, Controls) [VERIFIED STATIC — Strings/WinRT] / [NOT OBSERVED IN ACTIVE PLAYBACK RUNTIME]
│   │   │   ├── Synchronized Lyrics View (LRCLIB streamed typography pane) [VERIFIED STATIC — String URL/Regex] / [NOT OBSERVED IN ACTIVE RUNTIME]
│   │   │   ├── Today Dashboard (Calendar, Next Prayer, Tasks Done) [VERIFIED STATIC — Strings] / [NOT OBSERVED IN ACTIVE RUNTIME]
│   │   │   ├── Dual Widget Slot (Pomodoro, World Clock, Moon, Countdown) [VERIFIED STATIC — Strings] / [NOT OBSERVED IN ACTIVE RUNTIME]
│   │   │   ├── To-Do Checklist View (Task rows, Checkboxes, Add modal) [VERIFIED STATIC — Strings/tasks.txt] / [NOT OBSERVED IN ACTIVE RUNTIME]
│   │   │   ├── Water Hydration Tracker (Bottle visual, Drank/Refill/Snooze) [VERIFIED STATIC — Strings/water.txt] / [NOT OBSERVED IN ACTIVE RUNTIME]
│   │   │   ├── File Shelf Staging Grid (Drag-and-drop file tiles) [VERIFIED STATIC — Strings/IAT OLE] / [NOT TESTED INTERACTIVELY AT RUNTIME]
│   │   │   └── Clipboard History Drawer (Recent text snippet cards) [VERIFIED STATIC — Strings/clipboard.txt] / [NOT OBSERVED IN ACTIVE RUNTIME]
│   │   │
│   │   └── Page Navigation Dots (Horizontal pagination dots) [VERIFIED STATIC — String "page_icons"] / [INFERRED LAYOUT]
│   │
│   └── Transient Hardware HUD Overlay (Temporary notification pills) [VERIFIED RUNTIME — Volume] / [VERIFIED STATIC — HUD Strings]
│       ├── Audio Volume HUD Pill [VERIFIED RUNTIME — CoreAudio Callback]
│       ├── Screen Brightness HUD Pill [VERIFIED STATIC — String "hud_brightness"]
│       ├── Caps Lock Indicator Pill [VERIFIED STATIC — String "hud_caps"]
│       ├── Input Language Switcher HUD [VERIFIED STATIC — String "hud_lang"]
│       └── Power / Battery Warning Alert Pill [VERIFIED STATIC — Strings "Low battery", "Fully charged"]
│
├── NusaSettings Window (HWND Class: "NusaSettings", Dimensions: 654x556 px) [VERIFIED RUNTIME / MEASURED]
│   ├── Left Navigation Sidebar (~180 px, Deep Dark #151517) [VERIFIED RUNTIME / MEASURED]
│   │   └── 11 Configuration Tabs (Appearance, Behavior, Sounds, Music, Widgets, Water, Prayer, Shelf, HUD, Display, About)
│   └── Main Settings Content Surface (~474 px, Dark Gray #1C1C1E / #28282A) [VERIFIED RUNTIME / MEASURED]
│       ├── Section Headers & Description Typography
│       ├── Grouped Option Cards
│       ├── Toggle Switches & Radio Button Groups
│       └── Value Pickers (Scale dropdown, Prayer city selection)
│
└── Windows System Tray Integration (Shell_NotifyIconW) [VERIFIED RUNTIME — Tray Icon Active]
    ├── Dynamic Tray Notification Icon (16x16 / 32x32) [VERIFIED RUNTIME]
    └── Win32 Context Menu (Settings, Autostart, Mascot, Minimal notch, Size, Display, About, Exit) [VERIFIED STATIC — Strings .rdata]
```

---

## 3. Detailed Component Inventory Breakdown

### Komponen 01: Minimal Idle Capsule (Compact Notch Pill)
- **Fungsi:** Menyajikan status ambient sistem yang ringkas tanpa mengganggu konsentrasi pengguna di desktop.
- **Posisi & Layout:** Mengapung di tengah atas layar monitor utama (`Top = 0` pada mode Classic, atau `Top = 8–12 px` pada mode Floating).
- **Dimensi:** Lebar ~84 – 240 px, Tinggi 18 – 36 px (pengukuran visual idle awal menunjukkan bounding box non-black berukuran 84 × 18 px).
- **Kondisi Kemunculan:** Default state ketika aplikasi berjalan dan tidak ada interaksi kursor atau pemutaran media penuh.
- **Interaksi:** Kursor diarahkan (*hover*) atau diklik (*left click*) untuk memicu ekspansi panel.
- **Perubahan Visual:** Membesar sedikit (*peek animation*) sebelum berekspansi menjadi panel penuh.
- **Hubungan Komponen:** Kontainer induk bagi Circular Progress Ring dan Digital Status Typography.
- **Sumber Bukti (Evidence Source):** Win32 HWND `NotchWindow`, `GetWindowRect`, analisis bitmap `PrintWindow`.
- **Status Observasi (Observation Status):** Diamati langsung dan diukur secara numerik pada sesi runtime aktif.
- **Klasifikasi Bukti:** `[VERIFIED RUNTIME / MEASURED]`.

### Komponen 02: Circular Progress Ring (Hydration / Battery Gauge)
- **Fungsi:** Visualisasi sekilas persentase ketercapaian target air minum harian atau kapasitas sisa baterai laptop.
- **Posisi & Layout:** Sisi kiri kapsul idle, diameter ~16 px (koordinat vertikal Y=6 hingga 22 px pada kanvas).
- **Kondisi Kemunculan:** Selalu terlihat di dalam kapsul idle mode standar.
- **Interaksi:** Tidak ada aksi klik langsung terpisah; mewarisi klik kapsul.
- **Perubahan Visual:** Busur lingkaran terisi searah jarum jam dengan warna hijau cerah `#21FF54` (*Apple-style vibrant green*).
- **Sumber Bukti (Evidence Source):** Bitmap tangkapan layar `evidence/screenshots/notch_idle_cropped.png`, analisis piksel RGBA.
- **Status Observasi (Observation Status):** Tertangkap secara langsung pada tangkapan visual frame idle.
- **Klasifikasi Bukti:** `[VERIFIED VISUAL]`.

### Komponen 03: Digital Status Typography (Time / Gauge Label)
- **Fungsi:** Menampilkan jam digital desktop atau persentase angka status.
- **Posisi & Layout:** Di sebelah kanan Circular Progress Ring di dalam kapsul idle, margin kiri ~6 px dari ring.
- **Kondisi Kemunculan:** Selalu terlihat saat idle.
- **Interaksi:** Mewarisi interaksi kapsul idle.
- **Perubahan Visual:** Teks font `Segoe UI` warna putih bersih `#FFFFFF` dengan rendering anti-aliasing tajam.
- **Sumber Bukti (Evidence Source):** Bitmap tangkapan layar `evidence/screenshots/notch_idle_cropped.png`.
- **Status Observasi (Observation Status):** Tertangkap secara langsung pada tangkapan visual frame idle.
- **Klasifikasi Bukti:** `[VERIFIED VISUAL]`.

### Komponen 04: Expanded Panel Container (Dynamic Island Canvas)
- **Fungsi:** Kanvas utama penampil fitur interaktif penuh (media, to-do, widget, shelf).
- **Posisi & Layout:** Mengisi kanvas transparan `NotchWindow` dari titik tengah atas, berekspansi ke bawah hingga batas maksimum 740 × 212 px.
- **Kondisi Kemunculan:** Terpicu saat kursor hover di atas kapsul (mode `open_click=0`) atau diklik (mode `open_click=1`).
- **Interaksi:** Menerima klik kontrol, drag-and-drop berkas, klik navigasi dots, dan klik outside untuk menutup.
- **Perubahan Visual:** Panel kartu squircle hitam pekat (`#0C0C0C`, opacity ~95%) dengan sudut membulat continuous curvature (radius ~24 px).
- **Sumber Bukti (Evidence Source):** Win32 HWND `NotchWindow` style `0x94000000`, `GetWindowRect` (Right-Left=740, Bottom-Top=212).
- **Status Observasi (Observation Status):** Batas dimensi jendela kanvas diukur secara empiris pada runtime; bentuk visual panel squircle teramati saat ekspansi.
- **Klasifikasi Bukti:** `[VERIFIED RUNTIME / MEASURED]`.

### Komponen 05: Frosted Glass Backdrop (Acrylic DWM Blur)
- **Fungsi:** Memisahkan kanvas Dynamic Island yang terbuka dari jendela aplikasi desktop di belakangnya agar konten tetap terbaca jelas.
- **Posisi & Layout:** Berada tepat di belakang geometri Expanded Panel.
- **Kondisi Kemunculan:** Hanya aktif saat island terbuka penuh dan konfigurasi `glass=1` diaktifkan.
- **Interaksi:** Pasif.
- **Perubahan Visual:** Wallpaper desktop dan teks jendela latar belakang menjadi buram lembut (*gaussian acrylic blur*).
- **Sumber Bukti (Evidence Source):** Import API Win32 DWM (`DwmSetWindowAttribute`), string konfigurasi `glass` pada `.rdata`.
- **Status Observasi (Observation Status):** Efek blur akrilik tidak dapat ditangkap secara sempurna melalui GDI `PrintWindow` karena keterbatasan capture terhadap komposisi DWM hardware; keberadaan mekanisme dibuktikan via API dan konfigurasi.
- **Klasifikasi Bukti:** `[VERIFIED STATIC — API/STRINGS]` / `[INFERRED VISUAL]`.

### Komponen 06: Media Player Card
- **Fungsi:** Mengontrol pemutaran musik/video yang sedang berlangsung di Windows.
- **Posisi & Layout:** Menempati area utama Expanded Panel (halaman 1).
- **Kondisi Kemunculan:** Terpicu saat GSMTC mendeteksi pemutar media aktif, atau saat tab Media dibuka.
- **Elemen Sub-Komponen yang Teridentifikasi dari Artefak:**
  - Thumbnail Artwork Album (persegi membulat ~48 × 48 px di sisi kiri).
  - Teks Judul Lagu (font tebal putih).
  - Teks Nama Artis & Album (font abu-abu muted `#8E8E93`).
  - Progress Slider linimasa musik (garis horizontal dengan bulatan scrubber).
  - Tombol Kontrol: Previous, Play/Pause, Next.
  - Tombol Toggle Lyrics (ikon mikrofon/lirik).
- **Interaksi:** Klik tombol playback, geser slider waktu, klik tombol lirik.
- **Sumber Bukti (Evidence Source):** String biner `.rdata` (`lyrics`, `media_view`, `prev`, `next`, `play`), impor WinRT GSMTC (`Windows.Media.Control.GlobalSystemMediaTransportControlsSessionManager`).
- **Status Observasi (Observation Status):** **TIDAK DIAMATI PADA RUNTIME AKTIF PEMUTARAN MEDIA** (ditemukan melalui audit string biner dan impor WinRT GSMTC; struktur layout visual merupakan inferensi teknis).
- **Klasifikasi Bukti:** `[VERIFIED STATIC — STRINGS/WINRT]` / `[INFERRED LAYOUT]`.

### Komponen 07: Synchronized Lyrics View
- **Fungsi:** Menampilkan lirik lagu berjalan secara real-time yang diambil dari database cloud LRCLIB.
- **Posisi & Layout:** Menggantikan tampilan kontrol media saat tombol lirik ditekan.
- **Kondisi Kemunculan:** Ketika tombol lirik aktif dan track metadata ditemukan di `lrclib.net`.
- **Interaksi:** Scroll vertikal untuk menelusuri lirik; klik baris lirik untuk melompat (*scrub*) ke timestamp tersebut.
- **Perubahan Visual:** Baris lirik aktif disorot dengan warna putih tebal membesar, sementara baris non-aktif berwarna abu-abu redup.
- **Sumber Bukti (Evidence Source):** String URL `https://lrclib.net/api/search?track_name=`, regex timestamp `\[\d{2}:\d{2}\.\d{2}\]`, dan string `.rdata`.
- **Status Observasi (Observation Status):** **TIDAK DIAMATI PADA RUNTIME AKTIF** (ditemukan melalui URL REST API dan regex parser pada `.rdata`; interaksi scrolling merupakan inferensi fungsional).
- **Klasifikasi Bukti:** `[VERIFIED STATIC — STRINGS]` / `[INFERRED LAYOUT]`.

### Komponen 08: Today Dashboard Card
- **Fungsi:** Ringkasan informasi harian pengguna (agenda hari ini, waktu shalat berikutnya, tugas selesai).
- **Posisi & Layout:** Menempati halaman 2 Expanded Panel.
- **Kondisi Kemunculan:** Dipilih via page dots navigasi.
- **Elemen Sub-Komponen yang Teridentifikasi dari Artefak:**
  - Mini Kalender & Tanggal harian.
  - Kartu "Next Prayer" (Jadwal shalat Kemenag RI dengan hitung mundur jam:menit).
  - Indikator "Tasks Done Today" (Ringkasan jumlah tugas to-do yang telah dicentang).
- **Sumber Bukti (Evidence Source):** String biner `.rdata` (`Next prayer`, `Tasks done today`, `today_view`, endpoint `api.myquran.com`).
- **Status Observasi (Observation Status):** **TIDAK DIAMATI PADA RUNTIME AKTIF** (keberadaan fitur dibuktikan dari string biner; tata letak visual merupakan inferensi).
- **Klasifikasi Bukti:** `[VERIFIED STATIC — STRINGS]` / `[INFERRED LAYOUT]`.

### Komponen 09: Dual Widget Slot (Pomodoro, World Clock, Moon, Countdown)
- **Fungsi:** Menampilkan dua widget pilihan pengguna secara berdampingan dalam satu tampilan panel.
- **Posisi & Layout:** Menempati halaman 3 Expanded Panel, terbagi menjadi dua kolom kartu simetris.
- **Pilihan Widget Tersedia:** Pomodoro Focus Timer, World Clock, Moon Phase, Countdown Timer.
- **Interaksi:** Klik tombol Start/Pause pada Pomodoro, klik tombol reset, klik konfigurasi zona waktu.
- **Sumber Bukti (Evidence Source):** String biner `.rdata` (`World clock`, `Moon`, `Countdown`, `Focus today`, `widget_view`, `pomodoro`).
- **Status Observasi (Observation Status):** **TIDAK DIAMATI PADA RUNTIME AKTIF** (ditemukan dari string identifikasi widget pada biner; tata letak dual-slot disimpulkan dari struktur string).
- **Klasifikasi Bukti:** `[VERIFIED STATIC — STRINGS]` / `[INFERRED LAYOUT]`.

### Komponen 10: To-Do Checklist View
- **Fungsi:** Pengelolaan daftar checklist tugas harian ringkas.
- **Posisi & Layout:** Menempati halaman 4 Expanded Panel.
- **Kondisi Kemunculan:** Dipilih via page dots navigasi.
- **Elemen Sub-Komponen yang Teridentifikasi dari Artefak:**
  - Daftar baris tugas dengan kotak centang (*checkbox*).
  - Tombol "Add Task" (`+`).
  - Dialog pop-up input judul tugas ("Task title", Save, Cancel).
- **Interaksi:** Klik kotak centang untuk menandai tugas selesai (memicu efek suara chime via `PlaySoundW`), klik untuk mengedit/menghapus tugas.
- **Sumber Bukti (Evidence Source):** String biner `.rdata` (`Add Task`, `Edit Task`, `TodoList`, `tasks.txt`), impor WinMM `PlaySoundW`.
- **Status Observasi (Observation Status):** **TIDAK DIAMATI PADA RUNTIME AKTIF** (keberadaan form modal dan daftar tugas dibuktikan dari string teks UI dan file path; perilaku audio terbukti via import table).
- **Klasifikasi Bukti:** `[VERIFIED STATIC — STRINGS/FILES]` / `[INFERRED LAYOUT]`.

### Komponen 11: Water Hydration Tracker Card
- **Fungsi:** Pemantauan dan pencatatan asupan air minum pengguna sepanjang hari kerja.
- **Posisi & Layout:** Menempati halaman 5 Expanded Panel.
- **Elemen Sub-Komponen yang Teridentifikasi dari Artefak:**
  - Grafik botol minum dengan level air bertahap.
  - Teks volume terakumulasi vs target (misal: "1.2 L / 2.0 L").
  - Tombol "Drank" (Mencatat satu takaran minum).
  - Tombol "Refilled" (Mencatat pengisian ulang botol).
  - Tombol "Snooze" (Menunda pengingat hidrasi selama 5–15 menit).
- **Sumber Bukti (Evidence Source):** String biner `.rdata` (`water.txt`, `Drank`, `Refilled`, `Snooze`, `hydration`).
- **Status Observasi (Observation Status):** **TIDAK DIAMATI PADA RUNTIME AKTIF** (keberadaan tombol dan skema tracking air diverifikasi dari string dan file state; elemen visual botol disimpulkan dari dokumen referensi).
- **Klasifikasi Bukti:** `[VERIFIED STATIC — STRINGS/FILES]` / `[INFERRED LAYOUT]`.

### Komponen 12: File Shelf Staging Grid
- **Fungsi:** Area penampungan berkas sementara hasil penyeretan (*drag-and-drop*) dari File Explorer atau browser.
- **Posisi & Layout:** Menempati halaman 6 Expanded Panel.
- **Kondisi Kemunculan:** Dipilih via page dots navigasi atau otomatis terbuka saat kursor yang membawa berkas diarahkan ke atas notch.
- **Elemen Sub-Komponen yang Teridentifikasi dari Artefak:**
  - Grid ubin berkas (*file tiles*) berisi ikon tipe dokumen dan nama berkas ringkas.
  - Teks petunjuk ("Release to add", "Click to open or copy").
  - Tombol pembersihan berkas ("Clear Shelf").
- **Interaksi:** Menyeret berkas masuk ke kanvas, mengklik berkas untuk membuka/menyalin, menyeret berkas keluar ke aplikasi tujuan.
- **Sumber Bukti (Evidence Source):** String biner `.rdata` (`shelf.txt`, `Release to add`, `Clear Shelf`), impor Win32 OLE COM (`RegisterDragDrop`, `DoDragDrop`, `DragQueryFileW`).
- **Status Observasi (Observation Status):** **MEKANISME TERVERIFIKASI STATIS, TIDAK DIUJI SECARA INTERAKTIF PADA RUNTIME** (impor API OLE membuktikan dukungan drag-drop; penulisan berkas ke `%LOCALAPPDATA%\Nusa\Shelf\` disimpulkan dari string path dan API).
- **Klasifikasi Bukti:** `[VERIFIED STATIC — STRINGS/IAT]` / `[INFERRED LAYOUT]`.

### Komponen 13: Clipboard History Drawer
- **Fungsi:** Menampilkan riwayat teks cuplikan yang baru saja disalin pengguna di Windows.
- **Posisi & Layout:** Bersebelahan atau terintegrasi di dalam tab Shelf.
- **Kondisi Kemunculan:** Ketika fitur clipboard diaktifkan pada pengaturan.
- **Elemen Sub-Komponen yang Teridentifikasi dari Artefak:** Kartu cuplikan teks (5 item versi gratis, 50 item versi Pro).
- **Interaksi:** Klik salah satu kartu untuk langsung menyalin kembali teks tersebut ke clipboard aktif sistem.
- **Sumber Bukti (Evidence Source):** String biner `.rdata` (`clipboard.txt`, `Clipboard history`), impor Win32 Clipboard (`GetClipboardData`, `SetClipboardData`).
- **Status Observasi (Observation Status):** **TIDAK DIAMATI PADA RUNTIME AKTIF** (fitur dibuktikan dari string dan file persistensi; limitasi 5 vs 50 item diverifikasi dari string biner).
- **Klasifikasi Bukti:** `[VERIFIED STATIC — STRINGS/IAT]` / `[INFERRED LAYOUT]`.

### Komponen 14: Transient Hardware HUD Indicator Pills
- **Fungsi:** Menggantikan tampilan OSD hardware Windows standar dengan visual pill Dynamic Island yang elegan.
- **Jenis HUD yang Teridentifikasi:**
  1. *Volume HUD Pill:* Menampilkan ikon speaker dan progress bar volume master audio saat tombol Fn Volume ditekan.
  2. *Brightness HUD Pill:* Menampilkan ikon matahari dan progress bar kecerahan monitor.
  3. *Caps Lock Indicator Pill:* Menampilkan status Caps Lock ON / OFF saat tombol Caps Lock ditekan.
  4. *Input Language Switcher Pill:* Menampilkan kode bahasa keyboard aktif (misal "ENG", "INA") saat tombol pergantian bahasa ditekan.
  5. *Battery Status Pill:* Peringatan pop-up ("Low battery", "Battery critical", "Fully charged").
- **Posisi & Layout:** Kapsul mengapung di tengah atas layar, durasi tampil ~2–3 detik sebelum menghilang otomatis.
- **Sumber Bukti (Evidence Source):** CoreAudio API `IAudioEndpointVolumeCallback` (Volume HUD), string biner `.rdata` (`hud_volume`, `hud_caps`, `hud_brightness`, `hud_lang`, `Low battery`).
- **Status Observasi (Observation Status):** Volume HUD terbukti aktif saat runtime; Brightness, Caps Lock, dan Battery disimpulkan dari string biner.
- **Klasifikasi Bukti:** `[VERIFIED RUNTIME — VOLUME]` / `[VERIFIED STATIC — STRINGS]`.

### Komponen 15: Page Navigation Dots
- **Fungsi:** Mengindikasikan posisi halaman fitur aktif dan menyediakan titik navigasi manual.
- **Posisi & Layout:** Terletak di bagian bawah Expanded Panel, sejajar secara horizontal di tengah.
- **Kondisi Kemunculan:** Selalu terlihat saat Expanded Panel terbuka.
- **Interaksi:** Klik titik dot untuk berpindah halaman seketika. Hover di atas dot memunculkan ikon pratinjau mini fitur bersangkutan (`page_icons`).
- **Sumber Bukti (Evidence Source):** String biner `.rdata` (`page_icons`, `open_notch`).
- **Status Observasi (Observation Status):** Disimpulkan dari konfigurasi string dan dokumen fitur; posisi horizontal disimpulkan dari konvensi visual Dynamic Island.
- **Klasifikasi Bukti:** `[VERIFIED STATIC — STRINGS]` / `[INFERRED LAYOUT]`.

### Komponen 16: Settings Window (`NusaSettings`)
- **Fungsi:** Panel kendali konfigurasi menyeluruh terhadap perilaku, tampilan, suara, dan integrasi Nusa.
- **Struktur Window:**
  - Dimensi Terukur: **654 px Lebar × 556 px Tinggi**.
  - Posisi Terukur: Centering layar monitor (`Left = 356, Top = 106` pada layar 1366 × 768).
  - Gaya Window: Frameless borderless popup (`WS_POPUP | WS_VISIBLE | WS_CLIPSIBLINGS | WS_MINIMIZEBOX`).
- **Struktur Kolom Antarmuka:**
  - *Sidebar Kiri (~180 px, `#151517`):* Menampung daftar 11 tab navigasi pengaturan.
  - *Konten Kanan (~474 px, `#1C1C1E`):* Menampung kartu pengaturan terkelompok (`#28282A`) dengan toggle switches, radio buttons, dan slider.
- **Sumber Bukti (Evidence Source):** Win32 HWND `NusaSettings`, `GetWindowRect`, tangkapan layar `evidence/screenshots/NusaSettings.png`.
- **Status Observasi (Observation Status):** Diamati secara langsung, diukur koordinat dan dimensinya secara numerik, dan ditangkap secara visual pada runtime.
- **Klasifikasi Bukti:** `[VERIFIED RUNTIME / MEASURED]`.

### Komponen 17: System Tray Context Menu
- **Fungsi:** Titik kendali aplikasi di latar belakang (*background control surface*).
- **Interaksi:** Klik kanan pada ikon baki Nusa memunculkan menu konteks Win32 native.
- **Daftar Item Menu Terverifikasi:**
  1. `Settings...` (Membuka jendela `NusaSettings`)
  2. `Start with Windows` (Toggle registry auto-start `HKCU`)
  3. `Nusa Mascot` (Toggle maskot pixel-art di notch)
  4. `Minimal notch` (Toggle ukuran pil minimalis idle)
  5. `Notch size` (Submenu: Auto / 100%, 115%, 130%, 150%)
  6. `Display` (Submenu pemilihan layar monitor tujuan)
  7. `About Nusa` (Dialog kredit dan versi)
  8. `Exit` (Menutup dan menterminasi proses `nusa.exe`)
- **Sumber Bukti (Evidence Source):** Impor `Shell_NotifyIconW`, string menu pada `.rdata` offset `0x000E15B8` s/d `0x000E1710`.
- **Status Observasi (Observation Status):** Ikon baki aktif teramati pada runtime; daftar item menu diverifikasi dari string biner statis.
- **Klasifikasi Bukti:** `[VERIFIED RUNTIME — TRAY ICON]` / `[VERIFIED STATIC — MENU STRINGS]`.

---

## 4. Calibrated Summary Matrix & Evidence Provenance

| ID Komponen | Nama Komponen Deskriptif | Dimensi / Batas Layout | Pemicu Kemunculan | Sumber Bukti (Evidence Source) | Status Observasi (Observation Status) | Klasifikasi Bukti |
|:---|:---|:---|:---|:---|:---|:---:|
| **CMP-01** | Minimal Idle Capsule | ~84–240 × 18–36 px | Default saat idle | HWND `NotchWindow`, `GetWindowRect` | Diamati & diukur langsung pada runtime | `[VERIFIED RUNTIME / MEASURED]` |
| **CMP-02** | Circular Progress Ring | Diameter ~16 px (Y=6..22) | Selalu terlihat di kapsul idle | Analisis bitmap `PrintWindow` | Tertangkap langsung pada tangkapan visual | `[VERIFIED VISUAL]` |
| **CMP-03** | Digital Status Typography | Font Segoe UI White | Di sebelah progress ring | Analisis bitmap `PrintWindow` | Tertangkap langsung pada tangkapan visual | `[VERIFIED VISUAL]` |
| **CMP-04** | Expanded Panel Container | Maksimum 740 × 212 px | Hover atau klik pada kapsul | HWND `NotchWindow`, `GetWindowRect` | Batas kanvas terukur numerik pada runtime | `[VERIFIED RUNTIME / MEASURED]` |
| **CMP-05** | Frosted Glass Backdrop | Mengikuti batas panel | Panel terbuka (`glass=1`) | API DWM, string biner `.rdata` | Tidak terisolasi sempurna di PrintWindow; mekanisme via API | `[VERIFIED STATIC / INFERRED]` |
| **CMP-06** | Media Player Card | Area dalam panel (Halaman 1) | Lagu diputar / Tab Media aktif | String biner `.rdata`, WinRT GSMTC | **Tidak diamati pada runtime aktif**; disimpulkan dari string & API | `[VERIFIED STATIC / INFERRED]` |
| **CMP-07** | Synchronized Lyrics View | Area dalam panel | Tombol lirik aktif & ada data LRCLIB | String URL LRCLIB, regex timestamp | **Tidak diamati pada runtime aktif**; disimpulkan dari endpoint URL | `[VERIFIED STATIC / INFERRED]` |
| **CMP-08** | Today Dashboard Card | Area dalam panel (Halaman 2) | Tab Today dipilih | String `.rdata` (`Next prayer`, dll.) | **Tidak diamati pada runtime aktif**; disimpulkan dari string biner | `[VERIFIED STATIC / INFERRED]` |
| **CMP-09** | Dual Widget Slot | Area dalam panel (Halaman 3) | Tab Widgets dipilih | String `.rdata` (`World clock`, dll.) | **Tidak diamati pada runtime aktif**; disimpulkan dari string biner | `[VERIFIED STATIC / INFERRED]` |
| **CMP-10** | To-Do Checklist View | Area dalam panel (Halaman 4) | Tab To-Do dipilih | String `.rdata` (`tasks.txt`, dll.), WinMM | **Tidak diamati pada runtime aktif**; disimpulkan dari file & strings | `[VERIFIED STATIC / INFERRED]` |
| **CMP-11** | Water Tracker Card | Area dalam panel (Halaman 5) | Tab Water dipilih | String `.rdata` (`water.txt`, `Drank`) | **Tidak diamati pada runtime aktif**; disimpulkan dari file & strings | `[VERIFIED STATIC / INFERRED]` |
| **CMP-12** | File Shelf Staging Grid | Area dalam panel (Halaman 6) | Drop berkas / Tab Shelf dipilih | String `.rdata` (`shelf.txt`), IAT OLE COM | **Tidak diuji interaktif pada runtime**; mekanisme via API & strings | `[VERIFIED STATIC / INFERRED]` |
| **CMP-13** | Clipboard History Drawer | Area dalam Shelf | Fitur clipboard aktif | String `.rdata` (`clipboard.txt`), IAT | **Tidak diamati pada runtime aktif**; disimpulkan dari file & strings | `[VERIFIED STATIC / INFERRED]` |
| **CMP-14** | Transient Hardware HUD Pills | Kapsul HUD ~180–300 × 36 px | Event hardware (Volume, Caps, dll.) | CoreAudio API, string biner `.rdata` | Volume HUD diamati pada runtime; lainnya disimpulkan dari strings | `[VERIFIED RUNTIME / STATIC]` |
| **CMP-15** | Page Navigation Dots | Baris horizontal bawah panel | Expanded panel terbuka | String biner `.rdata` (`page_icons`) | Disimpulkan dari string biner & konvensi | `[VERIFIED STATIC / INFERRED]` |
| **CMP-16** | Settings Window (`NusaSettings`)| 654 × 556 px (Centering layar)| Klik "Settings..." di tray / `--settings` | HWND `NusaSettings`, `GetWindowRect` | Diamati, diukur, & ditangkap visual runtime | `[VERIFIED RUNTIME / MEASURED]` |
| **CMP-17** | System Tray Context Menu | Standar Win32 Context Menu | Klik kanan ikon notification area | `Shell_NotifyIconW`, string menu `.rdata` | Ikon diamati pada runtime; menu strings via `.rdata` | `[VERIFIED RUNTIME / STATIC]` |
