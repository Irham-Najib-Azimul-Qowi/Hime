# PHASE 11 — HIME MOBILE & DEVICE LAYER
**Aplikasi Android · Pairing aman · Transport terenkripsi · Storage HP · Remote Hime dari HP · Sinkronisasi · Keamanan perangkat**

**Entry:** Fase 10 `READY FOR OWNER ACCEPTANCE`. Event Engine, Policy Engine, Task Engine, Storage Service stabil.
**Kebutuhan pengguna:** UR-02, UR-04, UR-11 (bagian HP), UR-12, serta "remote Hime dari HP dengan tampilan".
**Owner Gates:** menghubungkan **perangkat Android sungguhan** (Gate #6) dan mengakses data nyata. Bangun dan uji dengan **emulator Android + data sintetis** lebih dulu; sediakan panduan uji perangkat nyata untuk owner.

## 1. Tujuan
Dua arah:
1. **Hime mengelola HP:** melihat dan mengelola storage HP (dalam izin yang diberikan), transfer file, status perangkat, dan laporan postur keamanan.
2. **Pengguna mengendalikan Hime dari HP:** chat dengan Hime, melihat Task Board dan Activity, **menyetujui/menolak approval** dari HP, menerima notifikasi/pengingat.

## 2. Realitas Android (jujur sejak awal — dokumentasikan di `docs/mobile/ANDROID-LIMITS.md`)
Android tidak mengizinkan "kontrol penuh jarak jauh" tanpa akses khusus. Hime **tidak** meminta root, **tidak** membuat shell jarak jauh, dan **tidak** menjadikan Accessibility Service sebagai fondasi. Kemampuan dibatasi ke API resmi:
- File: **Storage Access Framework (SAF)** pada folder yang dipilih pengguna + **MediaStore** untuk media. "All files access" (`MANAGE_EXTERNAL_STORAGE`) hanya opsi **opt-in eksplisit** dengan penjelasan, bukan default.
- Status: baterai, ruang, versi/patch keamanan, jaringan.
- Layar/penggunaan aplikasi: `UsageStatsManager` (izin Usage Access, **opt-in**).
- Notifikasi: `NotificationManager`; layanan latar via **Foreground Service** (dengan notifikasi persisten) dan WorkManager. Perhatikan pembatasan baterai vendor.
Semua keterbatasan ditampilkan di UI sebagai penjelasan jujur, bukan fitur palsu.

## 3. Arsitektur mobile

**Stack:** Kotlin + Jetpack Compose, arsitektur berlapis (UI / domain / data), Hilt (DI), Room (DB lokal terenkripsi bila perlu), coroutines/Flow, OkHttp + WebSocket, kunci di **Android Keystore**. Satu modul `app` + modul inti `core-net`, `core-storage`, `feature-*` bila ukuran membenarkan. Test: JUnit, Turbine, Compose UI test, Robolectric bila berguna; instrumented test di emulator.

**Struktur repo:** `apps/mobile/` (Gradle). Skema protokol dibagi dari sumber tunggal (`proto/` JSON Schema) → generate tipe Rust dan Kotlin; **contract test lintas bahasa**.

## 4. Task

### 11.1 Protokol Device (Core ↔ Mobile)
- **Transport:** WebSocket di atas **TLS dengan mutual authentication** (sisi Core: `rustls`; sisi Android: OkHttp + sertifikat klien dari Keystore). Pesan **JSON-RPC 2.0** + event; berversi.
- **Identitas perangkat:** pasangan kunci per perangkat (dibuat di Keystore, tidak dapat diekspor); sertifikat klien diterbitkan oleh **CA lokal Hime** (kunci CA di vault, tidak pernah keluar).
- **Sesi:** token berumur pendek + refresh dengan rotasi; deteksi replay; rate limit; revoke instan dari desktop.
- **Penemuan:** mDNS/NSD di LAN; alamat manual/QR sebagai cadangan. **Server perangkat hanya aktif saat fitur diaktifkan**, bind spesifik, bukan 0.0.0.0 tanpa autentikasi.
- **Di luar LAN:** **tanpa port forwarding terbuka.** v1 mendukung jaringan privat/mesh VPN yang sudah dipasang pengguna (Hime hanya memakai alamat privat itu). Relay terkelola = P2 (di luar v1). Catat keputusan di ADR.

### 11.2 Pairing aman
Desktop menampilkan **QR berisi** (alamat, fingerprint CA, token sekali pakai berumur pendek). HP memindai → verifikasi fingerprint → kirim CSR → desktop meminta **konfirmasi manual di desktop** (kode pendek dua arah) → terbitkan sertifikat. Token pairing sekali pakai dan kedaluwarsa. Daftar perangkat + revoke + rotasi di Settings → Devices. Uji: token dipakai ulang, QR lama, MITM (fingerprint salah), perangkat dicabut saat terkoneksi.

### 11.3 Aplikasi mobile (fitur)
Autentikasi/biometrik lokal untuk membuka aplikasi; pairing; **Chat** dengan Hime (streaming); **Task Board** & Activity; **Approval** (setujui/tolak aksi berisiko dengan pratinjau, TTL, anti-replay — memakai lifecycle yang sama); dashboard perangkat; dashboard storage HP; notifikasi/pengingat; Settings (izin, perangkat, privasi, mode ketegasan). UI konsisten dengan identitas Hime (obsidian), mendukung gelap/terang, aksesibilitas, bahasa Indonesia.

### 11.4 Storage HP
- Pilih folder (SAF) dan media; indeks metadata di sisi HP (Room) dan ringkasan di Core (hanya metadata dan hash; **konten tidak dikirim** kecuali transfer).
- Analitik (ruang, file besar, duplikat, folder terbesar), pencarian, **operasi dalam folder yang diizinkan** (pindah/ubah nama/hapus dengan approval, Recycle ala HP di folder app bila memungkinkan).
- Operasi dari desktop → perintah ke HP → eksekusi di HP → verifikasi → laporan. Penolakan izin Android ditangani anggun.
- Semua tool via Gateway: `devices.list`, `devices.storage_overview`, `devices.search_files`, `devices.propose_move`, `devices.execute_move`, `devices.transfer`.

### 11.5 Transfer & sinkronisasi
Antrean transfer; **chunked + resumable**; hash **SHA-256** per chunk/berkas (kompatibel lintas platform); pemulihan setelah putus; resolusi konflik (jangan menimpa diam-diam; ikuti pola versi Fase 10); kontrol bandwidth; Wi-Fi-only opsional; pemeriksaan integritas akhir. v1 = transfer yang diminta + folder sinkron satu-arah yang dikonfigurasi; sinkron dua arah penuh = P2.

### 11.6 Remote status dan operasi
Status jarak jauh (baterai, ruang, jaringan), riwayat operasi, antrean offline (perintah menunggu HP online, kedaluwarsa), approval di sisi pengguna. **Tidak ada** eksekusi perintah sembarang.

### 11.7 Postur keamanan perangkat (UR-12)
Laporan hanya dari API resmi: ada kunci layar (`KeyguardManager.isDeviceSecure`), tingkat patch keamanan, status enkripsi penyimpanan, aplikasi dengan izin berisiko (daftar dari `PackageManager`, tanpa mengubah), sumber tak dikenal, versi OS usang. Hasil → rekomendasi + pengingat. Hime **tidak** mengubah pengaturan keamanan HP sendiri.

### 11.8 Pengingat dan penggunaan layar (UR-11, sisi HP)
Pengingat tugas/jadwal belajar; **opt-in** pemantauan penggunaan (UsageStats): jika waktu layar pada kategori "pengalih" melewati ambang yang diatur pengguna, kirim *nudge* sesuai mode (strict/balanced/santai). Data lokal; hanya agregat yang disinkron ke Core dan **tidak** dikirim ke provider AI tanpa persetujuan.

### 11.9 Keamanan mobile
Revoke, rotasi token, penyimpanan aman (Keystore), tinjauan izin, deteksi akses tak sah, FLAG_SECURE pada layar sensitif, tanpa log data pribadi, backup Android dinonaktifkan untuk data sensitif, pembatasan screenshot approval opsional. Pengujian keamanan di `06-…` bagian 8 dan 11.

## 5. Pengujian
- Emulator Android (API level target ditetapkan di ADR; minimal satu versi lama dan satu baru); data sintetis.
- Contract test protokol lintas Rust↔Kotlin; uji pairing negatif; uji putus jaringan di tengah transfer; uji perangkat offline; uji approval lintas perangkat (desktop dan HP bersamaan: hanya satu yang bisa menyelesaikan tiket).
- Uji beban transfer (file besar), baterai (Foreground Service), perilaku vendor (catat sebagai risiko bila tak dapat diuji).
- **Panduan uji perangkat nyata** `docs/mobile/REAL-DEVICE-TEST-GUIDE.md` untuk owner (Gate #6); hingga dilakukan → `NOT TESTED` pada perangkat fisik.

## 6. Deliverables
`apps/mobile/` (APK debug dapat dibangun lewat skrip), modul Device Service di Core, CA lokal + penerbitan sertifikat, skema protokol bersama + contract test, UI Devices di Desktop, dokumentasi `docs/mobile/*`, `docs/architecture/DEVICE-LAYER.md`, ADR (transport, CA, framework mobile, jaringan di luar LAN), laporan fase.

## 7. Exit gate
Laptop dan emulator Android saling berkomunikasi **aman** (mutual TLS, pairing negatif lulus); chat, approval, notifikasi, storage overview, dan transfer terbukti; batasan Android terdokumentasi; tidak ada eksposur layanan tanpa autentikasi (pindai port); revoke bekerja seketika.

## 8. Di luar cakupan
Root/kontrol penuh; remote screen mirroring; relay cloud; sinkron dua arah penuh; iOS.
