# 05 — HARVEST & CLEANUP MANIFEST

Keputusan owner: Hime dibuat ulang di atas Bloom; **konfigurasi lama dihapus tanpa backup**; **apa yang bisa dipakai dan tidak merusak versi Bloom dipakai ulang**. Dokumen ini mengubahnya menjadi prosedur yang aman.

## 1. Fakta tentang repo lama (menurut laporan Fase 06–08; periksa nyata)
- Lokasi: `X:\folder_ai\Hime` (Windows).
- **Tidak ada commit** (zero commit; semua perubahan di working tree) → **penghapusan tidak dapat dibatalkan**. Karena itu aturan "hash-verified" di bagian 4 wajib.
- Workspace Rust 6 crate (`hime-core`, `hime-ipc`, `hime-pal`, `hime-security`, `hime-storage`, plus shell desktop Tauri) dan Desktop Tauri 2 + React + TS + Tailwind; test dilaporkan 37/37 (Core 2, Core Integration 7, Desktop Integration 3, IPC 14, PAL 1, Security 5, Storage 1; sisanya sesuai laporan).
- Dokumen: `project-management/` (phases, system-architecture [12 ADR, 16 constraints], product-requirements [33 FR, 18 NFR]), `docs/` (environment, security, runtime bila Fase 08 sudah berjalan), `scripts/`, `PROJECT_STATUS.md`, `BACKLOG.md`, `RISK_REGISTER.md`, `AGENTS.md`.

## 2. Klasifikasi

| Kelas | Arti | Tindakan |
|---|---|---|
| **KEEP-MOVE** | Dipakai apa adanya sebagai fondasi Hime Core/dokumen | Salin ke repo baru, verifikasi, lalu hapus dari lama |
| **ADAPT** | Berguna tetapi harus disesuaikan | Salin, ubah seperlunya, catat perubahan |
| **REFERENCE-THEN-DELETE** | Dibaca sebagai referensi perilaku/spesifikasi, tidak disalin | Ekstrak pelajaran ke dokumen, lalu hapus |
| **DELETE** | Tidak berguna / bisa merusak versi Bloom | Hapus |

### 2.1 Tabel (agent memperbaiki dengan inventaris nyata)

| Item lama | Kelas | Tujuan / catatan |
|---|---|---|
| Crate `hime-ipc` (framing 12-byte, JSON-RPC 2.0, Named Pipe, token+nonce) | **KEEP-MOVE** | `core/crates/hime-ipc`. Juga dipakai shell sebagai path-dependency (lihat berkas 06) |
| Crate `hime-core` (bootstrap, lifecycle, task+audit SQLite WAL, approval lifecycle, supervisor) | **KEEP-MOVE** | `core/crates/hime-core` |
| Crate `hime-security` (PEP/PDP, DPAPI vault, redaksi) | **KEEP-MOVE** | `core/crates/hime-security` |
| Crate `hime-pal` | **KEEP-MOVE** | tetap menjadi satu-satunya tempat API OS di Core |
| Crate `hime-storage` | **KEEP-MOVE** | diperdalam di Fase 10 |
| Workspace `Cargo.toml`/`Cargo.lock` Core | **ADAPT** | `core/Cargo.toml`; workspace **terpisah** dari `src-tauri` (jangan mengubah build Bloom) |
| Test integrasi Core dan IPC | **KEEP-MOVE** | harus lulus tanpa perubahan semantik |
| Test "Desktop Integration" (3) | **ADAPT** | tulis ulang sebagai test integrasi *bridge ↔ Core* (berkas 06) |
| Dokumen PRD, 33 FR, 18 NFR, UX Architecture | **KEEP-MOVE** → `docs/product/` | UX Architecture diberi banner "UI diganti Bloom; lihat `07-UX-SPEC…`" |
| System Architecture, 12 ADR, 16 constraints, spesifikasi implementasi Fase 04 | **KEEP-MOVE** → `docs/architecture/`, `docs/adr/` | ADR yang menyebut Desktop buatan sendiri → tambah ADR baru "ADR-013: Shell = fork Bloom" yang *supersedes* sebagian |
| Dokumen RE Nusa (binary/arsitektur/UX/feature parity) | **KEEP-MOVE** → `docs/reference/nusa/` | referensi saja; **tanpa aset berhak cipta** |
| `docs/security/*`, `SECURITY-BASELINE.md`, `RISK_REGISTER.md` | **KEEP-MOVE** | tambah risiko Bloom/GPL/shell |
| `docs/environment/*` (baseline hardware) | **KEEP-MOVE** | |
| Laporan fase lama (`PHASE-0x-*-REPORT.md`) | **ADAPT** → `docs/history/` | arsip sejarah, bukan sumber kebenaran |
| `PROJECT_STATUS.md`, `BACKLOG.md` | **ADAPT** | **tulis ulang** untuk Hime×Bloom (bagian atas = posisi saat ini) |
| `AGENTS.md` lama | **REFERENCE-THEN-DELETE** | aturan yang masih relevan masuk ke bagian "Hime" di `AGENTS.md` Bloom; sisanya dibuang |
| `scripts/check-env.ps1` | **ADAPT** | tambah cek Bun, WebView2, Visual Studio C++ Build Tools, Rust; tetap *non-mutating* |
| `scripts/build.ps1`, `test.ps1`, `dev.ps1` | **ADAPT/REPLACE** | sesuaikan: build Core (cargo) + shell (bun/tauri); dev.ps1 harus membersihkan proses dan memulihkan taskbar (pakai `run-safe.ps1`) |
| **UI Desktop lama**: aplikasi Tauri lama, React (Island, Omnibox, Conversation, Activity, Task Board, Notification Center, Settings, Floating Companion), design tokens, window presets | **REFERENCE-THEN-DELETE** | Baca untuk menyalin **kebutuhan perilaku**, bukan gaya/kode: terutama alur approval (TTL 60 dtk, tiket sekali pakai, anti-replay), model event/Activity, skema DTO IPC. Tulis ringkasannya di `docs/hime/OLD-UI-LESSONS.md`, lalu hapus |
| Tipe/DTO TypeScript klien IPC lama | **REFERENCE** | port **tipenya** ke `src/hime/types/` dalam gaya Bloom; jangan salin komponen |
| Konfigurasi toolchain UI lama: `package.json`, lockfile (`pnpm-lock`), `tailwind.config`, `postcss`, `vite.config`, `tsconfig`, ESLint/Prettier lama, `node_modules/` | **DELETE** | Bloom punya toolchain sendiri (Bun + oxfmt) |
| Konfigurasi dev lokal lama (`config/*.local.*`, `.env*`, file path mesin lama) | **DELETE** | owner: "konfigurasi lama dihapus saja" |
| Artefak build: `target/`, `dist/`, `build/`, cache | **DELETE** | |
| Data uji: database SQLite uji, vault uji, log | **DELETE** | pastikan tidak ada rahasia terbawa |
| Berkas draf/kerja sementara (`*.tmp`, catatan acak) | **DELETE** | |

## 3. Aturan panen (apa yang BOLEH dibawa ke repo Bloom)

1. **Tidak ada UI lama masuk ke `src/` atau `src-tauri/` Bloom.** Titik.
2. Core masuk sebagai `core/` (workspace terpisah). Tidak mengubah `src-tauri/Cargo.lock` kecuali lewat path-dependency yang disetujui (berkas 06 bagian 3).
3. Tidak ada dependensi frontend lama (Tailwind, pnpm, dll.) yang masuk `package.json` Bloom.
4. Tidak ada rahasia, token, path pribadi, database uji yang terbawa. Jalankan secret scan pada seluruh hasil panen.
5. Semua yang disalin dicatat di `docs/hime/harvest/HARVEST-MANIFEST.md`: sumber → tujuan → kelas → hash SHA-256.

## 4. Prosedur (Stage H1; detail di `roadmap/STAGE-H1-…`)

1. **Inventaris** repo lama (pohon + ukuran + status git) → `OLD-INVENTORY.md`. Jangan mengubah apa pun.
2. **Tulis manifest** (bagian 2.1 diperbarui dengan item nyata) sebelum menyalin/menghapus.
3. **Salin** KEEP-MOVE/ADAPT; hitung SHA-256 sumber dan salinan; cocokkan.
4. **Bangun dan uji Core mandiri** di repo baru: `cargo build`, `cargo test` pada `core/`. Jumlah test untuk crate yang dipindah harus **sama** dengan laporan lama (angka nyata dicatat; selisih = BLOCKER).
5. **Ekstrak pelajaran** dari kelas REFERENCE-THEN-DELETE ke dokumen.
6. **Verifikasi kelengkapan** (checklist 5.1).
7. **Hapus** DELETE dan REFERENCE-THEN-DELETE di repo lama; tulis `DELETION-LOG.md` (daftar jalur + ukuran) — **tanpa backup** sesuai perintah owner.
8. **Folder lama:** setelah checklist 5.1 lulus **dan** semua berkas KEEP/ADAPT terbukti identik (hash) di repo baru, folder lama sisanya boleh dihapus seluruhnya. Jika ada keraguan sekecil apa pun → jangan hapus, catat di `OWNER-DECISIONS-NEEDED.md`.

### 5.1 Checklist sebelum menghapus apa pun
- [ ] `core/` terbangun bersih dan seluruh test yang dipindah lulus (angka cocok).
- [ ] 12 ADR, 33 FR, 18 NFR, 16 constraints, spesifikasi Fase 04, dokumen keamanan, RISK_REGISTER, baseline hardware ada di repo baru (hash cocok).
- [ ] `OLD-UI-LESSONS.md` selesai (alur approval, event model, DTO).
- [ ] `HARVEST-MANIFEST.md` dan `DELETION-LOG.md` (daftar rencana) ditulis.
- [ ] Secret scan hasil panen bersih.
- [ ] Tidak ada pekerjaan Fase 08 yang belum tersalin (periksa `git status` lama: tracked-modified + untracked; semua dipertimbangkan).

## 6. Hal yang TIDAK boleh dihapus tanpa persetujuan eksplisit
Apa pun di luar folder repo lama itu sendiri; data pengguna di luar repo; instalasi Hermes/9Router milik owner; kunci/vault nyata milik owner. Perintah owner hanya mencakup "konfigurasi lama Hime" dan hasil UI lama.
