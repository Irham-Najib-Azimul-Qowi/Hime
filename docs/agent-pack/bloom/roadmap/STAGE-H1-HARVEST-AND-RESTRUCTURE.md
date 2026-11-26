# STAGE H1 — PANEN HIME LAMA DAN PENATAAN REPO

**Tujuan:** memindahkan apa yang berharga dari Hime lama (Core + dokumen + keamanan) ke repo Bloom baru, menghapus sisanya, dan membuat repo siap untuk integrasi.
**Entry:** Stage H0 selesai. **Rujukan:** berkas 05 (manifest dan checklist). Konfirmasi: owner memerintahkan konfigurasi lama dihapus tanpa backup.

## Task

### H1.1 Inventaris repo lama
Pohon, ukuran, `git status`/`git log` (ingat: kemungkinan zero commit) di `X:\folder_ai\Hime` → `docs/hime/harvest/OLD-INVENTORY.md`. Periksa juga apakah **ada pekerjaan Fase 08** (adapter Hermes/9Router, `docs/runtime/*`, laporan) → masukkan ke manifest sebagai KEEP/ADAPT.

### H1.2 Manifest
Perbarui tabel di berkas 05 §2.1 dengan item **nyata** → `HARVEST-MANIFEST.md` (sebelum menyalin apa pun).

### H1.3 Salin dan verifikasi
Salin KEEP-MOVE/ADAPT ke struktur target (berkas 06 §1): `core/`, `docs/product`, `docs/architecture`, `docs/adr`, `docs/reference/nusa`, `docs/security`, `docs/environment`, `docs/history`. Hitung SHA-256 sumber vs salinan. **Tidak ada UI lama ke `src/` atau `src-tauri/`.**

### H1.4 Core mandiri
Di `core/`: `cargo build`, `cargo test`. Bandingkan jumlah test per crate dengan laporan lama; catat angka nyata. Tulis **klien uji kecil** (di test integrasi `core/`) yang menjalankan Core, melakukan handshake IPC, mengirim request, menerima respons, dan shutdown anggun (pengganti "Desktop Integration"). Jalankan gerbang Core (legacy 04 §7). Temuan keamanan di Core diperbaiki jika BLOCKER.

### H1.5 Pelajaran UI lama
Baca UI lama sebelum dihapus; tulis `docs/hime/OLD-UI-LESSONS.md`: alur approval (TTL 60 dtk, tiket sekali pakai, anti-replay, fail-closed), model event Activity (dedup ID), skema DTO IPC, daftar state UI yang dibutuhkan; **tanpa gaya/kode UI**. Port **tipe TS** ke draf `docs/hime/types-draft.md` (dipakai saat H2).

### H1.6 Penataan dokumen
`PROJECT_STATUS.md` baru (templat legacy 08 T2) untuk Hime×Bloom; `BACKLOG.md` dikurasi; `RISK_REGISTER.md` ditambah risiko Bloom/GPL/shell; bagian "Hime" di akhir `AGENTS.md` (aturan isolasi, build Core, lokasi kode, link pack). Salin paket ini ke `docs/agent-pack/`.

### H1.7 Skrip
Adaptasi `check-env.ps1`; buat `scripts/build-core.ps1`; `scripts/dev-hime.ps1` (build Core + jalankan shell via `run-safe.ps1`); pastikan skrip lama yang tidak relevan **tidak** ikut.

### H1.8 CI lokal
Tambah job untuk `core/` pada workflow (cargo check/clippy/test, `--locked`), tanpa mengubah job Bloom. Tidak ada push.

### H1.9 Penghapusan
Jalankan checklist berkas 05 §5.1. Jika lulus: hapus kelas DELETE dan REFERENCE-THEN-DELETE di repo lama; tulis `DELETION-LOG.md`; lalu (bila semua hash cocok) hapus sisa folder lama. **Tanpa backup** sesuai perintah owner. Jika ragu → jangan hapus, tulis OD.

## Exit gate
- `core/` terbangun dan teruji; jumlah test cocok dengan sumber; klien uji handshake lulus.
- Semua dokumen berharga ada di repo baru; hash cocok; secret scan bersih.
- Repo lama dibersihkan sesuai `DELETION-LOG.md`.
- `PROJECT_STATUS.md`, `AGENTS.md` (bagian Hime), `TOUCHPOINTS.md` terisi.
- Commit lokal `chore(h1): ...`. Laporan `STAGE-H1-REPORT.md`.
