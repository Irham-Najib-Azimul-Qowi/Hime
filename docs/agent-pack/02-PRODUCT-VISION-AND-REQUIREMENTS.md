# 02 — PRODUCT VISION & REQUIREMENTS

## 1. Visi

**Hime** adalah *personal AI operating environment* untuk satu pengguna (mahasiswa Teknologi Rekayasa Perangkat Lunak). Hime menyatukan interaksi pengguna, kecerdasan agent, pengelolaan perangkat (laptop dan HP Android), produktivitas, dan pekerjaan software engineering dalam satu ekosistem yang aman dan terkendali.

Hime **bukan** chatbot yang ditempel ke desktop dan **bukan** replika Nusa dengan tambahan AI. Nusa (aplikasi Dynamic Island untuk Windows, source tidak tersedia) hanya menjadi referensi pengalaman; Hime direimplementasikan dari spesifikasi dan desain sendiri.

### Tiga fondasi + satu pusat kendali

| Komponen | Peran | Perlakuan |
|---|---|---|
| Nusa (referensi UX) | Pengalaman Dynamic Island | Direkonstruksi dari hasil reverse engineering; tanpa menyalin aset berhak cipta |
| Hermes Agent (Nous Research, Python) | Agent runtime: reasoning, tool calling | Dipakai apa adanya, diintegrasikan lewat adapter |
| 9Router (Node.js) | Gateway model/provider AI | Dipakai apa adanya, diintegrasikan lewat adapter, diamankan |
| **Hime Core** (Rust) | Orkestrasi, kebijakan, task, event, memori, supervisor | Dibangun sendiri |
| **Hime Desktop** (Tauri 2 + React + TS) | Antarmuka | Dibangun sendiri |

## 2. Profil pengguna dan konteks

- Pengguna: mahasiswa; pekerjaan sehari-hari mencakup tugas kuliah, laporan, presentasi, pengembangan software, dan manajemen file.
- Hardware target (terukur di Fase 01): Intel Core i5-8250U, RAM ±20,8 GB, Intel UHD 620, SSD, Windows 11 Pro, layar 1366×768. **AI cloud saja** (tanpa inferensi lokal).
- Pengguna **bukan perantara teknis**: Hime dan agent pembangunnya harus mandiri.
- Bahasa utama interaksi: Indonesia (dengan dukungan Inggris). Semua UI dan pesan harus dapat dilokalisasi.

## 3. Kebutuhan pengguna (sumber: pernyataan pengguna) → fitur

Setiap kebutuhan diberi ID `UR-xx`. Fase penanggung jawab merujuk ke `roadmap/`.

| ID | Kebutuhan pengguna (parafrase setia) | Fase |
|---|---|---|
| UR-01 | Mengelola storage laptop sendiri oleh Hime secara rapi, dengan Hime **bertanya** mau disimpan di mana lalu membantu mengaturnya | 10 |
| UR-02 | Mengelola storage HP dari laptop, bisa diremote dan dikelola Hime | 11 |
| UR-03 | Saat mengunduh file bernama sama / file baru, Hime bertanya ("sudah ada, ini versi terbaru?"); pengguna memutuskan, misalnya file lama diganti nama jadi versi lama | 10 |
| UR-04 | Hime membantu mengelola storage laptop dan HP secara efisien dan optimal | 10, 11 |
| UR-05 | Hime dapat mengakses browser; pilih opsi terbaik (aplikasi browser-agent open source atau fitur bawaan) | 13 |
| UR-06 | Hime membantu pekerjaan office (disebut: aplikasi dari "Artcraft" seperti Wordcraft, Figcraft, atau lainnya) secara rapi | 12 |
| UR-07 | Hime membantu membuat laporan dan mengerjakan tugas kuliah, kerja, atau lainnya | 12, 14 |
| UR-08 | Hime membantu mengelola GitHub dan Google Drive | 13 |
| UR-09 | Kanban board (seperti Nusa) dimodifikasi: pengguna dapat memberi tahu Hime beban tugasnya; Hime menagih tugas saat laptop dibuka dan berkala | 12 |
| UR-10 | Penjadwalan belajar; tingkat ketegasan Hime dapat diatur (strict / balanced / santai) lewat UI settings atau lewat chat | 12 |
| UR-11 | Perilaku pengingat berlaku juga di HP (mis. mendeteksi scrolling/penggunaan layar berlebihan dan mengingatkan untuk bangkit) | 11, 12 |
| UR-12 | Hime boleh meremote/mengontrol laptop dan HP (termasuk membantu keamanan perangkat), dengan batas izin yang jelas | 9, 11 |
| UR-13 | Hime membantu kuliah Software Engineering: analisis, perancangan, dan langkah lain untuk membangun sistem robust, aman, UI konsisten (web, mobile, sistem lain); juga peran data engineer/analyst/scientist/AI engineer | 13, 14 |
| UR-14 | Hasil kerja Hime (software, dokumen office) memiliki format dasar profesional, rapi, dan mudah dibaca | 12, 13 |
| UR-15 | Kompresi file, konversi, merge PDF, dan kebutuhan serupa, memakai tool open source sebagai skill | 12 |
| UR-16 | Saat pengguna memblok teks bahasa asing, **Dynamic Island menampilkan terjemahannya** | 9 |
| UR-17 | Sistem robust, andal, mudah diskalakan, masalah mudah diselesaikan; arsitektur multi-agent / skill ditentukan seoptimal mungkin | semua |
| UR-18 | Dapat dipakai di Linux juga (keputusan: Windows-first, cross-platform by design) | 15 (+PAL sejak 06) |

## 4. Katalog modul (produk)

| Modul | Isi utama |
|---|---|
| **Dynamic Island** | Pusat interaksi cepat: Omnibox, status agent, streaming, approval, progres, terjemahan, notifikasi |
| **Assistant** | Percakapan, riwayat, lampiran, status tool, lanjutkan task |
| **Workspace** | Kanban, task, proyek, kalender, pengingat, jadwal belajar |
| **Storage** | Indeks file, analitik, duplikat, klasifikasi, versi, backup, storage perangkat jauh |
| **AI Control Center** | Status Hermes dan 9Router, model/provider, penggunaan, diagnostik |
| **Activity Center** | Riwayat tindakan agent, approval, error, log eksekusi |
| **Developer Workspace** | GitHub, repository, build/test, alur software engineering |
| **Documents** | DOCX/XLSX/PPTX/PDF, OCR, konversi, kompresi, merge |
| **Browser** | Otomasi browser terkendali |
| **Devices** | Android bridge, pairing, remote storage, notifikasi, remote task |
| **Settings** | Umum, tampilan, AI, perilaku agent, izin, keamanan, perangkat, integrasi, resource |

## 5. Persyaratan fungsional inti (pegangan implementasi)

Repository sudah memuat 33 FR dan 18 NFR di `project-management/product-requirements/`. **Itu adalah acuan resmi.** Bagian ini hanya meringkas pedoman lintas fase; bila ada konflik, dokumen di repository menang dan selisihnya dicatat.

### 5.1 Perilaku agent (lintas modul)
- Hime **bertanya** saat ada ambiguitas yang berisiko (lokasi simpan, versi file, penimpaan) dan **tidak menebak**.
- Setiap aksi yang mengubah sistem: **pratinjau → (approval bila perlu) → eksekusi → verifikasi hasil → laporan**.
- Hime tidak boleh mengklaim file berhasil dipindah tanpa memeriksa lokasi akhir, build berhasil tanpa menjalankan build, dsb.
- Memori dan preferensi disimpan dengan sumber, waktu, dan aturan retensi; tidak seluruh riwayat dikirim ke model setiap kali.

### 5.2 Keamanan dan privasi
- Local-first untuk data pribadi; inferensi cloud-first.
- Pemantauan layar/aktivitas (UR-11) **opt-in**, data diproses lokal, dapat dimatikan, dan tidak dikirim ke provider AI tanpa persetujuan.
- Aksi risiko tinggi selalu melewati approval; Core yang menolak jika Core tidak tersedia (fail-closed).

### 5.3 Non-fungsional (target yang harus diukur)
| Aspek | Target |
|---|---|
| RAM Hime total | idle <1 GB; normal 1–3 GB; berat 3–4 GB |
| Concurrency awal | 1 Hermes; 1–2 reasoning task; 2 file worker; 1 browser; 1 pemrosesan dokumen berat |
| Startup | Island muncul dalam waktu yang diukur dan dicatat; target ditetapkan setelah benchmark Fase 08 |
| Reliabilitas | proses runtime pulih dari crash; tidak ada eksekusi tool ganda akibat retry |
| Observability | setiap task punya Task ID dan Correlation ID; log terstruktur; tanpa rahasia di log |
| Aksesibilitas | navigasi keyboard, kontras memadai, dukungan skala DPI |

Angka latensi (mis. "<2 ms proxy") **tidak boleh** dijadikan acceptance criteria sebelum diukur.

## 6. Prioritas

| Prioritas | Arti | Isi |
|---|---|---|
| **P0** | Fondasi wajib | Core, IPC, keamanan, supervisor, desktop shell (Fase 06–07) — *selesai bersyarat* |
| **P1** | Wajib v1.0 | Runtime AI (08), Intelligence Core (09), Storage (10), Productivity dasar + Dokumen (12), Island penuh, installer (15) |
| **P1** | Wajib v1.0 | Mobile + Device (11), Browser + Developer Workspace dasar (13) |
| **P2** | Setelah v1.0 awal tapi direncanakan | Specialist tambahan, accountability lanjutan, sinkronisasi dua arah lanjutan |
| **P3** | Eksperimental | Proaktif lanjutan, adaptive workflow, voice command |

> Pengguna meminta seluruh kebutuhan di bagian 3 diwujudkan. Karena itu tujuan v1.0 mencakup semua UR di atas dengan kedalaman "dapat dipakai", dan hal yang terbukti tidak realistis dicatat sebagai keterbatasan eksplisit, bukan dibuang diam-diam.

## 7. Kriteria sukses Hime v1.0

Hime v1.0 dinyatakan siap hanya jika **seluruh** butir berikut `VERIFIED` (runtime atau automated test) atau tercatat sebagai risiko yang diterima owner:

1. Hime Desktop berjalan; Dynamic Island, Omnibox, Conversation, Activity Stream, Task Board, Notification Center, Settings berfungsi **terbukti pada sesi GUI interaktif** (termasuk multi-monitor dan DPI).
2. Hime Core stabil; Hermes dan 9Router dikelola Process Supervisor (start, health, restart, shutdown, crash recovery).
3. Percakapan nyata dengan model melalui Hermes + 9Router, streaming, cancel, recovery.
4. **IB-01 dan IB-02 RESOLVED** sesuai acceptance criteria di `06-…`, atau penggunaan kredensial sensitif tetap diblokir dengan keputusan owner terdokumentasi.
5. Seluruh tool yang mengubah sistem melewati Tool Gateway + Policy Engine + approval; tidak ada jalur bypass yang terbukti.
6. Storage Intelligence: operasi file aman (staging, jurnal, pemulihan), deteksi unduhan duplikat/versi, tanpa kehilangan data pada uji kegagalan.
7. Aplikasi Android terpasang, ter-pairing aman, dapat melihat/mengelola storage yang diizinkan dan menerima notifikasi/pengingat.
8. Productivity: Kanban, pengingat, jadwal belajar, mode ketegasan (strict/balanced/santai) di desktop dan HP.
9. Dokumen: pembuatan/penyuntingan DOCX, XLSX, PPTX, PDF + konversi, kompresi, merge, OCR dengan hasil yang dibuka dan diverifikasi.
10. Browser, GitHub, Google Drive terintegrasi dengan izin minimal dan approval.
11. Terjemahan teks terpilih tampil di Island.
12. Resource diukur pada hardware target dan memenuhi anggaran, atau penyimpangan dicatat dengan alasan.
13. Installer, updater, rollback, dan manifest kompatibilitas Hermes/9Router berfungsi.
14. Pengujian keamanan (permission bypass, injection, path traversal, kebocoran data, eksposur layanan lokal) lulus tanpa temuan CRITICAL/HIGH terbuka.
15. Dokumentasi pengguna dan pengembang lengkap; `FINAL-HANDOVER-REPORT.md` tersedia.
