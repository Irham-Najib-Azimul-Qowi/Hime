# 04 — DESIGN SYSTEM: HIME ISLAND

**Arah:** modern, ringkas, **berwarna dan kontras**, hidup tapi tidak bising. Satu bahasa visual untuk notch, Settings, hub, dan aplikasi Android.
**Prinsip pinjaman dari Bloom:** *mekanik* tetap (pill yang bermorfosis dengan spring, klik-untuk-buka, peek, scroll/swipe antar panel, bubble di samping pill). **Yang diperbarui:** warna, kontras, ikon, tipografi detail, komponen (pill, tombol, progress), dan kepadatan.
**Sumber nilai:** ukuran, radius, spring, dan font **dari kode Bloom** (H0). Nilai di bawah adalah titik awal yang sudah dihitung kontrasnya; kamu **wajib** menjalankan skrip kontras pada nilai final.

## 1. Filosofi (lawan dari slop)
1. **Satu hal per tampilan.** Judul, satu baris deskripsi, satu aksi utama.
2. **Warna bermakna, bukan hiasan.** Setiap warna menandakan kategori, status, atau aksi. Tidak ada warna "biar ramai".
3. **Kontras sebagai fitur.** Teks ≥ 4.5:1; teks kecil pada pill ≥ 7:1 bila memungkinkan.
4. **Bentuk konsisten.** Satu keluarga radius, satu keluarga ikon, satu skala jarak.
5. **Gerak = informasi.** Animasi menjelaskan perubahan (asal, tujuan, status), bukan pamer.
6. **Padat dan tenang.** Ruang cukup; tidak ada dekorasi tanpa fungsi.

## 2. Token (sumber tunggal `design/tokens.json` → CSS variables + tema Compose)

### 2.1 Permukaan dan teks (gelap; terang opsional jika Bloom mendukung tema terang)
| Token | Nilai awal | Catatan |
|---|---|---|
| `--surface-island` | `#000000` | warna notch; menyatu dengan bezel (ikuti Bloom) |
| `--surface-1` | `#0D0E12` | dasar panel |
| `--surface-2` | `#14161C` | kartu/baris |
| `--surface-3` | `#1B1E26` | hover/terangkat |
| `--line` | `#262A34` | garis halus (1 px) |
| `--text-1` | `#F4F6FA` | kontras 16.7:1 pada surface-2 |
| `--text-2` | `#A7AEBB` | 8.1:1 — **teks sekunder terkecil yang diizinkan** |
| `--text-3` | `#6F7785` | 4.0:1 — **hanya untuk dekorasi/nonaktif, bukan teks bermakna** |

### 2.2 Aksen (dipakai untuk kategori, status, skill, progress, mata)
| Nama | Nilai | Teks gelap `#0A0B0E` di atasnya | Teks aksen di tint 18% | Peran |
|---|---|---|---|---|
| mint | `#34E3A6` | 11.9 | 7.4 | **Primer Hime**, sukses |
| cyan | `#38D6F5` | 11.3 | 7.1 | info, mata default |
| sky | `#5AA9FF` | 8.0 | 5.4 | kategori |
| violet | `#A18BFF` | 7.2 | 5.0 | kategori (hemat) |
| pink | `#FF5C93` | 6.8 | 4.8 | kategori, penting |
| coral | `#FF7A59` | 7.7 | 5.3 | peringatan lembut |
| amber | `#FFC247` | 12.2 | 7.5 | tenggat dekat, perhatian |
| lime | `#A8EE4B` | 14.1 | 8.3 | progres, streak |
| red | `#FF4D5E` | 6.1 | **4.5** | bahaya/terlambat; untuk teks pada tint pakai `#FF6B78` |

(Rasio dihitung saat penyusunan paket pada latar `#14161C`; jalankan ulang untuk nilai final. Putih di atas aksen **gagal** (1.4–3.2) — **jangan** pakai teks putih pada pill berisi aksen.)

**Aturan warna:**
- Pill **berisi (solid)**: latar aksen + teks `#0A0B0E`. Dipakai untuk status aktif, skill terpilih, aksi utama.
- Pill **tint**: latar aksen 18% + teks aksen. Dipakai untuk kategori/label pasif.
- Pill **garis (outline)**: hanya untuk opsi yang bisa dipilih.
- Maksimal **3 warna aksen** pada satu tampilan (kecuali daftar kategori).
- Warna kategori tugas **ditetapkan pengguna** dari palet di atas; default berurutan mint, sky, amber, pink, lime.
- **Gradien dilarang** kecuali pada progress aktif (dua-stop satu hue yang sama, subtil) dan mata. Tidak ada gradien ungu-biru.
- **Glow** hanya untuk satu hal per layar: mata Hime atau elemen yang sedang menunggu aksi.

### 2.3 Geometri
| Token | Nilai awal |
|---|---|
| skala jarak | 4 · 8 · 12 · 16 · 24 (px; kelipatan 4) |
| radius | pill = penuh; kartu = 14; ubin ikon = 10; input = 12 (selaraskan dengan Bloom bila beda) |
| tinggi baris daftar | 44 px (target sentuh/klik nyaman; 40 minimum) |
| ukuran ikon | 16 (inline) · 20 (baris) · 24 (tombol) |
| ketebalan progress | 6 px (bar), cincin 4 px |
| garis | 1 px `--line` |

### 2.4 Tipografi
Pakai **font Bloom** (hasil H0) untuk konsistensi dan karena sudah dimuat. Skala: judul 14/600, isi 13/500, deskripsi 12/400 `--text-2`, label 11/600 uppercase tidak dipakai. Angka tabular (`font-variant-numeric: tabular-nums`) untuk waktu/hitungan. Satu keluarga font saja; subset latin.

## 3. Komponen (semua berada di `src/hime/ui/`; satu implementasi, dipakai bersama)

| Komponen | Spesifikasi singkat |
|---|---|
| **Pill** | 3 varian (solid/tint/outline), ikon opsional, tinggi 22–28, teks satu baris, ellipsis; varian *dapat-dihapus* dengan ✕ |
| **SkillPill** | Pill solid berwarna sesuai kategori skill; dipakai di input chat (lihat 07) |
| **Button** | `primary` (solid mint, teks gelap), `secondary` (tint), `ghost`; tinggi 32/36; label **satu kata** bila bisa |
| **IconTile** | ubin 28–32 px radius 10; latar aksen 16% + ikon aksen (duotone) — sumber warna utama di daftar |
| **Row** | IconTile + judul (≤ 16 kar) + deskripsi (≤ 36 kar) + trailing (toggle/chip/chevron) |
| **Toggle** | trek 36×20; ON = mint solid; OFF = `--surface-3`; animasi spring |
| **Segmented** | 2–4 opsi; terpilih = pill solid |
| **Progress bar** | 6 px, trek `--surface-3`, isi aksen solid (+gradien hue sama bila aktif); animasi spring; label angka di kanan (tabular) |
| **Progress ring** | 4 px; untuk Tugas hari ini, timer fokus |
| **Keycap** | kotak 22 px, `--surface-3`, garis bawah 2 px; menampilkan pintasan |
| **Bubble** | rounded-square gaya Bloom + ikon/mata + 1 baris; warna aksen sebagai tepi/ikon, **bukan** latar penuh |
| **Card** | `--surface-2`, radius 14, padding 12; tanpa bayangan berat |
| **Input** | tinggi 40, radius 12, fokus: garis mint 1.5 px |
| **Toast** | tidak dipakai; ganti bubble + opsi "Urungkan" |
| **Empty state** | ikon besar duotone + 1 baris + 1 tombol; tidak ada ilustrasi panjang |

Semua komponen punya state: default, hover, fokus-keyboard (garis jelas), aktif, nonaktif, loading, error. Target klik ≥ 32 px.

## 4. Ikon
- **Satu keluarga:** rekomendasi **Phosphor Icons** (bobot *duotone* untuk ubin berwarna dan empty state, *bold* untuk aksi kecil, *regular* untuk inline). **Verifikasi lisensi (MIT) dan kualitas di Stage R0**, bandingkan singkat dengan keluarga yang sudah dipakai Bloom dan satu alternatif (mis. Tabler) pada 5 ikon kunci; catat keputusan di ADR. Yang menentukan: kejelasan pada 16–20 px, konsistensi, dukungan duotone, tree-shaking.
- **Impor per-ikon** (tidak ada barrel import) agar bundel kecil. Subset hanya ikon yang dipakai.
- `design/icon-map.md`: satu tabel konsep → ikon (mis. Chat, Mikrofon, Lampiran, Tugas, Kalender, Rutin, Fokus, Terjemah, Simpanan, Musik, Lirik, Pengaturan, Pintasan, Perangkat, Penting, Hapus, Unduh, Aman). **Satu konsep = satu ikon di seluruh produk** (desktop dan Android).
- Dilarang: emoji sebagai ikon, ✨, ikon robot/otak AI klise. Gunakan `HimeEyes` untuk apa pun yang berkaitan dengan Hime.
- Android: konversi SVG → ImageVector/VectorDrawable dari sumber yang sama.

## 5. Layout notch (ringkas; rinci di 06)
Zona: **Kiri** (mata/jam/ikon status), **Tengah** (judul atau info utama), **Kanan** (indikator/jumlah). Bentuk pill tunggal yang bermorfosis; panel terbuka di bawah/ke dalam bentuk yang sama.

## 6. Aturan teks UI (WAJIB)

**Prinsip:** judul pendek, deskripsi pendek, tombol satu kata. Tidak ada paragraf, tidak ada penjelasan yang bisa dihilangkan.

| Elemen | Batas | Contoh baik | Contoh buruk |
|---|---|---|---|
| Judul panel/baris | ≤ 16 karakter | "Tugas", "Simpanan" | "Manajemen Tugas dan Pengingat Harian" |
| Deskripsi | ≤ 36 karakter, 1 baris | "3 tenggat minggu ini" | "Di sini kamu dapat melihat semua tugasmu…" |
| Tombol | ≤ 10 karakter | "Simpan", "Tunda", "Mulai" | "Klik di sini untuk menyimpan" |
| Bubble | ≤ 40 karakter | "File sudah ada. Ganti?" | kalimat majemuk |
| Placeholder | ≤ 28 karakter | "Tanya Hime…" | "Ketik pertanyaan atau perintahmu di sini" |
| Label pengaturan | ≤ 20 karakter + deskripsi ≤ 40 | "Mata di notch" / "Sesekali tampil" | |
| Error | ≤ 44 karakter, tanpa istilah teknis | "Hime offline. Coba lagi?" | stack trace/kode |

Gaya: Indonesia, huruf kecil/kapital kalimat, tanpa tanda seru berlebihan, tanpa emoji, tanpa basa-basi ("Tentu!", "Baik, saya akan…"). Angka dan waktu ringkas ("Jum 19:00", "2 hari").
**Registry string:** semua teks UI di `src/hime/strings/id.ts` dengan metadata `{ key, text, max, kind }`; `scripts/check-copy.ts` (bun) gagal bila ada yang melebihi batas atau string UI literal di luar registry. Android memakai `strings.xml` dari sumber yang sama (generator).
Teks panjang yang sah (mis. kebijakan privasi) hanya di *Tentang* dan dokumen, bukan di alur utama.

## 7. Kepadatan dan responsivitas
- Notch menyesuaikan lebar layar (1366×768 adalah target terkecil) dan DPI 100–200%. Panel besar maksimal ±60% lebar dan ±55% tinggi layar.
- Skala teks mengikuti pengaturan; tidak ada teks < 11 px.
- Mode terang: ikuti dukungan Bloom; jika tidak ada, tunda (jangan setengah jadi).

## 8. LARANGAN AI SLOP (baca dan patuhi — ini penyebab kegagalan sebelumnya)

**Slop** = tampilan generik yang dihasilkan tanpa pertimbangan: terlihat "dibuat AI". Jangan lakukan:
1. **Gradien pelangi/ungu-biru**, glow neon di mana-mana, "glassmorphism" berlebihan, blur besar (juga mahal bagi GPU UHD 620).
2. **Kartu bertumpuk** dengan ikon di lingkaran gradien, judul besar, deskripsi dua baris, dan tombol di setiap kartu.
3. **Emoji/✨/sparkle** sebagai ikon atau dekorasi. Ikon robot klise. Avatar bergradien.
4. **Dashboard** di tempat yang cukup memakai satu baris. Statistik hiasan tanpa keputusan.
5. **Teks pemasaran**, tagline, "Selamat datang di…", paragraf penjelasan, tooltip di mana-mana.
6. **Palet acak** atau terlalu banyak warna; warna tanpa arti; kontras rendah demi "estetik".
7. **Border-radius dan bayangan tidak konsisten**; padding berbeda-beda per komponen.
8. **Animasi** memantul berlebihan, selalu bergerak, berputar tanpa makna, durasi > 500 ms.
9. **Mock/placeholder** (angka palsu, "Lorem ipsum", tombol tak berfungsi, status palsu).
10. **Chrome window Windows** untuk notch/dock/overlay; persegi translucent besar berisi pill kecil.
11. **Sistem desain/pustaka UI baru** (MUI, shadcn, dll.) di atas milik Bloom; font tambahan.
12. Menyatakan "modern" tanpa bukti: **tidak ada screenshot nyata = tidak selesai.**

**Cara menghindarinya secara aktif:** mulai dari komponen paling sedikit; bandingkan berdampingan dengan panel Bloom/Hime lain; hapus satu elemen dari setiap tampilan lalu lihat apakah masih jelas; tanyakan pada setiap warna "apa artinya?"; jalankan checklist 04 §9.

## 9. Checklist desain (wajib, dengan screenshot)
- [ ] Hanya token dari `design/tokens.json`; tidak ada hex di komponen.
- [ ] Kontras diverifikasi skrip (`scripts/check-contrast`), AA pada semua teks bermakna.
- [ ] Satu keluarga ikon; semua dari `icon-map.md`; tidak ada emoji.
- [ ] Teks lolos `check-copy` (judul ≤ 16, deskripsi ≤ 36, tombol ≤ 10).
- [ ] Pill/progress/tombol memakai komponen bersama; tidak ada varian lokal.
- [ ] Maksimal 3 aksen per tampilan; tidak ada gradien/glow terlarang.
- [ ] State lengkap (idle, hover, fokus, aktif, loading, kosong, error, offline).
- [ ] Animasi memakai preset spring (05); `prefers-reduced-motion`/Mode Hemat dihormati.
- [ ] Diuji 1366×768, DPI 100/125%, gelap; dibandingkan berdampingan dengan panel lain.
- [ ] CPU/RAM idle tidak naik (bandingkan baseline).
- [ ] Owner Visual Check (VC) dikirim dan dijawab.
