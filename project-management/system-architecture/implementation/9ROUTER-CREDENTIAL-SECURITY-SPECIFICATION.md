# 9Router Credential Security & In-Flight Injection Architecture (IB-01)

- **Document ID:** HIME-SPEC-04-04
- **Task ID:** TASK-04-04 (IB-01 Credential Security Architecture)
- **Phase:** Phase 04 — Engineering Planning & Implementation Specification
- **Authority:** System Architect & Technical Lead
- **Execution Role:** Senior Security Architect & Systems Engineer
- **Status:** **VALIDATED TECHNICAL SPECIFICATION — IB-01 REMAINS OPEN IMPLEMENTATION BLOCKER**
- **Date:** 2026-11-10

---

## 1. Executive Summary & Root Cause Analysis

### Akar Masalah Keamanan (Root Cause CWE-312):
Audit reverse engineering terhadap 9Router ([`RE/9ROUTER/10-DATA-STORAGE-AND-STATE.md`](file:///x:/folder_ai/Hime/RE/9ROUTER/10-DATA-STORAGE-AND-STATE.md)) membuktikan secara empiris bahwa 9Router menyimpan konfigurasi provider AI beserta API key dalam basis data SQLite internal (`9router.db` / `%LOCALAPPDATA%\9router\data.sqlite`) pada tabel `providerConnections` dan `providerNodes` secara **PLAINTEXT** di dalam kolom `data` (JSON blob) tanpa enkripsi apa pun (`[VERIFIED STATIC]`).

### Realitas Batasan Perlindungan DPAPI:
Windows Data Protection API (DPAPI) merupakan mekanisme enkripsi tingkat OS yang **hanya melindungi data yang secara eksplisit dienkripsi menggunakan API tersebut** (`CryptProtectData`). Menjalankan Hime Core dengan DPAPI **TIDAK SECARA OTOMATIS melindungi atau mengenkripsi basis data internal 9Router**. Jika 9Router menulis file basis datanya ke disk, file tersebut berada dalam keadaan telanjang dan dapat dibaca oleh proses atau malware apa pun yang berjalan di bawah User SID yang sama.

### Status Pemblokir Implementasi:
**IB-01 TETAP BERSTATUS `OPEN IMPLEMENTATION BLOCKER`** untuk seluruh penggunaan kunci API komersial/berbayar milik pengguna. Status ini tidak dapat ditutup sampai terdapat solusi teknis yang terbukti bekerja secara runtime tanpa merusak alur eksekusi, streaming, dan routing 9Router pada **Security Gate SG-01** (Phase 06 / Phase 08).

---

## 2. Investigasi Mendalam Arsitektur Asli 9Router

Untuk menilai kelayakan teknis dari skema *In-Flight Credential Proxy*, berikut adalah jawaban konkret atas 6 pertanyaan arsitektural inti berdasarkan kode sumber dan hasil reverse engineering 9Router:

### 1. Bagaimana 9Router menerima provider credential?
9Router menerima kredensial penyedia (API key, OAuth tokens) melalui dua mekanisme utama:
- **Web UI Dashboard / Management API (`/api/connections`):** Pengguna memasukkan kunci melalui antarmuka web, yang kemudian disimpan langsung oleh repository database (`src/lib/db/repos/providerConnections.js`) ke dalam SQLite (`9router.db`).
- **Custom Provider Nodes (`/api/nodes`):** Pengguna mendefinisikan node kustom (OpenAI-compatible atau Anthropic-compatible) dengan `baseUrl` dan `apiKey` eksplisit, yang disimpan di tabel `providerNodes`.

Kredensial disimpan dalam format plaintext JSON string tanpa enkripsi at-rest.

### 2. Di mana credential digunakan untuk membuat outbound request?
Kredensial dibaca dari database dan diekstraksi ke objek JavaScript runtime `credentials`. Saat request pemodelan diproses:
- Router memilih koneksi aktif dan memanggil modul eksekutor yang sesuai ([`open-sse/executors/base.js`](file:///x:/folder_ai/Hime/RE/9ROUTER/00-source-materials/9router/open-sse/executors/base.js#L11-L192)).
- Metode `buildHeaders(credentials, stream)` merangkai header HTTP autentikasi:
  * OpenAI/Generic: `Authorization: Bearer ${credentials.apiKey}`
  * Anthropic: `x-api-key: ${credentials.apiKey}`
- Eksekutor kemudian memanggil `proxyAwareFetch` (berbasis pustaka Node.js `undici` atau global `fetch`) langsung menuju URL upstream provider eksternal (contoh: `https://api.openai.com/v1/chat/completions`).

### 3. Apakah credential dapat diganti pada tahap outbound tanpa modifikasi source 9Router?
**TIDAK DAPAT SECARA LANGSUNG PADA LEVEL TRANSPORT DEFAULT.**
9Router secara default membuka koneksi HTTPS langsung ke server penyedia AI publik (`api.openai.com`, `api.anthropic.com`). Kredensial telah ditempelkan di dalam payload header HTTP sebelum paket keluar dari proses Node.js.
Untuk mengganti kredensial tanpa mengubah kode sumber 9Router, hanya ada dua jalur jaringan yang mungkin:
1. **Outbound HTTPS Forward Proxy (MITM Inspection):** Memaksa 9Router menggunakan proxy perantara via environment variable `HTTPS_PROXY=http://127.0.0.1:<proxy_port>`. Namun, karena trafik outbound adalah HTTPS terenkripsi TLS, proxy perantara harus melakukan TLS Interception (MITM) yang mewajibkan instalasi sertifikat root CA kustom ke dalam trust store Node.js (`NODE_EXTRA_CA_CERTS`).
2. **Base URL Redirection via Custom Node:** Mengarahkan target endpoint provider pada 9Router agar tidak langsung memanggil cloud, melainkan memanggil endpoint loopback lokal yang dikelola Hime Core (`http://127.0.0.1:<hime_port>/v1`).

### 4. Apakah dummy key dapat diterima oleh 9Router tanpa menyebabkan autentikasi provider gagal?
- Di dalam internal 9Router, dummy key (contoh: `"hime-managed-key"`) dapat disimpan di SQLite tanpa validasi format awal.
- Namun, **JIKA DUMMY KEY TERSEBUT TERKIRIM KE PROVIDER UPSTREAM**, provider cloud (OpenAI, Anthropic, Gemini) seketika menolak request dengan status `401 Unauthorized`.
- Selain itu, beberapa provider adapter di 9Router melakukan validasi pre-flight saat koneksi didaftarkan (misalnya memanggil endpoint `/models` untuk memvalidasi token). Jika request pre-flight ini mengirim dummy key ke endpoint upstream asli, registrasi provider akan gagal.

### 5. Bagaimana Hime dapat menyuntikkan credential asli ke request yang tepat?
Agar Hime Core dapat menyuntikkan kunci asli ke request yang tepat:
- **Jika menggunakan Egress Interceptor (HTTPS Proxy):** Proxy harus mem-parse header HTTP yang masuk dari 9Router, mengidentifikasi target host atau path, mencocokkan request ID / model ID ke DPAPI Vault Hime Core, dan menimpa header `Authorization` dengan kunci produksi sebelum meneruskannya ke internet.
- **Jika menggunakan Loopback Provider Adapter:** 9Router dikonfigurasi dengan Custom Node yang menunjuk ke Hime Core. Hime Core menerima request HTTP standar dari 9Router, membaca header atau model target, mendekripsi kunci yang sesuai dari DPAPI Vault, dan mengeksekusi panggilan HTTPS outbound langsung ke provider cloud atas nama 9Router.

### 6. Apakah desain tersebut masih mempertahankan routing, streaming, retry, dan error handling 9Router?
- **Routing & Combos:** Tetap dipertahankan karena 9Router tetap mengelola virtual model, pembagian beban (*load balancing*), dan fallback chain.
- **Streaming (SSE):** Sangat rentan mengalami distorsi. Proxy perantara wajib mendukung streaming tanpa buffering (*chunked transfer encoding & SSE passthrough*) tanpa menahan frame data token LLM. Latensi tambahan pada loopback proxy harus $< 2\text{ms}$.
- **Retry & Error Handling:** Jika provider upstream mengembalikan status `429 Too Many Requests` atau `5xx Server Error`, proxy harus meneruskan kode status dan body error secara transparan agar logika failover dan combo switching pada 9Router dapat terpicu secara akurat.

---

## 3. Perbandingan Analisis Alternatif Teknis

Berdasarkan realitas arsitektur di atas, dilakukan perbandingan mendalam terhadap 4 alternatif pendekatan:

| Kriteria Evaluasi | Alternatif A: In-Memory Credential Injection | Alternatif B: External Egress Interceptor Proxy (MITM) | Alternatif C: Hime-Owned Provider Adapter (Loopback) | Alternatif D: Controlled Upstream Modification (Fork/Patch) |
|:---|:---|:---|:---|:---|
| **Deskripsi Teknis** | Hime Core memanipulasi memori heap proses Node.js 9Router secara runtime. | 9Router diarahkan ke `HTTPS_PROXY` Hime Core yang membongkar TLS dan menimpa header. | 9Router mendaftarkan node OpenAI-compatible yang mengarah ke loopback adapter Hime Core. | Memodifikasi kode sumber JavaScript 9Router untuk mengambil kredensial via IPC Hime Core. |
| **Kelayakan Teknis (*Feasibility*)** | **Sangat Rendah:** Membutuhkan process injection / debugger API (memicu deteksi antivirus Windows Defender). | **Sedang:** Memerlukan local CA certificate injection ke Node.js trust store (`NODE_EXTRA_CA_CERTS`). | **Tinggi:** Menggunakan fitur standar 9Router (*Custom Provider Nodes*) tanpa trik jaringan kotor. | **Sangat Tinggi:** Secara kode sangat mudah diintegrasikan langsung pada eksekutor 9Router. |
| **Tingkat Keamanan (*Security*)** | Rapuh; memory offsets tidak stabil di Node.js V8 engine. | Tinggi terhadap disk; rentan pada pengelolaan private key CA lokal di host pengguna. | **Sangat Tinggi:** Kunci komersial tidak pernah menyentuh 9Router; tersimpan di vault DPAPI Hime. | **Tinggi:** Kunci dibaca on-demand via IPC internal, tidak pernah ditulis ke SQLite. |
| **Pemeliharaan (*Maintainability*)** | **Buruk:** Rusak setiap kali versi V8 runtime berubah. | **Rendah:** Rumit mengelola sertifikat TLS lokal dan penanganan HTTP/2 streaming framing. | **Tinggi:** Sepenuhnya decoupled; kontrak berbasis protokol HTTP/SSE OpenAI standar. | **Rendah s/d Sedang:** Memerlukan sinkronisasi rutin setiap kali upstream 9Router merilis versi baru. |
| **Kepatuhan ADR-006** | Patuh (tanpa ubah kode). | Patuh (tanpa ubah kode). | **Patuh Penuh** (menggunakan interface Custom Node resmi 9Router). | **Melanggar ADR-006** (memerlukan patch/fork pada repositori eksternal). |

---

## 4. Rekomendasi Teknis & Jalur Arsitektural Terpilih

Berdasarkan trade-off teknis yang jujur:

### Rekomendasi Utama: Pendekatan Dua Jalur (*Dual-Path Strategy*)

1. **Jalur Transpor Standar: Hime-Owned Provider Adapter (Alternatif C)**
   - Hime Core menyediakan layanan **Local Egress Gateway** pada loopback (`http://127.0.0.1:<hime_adapter_port>/v1`).
   - Di dalam 9Router, koneksi model dikonfigurasi sebagai Custom Provider Node yang mengarah ke Gateway Hime Core tersebut.
   - 9Router tetap menjalankan fungsinya sebagai pengelola virtual combo, load balancing, dan model switching.
   - Saat 9Router meneruskan request ke Gateway Hime Core, Hime Core mengambil kunci API resmi dari Vault DPAPI, membungkusnya dalam HTTPS request resmi, mengirimkannya ke OpenAI/Anthropic/Gemini, dan mengalirkan token SSE kembali ke 9Router secara non-blocking.
   - **Keuntungan:** Kunci komersial **0% menyentuh basis data 9Router**, tidak memerlukan instalasi root CA sertifikat (menghindari kompleksitas MITM TLS), dan sepenuhnya mematuhi kebijakan **ADR-006** (Zero Upstream Modification).

2. **Jalur Cadangan: Controlled Upstream Maintained Patch (Alternatif D)**
   - Jika pengujian runtime membuktikan bahwa Custom Provider Node 9Router memiliki batasan fungsional (misalnya keterbatasan dukungan tool calling atau parameter reasoning khusus), Hime Project Owner dapat mempertimbangkan revisi terhadap ADR-006 untuk memelihara fork vendor resmi 9Router (`hime-9router-runtime`) yang memodifikasi `src/lib/db/repos/` agar membaca kredensial via IPC Hime Core.

---

## 5. Keputusan Blocker & Kriteria Pembukaan (Unblocking Criteria)

### Keputusan Tata Kelola:
**IB-01 TETAP BERSTATUS `OPEN IMPLEMENTATION BLOCKER`.**
Spesifikasi ini menetapkan desain dan batasan teknis, namun blocker ini TIDAK BOLEH ditutup hanya berdasarkan dokumen perancangan. Blocker hanya dapat ditutup setelah verifikasi empiris kode berjalan pada Phase 06/Phase 08.

### Kriteria Pembukaan Blocker (Security Gate SG-01):
- [ ] Pemeriksaan heksadesimal terhadap seluruh direktori `%LOCALAPPDATA%\Hime\runtime\9router\` saat 9Router beroperasi membuktikan **0 kemunculan byte kunci API komersial**.
- [ ] Pemeriksaan tabel `providerConnections` dan `providerNodes` pada `9router.db` membuktikan kolom `data` tidak memuat kunci API asli.
- [ ] Panggilan end-to-end melalui Hime Desktop -> Hime Core -> 9Router -> Hime Egress Gateway -> Cloud AI Provider terbukti berhasil menerima respons valid `200 OK` dengan token streaming yang lancar ($\ge 30\text{ token/detik}$).
- [ ] Error upstream (429, 500) diteruskan secara transparan dan memicu failover combo pada 9Router.
