# HIME ISLAND — UPDATE PACK (v2)

**Untuk:** Antigravity (AI coding agent)  **Project Owner:** pengguna  **Dibuat:** 19 November 2026
**Fungsi paket ini:** memperbarui arah proyek **Hime × Bloom** menjadi **Hime Island**: produk sendiri dengan identitas, desain, fitur, dan standar teknis baru. Paket ini **dibaca bersama** paket sebelumnya (`hime-bloom-pack/`) dan paket lama (`legacy/`).

## 1. Urutan prioritas dokumen (jika bertentangan)

1. **Paket ini (update pack v2)** ← menang
2. `hime-bloom-pack/` (paket Bloom: Stage H0–H2, Fase 08R–15R, aturan integrasi)
3. `hime-bloom-pack/legacy/` (spesifikasi backend, keamanan, standar kerja)

Catat setiap konflik yang kamu temukan di `docs/hime/DECISIONS-LOG.md`. Detail ada di `02-DECISIONS-AND-PRECEDENCE.md`.

## 2. Ringkasan perubahan

| # | Perubahan | Berkas |
|---|---|---|
| 1 | Nama produk **Hime Island**; semua teks/logo/nama Bloom hilang dari produk | 03 |
| 2 | Bahasa desain baru: lebih berwarna, kontras tinggi, ikon bagus, **teks sangat singkat**, anti "AI slop" | 04 |
| 3 | Maskot **dua mata bot** berekspresi; menggantikan ikon bot klasik; tampil di notch menggantikan jam | 05 |
| 4 | Sistem animasi dan transisi | 05 |
| 5 | Manajemen kemunculan semua fitur di notch (siapa tampil kapan) | 06 |
| 6 | **Chat satu thread**: `/` skill menjadi pill berwarna, upload semua file, suara, kirim pesan tanpa menunggu | 07 |
| 7 | **Simpanan** (shelf + riwayat clipboard): tampung teks dan file, seret ke mana saja | 08 |
| 8 | **Terjemah di notch** + penjelasan singkat | 09 |
| 9 | **Tugas** (kartu notch, kategori), **Rutinitas**, **Komitmen**, penegakan (tutup window, batasi HP) | 10 |
| 10 | **Hime proaktif**: aturan dan katalog interaksi Windows dan HP | 11 |
| 11 | **Lirik** pada pemutar musik | 12 |
| 12 | **Pintasan keyboard** + satu pintasan daftar pintasan; redesain **Settings** | 13 |
| 13 | **Modular**: modul/skill/alat dapat dinonaktifkan, ditambah; kualitas kode tanpa kode hantu | 14 |
| 14 | **Optimasi resource** (anggaran dan cara) | 15 |
| 15 | **Aplikasi Android** sepadan (tema, chat, tugas, penegakan) | 16 |
| 16 | **Identitas "Hime"** ditanamkan ke Hermes/sistem | 17 |
| 17 | Roadmap yang diperbarui | 18 |
| 18 | **Satu folder `Hime`** (monorepo): Island, Core, mobile, runtimes, modul, desain, kontrak, skrip, dokumen — semua terkelola | 02b |

## 3. Urutan baca

`01-MASTER-PROMPT-UPDATE.md` → `02` → **`02b`** → `03` → `04` → `05` → `06` → `07` … `18`. Baca semuanya sebelum mengubah apa pun. Lalu baca ulang `hime-bloom-pack/00-README` dan berkas yang dirujuk.

## 4. Isi paket

```
hime-island-update-pack/
├── 00-README-START-HERE.md
├── 01-MASTER-PROMPT-UPDATE.md             <- tempel ke Antigravity
├── 02-DECISIONS-AND-PRECEDENCE.md
├── 02b-MONOREPO-LAYOUT-AND-MANAGEMENT.md   <- semua Hime dalam satu folder
├── 03-BRANDING-HIME-ISLAND.md
├── 04-DESIGN-SYSTEM.md
├── 05-MOTION-AND-EYES.md
├── 06-NOTCH-EXPERIENCE-AND-PRIORITY.md
├── 07-CHAT-SPEC.md
├── 08-SHELF-AND-CLIPBOARD.md
├── 09-TRANSLATION-AND-EXPLAIN.md
├── 10-TASKS-ROUTINES-ENFORCEMENT.md
├── 11-PROACTIVE-INTERACTION.md
├── 12-MUSIC-LYRICS.md
├── 13-KEYBINDS-AND-SETTINGS.md
├── 14-MODULARITY-AND-CODE-QUALITY.md
├── 15-RESOURCE-OPTIMIZATION.md
├── 16-MOBILE-PARITY.md
├── 17-HIME-IDENTITY-AND-PERSONA.md
└── 18-ROADMAP-UPDATE.md
```

## 5. Penempatan
Salin ke `docs/agent-pack/update-v2/` di folder `Hime` (monorepo; lihat 02b). Jangan menimpa `AGENTS.md`; perbarui bagian "Hime" di dalamnya agar menunjuk ke paket ini.

## 6. Jalur direktori
Semua jalur seperti `src/…` dan `src-tauri/…` di berkas 03–17 dibaca sebagai **`island/src/…`** dan **`island/src-tauri/…`**; peta lengkap di **02b §3**. Perintah `bun …` dijalankan di `island/` atau lewat `scripts/hime.ps1`.

## 7. Catatan jujur tentang isi
- Nilai visual konkret (ukuran notch, spring, radius) **harus diambil dari source** (Stage H0). Angka di paket ini adalah **titik awal yang sudah dicek kontrasnya**, bukan hukum.
- Angka resource adalah **target yang harus divalidasi dengan pengukuran**, bukan fakta.
- Beberapa kemampuan teknis bergantung pada versi nyata (WebView2, wry/Tauri, Hermes, 9Router, Android API). Semua ditandai **"verifikasi"**; jangan menganggapnya pasti.
