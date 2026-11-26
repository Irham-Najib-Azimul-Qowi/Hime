# 12 — MUSIK & LIRIK

Tambahkan **lirik tersinkron** pada pemutar musik Bloom. Pelajari dulu implementasi musik Bloom (COM/GSMTC media session, WASAPI visualizer) di H0 dan **perluas**, jangan menggantinya.

## 1. Pengalaman
- **Panel Musik:** tombol/tab **Lirik** (ikon map "lirik"; pintasan `Ctrl+Alt+L`). Tampilan: baris aktif **besar dan terang** (`text-1`), baris sebelum/sesudah redup (`text-2` ≈ 50%), gulir vertikal spring (`lyric.scroll`). Judul/artis/sampul tetap di atas (ringkas).
- **Pill tertutup:** saat lagu memutar dan lirik tersedia, **tengah pill** menampilkan **baris lirik aktif** (1 baris, ellipsis/marquee halus hanya bila perlu). Bila ada presentation lebih penting (P0–P4) → lirik menyingkir lalu kembali.
- **Karaoke** (opsional, bila data per-kata ada): isi warna per kata; jika tidak, per baris.
- **Offset:** `[` / `]` (atau ±) menggeser waktu 0.5 dtk bila lirik terlambat/terlalu cepat; tersimpan per lagu.
- **Tidak ada lirik:** state ringkas "Lirik tak ditemukan." + "Coba lagi"; **instrumental:** "Instrumental."
- Tema: aksen mengikuti warna dominan sampul **atau** mint default (hindari gradien berat; pengambilan warna ringan dan di-cache).

## 2. Sumber dan urutan
1. **Tag tertanam** di berkas lokal (USLT/SYLT/ID3, Vorbis `LYRICS`) bila pemutar menyediakan jalur berkas.
2. Berkas **`.lrc`** di sebelah berkas audio.
3. **Cache lokal** (SQLite Core).
4. **LRCLIB** (`https://lrclib.net/api`): `GET /api/get` dengan `track_name`, `artist_name`, `album_name`, `duration` (detik); respons memuat `syncedLyrics` (format LRC, bila ada), `plainLyrics`, `instrumental`. Fallback `GET /api/search` bila `get` tidak menemukan. **Sertakan User-Agent** yang jelas (`HimeIsland/<versi>`).
- **Catatan lisensi/penggunaan:** dokumentasi komunitas LRCLIB menyatakan lirik disediakan untuk penggunaan pribadi/edukasi, **bukan komersial**. Hime Island adalah penggunaan pribadi owner; jangan mengunggah/menerbitkan lirik, jangan mendistribusikan cache. **Verifikasi syarat terbaru di lrclib.net** saat implementasi dan catat di `docs/legal/`.
- **Aturan keamanan jaringan:** panggilan hanya dari **Core**, ke **host allowlist** (`lrclib.net`), dengan timeout, batas laju, dan tanpa mengirim data selain metadata lagu. **Dapat dimatikan** (Settings → Musik → "Ambil lirik online").

## 3. Pencocokan lagu
Ambil metadata dari media session (judul, artis, album, **durasi**). Normalisasi: hapus "(Official Video)", "[Lyrics]", "feat./ft.", sufiks remaster, kapitalisasi, spasi; coba kombinasi (dengan/ tanpa album). Terima hasil bila selisih durasi ≤ 2 dtk (atau pilih hasil terdekat). Cache hasil **termasuk "tidak ditemukan"** (TTL pendek) agar tidak menembak ulang.

## 4. Sinkronisasi waktu
- Posisi dari **timeline media session** (posisi + waktu pembaruan); ekstrapolasi monotonik di sisi frontend antar pembaruan; **koreksi saat seek/jeda/ganti lagu**.
- Parsing LRC di Core (Rust, **tes properti**: multi-timestamp per baris, milidetik, tag meta, baris kosong); kirim ke shell array `[ {ms, text} ]` terurut.
- Frontend menghitung indeks aktif dengan pencarian biner; **tidak ada loop per-frame saat tersembunyi**; saat terlihat: pembaruan hanya saat indeks berganti (timer berikutnya dijadwalkan ke `ms` baris berikut, bukan polling).
- Gulir/animasi hanya pada perubahan baris.

## 5. Teknis dan resource
- Modul Core `lyrics` (cache SQLite, parser, klien HTTP ber-allowlist) + modul shell `music-lyrics` (UI). OFF = tidak ada permintaan, tidak ada timer.
- Tidak memuat font/ikon tambahan berat; dukung aksara non-Latin lewat fallback font sistem (uji Jepang/Korea/Arab sederhana).
- Visualizer WASAPI Bloom tidak diubah; lirik tidak boleh menaikkan CPU idle di atas anggaran (ukur dengan lagu diputar vs tidak).

## 6. Settings (Musik)
Lirik (ON/OFF), Ambil lirik online (ON/OFF), Tampil di pill (ON/OFF), Karaoke (ON/OFF), Ukuran teks, Offset default, Hapus cache lirik.

## 7. Akseptansi
- [ ] Lagu lokal dan streaming (Spotify/browser/pemutar lain yang ditangkap Bloom): lirik tersinkron tampil; baris aktif akurat ±200 ms (uji seek/jeda/ganti lagu).
- [ ] Sumber berurutan benar; cache; instrumental; tidak ditemukan; offline.
- [ ] Pill menampilkan baris aktif dan menyingkir untuk presentation lebih penting.
- [ ] Parser LRC lulus tes properti; pencocokan judul lulus tabel kasus (judul berimbuhan).
- [ ] Tidak ada polling; CPU idle ~ baseline; OFF = nol aktivitas.
- [ ] Teks UI lolos lint; desain lolos checklist; **VC-U7** (Musik + lirik).
