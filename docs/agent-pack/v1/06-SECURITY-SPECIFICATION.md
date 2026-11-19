# 06 — SECURITY SPECIFICATION

Hime diberi kemampuan yang berbahaya jika salah: membaca/mengubah file, menjalankan perintah, membuka browser, mengakses akun, mengendalikan HP. Keamanan adalah fondasi, bukan fitur tambahan.

## 1. Prinsip

1. **Hime Core adalah satu-satunya Policy Enforcement Point.** Model AI bukan penentu izin; izin ditegakkan komponen deterministik di luar model.
2. Hermes bukan otoritas keamanan. 9Router bukan batas keamanan. `localhost` bukan autentikasi. Named Pipe DACL tidak melindungi endpoint HTTP 9Router. DPAPI tidak melindungi dari malware dengan hak akses pengguna yang sama.
3. **Least privilege, fail-closed.** Jika Core/Policy tidak tersedia → tolak.
4. **Defense in depth.** Jangan bergantung pada satu kontrol.
5. **Jujur soal sisa risiko.** Dokumentasikan batas setiap kontrol; jangan menjanjikan jaminan absolut.

## 2. Aset yang dilindungi

Kredensial provider dan token OAuth; file pribadi dan akademik; akses terminal/browser; akun GitHub/Drive; data perangkat Android; riwayat percakapan dan memori; integritas aplikasi dan update.

## 3. Threat model (STRIDE; perluas di `docs/security/THREAT-MODEL.md`)

| # | Ancaman | Kategori | Kontrol utama |
|---|---|---|---|
| T1 | Prompt injection via website/dokumen/hasil tool membuat agent melakukan aksi berbahaya | Tampering/EoP | Konten eksternal diperlakukan sebagai data tak tepercaya; tool melewati PEP; aksi berisiko butuh approval manusia; pemisahan konteks; sanitasi output tool |
| T2 | Akses file di luar izin / path traversal | InfoDisc/EoP | Allowlist root, kanonisasi path, tolak symlink/junction keluar root, sandbox uji |
| T3 | Perintah terminal berbahaya / command injection | EoP | Tidak ada shell bebas; command allowlist; argumen sebagai array (bukan string shell); workspace terisolasi |
| T4 | Pencurian API key | InfoDisc | Vault DPAPI; tak ada plaintext di DB 9Router/log/IPC/frontend; zeroization; redaksi berlapis |
| T5 | Kebocoran token OAuth | InfoDisc | Simpan di vault, scope minimal, rotasi, revoke |
| T6 | Endpoint lokal terekspos (9Router, WebSocket perangkat) | Spoofing/EoP | Autentikasi nyata, bind loopback/antarmuka spesifik, tanpa LAN exposure untuk 9Router, IPv4+IPv6 eksplisit |
| T7 | Akses jarak jauh tak sah ke laptop/HP | Spoofing | Pairing QR + mutual TLS, identitas perangkat, revoke, rotasi token, approval aksi sensitif |
| T8 | Eksekusi file berbahaya | EoP | Tidak mengeksekusi file hasil unduhan otomatis; karantina; konfirmasi |
| T9 | Eksekusi tool ganda akibat retry | Tampering | ID idempoten per tool call; periksa efek sebelum retry |
| T10 | Kesalahan agent menghapus data | DoS(data) | Staging, jurnal operasi, pemulihan, approval destruktif, backup |
| T11 | Supply chain (dependensi/biner update) | Tampering | Kunci versi, audit dependensi, verifikasi hash/tanda tangan, lisensi |
| T12 | Penyimpangan klaim keamanan (laporan menyatakan aman padahal tidak) | Repudiation | Klasifikasi bukti, review independen, uji bypass langsung |
| T13 | SSRF/akses jaringan internal lewat browser/tool jaringan | InfoDisc | Blok rentang IP privat/loopback/metadata, allowlist domain per kebijakan |
| T14 | Penyalahgunaan pemantauan aktivitas (privasi) | InfoDisc | Opt-in, lokal, minimisasi data, tidak dikirim ke provider tanpa persetujuan |

## 4. Tingkat izin dan klasifikasi tool

| Level | Operasi | Kebijakan default |
|---|---|---|
| L0 | Baca informasi yang diizinkan | Otomatis |
| L1 | Buat file/dokumen di lokasi yang diizinkan | Otomatis |
| L2 | Pindah/ubah file | Berdasarkan kebijakan; boleh minta approval |
| L3 | Hapus, timpa, akses sensitif | Approval wajib |
| L4 | Perintah berisiko, perubahan sistem | Approval eksplisit atau **dilarang** |

Klasifikasi tool minimum: **read-only**, **reversible**, **destructive**, **sensitive**. Izin ditentukan oleh *resource, operasi, scope, dan konteks* — level di atas hanya klasifikasi awal.

**Approval lifecycle (sudah dibangun Fase 07, pertahankan):** correlation ID, TTL 60 detik, tiket sekali pakai, anti-replay, fail-closed. Tampilkan pratinjau perubahan yang jelas dan jujur; jangan meminta approval untuk hal yang tidak dijelaskan.

## 5. IB-01 — Keamanan kredensial provider 9Router

**Masalah:** 9Router menyimpan kredensial provider secara historis **plaintext** di SQLite miliknya. Vault DPAPI Hime tidak otomatis mengamankan database itu.

**Arah desain terpilih (Fase 04):** **Hime-Owned Provider Adapter**.
```
Hermes → 9Router (Custom Node / endpoint kompatibel OpenAI) → Hime Core Provider Adapter → [Vault DPAPI: dekripsi sesaat] → Provider API
```
Kredensial asli tidak pernah masuk database 9Router. Kompatibilitas Custom Node **belum terbukti** — buktikan lewat contract test.

**Contract test wajib:** model discovery; chat completions; streaming SSE; non-streaming; authentication forwarding; error mapping; retry; timeout; cancellation; model selection; fallback.

**Alternatif bila Custom Node tak memadai (evaluasi, putuskan lewat ADR):** (a) in-memory credential injection; (b) proxy eksternal sadar-kredensial; (c) modifikasi upstream terkendali (**butuh Owner Gate #2**); (d) tidak memakai kredensial sensitif lewat 9Router sampai terselesaikan.

**Larangan:** menulis API key asli ke DB 9Router; dummy key yang diteruskan ke provider sungguhan; kredensial di log; kredensial di `localStorage`; plaintext lewat IPC tanpa kebutuhan; menyatakan DPAPI melindungi dari semua malware.

**Praktik:** umur plaintext di memori minimal; zeroization pada buffer yang dikendalikan aplikasi; dokumentasikan batas (salinan yang dikelola OS/HTTP client/runtime).

**IB-01 = RESOLVED hanya jika SEMUA benar:**
1. Kredensial asli tidak ada plaintext di DB 9Router.
2. Vault menyimpan/mengambil via DPAPI (round trip).
3. Provider request benar-benar sukses memakai secret dari vault.
4. Streaming dan model discovery berhasil.
5. Tidak ada eksposur lewat log, IPC, frontend, atau pesan error.
6. DB 9Router **diperiksa setelah operasi** dan bersih dari secret asli.
7. Diuji dengan **kredensial uji yang aman** (bukan kredensial pribadi/berbayar tanpa Owner Gate #1).
8. Hasil dan residual risk didokumentasikan.

Jika ada yang belum terpenuhi → **IB-01 tetap OPEN** dan jalur kredensial sensitif **dinonaktifkan** (fail-closed).

## 6. IB-02 — Keamanan localhost 9Router

**Masalah:** `dashboardGuard.js` memiliki bypass request lokal; header rahasia buatan Hime tidak berguna jika 9Router tidak memvalidasinya; port acak dan verifikasi PID bukan autentikasi; reverse proxy tidak cukup jika klien bisa langsung ke port 9Router.

**Investigasi wajib (di runtime yang benar-benar dipakai):** `dashboardGuard.js`, urutan middleware, registrasi route API, deteksi request lokal, validasi API key, autentikasi dashboard, penanganan token CLI, binding server, paparan IPv4/IPv6. **Buktikan** apakah autentikasi API key berjalan **sebelum** bypass lokal — jangan menyimpulkan dari satu fungsi.

**Opsi (evaluasi, pilih lewat ADR):**
1. Patch middleware autentikasi (modifikasi upstream terkendali → **Owner Gate #2**).
2. Isolasi proses dan batas jaringan yang benar-benar mencegah akses langsung (mis. binding ke antarmuka/alamat yang hanya dapat dijangkau Core, pipe/UDS, proses di bawah akun/token berbeda, aturan firewall per aplikasi tanpa admin bila memungkinkan).
3. Mekanisme integrasi alternatif yang didukung runtime.

**Larangan:** menjadikan port acak satu-satunya pengaman; menganggap `127.0.0.1`/firewall default/reverse proxy sebagai autentikasi.

**IB-02 = RESOLVED hanya jika SEMUA benar:**
1. Request tanpa autentikasi ditolak.
2. Token tidak valid ditolak.
3. Request valid dari Hime diterima.
4. Bypass localhost tidak melewati autentikasi.
5. Akses HTTP langsung tidak melewati batas keamanan.
6. Tidak ada eksposur LAN.
7. Binding IPv4 dan IPv6 diverifikasi.
8. Pengujian direct-access bypass lulus.
9. Pengujian dilakukan pada runtime 9Router **yang dipakai**.

Jika masih ada jalur bypass → **IB-02 tetap OPEN**.

## 7. Keamanan operasi file

Staging → hash → flush → rename atomik; sumber dipertahankan sampai tujuan terverifikasi; pratinjau perubahan; deteksi konflik; jurnal operasi (apa, kapan, siapa/agent, hasil); pemulihan bila memungkinkan; **jangan menjanjikan undo untuk operasi yang secara teknis tidak dapat dipulihkan**; penghapusan selalu dengan approval; semua uji pada sandbox.

## 8. Keamanan perangkat (Android)

Pairing QR satu kali; sertifikat per perangkat (mutual TLS); sesi berumur terbatas; rotasi token; revoke dari desktop; penyimpanan aman di Android Keystore; izin diminta minimal dan dijelaskan; operasi jauh berisiko butuh approval di sisi yang tepat; log audit lintas perangkat; deteksi akses tak sah.

## 9. Keamanan browser dan jaringan

Profil browser terpisah milik Hime; tanpa kredensial pribadi pengguna secara default; blok SSRF (loopback, RFC1918, link-local, metadata cloud); domain allowlist per kebijakan; unduhan masuk karantina; isian formulir sensitif butuh approval; konten halaman = data tak tepercaya (anti prompt injection).

## 10. Logging, privasi, dan retensi

Tanpa token/password/isi file sensitif/CoT privat di log; redaksi tidak dianggap sempurna karena regex — uji dengan **kredensial kanari** di seluruh jalur (log, IPC, event, UI, DB, crash dump). Data pemantauan aktivitas bersifat lokal, opt-in, dapat dihapus. Retensi dapat diatur.

## 11. Pengujian keamanan wajib

- Permission bypass (panggil tool tanpa lewat Gateway; manipulasi tiket; replay).
- Path traversal, symlink/junction escape, nama file khusus Windows (CON, ADS `:stream`).
- Command injection pada setiap tool yang memanggil proses.
- Prompt injection (halaman web/dokumen berisi instruksi jahat) → pastikan aksi berisiko tetap butuh approval.
- Kebocoran: kanari di seluruh jalur.
- Layanan lokal: pindai port yang dibuka Hime, Hermes, 9Router; uji akses dari proses lain dan dari LAN.
- Kriptografi: pairing, mutual TLS, kedaluwarsa dan revoke.
- Dependensi: audit kerentanan dan lisensi.
- Update: integritas dan rollback.

Temuan CRITICAL/HIGH harus nol sebelum rilis, atau diterima eksplisit oleh owner (Gate #8).
