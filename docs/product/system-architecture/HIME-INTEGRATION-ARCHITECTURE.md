# Hime Integration Architecture Specification

- **Document ID:** HIME-INTEGRATION-ARCHITECTURE
- **Task ID:** TASK-02-05 (System Architecture & Technical Design)
- **Phase:** Phase 02 — Nusa Reverse Engineering & Architecture Discovery
- **Authority:** Principal Systems Integration Architect & Staff Software Engineer
- **Status:** **APPROVED ARCHITECTURAL SPECIFICATION — PENDING OWNER REVIEW**
- **Date:** 2026-11-08

---

## 1. Executive Summary & Integration Strategy

Dokumen ini mendefinisikan arsitektur integrasi eksternal (*Integration Architecture*) untuk **Hime**, merancang adapter kontrak yang menghubungkan Hime Core dengan layanan backend lokal (Hermes Agent dan 9Router), perangkat keras pendamping (Android via USB ADB), serta ekosistem cloud dan produktivitas (GitHub, Google Drive, Browser Automation, dan Notifikasi Desktop).

### Prinsip Fondasi Integrasi Hime:
1. **Pola Adapter & Isolasi Antarmuka (*Adapter Pattern*):** Seluruh layanan eksternal diakses melalui adapter terstandarisasi. Logika bisnis inti Hime Core tidak pernah bergantung langsung pada struktur data proprietary pihak ketiga.
2. **Ketiadaan Modifikasi Repositori Eksternal (*Zero External Dependency Modification*):** Runtime pihak ketiga (Hermes Agent dari Nous Research dan 9Router dari decolua) diaudit dan dikonfigurasi melalui interface resmi (*configuration, CLI flags, environment variables*), tanpa mengubah source code internal mereka secara sepihak.
3. **Dual-Control Plane Topology:** Mengadopsi topologi Dual-Control Plane yang terverifikasi pada riset `RE/9ROUTER/13-HIME-INTEGRATION-FEASIBILITY.md` untuk memisahkan lalu lintas streaming data inferensi dari lalu lintas kontrol manajemen gateway.
4. **Keamanan Aliran Keluar (*Outbound Egress Security*):** Seluruh integrasi eksternal mematuhi prinsip *least privilege* dan verifikasi persetujuan pengguna untuk aksi yang bersifat publik atau destruktif.

---

## 2. Peta Topologi Integrasi Sistem Hime

```mermaid
graph TD
    subgraph HIME_CORE_HUB ["HIME CORE: INTEGRATION HUB & ADAPTER LAYER"]
        ADAPT_HERMES["Hermes Agent Adapter (IPC Stream)"]
        ADAPT_ROUTER["9Router Dual-Control Plane Adapter (HTTP Loopback)"]
        ADAPT_ADB["Android Device Bridge (USB ADB Client)"]
        ADAPT_BROWSER["Browser Automation Bridge (Playwright Subprocess)"]
        ADAPT_GITHUB["GitHub Workspace Adapter (Git CLI / REST)"]
        ADAPT_GDRIVE["Google Drive Sync Adapter (OAuth PKCE)"]
        ADAPT_NOTIF["Desktop Notification Adapter (Win32 Toast / D-Bus)"]
        ADAPT_DOCS["Document Text Extraction Engine (Native Parsers)"]
    end

    subgraph LOCAL_SERVICES ["LAYANAN BACKEND LOKAL (HOST)"]
        HERMES_PROC["Hermes Agent Runtime (Python 3.11 uv)"]
        ROUTER_PROC["9Router Engine (Node.js 20+ Port 20128)"]
        LOCAL_CHROME["Headless Chromium Sandbox Profile"]
        OS_NOTIF_DAEMON["OS Notification Center (Windows Action Center / D-Bus)"]
    end

    subgraph HARDWARE_EXTERNAL ["PERANGKAT KERAS EKSTERNAL"]
        ANDROID_PHONE["Smartphone Android (USB Debugging ADB)"]
    end

    subgraph CLOUD_SERVICES ["LAYANAN AWAN EKSTERNAL"]
        LLM_APIS["Cloud LLM Providers (OpenAI, Anthropic, Google, Groq)"]
        GITHUB_CLOUD["GitHub Cloud Repositories"]
        GOOGLE_CLOUD["Google Drive Storage Cloud"]
    end

    %% Hubungan
    ADAPT_HERMES <-->|Framed Stream IPC (Named Pipe / Domain Socket)| HERMES_PROC
    ADAPT_ROUTER <-->|Control Plane HTTP GET/POST /api/*| ROUTER_PROC
    HERMES_PROC <-->|Data Plane HTTP POST /v1/chat/completions| ROUTER_PROC
    ROUTER_PROC <-->|Outbound HTTPS SSE| LLM_APIS

    ADAPT_ADB <-->|ADB Socket Protocol over USB| ANDROID_PHONE
    ADAPT_BROWSER <-->|DevTools Protocol (CDP)| LOCAL_CHROME
    ADAPT_NOTIF -->|Win32 Toast / D-Bus Notifications| OS_NOTIF_DAEMON

    ADAPT_GITHUB <-->|HTTPS API / Git Transport| GITHUB_CLOUD
    ADAPT_GDRIVE <-->|HTTPS REST API (OAuth 2.0)| GOOGLE_CLOUD
```

---

## 3. Spesifikasi Rinci Adapter Integrasi Utama

---

### A. Integrasi Hermes Agent (Nous Research Runtime)
- **Peran:** Runtime penalaran otonom dan eksekusi workflow agentic.
- **Teknologi Interfacing:** High-Performance Stream IPC via Rust trait `IpcTransport` (Windows Named Pipe / Linux Unix Domain Socket).
- **Protokol:** Hime Wire Framing biner + payload JSON-RPC 2.0.
- **Mekanisme Otentikasi:** Handshake token CSPRNG 256-bit dan verifikasi Client PID kernel OS.
- **Pencegahan Bypass:** Seluruh modul alat pada registri alat Hermes dialihkan (*shimmed*) menggunakan adapter klien Python (`hermes_ipc_client.py`). Hermes Agent dilarang mengeksekusi aksi sistem operasi secara mandiri; seluruh permintaan dikirimkan ke Hime Core PEP/PDP.
- **Penanganan Kegagalan (*Failure Handling*):** Heartbeat ping 10 detik; jika proses Hermes crash, Hime Core Process Supervisor melakukan restart otomatis dan membangun kembali saluran komunikasi dalam waktu $\le \mathbf{2.0\text{ detik}}$.

---

### B. Integrasi 9Router AI Gateway (decolua Engine)
- **Peran:** Gateway multi-penyedia model AI terpadu, penyedia failover otomatis dua tingkat, dan streaming token inferensi.
- **Topologi:** **Dual-Control Plane Topology** (sesuai `RE/9ROUTER/13-HIME-INTEGRATION-FEASIBILITY.md`):
  1. **Data Plane (Jalur Data Inferensi):** Hermes Agent berkomunikasi langsung dengan 9Router via `http://127.0.0.1:20128/v1/chat/completions` menggunakan format payload standar OpenAI-compatible dengan Server-Sent Events (SSE) streaming.
  2. **Control Plane (Jalur Manajemen & Supervisi):** Hime Core berkomunikasi dengan 9Router melalui rute administratif `http://127.0.0.1:20128/api/*` untuk memantau status kesehatan gateway, memperbarui rute model aktif, dan mengecek ketersediaan kuota.
- **Konfigurasi Keamanan:**
  - `HOST=127.0.0.1` (Pemeriksaan ketat socket lokal, dilarang mengikat ke `0.0.0.0`).
  - `INITIAL_PASSWORD`: Token acak 32-karakter CSPRNG disuntikkan saat spawning proses, meniadakan ketergantungan pada fallback bawaan kode sumber `"123456"`.
  - Database lokal 9Router (`%APPDATA%\9router\db\data.sqlite`) diproteksi ACL 0700 dan wrapper enkripsi DPAPI terencana (`[PLANNED CONTROL]`).

---

### C. Integrasi Perangkat Mobile Android (USB ADB Client Bridge — DIR-02 / Phase 2)
- **Peran:** Mengelola status penyimpanan smartphone Android, mendeteksi berkas unduhan/foto lama yang menumpuk, dan memfasilitasi pemindahan berkas ke laptop secara aman.
- **Protokol Komunikasi:** Mengimplementasikan **Native ADB Client Protocol** dalam Rust yang berkomunikasi dengan daemon ADB lokal (`127.0.0.1:5037`) atau koneksi direct WinUSB ke smartphone:
  ```rust
  pub trait AndroidBridgeAdapter: Send + Sync {
      async fn query_device_storage_metrics(&self, device_serial: &str) -> Result<AndroidStorageInfo, AdbError>;
      async fn pull_file_verified(&self, device_serial: &str, remote_path: &str, local_target: &Path) -> Result<String, AdbError>;
  }
  ```
- **Siklus Hidup Transfer Aman:** Berkas dari Android ditarik ke berkas sementara `.hime_tmp` di laptop $\rightarrow$ Verifikasi hash SHA-256 $\rightarrow$ Rename atomik ke nama berkas final $\rightarrow$ Hapus berkas di ponsel hanya jika terkonfirmasi sukses 100%.

---

### D. Integrasi Repositori GitHub
- **Peran:** Membaca status repositori lokal, commit berkas hasil kerja agen, membuat branch baru, dan melakukan pull request.
- **Mekanisme Eksekusi:** Wrapper CLI terawasi di bawah Process Supervisor Hime Core.
- **Kontrol Keamanan:**
  - Operasi *Read-Only* (`git status`, `git diff`, `git log`) diklasifikasikan sebagai `RiskTier::Low` (Otomatis diizinkan di dalam workspace).
  - Operasi modifikasi (*Commit*, *Branching*) diklasifikasikan sebagai `RiskTier::Medium`.
  - Operasi publikasi (*Push ke Remote Origin*, *Force Push*) diklasifikasikan sebagai **`RiskTier::High` (Wajib Action Approval Modal eksplisit dari pengguna)**.
- **Kredensial:** Personal Access Token (PAT) disimpan di dalam `secrets_vault` Hime Core terenkripsi Windows DPAPI / Linux Secret Service.

---

### E. Integrasi Google Drive
- **Peran:** Sinkronisasi dokumen kerja dan pencadangan arsip proyek ke cloud pengguna.
- **Mekanisme Otentikasi:** OAuth 2.0 Authorization Code Flow dengan **PKCE (Proof Key for Code Exchange)** tanpa hardcoded client secret.
- **Batas Izin (*Scoped Access*):** Hanya meminta izin cakupan terbatas (`drive.file` — hanya mengakses berkas yang dibuat atau dibuka oleh aplikasi Hime), menolak izin akses penuh ke seluruh Google Drive pengguna.

---

### F. Integrasi Otomasi Peramban Web (Headless Browser Bridge — Phase 2/3)
- **Peran:** Melakukan pencarian web mendalam, membaca dokumentasi online, dan mengekstrak konten artikel berita/riset.
- **Arsitektur:** Menggunakan subprocess browser Chromium headless via Playwright/Puppeteer bridge.
- **Isolasi Keamanan:**
  - Menggunakan profil direktori browser sementara (*ephemeral clean profile*) yang dibersihkan setiap kali sesi selesai.
  - Dilarang mengakses profil browser harian pengguna (Chrome/Edge user data) untuk mencegah kebocoran session cookie atau password perbankan pengguna.

---

### G. Integrasi Notifikasi & Event Sistem Operasi
- **Platform Windows:** Menggunakan Win32 API `Shell_NotifyIconW` untuk system tray icon dan integrasi notifikasi desktop melalui Windows Action Center (WinRT Toast Notification).
- **Platform Linux:** Menggunakan D-Bus interface `org.freedesktop.Notifications` dan spesifikasi `StatusNotifierItem (SNI)` untuk baki sistem.
- **Intersepsi Volume Hardware:** Menggunakan Windows CoreAudio `IAudioEndpointVolumeCallback` dan PipeWire/PulseAudio native D-Bus listener untuk menampilkan HUD volume popup transient (~2.5 detik) tanpa polling timer.

---

### H. Integrasi Ekstraksi Dokumen (Document Extraction Engine)
- **Peran:** Ekstraksi teks multi-format dari berkas yang di-drop ke Dynamic Island.
- **Pustaka Native (In-Process Rust):**
  - PDF: Ekstraktor native `pdf-extract` (tanpa dependensi shell `pdftotext`).
  - Markdown / TXT / JSON / Code: Native UTF-8 stream reader dengan deteksi BOM.
- **Batas Beban:** Batas ukuran dokumen maksimum **50 MB** per berkas pada MVP untuk mencegah kehabisan alokasi RAM.
