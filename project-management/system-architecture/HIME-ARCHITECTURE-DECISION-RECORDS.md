# Hime Architecture Decision Records (ADR Register)

- **Document ID:** HIME-ARCHITECTURE-DECISION-RECORDS
- **Task ID:** TASK-02-05 (System Architecture & Technical Design)
- **Phase:** Phase 02 — Nusa Reverse Engineering & Architecture Discovery
- **Authority:** Principal Software Architect & Lead Systems Architect
- **Status:** **APPROVED ARCHITECTURAL DECISIONS — PENDING OWNER REVIEW**
- **Date:** 2026-11-08

---

## 1. Executive Summary & Governance Alignment

Dokumen ini mendokumentasikan 12 Catatan Keputusan Arsitektur (*Architecture Decision Records / ADR*) resmi untuk **Hime**. Setiap ADR dirumuskan secara komprehensif mengikuti standar IEEE/Architecture Tradeoff Analysis Method (ATAM) dengan mencakup konteks masalah, alternatif yang dievaluasi, keputusan definitif, justifikasi teknis, konsekuensi, implikasi keamanan, dampak performa, dan reversibilitas.

### Indeks 12 ADR Resmi Hime:
- **ADR-001:** Modular Monolith vs Microservices untuk Arsitektur Hime Core.
- **ADR-002:** Pemilihan Tauri 2.0 Dibandingkan Alternatif Desktop Framework.
- **ADR-003:** Pemisahan Batas Proses Rust dan Python (IPC Stream vs PyO3 In-Process).
- **ADR-004:** Arsitektur Basis Data SQLite WAL & FTS5 untuk Local-First MVP.
- **ADR-005:** Protokol IPC: Custom Framing Biner + JSON-RPC 2.0 vs gRPC / WebSockets.
- **ADR-006:** Orkestrasi Agen: Single Hermes Runtime Process untuk MVP vs Multi-Agent.
- **ADR-007:** Tool Gateway: Centralized PEP/PDP pada Hime Core vs Direct Agent Tools.
- **ADR-008:** Arsitektur Memori: SQLite FTS5 Teks Penuh dengan Jalur Evolusi Vektor.
- **ADR-009:** Manajemen Kredensial: Windows DPAPI & Linux Secret Service Wrapper.
- **ADR-010:** Supervisi Proses: Windows Job Objects & Linux prctl Anti-Orphan.
- **ADR-011:** Arsitektur Event: In-Memory Tokio Broadcast Bus + SQLite Event Journal.
- **ADR-012:** Strategi Konektivitas Android: USB ADB Client Engine (Phase 2).

---

## ADR-001: Modular Monolith vs Microservices untuk Arsitektur Hime Core
- **Status:** **ACCEPTED**
- **Context:** Hime Core bertindak sebagai pusat koordinasi sistem yang mengelola state bus, evaluasi izin, supervisi proses, pemantauan berkas, dan indexing memori.
- **Problem:** Apakah Hime Core sebaiknya dibangun sebagai kumpulan microservices independen yang berkomunikasi melalui jaringan/socket lokal, atau sebagai satu kesatuan Modular Monolith di dalam satu proses biner Rust?
- **Considered Alternatives:**
  - *Alternatif A: Microservices Architecture* (Layanan terpisah untuk Storage, Tool Gateway, Policy, Memory, dan Supervisor).
  - *Alternatif B: Monolith Tradisional Tak Terstruktur* (Seluruh kode berada dalam modul tunggal tanpa batas domain).
  - *Alternatif C (Terpilih): Modular Monolith* (Satu biner Rust dengan pemisahan domain yang ketat, interface berbasis Rust traits, dan in-memory message bus).
- **Decision:** Membangun Hime Core sebagai **Modular Monolith** berbasis Rust dan runtime asynchronous Tokio.
- **Rationale:**
  - Menghilangkan overhead serialisasi IPC dan latensi soket antar-layanan internal pada laptop pengguna.
  - Memastikan konsistensi transaksi ACID lokal tanpa memerlukan distributed transaction/two-phase commit.
  - Alokasi memori sangat efisien ($\le 45\text{ MB}$ footprint) dibandingkan menjalankan banyak proses daemon microservices.
  - Boundary domain yang ketat tetap memungkinkan pemecahan layanan di masa mendatang jika diperlukan (*high evolvability*).
- **Consequences:**
  - *Positif:* Performa in-process function call tanpa latensi jaringan; refactoring aman via type-safety Rust compiler; deployment mudah (single binary).
  - *Negatif:* Crash/panic fatal pada satu modul internal dapat merusak seluruh proses Hime Core jika tidak diisolasi via `std::panic::catch_unwind`.
- **Security Implications:** Mengurangi *attack surface* karena tidak ada port jaringan internal yang terekspos antar-layanan.
- **Performance Implications:** Latensi komunikasi antar-modul $\le 0.1\text{ ms}$ (in-process pointers / Tokio channels).
- **Reversibility:** Tinggi; karena setiap modul memiliki kontrak interface trait, modul dapat dipisahkan menjadi daemon mandiri jika beban kerja enterprise menuntutnya.

---

## ADR-002: Pemilihan Tauri 2.0 Dibandingkan Alternatif Desktop Framework
- **Status:** **ACCEPTED**
- **Context:** Hime Desktop membutuhkan antarmuka mengapung Dynamic Island dengan visual modern, animasi morphing 60 FPS, dan kemampuan memanipulasi jendela Win32 tingkat rendah.
- **Problem:** Framework desktop apa yang memberikan efisiensi memori terbaik, ukuran biner ringkas, dan kontrol native mendalam?
- **Considered Alternatives:**
  - *Alternatif A: Electron* (Chromium + Node.js bundle).
  - *Alternatif B: Flutter Desktop* (Dart + Skia / Impeller custom engine).
  - *Alternatif C: Qt 6 C++ / QML* (Native UI toolkit).
  - *Alternatif D (Terpilih): Tauri 2.0* (Rust native shell + System WebView2 / WebKit2GTK).
- **Decision:** Menggunakan **Tauri 2.0** dengan frontend React 19, TypeScript, Tailwind CSS, dan Motion.
- **Rationale:**
  - *Footprint Memori:* Electron mengonsumsi $\ge 150\text{ MB}$ saat idle; Tauri 2 hanya mengonsumsi $\approx 45\text{ MB}$ karena menggunakan WebView2 yang sudah terpasang di Windows 11.
  - *Ukuran Biner:* Installer Tauri $\approx 10\text{--}15\text{ MB}$, sedangkan Electron $\ge 100\text{ MB}$.
  - *Akses Native:* Tauri memungkinkan integrasi langsung kode Rust ke Win32 API (`DirectComposition`, `RegisterHotKey`, `WM_NCHITTEST`) tanpa overhead Node-API native addons.
- **Consequences:**
  - *Positif:* Waktu cold startup cepat ($\le 2.5\text{ detik}$); konsumsi RAM rendah; integrasi tipe aman antara Rust backend dan TypeScript frontend.
  - *Negatif:* Perbedaan rendering minor antara WebView2 (Chromium di Windows) dan WebKit2GTK (WebKit di Linux) menuntut pengujian visual lintas-platform.
- **Security Implications:** WebView2 berjalan di sandbox terisolasi; komunikasi ke Rust backend diverifikasi ketat melalui Tauri IPC Command scope.
- **Performance Implications:** Mempertahankan 60 FPS pada iGPU Intel UHD Graphics 620 dengan utilisasi CPU idle $< 0.5\%$.
- **Reversibility:** Menengah; logika UI berbasis web standar (React/TS) dapat dipindahkan ke platform webview lain jika diperlukan.

---

## ADR-003: Pemisahan Batas Proses Rust dan Python (IPC Stream vs PyO3 In-Process)
- **Status:** **ACCEPTED**
- **Context:** Hermes Agent dibangun di atas ekosistem Python, sedangkan Hime Core dibangun di atas Rust.
- **Problem:** Bagaimana cara mengintegrasikan Hermes Agent dengan Hime Core secara aman, stabil, dan terisolasi?
- **Considered Alternatives:**
  - *Alternatif A: In-Process Embedding via PyO3* (Menyematkan interpreter CPython langsung di dalam biner Rust Hime Core).
  - *Alternatif B (Terpilih): Out-of-Process IPC Stream* (Menjalankan Hermes Agent sebagai proses anak Python independen yang berkomunikasi via Named Pipe / Domain Socket).
- **Decision:** Menjalankan Hermes Agent sebagai **proses sistem operasi terpisah (*Out-of-Process*)** yang diawasi oleh Hime Core Process Supervisor.
- **Rationale:**
  - *Isolasi Kegagalan (Fault Isolation):* Crash fatal, memory leak, atau segment fault pada pustaka CPython tidak akan meruntuhkan proses utama Hime Core.
  - *Batas Keamanan Terverifikasi:* Hermes Agent tidak dapat memanipulasi memori Hime Core atau membypass fungsi penegakan kebijakan (*Policy Enforcement Point*).
  - *Manajemen Dependensi:* Memungkinkan runtime Python Hermes dikelola secara terisolasi via `uv` virtualenv tanpa mengotori sistem host.
- **Consequences:**
  - *Positif:* Ketahanan crash tinggi; pemantauan resource terpisah per-proses; penegakan izin alat deterministik via IPC gate.
  - *Negatif:* Memerlukan lapisan serialisasi data (JSON-RPC) dan sinkronisasi siklus hidup proses.
- **Security Implications:** Membentuk batas keamanan proses yang kuat (*strong process isolation boundary*); mitigasi ancaman prompt injection langsung ke memori sistem.
- **Performance Implications:** Overhead latensi IPC lokal sangat kecil ($\le 1.0\text{ ms}$ per pesan).
- **Reversibility:** Sangat Rendah; merupakan pilar pemisahan keamanan sistem Hime.

---

## ADR-004: Arsitektur Basis Data SQLite WAL & FTS5 untuk Local-First MVP
- **Status:** **ACCEPTED**
- **Context:** Hime membutuhkan basis data lokal untuk menyimpan riwayat obrolan, metadata berkas, jurnal pemindahan 30 hari, dan pencarian teks penuh.
- **Problem:** Mesin persistensi apa yang memberikan keandalan ACID, pencarian teks cepat, dan nol konfigurasi operasional pada laptop pengguna?
- **Considered Alternatives:**
  - *Alternatif A: Embedded Vector Database Kustom (Qdrant/Chroma)* (Konsumsi RAM tinggi $\ge 200\text{ MB}$, berat untuk hardware acuan).
  - *Alternatif B: Flat Files JSON/YAML* (Rentan korupsi saat mati listrik, tidak ada jaminan ACID, query lambat).
  - *Alternatif C (Terpilih): SQLite 3.45+ dengan WAL Mode & FTS5 Virtual Tables*.
- **Decision:** Menetapkan **SQLite 3.45+ (WAL Mode + FTS5)** sebagai basis data primer Hime Core MVP.
- **Rationale:**
  - *Zero Configuration:* Basis data berupa file tunggal (`hime.db`), tidak memerlukan daemon server terpisah.
  - *WAL Concurrency:* Write-Ahead Logging mengizinkan banyak pembaca paralel tanpa memblokir thread penulis.
  - *Kecepatan FTS5:* Pencarian teks penuh pada ribuan riwayat dokumen diselesaikan dalam waktu $\le 15\text{ ms}$.
  - *Footprint Sangat Kecil:* Alokasi memori SQLite hanya $\sim 8\text{--}15\text{ MB}$.
- **Consequences:**
  - *Positif:* Integritas data terjamin pasca power loss; backup mudah (file copy/vacuum); performa query lokal instan.
  - *Negatif:* Pencarian semantik murni berbasis vektor embedding ditunda ke Phase 2 (DIR-04).
- **Security Implications:** Database disimpan di direktori terproteksi OS ACL 0700; rahasia dienkripsi via DPAPI sebelum disimpan.
- **Performance Implications:** Throughput tulis WAL mencapai $> 2.000\text{ transactions/sec}$ pada SSD.
- **Reversibility:** Menengah; skema tabel mematuhi standar relasional SQL ANSI sehingga dapat dimigrasikan ke engine lain jika diperlukan.

---

## ADR-005: Protokol IPC: Custom Framing Biner + JSON-RPC 2.0 vs gRPC / WebSockets
- **Status:** **ACCEPTED**
- **Context:** Hime Core dan Hermes Agent memerlukan protokol komunikasi dua arah berbasis stream (Named Pipe Win32 / Unix Domain Socket Linux).
- **Problem:** Format protokol apa yang memberikan efisiensi parsing, kesederhanaan implementasi, dan interoperabilitas Python-Rust tanpa bloat dependensi?
- **Considered Alternatives:**
  - *Alternatif A: gRPC / Protocol Buffers* (Memerlukan kompilasi protobuf, runtime gRPC C++ tebal, dependensi berat).
  - *Alternatif B: Localhost WebSockets* (Membuka port jaringan TCP loopback, rentan terhadap intervensi firewall lokal).
  - *Alternatif C (Terpilih): Hime Custom Wire Framing Biner (Magic + Length) dengan Payload JSON-RPC 2.0*.
- **Decision:** Menggunakan **Custom Wire Framing Biner (Header 12-byte) dengan Dokumen JSON-RPC 2.0**.
- **Rationale:**
  - Mengeliminasi masalah packet boundary pada stream socket OS via 4-byte length prefix.
  - Mengadopsi standar JSON-RPC 2.0 yang matang dan didukung secara native oleh pustaka Python dan Rust.
  - Transparan untuk debugging (payload berupa readable UTF-8 JSON terstruktur).
- **Consequences:**
  - *Positif:* Tidak ada dependensi compiler eksternal; parsing cepat; payload mudah divalidasi skemanya.
  - *Negatif:* Ukuran data sedikit lebih besar dibandingkan biner protobuf (dimitigasi oleh kompresi opsional).
- **Security Implications:** Framing memvalidasi ukuran maksimum payload ($\le 16\text{ MB}$), mencegah serangan alokasi memori DoS.
- **Performance Implications:** Serialisasi/deserialisasi payload memakan waktu $< 0.5\text{ ms}$ per pesan.
- **Reversibility:** Tinggi; message envelope dapat disesuaikan pada peningkatan versi protokol minor.

---

## ADR-006: Orkestrasi Agen: Single Hermes Runtime Process untuk MVP vs Multi-Agent
- **Status:** **ACCEPTED**
- **Context:** Sisa memori fisik bebas pada hardware acuan pengembang (Intel i5-8250U) adalah ~3.7 GB saat aplikasi kerja pengguna aktif.
- **Problem:** Apakah Hime MVP harus mendukung eksekusi multi-agent konkuren secara simultan atau membatasi pada satu instance runtime tunggal?
- **Considered Alternatives:**
  - *Alternatif A: Multi-Agent Concurrency* (Membuat banyak proses anak Hermes Agent untuk tugas-tugas paralel).
  - *Alternatif B (Terpilih): Single Supervised Hermes Runtime dengan Antrean Tugas Serial (FIFO)*.
- **Decision:** Menetapkan **Satu Proses Hermes Agent Tunggal** untuk MVP, melayani 1 tugas aktif dalam satu waktu.
- **Rationale:**
  - Mencegah kehabisan RAM host (*memory exhaustion*); setiap runtime Python mengonsumsi 65–180 MB.
  - Menghindari race condition pada manipulasi berkas lokal dan deadlock persetujuan pengguna.
  - Menyederhanakan penegakan kebijakan keamanan dan jejak audit pada rilis pertama.
- **Consequences:**
  - *Positif:* Stabilitas sistem host terjamin; konsumsi daya rendah; arsitektur deterministik.
  - *Negatif:* Pengguna harus menunggu tugas agen pertama selesai sebelum tugas kedua dieksekusi (antrean FIFO).
- **Security Implications:** Mempermudah isolasi audit trail per-tugas karena tidak ada eksekusi thread yang tumpang tindih.
- **Performance Implications:** Memastikan alokasi CPU untuk agen dibatasi maksimal 2 core logika.
- **Reversibility:** Tinggi; arsitektur IPC `IpcTransport` telah dirancang mendukung multi-channel untuk Phase 14.

---

## ADR-007: Tool Gateway: Centralized PEP/PDP pada Hime Core vs Direct Agent Tools
- **Status:** **ACCEPTED**
- **Context:** Model AI membutuhkan kapabilitas eksekusi terminal shell, manipulasi berkas, dan inspeksi sistem.
- **Problem:** Di mana batas eksekusi alat harus ditegakkan untuk mencegah aksi destruktif atau serangan prompt injection?
- **Considered Alternatives:**
  - *Alternatif A: Direct Agent Execution* (Hermes Agent mengeksekusi subprocess dan penulisan berkas langsung di modul Python-nya).
  - *Alternatif B (Terpilih): Centralized PEP/PDP Tool Gateway pada Hime Core*.
- **Decision:** Menempatkan **seluruh penegakan kebijakan (PEP/PDP) dan eksekusi alat secara eksklusif di Hime Core**. Hermes Agent dilarang memiliki jalur eksekusi alat mandiri ke OS.
- **Rationale:**
  - Menjamin bahwa prompt injection pada model AI tidak dapat menembus sistem operasi tanpa persetujuan eksplisit.
  - Menyediakan titik tunggal untuk validasi whitelist direktori, pembatasan kuota waktu (timeout 30s), dan pencatatan audit trail.
  - Memungkinkan implementasi Action Approval Modal dengan jeda ergonomi anti-slip 500 ms secara native.
- **Consequences:**
  - *Positif:* Keamanan host absolut; tidak ada celah eksekusi bypass; transparansi operasional 100%.
  - *Negatif:* Menambah 1 langkah roundtrip IPC untuk setiap pemanggilan alat ($\approx 1\text{--}2\text{ ms}$).
- **Security Implications:** Menegakkan *Invarian 1: Zero Direct OS Bypass* sebagai pilar pertahanan terkuat Hime.
- **Performance Implications:** Overhead IPC terabaikan dibandingkan durasi eksekusi I/O alat nyata ($> 50\text{ ms}$).
- **Reversibility:** Sangat Rendah; merupakan prinsip keamanan non-negotiable proyek Hime.

---

## ADR-008: Arsitektur Memori: SQLite FTS5 Teks Penuh dengan Jalur Evolusi Vektor
- **Status:** **ACCEPTED (Penerapan Arahan DIR-04)**
- **Context:** Pengguna membutuhkan pencarian memori percakapan, dokumen kerja, dan preferensi lokal yang cepat dan relevan.
- **Problem:** Bagaimana merancang sistem memori yang ringan untuk MVP namun siap berekspansi ke semantic search?
- **Considered Alternatives:**
  - *Alternatif A: Memasukkan Local Vector DB (sqlite-vec / ONNX Embedding) langsung di MVP* (Menambah 120+ MB memori untuk model embedding lokal).
  - *Alternatif B (Terpilih): SQLite FTS5 Text Search sebagai Fondasi MVP dengan Abstraction Layer ke Vektor (Phase 2)*.
- **Decision:** Menggunakan **SQLite FTS5 Full-Text Search** untuk MVP, dibungkus dalam abstraksi `MemoryManager` trait untuk memungkinkan integrasi embeddings lokal pada Phase 2.
- **Rationale:**
  - Mematuhi arahan Project Owner DIR-04.
  - FTS5 memiliki latensi query ultra-rendah ($\le 15\text{ ms}$) dan konsumsi RAM mendekati nol.
  - Memadai untuk sebagian besar use case harian (pencarian nama berkas, cuplikan kode, kata kunci obrolan).
- **Consequences:**
  - *Positif:* Tidak ada ketergantungan model embedding besar di MVP; cold startup cepat; memori host hemat.
  - *Negatif:* Tidak mendukung pencarian makna semantik konseptual jika kata kuncinya tidak cocok secara leksikal.
- **Security Implications:** Seluruh indeks disimpan di database lokal `hime.db`; dilarang mengirim data memori ke cloud.
- **Performance Implications:** Indeks FTS5 berukuran hanya $\approx 10\%$ dari total ukuran teks dokumen asli.
- **Reversibility:** Tinggi; trait `MemoryManager` dirancang memfasilitasi swap engine tanpa mengubah kode pemanggil.

---

## ADR-009: Manajemen Kredensial: Windows DPAPI & Linux Secret Service Wrapper
- **Status:** **ACCEPTED**
- **Context:** Sistem Hime dan 9Router menyimpan kredensial berbayar (API key OpenAI, Anthropic, Google). Database 9Router (`data.sqlite`) saat ini menyimpan kredensial dalam bentuk teks polos (CWE-312 `[VERIFIED STATIC]`).
- **Problem:** Bagaimana mengamankan penyimpanan kredensial lokal dari pencurian malware host tanpa mengganggu stabilitas 9Router?
- **Considered Alternatives:**
  - *Alternatif A: Menyimpan dalam Plaintext Configs* (Sangat tidak aman, melanggar aturan tata kelola).
  - *Alternatif B: Enkripsi Kustom dengan Master Password Pengguna* (Mengurangi kenyamanan UX, pengguna harus mengetik password setiap boot).
  - *Alternatif C (Terpilih): Hardware-Backed OS Encryption (Windows DPAPI / Linux Secret Service) dengan Wrapper Terencana*.
- **Decision:** Menggunakan **Windows DPAPI (`CryptProtectData`)** pada Windows dan **FreeDesktop Secret Service (`libsecret`)** pada Linux untuk seluruh rahasia Hime Core, serta merencanakan wrapper enkripsi untuk database 9Router (`[PLANNED CONTROL]`).
- **Rationale:**
  - Kunci enkripsi dikelola langsung oleh sistem operasi dan terikat pada akun logon pengguna aktif.
  - Kredensial tidak dapat didekripsi jika file database disalin ke komputer lain atau dibaca oleh pengguna Windows lain.
  - Memisahkan secara transparan antara kondisi eksisting 9Router dan mitigasi terencana Hime Core.
- **Consequences:**
  - *Positif:* Keamanan kredensial standar enterprise; tidak memerlukan input password manual saat startup.
  - *Negatif:* Memerlukan pemeliharaan wrapper keamanan lokal atau patch upstream untuk 9Router (OQ-05).
- **Security Implications:** Menutup risiko kebocoran kredensial massal akibat pencurian file database secara pasif.
- **Performance Implications:** Enkripsi/dekripsi DPAPI memakan waktu $< 2\text{ ms}$ saat inisialisasi.
- **Reversibility:** Menengah; bergantung pada API native platform Windows dan Linux.

---

## ADR-010: Supervisi Proses: Windows Job Objects & Linux prctl Anti-Orphan
- **Status:** **ACCEPTED**
- **Context:** Hime Core meluncurkan sub-proses Hermes Agent (Python) dan 9Router (Node.js). Jika Hime Desktop crash atau dimatikan mendadak, sub-proses berisiko tertinggal sebagai proses yatim (*orphan processes*).
- **Problem:** Bagaimana menjamin seluruh proses anak mati secara instan dan deterministik saat proses induk mati?
- **Considered Alternatives:**
  - *Alternatif A: Heartbeat Polling Manual dari Proses Anak* (Jika proses anak hang, proses tidak akan mati).
  - *Alternatif B (Terpilih): OS Kernel-Level Process Supervision (Windows Job Objects & Linux prctl)*.
- **Decision:** Mengikat seluruh sub-proses ke dalam **Windows Job Object (`JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`)** pada Windows dan **`prctl(PR_SET_PDEATHSIG, SIGTERM)`** pada Linux.
- **Rationale:**
  - Jaminan terminasi ditegakkan langsung oleh kernel sistem operasi, bukan oleh logika aplikasi tingkat pengguna.
  - Bahkan jika Hime Desktop di-kill secara paksa melalui Windows Task Manager (`taskkill /F`), kernel Windows seketika membersihkan seluruh proses anak tanpa jeda.
  - Menjamin pemenuhan **Invarian 3: Zero Orphan Child Process**.
- **Consequences:**
  - *Positif:* Tidak ada proses liar yang tertinggal memakan memori host; shutdown selalu bersih.
  - *Negatif:* Memerlukan kode platform adapter native khusus Windows Win32 dan Linux POSIX.
- **Security Implications:** Mencegah 9Router atau Hermes terus berjalan tanpa pengawasan dan tanpa antarmuka UI.
- **Performance Implications:** Nol overhead CPU runtime (dikelola langsung oleh kernel scheduler).
- **Reversibility:** Sangat Rendah; merupakan pilar keandalan sistem Hime.

---

## ADR-011: Arsitektur Event: In-Memory Tokio Broadcast Bus + SQLite Event Journal
- **Status:** **ACCEPTED**
- **Context:** Modul-modul internal Hime Core perlu berkomunikasi secara asynchronous dan mengalirkan status real-time ke UI desktop.
- **Problem:** Bagaimana mendesain event architecture yang decoupled tanpa overhead message broker eksternal (seperti Redis atau RabbitMQ)?
- **Considered Alternatives:**
  - *Alternatif A: Message Broker Eksternal (Redis/NATS)* (Memerlukan instalasi service pihak ketiga, boros RAM).
  - *Alternatif B (Terpilih): In-Process Async Broadcast Channel (Tokio) + SQLite Event Persistence*.
- **Decision:** Menggunakan **`tokio::sync::broadcast` in-memory channel** untuk event transient real-time, dikombinasikan dengan pencatatan tabel SQLite untuk event audit dan storage journal.
- **Rationale:**
  - Sangat cepat (throughput $> 500.000\text{ events/sec}$, latensi sub-mikrodetik).
  - Decoupled: Modul penerbit (*publisher*) tidak perlu mengetahui siapa saja subscriber event.
  - Event penting (audit izin, transaksi pemindahan berkas) tetap persisten di SQLite untuk pemulihan pasca-crash.
- **Consequences:**
  - *Positif:* Performa in-memory maksimal; nol dependensi luar; integrasi mudah dengan stream Tauri UI.
  - *Negatif:* Event in-memory yang belum sempat dicatat ke database akan hilang jika host mati listrik mendadak.
- **Security Implications:** Event payload difilter untuk mencegah kebocoran kredensial atau private chain-of-thought.
- **Performance Implications:** Konsumsi memori ring buffer event dibatasi maksimal 1.000 event ($\le 2\text{ MB}$).
- **Reversibility:** Tinggi; abstraksi trait `EventEngine` memisahkan implementasi channel dari domain logic.

---

## ADR-012: Strategi Konektivitas Android: USB ADB Client Engine (Phase 2)
- **Status:** **ACCEPTED (Penerapan Arahan DIR-02)**
- **Context:** Hime dirancang mengelola ekosistem digital personal pengguna, termasuk memantau dan memindahkan berkas unduhan/foto dari smartphone Android ke laptop.
- **Problem:** Protokol konektivitas apa yang paling andal, aman, dan tidak memerlukan instalasi aplikasi mobile rumit pada tahap awal?
- **Considered Alternatives:**
  - *Alternatif A: Custom Wireless P2P Server (WebRTC / Wi-Fi Direct)* (Kompleks, rentan pemblokiran firewall, memerlukan aplikasi Android native yang matang sejak awal).
  - *Alternatif B (Terpilih): USB ADB Client Protocol Engine (Penerapan Arahan DIR-02)*.
- **Decision:** Menggunakan konektivitas fisik **kabel USB melalui Android Debug Bridge (ADB) Client Engine** sebagai mekanisme integrasi awal Android pada Phase 2.
- **Rationale:**
  - Mematuhi arahan definitif Project Owner DIR-02.
  - Kecepatan transfer data kabel USB sangat tinggi ($\ge 35\text{ MB/sec}$) dan stabil tanpa gangguan sinyal Wi-Fi.
  - Tidak memerlukan aplikasi Android native khusus yang rumit untuk operasi inspeksi storage dan penarikan file.
  - Membuka jalur evolusi alami menuju Android Companion App native (Kotlin) pada Phase 11.
- **Consequences:**
  - *Positif:* Sangat andal; transfer berkas besar (video/foto) cepat; zero setup network configuration.
  - *Negatif:* Memerlukan pengaktifan *USB Debugging* pada perangkat Android pengguna; smartphone harus terhubung fisik dengan kabel USB.
- **Security Implications:** Seluruh transfer diverifikasi hash SHA-256; perintah dibatasi pada storage non-root (`/sdcard/`).
- **Performance Implications:** Utilisasi CPU rendah selama streaming transfer socket ADB.
- **Reversibility:** Menengah; modul dibungkus dalam `DeviceBridgeAdapter` trait sehingga siap ditambah adapter nirkabel di masa depan.
