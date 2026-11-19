# Hime Implementation Constraints Registry

- **Document ID:** HIME-IMPLEMENTATION-CONSTRAINTS
- **Task ID:** TASK-02-05A (Architecture Acceptance Finalization & Implementation Constraints)
- **Phase:** Phase 02 — Nusa Reverse Engineering & Architecture Discovery
- **Authority:** Principal Software Architect, Senior Security Architect, Engineering Governance Reviewer
- **Status:** **APPROVED SPECIFICATION — QUALIFIED ACCEPTANCE COMPLIANT**
- **Date:** 2026-11-08

---

## 1. Pendahuluan & Skema Klasifikasi Batasan

Dokumen ini merupakan register resmi seluruh batasan implementasi rekayasa (*engineering implementation constraints*) yang wajib dipatuhi oleh seluruh developer, agen AI, dan sub-sistem selama fase implementasi Hime (Phase 06 s/d Phase 15).

Setiap batasan diturunkan langsung dari evaluasi arsitektural TASK-02-05, catatan peninjauan teknis Project Owner pada TASK-02-05A, serta profil perangkat keras dan lingkungan eksternal nyata.

### Skema Klasifikasi Status:
- `MANDATORY`: Batasan arsitektural mutlak yang wajib dipenuhi sebelum implementasi modul terkait dinyatakan selesai (*Definition of Done*).
- `IMPLEMENTATION BLOCKER`: Masalah keamanan atau arsitektur yang menghalangi penggunaan fitur, modul, atau integrasi tertentu sebelum solusi terverifikasi tersedia.
- `PLANNED CONTROL`: Mekanisme kontrol keamanan, keandalan, atau pengawasan yang telah dirancang tetapi belum diimplementasikan pada kode sumber.
- `PROPOSED TARGET`: Nilai metrik performa atau ambang batas sumber daya yang merupakan target desain dan wajib divalidasi melalui pengujian benchmark aktual.
- `OPEN DECISION`: Keputusan desain atau arsitektur yang memiliki alternatif teknis dan memerlukan arahan/persetujuan Project Owner.
- `RESIDUAL RISK`: Risiko teknis yang tetap ada setelah seluruh mitigasi yang direncanakan diterapkan, disebabkan oleh batasan intrinsik runtime pihak ketiga atau OS.

---

## 2. Domain A: Perlindungan Kredensial & Basis Data 9Router (ADR-012, FR-SEC-02)

| Parameter | Spesifikasi Batasan |
|:---|:---|
| **Constraint ID** | **CST-SEC-01** |
| **Sumber** | ADR-012, Catatan Teknis A, RE/9ROUTER/04-AUTHENTICATION-AND-ACCESS-CONTROL.md |
| **Status** | `IMPLEMENTATION BLOCKER` (untuk kunci berbayar/sensitif) / `PLANNED CONTROL` |
| **Deskripsi** | 9Router menyimpan API keys provider dalam SQLite (`data.sqlite`) secara plaintext (CWE-312 `[VERIFIED STATIC]`). Hime dilarang mengizinkan penyimpanan API keys produksi/berbayar langsung ke dalam SQLite native 9Router sampai solusi *secure proxy* atau injeksi memori terverifikasi dibangun. |
| **Alasan** | DPAPI dan Linux Secret Service adalah kontrol Hime Core untuk secret yang dikelolanya sendiri. File `data.sqlite` yang didekripsi saat 9Router aktif dapat dibaca oleh proses lokal lain yang berjalan di bawah konteks user yang sama. |
| **Komponen Terdampak** | `hime_core::security::credentials`, `hime_core::process_supervisor::service_9router`, 9Router runtime |
| **Acceptance Criteria** | 1. Kunci API komersial/sensitif tidak pernah ditulis dalam bentuk plaintext ke file `data.sqlite` pada disk.<br>2. Hime Core menolak startup 9Router jika direktori konfigurasi tidak memiliki DACL Windows / POSIX mode `0700`.<br>3. Jika menggunakan file wrapper, file dienkripsi at-rest saat 9Router berhenti. |
| **Metode Verifikasi** | Audit pembacaan file disk `data.sqlite` secara heksadesimal; pengujian pemblokiran akses ACL dari user/proses lain. |
| **Dependency** | Phase 06 (Secret Vault bootstrap), Phase 08 (9Router integration) |

---

| Parameter | Spesifikasi Batasan |
|:---|:---|
| **Constraint ID** | **CST-SEC-02** |
| **Sumber** | ADR-006, ADR-012, AGENTS.md Bagian A & E |
| **Status** | `MANDATORY` |
| **Deskripsi** | Larangan mutlak melakukan modifikasi kode sumber upstream `decolua/9router` atau melakukan patch langsung pada file JavaScript/Node.js di runtime upstream untuk menerapkan enkripsi. |
| **Alasan** | 9Router adalah dependensi eksternal yang diaudit dan dikonfigurasi, bukan di-fork atau ditulis ulang. Modifikasi lokal mematahkan *updateability* dan integritas audit. |
| **Komponen Terdampak** | Seluruh repositori dan proses packaging Hime |
| **Acceptance Criteria** | Git status pada submodule/vendored 9Router selalu clean (0 modified lines). |
| **Metode Verifikasi** | `git status` dan checksum SHA-256 berkala terhadap source bundle 9Router. |
| **Dependency** | Phase 01 (Asset Baseline), Phase 08 (Integrasi Runtime) |

---

| Parameter | Spesifikasi Batasan |
|:---|:---|
| **Constraint ID** | **CST-SEC-03** |
| **Sumber** | ADR-012, Catatan Teknis A |
| **Status** | `RESIDUAL RISK` |
| **Deskripsi** | Risiko residu: Jendela keterpaparan (*exposure window*) kunci API dalam memori proses Node.js 9Router dan file temporer SQLite selama 9Router berjalan tetap ada terhadap malware lokal yang memiliki hak akses setara user (*same-user context*). |
| **Alasan** | Tanpa arsitektur isolasi container/sandbox OS tingkat lanjut (AppContainer/Sandbox), proses yang berjalan di bawah User SID yang sama memiliki kapabilitas membaca memori dan file milik user tersebut. |
| **Komponen Terdampak** | Ekosistem Hime pada host Windows dan Linux |
| **Acceptance Criteria** | Pengguna diberikan dokumentasi peringatan (*security notice*) pada UI pengaturan bahwa kunci API berbayar dikelola dengan proteksi hak akses pengguna lokal (*user-level isolation*). |
| **Metode Verifikasi** | Review dokumentasi panduan pengguna dan UI settings warning dialog. |
| **Dependency** | Phase 07 (Desktop Settings UI) |

---

## 3. Domain B: Batas Eksekusi Alat Hermes & Centralized PEP/PDP (ADR-005, FR-SEC-01)

| Parameter | Spesifikasi Batasan |
|:---|:---|
| **Constraint ID** | **CST-TOOL-01** |
| **Sumber** | ADR-005, Catatan Teknis B, FR-SEC-01, FR-DEV-01 |
| **Status** | `MANDATORY` |
| **Deskripsi** | Seluruh pemanggilan tool sistem (`tool_call.request`) yang diinisiasi oleh model reasoning Hermes Agent WAJIB diarahkan melalui transport IPC ke Policy Enforcement Point (PEP) di Hime Core. Hermes Agent dilarang mengeksekusi shell (`subprocess`, `os.system`), sistem berkas, atau jaringan OS secara mandiri tanpa otorisasi Hime Core. |
| **Alasan** | Mencegah AI model mengeksekusi perintah destruktif secara otonom tanpa verifikasi kebijakan keamanan, audit logging, dan konfirmasi interaktif pengguna. |
| **Komponen Terdampak** | `HermesToolRegistry`, `hime_core::policy_engine`, `hime_core::tool_gateway` |
| **Acceptance Criteria** | 1. Tool built-in Hermes (`bash`, `file_editor`, `terminal`) dinonaktifkan atau digantikan (*shimmed*) oleh IPC client shim.<br>2. 100% tool invocations tercatat di database audit Hime Core.<br>3. Pemanggilan langsung `Popen` dari dalam worker Hermes terdeteksi dan diblokir oleh OS sandboxing / process restriction. |
| **Metode Verifikasi** | Pengujian unit & integrasi injeksi prompt manipulatif (jailbreak prompt) yang meminta eksekusi `rmdir /s /q` atau `powershell Stop-Computer`; verifikasi bahwa perintah tertahan di modal konfirmasi Hime Core. |
| **Dependency** | Phase 08 (IPC Transport), Phase 09 (Policy Engine & Action Approval Modal) |

---

| Parameter | Spesifikasi Batasan |
|:---|:---|
| **Constraint ID** | **CST-TOOL-02** |
| **Sumber** | ADR-005, Catatan Teknis B |
| **Status** | `PLANNED CONTROL` |
| **Deskripsi** | Pemisahan tegas antara operasi internal subproses Hermes Agent (internal logging, sqlite session reasoning cache, token streaming) dengan operasi eksekusi alat yang diarahkan model (*model-directed tool invocations*). |
| **Alasan** | Operasi internal runtime Python memerlukan performa tinggi dan akses lokal tanpa harus melalui persetujuan pengguna, namun tidak boleh mengekspos kapabilitas eksekusi sistem ke model. |
| **Komponen Terdampak** | Hermes Agent Subprocess Wrapper, IPC Adapter |
| **Acceptance Criteria** | Operasi internal Python dibatasi hanya pada direktori kerja virtualenv dan direktori cache Hime (`$LOCALAPPDATA/hime/hermes_cache`), tidak memiliki akses tulis ke direktori pengguna lain. |
| **Metode Verifikasi** | Audit path berkas yang dibuka oleh Hermes subproses menggunakan Sysinternals Process Monitor (ProcMon). |
| **Dependency** | Phase 08 (AI Runtime Infrastructure) |

---

| Parameter | Spesifikasi Batasan |
|:---|:---|
| **Constraint ID** | **CST-TOOL-03** |
| **Sumber** | ADR-005, FR-SEC-01, Catatan Teknis B |
| **Status** | `MANDATORY` |
| **Deskripsi** | Modal persetujuan tindakan (*Action Approval Modal*) pada UI Hime Desktop wajib menerapkan jeda ergonomi anti-slip (*anti-slip ergonomic delay*) minimal 500 milidetik sebelum tombol persetujuan (*Approve*) menjadi interaktif/dapat diklik. |
| **Alasan** | Mencegah penekanan tombol secara tidak sengaja oleh pengguna saat sedang mengetik cepat di keyboard (*accidental keystroke / muscle memory bypass*). |
| **Komponen Terdampak** | `hime-desktop::components::ActionApprovalModal` |
| **Acceptance Criteria** | Tombol "Approve" dalam keadaan disabled dengan visual countdown/timer selama minimal 500 ms setelah modal muncul di layar; penekanan tombol Enter/Space selama masa jeda diabaikan. |
| **Metode Verifikasi** | Pengujian interaksi UI otomatis via Playwright/WebDriver dengan simulasi klik pada 100ms (harus gagal) dan 600ms (harus berhasil). |
| **Dependency** | Phase 07 (Desktop Presentation Shell), Phase 09 (Interactive Safety Gate) |

---

## 4. Domain C: Keamanan Localhost & Verifikasi Socket 9Router (ADR-004, FR-SEC-03)

| Parameter | Spesifikasi Batasan |
|:---|:---|
| **Constraint ID** | **CST-NET-01** |
| **Sumber** | ADR-004, Catatan Teknis C, RE/9ROUTER/04-AUTHENTICATION-AND-ACCESS-CONTROL.md |
| **Status** | `IMPLEMENTATION BLOCKER` (untuk eksposur endpoint tanpa autentikasi) / `MANDATORY` |
| **Deskripsi** | Keberadaan kerentanan localhost authentication bypass pada 9Router dipertahankan sebagai **Implementation Blocker (IB-02)**. 9Router dilarang diekspos langsung ke aplikasi lain tanpa autentikasi. Hime Core wajib memverifikasi binding eksklusif pada `127.0.0.1` dan menerapkan verifikasi kepemilikan socket (PID match). |
| **Alasan** | Binding ke `127.0.0.1` hanya memblokir paket jaringan eksternal, tetapi tidak membatasi proses lokal lain yang berjalan pada mesin pengguna untuk menyalahgunakan endpoint AI berbayar. |
| **Komponen Terdampak** | `hime_core::process_supervisor::service_9router`, Network Verification Worker |
| **Acceptance Criteria** | 1. 9Router dikonfigurasi dengan random high-entropy master token saat cold start.<br>2. Hime Core memverifikasi bahwa socket TCP `20128` dimiliki secara sah oleh child PID 9Router yang diluncurkan Hime Core via `GetExtendedTcpTable` (Windows) / `/proc/net/tcp` (Linux).<br>3. Jika port telah terokupasi oleh proses lain sebelum 9Router aktif (*port squatting*), Hime Core wajib melakukan abort (*Fail-Closed*). |
| **Metode Verifikasi** | Pengujian automated port collision: jalankan socket listener tiruan pada port 20128 sebelum Hime start; verifikasi bahwa Hime Core mendeteksi PID mismatch dan membatalkan inisialisasi dengan pesan error yang aman. |
| **Dependency** | Phase 06 (Process Supervisor), Phase 08 (9Router Integration) |

---

| Parameter | Spesifikasi Batasan |
|:---|:---|
| **Constraint ID** | **CST-NET-02** |
| **Sumber** | ADR-004, Catatan Teknis C |
| **Status** | `OPEN DECISION` (OD-01) |
| **Deskripsi** | Keputusan terbuka arsitektur: Apakah Hime Core akan bertindak sebagai *Authenticating Reverse Proxy* lokal di depan 9Router (menerima request Hermes pada Unix Domain Socket / Named Pipe terotentikasi, lalu mem-forward ke HTTP 9Router dengan secret header internal), ATAU menerapkan upstream patch jika upstream merilis mekanisme loopback auth yang aman. |
| **Alasan** | Menghindari modifikasi source code upstream 9Router (ADR-006) sekaligus menutup total risiko localhost cross-process attack surface. |
| **Komponen Terdampak** | `hime_core::gateway_proxy`, arsitektur jaringan lokal |
| **Acceptance Criteria** | Keputusan disetujui Project Owner sebelum Phase 08 dimulai. |
| **Metode Verifikasi** | Dokumentasi keputusan ADR tambahan jika arsitektur reverse proxy diadopsi. |
| **Dependency** | Phase 08 AI Runtime Infrastructure |

---

## 5. Domain D: Dynamic Island Cross-Platform Windowing (ADR-010, FR-CORE-01, FR-XPLAT-01/02)

| Parameter | Spesifikasi Batasan |
|:---|:---|
| **Constraint ID** | **CST-WIN-01** |
| **Sumber** | ADR-010, Catatan Teknis D, DIR-03, FR-CORE-01, FR-XPLAT-01/02 |
| **Status** | `MANDATORY` |
| **Deskripsi** | Dynamic Island HUD WAJIB menerapkan adapter windowing spesifik platform dan dilarang memaksakan paritas perilaku visual yang identik (*No Forced Parity*) jika compositor OS tidak mendukungnya. |
| **Alasan** | Windows DirectComposition/Win32 mendukung borderless transparent top-level overlay dengan tembus-klik presisi. Wayland pada Linux (terutama GNOME Mutter) melarang penentuan posisi jendela absolut, overlay global di atas lockscreen/fullscreen, dan click-through mask sewenang-wenang demi alasan keamanan sandbox display server. |
| **Komponen Terdampak** | `hime-desktop::window::WindowingAdapter`, Platform Adapters (`src/platform/windows/*`, `src/platform/linux/*`) |
| **Acceptance Criteria** | 1. **Windows 10/11:** Mendukung Top-Center Dynamic Island Overlay, DirectComposition hardware acceleration, tembus-klik `HTTRANSPARENT`.<br>2. **Linux X11:** Mendukung Dock type overlay (`_NET_WM_WINDOW_TYPE_DOCK`) dengan komposit transparansi.<br>3. **Linux Wayland (wlroots/Hyprland/Sway):** Mendukung overlay via protokol `zwlr_layer_shell_v1`.<br>4. **Linux Wayland (GNOME/Mutter):** Secara otomatis beralih (*fallback*) ke Floating Utility Companion Window atau System Tray Anchor tanpa mengalami crash display server. |
| **Metode Verifikasi** | Pengujian eksekusi jendela pada Windows 11 23H2, Ubuntu 24.04 (GNOME Wayland), dan Arch Linux (Hyprland Wayland). |
| **Dependency** | Phase 07 (Desktop Presentation Shell), Phase 11 (Linux Compatibility Tier) |

---

| Parameter | Spesifikasi Batasan |
|:---|:---|
| **Constraint ID** | **CST-WIN-02** |
| **Sumber** | ADR-010, FR-SYS-02 |
| **Status** | `MANDATORY` |
| **Deskripsi** | Window Dynamic Island dilarang mencuri fokus keyboard aktif pengguna saat melakukan animasi ekspansi, menampilkan notifikasi, atau memperbarui status (*Focus Stealing Prevention*). |
| **Alasan** | Menghindari gangguan alur kerja pengguna (interupsi mengetik pada aplikasi latar belakang seperti IDE, terminal, atau browser). |
| **Komponen Terdampak** | Win32 Window Procedure Subclassing, Event Dispatcher |
| **Acceptance Criteria** | Penerapan window styles `WS_EX_NOACTIVATE` dan handling pesan `WM_MOUSEACTIVATE` return `MA_NOACTIVATE` pada Windows; window flag `skip_taskbar` dan non-activating hint pada X11/Wayland. Pengujian mengetik di Notepad/VS Code tidak kehilangan kursor saat island beranimasi. |
| **Metode Verifikasi** | Automated focus retention test: skrip pengetikan kontinu di jendela foreground sementara event notifikasi Hime diinjeksi berulang kali; verifikasi karakter tidak ada yang terpotong. |
| **Dependency** | Phase 07 (Desktop Presentation Shell) |

---

## 6. Domain E: Alokasi Sumber Daya, Batas Konkurensi & Backpressure (ADR-011, NFR-RES-01..03, NFR-PERF-05)

| Parameter | Spesifikasi Batasan |
|:---|:---|
| **Constraint ID** | **CST-RES-01** |
| **Sumber** | ADR-011, Catatan Teknis E, NFR-PERF-05 |
| **Status** | `MANDATORY` |
| **Deskripsi** | Batasan konkurensi eksekusi agen pada MVP dibatasi tepat **1 Active Agent Task** pada satu waktu. Permintaan tugas baru yang masuk saat agen sedang aktif wajib dimasukkan ke dalam antrean (FIFO queue) atau ditolak dengan konfirmasi pengguna. |
| **Alasan** | Profil perangkat keras host pengembangan adalah prosesor hemat daya Intel Core i5-8250U (4C/8T) dengan sisa RAM bebas sekitar ~3.7 GB. Menjalankan multi-agent serentak berpotensi memicu thrashing memori dan degradasi responsivitas desktop. |
| **Komponen Terdampak** | `hime_core::agent_orchestrator`, Semaphore Concurrency Controller |
| **Acceptance Criteria** | Semaphore konkurensi bernilai 1. Percobaan menjalankan tugas kedua secara paralel menghasilkan status `QUEUED` atau modal dialog `Task Already in Progress`. |
| **Metode Verifikasi** | Unit test pengiriman dua perintah tugas simultan; verifikasi tugas kedua dieksekusi hanya setelah tugas pertama selesai atau dibatalkan. |
| **Dependency** | Phase 08 (Agent Orchestrator) |

---

| Parameter | Spesifikasi Batasan |
|:---|:---|
| **Constraint ID** | **CST-RES-02** |
| **Sumber** | ADR-011, Catatan Teknis E, NFR-RES-01, NFR-RES-02, NFR-RES-03 |
| **Status** | `PROPOSED TARGET` |
| **Deskripsi** | Target alokasi RAM per-proses dan total stack ekosistem Hime (seluruh nilai berstatus target desain yang wajib divalidasi secara empiris melalui pengujian benchmark): |

### Rincian Anggaran Memori Per-Proses & Total Ekosistem:
| Komponen Proses | Idle Target (`[PROPOSED]`) | Active Target (`[PROPOSED]`) | Peak Target (`[PROPOSED]`) | Warning Threshold | Critical Threshold |
|:---|:---:|:---:|:---:|:---:|:---:|
| **Hime Desktop (Tauri WebView2)** | $\le 85\text{ MB}$ | $\le 160\text{ MB}$ | $\le 220\text{ MB}$ | $> 240\text{ MB}$ | $> 280\text{ MB}$ |
| **Hime Core (Rust Engine)** | $\le 30\text{ MB}$ | $\le 70\text{ MB}$ | $\le 120\text{ MB}$ | $> 130\text{ MB}$ | $> 160\text{ MB}$ |
| **Hermes Agent (Python + uv)** | $\le 60\text{ MB}$ | $\le 160\text{ MB}$ | $\le 240\text{ MB}$ | $> 260\text{ MB}$ | $> 320\text{ MB}$ |
| **9Router (Node.js Service)** | $\le 40\text{ MB}$ | $\le 80\text{ MB}$ | $\le 110\text{ MB}$ | $> 120\text{ MB}$ | $> 160\text{ MB}$ |
| **TOTAL ECOSYSTEM STACK** | **$\le 215\text{ MB}$** | **$\le 470\text{ MB}$** | **$\le 690\text{ MB}$** | **$> 750\text{ MB}$** | **$> 920\text{ MB}$** |

- **Mekanisme Backpressure & Throttling (CST-RES-03 — `PLANNED CONTROL`):**
  1. *Warning Level ($> 750\text{ MB}$ Total):* Hime Core menangguhkan indeksasi FTS latar belakang, menurunkan refresh rate animasi UI dari 60 FPS ke 30 FPS, dan menunda garbage collection non-esensial.
  2. *Critical Level ($> 920\text{ MB}$ Total):* Hime Core menolak inisiasi tugas baru, memicu forced GC pada subproses Python dan Node.js, melakukan WAL checkpoint SQLite, dan memunculkan peringatan konsumsi memori pada dashboard Hime.
- **Metodologi Verifikasi:** Uji benchmark 8 skenario terstandardisasi pada Phase 08–10 (Idle, Single Q&A, Heavy Tool, Large FTS Ingestion, Browser Automation, Hostile Prompt Burst, Crash Recovery, Multi-hour Idle Leak).

---

## 7. Domain F: Protokol IPC, Binary Framing & Keandalan Transpor (ADR-003, ADR-009)

| Parameter | Spesifikasi Batasan |
|:---|:---|
| **Constraint ID** | **CST-IPC-01** |
| **Sumber** | ADR-003, ADR-009, Catatan Teknis F, FR-SEC-03 |
| **Status** | `MANDATORY` |
| **Deskripsi** | Kontrak biner framing transport IPC wajib mengikuti spesifikasi baku dan dilarang mengalokasikan memori sebelum validasi panjang payload: |

### Spesifikasi Biner Framing `HIME` (12-Byte Header):
```text
 0                   1                   2                   3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                  Magic: 'H' 'I' 'M' 'E' (0x48494D45)          |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|  Version(0x01)|MsgType(0x01..)|          Reserved (0x0000)    |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                   Payload Length (Big-Endian u32)             |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                   Payload Data (JSON-RPC 2.0 UTF-8)           |
|                               ...                             |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
```
- **Batasan Ukuran Maksimum Payload:** $16\text{ MB}$ ($16,777,216$ bytes).
- **Validasi Pre-Allocation:** Jika `Payload Length` $> 16\text{ MB}$, pembacaan ditolak seketika tanpa alokasi buffer heap, koneksi socket ditutup (*Fail-Closed*), dan event keamanan dicatat.
- **Malformed Frame Handling:** Jika 4 byte pertama $\ne$ `0x48494D45` atau `Version` $\ne 0x01$, transpor wajib membersihkan buffer, memutus koneksi seketika, dan tidak mencoba menebak data.
- **Dependency:** Phase 08 (IPC Transport Implementation).

---

| Parameter | Spesifikasi Batasan |
|:---|:---|
| **Constraint ID** | **CST-IPC-02** |
| **Sumber** | ADR-009, Catatan Teknis F |
| **Status** | `MANDATORY` |
| **Deskripsi** | Mekanisme Anti-Replay dan Keamanan Sesi Transpor: |

1. **Autentikasi Handshake:** Menggunakan token ephemeral CSPRNG 256-bit yang dibuat Hime Core saat meluncurkan subproses dan diinjeksikan via secure stdin/environment variable (tidak pernah ditulis ke disk).
2. **Monotonic Sequence Nonce:** Setiap pesan wajib menyertakan sequence integer strictly monotonic ($N_{\text{msg}} = N_{\text{last\_seen}} + 1$). Pesan dengan nonce lebih kecil atau sama dengan nonce sebelumnya DITOLAK SEKETIKA (`REPLAY_ATTACK_DETECTED`).
3. **Timestamp Drift Tolerance:** Timestamp pesan divalidasi terhadap waktu lokal host: $|T_{\text{msg}} - T_{\text{host}}| \le 5\text{ detik}$. Toleransi waktu ini hanya sebagai lapis filter tambahan, BUKAN satu-satunya proteksi replay.
4. **Semantik Timeout & Cancellation:** Timeout default request adalah $30\text{ detik}$. Frame pembatalan (`MsgType = 0x06 Cancel`) wajib mengirimkan sinyal interupsi ke worker subproses dan membatalkan transaksi penulisan staging yang belum di-commit. Dilarang meninggalkan subproses/tugas yatim (*No Orphan Execution*).
5. **Pemulihan Koneksi (Reconnect):** Reconnect menggunakan exponential backoff dengan jitter ($100\text{ms}, 200\text{ms}, 400\text{ms} \dots \max 5\text{s}$, maks 5 percobaan). Sesi baru memerlukan token handshake baru; counter nonce direset ke 1 hanya dengan token sesi baru.

---

## 8. Domain G: Integritas Sistem Berkas 5-Dimensi & Zero Source Loss (ADR-008, FR-FILE-04)

| Parameter | Spesifikasi Batasan |
|:---|:---|
| **Constraint ID** | **CST-DATA-01** |
| **Sumber** | ADR-008, FR-FILE-04, NFR-DATA-01, NFR-DATA-02 |
| **Status** | `MANDATORY` |
| **Deskripsi** | Operasi penulisan dan pemindahan berkas wajib menerapkan garansi 5-Dimensi Integritas Sistem Berkas: |

1. **Same-Volume Atomic Rename:** Pemindahan berkas pada partisi/volume yang sama wajib menggunakan syscall atomic OS (`MoveFileExW(MOVEFILE_REPLACE_EXISTING)` pada Windows, `renameat2(RENAME_NOREPLACE)` pada Linux).
2. **Cross-Volume 4-Step Lifecycle:** Pemindahan lintas partisi (misal dari C: ke D: atau Flashdisk) WAJIB mengikuti urutan: (1) Copy berkas ke direktori staging `.hime_staging/`, (2) Hitung dan cocokkan checksum SHA-256 berkas sumber vs berkas tujuan, (3) Lakukan flushing buffer disk (`FlushFileBuffers` / `fsync`), (4) Hapus berkas sumber HANYA JIKA verifikasi SHA-256 bernilai identik 100%. Jika ada galat pada tahap apa pun, berkas sumber TIDAK BOLEH DIHAPUS (*Zero Source File Loss*).
3. **Reversible Safety Trash:** Berkas yang dihapus oleh agen dipindahkan ke sistem penampungan temporer Hime (`$LOCALAPPDATA/hime/safety_trash/`) selama masa retensi 30 hari sebelum penghapusan permanen.
4. **Dependency:** Phase 10 (Storage Intelligence & Safe Move Engine).

---

## 9. Matriks Keterlacakan Batasan Implementasi ke Roadmap Fase

| Constraint ID | Domain Rekayasa | Status Klasifikasi | Fase Penegakan | Gate Terkait |
|:---|:---|:---:|:---:|:---:|
| **CST-SEC-01** | Kredensial 9Router | `IMPLEMENTATION BLOCKER` / `PLANNED CONTROL` | Phase 06 & Phase 08 | Security Gate SG-01 |
| **CST-SEC-02** | Zero Upstream Modification 9Router | `MANDATORY` | Phase 01 s/d Phase 15 | Governance Gate GG-01 |
| **CST-SEC-03** | Residual Risk Memory Exposure | `RESIDUAL RISK` | Phase 07 (UI Notice) | Security Notice SN-01 |
| **CST-TOOL-01**| Centralized PEP/PDP Hermes Interception | `MANDATORY` | Phase 08 & Phase 09 | Security Gate SG-02 |
| **CST-TOOL-02**| Isolasi Runtime Internal vs Model Tool | `PLANNED CONTROL` | Phase 08 | Verification Gate VG-01 |
| **CST-TOOL-03**| Action Approval Modal 500ms Delay | `MANDATORY` | Phase 07 & Phase 09 | UX Ergonomics Gate UG-01 |
| **CST-NET-01** | 9Router Localhost Bypass & PID Check | `IMPLEMENTATION BLOCKER` / `MANDATORY` | Phase 06 & Phase 08 | Security Gate SG-03 |
| **CST-NET-02** | Loopback Authenticating Proxy Decision | `OPEN DECISION` (OD-01) | Phase 08 Pre-start | Decision Gate DG-01 |
| **CST-WIN-01** | Windowing Adapter & Wayland Fallback | `MANDATORY` | Phase 07 & Phase 11 | Platform Gate PG-01 |
| **CST-WIN-02** | Focus Stealing Prevention (No Stealing) | `MANDATORY` | Phase 07 | UX Gate UG-02 |
| **CST-RES-01** | Single Active Task Concurrency MVP | `MANDATORY` | Phase 08 | Resource Gate RG-01 |
| **CST-RES-02** | Anggaran RAM Per-Proses & Total Stack | `PROPOSED TARGET` | Phase 08 s/d Phase 10 | Benchmark Gate BG-01 |
| **CST-RES-03** | Throttling & Backpressure 2-Tier | `PLANNED CONTROL` | Phase 09 | Resource Gate RG-02 |
| **CST-IPC-01** | Biner Framing `HIME` & Max 16MB Limit | `MANDATORY` | Phase 08 | Verification Gate VG-02 |
| **CST-IPC-02** | Monotonic Nonce & Session Lifecycle | `MANDATORY` | Phase 08 | Security Gate SG-04 |
| **CST-DATA-01**| 5-Dimensi Integritas FS & Zero Loss | `MANDATORY` | Phase 10 | Data Integrity Gate DIG-01 |
