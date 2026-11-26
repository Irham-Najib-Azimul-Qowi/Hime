# 08 — SIMPANAN (SHELF + RIWAYAT CLIPBOARD)

**Simpanan** = tempat sementara untuk **teks dan file** yang bisa diambil dan **diseret ke mana saja**. Berlaku **global** (tidak hanya di chat): tersedia dari notch kapan saja. Memperluas **shelf Bloom** (yang sudah ada: drop file/teks + preview) — pelajari dulu di H0 dan **bangun di atasnya**, jangan membuat shelf kedua.

## 1. Perilaku inti
| Aksi pengguna | Hasil |
|---|---|
| **Seret** file/teks/gambar/tautan ke pill atau area notch | Zona jatuh membesar → item masuk Simpanan (animasi `shelf.drop`) |
| **Salin** (Ctrl+C) teks/gambar/file | Bubble kecil P4 "Disalin" (ikon tipe + 3–4 kata pertama, ≤ 2 dtk, tidak merebut fokus) → item otomatis masuk **Riwayat** |
| Buka panel **Simpanan** | Daftar item terbaru di atas; chip filter (Semua · Teks · File · Gambar · Tautan); pencarian; **Disematkan** di bagian atas |
| **Seret item keluar** | Ke Explorer/aplikasi lain (file/gambar), ke kolom teks (teks), ke **chat** (jadi lampiran), ke **kartu Tugas** (jadi tugas dari teks) |
| Klik item | Salin ulang ke clipboard (umpan balik singkat) |
| Aksi item (hover) | Salin · Sematkan · Tanya Hime · Simpan ke… (folder via skill storage) · Hapus |
| Tempel dari Simpanan | Pintasan "tempel terakhir" / pilih item lalu seret; opsional picker cepat (13) |

**Dua tingkat penyimpanan:**
- **Sementara** (Riwayat clipboard + item yang diseret): kedaluwarsa otomatis (default **24 jam**, dapat diatur), batas jumlah (mis. 50 item) dan ukuran.
- **Disematkan:** tidak kedaluwarsa sampai dihapus; batas lebih longgar.

## 2. Tipe dan penyimpanan
| Tipe | Penyimpanan | Catatan |
|---|---|---|
| Teks | SQLite (Core), terenkripsi at-rest opsional (DPAPI) | pratinjau 1–2 baris; deteksi tautan/kode/nomor (ikon berbeda) |
| Gambar (clipboard) | berkas di `%APPDATA%/hime/shelf/` + thumbnail kecil malas | batasi ukuran/resolusi simpan |
| File | **Referensi ke jalur asli** (default, hemat ruang) atau **salinan** ke `shelf/` (pilihan "Salin ke Hime", otomatis untuk file di lokasi sementara/Unduhan) | tampilkan status "file sumber hilang" |
| Tautan | teks + judul (opsional, diambil Core saat diminta; tidak otomatis) | |

Metadata: id, tipe, sumber (aplikasi aktif saat salin, bila diizinkan), waktu, hash, sematan, kedaluwarsa. Semua lokal.

## 3. Privasi dan keamanan (penting)
- **Opt-in** untuk **menangkap clipboard otomatis** (default: ON hanya setelah owner menyalakannya di pengaturan pertama; jelaskan singkat). Drag-and-drop manual tidak butuh opt-in.
- **Hormati penanda pengecualian Windows** pada data clipboard (mis. format `ExcludeClipboardContentFromMonitorProcessing`, `CanIncludeInClipboardHistory` = 0) yang dipakai pengelola kata sandi — **verifikasi** nama format di dokumentasi Windows; item seperti itu **tidak** ditangkap.
- **Daftar aplikasi dikecualikan** (pengelola sandi, bank, dll.; dapat diatur), jendela kolom password tidak ditangkap.
- **Deteksi pola sensitif** (kunci API, nomor kartu, token): jangan simpan otomatis atau simpan dengan label "Sensitif" dan kedaluwarsa cepat (1 jam) — pilihan di pengaturan; jangan pernah kirim ke AI tanpa persetujuan.
- Tidak ada pengiriman ke cloud kecuali pengguna memakai **Tanya Hime** pada item itu (dengan indikator).
- Hapus semua / hapus per item; kedaluwarsa nyata (dihapus dari disk, bukan disembunyikan).
- File dari Simpanan **tidak pernah dieksekusi** otomatis; saat diseret keluar = operasi OS biasa.

## 4. Teknis (verifikasi di Windows)
- **Clipboard:** `AddClipboardFormatListener` (event `WM_CLIPBOARDUPDATE`) — **tanpa polling**. Baca format yang relevan (teks Unicode, `CF_HDROP` untuk file, `CF_DIB/PNG` untuk gambar). Debounce 150 ms; abaikan salinan yang dibuat Hime sendiri (penanda sumber).
- **Seret keluar ke aplikasi lain:** webview Tauri tidak menyediakan drag-source native untuk file → pakai plugin/crate drag (kandidat: `tauri-plugin-drag`/`drag-rs`) **atau** implementasi `DoDragDrop` (OLE) di Rust lewat crate `windows` (tambahkan fitur yang dibutuhkan, pin 0.61). **Verifikasi opsi yang bekerja** dengan versi Tauri/wry yang dipakai; catat di ADR. Teks: sediakan `CF_UNICODETEXT`; file: `CF_HDROP`; gambar: format DIB/PNG.
- **Seret masuk:** event drag-drop webview/Tauri untuk file; teks via data transfer; zona jatuh dikoordinasi NCM.
- **Penyimpanan:** modul Core `shelf` (SQLite + berkas), API: `shelf.list/add/remove/pin/search/export`. Shell hanya menampilkan.
- **Sinkron ke HP:** ringkasan item **yang dipilih/disematkan** dapat dikirim ke Android (16: "Bagikan ke Hime" dua arah); default tidak menyinkronkan riwayat clipboard otomatis (privasi, bandwidth).

## 5. UI (panel Simpanan)
- Header: judul "Simpanan" + jumlah + tombol "Hapus semua" (ikon).
- Chip filter (Pill outline/solid), pencarian satu baris.
- Daftar baris 44 px: **IconTile tipe** (teks=sky, file=mint, gambar=violet, tautan=cyan) + 1 baris pratinjau + waktu singkat ("2 mnt") + ikon sematan; hover menampilkan aksi.
- Grid kecil untuk gambar (opsional).
- Kosong: ikon + "Seret atau salin sesuatu." (≤ 36 kar).
- Bubble salin: ikon tipe + pratinjau pendek; klik = buka Simpanan.
- Tidak ada teks penjelasan panjang.

## 6. Interaksi dengan modul lain
Chat (lampiran), Tugas (teks → tugas), Terjemah (hasil → Simpan), Storage (simpan ke folder), Android (bagikan), Hime proaktif (mis. "Ada URL disalin. Simpan sebagai tugas?" — hanya bila diminta/opt-in; 11).

## 7. Resource
Modul malas dimuat; listener clipboard satu-satunya "pekerja" aktif (event-driven); thumbnail on-demand; batas ukuran/jumlah; pembersihan kedaluwarsa dengan timer terkoalesensi (1 jam sekali); tidak menyimpan biner besar di SQLite. Estimasi RAM modul dicatat di manifest dan diverifikasi.

## 8. Akseptansi
- [ ] Seret masuk (file/teks/gambar/tautan) dan keluar (Explorer, editor teks, chat, Tugas) terbukti **pada aplikasi nyata** (screenshot/rekaman langkah).
- [ ] Salin otomatis (opt-in) → bubble → riwayat; **tidak** menangkap dari pengelola sandi (diuji dengan format pengecualian); aplikasi dikecualikan bekerja.
- [ ] Kedaluwarsa dan hapus nyata; sematan; pencarian; filter.
- [ ] Tidak ada polling; CPU idle tidak naik; RAM modul sesuai anggaran.
- [ ] Modul OFF → listener dilepas, tidak ada biaya.
- [ ] Teks UI lolos copy-lint; desain lolos checklist; **VC-U3** (Simpanan + seret).
