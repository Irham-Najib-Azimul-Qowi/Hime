# 13-HIME INTEGRATION FEASIBILITY: 9ROUTER
**Project:** Hime Assistant — AI Model Routing Layer Research  
**Target:** System Architecture Integration of 9Router into Hime Desktop Platform  
**Scope:** Architectural Options, Separation of Concerns, Topology, & Reliability  
**Author:** Senior Reverse Engineer, AI Infrastructure Architect, Backend Engineer & Security Researcher  
**Date:** November 2026  
**Status:** Completed  

---

## 1. Executive Summary & Feasibility Verdict

Studi kelayakan arsitektural ini mengevaluasi peran dan penempatan **9Router** di dalam sistem **Hime Assistant**. Arsitektur konseptual Hime menempatkan Hime sebagai antarmuka desktop melayang (*floating dynamic island command center*), Hermes Agent sebagai otak penalaran otonom, dan 9Router sebagai lapisan gerbang model AI (*AI model routing layer*).

### Kesimpulan Kelayakan (Feasibility Verdict):
> **STATUS: SANGAT DIREKOMENDASIKAN (HIGHLY RECOMMENDED — MODEL HIBRIDA B)**
> 
> 9Router sangat layak dan sangat disarankan untuk diadopsi sebagai infrastruktur perutean model Hime. Keberadaan 9Router membebaskan Hime dan Hermes dari beban mengelola puluhan format API vendor (OpenAI, Anthropic, Gemini, Vertex, Bedrock) dan mengeliminasi risiko kegagalan sesi akibat batas kuota akun tunggal.

---

## 2. Evaluation of Architectural Topology Options

Kami menguji dan membandingkan 3 opsi topologi integrasi:

```
[ OPTION A: DIRECT DECOUPLED ]      [ OPTION B: DUAL-CONTROL PLANE (REKOMENDASI) ]    [ OPTION C: FULL PROXY ABSTRACTION ]
                                                                                      
      Hime UI                               Hime UI (Dynamic Island)                         Hime UI
         |                                     |                \                               |
  Hime Native Core                      Hime Native Core         \ (Status/Quota Query)  Hime Native Core
         |                              /              \          \                             |
   Hermes Agent              Hermes Agent              9Router     \                      Hime Proxy Bus
         |                        |                    /            v                           |
      9Router                     +-------------------+        (9Router APIs)            Hermes Agent
         |                                  |                                                   |
    AI Providers                       AI Providers                                          9Router
                                                                                                |
                                                                                           AI Providers
```

### Analisis Komparasi Arsitektural:

| Parameter Evaluasi | Opsi A: Hermes Direct to 9Router | Opsi B: Dual-Control Plane (Direkomendasikan) | Opsi C: Hime Abstraction Layer |
| :--- | :--- | :--- | :--- |
| **Deskripsi** | Hime hanya mengontrol Hermes; Hermes berkomunikasi langsung ke 9Router via `OPENAI_BASE_URL`. | Hermes mengambil inferensi langsung dari 9Router, namun Hime Native Core terhubung ke `/api/*` 9Router untuk metadata model & kuota. | Hime mencegat dan membungkus seluruh lalu lintas antara Hermes dan 9Router melalui proxy internal. |
| **Keuntungan** | Arsitektur paling sederhana dan minim titik kegagalan (*lowest complexity*). | **Sangat optimal:** Hime dapat menampilkan daftar model dan status kuota secara instan di Dynamic Island, sementara stream inferensi token tetap berkecepatan tinggi tanpa perantara tambahan. | Kontrol mutlak terhadap setiap byte data yang lewat. |
| **Kelemahan** | Hime tidak mengetahui status kesehatan akun atau sisa kuota kecuali Hermes melaporkan error. | Memerlukan dua koneksi IPC/HTTP lokal dari Hime Core (ke Hermes Gateway dan ke 9Router). | Sangat kompleks; menambahkan latensi overhead pada streaming token; rawan bottleneck memori. |
| **Kompleksitas Kode** | Rendah (Low) | **Sedang (Moderate — Seimbang)** | Sangat Tinggi (High / Over-engineered) |
| **Latensi Streaming** | Minimal (~0 ms overhead) | **Minimal (~0 ms overhead)** | Ada tambahan latensi proxy internal (+5-15 ms). |
| **Efek Kegagalan** | Terisolasi. | **Sangat tangguh:** Jika UI Hime ditutup, Hermes dan 9Router tetap dapat menyelesaikan tugas. | Jika proxy Hime crash, seluruh komunikasi Hermes terputus. |

---

## 3. Critical Integration Architectural Questions & Answers

### 3.1. Apakah Hermes Harus Berkomunikasi Langsung dengan 9Router?
**YA.** Hermes Agent harus mengirimkan permintaan `/v1/chat/completions` langsung ke 9Router. Mengalirkan token inferensi langsung antara proses Python Hermes dan Node.js 9Router memastikan kecepatan transmisi maksimum tanpa membebani thread antarmuka UI Hime.

### 3.2. Apakah Hime Harus Berkomunikasi dengan 9Router Secara Terpisah?
**YA.** Hime Native Core (Rust) harus membuka koneksi ke port manajemen 9Router untuk:
1. Memanggil `GET /v1/models/info` guna menampilkan daftar model dan kapabilitasnya pada menu pemilih model (*model selector flyout*) di Dynamic Island.
2. Memanggil `GET /api/usage/summary` untuk menampilkan widget biaya atau konsumsi token harian di panel drawer asisten.
3. Membaca event streaming log (`/dashboard/console-log` SSE) untuk mendeteksi event failover (*"Model X rate limited, switched to Model Y"*) dan menampilkannya sebagai animasi notifikasi halus pada Dynamic Island.

### 3.3. Bagaimana Kredensial & API Key Sebaiknya Dikelola?
- **Penyimpanan:** 9Router menyimpan kredensial di `%USERPROFILE%\.9router\9router.db`.
- **Pengaturan Pengguna:** Hime Assistant menyediakan tombol pengaturan cepat di island:
  - Pengguna dapat memasukkan API Key secara langsung melalui jendela native Hime (yang memanggil `POST /api/providers`), atau
  - Mengklik tombol *"Buka Dashboard Web 9Router"* untuk menjalankan alur OAuth browser yang kompleks (seperti login Google Gemini, GitHub Copilot, atau Kiro).

### 3.4. Bagaimana Pengelolaan Siklus Hidup Proses (*Process Lifecycle*)?
9Router harus dijalankan sebagai **Managed Background Child Process** yang diawasi oleh Hime Native Core:
1. **Pemeriksaan Startup:** Saat Hime diluncurkan, Rust core memeriksa apakah port `20128` aktif via HTTP ping ke `http://127.0.0.1:20128/api/health`.
2. **Auto-Spawn:** Jika tidak aktif, Hime mengeksekusi child process secara tersembunyi (*headless* tanpa jendela konsol hitam):
   `node custom-server.js --port 20128`
3. **Graceful Teardown:** Saat Hime ditutup, Hime mengirimkan sinyal penghentian bersih atau membiarkan 9Router tetap berjalan di background jika pengguna memilih mode standby.

---

## 4. Visualizing Fallback Events on Hime Dynamic Island

Salah satu keunggulan terbesar bagi pengalaman pengguna (*user experience*) Hime Assistant:

```
[ KONDISI NORMAL: PROMPT DIKIRIM ]
+---------------------------------------------------------------------------+
| (Hime Island) Hermes: Menulis kode refactoring... [Model: Claude 3.5]     |
+---------------------------------------------------------------------------+
                                  |
                                  | (Upstream Anthropic merespon HTTP 429)
                                  v
[ KONDISI FAILOVER: 9ROUTER MELAKUKAN FALLBACK TRANSPARAN ]
+---------------------------------------------------------------------------+
| (Hime Island) ⇄ Claude limit! Beralih otomatis ke DeepSeek-Coder...       |
| (Animasi Denyut Oranye Halus selama 2 detik)                              |
+---------------------------------------------------------------------------+
                                  |
                                  | (DeepSeek menghasilkan respons)
                                  v
[ KONDISI LANJUTAN: RESPON MENGALIR KEMBALI ]
+---------------------------------------------------------------------------+
| (Hime Island) Hermes: Meneruskan penulisan kode... [Model: DeepSeek]      |
+---------------------------------------------------------------------------+
```
Pengguna mendapatkan visibilitas instan atas apa yang terjadi tanpa sesi terhenti atau muncul dialog pesan error yang membingungkan.

---

## 5. Risk Assessment & Mitigations

| Potensi Risiko | Tingkat Dampak | Probabilitas | Strategi Mitigasi Hime |
| :--- | :--- | :--- | :--- |
| **9Router Terhenti / Crash** | Tinggi | Rendah | Hime Native Core menyematkan *Health Check Heartbeat* (setiap 5 detik). Jika 9Router mati, Hime merestartnya secara otomatis dalam <1 detik. |
| **Konflik Port TCP 20128** | Sedang | Rendah | Hime mendeteksi ketersediaan port; jika terpakai, Hime dapat mengalokasikan port dinamis berikutnya (misal `20129`) dan mengoperkannya via env. |
| **Konsumsi Memori Kumulatif** | Rendah | Sedang | Hime UI (~30 MB) + Hermes (~150 MB) + 9Router (~70 MB) = Total ~250 MB RAM, jauh di bawah batas wajar aplikasi desktop modern (Electron rata-rata memakan 500 MB - 1 GB). |
| **Kerentanan File DB Plaintext** | Sedang | Sedang | Hime menerapkan izin ACL Windows ketat pada direktori `%USERPROFILE%\.9router` agar hanya akun Windows aktif yang memiliki izin baca/tulis. |

---

## 6. Kesimpulan Kelayakan Integrasi

Integrasi 9Router ke dalam ekosistem Hime Assistant **sangat layak, aman, dan merupakan keputusan arsitektur strategis bernilai tinggi**. 

Dengan memadukan:
1. **Nusa Dynamic Island Shell (Tauri 2.0 / Rust)** sebagai antarmuka visual kelas dunia.
2. **Hermes Agent (Nous Research)** sebagai mesin eksekusi penalaran otonom dan pengontrol sistem Windows.
3. **9Router (`decolua`)** sebagai jaringan transmisi model AI multi-provider yang kebal terhadap kegagalan kuota.

Hime Assistant akan memiliki fondasi infrastruktur AI terlengkap, paling tangguh, dan paling hemat biaya di platform desktop Windows.

