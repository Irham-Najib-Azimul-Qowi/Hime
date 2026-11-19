# STAGE H0 — STUDI DAN IMPOR BLOOM

**Tujuan:** mendapatkan Bloom yang berjalan di mesin owner dengan aman, memahami source-nya, dan menetapkan standar kerja dari kenyataan kode. **Belum ada fitur Hime.**
**Entry:** paket dibaca. **Dilarang:** menulis fitur Hime, mengubah perilaku Bloom (kecuali yang diwajibkan §4 dan §5).

## Task

### H0.1 Lingkungan
Jalankan/adaptasi `scripts/check-env.ps1` (non-mutating): Windows 10/11, **Bun ≥ 1.4**, **Rust stable (MSVC)**, **Visual Studio C++ Build Tools**, WebView2, Git. Jika ada yang kurang dan butuh admin → `OWNER-DECISIONS-NEEDED.md` (Gate #4), jangan memasang diam-diam.

### H0.2 Klon dan pin
`git clone https://github.com/little-systems/bloom.git` ke folder baru (default `X:\folder_ai\Hime-Bloom`; jangan di dalam repo lama). Checkout **tag rilis terbaru** (saat penulisan paket: v3.10.1) atau tag stabil terbaru yang kamu temukan; catat tag + SHA di `docs/hime/UPSTREAM.md`. Tambahkan remote `upstream` (tetap), **hapus/jangan atur remote push**. Jangan membuat branch di remote. Branch kerja: `hime/h0-study`.
Jika URL gagal (mis. pengalihan ke `SehajveerSingh2005/bloom`), gunakan yang bekerja dan catat.

### H0.3 Kepatuhan lisensi
Baca `LICENSE` (GPL-3.0), `SECURITY.md`, `CODE_OF_CONDUCT.md`. Buat `docs/legal/THIRD-PARTY.md` (lisensi dependensi Bun dan Cargo yang terlihat; `cargo license`/`bun pm ls` bila tersedia) dan catatan GPL di `docs/legal/GPL-NOTES.md`.

### H0.4 Keselamatan sebelum menjalankan (G15)
Buat dan **uji** `scripts/restore-taskbar.ps1` dan `scripts/run-safe.ps1` (berkas 04 §5). Dokumentasikan `docs/hime/RECOVERY.md`. **Jangan** menjalankan Bloom sebelum ini lulus.

### H0.5 Netralkan jaringan keluar dan updater (G14, D9)
Sebelum build pertama dijalankan:
1. Audit **semua** panggilan jaringan keluar: CSP di `tauri.conf.json`, kode updater, announcement, cuaca, telemetri/analytics, apa pun (`grep` URL, `fetch`, `reqwest`, plugin Tauri).
2. **Nonaktifkan updater otomatis dan announcement** (konfigurasi, bukan hapus kode bila memungkinkan) agar Hime tidak tertimpa Bloom. Catat tiap perubahan di `TOUCHPOINTS.md`.
3. Dokumentasikan hasil di `docs/hime/BLOOM-RISKS.md`. Bila ada telemetri tak terduga → laporkan ke owner.

### H0.6 Build dan jalankan Bloom asli (tanpa perubahan fungsional)
`bun install --frozen-lockfile` → `bun run build` → `cargo check --locked`, `cargo clippy --locked --all-targets`, `cargo test --locked` (catat hasil nyata; CI tidak menjalankan test). Jalankan lewat **`run-safe.ps1`**. Catat apa yang berhasil dan gagal.

### H0.7 Studi source
Ikuti `03-BLOOM-STUDY-PROTOCOL.md` §2 (pertanyaan A1–C17). Baca **semua** berkas di `src/` dan `src-tauri/src/` yang relevan, `tauri.conf.json`, `capabilities/`, `Cargo.toml`, `package.json`, `vite.config.ts`, `scripts/`, `.github/workflows`, `SETTINGS.md`, `TRAY_INTEGRATION.md`, `RELEASE_CHECKLIST.md`, `docs/`.

### H0.8 Baseline visual dan resource
Screenshot setiap mode/panel/bubble/state utama Bloom (100% dan 125% DPI, 1366×768) → `docs/hime/baseline/`. Ukur CPU idle dan RAM (shell + tiap WebView2) beberapa menit → `docs/hime/baseline/RESOURCES.md`. Jika tidak ada sesi interaktif → `NOT TESTED` + `VC-00` ke owner (minta screenshot baseline dari owner).

### H0.9 Dokumen standar
Tulis: `BLOOM-CONVENTIONS.md`, `BLOOM-ARCHITECTURE-NOTES.md`, `BLOOM-TOUCHPOINT-CANDIDATES.md`, `BLOOM-RISKS.md` (templat 09 T-B). Konfirmasi atau koreksi "fakta Bloom" di berkas 03 §1.

### H0.10 ADR
ADR-013 "Shell Hime = fork Bloom (Hime Core tetap proses terpisah)", ADR-014 "Integrasi aditif + touchpoint registry", ADR-015 "Updater/announcement dinonaktifkan". Format legacy 08 T4.

## Exit gate
- Bloom asli terbangun, teruji (angka nyata), berjalan aman, dan terpulihkan dengan skrip.
- Updater/announcement dinetralkan; jaringan keluar terdaftar.
- Dokumen standar lengkap dengan rujukan `path:baris`.
- Baseline visual dan resource ada (atau `VC-00` diminta).
- Commit lokal `chore(h0): ...` (Conventional Commits) di branch kerja. **Tanpa push.**
- Laporan: `project-management/phases/STAGE-H0-REPORT.md`.
