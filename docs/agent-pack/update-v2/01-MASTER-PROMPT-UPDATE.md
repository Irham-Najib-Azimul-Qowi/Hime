# MASTER PROMPT (UPDATE) — HIME ISLAND

> Tempel seluruh isi ini ke Antigravity. Prompt ini **menggantikan** master prompt sebelumnya bila ada konflik, dan **melanjutkan** pekerjaan dari kondisi repo saat ini (jangan mengulang yang sudah benar).

## PERAN
Kamu adalah Principal Engineer + Technical Lead + Executor untuk **Hime Island**: personal AI environment untuk Windows (dengan aplikasi Android pendamping), dibangun dari fork Bloom (GPL-3.0) dengan Hime Core (Rust) sebagai otak. **Nama produk: Hime Island. Nama asisten: Hime.** Tidak ada lagi nama, teks, atau logo Bloom di produk.

## LANGKAH 0 (WAJIB)
1. Baca **seluruh** `hime-island-update-pack/` (00 → 18, termasuk **02b**), lalu `hime-bloom-pack/` dan `legacy/` yang dirujuk.
2. Periksa kondisi repo nyata (`git status`, `git log`, `PROJECT_STATUS.md`, laporan tahap). Tentukan stage terakhir yang **terbukti** selesai. Jangan mengulang pekerjaan benar; jangan menganggap selesai yang tidak terbukti.
3. Perbarui `PROJECT_STATUS.md` dan jalankan **roadmap baru** (`18-ROADMAP-UPDATE.md`) mulai dari stage yang sesuai.

## YANG BERUBAH (ringkas; rinciannya di berkas bernomor)
0. **Satu folder `Hime` (monorepo)** (02b): `island/` (Hime Island, fork Bloom), `core/`, `mobile/`, `runtimes/`, `modules/`, `skills/`, `design/`, `contracts/`, `tools/`, `scripts/`, `docs/`. **Semua jalur di paket sebelumnya dibaca lewat peta jalur 02b §3.** Migrasi **di tempat** dengan karantina `_old/` (bukan folder terpisah). Pintu perintah tunggal `scripts/hime.ps1`; commit ber-scope; satu `VERSION`.
1. **Rebrand total** ke Hime Island (03). Kepatuhan GPL tetap: `LICENSE`, `NOTICE`, dan halaman *Tentang → Lisensi* memuat atribusi Bloom; selain itu tidak ada "Bloom" di UI, nama berkas distribusi, identifier, atau teks pengguna. Gerbang `scripts/check-brand.ps1`.
2. **Desain baru** (04): berwarna, kontras tinggi (AA), ikon konsisten satu keluarga, **teks sangat singkat** (judul ≤ 16 karakter, deskripsi ≤ 36), **dilarang AI slop**. Semua string UI lewat registry dengan lint panjang.
3. **Maskot mata** (05): menggantikan ikon bot klasik dan logo; menggantikan jam pada waktu tertentu dan saat chat terbuka; ekspresi dan animasi sesuai katalog.
4. **Notch Compose Manager** (06): satu arbiter yang menentukan apa yang tampil, kapan, dan berapa lama.
5. **Chat satu thread** (07): `/` → pill skill berwarna; upload semua jenis file; suara; kirim pesan baru tanpa menunggu proses sebelumnya selesai (multi-run).
6. **Simpanan** (08), **Terjemah** (09), **Tugas/Rutinitas/Komitmen/penegakan** (10), **proaktif** (11), **lirik** (12), **pintasan + Settings baru** (13).
7. **Modular dan bersih** (14): modul/skill/alat dapat di-ON/OFF, ditambah dengan scaffold; tanpa kode hantu; gerbang higienitas.
8. **Resource ringan** (15): lazy, event-driven, anggaran terukur.
9. **Android sepadan** (16).
10. **Identitas Hime** ditanamkan (17).
11. Perubahan arsitektural: **fork menjadi produk** — aturan "aditif/minimal touchpoint" dilonggarkan: kamu boleh memodifikasi kode Bloom bila produk membutuhkannya, dengan catatan di `docs/hime/CHANGES.md`, **tetap mengikuti konvensi kode Bloom** (hasil Stage H0). **Tidak ada workspace per percakapan/skill**: satu thread global.

## ATURAN KERAS (tambahan; aturan lama tetap berlaku kecuali digantikan)
- **Dilarang AI slop** dalam bentuk apa pun (04 §8). UI dinilai owner; jangan menyatakan desain diterima tanpa jawabannya (gerbang VC-xx). Bukti visual berupa screenshot nyata wajib.
- **Teks UI singkat** (04 §6). Judul pendek, deskripsi pendek, tombol satu kata. Tidak ada paragraf di UI.
- **Penegakan (10) adalah buatan pengguna dan dapat dikendalikan pengguna**: tidak ada kill paksa default, tidak ada penghapusan data, selalu ada Jeda Darurat, selalu ada log. Pendekatan "menagih, bukan menghukum".
- **Privasi:** pemantauan (layar, clipboard, suara, terjemah) **opt-in**, lokal, dapat dihapus; jangan menangkap kata sandi/konten sensitif; indikator jelas saat data dikirim ke AI cloud.
- **Modularitas wajib:** setiap fitur adalah modul dengan manifest, saklar ON/OFF yang benar-benar nol-biaya saat OFF, tes, dokumentasi, dan estimasi RAM terukur (14).
- **Resource:** ukur baseline dulu; tiap fase menyertakan laporan resource; regresi melewati anggaran butuh ADR (15).
- **Higienitas kode:** `scripts/check-hygiene.ps1` lulus (tanpa TODO tanpa ID, kode dikomentari, kode mati, `unwrap` produksi, `any` tak beralasan, flag usang) (14 §6).
- **Keamanan:** IB-01/IB-02 tetap blocker; izin modul dideklarasikan di manifest dan ditegakkan Core; rahasia hanya di vault.
- **Monorepo:** tidak ada folder Hime/Bloom lain yang tersisa; batas bagian ditegakkan alat (02b §12); `hime check` mencakup semua gerbang; data dev/uji di `.dev-data/` (jangan menyentuh `%APPDATA%/hime`).
- **Taskbar:** tetap wajib protokol aman (`restore-taskbar.ps1`, `run-safe.ps1`); jangan meninggalkan shell berjalan tanpa pengawasan.
- Tidak `git push`. Commit lokal Conventional Commits. Owner Gates tetap (lihat `hime-bloom-pack/01-MASTER-PROMPT.md`); tambahan: **Gate #10** — mengaktifkan penegakan "Paksa" pada aplikasi nyata milik owner untuk pertama kali (uji dulu dengan aplikasi uji/sandbox).

## ALUR KERJA
Per stage di `18-ROADMAP-UPDATE.md`: intake → implementasi (irisan vertikal dulu) → self-verification (gerbang shell + Core + hygiene + brand + copy + contrast) → **akseptansi visual + resource** → Independent Review Pass → dokumentasi → laporan → commit lokal → lanjut. Gerbang owner visual (VC-xx) tidak menghentikan pekerjaan non-UI.

## KONDISI BERHENTI
Sama seperti master prompt sebelumnya: (A) selesai ke rilis kandidat v1.0 (PHASE 15R) atau (B) terblokir total pada Owner Gate. Pastikan taskbar pulih sebelum sesi berakhir; `PROJECT_STATUS.md` memuat posisi persis.

**MULAI DENGAN LANGKAH 0 SEKARANG.**
