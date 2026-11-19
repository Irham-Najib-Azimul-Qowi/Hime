# PHASE 10 — STORAGE INTELLIGENCE
**Indeks file · Analisis penyimpanan · Organizer cerdas · Operasi file aman · Intelijen unduhan · Backup & pemulihan**

**Entry:** Fase 09 `READY FOR OWNER ACCEPTANCE`. Tool Gateway + Policy + approval berfungsi.
**Kewajiban dibawa:** G9 (kebijakan trash/pemulihan), G13 (`hime-storage` diperdalam).
**Kebutuhan pengguna:** UR-01, UR-03, UR-04.
**Keselamatan uji:** seluruh pengujian pada **direktori sandbox buatan**. Operasi destruktif pada file pribadi nyata = Owner Gate #5.

## 1. Tujuan
Hime menjadi pengelola penyimpanan laptop yang **rapi, aman, dan interaktif**: ia **bertanya** di mana file disimpan, mengingat pilihan, menangkap unduhan baru/duplikat dan menanyakan keputusan pengguna, dan tidak pernah menyebabkan kehilangan data tanpa mitigasi.

## 2. Task

### 10.1 Filesystem indexing
- Scan awal + pembaruan inkremental via `notify` (ReadDirectoryChangesW) dengan debounce; `walkdir`/`ignore` untuk traversal; metadata (path kanonis, ukuran, mtime/ctime, ekstensi, MIME, atribut).
- **Batas izin:** daftar root yang disetujui pengguna (mis. Documents, Downloads, Desktop, folder kuliah); kecualikan folder sistem/aplikasi/`node_modules`/`.git`/cache secara default (dapat diatur).
- Indeks di SQLite (`indexed_files`) + **FTS5** untuk nama/path; teks isi dokumen opsional (ekstraksi ringan, dibatasi ukuran, opt-in).
- Resource: worker maks 2, prioritas rendah, jeda saat beban tinggi (anggaran RAM tambahan indeks diukur).

### 10.2 Storage analyzer
Penggunaan disk per folder/tipe, file besar, kandidat duplikat, umur file, statistik folder, "yang bisa dibersihkan" (cache, installer lama, unduhan lama). Visual di modul Storage (treemap/ringkasan sederhana). Semua angka dari scan nyata.

### 10.3 Deteksi duplikat dan versi
- Duplikat: kelompok by ukuran → hash parsial (BLAKE3 pada head/tail) → hash penuh. Tidak pernah menghapus otomatis.
- Versi: nama mirip (`laporan (1).docx`, `laporan_final_v2`), hash berbeda, mtime, metadata dokumen → **kandidat versi**; ambiguitas → tanya pengguna.

### 10.4 Smart organizer (percakapan + aturan)
- Hime mengusulkan folder tujuan + penamaan berdasarkan klasifikasi (tipe, proyek/mata kuliah, tanggal, kata kunci) dan **bertanya bila ragu**. Jawaban pengguna ("simpan semua file kuliah X di sini") menjadi **aturan** (`file_rules`) yang dapat dilihat, diedit, dinonaktifkan.
- Konvensi penamaan yang dapat diatur; deteksi konflik nama.
- Semua perubahan = **proposal → pratinjau → (approval) → eksekusi → verifikasi → laporan**.

### 10.5 Download intelligence (UR-03)
- Pantau folder unduhan; saat berkas baru: deteksi (a) nama sama/serupa, (b) hash sama (duplikat persis), (c) kandidat versi baru.
- Notifikasi/Island menanyakan: *"Berkas ini sudah ada. Ini versi terbaru?"* Pilihan: **ganti & arsipkan yang lama sebagai versi lama (`nama_v1`/dengan tanggal)**, simpan keduanya, hapus yang baru (dengan approval), abaikan, "selalu lakukan ini untuk pola ini".
- Tunggu unduhan selesai (file sedang ditulis/`.crdownload`/`.part`) sebelum bertanya; jangan mengganggu berulang (rate limit notifikasi).

### 10.6 Safe file operations (jantung fase ini)
Operasi: move, rename, copy, archive, delete-with-approval, restore.
- **Alur atomik:** staging → hash sumber → salin → flush → verifikasi hash tujuan → rename atomik → verifikasi → baru sumber dipindah ke **Hime Recycle** (bukan hapus langsung) → catat jurnal.
- **Jurnal operasi** (`file_operations`): ID, jenis, sumber, tujuan, hash, waktu, aktor (pengguna/agent/task), approval ID, status, data pemulihan. Jurnal tahan crash (transaksi + pemulihan pada startup: selesaikan atau rollback operasi setengah jadi).
- **Hime Recycle** (G9): lokasi terkelola, retensi dapat diatur (default 30 hari **dengan persetujuan pengguna di pengaturan**, bukan otomatis diam-diam), kapasitas dibatasi, notifikasi sebelum pembersihan, pemulihan satu klik; kegagalan disk penuh ditangani.
- **Undo** hanya bila secara teknis mungkin dan **dinyatakan jujur** (mis. jangan menjanjikan undo setelah Recycle dikosongkan).
- Windows khusus: path panjang, nama dicadangkan (CON/NUL), alternate data streams, hak akses, file terkunci proses lain, junction/symlink (tolak keluar root), kasus huruf, volume berbeda.

### 10.7 Backup & recovery
Kebijakan backup (folder, frekuensi, tujuan lokal/eksternal), restore, penanganan konflik, verifikasi integritas, uji pemulihan terjadwal. Integrasi Google Drive sebagai tujuan opsional menunggu Fase 13.

### 10.8 UI Storage
Dashboard analitik, daftar duplikat/versi, antrean proposal, editor aturan, timeline operasi (dari jurnal), pemulihan. State loading/kosong/error; keyboard.

### 10.9 Skill + tool
Tool: `storage.search_files`, `storage.inspect_file`, `storage.analyze`, `storage.propose_move`, `storage.execute_move` (butuh ticket proposal yang disetujui), `storage.restore`, `storage.find_duplicates`, `storage.handle_new_download`. Skill: lihat `07-…` B3. Semua lewat Tool Gateway.

## 3. Pengujian (wajib)
- **Sandbox** berisi ribuan file sintetis; uji kegagalan: kill proses di tengah operasi (setiap titik alur), disk penuh, file terkunci, izin ditolak, volume terlepas → verifikasi **tidak ada kehilangan data** dan pemulihan jurnal.
- Uji path jahat: traversal, symlink/junction escape, nama khusus Windows, ADS, path panjang.
- Uji duplikat/versi: berbagai pola nama; unduhan parsial.
- Uji agent: instruksi ambigu → Hime bertanya; instruksi di dalam isi file (prompt injection) tidak dieksekusi.
- Performa: throughput indeks, RAM saat indeks (ukur), CPU, dampak pada UI.

## 4. Deliverables
Modul `hime-storage` (diperdalam) + service Storage; skema DB + migrasi; tool/skill; UI; `docs/architecture/STORAGE-INTELLIGENCE.md`, `docs/runbooks/FILE-RECOVERY.md`, laporan fase.

## 5. Exit gate
Operasi file penting dapat diverifikasi; kegagalan tidak menyebabkan kehilangan data tanpa mitigasi; tindakan destruktif terlindungi approval; alur unduhan-duplikat bekerja end-to-end pada sandbox; indeks memenuhi anggaran resource yang diukur.

## 6. Di luar cakupan
Storage HP (Fase 11); sinkronisasi cloud (13); OCR/ekstraksi dokumen mendalam (12).
