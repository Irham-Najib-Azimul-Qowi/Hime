# 03 — BRANDING: HIME ISLAND

## 1. Tujuan
Tidak ada "Bloom" yang terlihat atau tersimpan di produk. Semua identitas = **Hime Island**; ikon bot klasik dan logo lama diganti **maskot dua mata**.

## 2. Batas hukum (GPL-3.0) — jangan dilanggar
Bloom berlisensi GPL-3.0. Menghapus *nama dan merek* dari tampilan produk itu boleh; **menghapus pemberitahuan hak cipta dan lisensi tidak boleh**. Karena itu:
- **Pertahankan:** `LICENSE` (GPL-3.0), pemberitahuan hak cipta di header berkas yang ada, riwayat git, dan berkas `NOTICE` baru: "Hime Island adalah karya turunan dari Bloom (GPL-3.0) oleh pengembang aslinya; perubahan oleh pemilik proyek ini." Tandai berkas yang diubah secara signifikan (komentar header singkat atau `docs/legal/MODIFICATIONS.md`).
- **Tambahkan** halaman *Tentang → Lisensi* (di Settings) yang mencantumkan komponen open source termasuk Bloom — **bukan** di UI utama.
- Jika produk didistribusikan ke orang lain: sediakan source (GPL). Untuk pemakaian pribadi tidak ada kewajiban distribusi.
- Catatan: ini panduan teknis, bukan nasihat hukum. Bila ragu, minta owner meninjau.

## 3. Inventaris rebrand (kerjakan dengan alat, bukan tebakan)
Jalankan pencarian **tanpa memperhatikan huruf besar/kecil** untuk "bloom" di seluruh repo (kode, konfigurasi, dokumen, aset, nama berkas, nama folder, metadata paket). Klasifikasikan di `docs/hime/brand/INVENTORY.md`:

| Kelas | Contoh | Tindakan |
|---|---|---|
| A. Teks terlihat pengguna | judul window, tooltip tray, label Settings, pesan, dialog, installer | Ganti ke "Hime Island"/"Hime" (lewat registry string) |
| B. Identitas teknis | `productName`, `identifier`, nama exe, nama paket npm/crate, nama installer (NSIS), nama mutex/pipe, AppUserModelID | Ganti: `productName` = "Hime Island"; exe `hime-island.exe`; identifier `local.hime.island` (tetap unik; jangan mengklaim domain milik orang lain) |
| C. Data tersimpan | kunci settings `bloom-*`, kunci `localStorage`, path `%APPDATA%/bloom`, nama class CSS `bloom-*` | Ganti ke `hime-*`; akar data `%APPDATA%/hime/` (U10); jika `%APPDATA%/bloom/settings.json` ada, migrasikan sekali lalu abaikan |
| D. Simbol internal | nama variabel/fungsi/komponen berisi "bloom" | Ganti dengan refaktor otomatis yang aman (rename simbol), dibuktikan build + test + diff terbatas |
| E. Aset | ikon, logo, gambar splash, favicon, gambar README/website | Ganti total ke aset Hime (bagian 5) |
| F. Dokumen | README, SETTINGS.md, CONTRIBUTING, RELEASE_CHECKLIST, docs/ | Tulis ulang untuk Hime Island; hapus tautan promosi Bloom |
| G. `website/` | situs promosi Bloom | **Hapus** (bukan bagian produk) atau ganti kosong; catat di CHANGES |
| H. Atribusi hukum | `LICENSE`, `NOTICE`, header hak cipta, `docs/legal/*`, `docs/hime/UPSTREAM.md` | **Dipertahankan** (daftar putih) |
| I. Jaringan | URL repo/updater/announcement/analytics Bloom | Sudah dinonaktifkan (H0.5); hapus referensi; tidak ada telemetri ke pihak lain |

## 4. Gerbang `scripts/check-brand.ps1`
Gagal (exit ≠ 0) jika menemukan "bloom" (case-insensitive) di luar **daftar putih** (kelas H + riwayat git). Juga memeriksa: `productName`, `identifier`, judul window, tooltip tray, metadata installer, ikon (hash tidak sama dengan ikon Bloom awal), `package.json`/`Cargo.toml` name/description/repository. Dijalankan di gerbang kualitas dan sebelum commit.

## 5. Aset identitas (maskot dua mata)
Spesifikasi maskot lengkap di `05-MOTION-AND-EYES.md`. Aset yang harus dibuat:

| Aset | Spesifikasi |
|---|---|
| **Logo mark** | Dua mata berbentuk kapsul tegak berdampingan, warna aksen utama (mint/cyan, lihat 04), di atas latar squircle gelap. Sederhana, terbaca di 16 px. Satu SVG sumber di `design/brand/hime-mark.svg` |
| **Ikon aplikasi** | Dari mark → PNG 1024 → `bun run tauri icon` (hasilkan `.ico`, PNG berbagai ukuran) → ganti semua ikon Bloom |
| **Ikon tray** | Mata monokrom, adaptif terang/gelap, ≥ 16/20/24/32 px; keadaan: normal, bekerja, perlu perhatian, offline |
| **Avatar chat** | Mata dalam lingkaran/squircle kecil (mengikuti ekspresi saat tampil di chat) |
| **Logo installer/NSIS** | Header/sidebar installer memakai mark |
| **Android** | Adaptive icon (foreground mata, background gelap) + monokrom untuk themed icon; notifikasi small icon monokrom |
| **Wordmark** | "Hime Island" dalam font UI sistem desain; hanya dipakai di Tentang dan installer |

**Penggantian ikon bot lama:** semua ikon "bot klasik" (robot kotak, sparkle ✨, kepala bot) diganti `HimeEyes` (komponen) atau mark statis di tempat yang tidak beranimasi. Larangan: emoji sebagai ikon.

## 6. Nama di dalam sistem AI
Nama "Hime" ditanamkan ke persona Hermes dan seluruh pesan sistem (lihat `17-…`). Nama **"Hime Island"** dipakai untuk aplikasi; **"Hime"** untuk asisten.

## 7. Akseptansi
- [ ] `check-brand` hijau; pencarian "bloom" di luar daftar putih = 0.
- [ ] Ikon exe/tray/installer/window baru; tidak ada ikon Bloom tersisa (periksa hasil build, bukan hanya sumber).
- [ ] Settings → Tentang menampilkan nama, versi, dan Lisensi dengan atribusi.
- [ ] Settings/data baru di `%APPDATA%/hime/`; migrasi sekali jalan teruji (ada/tidak ada berkas lama).
- [ ] Installer berjudul Hime Island; menu Start, pintasan, startup entry, dan Add/Remove Programs berlabel Hime Island.
- [ ] `docs/legal/` dan `NOTICE` ada; GPL tidak dilanggar.
- [ ] Owner Visual Check **VC-R0** berisi screenshot logo/ikon/tray/installer.
