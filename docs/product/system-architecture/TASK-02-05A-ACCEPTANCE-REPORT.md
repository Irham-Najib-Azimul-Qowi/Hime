# Architecture Acceptance Package: TASK-02-05A

- **Document ID:** TASK-02-05A-ACCEPTANCE-REPORT
- **Task ID:** TASK-02-05A (Architecture Acceptance Finalization & Implementation Constraints)
- **Parent Task:** TASK-02-05 (Hime System Architecture & Technical Design)
- **Phase:** Phase 02 — Nusa Reverse Engineering & Architecture Discovery
- **Status:** **QUALIFIED ACCEPTANCE — PENDING OWNER APPROVAL**
- **Date:** 2026-11-08
- **Authority:** Principal Software Architect, Senior Security Architect, Engineering Governance Reviewer

---

## 1. Executive Summary

Laporan ini menyajikan paket penerimaan arsitektur final (**Architecture Acceptance Package**) untuk **TASK-02-05 (Hime System Architecture & Technical Design)**.

Berdasarkan evaluasi menyeluruh terhadap 16 dokumen arsitektur cetak biru (DEL-02-46 s/d DEL-02-61), Project Owner merekomendasikan status **QUALIFIED ACCEPTANCE** dengan syarat enam catatan teknis kritis diformalkan ke dalam batasan implementasi, pemblokir implementasi, keputusan terbuka, dan matriks verifikasi terukur.

TASK-02-05A menyelesaikan seluruh formalisasi tersebut tanpa merancang ulang arsitektur Hime, tanpa memodifikasi kode sumber implementasi, tanpa membuat git commit/push sepihak, dan tanpa memodifikasi dependensi pihak ketiga (Hermes Agent & 9Router).

Hasil evaluasi menetapkan:
- **5 ADR Dipertahankan Penuh Tanpa Syarat Tambahan:** ADR-001 (Topologi 4-Proses), ADR-002 (Modular Monolith Rust), ADR-006 (Zero Upstream Modification), ADR-007 (Single-Writer WAL SQLite), dan ADR-008 (5-Dimensi Integritas Sistem Berkas).
- **7 ADR Diberikan Kualifikasi Teknis & Batasan Implementasi:** ADR-003 & ADR-009 (IPC Framing & Replay Guard), ADR-004 (9Router Localhost Security), ADR-005 (Hermes Tool Boundary PEP/PDP), ADR-010 (Cross-Platform Windowing Adapter), ADR-011 (Resource Budget & 1-Task Concurrency), dan ADR-012 (9Router Credential Protection).
- **2 Open Implementation Blockers Resmi:** IB-01 (Kredensial 9Router Plaintext SQLite CWE-312) dan IB-02 (Localhost Auth Bypass 9Router). Keduanya memblokir penggunaan API berbayar/sensitif sebelum lapisan proteksi terverifikasi dibangun.
- **Rekomendasi Status:** **`QUALIFIED ACCEPTANCE — PENDING OWNER APPROVAL`**.

---

## 2. Scope Review & Audit Kepatuhan Dokumen

| Aspek Peninjauan | Evaluasi Kepatuhan | Bukti & Referensi |
|:---|:---:|:---|
| **Cakupan Dokumen TASK-02-05** | 16 Dokumen Lengkap (100%) | DEL-02-46 s/d DEL-02-61 di direktori `project-management/` |
| **Ketertelusuran Kebutuhan (Traceability)** | 33 FRs & 18 NFRs Terlacak Penuh | Matriks lengkap pada `HIME-TECHNICAL-IMPLEMENTATION-ROADMAP.md` |
| **Integritas Format File** | 100% Lolos Validasi | 0 CRLF (strictly LF), 0 trailing whitespace, UTF-8 valid, single EOF newline |
| **Batasan Rekayasa Mutlak** | 100% Dipatuhi | Zero Code, Zero Commit, Zero Push, Zero Upstream Mod, Zero Task-02-06 Start |

---

## 3. Resolusi Enam Catatan Teknis Wajib

### A. Catatan A: ADR-012 — 9Router Credential Protection

1. **Evaluasi Mekanisme Kompatibel dengan Runtime 9Router:**
   - 9Router upstream (`decolua/9router`) membaca dan menulis konfigurasi provider AI beserta API key langsung ke file `data.sqlite` via driver SQLite Node.js secara native (`[VERIFIED STATIC]`).
   - Upstream 9Router TIDAK memiliki plugin interface, hook eksternal, atau integrasi native ke Windows Credential Manager / DPAPI maupun Linux Secret Service.
2. **Diferensiasi Perlindungan:**
   - *At-Rest (Saat Hime Mati):* Hime Core dapat menerapkan enkripsi simetris (AES-256-GCM) dengan master key yang dilindungi DPAPI (`CryptProtectData`) pada Windows atau libsecret pada Linux sebelum Hime berhenti. File `data.sqlite` terenkripsi aman saat di disk.
   - *Saat Inisialisasi/Startup:* Hime Core mendekripsi file ke lokasi runtime yang dilindungi hak akses.
   - *In-Use (Saat 9Router Berjalan):* Karena 9Router membutuhkan file SQLite yang valid untuk query SQL, file tersebut berada dalam keadaan plaintext selama 9Router hidup. Memori proses Node.js juga memuat API key secara plaintext.
   - *Access Control (ACL):* Direktori `.9router/` WAJIB diberi proteksi NTFS DACL (Windows: hanya Current User SID, hapus inheritance, blokir Network/System shares) dan POSIX mode `0700` (Linux: `rwx------`).
3. **Evaluasi Alternatif:**
   - *Credential Migration ke Secret Store:* Ditolak untuk MVP karena memerlukan refactoring mendalam pada internal 9Router yang melanggar ADR-006 (Zero Upstream Modification).
   - *Controlled Decrypt-on-Use:* Diterima sebagai mitigasi at-rest pembungkus (*wrapper mitigation*), namun diakui memiliki celah saat proses aktif.
   - *ACL & Process Isolation:* Diterima sebagai mitigasi wajib minimum.
   - *Upstream Security Improvement:* Diusulkan sebagai rencana kontribusi jangka panjang ke repositori upstream `decolua/9router`.
4. **Status & Klasifikasi:**
   - Untuk penggunaan API keys komersial/berbayar milik pengguna, status diklasifikasikan sebagai **`OPEN IMPLEMENTATION BLOCKER` (IB-01)** sampai mekanisme proxy atau credential injection in-memory terverifikasi.
   - Untuk pengujian lokal / provider tanpa biaya (mock/free tier), proteksi ACL `0700` berstatus `PLANNED CONTROL` dengan `RESIDUAL RISK` keterpaparan pada memori same-user context.

---

### B. Catatan B: ADR-005 — Hermes Tool Execution Boundary

1. **Dokumentasi Kategori Tool Hermes:**
   - *Kategori 1 — System Execution Tools:* `bash`, `terminal`, `command_runner` (risiko eksekusi arbitrary shell command).
   - *Kategori 2 — Filesystem Tools:* `file_editor`, `read_file`, `write_file`, `patch_file` (risiko path traversal & file loss).
   - *Kategori 3 — Inspection & Browser Tools:* `view_image`, `web_search`, `fetch_web` (risiko SSRF & exfiltration).
   - *Kategori 4 — Internal Runtime Operations:* SQLite session history logging, internal tokenizer, context cache.
2. **Mekanisme Intersepsi & Adapter:**
   - Tool pada Kategori 1 s/d 3 dinonaktifkan dari registri native Hermes atau digantikan oleh thin adapter (*IPC Client Shim*).
   - Saat model menginisiasi pemanggilan tool (`tool_call.request`), adapter mengemas parameter dan mengirimkannya melalui IPC `\\.\pipe\hime-hermes` ke Hime Core.
   - Hime Core bertindak sebagai satu-satunya *Policy Decision Point (PDP)* & *Policy Enforcement Point (PEP)*. Jika tool tergolong risiko tinggi (misal shell execution atau write di luar workspace), Hime Core memicu *Action Approval Modal* dengan jeda ergonomi anti-slip 500 ms (CST-TOOL-03).
3. **Pemisahan Runtime Internal vs Model Tool:**
   - Operasi Kategori 4 (internal runtime) diizinkan berjalan di dalam subproses Python, namun dibatasi oleh OS Job Object / sandbox direktori lokal (`$LOCALAPPDATA/hime/hermes_cache`), tidak memiliki hak akses tulis ke direktori pengguna lain.
4. **Mitigasi Jalur Bypass:**
   - Dilarang mendaftarkan tool yang mengeksekusi `eval()` atau `exec()` Python dinamis.
   - Lingkungan virtualenv Python diproteksi dengan melarang import pustaka berbahaya di luar paket resmi Hermes.
5. **Status Klasifikasi:**
   - Kontrak & Arsitektur Intersepsi: `DESIGN SPECIFIED`.
   - Implementasi Shim & Adapter IPC: `IMPLEMENTATION REQUIRED` (Phase 08 & Phase 09).
   - Uji Penetrasi Jalur Bypass: `NOT VERIFIED` (memerlukan pengujian hostile suite pada Phase 09).

---

### C. Catatan C: ADR-004 — 9Router Localhost Security

1. **Pertahanan Localhost Bypass sebagai Implementation Blocker:**
   - Temuan localhost authentication bypass dan ketiadaan autentikasi kuat pada endpoint native 9Router dipertahankan secara tegas sebagai **`IMPLEMENTATION BLOCKER` (IB-02)** untuk lingkungan produksi atau penggunaan model berbayar.
2. **Kebutuhan Autentikasi Endpoint Sensitif:**
   - Endpoint `/v1/chat/completions` (eksekusi model berbayar): Wajib memverifikasi bearer token per-sesi.
   - Endpoint `/api/config/*` & `/api/providers/*` (manajemen kredensial dan provider): Wajib ditolak dari akses publik localhost dan hanya dapat diakses melalui socket terotentikasi Hime Core.
3. **Verifikasi Runtime Berlapis (Fail-Closed):**
   - *IP Bind Address Verification:* Hime Core memverifikasi bahwa parameter peluncuran 9Router memuat `--host 127.0.0.1` dan binding hanya terjadi pada loopback IPv4.
   - *Socket Owner PID Check:* Setelah 9Router aktif, Hime Core memanggil API OS (`GetExtendedTcpTable` pada Windows / `/proc/net/tcp` pada Linux) untuk memeriksa apakah socket listener pada port `20128` benar-benar dimiliki oleh Child PID 9Router yang diluncurkan oleh Hime Core.
   - *Fail-Closed Action:* Jika port `20128` terdeteksi telah diduduki oleh proses asing (*port squatting*) atau PID tidak cocok, Hime Core seketika mematikan subproses, mencatat security alert, dan membatalkan inisialisasi aplikasi.
4. **Evaluasi Alternatif Arsitektural:**
   - Evaluasi menunjukkan bahwa pendekatan paling kokoh tanpa melanggar ADR-006 adalah menjadikan Hime Core sebagai *Loopback Authenticating Reverse Proxy* lokal (OD-01).

---

### D. Catatan D: ADR-010 — Dynamic Island Cross-Platform Windowing

1. **Matriks Kapabilitas Windowing Platform:**
   | Platform / Compositor | Top-Center Borderless Overlay | Direct Hardware Acceleration | Arbitrary Stacking (Topmost) | Click-Through Precision (`HTTRANSPARENT`) | Status Kesesuaian Dynamic Island |
   |:---|:---:|:---:|:---:|:---:|:---|
   | **Windows 10 / 11** | Penuh | Penuh (DirectComposition) | Penuh (`WS_EX_TOPMOST`) | Penuh (Win32 hit-test hook) | **Native Dynamic Island Penuh (MVP Tier 1)** |
   | **Linux X11** | Penuh | Penuh (OpenGL/EGL) | Penuh (`_NET_WM_STATE_STAYS_ON_TOP`) | Penuh (XShape / Input Shape) | **Native Island Penuh (Tier 2)** |
   | **Linux Wayland (wlroots)** | Penuh | Penuh (Wayland EGL) | Penuh (`zwlr_layer_shell_v1`) | Parsial (Layer shell input region) | **Native Island (Tier 2)** |
   | **Linux Wayland (GNOME/Mutter)** | Terbatas / Dilarang | Penuh (Wayland EGL) | Ditolak oleh sandbox compositor | Terbatas pada batas jendela | **Floating Companion Fallback (Tier 3)** |

2. **Keterbatasan Wayland & Fallback Behavior:**
   - Protokol standar Wayland (`xdg_shell`) melarang klien mengatur koordinat absolut dan melarang jendela berada di atas lockscreen/fullscreen app demi alasan keamanan display server.
   - *Fallback Behavior:* Pada GNOME Wayland di mana `wlr-layer-shell` tidak didukung, Hime Desktop otomatis beralih ke mode **Floating Companion Window** (jendela utilitas mengambang dengan Client-Side Decorations yang dapat dipindahkan pengguna) atau **System Tray Anchor Drawer**, tanpa mengalami crash.
3. **Pemisahan Fitur Wajib MVP vs Fitur Spesifik Platform:**
   - *Wajib MVP (Windows):* Transparent notch dynamic island, DirectComposition acceleration, fluid spring transition 60 FPS, global shortcut `Alt+Space`, non-activating focus protection (`WS_EX_NOACTIVATE`).
   - *Fitur Spesifik Linux (Phase 11):* Dukungan `wlr-layer-shell` untuk lingkungan wlroots (Hyprland/Sway) dan adapter fallback otomatis untuk GNOME Mutter.

---

### E. Catatan E: ADR-011 — Resource Budget Validation

1. **Konkurensi Agen:** Dibatasi mutlak tepat **1 Active Agent Task** pada satu waktu untuk MVP.
2. **Kompilasi Anggaran Sumber Daya:**
   - Seluruh nilai RAM ditetapkan sebagai **`PROPOSED TARGET`** yang wajib diuji melalui benchmark empiris:
   - *Stack Idle Target:* $\le 215\text{ MB}$ (Warning: $> 750\text{ MB}$, Critical: $> 920\text{ MB}$).
   - *Stack Normal Active Target:* $\le 470\text{ MB}$.
   - *Stack Peak Target:* $\le 690\text{ MB}$.
3. **Mekanisme Backpressure:**
   - Hime Core runtime supervisor memantau memori secara periodik (setiap 2 detik).
   - *Ambang Peringatan ($> 750\text{ MB}$):* Throttle background indexing & turunkan FPS animasi.
   - *Ambang Kritis ($> 920\text{ MB}$):* Tolak tugas baru, trigger GC subproses, paksa SQLite WAL checkpoint.
4. **Metodologi Benchmark Empiris (8 Skenario Uji):**
   - BM-01: Baseline Idle (2 jam pengukuran kebocoran memori).
   - BM-02: Single-Turn Q&A (latensi TTFT dan lonjakan RAM transien).
   - BM-03: Heavy Tool Execution (eksekusi terminal dan manipulasi file berganda).
   - BM-04: Large Document Ingestion (indeksasi berkas teks/PDF $\ge 50\text{ MB}$ ke FTS5).
   - BM-05: Headless Browser Automation (Playwright rendering overhead).
   - BM-06: Hostile Prompt Burst (pengiriman 20 request beruntun untuk menguji antrean FIFO).
   - BM-07: Subprocess Crash & Recovery (kill mendadak proses Hermes/9Router dan verifikasi restart).
   - BM-08: Battery-Saver Throttling (pengurangan konsumsi CPU saat laptop beralih ke daya baterai).

---

### F. Catatan F: ADR-003 & ADR-009 — IPC Security & Reliability

1. **Format Biner Framing `HIME`:**
   - Header 12-byte: Magic `0x48494D45`, Version `0x01`, MsgType (Request `0x01`, Response `0x02`, Notification `0x03`, StreamChunk `0x04`, StreamEnd `0x05`, Cancel `0x06`), Reserved `0x0000`, Payload Length `u32` (Big-Endian).
   - *Maximum Payload:* $16\text{ MB}$ ($16,777,216$ bytes).
   - *Pre-allocation Check:* Validasi panjang payload sebelum alokasi heap. Jika $> 16\text{ MB}$, tolak seketika, putus koneksi, dan catat security alert.
   - *Malformed Frame:* Jika magic header tidak cocok, flush buffer stream dan putus koneksi (*Fail-Closed*).
2. **Kepatuhan JSON-RPC 2.0:**
   - Payload diwajibkan mematuhi spesifikasi standar JSON-RPC 2.0 (`jsonrpc: "2.0"`, `method`, `params`, `id`, `result`, `error`).
3. **Aturan Anti-Replay & Siklus Sesi:**
   - Token handshake 256-bit CSPRNG dibuat unik per-sesi peluncuran subproses.
   - Counter sequence nonce strictly monotonic ($N_{\text{msg}} = N_{\text{last}} + 1$). Nonce yang sama atau lebih kecil ditolak seketika (`REPLAY_ATTACK_DETECTED`).
   - Timestamp drift tolerance $\le \pm 5\text{ detik}$ ditegakkan sebagai filter tambahan terhadap desinkronisasi jam.
4. **Semantik Pembatalan & Timeout:**
   - Timeout standar: 30 detik.
   - Pengiriman frame pembatalan (`Cancel`) memicu penghentian paksa worker task dan pembersihan staging file. Dilarang meninggalkan subproses/tugas yatim (*Zero Orphan Processes*).
5. **Pemulihan Koneksi:**
   - Reconnect backoff dengan jitter: $100\text{ms}, 200\text{ms}, 400\text{ms} \dots \max 5\text{s}$, maks 5 percobaan. Handshake baru wajib dilakukan saat koneksi ulang; counter nonce direset ke 1 hanya dengan token sesi baru.

---

## 4. Register Batasan Implementasi (Implementation Constraints Summary)

Rincian lengkap tertuang dalam dokumen [`HIME-IMPLEMENTATION-CONSTRAINTS.md`](file:///x:/folder_ai/Hime/project-management/system-architecture/HIME-IMPLEMENTATION-CONSTRAINTS.md):
- **CST-SEC-01:** 9Router Plaintext SQLite API Key Storage (`IMPLEMENTATION BLOCKER` / `PLANNED CONTROL`)
- **CST-SEC-02:** Zero Upstream Modification Policy for 9Router (`MANDATORY`)
- **CST-SEC-03:** Residual Risk API Key In-Memory Exposure (`RESIDUAL RISK`)
- **CST-TOOL-01:** Centralized PEP/PDP Hermes Interception via IPC (`MANDATORY`)
- **CST-TOOL-02:** Separation of Internal Runtime vs Model Tool Invocations (`PLANNED CONTROL`)
- **CST-TOOL-03:** Action Approval Modal 500ms Anti-Slip Ergonomic Delay (`MANDATORY`)
- **CST-NET-01:** 9Router Localhost Bypass Mitigation & Socket PID Check (`IMPLEMENTATION BLOCKER` / `MANDATORY`)
- **CST-NET-02:** Loopback Authenticating Proxy Architecture (`OPEN DECISION` OD-01)
- **CST-WIN-01:** Cross-Platform Windowing Adapter & GNOME Wayland Fallback (`MANDATORY`)
- **CST-WIN-02:** Focus Stealing Prevention `WS_EX_NOACTIVATE` (`MANDATORY`)
- **CST-RES-01:** Single Active Task Concurrency Constraint for MVP (`MANDATORY`)
- **CST-RES-02:** Calibrated RAM Budget Per-Process & Total Stack (`PROPOSED TARGET`)
- **CST-RES-03:** 2-Tier Memory Backpressure & Throttling (`PLANNED CONTROL`)
- **CST-IPC-01:** Binary Framing `HIME` Header & 16MB Pre-Allocation Limit (`MANDATORY`)
- **CST-IPC-02:** Monotonic Sequence Nonces & Replay Guard (`MANDATORY`)
- **CST-DATA-01:** 5-Dimension Filesystem Integrity & Zero Source Loss (`MANDATORY`)

---

## 5. Evaluasi Status Architecture Decision Records (ADR)

### ADR yang Dipertahankan Penuh Tanpa Kualifikasi:
1. **ADR-001 (Isolated 4-Process Topology):** Dipertahankan penuh. Bounded context terbukti tepat membatasi fault domain.
2. **ADR-002 (Modular Monolith Rust Engine for Hime Core):** Dipertahankan penuh. Tokio async dan in-process traits optimal untuk performa.
3. **ADR-006 (Zero Upstream Modification Policy):** Dipertahankan penuh. Tidak ada modifikasi kode pada Hermes dan 9Router.
4. **ADR-007 (Single-Writer WAL SQLite + FTS5 Architecture):** Dipertahankan penuh. Konkurensi single-writer/multi-reader stabil.
5. **ADR-008 (Five-Dimension Atomic & Safe Filesystem Operations):** Dipertahankan penuh. Garansi Zero Source Loss terdefinisi kuat.

### ADR yang Diberikan Kualifikasi Teknis:
1. **ADR-003 & ADR-009 (IPC Transport & Security):** Dikualifikasikan dengan spesifikasi header biner 12-byte, pre-allocation check $\le 16\text{MB}$, dan monotonic sequence nonces (CST-IPC-01, CST-IPC-02).
2. **ADR-004 (9Router Localhost Security):** Dikualifikasikan dengan status Implementation Blocker (IB-02), verifikasi kepemilikan PID socket, dan keputusan terbuka Authenticaticating Reverse Proxy (OD-01, CST-NET-01).
3. **ADR-005 (Hermes Tool Execution Boundary):** Dikualifikasikan dengan kewajiban intersepsi total pemanggilan tool model dan delay 500 ms modal persetujuan (CST-TOOL-01, CST-TOOL-03).
4. **ADR-010 (Cross-Platform Windowing):** Dikualifikasikan dengan prinsip No Forced Parity dan mekanisme fallback pada GNOME Wayland (CST-WIN-01).
5. **ADR-011 (Resource Budget & Concurrency):** Dikualifikasikan dengan label `PROPOSED TARGET` untuk seluruh angka RAM dan penetapan batas 1 active agent task pada MVP (CST-RES-01, CST-RES-02).
6. **ADR-012 (9Router Credential Protection):** Dikualifikasikan dengan status Implementation Blocker (IB-01) untuk API key berbayar dan proteksi DACL/POSIX `0700` pada disk (CST-SEC-01).

---

## 6. Open Implementation Blockers Resmi

| Blocker ID | Deskripsi Masalah | Komponen Terdampak | Syarat Pembukaan Blocker (Unblocking Criteria) | Target Fase Penyelesaian |
|:---:|:---|:---|:---|:---:|
| **IB-01** | Kredensial 9Router tersimpan plaintext dalam SQLite (`data.sqlite`, CWE-312). | `hime_core::security`, 9Router | Desain & implementasi mekanisme credential injection in-memory terverifikasi ATAU proxy credentials Hime Core yang mencegah penulisan API key komersial ke disk. | Phase 06 / Phase 08 (Security Gate SG-01) |
| **IB-02** | Localhost authentication bypass pada 9Router loopback port. | `hime_core::process_supervisor`, 9Router | Implementasi Loopback Authenticating Reverse Proxy di Hime Core dengan verifikasi token per-request ATAU patch resmi upstream 9Router yang disahkan. | Phase 08 (Security Gate SG-03) |

---

## 7. Open Decisions Resmi

| Decision ID | Deskripsi Keputusan Arsitektur | Alternatif Solusi yang Dievaluasi | Rekomendasi Arsitek | Target Resolusi |
|:---:|:---|:---|:---|:---:|
| **OD-01** | Arsitektur Perlindungan Loopback 9Router | **Opsi A:** Authenticating Reverse Proxy lokal di Hime Core.<br>**Opsi B:** Upstream patch loopback auth ke repositori 9Router. | **Opsi A (Recommended):** Menjaga kepatuhan 100% pada ADR-006 tanpa bergantung pada siklus rilis upstream. | Sebelum Phase 08 Dimulai |
| **OD-02** | Stance UX Dynamic Island pada GNOME Wayland | **Opsi A:** Floating Utility Companion Window dengan CSD.<br>**Opsi B:** System Tray Anchor Drawer.<br>**Opsi C:** GNOME Shell Extension khusus. | **Opsi A (Recommended):** Paling portabel dan tidak memerlukan instalasi ekstensi pihak ketiga pada sistem host. | Phase 07 / Phase 11 |
| **OD-03** | Kebijakan Eviksi Memori Subproses Hermes | **Opsi A:** Restart subproses Python secara berkala setelah N tugas.<br>**Opsi B:** Trigger `gc.collect()` paksa via sinyal IPC saat ambang Warning tercapai. | **Opsi B (Recommended):** Menghemat overhead cold startup Python runtime. | Phase 08 / Phase 09 |

---

## 8. Residual Risks yang Diakui Resmi

| Risk ID | Deskripsi Risiko Residu | Justifikasi Ketidakmungkinan Mitigasi Total | Rencana Kontinjensi / Pengawasan |
|:---:|:---|:---|:---|
| **RR-01** | Jendela keterpaparan kredensial dalam memori proses Node.js 9Router selama aktif. | Batasan fundamental OS: proses yang berjalan di bawah User SID yang sama dapat membaca memori proses lain tanpa isolasi virtualisasi/container. | Menampilkan Security Notice pada UI Hime dan merekomendasikan penggunaan token API berizin minimal (*least privilege scopes*). |
| **RR-02** | Risiko port squatting pada loopback TCP sebelum Hime Core aktif. | Port TCP loopback bersifat global per-network namespace pada Windows/Linux tanpa root privileges. | Pemeriksaan PID pemilik socket via `GetExtendedTcpTable` saat inisialisasi; jika PID tidak cocok, Hime Core melakukan abort seketika (*Fail-Closed*). |
| **RR-03** | Inkonsistensi implementasi protokol Wayland antar distribusi Linux. | Fragmentasi ekosistem compositor Linux (Mutter vs KWin vs wlroots vs Mir). | Menerapkan arsitektur platform adapter bertingkat dengan fallback otomatis tanpa crash. |
| **RR-04** | Lonjakan konsumsi RAM transien saat manipulasi dokumen besar ($\ge 50\text{ MB}$). | Operasi parsing AST atau OCR membutuhkan buffer memori sesaat yang cukup besar. | Menerapkan stream chunking pada modul parser dokumen dan membatasi ukuran berkas maksimal yang diproses di memori. |

---

## 9. Matriks Pengujian & Verifikasi Arsitektural (Verification Matrix)

| Test ID | Domain Pengujian | Prosedur Pengujian | Kriteria Lolos (Pass Criteria) | Status Kesiapan |
|:---:|:---|:---|:---|:---:|
| **VT-SEC-01** | Audit At-Rest 9Router SQLite | Dump byte heksadesimal direktori `.9router` saat Hime tidak aktif. | 0 pola API key dalam teks terbaca pada disk; file terenkripsi DPAPI. | Phase 06 Test Ready |
| **VT-SEC-02** | Verifikasi Fail-Closed Port 9Router | Buka port `20128` dengan listener Python tiruan sebelum Hime Core start. | Hime Core mendeteksi PID mismatch, membatalkan startup, 0 data terkirim. | Phase 08 Test Ready |
| **VT-TOOL-01**| Intersepsi Shell Hermes Agent | Injeksi prompt jailbreak meminta eksekusi shell destruktif via model. | Perintah tertahan di PEP Hime Core; Action Approval Modal muncul dengan delay 500ms. | Phase 09 Test Ready |
| **VT-WIN-01** | Tembus-Klik Window Dynamic Island | Kirim klik mouse pada area transparan Dynamic Island di atas aplikasi background. | Event klik diteruskan ke jendela background tanpa mencuri fokus aktif (`HTTRANSPARENT`). | Phase 07 Test Ready |
| **VT-WIN-02** | Fallback GNOME Wayland | Jalankan shell desktop pada sesi GNOME Wayland tanpa `wlr-layer-shell`. | UI beralih ke Floating Companion Window tanpa memicu crash Mutter / abort. | Phase 11 Test Ready |
| **VT-IPC-01** | Frame Biner Payload Overflow | Kirim frame dengan header panjang payload $17\text{ MB}$ ke server IPC. | Frame ditolak sebelum alokasi heap; socket diputus seketika (`FRAME_TOO_LARGE`). | Phase 08 Test Ready |
| **VT-IPC-02** | Uji Serangan Replay Nonce | Kirim ulang pesan valid dengan sequence nonce yang sama atau lebih kecil. | Server menolak pesan seketika dengan kode error `REPLAY_ATTACK_DETECTED`. | Phase 08 Test Ready |
| **VT-RES-01** | Benchmark Beban Idle 2 Jam | Jalankan ekosistem Hime dalam keadaan idle selama 2 jam kontinu. | Rata-rata RAM total stack $\le 215\text{ MB}$; 0 kebocoran memori linier ($< 5\text{ MB}$ drift). | Phase 08 Benchmark Ready |
| **VT-DATA-01**| Zero Source Loss Lintas-Volume | Simulasikan disk full / kegagalan write saat pemindahan berkas lintas drive. | Berkas sumber tetap utuh 100% pada lokasi asal; staging dibersihkan. | Phase 10 Test Ready |

---

## 10. Rekomendasi Status Penerimaan

Berdasarkan seluruh hasil perumusan arsitektur teknis, mitigasi rekayasa terstruktur, formalisasi register batasan implementasi, serta penetapan pemblokir dan kriteria verifikasi yang teruji:

Tim Arsitektur merekomendasikan penerimaan resmi arsitektur sistem Hime dengan status:

### **`QUALIFIED ACCEPTANCE — PENDING OWNER APPROVAL`**

**Catatan Kualifikasi:**
1. Status penerimaan ini mengesahkan seluruh cetak biru arsitektur TASK-02-05 sebagai fondasi sah bagi fase implementasi berikutnya.
2. Pelaksanaan implementasi pada Phase 06 s/d Phase 15 WAJIB mematuhi seluruh batasan pada [`HIME-IMPLEMENTATION-CONSTRAINTS.md`](file:///x:/folder_ai/Hime/project-management/system-architecture/HIME-IMPLEMENTATION-CONSTRAINTS.md).
3. Dua pemblokir implementasi (**IB-01** dan **IB-02**) wajib diselesaikan sebelum fitur yang melibatkan API keys berbayar atau endpoint 9Router diekspos ke pengguna.
