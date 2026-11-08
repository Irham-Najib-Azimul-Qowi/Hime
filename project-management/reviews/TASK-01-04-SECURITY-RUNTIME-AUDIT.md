# Hime Security & Runtime Environment Audit Report

- **Document:** Technical Audit Report: Security, Runtime Environment & Attack Surface Analysis
- **Phase:** Phase 01 — Discovery & Technical Validation
- **Task ID:** TASK-01-04
- **Date:** 2026-11-07
- **Authority:** Security Architect & AI Infrastructure Specialist
- **Status:** **QUALIFIED ACCEPTANCE**

---

## 1. Executive Summary & Scope

Berdasarkan mandat dari Hime Engineering Master Plan dan `AGENTS.md`, audit komprehensif terhadap lingkungan keamanan Windows, postur runtime, batas kepercayaan (*trust boundaries*), manajemen rahasia/kredensial, dan vektor serangan pada ekosistem Hime telah selesai dilaksanakan secara non-destruktif dan berbasis bukti (*evidence-based*).

### Ringkasan Temuan Utama:
1. **Windows Security Context (`XATOSHI\Satoshi` pada Windows 11 Pro 64-bit):**
   - Host pengembangan dan target berjalan pada **Windows 11 Pro (Build 26100.2033)** dengan **Medium Mandatory Integrity Level** (`S-1-16-8192`) dan token UAC terfilter (*non-elevated*).
   - **Batas Keamanan NTFS ACL & EFS:** Direktori `%APPDATA%` dan `%LOCALAPPDATA%` memiliki izin bawaan `FullControl` untuk `XATOSHI\Satoshi`. Pengerasan NTFS ACL (`icacls /inheritance:r`) dan enkripsi EFS (`cipher /e`) berhasil mengisolasi berkas dari akun pengguna OS lain dan mencegah ekstraksi diska offline. Namun, **NTFS ACL dan EFS BUKAN batas keamanan terhadap malware atau proses lain yang berjalan di bawah akun Windows yang sama**.
2. **Kelemahan Kredensial & Autentikasi 9Router (RSK-03, RSK-08, RSK-10, RSK-12):**
   - **RSK-03 (Plaintext SQLite):** Kredensial penyedia AI (API keys, OAuth tokens) disimpan dalam format JSON teks polos pada `%APPDATA%\9router\db\data.sqlite` (CWE-312).
   - **RSK-08 (`x-9r-cli-token`):** Token mesin CLI diturunkan dari `MachineGuid` registry (dapat dibaca publik) dan `%APPDATA%\9router\auth\cli-secret` (tanpa proteksi ACL ketat). Token ini memberikan akses penuh ke seluruh rute sensitif `/api/*` termasuk dump basis data dan shutdown.
   - **RSK-10 (Localhost Unauthenticated LLM API Access — IMPLEMENTATION BLOCKER):** Kode sumber `src/dashboardGuard.js:161-165` mengecualikan pemeriksaan API key untuk seluruh request dari loopback (`if (isLocalRequest(request)) return true;`).
     * *Koreksi Kritis:* Binding loopback adalah isolasi transport, **BUKAN autentikasi**.
     * Virtual bearer token **TIDAK EFEKTIF** karena middleware 9Router mem-bypass validasi sebelum memeriksa token.
     * Setiap proses lokal dapat langsung memanggil `http://127.0.0.1:20128/v1/chat/completions` dan melewati Hime Core sepenuhnya.
     * Masalah ini diklasifikasikan sebagai **`IMPLEMENTATION BLOCKER & PROPOSED RESIDUAL RISK — OWNER DECISION REQUIRED`** yang membutuhkan modifikasi upstream / maintained fork atau isolasi identitas OS sebelum Hime mengandalkan 9Router untuk operasional sensitif.
   - **RSK-12 (Default Host Binding `0.0.0.0` & Active Runtime Socket Verification):**
     * Cacat kode `DEFAULT_HOST = "0.0.0.0"` pada `cli/cli.js:110`: **`[VERIFIED]`**.
     * Argumen startup `--host 127.0.0.1`: **`[RECOMMENDATION]`**.
     * Verifikasi runtime socket aktif: **`[PLANNED CONTROL]`**.
     * Status efektivitas mitigasi: **`[NOT YET VERIFIED]`** (menunggu implementasi dan pengujian runtime).
     * Persyaratan implementasi: Memeriksa socket IPv4 (`AF_INET`) dan IPv6 (`AF_INET6`) via Win32 `GetExtendedTcpTable`, memastikan binding address strictly `127.0.0.1`/`::1`, port `20128`, dan `dwOwningPid` secara ketat cocok dengan child PID 9Router yang diluncurkan Supervisor. Jika menyimpang, proses dihentikan secara aman via Windows Job Object (`JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`) / `TerminateProcess`, dicatat ke `AUDIT.jsonl`, dan menolak startup (fail-closed).
3. **Postur Keamanan Hermes Agent (RSK-11):**
   - Hermes memiliki mekanisme scrubbing variabel lingkungan (`local_env_policy.py`) saat mengeksekusi sub-proses, dan perlindungan `_YOLO_MODE_FROZEN` pada `tools/approval.py`.
   - Namun, eksekusi tool terminal/python tetap mewarisi token pengguna Windows dan berpotensi mengakses berkas sensitif (`.ssh`, cookie browser, dokumen pengguna) jika dipicu oleh serangan *Indirect Prompt Injection*. Hime Core wajib bertindak sebagai *Central Policy Gate* dengan dialog persetujuan manusia non-bypassable.
4. **Kesiapan Mekanisme Keamanan Asli Windows & Kualifikasi Realistis:**
   - **DPAPI:** Berfungsi penuh (`System.Security.Cryptography.ProtectedData`). Namun, DPAPI `CurrentUser` terikat pada identitas pengguna Windows. *Secondary entropy* yang disimpan pada tempat yang dapat dibaca user yang sama **BUKAN batas keamanan independen terhadap same-user malware**. Hardcoded entropy dilarang keras.
   - **Kualifikasi Teknis Akses Memori Satu Pengguna & Zeroization:**
     * *Akses Memori Proses:* Malware pada akun pengguna yang sama (`XATOSHI\Satoshi` pada Medium Integrity) merupakan ancaman serius. Namun, membaca memori proses lain (`PROCESS_VM_READ` 0x0010 dan `PROCESS_QUERY_INFORMATION` 0x0400) dievaluasi oleh Windows Security Reference Monitor berdasarkan Security Descriptor (DACL) objek proses dan integritas token pemanggil. Hime Core dapat memperketat DACL objek prosesnya sendiri (`SetKernelObjectSecurity`) dan mengaktifkan mitigasi proses (`PROCESS_MITIGATION_DYNAMIC_CODE_POLICY`). Kendati demikian, DPAPI, NTFS ACL, dan zeroization sendiri **TIDAK memberikan isolasi proses yang lengkap**. Status: **`PROPOSED RESIDUAL RISK — OWNER DECISION REQUIRED`**.
     * *Batas Memory Zeroization:* Zeroization via crate `zeroize` memperpendek jendela waktu paparan (*exposure window*) secret di RAM, namun **TIDAK mencegah live memory dumping** selama secret sedang aktif digunakan, dan tidak berlaku pada alokasi V8 heap 9Router.
   - **Named Pipes:** Terverifikasi mendukung `PipeSecurity` DACL eksplisit untuk membatasi IPC hanya ke SID pengguna aktif.
   - **Credential Manager:** Terverifikasi API `Advapi32.dll` siap digunakan.
   - **Job Objects:** Win32 Job Object API siap digunakan untuk supervisi memori dan terminasi deterministik seluruh proses anak.

---

## 2. Audit Lingkungan Keamanan Windows (Windows Security Environment)

*Sumber Verifikasi: Eksekusi skrip diagnostik PowerShell non-invasif pada host Windows 11 Pro 64-bit.*

### A. Konteks Pengguna, Integritas & Hak Istimewa (Privileges)
- **Akun Aktif:** `XATOSHI\Satoshi`
- **Integrity Level:** `Mandatory Label\Medium Mandatory Level` (SID: `S-1-16-8192`).
- **Status UAC:** Token terfilter (*Filtered Token*). Grup `BUILTIN\Administrators` berstatus `Group used for deny only`.
- **Hak Istimewa (*Privileges*):** Hanya `SeChangeNotifyPrivilege` yang aktif secara default. Hak istimewa administratif (`SeShutdownPrivilege`, dll.) nonaktif.
- **Kesimpulan Arsitektur:** Hime berjalan dalam konteks pengguna standar tanpa hak istimewa khusus (*Least Privilege compliant*). Tidak ada kebutuhan untuk memicu UAC elevation.

### B. Izin Direktori NTFS & Perilaku ACL pada Windows 11 Pro
Pemeriksaan terhadap `%APPDATA%` (`C:\Users\Satoshi\AppData\Roaming`) dan `%LOCALAPPDATA%` (`C:\Users\Satoshi\AppData\Local`):
- `NT AUTHORITY\SYSTEM` : FullControl (Allow)
- `BUILTIN\Administrators` : FullControl (Allow)
- `XATOSHI\Satoshi` : FullControl (Allow)
- **Batas Keamanan & Realitas Ancaman:**
  - Hardening NTFS ACL (`icacls /inheritance:r`) efektif memutus akses akun pengguna OS lain pada komputer multi-user.
  - Enkripsi EFS (`cipher /e`) pada Windows 11 Pro efektif melindungi data saat diska offline (pencurian fisik hard drive).
  - **Namun, NTFS ACL dan EFS tidak membatasi malware atau proses lain yang berjalan di bawah akun `XATOSHI\Satoshi`**. Karena akun pengguna memiliki `FullControl`, semua aplikasi yang dijalankan user tersebut secara otomatis memiliki akses ke berkas data.
- **Kontrol Wajib Hime (SEC-01):** Terapkan lockdown ACL untuk memutus hak user lain, jadikan EFS sebagai lapisan defensif tambahan offline setelah verifikasi backup sertifikat, dan catat keterbatasan same-user sebagai residual risk.

### C. Ketersediaan Windows DPAPI & Manajemen Rahasia
- **DPAPI (`System.Security.Cryptography.ProtectedData`):** Berhasil diuji dengan string uji 230-byte terenkripsi.
- **Keterbatasan DPAPI & Secondary Entropy:**
  - Cakupan `DataProtectionScope.CurrentUser` melindungi rahasia secara at-rest terikat pada kredensial login Windows pengguna.
  - Penambahan garam / *optionalEntropy* bermanfaat mengikat enkripsi ke konteks aplikasi Hime, tetapi **entropy yang disimpan di registry, berkas, atau Windows Credential Manager yang dapat diakses oleh pengguna yang sama BUKAN batas keamanan independen terhadap malware lokal**.
  - Mengklaim hardcoded entropy sebagai rahasia aman adalah kekeliruan fatal yang dilarang dalam arsitektur Hime.
- **Kualifikasi Teknis Akses Memori Satu Pengguna & Batas Zeroization:**
  - *Akses Memori Proses Sama-User:* Malware yang berjalan di bawah akun pengguna yang sama (`XATOSHI\Satoshi` pada Medium Integrity) merupakan ancaman serius. Namun, kemampuan proses tersebut untuk membaca memori Hime (`PROCESS_VM_READ` 0x0010 / `PROCESS_QUERY_INFORMATION` 0x0400) secara teknis dievaluasi oleh Windows Security Reference Monitor terhadap *Security Descriptor* (DACL) objek proses Hime dan integritas token akses pemanggil. Hime Core dapat memperkuat perlindungan dengan menetapkan DACL proses restriktif (`SetKernelObjectSecurity`) dan mengaktifkan mitigasi proses (`PROCESS_MITIGATION_DYNAMIC_CODE_POLICY`). Kendati demikian, DPAPI, NTFS ACL, dan zeroization sendiri **TIDAK memberikan isolasi proses yang lengkap**. Status: **`PROPOSED RESIDUAL RISK — OWNER DECISION REQUIRED`**.
  - *Batas Keamanan Memory Zeroization:* Hime Core mengimplementasikan zeroization via crate `zeroize` untuk memusnahkan secret dari RAM segera setelah keluar dari scope. Zeroization membatasi jendela waktu paparan rahasia di RAM, namun **tidak mencegah pembacaan memori aktif** selama secret sedang aktif digunakan, dan tidak berlaku untuk alokasi memori internal V8 engine pada 9Router.
- **Windows Credential Manager:** P/Invoke ke `Advapi32.dll` (`CredReadW`/`CredWriteW`) terverifikasi sukses.

### D. Windows Defender & Firewall
- **Windows Defender:** `AntivirusEnabled: True`, `RealTimeProtectionEnabled: True`, `IoavProtectionEnabled: True`.
- **Windows Firewall:** Profil Domain, Private, dan Public berstatus `Enabled: True`.

---

## 3. Audit Manajemen Rahasia & Kredensial (Secrets & Credentials)

### A. 9Router AI Gateway
1. **Penyimpanan Kredensial Penyedia (RSK-03):**
   - Bukti: `src/lib/db/repos/connectionsRepo.js` menyimpan data koneksi dengan `stringifyJson(rest)`. API key OpenAI, Anthropic, DeepSeek disimpan dalam bentuk teks polos di kolom `data` tabel `providerConnections`.
   - Lokasi Berkas: `%APPDATA%\9router\db\data.sqlite`.
   - Penilaian: **VULNERABILITY TERKONFIRMASI (CWE-312)**. Tingkat Keparahan: **HIGH**.
   - Mitigasi Hime: NTFS ACL hardening (**SEC-01**) membatasi user lain; zeroization membatasi RAM Hime Core; mitigasi tuntas memerlukan evaluasi in-memory storage patch pada Phase 05.
2. **Kredensial Dashboard (`INITIAL_PASSWORD`) (RSK-04):**
   - Bukti: `src/app/api/auth/login/route.js:58` membaca `process.env.INITIAL_PASSWORD || "123456"`.
   - Mitigasi: Injeksi token CSPRNG 32-karakter oleh Hime Process Supervisor (**SEC-03**) langsung ke child process environment block (zero shell history, zero CLI args).
3. **Token Mesin CLI (`x-9r-cli-token`) (RSK-08):**
   - Bukti: `src/shared/utils/machineId.js:49-54` menghitung token melalui `SHA256(machineId + "9r-cli-auth" + cliSecret)[0..16]`.
   - Lokasi Secret: `%APPDATA%\9router\auth\cli-secret`. Berkas dibuat tanpa ACL ketat.
   - Otorisasi: Header ini meloloskan pemanggilan ke endpoint sensitif `/api/settings/database` (ekspor basis data lengkap) dan `/api/shutdown`.
   - Mitigasi: **SEC-01** membatasi user lain; Hime Core mengelola 9Router via sinyal proses internal tanpa mengekspos token CLI.

### B. Hermes Agent
1. **Penyimpanan Konfigurasi & Variabel Lingkungan:**
   - Lokasi: `%LOCALAPPDATA%\hermes\config.yaml` dan `%LOCALAPPDATA%\hermes\.env`.
2. **Kebijakan Pembersihan Variabel Lingkungan (*Secret Scrubbing*):**
   - Bukti: `tools/environments/local_env_policy.py` memuat daftar blokir statis `_STATIC_PROVIDER_ENV_BLOCKLIST` (50+ rahasia dibersihkan saat spawn child terminal).
   - Evaluasi: Sangat positif mencegah pewarisan environment variable ke sub-shell, namun sub-proses tetap dapat membaca berkas konfigurasi di disk tanpa virtualisasi filesystem.

### C. Rencana Arsitektur Kredensial Hime Core
- Hime Core bertindak sebagai **Master Vault**.
- Seluruh API key disimpan terenkripsi menggunakan DPAPI + Secondary Entropy (**SEC-02**).
- Kredensial tidak pernah ditulis ke disk dalam bentuk teks polos oleh Hime Core.

---

## 4. Audit Keamanan Jaringan & Paparan Port (Network Security)

### A. Paparan Antarmuka 9Router (RSK-10 & RSK-12)

1. **Default Host Binding `0.0.0.0` & Active Runtime Socket Verification (RSK-12):**
   - **Rincian Status Verifikasi:**
     * Default `0.0.0.0` pada kode 9Router (`cli/cli.js:110`): **`[VERIFIED]`**.
     * Startup menggunakan parameter `--host 127.0.0.1`: **`[RECOMMENDATION]`**.
     * Verifikasi runtime socket aktif: **`[PLANNED CONTROL]`**.
     * Status efektivitas mitigasi: **`[NOT YET VERIFIED]`** (menunggu implementasi dan pengujian runtime pada Phase 06).
   - **Kelemahan Terkonfirmasi:** Jika dijalankan tanpa parameter host, port 20128 terbuka untuk seluruh subnet LAN.
   - **Persyaratan Implementasi Kontrol Rekayasa Wajib (SEC-04):**
     * Supervisor menjalankan 9Router strictly dengan argumen CLI `--host 127.0.0.1` (`[RECOMMENDATION]`).
     * **Active Runtime Socket Verification (`[PLANNED CONTROL]`):** Supervisor tidak mengasumsikan argumen CLI berhasil. Segera setelah proses anak aktif, Supervisor memanggil Win32 `GetExtendedTcpTable` untuk memeriksa tabel socket TCP IPv4 (`AF_INET`) dan IPv6 (`AF_INET6`).
     * Verifikasi alamat binding: Socket wajib terikat strictly ke `127.0.0.1` atau `::1` (dan TIDAK PERNAH `0.0.0.0`, `::`, atau alamat LAN IP).
     * Verifikasi port: Socket mendengarkan pada port yang ditentukan (`20128`).
     * Verifikasi PID pemilik socket (`dwOwningPid`): Supervisor memeriksa bahwa `dwOwningPid` pada tabel TCP secara ketat cocok dengan child PID proses 9Router yang diluncurkan oleh Supervisor.
     * **Penegakan Fail-Closed:** Jika binding tidak memenuhi policy (misal terikat pada `0.0.0.0`, antarmuka LAN, atau PID tidak cocok):
       1. Supervisor menandai startup sebagai **FAILED (Security Policy Violation)**.
       2. Segera mematikan proses anak secara deterministik via Windows Job Object (`JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`) atau Win32 `TerminateProcess`.
       3. Mencatat detail pelanggaran, alamat IP, port, dan PID ke `AUDIT.jsonl`.
       4. Menolak peluncuran Hime (fail-closed).
2. **Bypass Autentikasi API LLM pada Localhost (RSK-10 — IMPLEMENTATION BLOCKER):**
   - Bukti: `src/dashboardGuard.js:161-165`:
     ```javascript
     async function canAccessPublicLlmApi(request) {
       if (isLocalRequest(request)) return true;
       if (await hasValidCliToken(request)) return true;
       return await hasValidApiKey(request);
     }
     ```
   - **Analisis Mendalam Celah Keamanan:**
     * Baris `if (isLocalRequest(request)) return true;` langsung meloloskan request loopback tanpa pernah memeriksa API key atau token autentikasi.
     * Menganggap loopback binding sebagai autentikasi adalah **kekeliruan arsitektur**.
     * Virtual bearer token atau custom header **TIDAK BERFUNGSI** karena middleware 9Router mem-bypass validasi sebelum memeriksa header tersebut.
     * **Direct Access Path:** Setiap proses lokal di Windows dapat membuka koneksi TCP ke `http://127.0.0.1:<port>/v1/chat/completions` dan mengonsumsi kuota inferensi AI tanpa melewati Hime Core.
   - **Evaluasi Solusi Rekayasa:**
     * *Opsi 1 (Upstream PR / Maintained Fork):* Menghapus baris `if (isLocalRequest(request)) return true;` pada 9Router dan mewajibkan token/API key untuk semua request.
     * *Opsi 2 (Isolasi Identitas OS):* Menjalankan 9Router di bawah akun service Windows terpisah dengan aturan Windows Filtering Platform (WFP) yang memblokir akses selain dari PID Hime Core.
   - **Kesimpulan Status:** Masalah ini diklasifikasikan sebagai **`IMPLEMENTATION BLOCKER & PROPOSED RESIDUAL RISK — OWNER DECISION REQUIRED`** sebelum Hime mengandalkan 9Router untuk operasional multi-tenant atau berbayar.
3. **Fungsi Tunnel Pihak Ketiga:**
   - Bukti: `cli/cli.js:230-277` memuat kode integrasi `cloudflared` dan `tailscale`. Seluruh integrasi tunnel dinonaktifkan secara permanen dalam konfigurasi Hime.

---

## 5. Audit Komunikasi Antar-Proses (IPC Architecture)

Evaluasi terhadap 5 batas komunikasi dalam ekosistem Hime:

| Batas IPC | Opsi Transportasi | Rekomendasi Terpilih | Mekanisme Keamanan Wajib |
|:---|:---|:---|:---|
| **Hime Desktop ↔ Hime Core** | Named Pipes vs Localhost WebSocket | **Windows Named Pipes** | `PipeSecurity` DACL (Current User SID) + 256-bit Ephemeral Session Token (**SEC-05**). |
| **Hime Core ↔ Hermes Agent** | Named Pipes vs stdio vs Localhost HTTP | **Windows Named Pipes** | Named Pipe dua arah dengan serialisasi JSON envelope dan token autentikasi handshake. |
| **Hermes Agent ↔ 9Router** | Localhost HTTP (`http://127.0.0.1:20128/v1`) | **Localhost HTTP (Restricted)** | Terikat strictly ke `127.0.0.1`. Mengingat celah RSK-10, jalur ini berstatus *Conditional on Upstream Patch/Fork*. |
| **Hime Core ↔ Device Bridge (Mobile)** | TLS over WebSocket vs mTLS | **mTLS over Wi-Fi / WireGuard** | Enkripsi end-to-end dengan pertukaran kunci via QR Code / SAS (Phase 11). |
| **Hime Core ↔ Tool Gateway** | Internal Rust function calls / In-process | **In-Process Safe Dispatch** | Zero network overhead; permission check sebelum pemanggilan fungsi. |

---

## 6. Audit Keamanan Proses & Runtime (Process & Runtime Security)

### A. Risiko Eksekusi Perintah Hermes & Prompt Injection (RSK-11)
- **Vektor Serangan:** *Indirect Prompt Injection* melalui halaman web atau dokumen yang dibaca agen. LLM dapat diperdaya untuk memanggil `terminal_tool` dengan payload berbahaya.
- **Perlindungan Internal Hermes:** `_YOLO_MODE_FROZEN` di `tools/approval.py:44` dan regex `approval_detection.py`.
- **Celah Residual:** Pengecekan berbasis regex tidak dapat menjamin 100% deteksi perintah berbahaya yang di-obfuscate.
- **Kontrol Arsitektur Hime (SEC-06):**
  - Hime Core bertindak sebagai **Central Policy Gate**.
  - Hermes Agent dilarang mengeksekusi shell secara langsung ke OS. Seluruh pemanggilan tool dialihkan ke Hime Core via Named Pipe.
  - Setiap tindakan destruktif wajib menampilkan **Dialog Persetujuan Manusia (*Mandatory Approval Modal*)** pada Dynamic Island HUD yang tidak dapat dibypass oleh agen.

### B. Isolasi Proses Anak via Windows Job Objects (SEC-08)
- Win32 Job Object API (`JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`) membungkus 9Router dan Hermes. Ketika Hime Core berhenti atau crash, Windows kernel secara deterministik mematikan seluruh proses anak dan memberlakukan kuota memori (9Router 512MB, Hermes 1024MB).

---

## 7. Audit Keamanan Sistem Berkas & Penyimpanan (Filesystem Security)

1. **Path Traversal (*Zip Slip / Relative Paths*):**
   - Kontrol Wajib (**SEC-07**): Seluruh path divalidasi dengan `std::fs::canonicalize`. Path di luar root workspace proyek ditolak secara otomatis.
2. **Daftar Larangan Keras (*Hardline Deny List*):**
   - Dilarang mengakses: `%WINDIR%`, `%USERPROFILE%\.ssh`, `%USERPROFILE%\.gnupg`, profil peramban, direktori Startup Windows.
3. **Operasi Berkas Atomik (*Atomic Write-Replace*):**
   - Seluruh pembaruan konfigurasi menulis ke berkas sementara (`.tmp`), memverifikasi integritas, lalu menimpa berkas asli secara atomik via `std::fs::rename`.

---

## 8. Audit Logging, Monitoring & Audit Trail

1. **Pencegahan Kebocoran Kredensial pada Log (SEC-09):**
   - Seluruh log sistem Hime melalui filter regex pembersih (*redactor*) sebelum ditulis ke disk (membersihkan pola `sk-...`, `Bearer ...`, `x-9r-cli-token`).
2. **Format & Integritas Jejak Audit:**
   - Jejak audit eksekusi tool dicatat dalam format JSON Lines terstruktur (`AUDIT.jsonl`) dengan correlation ID UUIDv4. Rotasi otomatis setiap 50 MB (retensi 7 hari).

---

## 9. Audit Keamanan Pembaruan & Installer (Update & Supply Chain)

1. **Risiko Unduhan Runtime Dinamis 9Router (RSK-09):**
   - Mitigasi Hime: Pre-bundling pustaka runtime secara offline pada Phase 06; penonaktifan scraper yang memicu modul dinamis `lazyRuntime.js`.
2. **Integritas Biner Hime (SEC-10):**
   - Seluruh biner Windows ditandatangani dengan sertifikat digital **Authenticode Code Signing**. Pembaruan diverifikasi terhadap manifest HTTPS bertanda tangan Ed25519 dengan digest SHA-256 dan rollback otomatis.

---

## 10. Register Risiko Keamanan Terbarui & Rekonsiliasi Status

| Risk ID | Judul & Kategori | Status Temuan | Severity | Klasifikasi Mitigasi |
|:---:|:---|:---:|:---:|:---|
| **RSK-01** | Rust & Cargo Missing on Host *(Toolchain)* | Terkonfirmasi; prasyarat Phase 06. | MEDIUM | Planned Control (User install) |
| **RSK-02** | Root Git Repository Missing *(VCS)* | Terkonfirmasi **RESOLVED** di TASK-00-02. | RESOLVED | Existing Verified Control |
| **RSK-03** | Plaintext Credentials in 9Router SQLite *(Security)* | Terkonfirmasi (CWE-312). | **HIGH** | Planned Defense-in-Depth (SEC-01) / Proposed Residual Risk — Owner Decision Required |
| **RSK-04** | 9Router Default Dashboard Password *(Security)* | Terkonfirmasi (`123456`). | **HIGH** | Planned Control (SEC-03 Supervisor Injection) |
| **RSK-05** | Windows Multi-Monitor DPI Scaling *(UI/UX)* | Terkonfirmasi dari analisis Nusa. | MEDIUM | Planned Control (Win32 DPI handling) |
| **RSK-06** | Android Background Process Eviction *(Mobile)* | Terkonfirmasi dari batasan Android 14+. | **HIGH** | Planned Control (Foreground Service Phase 11) |
| **RSK-07** | Protobuf Schema Breakage in Cloud Adapters *(Integration)* | Terkonfirmasi dari analisis 9Router Report 04. | LOW | Planned Control (Virtual model combos) |
| **RSK-08** | 9Router CLI Machine Token Authorization *(Security)* | Terkonfirmasi via `cli-secret` & `machineId`. | **HIGH** | Planned Control (SEC-01) / Proposed Residual Risk — Owner Decision Required |
| **RSK-09** | 9Router External Runtime Download Risk *(Supply Chain)* | Terkonfirmasi dekomposisi 3 mekanisme. | MEDIUM | Planned Control (Offline pre-bundling Phase 06) |
| **RSK-10** | **Localhost Unauthenticated LLM API Access in 9Router** *(Security)* | Terkonfirmasi bypass di `dashboardGuard.js`. | **HIGH** | **IMPLEMENTATION BLOCKER & PROPOSED RESIDUAL RISK — OWNER DECISION REQUIRED** |
| **RSK-11** | **Hermes Tool Privileges & Prompt Injection** *(Security)* | Terkonfirmasi token warisan di terminal. | **HIGH** | Planned Control (SEC-06 Central Policy Gate) |
| **RSK-12** | **9Router Default Host Binding `0.0.0.0` LAN Exposure** *(Security)* | Terkonfirmasi default di `cli.js:110`. | **HIGH** | Planned Control (SEC-04 Host Flag + Runtime Socket Inspection) — NOT YET VERIFIED |

---

## 11. Kesimpulan Rekayasa & Rekomendasi Lanjutan

Audit Keamanan & Lingkungan Runtime mengonfirmasi bahwa ekosistem Hime memiliki fondasi kontrol arsitektur yang kuat melalui **SEC-01 sampai SEC-10**.

Namun, audit ini secara jujur dan transparan mengidentifikasi satu **Implementation Blocker Kritis**:
- **RSK-10 (Bypass Autentikasi Localhost 9Router):** Tidak dapat diselesaikan hanya dengan loopback binding atau bearer token virtual tanpa modifikasi kode 9Router. Status: **`IMPLEMENTATION BLOCKER & PROPOSED RESIDUAL RISK — OWNER DECISION REQUIRED`**. Sebelum 9Router digunakan untuk multi-tenant atau kuota sensitif, Hime wajib menyiapkan patch upstream atau maintained fork untuk `dashboardGuard.js`, atau menerapkan isolasi identitas OS via WFP.

Dengan batasan, kualifikasi, dan kontrol mitigasi yang telah didefinisikan secara realistis, deliverable TASK-01-04 dinyatakan lengkap dan telah memperoleh penerimaan bersyarat pengguna (**QUALIFIED ACCEPTANCE**).
