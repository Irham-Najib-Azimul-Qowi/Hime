# 13 — PINTASAN KEYBOARD & SETTINGS

# BAGIAN A — PINTASAN (KEYBIND)

## A1. Prinsip
Cepat dihafal, tidak bentrok, dapat diubah, dan **ringan** (global lewat `RegisterHotKey` — event-driven, tidak mengintersepsi input; Bloom tetap memakai hook miliknya untuk fungsinya sendiri). Satu pintasan membuka **daftar semua pintasan**.

## A2. Peta default (global)
Awalan konsisten **`Ctrl+Alt+`** (tidak memakai tombol Win, yang dicadangkan Windows/Bloom Win+1..9). Verifikasi bentrok pada keyboard Indonesia/US dan pada aplikasi umum; semua dapat diubah.

| Pintasan | Aksi | Catatan |
|---|---|---|
| `Ctrl+Alt+Space` | **Buka/tutup chat Hime** (fokus ke input) | pintasan utama |
| `Ctrl+Alt+M` | **Suara**: tahan = push-to-talk; ketuk = mulai/berhenti | |
| `Ctrl+Alt+T` | **Terjemahkan** teks terpilih | |
| `Ctrl+Alt+N` | **Tugas baru** (fokus quick-add) | |
| `Ctrl+Alt+S` | **Simpanan** (buka panel) | |
| `Ctrl+Alt+V` | **Tempel dari Simpanan** (picker cepat; item terakhir dengan `Enter`) | |
| `Ctrl+Alt+F` | **Fokus**: mulai/jeda sesi belajar | |
| `Ctrl+Alt+L` | **Lirik** (tampil/sembunyi di panel Musik) | |
| `Ctrl+Alt+H` | **Buka/tutup Island** (panel terakhir) | |
| `Ctrl+Alt+Shift+P` | **Jeda Darurat** penegakan | sengaja lebih sulit ditekan tak sengaja |
| **`Ctrl+Alt+/`** | **Daftar pintasan** (cheat sheet) | juga `?` saat notch terbuka dan fokus bukan di input |

### Di dalam notch (lokal, hanya saat notch fokus)
| Tombol | Aksi |
|---|---|
| `Esc` | tutup/mundur satu tingkat |
| `←` `→` atau `Ctrl+[` `Ctrl+]` | pindah panel |
| `1`–`6` | lompat ke panel (bila fokus bukan di input) |
| `/` | menu skill (di chat) |
| `Enter` / `Shift+Enter` | kirim / baris baru |
| `Ctrl+Enter` | kirim dan tetap di input |
| `Ctrl+F` | cari (di chat/Simpanan) |
| `↑` `↓` `Enter` | navigasi menu/daftar |
| `Delete` | hapus item terpilih (dengan urungkan) |
| `Space` | tandai selesai (di daftar tugas) |
| `Ctrl+Z` | urungkan tindakan terakhir Hime/tugas (bila ada) |

## A3. Cheat sheet (daftar pintasan)
- **Muncul** sebagai kartu besar di notch (bukan window baru), animasi `notch.morph`; tutup dengan `Esc`/pintasan yang sama.
- **Isi:** dikelompokkan — Umum · Chat · Tugas · Simpanan · Terjemah · Musik · Penegakan. Setiap baris: **ikon + nama aksi + keycap** (komponen Keycap). Pencarian satu baris ("ketik untuk mencari aksi").
- **Dibaca dari pengaturan saat ini** (bukan teks tetap) → selalu akurat setelah diubah. Pintasan yang konflik/nonaktif ditandai (pill amber "Bentrok").
- Klik baris → mode **ubah** (rekam kombinasi baru; validasi bentrok; "Atur ulang"). Tombol "Semua default" dengan konfirmasi.
- Teks ringkas: judul "Pintasan", deskripsi kelompok ≤ 20 kar.

## A4. Manajer pintasan (teknis)
- Modul `keybinds` (shell Rust): registry aksi `{ id, nama, default, global|lokal, modul }`; **modul mendaftarkan** pintasannya lewat manifest (14); modul OFF → pintasan **dilepas** (`UnregisterHotKey`).
- Registrasi gagal (sudah dipakai aplikasi lain) → state "Bentrok" + saran alternatif; tidak crash; tidak diam.
- Daftar **terlarang/rawan** (mis. `Ctrl+Alt+Del`, kombinasi sistem) ditolak saat merekam.
- Tombol modifier-only tidak diizinkan; penanganan layout (AltGr) dicatat.
- Pintasan tersimpan di settings (`hime-key-<aksi>`), dapat diekspor (bukan rahasia).
- Uji: bentrok, rebind, modul OFF, beberapa monitor, aplikasi layar penuh (pintasan Hime tidak mengganggu game; opsi "Nonaktifkan di layar penuh").

## A5. Akseptansi pintasan
- [ ] Semua pintasan default berfungsi dan teruji di ≥ 3 aplikasi dan layar penuh.
- [ ] Cheat sheet terbuka dengan `Ctrl+Alt+/` dan `?`, menampilkan **pengaturan saat ini**, bisa mencari dan mengubah.
- [ ] Rebind, bentrok, modul OFF terbukti; tidak ada hook baru (hanya `RegisterHotKey`).
- [ ] **VC-U8** (cheat sheet + beberapa pintasan).

---

# BAGIAN B — SETTINGS (REDESAIN)

Pelajari `Settings.tsx` Bloom (H0) lalu **perbarui desain** dengan sistem 04. Tetap satu window `settings` (bukan banyak window).

## B1. Prinsip
**Ringkas dan mudah dipindai:** baris dengan ikon berwarna, judul pendek, deskripsi pendek, kontrol di kanan. Tidak ada dinding teks. Pencarian pengaturan (`Ctrl+K`). Perubahan **berlaku langsung** (tanpa tombol Simpan) dengan umpan balik halus; tombol "Atur ulang" per bagian.

## B2. Struktur
Sidebar dengan **ubin ikon berwarna** + judul (≤ 16 kar). Urutan:

| # | Bagian | Isi (ringkas) |
|---|---|---|
| 1 | **Umum** | Bahasa, mulai bersama Windows, nama tampilan |
| 2 | **Tampilan** | Tema (gelap/terang bila ada), warna aksen, ukuran teks, kepadatan, animasi (Penuh/Hemat/Mati) |
| 3 | **Notch** | Daftar & urutan panel, visibilitas (Tetap/Pintar/Peek), jam/format, **Mata di notch** (Mati/Jarang/Sedang/Sering), pandangan ikuti kursor, tutup otomatis |
| 4 | **Hime & AI** | Model/penyedia (via Core), kredensial uji (masuk vault), gaya jawaban (Ringkas/Biasa), run paralel, ingatan |
| 5 | **Chat** | Lampiran (batas), suara (mesin, bahasa, kirim otomatis), `/` skill (urutan), riwayat |
| 6 | **Modul** | **Saklar tiap modul** + estimasi RAM + status (lihat B4) |
| 7 | **Tugas & Rutinitas** | Kategori, aturan tenggat, rutinitas, jam tenang |
| 8 | **Fokus & Penegakan** | Tingkat per komitmen, daftar pengalih/belajar, daftar pelindung, Jeda, masa tunggu pelemahan, Riwayat penegakan |
| 9 | **Simpanan** | Tangkap clipboard (ON/OFF), kedaluwarsa, batas, aplikasi dikecualikan, pola sensitif |
| 10 | **Terjemah** | Mode pemicu, bahasa target, mode Cepat/Paham, cache, aplikasi dikecualikan |
| 11 | **Musik & Lirik** | lihat 12 §6 |
| 12 | **Pintasan** | daftar + ubah (sama dengan cheat sheet) |
| 13 | **Perangkat** | HP terpasang, pairing, izin, sinkron, cabut |
| 14 | **Hime bertanya** | Pendiam/Seimbang/Cerewet, jam tenang, jenis interaksi (ON/OFF per jenis), reset belajar |
| 15 | **Privasi & Data** | Apa yang dikumpulkan (tabel ringkas), hapus data per modul, ekspor non-rahasia |
| 16 | **Sumber Daya** | RAM/CPU per proses (langsung), **Mode Hemat** (Mati/Otomatis/Selalu), runtime AI idle-stop (menit) |
| 17 | **Tentang** | Versi, **Lisensi** (atribusi, termasuk Bloom), diagnostik, buka log, pulihkan taskbar |

Maks 17 bagian; yang jarang (Diagnostik) disembunyikan di Tentang. Bagian modul OFF tidak tampil (ringkasannya tetap di **Modul**).

## B3. Registry pengaturan (sumber tunggal)
`src/hime/settings/schema.ts`: setiap pengaturan = `{ key, type, default, min?, max?, options?, section, label, desc, icon, module?, requiresRestart?, secret:false }`. **Menghasilkan** UI, validasi, default, dokumentasi (`SETTINGS.md` bagian Hime lewat generator), dan migrasi. **Rahasia tidak pernah** di schema ini (vault Core). Label/desc tunduk copy-lint. Penulisan mengikuti pola Bloom (`localStorage` → `save_setting`), kunci `hime-*`.

Contoh kunci baru (ringkas; semua string di `settings.json` mengikuti pola Bloom):
| Kunci | Tipe/default | Label (≤ 20 kar) |
|---|---|---|
| `hime-eyes-ambient` | enum `off/rare/normal/often` · `normal` | Mata di notch |
| `hime-eyes-follow` | bool · `true` | Mata ikuti kursor |
| `hime-quiet-hours` | range · `23:00–06:00` | Jam tenang |
| `hime-chatty` | enum `quiet/balanced/chatty` · `balanced` | Seberapa cerewet |
| `hime-runs-parallel` | int 1–3 · `2` | Run paralel |
| `hime-voice-engine` | enum `auto/cloud/local` · `auto` | Mesin suara |
| `hime-voice-autosend` | bool · `false` | Kirim otomatis |
| `hime-clip-capture` | bool · `false` | Tangkap clipboard |
| `hime-clip-ttl-h` | int · `24` | Simpan sementara (jam) |
| `hime-translate-trigger` | enum `hotkey/auto` · `hotkey` | Pemicu terjemah |
| `hime-translate-target` | string · `id` | Bahasa target |
| `hime-translate-mode` | enum `fast/explain/auto` · `auto` | Mode terjemah |
| `hime-lyrics` | bool · `true` | Lirik |
| `hime-lyrics-online` | bool · `true` | Ambil lirik online |
| `hime-enforce-delay-min` | int · `10` | Tunggu sebelum melonggarkan |
| `hime-screentime` | bool · `false` | Pantau layar |
| `hime-save-mode` | enum `off/auto/on` · `auto` | Mode Hemat |
| `hime-ai-idle-min` | int · `10` | AI berhenti saat idle |
| `hime-module-<id>` | bool | (per modul) |
Tambahkan sesuai kebutuhan modul; **tiap kunci baru wajib masuk schema**.

## B4. Bagian "Modul" (kontrol ON/OFF)
Daftar modul sebagai Row: IconTile + nama + deskripsi pendek + **estimasi RAM** (`~12 MB`) + status (pill: Aktif/Berhenti/Gagal) + Toggle. Mematikan modul: konfirmasi ringkas bila ada ketergantungan ("Chat butuh Runs"), lalu efek **nyata dan segera** (14 §3). Modul inti (tidak dapat dimatikan) ditandai kunci.

## B5. Desain halaman
- Window gaya Settings Bloom (borderless/ikuti Bloom), sidebar kiri 200 px, konten kanan; baris 48 px; kelompok dengan judul kecil. Kontrol: Toggle (spring), Segmented, Slider dengan **nilai di pill**, Swatch warna aksen, Keycap-editor, Select bergaya.
- Animasi: ganti bagian (`panel.switch` ringan), kontrol (`snappy`). Reduce motion dihormati.
- **Pencarian** menyorot bagian/baris yang cocok.
- **Status hidup** pada baris yang relevan (mis. RAM saat ini, koneksi Core, baterai HP).
- **Tidak** ada teks panjang; tautan "Pelajari" tidak diperlukan.

## B6. Akseptansi settings
- [ ] Semua pengaturan lahir dari schema; `SETTINGS.md` dihasilkan; tidak ada kunci di luar schema.
- [ ] Perubahan langsung berlaku (termasuk lintas window) lewat sinkron Bloom; migrasi dari kunci lama ok.
- [ ] Bagian Modul: toggle benar-benar memuat/membongkar modul (RAM turun terukur).
- [ ] Pencarian, reset per bagian, ekspor non-rahasia.
- [ ] Tidak ada rahasia di `settings.json`/localStorage (diuji kanari).
- [ ] Teks lolos copy-lint; desain lolos checklist; **VC-U8** (Settings).
