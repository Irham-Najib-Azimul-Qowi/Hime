# 16 — APLIKASI ANDROID: SEPADAN DENGAN HIME ISLAND

Aplikasi Android (`mobile/`) adalah **Hime di saku**: tema sama, maskot sama, chat yang sama, tugas dan komitmen yang sama — dan bekerja baik. Ia melanjutkan `hime-bloom-pack/roadmap/PHASE-11R` dan `legacy/.../PHASE-11` (pairing, mutual TLS, transfer, postur keamanan); dokumen ini menambah **paritas UI/fitur, penegakan HP, dan keandalan**.

## 1. Prinsip
1. **Satu identitas:** token desain, ikon (`icon-map`), maskot (`eyes.json`), teks (registry string), warna kategori **dihasilkan dari sumber yang sama** (`design/`, `contracts/`) → tema Compose `HimeTheme`.
2. **Satu thread, satu data:** Core adalah sumber kebenaran (chat, run, tugas, rutin, simpanan). HP = klien + pelaksana lokal untuk aturan yang harus jalan tanpa laptop.
3. **Offline-first untuk yang bisa:** rutinitas, pengingat, penegakan, dan pencatatan tugas bekerja **tanpa laptop**; chat AI butuh Core terjangkau (jujur soal ini).
4. **Ringan dan hemat baterai.**

## 2. Layar (navigasi bawah, 5 tab maks)
| Tab | Isi |
|---|---|
| **Hime** | Chat satu thread: `/` skill → pill berwarna, lampiran (semua file), suara, kartu run (multi-run), mata di header |
| **Tugas** | Hari ini/Mendatang/Selesai, kategori berwarna, quick-add (parser sama), cincin progres |
| **Fokus** | Rutinitas, sesi aktif (timer), komitmen & penegakan, Penggunaan (bar kategori), Jeda Darurat |
| **Simpanan** | Teks/file sementara; **Bagikan ke Hime** dari aplikasi lain; seret/ketuk-tahan untuk kirim ke laptop |
| **Lainnya** | Perangkat (pairing, status laptop), Storage HP, Pengaturan, Tentang |
Desain: gelap, ubin ikon berwarna, pill, progress tebal, mata Hime; **teks singkat** (batas sama). Material 3 sebagai fondasi perilaku (aksesibilitas, state), tetapi **tampilan mengikuti token Hime** (bukan template Material default).

## 3. Paritas fitur
| Fitur | HP |
|---|---|
| Chat + `/` skill + pill | Ya (menu dari registry Core; skill HP-aware) |
| Lampiran semua jenis | Picker SAF + kamera + **Share target** ("Bagikan ke Hime" dari galeri/browser/pemutar); unggah resumable ke Core; peringatan privasi sama |
| Suara | `SpeechRecognizer` (id-ID; on-device bila tersedia) → teks ke input; tahan tombol mik; mata `dengar`; tanpa menyimpan audio |
| Multi-run | Kartu proses sama; notifikasi saat selesai |
| Tugas + kategori + aturan tenggat | Sama; sinkron dua arah; offline antre |
| Rutinitas & komitmen | Sama; **dijalankan lokal** (alarm tepat waktu) |
| Simpanan | Berbagi dua arah **item terpilih/disematkan**; riwayat clipboard HP **tidak** otomatis (batasan Android + privasi) |
| Terjemah | "Bagikan teks ke Hime → Terjemah"; bubble notifikasi hasil; (opsional P2) mengambang tidak dibuat |
| Lirik | Tidak diperlukan |
| Storage HP | SAF/MediaStore, analitik, duplikat/versi, operasi dengan approval (PHASE-11R) |
| Approval | Notifikasi aksi + layar detail; tiket sekali pakai |
| Hime proaktif | Notifikasi dengan aksi (katalog 11): file ganda HP, storage, tenggat, rutin, pengalih, baterai |
| Mata/ekspresi | Di header chat, notifikasi (ikon kecil), layar Fokus, overlay penegakan |

## 4. Penegakan di Android (jujur dan bertingkat)
Android membatasi kontrol atas aplikasi lain. Tingkat dan izin:

| Tingkat | Mekanisme | Izin (dijelaskan di onboarding, ringkas) | Efek |
|---|---|---|---|
| **A. Ingatkan** | Notifikasi + aksi; alarm | Notifikasi | Pengingat rutin/tenggat/pengalih |
| **B. Tegas** | Deteksi aplikasi latar depan + **overlay layar penuh** bermata Hime (hitung mundur, Jeda/Alasan) | **Usage Access** (UsageStatsManager) + **Tampil di atas aplikasi lain** (SYSTEM_ALERT_WINDOW) | Menutupi aplikasi pengalih saat batas/blok belajar |
| **C. Paksa** (opt-in kuat) | **Layanan Aksesibilitas** (event window berubah, aksi *Home*) untuk mengembalikan ke Beranda | **Aksesibilitas** (dengan penjelasan layar penuh: apa yang dibaca/tidak dibaca) | Pengguna dibawa kembali ke Beranda; tidak menutup paksa aplikasi |
- **Deteksi:** B memakai `UsageEvents` dengan **polling adaptif** (hanya saat ada aturan aktif; interval turun saat layar mati); C **berbasis event** (lebih efisien) dan dipakai bila diizinkan.
- **Batas jujur:** Hime **tidak** bisa membunuh aplikasi lain tanpa akses khusus (root/device admin) — tidak digunakan. Penegakan = **overlay + kembali ke Beranda**. Tulis di `docs/mobile/ANDROID-LIMITS.md` dan di UI (satu baris).
- **Pengaman sama dengan desktop:** Jeda Darurat (tahan 3 dtk + alasan), jeda pelemahan aturan, daftar pelindung (telepon, kontak darurat, aplikasi sistem, peta, pembayaran bila ditandai), log, batas laju, tanpa penghapusan data. Aksesibilitas **tidak membaca isi layar** untuk tujuan lain; hanya nama paket/event window.
- **Kuota gabungan** (laptop+HP): Core menjumlahkan agregat harian; HP menegakkan secara lokal dari aturan terakhir yang tersinkron (offline OK).
- **Opsional P3:** integrasi mode Fokus/Digital Wellbeing, Device Policy (kunci layar) — **tidak** di v1.

## 5. Konektivitas
- Pairing QR + mutual TLS + CA lokal (PHASE-11R). Transport utama LAN; di luar LAN lewat jaringan privat/VPN pengguna (tanpa port forwarding).
- **Status koneksi** jelas di UI (mata `offline` + "Laptop tidak terjangkau"); **antrean keluar** (pesan, lampiran, tugas) dikirim saat tersambung; tidak ada kehilangan; kedaluwarsa ditampilkan.
- Sinkron: tugas/rutin/simpanan terpilih/ pengaturan non-rahasia; riwayat chat diambil saat tersambung (paginated).
- Chat butuh laptop menyala dan runtime tersedia (Core dapat menyalakan runtime on-demand). Bila tidak: "Laptop offline. Pesanmu antre." Opsi masa depan (P2): relay milik pengguna.

## 6. Keandalan Android (konteks Indonesia: banyak OEM dengan pembatas baterai agresif)
- Onboarding "**Agar Hime tidak dimatikan**": halaman ringkas yang mendeteksi merek (Xiaomi/Oppo/Vivo/Realme/Samsung/Huawei) dan membuka pengaturan **Autostart / Hemat baterai: Tanpa batasan** yang relevan (intent yang tersedia; jelaskan langkah manual bila tidak ada).
- Alarm tepat waktu: `AlarmManager.setExactAndAllowWhileIdle` (izin alarm tepat bila perlu) untuk rutinitas/tenggat; **WorkManager** untuk sinkron; Foreground Service **hanya** saat sesi fokus/penegakan aktif atau transfer; notifikasi persisten minimal.
- **Pemulihan:** boot receiver menjadwalkan ulang alarm; periksa izin yang dicabut → tampilkan state jelas ("Izin Usage Access dimatikan") dengan satu tombol perbaikan.
- Uji pada ≥ 3 perangkat/OEM (atau emulator + catatan keterbatasan jujur); matriks API (min SDK ditetapkan di ADR; target SDK terbaru).

## 7. Kinerja dan ukuran (target; ukur)
Cold start < 1.5 dtk (kelas menengah), RAM idle ≤ 80 MB proses app, APK/AAB seminimal mungkin (R8, resource shrinking, tanpa pustaka besar), baterai: tanpa wakelock berlebih, polling adaptif. Baseline profile untuk startup. Compose: stabilitas, `remember`, daftar lazy.

## 8. Modul sisi mobile
Setiap modul punya paket Kotlin `modules/<id>/` dengan `Feature` (register/dispose); OFF = tidak ada layanan/izin/jadwal. Manifest yang sama (`mobile` bagian).

## 9. Keamanan mobile
Keystore untuk kunci perangkat; sertifikat klien; token berumur pendek + rotasi; `FLAG_SECURE` pada layar sensitif (approval); tanpa log data pribadi; backup Android dinonaktifkan untuk data sensitif; izin minimum dan dijelaskan; perilaku revoke seketika; pengujian keamanan sesuai legacy 06 §8/§11.

## 10. Akseptansi
- [ ] Tema/maskot/ikon/teks sama dengan desktop (diperiksa berdampingan; sumber token yang sama).
- [ ] Chat satu thread sinkron; `/` pill; lampiran + Bagikan ke Hime; suara; multi-run; notifikasi hasil.
- [ ] Tugas/Rutinitas/Komitmen jalan lokal offline dan sinkron; Tier A/B/C bekerja pada perangkat uji dengan izin masing-masing; Jeda Darurat; log.
- [ ] Alarm tepat waktu bertahan setelah reboot/Doze; panduan OEM; deteksi izin dicabut.
- [ ] Pairing negatif lulus; revoke; tidak ada eksposur layanan.
- [ ] Anggaran kinerja diukur; baterai wajar (catat).
- [ ] **VC-U9** (tangkapan layar semua tab + overlay penegakan + notifikasi aksi). Perangkat nyata = Gate #6.
