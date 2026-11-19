# 03 — BLOOM STUDY PROTOCOL

Tujuan: **memahami Bloom dari source-nya langsung** (bukan reverse engineering biner; source terbuka), lalu menuliskan kesimpulannya sebagai standar kerja. Jangan menulis satu baris kode Hime di repo Bloom sebelum Stage H0 selesai.

## 1. Fakta Bloom yang sudah diketahui (dari dokumen publik)

Sumber: README, `AGENTS.md`, `CONTRIBUTING.md`, dan release notes v3.9–v3.10.1 di GitHub, dibaca saat penyusunan paket (Nov 2026). **Ini bukan hasil pembacaan kode.** Perlakukan sebagai hipotesis kuat yang harus kamu konfirmasi (label `VERIFIED STATIC` setelah mencocokkan dengan source).

### 1.1 Stack dan perintah
- **Windows-only Tauri v2**: backend Rust, frontend **React 19 + TypeScript**, package manager **Bun** (1.4.0), Rust stable. CI di `windows-latest`.
- `bun install` (CI: `--frozen-lockfile`; **jangan** membuat `package-lock.json` di root).
- `bun run tauri dev` = loop dev penuh. **Vite berjalan di port 1420 strict; jangan jalankan `bun run dev` terpisah.**
- `bun run build` = `tsc && vite build` = **typecheck frontend** (mode `strict` + `noUnusedLocals`/`noUnusedParameters` membuat build gagal). **Tidak ada lint/test frontend di root.**
- Gerbang Rust (di `src-tauri`): `cargo check --locked`, `cargo clippy --locked --all-targets`. `cargo test --locked` berisi tes fungsi murni; **CI tidak menjalankannya → jalankan sendiri.**
- `bun run format` (formatter oxfmt, `.oxfmtrc.json`) untuk frontend dan Rust. `bun run tauri build` membuat installer.
- `bun run bump <x.y.z>` mengubah serentak `package.json`, `tauri.conf.json`, `Cargo.toml`, `Cargo.lock`; `bun run release <x.y.z>` = bump + commit + tag + push (**jangan jalankan di fork Hime; ada push**).
- `website/` = situs statis terpisah (npm), bukan bagian build.

### 1.2 Struktur
Root: `.github/`, `.vscode/`, `docs/`, `public/`, `scripts/`, `src-tauri/`, `src/`, `website/`; berkas: `AGENTS.md`, `CONTRIBUTING.md`, `README.md`, `SETTINGS.md`, `TRAY_INTEGRATION.md`, `RELEASE_CHECKLIST.md`, `SECURITY.md`, `CODE_OF_CONDUCT.md`, `LICENSE` (GPL-3.0), `bun.lock`, `vite.config.ts`, `tsconfig*.json`, `.oxfmtrc.json`, serta **empat entri HTML**: `index.html`, `dock.html`, `overlay.html`, `settings.html`.

### 1.3 Empat window webview (masing-masing entri Vite terpisah)
| Window | HTML → entry | Peran |
|---|---|---|
| `main` | `index.html` → `src/main.tsx` → `src/App.tsx` | **Notch / Island** |
| `dock` | `dock.html` → `src/Dock.tsx` | Dock (pengganti taskbar) |
| `overlay` | `overlay.html` → `src/Overlay.tsx` | Overlay (HUD volume/brightness dll.) |
| `settings` | `settings.html` → `src/Settings.tsx` | Pengaturan |

Menambah/mengganti nama window = edit `src-tauri/tauri.conf.json`, input di `vite.config.ts`, berkas HTML, dan `src-tauri/capabilities/default.json`.

### 1.4 Backend Rust
- Entry `src-tauri/src/main.rs`; `lib.rs` hanya stub kosong. **Setiap `#[tauri::command]` wajib didaftarkan di daftar `generate_handler!` di `main.rs`.**
- `commands.rs` = permukaan IPC; `services.rs` = hook Win32 dan pekerja berumur panjang (keyboard hook, thumbnail capture, audio visualizer, registrasi AppBar); `utils.rs` = cache settings dan file IO; `state.rs`/`types.rs` = static dan tipe bersama.
- Crate `windows` dipatok **0.61** dengan daftar fitur eksplisit di `Cargo.toml` (tambah fitur di sana bila memakai API Win32 baru).
- Teknologi sistem: global hook, **WASAPI** (audio sistem real-time), **COM** (sesi media), **WMI** (hardware).
- Event backend→frontend via `app.emit`: `settings-changed`, `settings-external-changed`, `media-update`, `volume-change`, `brightness-change`, `dock-overlap`, `notch-overlap`, `windows-changed`, `update-available`, `auto-update-status` (dengarkan lewat `@tauri-apps/api/event`).

### 1.5 Settings
- Disimpan di `%APPDATA%/bloom/settings.json` sebagai objek datar berkunci berawalan `bloom-`; **semua nilai string** (`"true"`/`"false"`). `SETTINGS.md` mendokumentasikan setiap kunci.
- Jalur tulis UI: set `localStorage` → `invoke("save_setting")` → backend cache + emit `settings-changed`. Perubahan file eksternal dipantau → `settings-external-changed`.
- `src/hooks/useSettingsSync.ts` = **satu-satunya** hook sinkron (normalisasi boolean/angka string).
- CSP di `tauri.conf.json` mengizinkan origin jaringan secara eksplisit (hanya API cuaca); origin baru harus ditambah di sana. Plugin Tauri baru butuh izin di `capabilities/default.json`.

### 1.5b Perilaku dan kualitas yang diklaim
- Notch (rebuilt v3.10): model **Dynamic Island**; *klik untuk membuka* (hover hanya mengintip strip status), ingat panel terakhir (dapat dimatikan: Settings → Notch); **live activities** (musik, timer, shelf) tampil sebagai *bubble* di samping pill, bubble berbentuk rounded-square; mode panel (Music, Command Center, Status, Calendar+Pomodoro) berganti dengan scroll/swipe; Wi-Fi/Bluetooth view; **shelf** (drop file/teks); preview berkas; transisi **spring** (lebar, tinggi, radius, posisi beranimasi independen); mode visibilitas **Fixed / Smart / Peek** (notch dan dock); dukungan multi-monitor; flare sudut tanpa seam pada skala 125%/150%.
- Dock: pengganti taskbar, urut-ulang drag, preview jendela, extras (drive/folder/Recycle Bin), Win+1..9, menu tray di ikon dock, Task View opsional.
- Ketahanan: bila webview crash, Bloom restart sendiri dan **memulihkan taskbar**; penanda berkas memulihkan taskbar setelah crash; "runtime diagnostics" terbatas; updater bawaan (auto-install ±24 jam) dan pengumuman dalam-app.
- Jaminan idle: pekerja tidur saat tak dibutuhkan (visualizer jeda, monitor kursor mati saat dock tak ada, thumbnail hanya saat fokus).

### 1.6 Aturan kontribusi (menjadi aturan kita juga)
- Prioritas: **kebenaran shell, perilaku input/fokus, aksesibilitas, CPU/memori idle rendah.** Dock untuk aplikasi, notch untuk informasi ringkas dan kontrol. Fallback native Windows lebih baik daripada menduplikasi aplikasi utuh.
- Perubahan perilaku Windows (fokus, restorasi taskbar, tray, DPI) **harus diverifikasi di Windows**; build/unit test lintas platform tidak cukup.
- **Commit mengikuti Conventional Commits** (`feat:`, `fix:`, `chore:`); catatan rilis dibangkitkan dari commit; commit yang hanya menyentuh `website/*`, `.github/*`, `*.md` difilter.
- Jangan tambah dependensi tanpa alasan; cocokkan gaya kode; pisahkan pembersihan yang tak terkait.
- Jebakan: **setiap jalur exit/restart baru wajib memulihkan taskbar dan melepas AppBar** (seperti menu tray di `main.rs`).

## 2. Yang HARUS kamu pelajari dari source (belum diketahui paket ini)

Jawab semuanya dengan rujukan `path:baris` di `docs/hime/BLOOM-CONVENTIONS.md` dan `docs/hime/BLOOM-ARCHITECTURE-NOTES.md`:

**A. Frontend**
1. Pohon `src/` lengkap: komponen, hooks, pustaka util, tipe, aset; bagaimana `App.tsx` mengatur mode panel Island.
2. **Mesin animasi/spring:** pustaka apa (Framer Motion? implementasi sendiri?), konfigurasi spring (stiffness/damping/mass), tabel ukuran pill/panel/bubble, bagaimana lebar/tinggi/radius/posisi beranimasi mandiri.
3. **Styling:** CSS biasa / modul / Tailwind / CSS-in-JS? token warna, radius, bayangan, tipografi (font Inter?), pengelolaan tema gelap/terang, ukuran ikon, set ikon.
4. Pola state (context/store/hook), pola `invoke`/`listen`, penanganan error dan state kosong/loading.
5. Cara panel didaftarkan (daftar mode, urutan, scroll/swipe), cara **live activity bubble** didefinisikan (prioritas, penjadwalan, ukuran, klik), cara *panel arrows* dan *peek* bekerja.
6. Struktur `Settings.tsx`: tab, komponen kontrol, pola menambah pengaturan baru; kaitannya dengan `SETTINGS.md`.
7. Pola komponen Dock dan Overlay; cara window `overlay` dipakai (HUD).
8. Aksesibilitas yang ada (fokus, keyboard, ARIA).

**B. Backend**
9. Daftar semua `#[tauri::command]`, semua `app.emit`, semua `services` (thread/worker), dan siklus hidupnya; cara menjaga CPU idle rendah.
10. Penanganan window: pembuatan, posisi, AppBar, multi-monitor, DPI, click-through, always-on-top, fokus (`main.rs`, `services.rs`).
11. Pemulihan taskbar: marker file, jalur exit, watchdog webview; **tepat di mana** kode ini ada.
12. Updater dan announcement: endpoint, kunci publik, jadwal, apa yang dikirim keluar (telemetri?). Semua panggilan jaringan keluar (CSP + kode) → daftar lengkap.
13. `tauri.conf.json` (window, security, bundle, updater), `capabilities/default.json`, `Cargo.toml` (fitur `windows`, plugin).
14. Pola pengujian Rust yang ada; apa yang bisa ditiru.
15. `scripts/` (bump, release, lainnya), `.github/workflows` (gerbang CI), `RELEASE_CHECKLIST.md`, `SETTINGS.md`, `TRAY_INTEGRATION.md`, `docs/`.

**C. Perilaku nyata**
16. Jalankan Bloom **sesuai protokol aman** (berkas 04 bagian 5): catat bagaimana notch, dock, mode, bubble, animasi tampak dan bekerja. **Ambil screenshot** setiap mode/panel/bubble/state utama (100% dan 125% DPI, 1366×768). Ini menjadi **baseline visual** (`docs/hime/baseline/`).
17. Ukur baseline resource: CPU idle, RAM per proses (shell + WebView2 per window), dalam beberapa menit.

## 3. Keluaran Stage H0 (lihat `roadmap/STAGE-H0-…`)

| Berkas | Isi |
|---|---|
| `docs/hime/BLOOM-CONVENTIONS.md` | **Aturan kode dan desain Bloom yang nyata**, dengan rujukan berkas. Ini menjadi standar kita (menggantikan standar lama untuk kode shell) |
| `docs/hime/BLOOM-ARCHITECTURE-NOTES.md` | Peta arsitektur, alur data, siklus hidup, ukuran/tabel animasi, daftar command/event/service |
| `docs/hime/BLOOM-TOUCHPOINT-CANDIDATES.md` | Titik sentuh minimal untuk integrasi Hime (di mana mendaftarkan panel, bubble, command, event, window baru, tab settings) |
| `docs/hime/BLOOM-RISKS.md` | Risiko (taskbar, updater, telemetri, hak akses, DPI, GPL) dan mitigasi |
| `docs/hime/baseline/` | Screenshot + metrik resource baseline |
| `docs/legal/THIRD-PARTY.md` | Lisensi Bloom + dependensinya |

Jika source menyimpang dari "fakta" di bagian 1, **source menang**; catat selisihnya di `BLOOM-ARCHITECTURE-NOTES.md`.
