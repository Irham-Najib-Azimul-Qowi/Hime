# Phase 04 Architectural Validation & Targeted Corrections Report

- **Document ID:** PHASE-04-ARCHITECTURAL-VALIDATION-REPORT
- **Task ID:** PHASE-04-ARCHITECTURAL-VALIDATION
- **Phase Covered:** Phase 04 — Engineering Planning & Implementation Specification
- **Date:** 2026-11-10
- **Authority:** System Architect & Technical Lead
- **Execution Role:** Senior Systems Engineer, Software Architect, and Technical Execution Agent
- **Status Akhir:** **READY FOR FINAL ARCHITECT REVIEW**
- **Prinsip Tata Kelola:** *EVIDENCE-FIRST, ZERO CODE, ZERO COMMIT, ZERO PUSH, ZERO AUTOMATIC SOFTWARE INSTALLATION, ZERO UPSTREAM MODIFICATION*

---

## 1. Executive Summary

Laporan ini mendokumentasikan hasil pelaksanaan **Validasi Arsitektural dan Koreksi Terarah (*Architectural Validation & Targeted Corrections*)** terhadap 8 dokumen spesifikasi implementasi perangkat lunak Phase 04 pada ekosistem Hime.

Pekerjaan ini dilakukan berdasarkan arahan langsung dari System Architect & Technical Lead untuk:
1. Menghilangkan asumsi teknis yang tidak realistis dan klaim absolut yang tidak dapat dibuktikan.
2. Mengkalibrasi spesifikasi teknis terhadap arsitektur nyata dari dependensi eksternal (9Router dan Hermes Agent).
3. Mengakui batasan fisik sistem operasi Windows 11 dan Linux dalam lingkungan non-administrator (*standard user privileges*).
4. Menegaskan status **IB-01** dan **IB-02** sebagai **OPEN IMPLEMENTATION BLOCKERS**.

Seluruh 8 dokumen spesifikasi implementasi modular, master index, dan laporan penyelesaian telah disinkronkan secara konsisten. Tidak ada implementasi kode sumber baru yang ditulis (`ZERO IMPLEMENTATION CODE`), tidak ada modifikasi pada kode sumber upstream 9Router maupun Hermes (`ZERO UPSTREAM MODIFICATION`), dan tidak ada commit atau push ke repositori Git (`ZERO COMMIT, ZERO PUSH`).

---

## 2. Temuan Validasi & Koreksi Spesifik (7 Area Inti)

### Area 1: Validasi IB-01 — 9Router Credential Security
- **Dokumen:** [`9ROUTER-CREDENTIAL-SECURITY-SPECIFICATION.md`](file:///x:/folder_ai/Hime/project-management/system-architecture/implementation/9ROUTER-CREDENTIAL-SECURITY-SPECIFICATION.md)
- **Temuan Validasi:**
  * 9Router menerima kredensial melalui Web UI atau API manajemen (`/api/connections` dan `/api/nodes`) dan menyimpannya secara **PLAINTEXT** pada kolom `data` tabel `providerConnections` dan `providerNodes` di dalam basis data SQLite (`9router.db` / `data.sqlite`).
  * Kredensial digunakan pada modul `open-sse/executors/base.js` dalam metode `buildHeaders(credentials, stream)`, yang memanggil `proxyAwareFetch` langsung ke URL cloud provider publik (`https://api.openai.com/...`).
  * Kredensial **TIDAK DAPAT** diganti pada tahap outbound tanpa modifikasi kode 9Router KECUALI trafik outbound dialihkan melalui proxy MITM HTTPS (yang mewajibkan instalasi sertifikat root CA ke dalam trust store Node.js) ATAU melalui konfigurasi Custom Node yang mengarahkan `baseUrl` ke antarmuka loopback lokal.
  * Dummy key (`"hime-managed-key"`) yang disimpan di 9Router akan menyebabkan provider cloud upstream mengembalikan error `401 Unauthorized` jika diteruskan langsung ke internet, serta berpotensi menggagalkan validasi pre-flight model saat registrasi.
- **Koreksi yang Dilakukan:**
  * Menolak klaim naif bahwa in-flight proxy dapat bekerja transparan tanpa mekanisme rerouting eksplisit.
  * Membandingkan 4 alternatif arsitektural: *In-memory injection* (ditolak: memori V8 terpisah, butuh debugger API berbahaya), *External Egress MITM Proxy* (kompleks: butuh sertifikat root CA lokal), *Hime-Owned Provider Adapter* (direkomendasikan: memanfaatkan interface Custom Node resmi 9Router), dan *Controlled Upstream Modification* (opsi cadangan: fork vendor resmi).
  * Menetapkan rekomendasi utama: Hime Core bertindak sebagai **Local Egress Gateway** (`http://127.0.0.1:<hime_port>/v1`). 9Router mendaftarkan Custom Node OpenAI-compatible yang mengarah ke gateway ini. Kunci komersial **0% menyentuh disk atau basis data 9Router**.
  * **Status:** **IB-01 TETAP BERSTATUS `OPEN IMPLEMENTATION BLOCKER`**.

---

### Area 2: Validasi IB-02 — Localhost Security Boundary
- **Dokumen:** [`9ROUTER-LOCALHOST-SECURITY-SPECIFICATION.md`](file:///x:/folder_ai/Hime/project-management/system-architecture/implementation/9ROUTER-LOCALHOST-SECURITY-SPECIFICATION.md)
- **Temuan Validasi:**
  * Upstream 9Router **TIDAK MEMILIKI MIDDLEWARE** untuk memvalidasi header rahasia kustom `X-Hime-Internal-Secret`. Mengklaim header ini sebagai pengaman localhost adalah asumsi keliru.
  * Port loopback `127.0.0.1` tetap dapat diakses langsung oleh proses lokal mana pun di bawah pengguna yang sama (*same-user context*), melewati (*bypassing*) reverse proxy Hime Core secara penuh.
  * Pemeriksaan PID via Win32 API `GetExtendedTcpTable` hanya dapat memverifikasi PID pemilik listener socket saat cold start (mendeteksi *port squatting*), tetapi **TIDAK DAPAT MEMERIKSA CLIENT PID** dari koneksi TCP inbound tanpa driver kernel.
  * Penggunaan port acak dinamis (*ephemeral*) hanya memberikan *security through obscurity*; proses lokal dapat memindai dan menemukan port `node.exe` dalam waktu $< 50\text{ms}$.
  * Pada Windows 11 non-admin, aplikasi tidak dapat mengonfigurasi aturan Windows Filtering Platform (WFP) atau Windows Firewall per-PID.
- **Koreksi yang Dilakukan:**
  * Mengeliminasi ketergantungan pada header fiktif `X-Hime-Internal-Secret`.
  * Mengalihkan strategi pengamanan ke **Lapisan Autentikasi Aplikasi Native 9Router**: Hime Core mengaktifkan `requireApiKey = true` di basis data 9Router dan menyuntikkan token master acak CSPRNG 256-bit ke tabel `apiKeys` yang hanya diketahui oleh Hime Core. 9Router secara native menolak request langsung tanpa token dengan respons `401 Unauthorized`.
  * Mendokumentasikan secara jujur risiko residual (akses berkas `9router.db` oleh malware dalam User SID yang sama).
  * **Status:** **IB-02 TETAP BERSTATUS `OPEN IMPLEMENTATION BLOCKER`**.

---

### Area 3: Validasi Process Supervision
- **Dokumen:** [`SUBPROCESS-LIFECYCLE-SPECIFICATION.md`](file:///x:/folder_ai/Hime/project-management/system-architecture/implementation/SUBPROCESS-LIFECYCLE-SPECIFICATION.md)
- **Temuan Validasi:**
  * Klaim absolut "Zero Orphan Processes" mengabaikan kondisi kegagalan tingkat OS (kernel panic, crash driver, pemutusan daya fisik, atau status I/O uninterruptible D-state).
  * Jika pemanggilan `AssignProcessToJobObject` gagal, proses anak yang diluncurkan dengan `CREATE_SUSPENDED` tidak boleh dilanjutkan (*must not be resumed*).
  * Subproses dapat melepaskan diri dari Job Object jika flag breakaway diaktifkan.
- **Koreksi yang Dilakukan:**
  * Mengualifikasikan klaim "Zero Orphan Processes" sebagai target operasional kernel sistem operasi di bawah kondisi normal, bukan jaminan matematis mutlak.
  * Menetapkan protokol kegagalan assignment **Fail-Closed**: Jika `AssignProcessToJobObject` gagal, Hime Core seketika memanggil `TerminateProcess(hProcess, 1)`, menutup seluruh handle, dan membatalkan startup tanpa pernah memanggil `ResumeThread`.
  * Menegaskan kebijakan anti-breakaway: `JOB_OBJECT_LIMIT_BREAKAWAY_OK` dan `JOB_OBJECT_LIMIT_SILENT_BREAKAWAY_OK` secara eksplisit **TIDAK DIAKTIFKAN**.
  * Mendokumentasikan perilaku hierarkis *Nested Job Objects* pada Windows 8/10/11 (anak job otomatis hancur saat induk ditutup).
  * Menetapkan manajemen handle berbasis RAII pada Rust untuk mencegah kebocoran handle `hProcess` dan `hThread`.
  * Mendefinisikan lapisan abstraksi pengawasan Linux menggunakan `libc::prctl(PR_SET_PDEATHSIG, SIGTERM)`, subreaper `PR_SET_CHILD_SUBREAPER`, dan `cgroup.kill` pada cgroups v2.

---

### Area 4: Validasi IPC Binary Protocol
- **Dokumen:** [`IPC-BINARY-PROTOCOL-SPECIFICATION.md`](file:///x:/folder_ai/Hime/project-management/system-architecture/implementation/IPC-BINARY-PROTOCOL-SPECIFICATION.md)
- **Temuan Validasi:**
  * Sequence counter monotonik tunggal ($N = N_{\text{last}} + 1$) pada saluran yang melayani request konkuren multiplexing dan token streaming menyebabkan race condition dan false positive replay detection.
  * State machine sesi belum merinci siklus hidup transisi dan token rotasi saat terjadi diskoneksi transien.
  * Alokasi buffer memori berpotensi dieksploitasi jika parser tidak memeriksa batas panjang payload sebelum memanggil fungsi alokasi heap.
- **Koreksi yang Dilakukan:**
  * Mengisolasi sequence nonces ke dalam lingkup terpisah (*Scoped Sequence Nonces*): monotonic nonces hanya berlaku untuk request klien; streaming chunks menggunakan penomoran urutan internal `chunk_seq` yang terikat pada request ID terkait (`id`).
  * Mendefinisikan state machine sesi autentikasi lengkap: `UNAUTHENTICATED`, `AUTHENTICATED`, `ACTIVE`, `RECONNECTING`, dan `TERMINATED`.
  * Menetapkan mekanisme tiket re-koneksi sekali pakai (*one-time reconnect ticket*) dengan batas waktu validitas 5 detik untuk penanganan diskoneksi soket transien yang aman.
  * Mewajibkan aturan pre-allocation guard mutlak: panjang payload divalidasi $\le 16\text{ MiB}$ sebelum alokasi heap `Vec<u8>::with_capacity()` dieksekusi.
  * Menetapkan pemisahan deterministik penanganan error: error framing biner (magic salah, panjang $> 16\text{MB}$) seketika memutus soket; error JSON-RPC mengembalikan frame respons error tanpa memutus transport.
  * Menyediakan **4 test vectors biner tingkat byte heksadesimal** (Valid Handshake, Valid StreamChunk, Bad Magic, dan Oversize Payload) sebagai panduan uji implementasi.

---

### Area 5: Validasi Filesystem Safety
- **Dokumen:** [`STORAGE-AND-FILESYSTEM-SAFETY-SPECIFICATION.md`](file:///x:/folder_ai/Hime/project-management/system-architecture/implementation/STORAGE-AND-FILESYSTEM-SAFETY-SPECIFICATION.md)
- **Temuan Validasi:**
  * Pemindahan berkas lintas drive/partisi diklaim sebagai transaksi atomik, padahal secara fisik pada sistem operasi merupakan operasi copy-then-delete non-atomik.
  * Kebijakan retensi 30 hari berpotensi disalahartikan sebagai penghapusan otomatis tanpa persetujuan pengguna.
  * Tokenizer standar FTS5 `unicode61` dengan `porter` stemmer gagal melakukan word-breaking pada teks non-whitespace (bahasa Jepang Kanji/Kana dan bahasa Mandarin).
  * Mode SQLite WAL tidak dapat beroperasi pada network shares (SMB/NFS) karena keterbatasan shared memory (`.shm`).
- **Koreksi yang Dilakukan:**
  * Mengualifikasikan pemindahan lintas volume secara jujur sebagai **Non-Atomic 7-Step Verified Sequence** dengan jaminan zero source loss (sumber tidak pernah dihapus sebelum berkas tujuan di-flush via `FlushFileBuffers` dan diverifikasi hash SHA-256).
  * Menegaskan bahwa retensi 30 hari adalah batas tinjauan (*review boundary*), **BUKAN PENGHAPUSAN OTOMATIS TANPA KEBIJAKAN**. Penghapusan permanen wajib mendapatkan konfirmasi pengguna.
  * Mewajibkan seluruh operasi destruktif (hapus, overwrite, bulk move $> 10$ file) melewati persetujuan interaktif Policy Engine (PEP/PDP).
  * Mengganti konfigurasi tokenizer FTS5 menjadi `tokenize = 'trigram'` untuk mendukung pencarian teks multibahasa (Indonesia, Inggris, Jepang/CJK) dan kode sumber secara akurat tanpa kamus eksternal.
  * Mendokumentasikan batasan sistem berkas SQLite WAL: melarang penggunaan network shares dan membatasi konkurensi tulis pada single-writer dedicated actor thread.

---

### Area 6: Validasi Secret Redaction & Observability
- **Dokumen:** [`OBSERVABILITY-AND-LOGGING-SPECIFICATION.md`](file:///x:/folder_ai/Hime/project-management/system-architecture/implementation/OBSERVABILITY-AND-LOGGING-SPECIFICATION.md)
- **Temuan Validasi:**
  * Mengklaim bahwa pembungkusan `Zeroize` memusnahkan seluruh salinan rahasia dari memori adalah tidak akurat secara teknis karena adanya proliferasi memori runtime.
- **Koreksi yang Dilakukan:**
  * Mendokumentasikan secara rinci keterbatasan nyata `Zeroize`: buffer soket TCP/TLS HTTP client, alokasi heap sementara pada serialisasi JSON (`serde_json`), compiler register spills dan stack frame temporer, string error dari upstream SDK, pembuatan dump crash minidump, dan Process Environment Block (`PEB`).
  * Menjelaskan mengapa pertahanan berlapis (*Defense-in-Depth*) wajib diterapkan: dekripsi JIT sedekat mungkin dengan socket, allowlist bidang log terstruktur yang ketat, sanitasi karakter kontrol sebelum serialisasi, tipe pembungkus `SecretString`, dan fallback regex multi-pola.

---

### Area 7: Validasi QA & Traceability
- **Dokumen:** [`QUALITY-ASSURANCE-AND-DEFINITION-OF-DONE.md`](file:///x:/folder_ai/Hime/project-management/system-architecture/implementation/QUALITY-ASSURANCE-AND-DEFINITION-OF-DONE.md)
- **Temuan Validasi:**
  * Matriks keterlacakan sebelumnya belum memiliki kolom lengkap untuk Requirement ID, Component Owner, Test ID, Test Type, Expected Result, dan Acceptance Criteria.
  * Status pengujian perlu ditegaskan secara eksplisit agar tidak ada kesalahpahaman bahwa test suite telah dieksekusi.
- **Koreksi yang Dilakukan:**
  * Menyusun ulang tabel Traceability Matrix dengan mencakup seluruh 6 bidang wajib secara presisi untuk kebutuhan fungsional (FR), non-fungsional (NFR), dan keamanan (SEC).
  * Menetapkan status eksplisit: **`0 TESTS EXECUTED (SPECIFICATION STAGE)`**. Seluruh kasus uji saat ini berstatus `PLANNED` atau `TEST BLOCKED` (untuk IB-01 dan IB-02).
  * Menegaskan larangan klaim sukses tanpa bukti eksekusi terminal nyata.

---

## 3. Matriks Keputusan Teknis yang Berubah & Dikalibrasi

| Komponen / Topik | Desain Awal (Sebelum Validasi) | Desain Terkalibrasi (Hasil Validasi) | Alasan Perubahan Teknis |
|:---|:---|:---|:---|
| **Kredensial 9Router (IB-01)** | In-Flight Proxy generic via `open-sse/` | Hime-Owned Provider Adapter via Custom Node Loopback | 9Router tidak memiliki outbound header hook; dummy key memicu 401 Unauthorized dari provider cloud. |
| **Localhost Auth (IB-02)** | Header internal `X-Hime-Internal-Secret` | Penyuntikan token CSPRNG ke tabel native `apiKeys` 9Router | 9Router tidak memvalidasi header custom; proteksi native 9Router adalah satu-satunya opsi non-admin yang efektif. |
| **Process Supervision** | Zero Orphan matematis mutlak | Kernel-enforced operational target + Fail-Closed assignment | Batasan kegagalan fatal OS; pencegahan resume thread jika Job Object assignment gagal. |
| **IPC Nonces & Replay** | Global strictly monotonic ($N = N_{\text{last}} + 1$) | Scoped request nonces + stream chunk correlators | Mencegah false positive replay attack pada concurrent RPC dan streaming token LLM. |
| **Filesystem Move** | Transaksi atomik lintas volume | Non-atomic 7-step staged copy-verify sequence | Sistem operasi tidak mendukung pemindahan atomik lintas filesystem/drive fisik. |
| **Pencarian Teks (FTS5)** | Tokenizer `unicode61` + `porter` stemmer | Tokenizer `trigram` | `unicode61` gagal memisahkan kata pada teks CJK (Jepang/Mandarin); trigram mendukung CJK dan kode. |
| **Pembersihan Memori** | Klaim total pemusnahan via `Zeroize` | Pengakuan proliferasi memori (HTTP/JSON/spills) | `Zeroize` hanya membersihkan buffer lokal; pertahanan berlapis mutlak diperlukan. |
| **Status Pengujian** | Test case dianggap siap | Status eksplisit: `0 TESTS EXECUTED (PLANNED)` | Kepatuhan pada aturan *Evidence-First* tata kelola proyek Hime. |

---

## 4. Keputusan Arsitektur Terbuka (*Open Architecture Decisions*)

Tiga keputusan arsitektural terbuka dicatat dan direkomendasikan untuk keputusan final Architect:

1. **OD-04-01: Arsitektur Egress Kredensial 9Router (IB-01)**
   - *Pilihan:* (A) Hime Core Egress Gateway Loopback Adapter vs (B) Controlled Upstream Fork 9Router.
   - *Rekomendasi:* **Opsi A** diutamakan karena mematuhi ADR-006 tanpa mengubah repositori upstream; Opsi B disiapkan sebagai cadangan jika ditemukan batasan protokol pada Custom Node 9Router.
2. **OD-04-02: Strategi Autentikasi Localhost 9Router (IB-02)**
   - *Pilihan:* (A) Pemberdayaan native API key 9Router vs (B) Driver kernel WFP Windows.
   - *Rekomendasi:* **Opsi A** satu-satunya opsi yang layak untuk aplikasi desktop non-administrator.
3. **OD-04-03: Format Serialisasi Konfigurasi Runtime**
   - *Pilihan:* (A) JSON murni vs (B) TOML.
   - *Rekomendasi:* **Opsi B (TOML)** karena ramah dibaca manusia dan mendukung komentar konfigurasi.

---

## 5. Status Pemblokir Implementasi (IB-01 & IB-02)

| Pemblokir Implementasi | Status Resmi | Justifikasi Teknis Bertahan Sebagai Blocker |
|:---|:---:|:---|
| **IB-01 (9Router Credential Security)** | **`OPEN IMPLEMENTATION BLOCKER`** | Solusi desain telah dirumuskan (Hime-Owned Provider Adapter), namun belum diimplementasikan dalam kode dan belum diverifikasi runtime. Penyimpanan plaintext pada `9router.db` tetap menjadi risiko nyata jika dummy key atau alur adapter belum teruji. |
| **IB-02 (9Router Localhost Auth Bypass)** | **`OPEN IMPLEMENTATION BLOCKER`** | Tidak ada mekanisme non-admin murni di Windows 11 yang dapat mencegah proses lokal lain di bawah User SID yang sama untuk mengakses socket loopback jika token master bocor atau file database dibaca oleh malware lokal. |

---

## 6. Hasil Pemeriksaan Konsistensi Format & Kualitas

Pemeriksaan konsistensi otomatis menggunakan script Python terhadap seluruh file markdown di direktori `project-management/system-architecture/implementation/` dan `project-management/phases/`:

```text
================================================================================
FORMAT & CONSISTENCY VERIFICATION AUDIT
================================================================================
[PASS] Line Endings: Seluruh file menggunakan Unix LF (0 CRLF detected).
[PASS] Trailing Whitespace: 0 baris dengan trailing whitespace.
[PASS] Encoding: 100% valid UTF-8 encoding.
[PASS] EOF: Tepat satu karakter newline pada akhir berkas.
[PASS] Cross-References: Seluruh tautan dokumen dan Task ID valid dan sinkron.
[PASS] Policy Compliance: 0 Implementation Code, 0 Automatic Installs, 0 Upstream Mods.
================================================================================
```

---

## 7. Status Git Repositori Terkini

Status Git aktual pada workspace `x:\folder_ai\Hime` (Branch: `main`):

```text
On branch main
Changes not staged for commit:
  (use "git add <file>..." to update what will be committed)
  (use "git restore <file>..." to discard changes in working directory)
	modified:   PROJECT_STATUS.md

Untracked files:
  (use "git add <file>..." to include in what will be committed)
	project-management/PHASE-02-03-CONSOLIDATED-CLOSURE-REPORT.md
	project-management/PHASE-02-ACCEPTANCE.md
	project-management/PHASE-03-ACCEPTANCE.md
	project-management/ROADMAP-RECONCILIATION-AND-EXECUTION-PLAN.md
	project-management/phases/PHASE-04-ARCHITECTURAL-VALIDATION-REPORT.md
	project-management/phases/PHASE-04-COMPLETION-REPORT.md
	project-management/phases/PHASE-04-SPECIFICATION.md
	project-management/phases/PHASE-05-SPECIFICATION.md
	project-management/phases/PHASE-06-SPECIFICATION.md
	project-management/product-requirements/
	project-management/reverse-engineering/
	project-management/system-architecture/
	project-management/tasks/TASK-02-01-nusa-discovery.md
	project-management/tasks/TASK-02-02-nusa-architecture.md
	project-management/tasks/TASK-02-02A-evidence-correction.md
	project-management/tasks/TASK-02-02B-FINAL-CORRECTION.md
	project-management/tasks/TASK-02-02B-architecture-calibration.md
	project-management/tasks/TASK-02-03-nusa-ux.md
	project-management/tasks/TASK-02-03B-UX-EVIDENCE-CALIBRATION.md
	project-management/tasks/TASK-02-04-product-requirements.md
	project-management/tasks/TASK-02-05-system-architecture.md
	project-management/tasks/TASK-02-05A-architecture-acceptance.md

no changes added to commit (use "git add" and/or "git commit -a")
```

- **Commit Terakhir (HEAD):** `cfce9da` (`docs(governance): correct phase 01 task acceptance statuses`).
- **Aturan Mutlak:** Sesuai instruksi System Architect, **TIDAK ADA COMMIT DAN TIDAK ADA PUSH** yang dilakukan pada langkah ini (`ZERO COMMIT, ZERO PUSH`).

---

## 8. Rekomendasi Acceptance & Kesimpulan

Putaran validasi arsitektural dan koreksi terarah ini telah berhasil mengoreksi kelemahan teknis, menghilangkan asumsi yang tidak berdasar, dan mematangkan spesifikasi implementasi Phase 04 menjadi dokumen rekayasa yang kokoh dan dapat dipertanggungjawabkan secara empiris.

### Rekomendasi Tim Rekayasa:
Direkomendasikan kepada System Architect & Technical Lead untuk memberikan pengesahan teknis atas spesifikasi Phase 04 yang telah terkalibrasi ini guna mempersiapkan dokumen tata kelola Phase 05 (Implementation Governance & Test Fixtures).

### Status Akhir Pekerjaan:
# **`READY FOR FINAL ARCHITECT REVIEW`**

*(Phase 04 belum berstatus ACCEPTED; Phase 05 dan Phase 06 belum dimulai).*
