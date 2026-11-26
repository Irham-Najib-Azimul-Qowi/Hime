# 14-FINAL REVERSE ENGINEERING REPORT: 9ROUTER
**Project:** Hime Assistant — AI Model Routing Layer Research  
**Target:** 9Router Comprehensive Reverse Engineering Master Report  
**Author:** Senior Reverse Engineer, AI Infrastructure Architect, Backend Engineer & Security Researcher  
**Date:** November 2026  
**Status:** Master Document — Final  

---

## 1. Executive Summary

Laporan ini merupakan dokumen sintesis final (*Master Deliverable*) dari penyelidikan rekayasa terbalik (*reverse engineering*) terhadap **9Router** (`9router-app`, `v0.5.112`, commit `1fe586bb`) yang dikembangkan oleh `decolua`. 

Penyelidikan dilakukan secara sistematis tanpa modifikasi kode, mencakup bedah biner kode sumber Next.js, engine inti `open-sse`, arsitektur adapter 140+ provider AI, mekanisme translasi protokol, sistem persistensi SQLite WAL, keandalan jaringan, pengujian kompatibilitas dengan Hermes Agent, dan kajian kelayakan integrasi ke dalam proyek Hime Assistant.

Penyelidikan membuktikan bahwa 9Router adalah solusi gerbang model AI (*multi-provider routing gateway*) yang luar biasa matang, berkinerja tinggi, dan siap pakai. 9Router menyediakan kompatibilitas penuh dengan protokol OpenAI API, mampu mengisolasi seluruh kegagalan rate limit penyedia upstream melalui rotasi akun dan rangkaian model virtual (*combos*), serta dapat diintegrasikan secara instan dengan Hermes Agent dan Hime Assistant.

---

## 2. Answers to the 12 Fundamental Architectural Questions

### 1. Apa sebenarnya 9Router itu?
9Router adalah gateway perutean model AI lokal dan dashboard manajemen berbasis Next.js 16 / Node.js. 9Router mengekspos titik akhir tunggal yang sepenuhnya kompatibel dengan standar OpenAI (`/v1/*`), namun di belakang layar mampu mengarahkan lalu lintas ke lebih dari 140 provider AI (OpenAI, Anthropic, Gemini, DeepSeek, Nous Research, Grok, Kiro, Cursor, dll.) dengan translasi protokol format otomatis, multi-account rotation, fallback model bertingkat, token refresh OAuth, dan kompresi token (*RTK*).

### 2. Bagaimana arsitektur internalnya bekerja?
9Router mengadopsi arsitektur *Clean Hexagonal*:
- **Lapisan Gateway HTTP (`custom-server.js`):** Membungkus server Next.js untuk sanitasi IP TCP nyata, proteksi downgrade `h2c`, dan penyematan token internal `PEER_TOKEN`.
- **Lapisan Filter & Keamanan (`src/proxy.js`, `dashboardGuard.js`):** Memisahkan akses publik API (`/v1/*`) dari akses dashboard terproteksi JWT, dan membatasi rute eksekusi host hanya untuk loopback `localhost`.
- **Lapisan Aplikasi Glue (`src/sse/handlers/chat.js`):** Mengelola ekspansi model combo, deteksi modalitas input (gambar/audio/PDF), dan loop seleksi akun.
- **Mesin Inti Agnostik (`open-sse/handlers/chatCore.js`):** Mengorkestrasi deteksi format, pemanggilan matriks penerjemah (`open-sse/translator/`), penghemat token (`open-sse/rtk/`), dan eksekusi jaringan via `open-sse/executors/`.
- **Lapisan Persistensi (`src/lib/db/`):** Basis data relasional SQLite WAL (`9router.db`) dengan multi-driver fallback (`bun:sqlite` → `better-sqlite3` → `node:sqlite` → `sql.js`).

### 3. Bagaimana sebuah request mengalir dari Hermes ke Provider AI?
1. Hermes Agent mengirimkan permintaan `POST /v1/chat/completions` (OpenAI format, streaming `stream: true`, memuat array `tools`).
2. `custom-server.js` memvalidasi socket TCP dan Next.js me-rewrite path ke `/api/v1/chat/completions`.
3. `chat.js` memvalidasi API Key lokal, mendeteksi kapabilitas yang dibutuhkan (misal: vision), dan mengevaluasi apakah model adalah virtual combo.
4. `chat.js` mengambil kredensial akun aktif dari tabel SQLite `providerConnections`.
5. `chatCore.js` mendeteksi format sumber (`openai`) dan format target provider (misal: `claude` untuk Anthropic).
6. `translator/index.js` mentranslasikan payload ke format target secara presisi, menormalisasi ID tool call, dan merapikan argumen JSON.
7. `rtk/index.js` mengompresi payload teks hasil tool berukuran besar secara in-place.
8. Eksekutor provider memanggil upstream HTTP API menggunakan pool koneksi `undici`.
9. Respons stream upstream di-peek oleh `emptyFailover.js` untuk memastikan tidak kosong, lalu potongan chunk SSE diterjemahkan kembali ke format OpenAI stream secara real-time dan dipancarkan ke Hermes.
10. Penggunaan token dicatat ke tabel `usageHistory` dan `requestDetails`.

### 4. Bagaimana seleksi provider bekerja?
- Jika dipanggil dengan format terkuota `provider/model` (misal `anthropic/claude-3-5-sonnet`), 9Router langsung menargetkan provider tersebut.
- Jika dipanggil dengan nama pendek tanpa prefix (misal `gpt-4o`), 9Router mencari provider default di katalog static `PROVIDER_MODELS`.
- Jika dipanggil dengan nama virtual combo (misal `smart-coding`), seleksi ditentukan oleh strategi combo yang aktif: `fallback` (urutan array), `roundrobin` (rotasi seimbang), `sticky` (mengunci model per sesi percakapan), `smart` (analisis kecocokan prompt), atau `fusion` (kueri paralel multi-model). Model yang tidak memiliki kapabilitas yang dibutuhkan (misal tidak bisa membaca gambar) secara otomatis digeser ke urutan bawah oleh `reorderByCapabilities`.

### 5. Bagaimana smart fallback bekerja?
Fallback beroperasi pada 2 tingkatan:
- **Tingkat Akun (Account Fallback):** Jika akun provider primer merespons dengan HTTP 429 (Rate Limit), 401/403 (setelah gagal auto-refresh token), atau 5xx, akun tersebut diberi cooldown (1 detik hingga 4 menit exponential backoff) dan dikunci spesifik per-model (`modelLock_${model}`). 9Router langsung mencoba akun berikutnya untuk provider yang sama di dalam loop `while (true)`.
- **Tingkat Combo (Model Fallback):** Jika seluruh akun untuk model tersebut habis atau terkunci, atau jika model upstream mengembalikan stream kosong (`emptyFailover.js`), 9Router melanjutkan ke model alternatif berikutnya di dalam combo.
- **Proteksi Error 400:** Error 400 akibat *context overflow* diberi flag `requestScoped: true` sehingga akun tidak dikunci, melainkan combo beralih ke model dengan jendela konteks lebih besar.

### 6. Bagaimana kredensial dan akun dikelola?
- Kredensial disimpan di tabel SQLite `providerConnections` dalam kolom `data` (JSON).
- Mendukung dua mode otentikasi utama: Static API Key (Bearer/Header kustom) dan OAuth Token (Access Token, Refresh Token, Expiry Date).
- Modul `backgroundTokenRefresh` berjalan setiap 60 detik untuk mendeteksi token OAuth yang mendekati kedaluwarsa (<5 menit) dan memperbaruinya secara otomatis di latar belakang.
- **Catatan Keamanan:** Kredensial di database tersimpan dalam bentuk *plaintext JSON* (tanpa enkripsi AES/DPAPI).

### 7. Seberapa kompatibel 9Router dengan Hermes Agent?
**Kompatibel 100% (Sempurna)**:
- Mendukung seluruh parameter chat completion yang dibutuhkan Hermes: `messages`, `tools`, `tool_choice`, `stream: true`, `reasoning_effort`, dan `response_format`.
- Mengalirkan pemanggilan alat bertahap (*streaming tool calls*) dengan format delta indeks dan ID fungsi yang konsisten.
- Mengalirkan token penalaran pada field `delta.reasoning_content` atau tag `<think>`, memungkinkan Hime menampilkan status berpikir agen secara real-time.
- Fallback akun dan combo berjalan transparan tanpa disadari oleh Hermes.

### 8. Apa saja keterbatasan (limitasi) 9Router?
1. **Tidak Ada Fallback pada Stream yang Sudah Berjalan:** Jika kegagalan jaringan terjadi *setelah* token pertama berhasil dikirim ke klien, 9Router tidak dapat mengalihkan sesi ke provider lain karena header HTTP 200 dan sebagian teks sudah terlanjur diterima klien.
2. **Ketergantungan Protobuf:** Adapter upstream berbasis binary Protobuf (Cursor, Kiro) dapat rusak jika penyedia cloud mengubah skema field proto tanpa pemberitahuan.
3. **Beban Memori JSON Parsial:** Parsing payload konteks raksasa (>10 MB JSON) berjalan di thread utama Node.js V8.

### 9. Masalah keamanan apa yang harus diperhatikan Hime?
1. **Plaintext Credential Storage:** File `%USERPROFILE%\.9router\9router.db` menyimpan seluruh API key dan token OAuth tanpa enkripsi. Hime harus mengamankan izin direktori Windows ACL.
2. **Default Password Risiko:** Nilai default `INITIAL_PASSWORD="123456"` pada instalasi baru harus langsung diganti dengan string acak kriptografis oleh Hime Native Core.
3. **Host Exposure:** 9Router harus selalu dijalankan dengan flag binding loopback `HOSTNAME=127.0.0.1` dan dilarang mengikat ke `0.0.0.0` pada instalasi desktop.

### 10. Apakah Hime harus menggunakan 9Router apa adanya, membungkusnya, atau membangun sendiri?
**Gunakan 9Router apa adanya sebagai Managed Background Service (Opsi B: Dual-Control Plane).**  
Membangun router AI sendiri dari nol adalah pemborosan sumber daya raksasa (*reinventing the wheel*) karena 9Router telah memecahkan ratusan kasus sudut (*edge cases*), quirks provider, dan skema OAuth dari 140+ vendor selama bertahun-tahun. Membungkusnya dengan proxy internal tambahan (Opsi C) hanya menambah latensi dan kompleksitas.

### 11. Bagian mana dari 9Router yang harus diintegrasikan langsung oleh Hime?
1. **`/v1/chat/completions`:** Digunakan langsung oleh Hermes Agent sebagai inferensi model AI utama.
2. **`GET /v1/models/info`:** Dipanggil oleh Hime Native Core (Rust) untuk mengisi menu pemilihan model dan deteksi kapabilitas (apakah tombol lampiran gambar/file harus aktif).
3. **`GET /api/usage/summary`:** Dipanggil oleh Hime Native Core untuk menampilkan widget biaya dan konsumsi token di Dynamic Island drawer.
4. **`/dashboard/console-log` (SSE):** Dipantau oleh Hime Native Core untuk mendeteksi event rotasi akun atau failover combo dan menampilkan animasi notifikasi halus di island.

### 12. Bagian mana yang harus tetap independen?
1. **Dashboard Web 9Router (`/dashboard`):** Tetap berjalan sebagai aplikasi Next.js mandiri. Pengguna yang ingin mengatur OAuth rumit atau menyusun graf node dapat membukanya di browser default via tombol pintasan di Hime.
2. **SQLite Database Engine 9Router:** Dikelola sepenuhnya oleh 9Router; Hime hanya membaca file `9router.db` dalam mode read-only jika diperlukan.

---

## 3. Overall Technical Assessment & Recommendations

### Penilaian Teknis Menyeluruh:
- **Kematangan Arsitektur:** 9.5 / 10 (Sangat tinggi, modular, port-and-adapter bersih).
- **Keandalan Jaringan:** 9.0 / 10 (Didukung pool `undici`, timeouts toleran, exponential backoff).
- **Kompatibilitas Hermes:** 10 / 10 (Paritas penuh dengan standar OpenAI).
- **Keamanan Default:** 7.5 / 10 (Perlindungan jaringan sangat baik, namun penyimpanan kredensial lokal belum terenkripsi).

### Rekomendasi Utama untuk Hime Assistant:
1. **Adopsi 9Router sebagai Komponen Standar:** Masukkan 9Router ke dalam arsitektur Hime Assistant sebagai penyedia model resmi di samping Hermes Agent.
2. **Managed Lifecycle via Rust Supervisor:** Tauri Rust core memonitor kesehatan proses 9Router melalui endpoint `/api/health` dan melakukan auto-restart jika terjadi crash.
3. **Keamanan Tambahan:** Terapkan pembatasan Windows DPAPI pada saat instalasi Hime untuk melindungi kunci master.

---

## 4. Master Research Deliverables Directory

Seluruh 14 laporan riset teknis formal telah disusun secara lengkap di direktori:  
**[`X:\folder_ai\Hime\RE\9ROUTER\`](file:///x:/folder_ai/Hime/RE/9ROUTER/)**

1. [`01-INITIAL-RECONNAISSANCE.md`](file:///x:/folder_ai/Hime/RE/9ROUTER/01-INITIAL-RECONNAISSANCE.md)
2. [`02-SOURCE-CODE-ARCHITECTURE.md`](file:///x:/folder_ai/Hime/RE/9ROUTER/02-SOURCE-CODE-ARCHITECTURE.md)
3. [`03-CORE-ROUTING-ENGINE.md`](file:///x:/folder_ai/Hime/RE/9ROUTER/03-CORE-ROUTING-ENGINE.md)
4. [`04-PROVIDER-ADAPTER-ARCHITECTURE.md`](file:///x:/folder_ai/Hime/RE/9ROUTER/04-PROVIDER-ADAPTER-ARCHITECTURE.md)
5. [`05-MODEL-DISCOVERY-AND-MANAGEMENT.md`](file:///x:/folder_ai/Hime/RE/9ROUTER/05-MODEL-DISCOVERY-AND-MANAGEMENT.md)
6. [`06-SMART-ROUTING-COMBO-AND-FALLBACK.md`](file:///x:/folder_ai/Hime/RE/9ROUTER/06-SMART-ROUTING-COMBO-AND-FALLBACK.md)
7. [`07-API-GATEWAY-AND-PROTOCOL.md`](file:///x:/folder_ai/Hime/RE/9ROUTER/07-API-GATEWAY-AND-PROTOCOL.md)
8. [`08-DASHBOARD-AND-UI-ARCHITECTURE.md`](file:///x:/folder_ai/Hime/RE/9ROUTER/08-DASHBOARD-AND-UI-ARCHITECTURE.md)
9. [`09-AUTHENTICATION-AND-SECURITY.md`](file:///x:/folder_ai/Hime/RE/9ROUTER/09-AUTHENTICATION-AND-SECURITY.md)
10. [`10-DATA-STORAGE-AND-STATE.md`](file:///x:/folder_ai/Hime/RE/9ROUTER/10-DATA-STORAGE-AND-STATE.md)
11. [`11-PERFORMANCE-AND-RELIABILITY.md`](file:///x:/folder_ai/Hime/RE/9ROUTER/11-PERFORMANCE-AND-RELIABILITY.md)
12. [`12-HERMES-INTEGRATION-ANALYSIS.md`](file:///x:/folder_ai/Hime/RE/9ROUTER/12-HERMES-INTEGRATION-ANALYSIS.md)
13. [`13-HIME-INTEGRATION-FEASIBILITY.md`](file:///x:/folder_ai/Hime/RE/9ROUTER/13-HIME-INTEGRATION-FEASIBILITY.md)
14. [`14-FINAL-REVERSE-ENGINEERING-REPORT.md`](file:///x:/folder_ai/Hime/RE/9ROUTER/14-FINAL-REVERSE-ENGINEERING-REPORT.md)

---

*Dengan selesainya laporan ini, seluruh pilar infrastruktur Hime Assistant (Antarmuka Desktop Nusa, Kecerdasan Otonom Hermes Agent, dan Gerbang Perutean Model 9Router) telah berhasil dibedah secara komprehensif dan siap memasuki tahap perancangan arsitektur sistem bersama ChatGPT.*

