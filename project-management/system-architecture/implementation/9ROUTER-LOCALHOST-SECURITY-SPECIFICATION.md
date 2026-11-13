# 9Router Localhost Authentication & Security Boundary Analysis (IB-02)

- **Document ID:** HIME-SPEC-04-05
- **Task ID:** TASK-04-05 (IB-02 Localhost Authentication & Proxy Security)
- **Phase:** Phase 04 — Engineering Planning & Implementation Specification
- **Authority:** System Architect & Technical Lead
- **Execution Role:** Senior Security Architect & Systems Engineer
- **Status:** **VALIDATED TECHNICAL SPECIFICATION — IB-02 REMAINS OPEN IMPLEMENTATION BLOCKER**
- **Date:** 2026-11-10

---

## 1. Executive Summary & Problem Formulation

### Hakikat Kerentanan Localhost Bypass (IB-02):
Konfigurasi 9Router yang mengikat port pada antarmuka loopback (`127.0.0.1` atau `[::1]`) secara efektif memblokir akses paket jaringan dari luar mesin host (*remote network boundary*). Namun, **binding ke localhost BUKAN merupakan mekanisme autentikasi proses**.

Pada sistem operasi Windows 11 dan Linux, seluruh proses aplikasi yang dijalankan di bawah konteks pengguna yang sama (*same-user security context*) berbagi *network namespace* loopback yang sama. Akibatnya, aplikasi atau malware lokal yang tidak berizin dapat langsung membuka koneksi TCP ke port 9Router dan memanggil endpoint perutean model AI tanpa melalui otorisasi Hime Core (*Direct Port Bypass*).

### Status Pemblokir Implementasi:
**IB-02 TETAP BERSTATUS `OPEN IMPLEMENTATION BLOCKER`.**
Keterbatasan struktural sistem operasi Windows 11 bagi aplikasi non-administrator menuntut evaluasi yang jujur: tidak ada mekanisme non-admin murni pada Windows yang mampu secara kriptografis memblokir koneksi TCP loopback antar-proses dalam konteks pengguna yang sama jika server target (9Router) tidak memvalidasi autentikasi per-koneksi.

---

## 2. Validasi Kritis & Klarifikasi Asumsi Teknis

Berdasarkan tinjauan arsitektural dan verifikasi terhadap kode sumber 9Router, dilakukan koreksi mendasar terhadap klaim-klaim sebelumnya:

### 1. Apakah 9Router memvalidasi header `X-Hime-Internal-Secret`?
**TIDAK, 9ROUTER TIDAK MEMVALIDASI HEADER TERSEBUT.**
- Kode sumber upstream 9Router (Express / Node.js HTTP routes) **TIDAK MEMILIKI middleware** yang memeriksa header kustom seperti `X-Hime-Internal-Secret`.
- 9Router hanya memeriksa token `Authorization: Bearer <key>` terhadap tabel internal `apiKeys` miliknya (jika autentikasi API diaktifkan) atau memeriksa sesi cookie login web.
- **Koreksi Arsitektural:** Menyuntikkan header rahasia kustom dari Hime Core Reverse Proxy ke 9Router sama sekali **TIDAK MEMBERIKAN PERLINDUNGAN** jika 9Router sendiri mengabaikan header tersebut dan tetap memproses request yang datang tanpa header itu.

### 2. Apakah endpoint localhost 9Router tetap dapat diakses langsung?
**YA, DAPAT DIAKSES LANGSUNG SECARA BEBAS.**
- Port loopback yang dibuka oleh 9Router dapat dihubungi oleh proses apa pun di komputer lokal melalui panggilan TCP standar (`net.Socket` Node.js, `curl`, Python `requests`, Rust `TcpStream`).
- Jika sebuah proses lokal mengirimkan HTTP POST langsung ke `http://127.0.0.1:<port>/v1/chat/completions`, request tersebut akan langsung diproses oleh 9Router tanpa melewati Hime Core Reverse Proxy.

### 3. Apakah Reverse Proxy Hime Core dapat dilewati (*Bypassed*)?
**YA, REVERSE PROXY DAPAT DILEWATI SECARA PENUH.**
- Hime Core Reverse Proxy hanya menjadi gerbang bagi klien yang secara sukarela mengarahkan request kepadanya.
- Reverse proxy tidak memiliki kuasa fisik di level OS untuk memaksa koneksi pihak ketiga melewati dirinya, kecuali port 9Router diproteksi oleh firewall kernel.

### 4. Apakah verifikasi PID (`GetExtendedTcpTable`) dapat mendeteksi atau mencegah pembajakan port?
- **Yang DAPAT dilakukan PID Verification:** Win32 API `GetExtendedTcpTable` dapat memeriksa PID dari proses yang sedang *LISTENING* pada port target saat Hime Core melakukan inisialisasi. Ini efektif untuk mendeteksi *Port Squatting* (proses asing menduduki port sebelum 9Router dinyalakan).
- **Yang TIDAK DAPAT dilakukan PID Verification:** API ini **TIDAK DAPAT MEMERIKSA CLIENT PID** dari koneksi TCP inbound yang masuk ke 9Router secara runtime. Win32 socket stack standar tidak mengaitkan PID klien ke server socket tanpa inspeksi mendalam (ETW / Windows Filtering Platform kernel driver). Dengan demikian, PID check tidak mencegah proses lokal lain menjadi *klien* langsung ke 9Router.

### 5. Bagaimana penanganan IPv4 dan IPv6?
- Jika 9Router mengikat port hanya pada IPv4 `127.0.0.1`, request klien yang me-resolve `localhost` ke IPv6 `::1` akan gagal (*connection refused*).
- Jika 9Router mengikat port secara dual-stack (`::` atau `::1`), permukaan serangan lokal terbuka pada kedua antarmuka loopback.
- Hime wajib memastikan konfigurasi bind 9Router disinkronkan secara eksplisit (`--host 127.0.0.1` atau loopback absolut) untuk mencegah exposure tidak disengaja ke interface publik (`0.0.0.0`).

### 6. Apakah penggunaan Port Ephemeral benar-benar memberi manfaat keamanan?
**HANYA MEMBERIKAN *SECURITY THROUGH OBSCURITY*, BUKAN *SECURITY BOUNDARY*.**
- Port acak (misalnya 54321) memang mencegah penyerang menargetkan port statis terkenal `20128`.
- Namun, proses apa pun yang berjalan di bawah pengguna yang sama dapat memanggil Win32 API `GetExtendedTcpTable` atau utilitas `netstat -ano` untuk menemukan port listener yang dimiliki oleh proses `node.exe` / `9router` dalam waktu $< 50\text{ms}$.
- Oleh karena itu, port ephemeral bukan kontrol keamanan primer yang dapat dibuktikan secara kriptografis.

---

## 3. Realitas Penegakan Keamanan Windows 11 Tanpa Hak Administrator

Pada lingkungan Windows 11 standar, Hime beroperasi sebagai aplikasi desktop non-administrator (*standard user privileges*):

| Mekanisme OS | Kebutuhan Privilese | Efektivitas Isolasi Loopback | Kelayakan untuk Hime Non-Admin |
|:---|:---:|:---|:---:|
| **Windows Filtering Platform (WFP) Rules** | Administrator / Driver | **Sangat Tinggi:** Dapat memblokir koneksi TCP loopback berdasarkan App ID / PID. | **TIDAK LAYAK** (Membutuhkan hak admin / UAC prompt). |
| **Windows Defender Firewall per-Port Rule** | Administrator | **Tinggi:** Dapat membatasi port loopback hanya untuk binary tertentu. | **TIDAK LAYAK** (Membutuhkan hak admin). |
| **AppContainer Isolation / Sandbox** | Standard User | **Tinggi:** Membatasi akses jaringan inter-process antar sandbox. | **Sangat Rumit:** Memerlukan packaging MSIX penuh dan membatasi akses filesystem runtime. |
| **User Account Isolation (Dedicated Service Account)** | Administrator | **Sangat Tinggi:** Menjalankan 9Router di bawah SID pengguna berbeda. | **TIDAK LAYAK** (Membutuhkan hak admin untuk membuat akun layanan). |
| **Token-Based Application Auth (API Key di 9Router)** | Standard User | **Tinggi:** 9Router sendiri menolak request tanpa valid API key. | **LAYAK:** Memanfaatkan fitur `apiKeys` internal milik 9Router. |

---

## 4. Strategi Pertahanan Realistis untuk Penanganan IB-02

Karena pembatasan jaringan tingkat kernel tidak dapat dilakukan tanpa hak administrator, penegakan keamanan localhost wajib dialihkan ke **Lapisan Autentikasi Aplikasi 9Router**:

```mermaid
graph TD
    subgraph Client_Space ["Local Client Space"]
        LegitClient["Hime Desktop / Hermes Agent"]
        RogueApp["Untrusted Local App"]
    end

    subgraph Hime_Core_Domain ["Hime Core Security Domain"]
        PEP["Policy Engine & Reverse Proxy"]
        TokenGen["CSPRNG Master Key Manager"]
    end

    subgraph Router_Domain ["9Router Engine (Localhost)"]
        RouterEngine["9Router Internal API Gateway"]
        AuthTable["9router.db (apiKeys table)"]
    end

    %% Flows
    TokenGen -->|1. Generate Random Master Key & Inject via CLI/DB| AuthTable
    LegitClient -->|2. Authenticated IPC Request| PEP
    PEP -->|3. Forward HTTP + Bearer sk-hime-...| RouterEngine
    RouterEngine -->|4. Validates Bearer Token| AuthTable
    RouterEngine -->|5. 200 OK Response| PEP

    %% Rogue Attack
    RogueApp -.->|Direct HTTP Request (No Key or Guess Key)| RouterEngine
    RouterEngine -.->|401 Unauthorized: Invalid API Key| RogueApp
```

### Langkah-Langkah Penegakan:
1. **Pemberdayaan Autentikasi Asli 9Router (`requireApiKey: true`):**
   - Saat inisialisasi cold start, Hime Core mengonfigurasi basis data 9Router (`settings` table) dengan flag `requireApiKey = true`.
   - Hime Core menghasilkan kunci acak CSPRNG tingkat tinggi (`sk-hime-master-` diikuti 64 karakter hex) dan memasukkannya ke dalam tabel `apiKeys` pada `9router.db`.
2. **Eksklusivitas Kepemilikan Kunci Master:**
   - Kunci master ini HANYA diketahui oleh Hime Core dan disimpan di memori `Zeroize`.
   - Hime Core Reverse Proxy selalu menyematkan header `Authorization: Bearer sk-hime-master-...` pada setiap request yang diteruskan ke 9Router.
3. **Penolakan Otentikasi Native oleh 9Router:**
   - Setiap proses lokal lain yang mencoba mengakses port loopback 9Router secara langsung tanpa menyertakan kunci master ini akan ditolak secara native oleh engine 9Router dengan kode status `401 Unauthorized`.
4. **Proteksi File Database `9router.db`:**
   - Karena penyerang lokal berpotensi membaca file `9router.db` untuk mencuri kunci master, direktori `%LOCALAPPDATA%\Hime\runtime\9router\` wajib dilindungi menggunakan DACL NTFS ketat yang hanya memberi izin baca kepada User SID Hime.

---

## 5. Batasan Residual & Alasan IB-02 Tetap Menjadi Blocker

Meskipun strategi di atas sangat realistis dan dapat diimplementasikan tanpa hak administrator, terdapat **batasan residual inheren (*Inherent Residual Risks*)**:
- **Same-User File Access:** Malware atau proses jahat yang berjalan di bawah User SID yang sama dengan pengguna memiliki hak akses NTFS yang sama untuk membaca file `9router.db` di mana kunci master disimpan.
- **Process Memory Inspection:** Proses lain dengan tingkat integritas yang sama (*Medium Integrity*) dapat menggunakan API diagnostik OS untuk membaca memori proses 9Router jika tidak diproteksi oleh anti-debugging.

### Kesimpulan Tata Kelola:
**IB-02 TETAP BERSTATUS `OPEN IMPLEMENTATION BLOCKER`** sampai strategi integrasi token master 9Router ini diimplementasikan, diuji secara dinamis terhadap skenario serangan penetrasi lokal (*Direct Bypass Attack Test*), dan diverifikasi bahwa 9Router benar-benar menolak koneksi langsung tanpa token pada **Security Gate SG-03** (Phase 08).
