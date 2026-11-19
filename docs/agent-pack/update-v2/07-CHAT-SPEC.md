# 07 — CHAT SPEC (SATU THREAD, SKILL `/`, FILE, SUARA, MULTI-RUN)

Tujuan: chat dengan Hime yang **cepat, sederhana, modern, tidak slop**, dan tidak pernah membuat pengguna menunggu.

## 1. Prinsip
1. **Satu thread global.** Tidak ada "chat baru", tidak ada workspace per topik/skill. Riwayat tidak hilang. Hime menjaga konteks lewat ringkasan bergulir dan memori (Core), bukan dengan memecah percakapan.
2. **Chat hidup di notch** (panel Hime, bisa membesar). Tidak ada chat di window lain. Android memakai thread yang sama.
3. **Tidak memblokir.** Pengguna bisa mengirim pesan berikutnya **saat pesan sebelumnya masih diproses**.
4. **Ringkas.** Jawaban default pendek; detail tersedia lewat "Lebih" (expand). Tidak ada basa-basi.
5. **Semua skill, satu tempat.** Dipanggil dengan `/`.

## 2. Layout (panel Hime)

```
┌─────────────────────────────────────────────┐
│  [mata Hime]  Hime                  [ ⋯ ]   │  ← header: mata (menggantikan jam), judul, menu
├─────────────────────────────────────────────┤
│  (riwayat thread; pesan terbaru di bawah)   │
│   user ·····························        │
│   Hime ·····························        │
│   [kartu proses: "Ringkas file…" ▓▓▓░ 60%]  │  ← run card (non-blocking)
├─────────────────────────────────────────────┤
│  [pill skill] [lampiran] ……                 │  ← baris pill & lampiran
│  [ + ]  Tanya Hime…                [mik][kirim]  │  ← input (ikon dari map, bukan emoji)
└─────────────────────────────────────────────┘
```
- Header: **mata Hime** bereaksi pada keadaan (05 §4.5) + judul "Hime" + menu `⋯` (Cari, Bersihkan tampilan, Pengaturan chat).
- Pesan: pengguna rata kanan (latar `--surface-3`), Hime rata kiri (tanpa bubble penuh; teks langsung di `--surface-1`, tipis); tanpa avatar besar. Waktu hanya saat hover.
- Gaya teks Hime: markdown **tersanitasi** (daftar, kode, tautan); kode dengan tombol salin; tabel scroll horizontal.
- Input: tinggi dinamis (1–6 baris), `Enter` kirim, `Shift+Enter` baris baru. Tombol `+` (lampiran), mikrofon, kirim.
- **Ukuran:** default "terbuka" (ringkas) → otomatis **besar** bila riwayat/konten panjang atau saat diseret file ke dalamnya; kembali ringkas saat ditutup.
- Salin/Urungkan/Ulangi tersedia sebagai ikon kecil saat hover (tidak berupa teks).

## 3. Skill dengan `/` (pill berwarna)

**Alur:**
1. Ketik `/` di awal input atau setelah spasi → muncul **menu skill** di atas input (maks 6 baris terlihat, scroll).
2. Menu mencantumkan skill **yang aktif** (modul ON) dari Skill Registry Core: **ikon (IconTile berwarna kategori) + nama (≤ 16 kar) + deskripsi (≤ 36 kar)**, dikelompokkan per kategori dengan penanda warna. Ketik untuk **menyaring** (nama, alias); ↑/↓ memilih, `Enter`/`Tab` memasukkan, `Esc` menutup. Skill yang sering dipakai muncul di atas.
3. Skill terpilih menjadi **pill solid berwarna kategori** di dalam input (teks `#0A0B0E`, ikon kecil). Teks bebas sesudahnya = argumen. **Backspace** pada awal teks menghapus pill. **Beberapa pill diperbolehkan** (urutan = rantai, mis. `/ringkas` `/terjemah`).
4. Klik pill → popover kecil (parameter opsional, mis. bahasa tujuan); lengkapnya tidak wajib.
5. Mengirim tanpa pill = Hime memilih skill sendiri (router). Pill = instruksi eksplisit.
6. **Perintah cepat** (bukan skill): `/stop` (hentikan run terakhir), `/jeda` (jeda penegakan, lihat 10), `/cari` (cari di thread).

**Kategori warna skill (selaras palet 04):** Tugas = sky · Waktu/Rutin = amber · File/Simpanan = mint · Dokumen = violet · Kode/Dev = lime · Web/Riset = cyan · Terjemah = pink · Perangkat = coral · Sistem = abu. Warna kategori **konsisten** di seluruh produk (pill di chat, ubin di Settings, kartu).

**Sumber skill:** Core Skill Registry (14 §4). Skill ditambah/diaktifkan **tanpa mengubah kode UI**; menu dibangun dari manifest (`id, nama, deskripsi, ikon, kategori, alias, parameter?`). Teks manifest tunduk pada batas teks 04 §6.

## 4. Lampiran (semua jenis file)
- **Cara menambah:** tombol `+` (pemilih file), **seret-lepas** ke notch/chat (notch otomatis membesar), **tempel** (Ctrl+V gambar/file/teks panjang), **dari Simpanan** (seret keluar-masuk), dari Explorer (konteks "Kirim ke Hime" — P2).
- **Tampilan:** chip lampiran (ikon tipe berwarna + nama dipotong + ukuran) dengan ✕; gambar punya thumbnail; banyak file → "+N".
- **Pemrosesan di Core (modul `attachments`):** simpan **referensi + hash + metadata**; salin ke `%APPDATA%/hime/attachments/` hanya bila perlu (file kecil < batas atau permintaan skill); deteksi tipe dengan **magic bytes** (bukan hanya ekstensi); thumbnail malas; **ekstraksi isi hanya oleh skill** (OCR, dokumen, dll.) sesuai izin.
- **Keamanan:** tidak pernah mengeksekusi atau membuka otomatis; peringatan untuk executable/skrip/arsip; batas ukuran dan jumlah (dapat diatur; default masuk akal, mis. 10 file/pesan); cegah zip bomb dan path traversal; file sensitif ditandai.
- **Privasi:** bila isi file akan dikirim ke penyedia AI cloud → indikator jelas "Dikirim ke AI" sebelum kirim; opsi "Tanya dulu" per tipe; kecualikan folder sensitif.
- Semua tipe didukung **sebagai lampiran**; seberapa jauh Hime memahami isi tergantung skill (jelaskan dengan jujur bila tipe belum bisa dibaca).

## 5. Suara (perintah suara)
- **Pemakaian:** tombol mikrofon (ketuk = mulai/berhenti) **atau tahan pintasan** (push-to-talk; 13). Saat merekam: mata `dengar` + cincin denyut; transkrip **muncul bertahap di input** dan dapat diedit; opsi "Kirim otomatis setelah jeda".
- **Bahasa:** Indonesia default (terdeteksi otomatis untuk campuran).
- **Mesin STT (pluggable, modul `voice`):** kandidat (1) **STT penyedia cloud** via Core (akurat, butuh internet, data suara terkirim — indikator jelas), (2) **Whisper lokal** (whisper.cpp; model kecil terkuantisasi **dimuat saat dipakai dan dibongkar setelah idle**). **Putuskan lewat pengukuran** (akurasi Indonesia, latensi, RAM, CPU pada i5-8250U) → ADR. Default aman: engine yang lolos anggaran dan privasi; owner dapat memilih.
- **Privasi:** opt-in; indikator mikrofon selalu terlihat; audio **tidak disimpan** (hanya transkrip); VAD memotong diam; izin mikrofon Windows ditangani anggun.
- **Aksesibilitas:** alternatif keyboard penuh; transkrip terbaca pembaca layar.

## 6. Multi-run: tidak perlu menunggu
**Konsep:** setiap pesan pengguna yang membutuhkan kerja menjadi **Run** di Core.
1. Kirim pesan → **konfirmasi instan** (≤ 1 dtk) dalam thread: kartu proses kecil ("Mengerjakan…") dengan progress pill, lalu input langsung **kosong dan siap**.
2. Pengguna boleh mengirim pesan berikutnya **kapan saja**. Core menjadwalkan: Run independen berjalan **paralel hingga batas** (default 2, diatur oleh pengukuran resource); sisanya **antre** dengan penanda "Antre #n". Run yang bergantung pada sumber daya sama diserialkan dengan catatan singkat.
3. **Kartu proses** (satu per run): judul pendek, status, progress/langkah (dari event), ✕ batal, "Lebih" (log aktivitas ringkas — bukan chain-of-thought). Saat selesai, kartu **berubah** menjadi hasil di tempat yang sama; urutan thread = urutan **mulai**, hasil ditandai waktu selesai.
4. **Pertanyaan balik dari Hime** (butuh klarifikasi) muncul sebagai kartu yang menjawab ke run terkait; run lain tidak terhenti.
5. **Steering:** pesan susulan yang jelas merujuk run berjalan ("batalkan yang tadi", "tambahkan juga X") dikenali router dan **diterapkan ke run itu** (bukan run baru) — atau Hime menanyakan singkat bila ambigu.
6. **Approval** dari run mana pun → bubble P0 (06) dan kartu inline; run menunggu tanpa memblokir yang lain.
7. **Batas aman:** jumlah run aktif, antrean, timeout, anggaran token; indikator saat penuh ("Penuh. Pesanmu antre."). Tidak ada kehilangan pesan.
8. **Pemulihan:** run dan antrean persisten (SQLite); setelah restart Core, run yang tersisa ditandai "Terhenti" dengan opsi lanjut/ulang; eksekusi tool tidak menggandakan efek (idempotensi, legacy 03).
9. **Mata:** `berpikir` selama ada run aktif; `senang` singkat saat selesai; `khawatir` bila gagal.

**Backend:** *Run Scheduler* di Core (modul `runs`): antrean prioritas, batas konkurensi, pembatalan, kebijakan serialisasi, event `run.*`. **Verifikasi** kemampuan Hermes memegang beberapa sesi bersamaan (08R); bila tidak, Core menjalankan **satu Hermes dengan beberapa sesi berurutan cepat** atau beberapa sesi terisolasi sesuai hasil pengukuran RAM — putuskan dengan bukti. Jalur cepat tanpa Hermes (terjemah, lirik, parsing tugas) **tidak** memakai antrean run berat.

## 7. Riwayat dan konteks
- Thread tak terbatas (retensi dapat diatur), gulir tak hingga (virtualisasi), pencarian (`Ctrl+F` / `/cari`).
- Konteks yang dikirim ke model dibatasi anggaran token: ringkasan bergulir + memori relevan (FTS) + pesan terbaru (Core, 09R).
- Tidak ada fitur "chat baru". Ada "Bersihkan tampilan" (menyembunyikan, bukan menghapus) dan "Hapus riwayat" (di Privasi).
- Lampiran dan hasil dapat disimpan ke **Simpanan** (seret dari pesan).

## 8. State dan kesalahan
Kosong ("Tanya apa saja." + 3 saran singkat sebagai pill), memuat, streaming, proses, antre, gagal ("Gagal. Coba lagi?" + tombol), offline ("Hime offline" + mata `offline` + "Coba lagi"), runtime AI mati, batas run penuh, file ditolak (alasan 1 baris). Tidak ada stack trace; tidak ada kebocoran rahasia.

## 9. Sinkron dengan Android
Thread, run, lampiran, dan skill sama (Core adalah sumber kebenaran). Android: input dengan `/` yang sama, mikrofon (SpeechRecognizer), lampiran (picker + **Bagikan ke Hime**). Lihat 16.

## 10. Akseptansi
- [ ] `/` membuka menu dari registry; pill berwarna; beberapa pill; backspace menghapus; filter; keyboard penuh.
- [ ] Skill yang modulnya OFF tidak muncul.
- [ ] Lampiran semua tipe (seret, tempel, picker, Simpanan); keamanan dan indikator privasi bekerja; file berbahaya tidak pernah dieksekusi.
- [ ] Suara: rekam → transkrip → edit/kirim; mikrofon indikator; audio tak tersimpan; engine sesuai ADR; model dibongkar setelah idle (diukur).
- [ ] **Multi-run:** kirim 3 pesan berturut-turut saat run pertama berjalan → konfirmasi instan, paralel/antre sesuai batas, hasil masuk thread, batal bekerja, pemulihan setelah restart, tidak ada efek ganda.
- [ ] Satu thread; tidak ada konsep workspace/chat baru.
- [ ] Teks UI lolos `check-copy`; desain lolos checklist 04 §9; **VC-U2** (chat).
