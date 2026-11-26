# PHASE 12 — PRODUCTIVITY, ACCOUNTABILITY & DOCUMENT SUITE
**Kanban & tugas · Pengingat & jadwal belajar · Mode ketegasan · Penggunaan layar · Layanan dokumen · Konversi/kompres/merge/OCR**

**Entry:** Fase 11 `READY FOR OWNER ACCEPTANCE` (Task Engine, Event Engine, Device Layer siap). Pekerjaan dokumen tidak bergantung pada mobile dan dapat dimulai paralel setelah Fase 09 bila perlu.
**Kebutuhan pengguna:** UR-06, UR-07, UR-09, UR-10, UR-11 (sisi laptop), UR-14, UR-15.

# BAGIAN A — PRODUKTIVITAS DAN AKUNTABILITAS

## A1. Task management & Kanban (UR-09)
- Entitas: proyek, task, prioritas, status, tenggat, dependensi, label (mis. mata kuliah), estimasi usaha, catatan, lampiran (referensi file).
- Kanban (parity Nusa + perbaikan): kolom dapat diatur, drag-drop, filter, pencarian, tampilan kalender.
- **Capture lewat chat/Omnibox:** "saya punya tugas X deadline Jumat" → Hime membuat task (konfirmasi singkat), menanyakan hal yang kurang (tenggat, prioritas) hanya bila perlu.
- Pemecahan tugas: Hime dapat mengusulkan subtask dan estimasi; pengguna menyetujui.

## A2. Pengingat dan penjadwalan (UR-10)
- Pengingat sekali/berulang (RRULE), zona waktu (default perangkat), notifikasi desktop (Windows toast via PAL) dan HP (via Device Layer).
- **Jadwal belajar:** pengguna menyatakan beban + ketersediaan → Hime menyusun blok belajar (aturan sederhana + LLM untuk penjelasan; **penjadwalan itu sendiri deterministik** dan dapat diubah pengguna), memperhitungkan tenggat dan prioritas; dapat dijadwal ulang saat meleset.

## A3. Accountability system — mode ketegasan (UR-10, UR-11)
Tiga mode, dapat diatur lewat **UI Settings dan lewat chat**, berlaku di desktop dan HP:

| Mode | Perilaku |
|---|---|
| **Santai** | Pengingat lembut, ringkasan harian, tanpa eskalasi |
| **Biasa (Balanced)** | Check-in saat laptop dibuka + pengingat berkala; eskalasi ringan bila tugas penting menunggak |
| **Strict** | Check-in wajib saat laptop dibuka (layar sapa berisi tugas prioritas + tombol "mulai sekarang" yang membuka konteks dan menyalakan timer fokus); pengingat sering; eskalasi bertahap bila tenggat mendekat |

Aturan desain (penting):
- **Hime menagih, bukan mengunci.** Tidak mengunci perangkat, tidak mematikan aplikasi, tidak memblokir akses darurat. "Tunda" selalu tersedia dengan **alasan singkat** yang dicatat (untuk refleksi, bukan hukuman).
- Strict adalah **pilihan pengguna** yang dapat dimatikan kapan saja; ganti mode butuh konfirmasi, bukan sihir.
- Nada: mendukung dan manusiawi, tidak menghakimi; tidak memanipulasi emosi.
- **Mode tidak mengubah izin keamanan** (jangan pernah melonggarkan Policy karena "strict/santai").
- Ulasan progres (harian/mingguan), penanganan tugas terlewat (jadwal ulang yang dapat dipilih), statistik penyelesaian.

## A4. Penggunaan layar dan pengalih (UR-11, sisi laptop)
- **Opt-in.** Lacak aplikasi/jendela aktif (PAL: foreground window) dan durasi; kategori (belajar/produktif/pengalih) dapat diatur; daftar pengecualian; data lokal terenkripsi, retensi terbatas, dapat dihapus; **tidak dikirim ke provider AI** tanpa persetujuan.
- Nudge bila melewati ambang (mis. waktu pada kategori pengalih saat ada tugas mendesak), sesuai mode. Nudge menawarkan aksi konkret ("mulai 25 menit fokus pada tugas X").
- Transparansi: halaman yang menunjukkan data apa yang dikumpulkan dan tombol hapus semua.

## A5. Pengujian bagian A
Skenario waktu (jam disimulasikan/dikontrol), zona waktu, DST, jeda tidur/hibernasi, kehilangan jaringan, banyak perangkat (hindari notifikasi ganda: satu sumber kebenaran di Core), uji mode (perilaku berbeda terbukti), uji privasi (data tak terkirim saat opt-out).

# BAGIAN B — LAYANAN DOKUMEN (UR-06, UR-07, UR-14, UR-15)

## B1. Arsitektur
- **Document Service** di Core mengorkestrasi **worker proses terisolasi** (Python venv terkelola untuk python-docx/openpyxl/python-pptx/pikepdf, serta tool CLI). Worker dibatasi resource (1 pemrosesan berat sekaligus), timeout, direktori kerja sandbox; input/output via berkas, **bukan** shell bebas.
- Semua operasi = tool via Gateway: `documents.create`, `documents.edit`, `documents.read`, `documents.convert`, `documents.compress`, `documents.merge_pdf`, `documents.split_pdf`, `documents.ocr`, `documents.extract`, `documents.render_preview`.
- **Verifikasi hasil wajib:** setelah membuat/mengubah dokumen, render ke PDF/gambar (LibreOffice headless) dan periksa (jumlah halaman, font tertanam, tidak ada teks meluap, gambar muncul). Hime melaporkan bukti verifikasi.

## B2. Tool open source (verifikasi lisensi sebelum dibundel; dipasang sebagai tool terkelola, bukan disalin sembarangan)
| Kebutuhan | Pilihan awal | Catatan |
|---|---|---|
| DOCX/XLSX/PPTX | python-docx (atau docx-js), openpyxl, python-pptx | pakai **styles**, bukan format manual |
| Konversi office↔PDF | LibreOffice headless | ukuran besar; instal terkelola, opsional |
| Teks/markup | pandoc | untuk markdown→docx/pdf |
| PDF (merge/split/rotate/enkripsi) | qpdf / pikepdf | lisensi permisif |
| Kompres PDF | Ghostscript | **AGPL**: jangan dibundel tanpa meninjau; panggil sebagai tool eksternal terpasang atau cari alternatif (qpdf optimasi + pengurangan gambar via Pillow) — putuskan lewat ADR |
| OCR | Tesseract (+ OCRmyPDF) | paket bahasa Indonesia/Inggris |
| Gambar | Pillow/ImageMagick | kompres/konversi gambar |
| Arsip | 7-Zip/zip bawaan | |

## B3. Standar kualitas hasil dokumen (UR-14) — wajib dipatuhi semua pembuat dokumen
**Prinsip:** profesional, rapi, mudah dibaca.
- **Struktur:** heading bertingkat konsisten (Heading 1–3 via style), judul, daftar isi otomatis untuk >5 halaman, nomor halaman, header/footer bila sesuai, penomoran gambar/tabel dan *caption*, referensi silang.
- **Tipografi:** font terbaca dan umum (default templat umum: Calibri/Aptos 11 atau Times New Roman 12 untuk akademik), spasi 1,15–1,5 sesuai templat, perataan konsisten, tanpa campuran font acak.
- **Tata letak:** A4, margin sesuai templat; **templat "Akademik-ID" (opsi)**: Times New Roman 12, spasi 1,5, margin kiri 4 cm / lainnya 3 cm — **hanya bila pengguna/dosen tidak memberi pedoman lain; pedoman dari pengguna selalu menang.**
- **Tabel/grafik:** header jelas, kolom sejajar, angka rata kanan, sumber data dicantumkan, grafik dengan sumbu berlabel dan kontras memadai.
- **Konten:** tidak mengarang data/sitasi; referensi diberi tanda jika belum diverifikasi; jelaskan asumsi; bahasa baku dan ringkas.
- **Presentasi (PPTX):** satu gagasan per slide, hierarki visual, maks. teks wajar, kontras, konsistensi master slide, catatan pembicara bila diminta.
- **Spreadsheet (XLSX):** header beku, format angka/tanggal benar, **rumus hidup bukan nilai tempel**, validasi data, sheet dokumentasi asumsi; tanpa nilai hardcode tersembunyi.
- **Aksesibilitas:** teks alternatif gambar, urutan baca logis, bahasa dokumen diset, metadata (judul/penulis).
- **Verifikasi visual** sebelum diserahkan (B1).

## B4. "Artcraft" (Wordcraft, Figcraft, dll.) — UR-06
Pengguna menyebut aplikasi office bernama **"Artcraft"** (mis. Wordcraft, Figcraft). Penulis paket **tidak dapat memverifikasi** produk tersebut. Langkah: (1) **discovery** — periksa apakah terpasang di mesin, format berkas, ada CLI/API/skrip/ekstensi? (2) jika punya antarmuka terprogram atau format terbuka → buat adapter di `services/documents/` dengan antarmuka yang sama seperti worker lain; (3) jika tidak → bekerja lewat format standar (DOCX/PPTX/PDF/SVG) yang bisa dibuka aplikasi itu, dan **catat keterbatasan**; jangan mengotomasi GUI aplikasi dengan klik buta. Hasil discovery masuk ADR dan dilaporkan; bila ambigu, buat entri di `OWNER-DECISIONS-NEEDED.md` (bukan Gate yang menghalangi).

## B5. Skill dokumen (lihat `07-…` B3)
`docs.create-report` (berbasis templat + outline yang disetujui), `docs.create-presentation`, `docs.create-spreadsheet`, `docs.convert`, `docs.compress`, `docs.merge-pdf`, `docs.ocr`, `docs.extract`. Laporan akademik: alur outline → draf per bagian → rujukan → periksa konsistensi → render → verifikasi.

## B6. Pengujian dokumen
Set dokumen sampel (kecil/besar/rusak/terenkripsi/berpassword); uji hasil dibuka di LibreOffice dan (bila tersedia) Office; uji kegagalan (file korup, disk penuh, timeout worker); uji keamanan (makro, tautan eksternal, zip bomb, path traversal dalam arsip, XXE pada XML); uji performa (ukuran halaman, RAM worker) — anggaran dari pengukuran.

# DELIVERABLES
Modul Productivity (Core + UI Workspace/Kanban/Kalender/Pengaturan Mode), Document Service + worker, templat dokumen (`resources/templates/`), skill, UI Documents, dokumentasi `docs/architecture/PRODUCTIVITY.md`, `docs/architecture/DOCUMENTS.md`, `docs/user/MODE-KETEGASAN.md`, `docs/legal/THIRD-PARTY.md` (lisensi tool), ADR (kompresi PDF, Artcraft, jadwal), laporan fase.

# EXIT GATE
Kanban + pengingat + jadwal belajar + tiga mode bekerja end-to-end di desktop dan HP (tanpa notifikasi ganda); data pemantauan opt-in dan dapat dihapus; alur dokumen menghasilkan DOCX/XLSX/PPTX/PDF yang **dibuka dan diverifikasi**; konversi/kompres/merge/OCR berfungsi pada sampel; seluruh output memenuhi standar B3; lisensi tool tercatat.

# DI LUAR CAKUPAN
Kolaborasi multi-pengguna, penyuntingan dokumen real-time, penyimpanan cloud dokumen (Fase 13), otomasi GUI aplikasi tertutup.
