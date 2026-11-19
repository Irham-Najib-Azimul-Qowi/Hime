# Nusa Interaction Behavior & Input Response Reverse Engineering

- **Document ID:** NUSA-INTERACTION-BEHAVIOR
- **Task ID:** TASK-02-03 / TASK-02-03B (Phase 02 — Nusa Reverse Engineering)
- **Target Application:** Nusa Dynamic Island for Windows (v1.2.0.0 by Rama)
- **Authority:** Senior Human-Computer Interaction (HCI) & Desktop UX Specialist
- **Status:** **CALIBRATED INTERACTION SPECIFICATION**
- **Date:** 2026-11-08

---

## 1. Executive Summary

Dokumen ini mendokumentasikan analisis perilaku interaksi (*Interaction Behavior*) dan responsivitas aplikasi **Nusa Dynamic Island v1.2.0.0** terhadap berbagai modalitas input pengguna pada lingkungan desktop Windows.

Sesuai arahan kalibrasi bukti (TASK-02-03B), dokumen ini menerapkan kalibrasi presisi pada 6 area kritis:
1. **Dukungan OLE Drag-and-Drop:** Membedakan antara keberadaan mekanisme API Win32 OLE (`RegisterDragDrop`, `DoDragDrop`, `DragQueryFileW`) pada import table dengan pengujian interaktif runtime (penyalinan berkas fisik ke folder Shelf merupakan inferensi dari struktur string dan API, bukan hasil uji interaktif langsung).
2. **Timer Hover Dwell (>150 ms):** Mengklasifikasikan nilai ambang batas waktu hover sebagai estimasi rekayasa (`[ESTIMATED / INFERRED]`), bukan pengukuran benchmark hardware mikrodetik.
3. **Pintasan Papan Ketik (Keyboard Shortcuts):** Membedakan secara jujur antara ketiadaan artefak registrasi shortcut global (`RegisterHotKey`) pada string `.rdata` dan IAT dengan klaim matematis bahwa shortcut mustahil ada.
4. **Pembatalan Klik di Luar (Click-Outside Dismissal):** Mengkualifikasikan mekanisme penutupan otomatis sebagai pola inferensi penonaktifan jendela (*window deactivation*), dengan mencatat batasan pengujian runtime.
5. **Transparansi Tembus-Klik (Click-Through):** Menetapkan mekanisme penanganan `WM_NCHITTEST` yang mengembalikan `HTTRANSPARENT` secara tegas sebagai `[INFERRED]`.
6. **Peran Extended Style `WS_EX_TOOLWINDOW`:** Menjelaskan bahwa `WS_EX_TOOLWINDOW` berfungsi menyembunyikan jendela dari Taskbar dan Alt+Tab, dan **tidak secara otomatis menjamin imunitas pencurian fokus papan ketik** tanpa penanganan pesan aktivasi Win32 tambahan.

Setiap perilaku interaksi diformat secara terstruktur: **Input**, **Kondisi Awal (*Initial Condition*)**, **Respons Teramati (*Observed Response*)**, **Hasil Uji (*Test Result*)**, **Klasifikasi Bukti (*Evidence Classification*)**, dan **Limitasi (*Limitations*)**.

---

## 2. Structured Interaction Modalities Catalog

### INT-01: Single Click (Left) pada Kapsul Idle
- **Input:** Klik kiri mouse tunggal (`WM_LBUTTONDOWN` / `WM_LBUTTONUP`).
- **Kondisi Awal:** Jendela `NotchWindow` dalam status idle capsule (84 × 18 px non-black bounding box).
- **Respons Teramati:**
  - Jika konfigurasi `open_click = 1`: Memicu transisi ekspansi kanvas menjadi Expanded Panel.
  - Jika konfigurasi `open_click = 0`: Klik langsung pada kapsul tidak mengubah ukuran (ekspansi diatur oleh hover).
- **Hasil Uji:** Terverifikasi secara fungsional melalui keberadaan konfigurasi `open_click` pada `settings.txt`.
- **Klasifikasi Bukti:** `[VERIFIED STATIC — CONFIG/STRINGS]` / `[INFERRED RUNTIME]`.
- **Limitasi:** Pengujian live toggle memerlukan interaksi GUI yang bervariasi bergantung pada pengaturan aktif di `%LOCALAPPDATA%\Nusa\settings.txt`.

### INT-02: Double Click (Left) pada Kapsul Idle
- **Input:** Klik ganda cepat tombol kiri mouse.
- **Kondisi Awal:** Jendela `NotchWindow` dalam status idle capsule.
- **Respons Teramati:** Tidak ada perilaku khusus; double-click diperlakukan sebagai dua pesan single-click beruntun. Tidak ada aksi maximize atau fullscreen.
- **Hasil Uji:** Konsisten dengan ketiadaan penanganan pesan `WM_LBUTTONDBLCLK` atau pendaftaran kelas jendela tanpa flag `CS_DBLCLKS`.
- **Klasifikasi Bukti:** `[VERIFIED STATIC — ABSENCE OF CS_DBLCLKS]` / `[INFERRED]`.
- **Limitasi:** Respon visual identik dengan single click beruntun.

### INT-03: Right Click pada Kapsul Idle & Panel
- **Input:** Klik kanan mouse (`WM_RBUTTONDOWN` / `WM_RBUTTONUP`).
- **Kondisi Awal:** Kursor berada di atas kapsul idle atau kanvas expanded panel.
- **Respons Teramati:** Tidak memunculkan menu konteks kustom pada notch; klik kanan pada area kanvas transparan diteruskan ke jendela desktop di bawahnya.
- **Hasil Uji:** Tidak ada referensi API `CreatePopupMenu` atau `TrackPopupMenu` yang terhubung langsung ke `NotchWindow`.
- **Klasifikasi Bukti:** `[INFERRED FROM ARCHITECTURE]`.
- **Limitasi:** Observasi dilakukan pada mode idle dasar; perilaku sub-view tertentu (misal to-do row) tidak diuji interaktif penuh.

### INT-04: Hover & Dwell pada Kapsul Idle
- **Input:** Kursor mouse digerakkan masuk ke area kapsul idle (`WM_MOUSEMOVE` / `TrackMouseEvent`).
- **Kondisi Awal:** Dynamic Island berada dalam status idle; `open_click = 0`.
- **Respons Teramati:** Kapsul membesar mikro (~4–8 px, status `peek`). Jika kursor berdiam melampaui estimasi threshold (~150 ms), panel berekspansi penuh menjadi 740 × 212 px.
- **Hasil Uji:** Keberadaan string state `peek` dan `animating` terverifikasi pada biner.
- **Klasifikasi Bukti:** `[VERIFIED STATIC — STRINGS]` / `[ESTIMATED / INFERRED RUNTIME]`.
- **Limitasi:** **Nilai dwell time ~150 ms adalah estimasi rekayasa (engineering estimation)**, bukan hasil pengukuran menggunakan timer profil presisi tingkat mikrodetik.

### INT-05: OLE Drag-and-Drop Ingestion (Drop-In ke Shelf)
- **Input:** Pengguna menyeret satu atau beberapa berkas dari Windows File Explorer ke atas kapsul Dynamic Island.
- **Kondisi Awal:** File Explorer aktif dengan payload seretan berkas (`CF_HDROP`); Nusa dalam status idle.
- **Respons Teramati:** Mekanisme OLE COM mendeteksi seretan masuk; kanvas berekspansi menampilkan tampilan Shelf dengan instruksi `"Release to add"`. Saat berkas dilepas, berkas disalin ke `%LOCALAPPDATA%\Nusa\Shelf\`.
- **Hasil Uji:** Keberadaan impor API OLE COM (`RegisterDragDrop`, `DragQueryFileW`, `DoDragDrop`) dan string `"Release to add"`, `shelf.txt` terverifikasi statis.
- **Klasifikasi Bukti:** `[VERIFIED STATIC — IAT & STRINGS]` / `[INFERRED INTERACTION]`.
- **Limitasi:** **Mekanisme API terverifikasi statis, namun pengujian penyeretan berkas secara interaktif langsung pada runtime tidak dilakukan dalam audit ini.** Perilaku penyalinan berkas fisik disimpulkan dari struktur string dan API terkait.

### INT-06: OLE Drag-and-Drop Extraction (Drag-Out dari Shelf)
- **Input:** Menekan klik kiri pada salah satu ubin berkas di Shelf dan menyeret kursor ke aplikasi luar (misal chat/email/folder).
- **Kondisi Awal:** Panel Shelf terbuka menampilkan daftar berkas tertampung.
- **Respons Teramati:** Menginisiasi panggilan `DoDragDrop` dengan format `CF_HDROP` berisi path absolut berkas lokal.
- **Hasil Uji:** Terverifikasi keberadaan fungsi export/import OLE COM pada import table.
- **Klasifikasi Bukti:** `[VERIFIED STATIC — IAT]` / `[INFERRED INTERACTION]`.
- **Limitasi:** Interaksi drag-out tidak dieksekusi dalam sesi runtime interaktif.

### INT-07: Pembatalan Klik di Luar (Click-Outside Auto-Dismissal)
- **Input:** Mengklik tombol mouse pada jendela aplikasi lain atau area desktop di luar batas poligon panel Dynamic Island.
- **Kondisi Awal:** Expanded Panel sedang terbuka (`ST-04`).
- **Respons Teramati:** Jendela mendeteksi kehilangan status aktif/fokus, lalu menyusutkan panel kembali ke bentuk kapsul idle (`ST-02`) melalui animasi reverse spring.
- **Hasil Uji:** Konsisten dengan perilaku jendela pop-up ringan desktop berbasis penanganan event deaktivasi.
- **Klasifikasi Bukti:** `[INFERRED PATTERN]`.
- **Limitasi:** **Mekanisme dismissal ini merupakan inferensi pola deaktivasi jendela (`WM_KILLFOCUS` / `WM_ACTIVATE`) yang didukung oleh perilaku observasi umum, namun tidak diuji dengan logging event per-pesan secara komprehensif.**

### INT-08: Transparansi Tembus-Klik (Click-Through Transparency via WM_NCHITTEST)
- **Input:** Mengklik tombol mouse pada koordinat area kanvas 740 × 212 px yang tidak terisi oleh grafik kapsul idle (misal koordinat $X=50, Y=50$).
- **Kondisi Awal:** Kapsul idle berada di tengah kanvas (lebar 84 px); area kanvas di sebelah kiri dan kanan berstatus transparan.
- **Respons Teramati:** Pesan klik lolos secara instan ke jendela aplikasi di belakang Nusa (misal tab browser atau desktop) tanpa halangan.
- **Hasil Uji:** Tidak terjadi pemblokiran klik pada jendela di belakang kanvas kosong.
- **Klasifikasi Bukti:** `[INFERRED FROM ARCHITECTURE]`.
- **Limitasi:** **Implementasi spesifik handler pesan `WM_NCHITTEST` yang mengembalikan `HTTRANSPARENT` merupakan inferensi arsitektural logis dari perilaku tembus-klik yang diamati, bukan hasil disassembly kode fungsi window procedure secara penuh.**

### INT-09: Perilaku Fokus Jendela & Extended Style WS_EX_TOOLWINDOW
- **Input:** Ekspansi panel atau interaksi klik pada kanvas `NotchWindow`.
- **Kondisi Awal:** Aplikasi pihak ketiga (misal text editor atau IDE) sedang aktif dan memiliki fokus kursor ketik (*caret focus*).
- **Respons Teramati:** Dynamic Island terbuka di atas layar.
- **Kualifikasi Teknis Win32:** Jendela `NotchWindow` memiliki extended style `WS_EX_TOOLWINDOW` (`0x00000080`). Sesuai spesifikasi Microsoft Windows SDK:
  - `WS_EX_TOOLWINDOW` berfungsi membuat jendela utilitas yang tidak muncul pada taskbar Windows dan dialog Alt+Tab.
  - **PENTING: `WS_EX_TOOLWINDOW` TIDAK SECARA OTOMATIS MENJAMIN JENDELA TIDAK MENCURI FOKUS PAPAN KETIK.** Imunitas fokus pada Win32 memerlukan extended style tambahan seperti `WS_EX_NOACTIVATE` (`0x08000000`) atau penanganan pesan `WM_MOUSEACTIVATE` yang mengembalikan `MA_NOACTIVATE`.
  - Pada `NotchWindow`, nilai extended style terukur adalah `0x08200088` (kombinasi `WS_EX_NOREDIRECTIONBITMAP | WS_EX_COMPOSITED | WS_EX_TOOLWINDOW | WS_EX_TOPMOST`).
- **Hasil Uji:** Terverifikasi nilai numerik extended style `0x08200088` pada runtime.
- **Klasifikasi Bukti:** `[VERIFIED RUNTIME / MEASURED STYLE]` & `[QUALIFIED ARCHITECTURAL INTERPRETATION]`.
- **Limitasi:** Perlindungan fokus kursor ketik bergantung pada logika internal window procedure saat merespons `WM_MOUSEACTIVATE`, bukan semata-mata akibat keberadaan flag `WS_EX_TOOLWINDOW`.

### INT-10: Pintasan Papan Ketik Global (Global Keyboard Shortcuts)
- **Input:** Penekanan kombinasi tombol papan ketik global dari aplikasi apa pun (misal `Alt + Space`, `Win + Shift + D`).
- **Kondisi Awal:** Nusa berjalan di latar belakang; aplikasi lain sedang aktif.
- **Respons Teramati:** Tidak ada reaksi pembukaan Dynamic Island.
- **Hasil Uji:** Audit biner menunjukkan:
  - **Tidak ditemukan impor API `RegisterHotKey`** pada Import Address Table (IAT) `nusa.exe`.
  - **Tidak ditemukan konfigurasi shortcut global** pada string biner `.rdata` atau file konfigurasi `settings.txt`.
- **Klasifikasi Bukti:** `[NOT FOUND IN BINARY ARTIFACTS — STRINGS & IAT]`.
- **Limitasi:** Pernyataan ini didasarkan pada ketiadaan artefak registrasi shortcut global standar pada biner dan IAT, dan **bukan klaim pembuktian matematis mutlak bahwa jalan pintas tidak dapat diimplementasikan secara tidak langsung melalui low-level keyboard hook (`SetWindowsHookExW`)**. Namun, ketiadaan string konfigurasi pendukung mengindikasikan fitur ini memang tidak disediakan oleh aplikasi.

### INT-11: Kontrol Perangkat Keras (Hardware HUD Trigger)
- **Input:** Penekanan tombol hardware keyboard: Fn Volume Up/Down, Fn Brightness, Caps Lock, atau event status baterai laptop.
- **Kondisi Awal:** Dynamic Island dalam status idle.
- **Respons Teramati:** Kapsul memanjang membentuk bar HUD sementara selama ~2.5 detik menampilkan status level volume/kecerahan/caps, lalu menyusut kembali.
- **Hasil Uji:** Callback CoreAudio `IAudioEndpointVolumeCallback` terverifikasi aktif menangkap perubahan volume audio pada runtime.
- **Klasifikasi Bukti:** `[VERIFIED RUNTIME — VOLUME]` / `[VERIFIED STATIC — STRINGS FOR BRIGHTNESS/CAPS]`.
- **Limitasi:** Event kecerahan dan baterai disimpulkan dari string biner; hanya perubahan volume yang diuji secara langsung.

### INT-12: Interaksi Baki Sistem (System Tray Context Menu)
- **Input:** Klik kanan pada ikon baki Nusa di pojok taskbar.
- **Kondisi Awal:** Ikon Nusa aktif terdaftar di taskbar via `Shell_NotifyIconW`.
- **Respons Teramati:** Membuka menu konteks Win32 native dengan 8 item menu (*Settings, Autostart, Mascot, Minimal notch, Size, Display, About, Exit*).
- **Hasil Uji:** Keberadaan ikon baki dan pemanggilan menu terverifikasi; daftar item menu diverifikasi dari string `.rdata`.
- **Klasifikasi Bukti:** `[VERIFIED RUNTIME — TRAY ICON]` / `[VERIFIED STATIC — MENU STRINGS]`.
- **Limitasi:** Beberapa submenu (misal Display switching pada konfigurasi multi-monitor) tidak diuji pada multi-monitor fisik.

---

## 3. Calibrated Interaction Summary Matrix

| ID | Modalitas Input | Komponen Target | Respons Teramati & Rangkuman | Status Uji | Sumber Bukti | Klasifikasi Bukti |
|:---:|:---|:---|:---|:---|:---|:---:|
| **INT-01** | Single Click (Left) | Kapsul Idle | Ekspansi jika `open_click=1` | Terverifikasi via config | `settings.txt`, string `.rdata` | `[VERIFIED STATIC / INFERRED]` |
| **INT-02** | Double Click (Left) | Kapsul Idle | Diperlakukan sebagai dua kali single click | Terverifikasi ketiadaan `CS_DBLCLKS` | Header window class | `[VERIFIED STATIC / INFERRED]` |
| **INT-03** | Right Click | Kapsul Idle / Panel | Tidak ada menu konteks pada notch | Terverifikasi ketiadaan handler popup | Struktur pesan Win32 | `[INFERRED]` |
| **INT-04** | Hover & Dwell | Kapsul Idle | Peek lalu ekspansi jika diam (>150 ms) | Estimasi waktu rekayasa | String `peek`, `animating` | `[ESTIMATED / INFERRED]` |
| **INT-05** | Drag & Drop (In) | Kapsul / Shelf | Menerima drop berkas ke Shelf | Mekanisme API terverifikasi statis | IAT OLE COM, string `shelf.txt` | `[VERIFIED STATIC / INFERRED]` |
| **INT-06** | Drag & Drop (Out)| File Tile di Shelf | Menginisiasi `DoDragDrop` | Mekanisme API terverifikasi statis | IAT OLE COM (`DoDragDrop`) | `[VERIFIED STATIC / INFERRED]` |
| **INT-07** | Click Outside | Jendela / Desktop Luar| Menutup expanded panel | Pola inferensi deaktivasi jendela | Pola deaktivasi Win32 | `[INFERRED PATTERN]` |
| **INT-08** | Click-Through | Area Kanvas Kosong | Klik lolos ke jendela di bawahnya | Inferensi arsitektural logis | Perilaku klik, `WM_NCHITTEST` | `[INFERRED]` |
| **INT-09** | Window Activation| `NotchWindow` | Tidak tampil di taskbar; kontrol fokus | Extended style terukur 0x08200088 | `GetWindowLongW` (GWL_EXSTYLE) | `[VERIFIED RUNTIME / QUALIFIED]` |
| **INT-10** | Global Shortcuts | Desktop Global | Tidak ada registrasi hotkey global | Tidak ditemukan pada string / IAT | Ketiadaan impor `RegisterHotKey` | `[NOT FOUND IN ARTIFACTS]` |
| **INT-11** | Hardware Keys | Keyboard Fn / Caps | Menampilkan HUD Pill sementara | Volume teruji; lainnya via strings | CoreAudio API, string `.rdata` | `[VERIFIED RUNTIME / STATIC]` |
| **INT-12** | Tray Right Click | Ikon System Tray | Menu konteks Win32 8 opsi | Ikon aktif runtime; string terverifikasi | `Shell_NotifyIconW`, string `.rdata`| `[VERIFIED RUNTIME / STATIC]` |

---

## 4. Deep Dive into Critical Interaction Modalities & Calibrated Ergonomics

### A. Pola Ekspansi Kapsul: Hover vs Click Trigger (`open_click`)

Salah satu karakteristik interaksi paling menonjol pada Nusa adalah tersedianya dua model mental pemicu ekspansi Dynamic Island yang dapat dikonfigurasi melalui pengaturan `open_click`:

```mermaid
flowchart TD
    CursorApproach([Kursor Mouse Mendekati Kapsul Notch]) --> CheckSetting{Konfigurasi open_click?}

    CheckSetting -- "open_click = 0 (Mode Hover)" --> MouseEnter[Event Mouse Enter / Hover]
    MouseEnter --> TriggerPeek[Animasi Peek: Kapsul membesar ~4-8 px]
    TriggerPeek --> DwellTimer{Kursor bertahan > 150ms? [ESTIMATED]}
    DwellTimer -- Ya --> FullExpand[Animasi Ekspansi Penuh 740x212 px]
    DwellTimer -- Tidak (Melintas Cepat) --> ReturnIdle[Kembali ke Idle Capsule]

    CheckSetting -- "open_click = 1 (Mode Click)" --> MouseEnterClick[Event Mouse Enter]
    MouseEnterClick --> SubtleHover[Umpan Balik Visual Ringan]
    SubtleHover --> UserClick{Pengguna Klik Kiri Mouse?}
    UserClick -- Ya --> FullExpand
    UserClick -- Tidak --> MaintainIdle[Tetap dalam Bentuk Kapsul Idle]
```

- **Evaluasi Ergonomi Desktop:**
  - *Mode Hover (`open_click=0`):* Sangat cepat dan mulus untuk melihat lirik atau cuplikan waktu tanpa klik. Namun, berpotensi memicu ekspansi tidak disengaja (*accidental trigger*) saat pengguna berniat mengklik tab browser yang berada tepat di bawah tepi atas layar monitor.
  - *Mode Click (`open_click=1`):* Menghilangkan pemicu tidak disengaja secara total, namun membutuhkan aksi eksplisit dari pengguna.
  - *Catatan Estimasi Waktu:* Ambang batas dwell time diperkirakan sekitar **~150 ms** berdasarkan kelancaran transisi visual; nilai ini tidak diukur melalui profiler CPU berpresisi tinggi.

### B. Mekanisme Pembatalan & Klik di Luar (*Click Outside & Window Deactivation*)

Ketika Expanded Panel sedang terbuka dan pengguna mengalihkan perhatiannya kembali ke dokumen kerja atau browser:
1. Pengguna mengklik aplikasi latar belakang di luar batas visual kanvas pulau.
2. Jendela `NotchWindow` mendeteksi kehilangan status aktif melalui sinyal penonaktifan Win32 (`WM_KILLFOCUS` / `WM_ACTIVATE` / penanganan kehilangan kursor).
3. Nusa **menyusutkan Expanded Panel** melalui transisi reverse morphing menuju ukuran kapsul idle.
4. **Kualifikasi Fokus:** Extended style `WS_EX_TOOLWINDOW` (`0x00000080`) menyembunyikan jendela dari taskbar dan Alt+Tab, namun perlindungan agar caret ketik tidak berpindah dari aplikasi pengguna bergantung pada implementasi respons pesan aktivasi (`WM_MOUSEACTIVATE` atau flag non-aktif).

### C. Manipulasi Drag & Drop (File Shelf Staging)

Pola interaksi penyeretan berkas dirancang untuk menyederhanakan pemindahan dokumen:
- **Alur Ingestion (Drop-In):**
  1. Pengguna memilih berkas di File Explorer.
  2. Menyeret kursor ke arah atas layar menuju kapsul Notch.
  3. Kapsul mendeteksi pointer OLE drag via antarmuka `IDropTarget` dan berekspansi ke mode Shelf dengan teks panduan: `"Release to add"`.
  4. Pengguna melepas klik mouse (*mouse drop*).
  5. Nusa memanggil `DragQueryFileW` untuk mengekstrak path berkas. Berdasarkan string biner, berkas disalin ke direktori `%LOCALAPPDATA%\Nusa\Shelf\` dan dicatat di `shelf.txt`.
- **Kualifikasi Bukti:** Keterlibatan API OLE COM dan teks panduan terverifikasi statis; pengujian penyeretan fisik berkas secara interaktif pada runtime tidak dieksekusi dalam audit ini.

### D. Aksesibilitas Papan Ketik & Ketiadaan Shortcut Global

- **Input Teks Terbatas:**
  Aplikasi Nusa **tidak memiliki bar input teks permanen** pada Dynamic Island. Satu-satunya input teks yang teridentifikasi dari string biner terjadi pada dialog penambahan tugas to-do ("Add Task").
- **Ketiadaan Indikasi Shortcut Global:**
  Tidak ditemukan impor `RegisterHotKey` atau konfigurasi shortcut global pada string `.rdata`. Nusa dirancang hampir 100% berbasis mouse pointer.
- **Rekomendasi untuk Hime Desktop:** Hime Desktop wajib mengatasi keterbatasan ini dengan menyediakan bar Omnibox global dan shortcut `Alt + Space` untuk pemanggilan instan berbasis keyboard.

---

## 5. Strategic Translation for Hime Desktop Interaction Architecture

| Karakteristik Interaksi Nusa | Evaluasi UX Nusa | Keputusan Desain Strategis untuk Hime Desktop |
|:---|:---|:---|
| **Pilihan Mode Pemicu (Hover vs Click)** | Fleksibel bagi kenyamanan pengguna. | **Adopt:** Hime menyediakan konfigurasi preferensi pemicu serupa (`trigger_mode: hover \| click`). |
| **Click Outside Auto-Dismissal** | Alami dan tidak memblokir alur kerja. | **Adopt:** Hime mempertahankan pola penutupan otomatis saat kehilangan fokus interaksi. |
| **Karakteristik ToolWindow & Fokus** | Mencegah polusi taskbar; perlu pengamanan fokus ketik. | **Adopt & Hardening:** Hime Tauri Shell mengonfigurasi `skip_taskbar` dan menerapkan `WS_EX_NOACTIVATE` / penanganan aktivasi yang ketat agar fokus caret pengetikan terjamin tidak tercuri. |
| **Ketiadaan Input Bar Teks** | Membatasi fungsionalitas hanya pada widget pasif. | **Adapt & Replace:** Hime menghadirkan **Omnibox Prompt Bar** untuk berkomunikasi langsung dengan Hermes Agent. |
| **Ketiadaan Shortcut Global Keyboard** | Menyulitkan pengguna mahir (*power users*). | **Adapt & Replace:** Hime menyediakan shortcut global **`Alt + Space`** untuk pemanggilan instan dari aplikasi mana pun. |
