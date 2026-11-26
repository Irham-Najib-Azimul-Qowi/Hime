# 09 — TERJEMAH & PENJELASAN DI NOTCH

Fitur wajib: **saat pengguna memblok teks, Dynamic Island menampilkan terjemahan, dengan penjelasan singkat di bawahnya.** Harus cepat, nyaman, dan tidak mengganggu.

## 1. Pengalaman (dirancang)

**Pemicu (pilih satu atau dua di Settings):**
| Mode | Cara kerja | Default |
|---|---|---|
| **Pintasan** | Blok teks → tekan `Ctrl+Alt+T` | **ON** |
| **Otomatis** | Setelah teks diblok dan **tahan ±600 ms** (tanpa menggeser), notch menampilkan terjemahan **hanya bila** teks terdeteksi bukan bahasa target dan cukup panjang (≥ 2 kata atau 1 kata asing) | OFF (opt-in) |
| **Menu notch** | Klik bubble "Terjemah?" yang muncul sesaat setelah seleksi asing (P4, auto-hilang) | OFF |

**Tampilan (kartu terjemah, dari bubble yang membesar; P4):**
```
┌ Inggris → Indonesia        [ salin ][ simpan ][ ✕ ]┐   ← header: pill bahasa (sumber → target), aksi ikon
│ Terjemahan besar & jelas (maks 3 baris, 14/600)    │
│ ───────────────────────────────                    │
│ Arti singkat · catatan 1–2 baris (12, text-2)      │   ← PENJELASAN
│ [ Detail ▾ ]            [ Tanya Hime ]             │
└────────────────────────────────────────────────────┘
```
- **Terjemahan dulu** (cepat, streaming); **penjelasan** menyusul di bawahnya (skeleton 1 baris sampai tiba).
- **Penjelasan menyesuaikan jenis teks** (ditentukan Core dari panjang/jenis):
  | Jenis | Penjelasan yang ditampilkan (ringkas) |
  |---|---|
  | **Kata/frasa** | kelas kata, arti utama, 1 contoh pendek, nuansa/idiom (bila ada); pelafalan bila relevan |
  | **Kalimat** | terjemahan natural + (opsional) arti harfiah bagian penting; 1–2 poin tata bahasa/kosakata kunci |
  | **Paragraf** | terjemahan + ringkasan 1 kalimat + istilah sulit (maks 3) |
  | **Sudah bahasa Indonesia** | tawarkan **Jelaskan** (arti/penyederhanaan) atau **Ke Inggris**; tidak menerjemahkan diam-diam |
  | **Kode/teknis** | terjemah komentar/istilah; jangan mengubah kode |
- **"Detail ▾"** membuka penjelasan lengkap di kartu yang sama (tetap ringkas, scroll internal); **"Tanya Hime"** membuka chat dengan teks sudah terisi sebagai konteks (satu thread, tanpa workspace baru).
- **Auto-tutup** 8–10 dtk bila tidak dihover; hover menahan; klik pin; `Esc` menutup; seleksi baru mengganti kartu (tanpa menumpuk).
- Bila **chat sedang terbuka**: kartu terjemah muncul di bagian atas notch dan chat dipertahankan, kembali setelah ditutup.
- Mata: `penasaran` sesaat saat teks asing terdeteksi (jika ambient diizinkan).
- Pengaturan cepat di kartu: ikon bahasa target (ganti) dan mode **Cepat/Paham** (menyembunyikan/menampilkan penjelasan).

## 2. Bahasa dan mode
- Target default: **Indonesia**; sumber **otomatis**. Daftar bahasa umum + pencarian.
- **Cepat** = terjemahan saja (model cepat/murah); **Paham** = + penjelasan. Default **Paham** untuk kata/frasa pendek, **Cepat** untuk teks panjang (otomatis; dapat diubah).
- Menyimpan kamus kecil pribadi (kata yang sering): modul opsional P2 (jangan dibuat di v1 kecuali waktu cukup).

## 3. Teknis

### 3.1 Menangkap teks terpilih (Windows; verifikasi)
Urutan strategi (modul `selection` di shell/Core, lewat PAL):
1. **UI Automation `TextPattern`** (`GetSelection`) dari elemen terfokus — tidak menyentuh clipboard; bekerja di banyak aplikasi (Edge/Chrome dengan aksesibilitas, Office, editor).
2. **Fallback clipboard**: kirim `Ctrl+C` tersimulasi → baca → **pulihkan isi clipboard asli** (semua format) dalam ≤ 200 ms; tandai salinan sebagai internal (jangan masuk riwayat Simpanan); hindari bila fokus di kolom password/aplikasi dikecualikan.
3. Tidak tersedia → bubble "Teks tak terbaca di aplikasi ini" (≤ 40 kar) + saran salin manual.
- **Pemicu otomatis:** gunakan event UIA (`TextSelectionChanged` bila didukung) dengan debounce; **jangan polling**. Jika tidak andal, mode otomatis dinyatakan terbatas.
- **Jangan** mengambil teks dari kolom password, aplikasi dalam daftar kecuali, atau saat mode Jangan Ganggu.

### 3.2 Terjemahan (Core, **jalur cepat** — tidak memakai Hermes)
- Modul Core `translate`: panggilan **ringan** ke penyedia model lewat jalur yang sama dengan adapter kredensial (IB-01 tetap berlaku). Prompt tetap, keluaran terstruktur (`translation`, `detected_lang`, `kind`, `explanation{...}`), streaming.
- **Deteksi bahasa** lokal cepat (heuristik/n-gram ringan) sebelum memanggil model untuk memutuskan "sudah bahasa target".
- **Cache** (SQLite, kunci hash teks+bahasa+mode, batas ukuran, TTL) → terjemahan berulang instan dan hemat biaya.
- Timeout, pembatalan saat seleksi baru, penanganan offline ("Offline. Coba lagi?"), batas panjang (potong dengan catatan).
- Tidak menjalankan Hermes/9Router bila belum hidup: jalur cepat harus **tidak memicu** startup runtime berat (lihat 15).

### 3.3 Privasi
- **Opt-in**; indikator kecil "ke AI" pada kartu; daftar aplikasi dikecualikan; tidak ada penyimpanan teks seleksi selain cache (dapat dimatikan dan dihapus); tidak bekerja pada password.

## 4. Settings (Terjemah)
Aktif (modul), Mode pemicu (Pintasan/Otomatis), Bahasa target, Mode (Cepat/Paham/Otomatis), Auto-tutup (detik), Aplikasi dikecualikan, Cache (ON/hapus), Pintasan.

## 5. Akseptansi
- [ ] Blok teks → `Ctrl+Alt+T` → kartu di notch ≤ ~1.5 dtk untuk teks pendek (terukur; model dan jaringan menentukan; laporkan angka nyata).
- [ ] Kata, kalimat, paragraf, teks Indonesia, kode: perilaku penjelasan sesuai tabel.
- [ ] Teruji di ≥ 4 aplikasi (browser, PDF reader, editor teks/kode, Office/Word); fallback clipboard memulihkan clipboard asli (uji isi beragam: teks, file, gambar).
- [ ] Tidak aktif di kolom password/aplikasi dikecualikan; mode otomatis tidak menimbulkan kilatan/perebutan fokus.
- [ ] Cache, batal, offline, runtime mati: perilaku jujur.
- [ ] Kartu mengikuti desain (04), teks lolos copy-lint, animasi `translate.expand`; **VC-U3**.
