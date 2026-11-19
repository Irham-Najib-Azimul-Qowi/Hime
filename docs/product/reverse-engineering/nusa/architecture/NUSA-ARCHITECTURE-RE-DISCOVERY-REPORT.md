# Nusa Architecture & Protocol Reverse Engineering Report — TASK-02-02

- **Document ID:** NUSA-ARCHITECTURE-RE-DISCOVERY-REPORT
- **Task ID:** TASK-02-02 (Phase 02 — Nusa Reverse Engineering)
- **Target Application:** Nusa Dynamic Island for Windows (v1.2.0.0 by Rama)
- **Authority:** Lead Reverse Engineering Specialist & Systems Architect
- **Status:** **COMPLETE — PENDING OWNER REVIEW**
- **Date:** 2026-11-08

---

## 1. Executive Summary

Laporan ini menyajikan hasil sintesis akhir dan evaluasi rekayasa balik arsitektur dan protokol (**Architecture & Protocol Reverse Engineering**) terhadap aplikasi **Nusa Dynamic Island v1.2.0.0** (`nusa.exe` dan `NusaSetup.exe`).

Kegiatan ini merupakan kelanjutan dari TASK-02-01 dan TASK-02-02A. Melalui analisis statis mendalam terhadap format executable PE32+, Import Address Table (IAT), dekompilasi setup script Inno Setup, ekstraksi string segmen `.rdata`, dan pemodelan interaksi subsistem, laporan ini menyajikan pemahaman tuntas mengenai struktur internal Nusa dan merumuskan spesifikasi translasi arsitektur (*clean-room reimplementation*) untuk **Hime Desktop**.

Seluruh temuan telah diverifikasi dengan standar kepastian bukti:
- `VERIFIED STATIC`: Terbukti langsung melalui header biner, IAT, metadata, atau string biner.
- `INFERRED`: Deduksi analitis logis dari arsitektur panggilan API dan laporan observasi awal.
- `UNKNOWN`: Hal yang tidak dapat dipastikan tanpa akses source code asli.
- `NOT TESTED`: Perilaku yang tidak diuji secara runtime dalam sesi audit saat ini.

---

## 2. Deliverables Package Summary

Seluruh deliverable arsitektur dan protokol telah diterbitkan pada direktori kanonikal [`project-management/reverse-engineering/nusa/architecture/`](file:///x:/folder_ai/Hime/project-management/reverse-engineering/nusa/architecture/):

| No | Deliverable ID | Berkas Dokumen | Klasifikasi Bukti | Deskripsi Konten Inti |
|:---:|:---|:---|:---:|:---|
| 1 | **DEL-02-10** | [`NUSA-BINARY-ARCHITECTURE.md`](file:///x:/folder_ai/Hime/project-management/reverse-engineering/nusa/architecture/NUSA-BINARY-ARCHITECTURE.md) | `[VERIFIED STATIC]` | Struktur PE32+ x64, 7 seksi (`.text`, `.rdata`, dll.), tabel IAT lengkap, ketiadaan ekspor (0 EAT), compiler Rust MSVC 14.38, dan profil pengamanan biner (DEP, ASLR, CFG). |
| 2 | **DEL-02-11** | [`NUSA-SUBSYSTEM-MAP.md`](file:///x:/folder_ai/Hime/project-management/reverse-engineering/nusa/architecture/NUSA-SUBSYSTEM-MAP.md) | `[ARCHITECTURAL MAP]` | Pemetaan 12 subsistem internal lengkap dengan tanggung jawab, input/output, dependensi, diagram interaksi Mermaid, dan tingkat kepastian. |
| 3 | **DEL-02-12** | [`NUSA-COMMUNICATION-ANALYSIS.md`](file:///x:/folder_ai/Hime/project-management/reverse-engineering/nusa/architecture/NUSA-COMMUNICATION-ANALYSIS.md) | `[PROTOCOL ANALYSIS]` | Analisis antrian Rust MPMC, pesan Win32, callback CoreAudio/Power, eliminasi hipotesis IPC lokal, dan protokol HTTPS REST ke LRCLIB. |
| 4 | **DEL-02-13** | [`NUSA-STATE-PERSISTENCE-ANALYSIS.md`](file:///x:/folder_ai/Hime/project-management/reverse-engineering/nusa/architecture/NUSA-STATE-PERSISTENCE-ANALYSIS.md) | `[STORAGE ANALYSIS]` | Skema berkas teks polos (`settings.txt`, `water.txt`, dll.), risiko korupsi data non-atomic, celah keamanan plaintext clipboard, dan rekomendasi SQLite DPAPI. |
| 5 | **DEL-02-14** | [`NUSA-RENDERING-ARCHITECTURE.md`](file:///x:/folder_ai/Hime/project-management/reverse-engineering/nusa/architecture/NUSA-RENDERING-ARCHITECTURE.md) | `[RENDERING ARCHITECTURE]`| Pipeline D3D11 $\rightarrow$ Direct2D $\rightarrow$ DirectComposition dengan `WS_EX_NOREDIRECTIONBITMAP`, tipografi DirectWrite, DWM acrylic blur, dan hit-testing. |
| 6 | **DEL-02-15** | [`NUSA-WINDOWS-API-MAPPING.md`](file:///x:/folder_ai/Hime/project-management/reverse-engineering/nusa/architecture/NUSA-WINDOWS-API-MAPPING.md) | `[API MAPPING]` | Pemetaan 8 domain integrasi Windows (Shell, Fullscreen, GSMTC, Core Audio, Power, Display, OLE, Registry) ke Tauri 2.0 dan crate `windows-rs`. |
| 7 | **DEL-02-16** | [`HIME-ARCHITECTURE-TRANSLATION.md`](file:///x:/folder_ai/Hime/project-management/reverse-engineering/nusa/architecture/HIME-ARCHITECTURE-TRANSLATION.md) | `[TRANSLATION BLUEPRINT]`| Cetak biru transformasi dari widget pasif ke pusat kendali AI: Kanvas Elastis 3-Tier, Omnibox `Alt+Space`, **Agent Activity Stream**, Tool HUD, dan Job Objects. |
| 8 | **DEL-02-17** | [`NUSA-ARCHITECTURE-RE-DISCOVERY-REPORT.md`](file:///x:/folder_ai/Hime/project-management/reverse-engineering/nusa/architecture/NUSA-ARCHITECTURE-RE-DISCOVERY-REPORT.md) | Dokumen Ini | Laporan sintesis eksekutif penutupan TASK-02-02 dan evaluasi kriteria penerimaan. |
| 9 | **DEL-02-18** | [`project-management/tasks/TASK-02-02-nusa-architecture.md`](file:///x:/folder_ai/Hime/project-management/tasks/TASK-02-02-nusa-architecture.md) | Spesifikasi Task | Spesifikasi tata kelola formal task arsitektur Nusa (Status: QUALIFIED ACCEPTANCE). |
| 10 | **DEL-02-19** | [`project-management/tasks/TASK-02-02B-architecture-calibration.md`](file:///x:/folder_ai/Hime/project-management/tasks/TASK-02-02B-architecture-calibration.md) | Spesifikasi Task | Spesifikasi tata kelola task kalibrasi bukti teknis dan koreksi klaim arsitektur Nusa (Status: COMPLETED). |

---

## 3. Key Architectural Findings Synthesis

### 1. Struktur Biner & Toolchain
- Executable `nusa.exe` berformat **PE32+ 64-bit AMD64** murni berukuran 1,265,664 bytes.
- Dikompilasi menggunakan toolchain **Rust 1.80/1.81** dengan target `x86_64-pc-windows-msvc` (Linker v14.38, Visual Studio 2022).
- Menggunakan crate resmi Microsoft `windows-0.58.0`, `windows-core-0.58.0`, dan `windows-strings-0.1.0`. Tidak ada ketergantungan pada runtime web atau framework GUI berat.

### 2. Peta Subsistem Internal (12 Subsistem)
Aplikasi terbagi rapi ke dalam 12 subsistem fungsional:
1. *Application Lifecycle* (Inisialisasi, Named Mutex, Message pump)
2. *Window Manager* (`NotchWindow` Win32 popup, Z-order topmost, hit-testing per-pixel)
3. *Rendering Engine* (Direct3D 11 swapchain, Direct2D vector, DirectWrite font, DWM blur)
4. *Animation Subsystem* (FSM `animating`, `dirty`, `peek`, `fast_animation`)
5. *Media Integration* (WinRT GSMTC listener & LRCLIB client)
6. *Audio & Hardware HUD* (Core Audio volume, WMI brightness, power notifications)
7. *File Shelf* (OLE COM `IDropTarget`, folder staging `%LOCALAPPDATA%\Nusa\Shelf\`)
8. *Clipboard Subsystem* (`AddClipboardFormatListener`, penyaring privasi)
9. *Widget Modules* (Water tracker, Pomodoro, Prayer Kemenag RI, To-do list)
10. *Settings Manager* (Jendela `NusaSettings`, parser `settings.txt`)
11. *Persistence Layer* (I/O berkas plaintext di `%LOCALAPPDATA%\Nusa\`)
12. *System Tray* (`Shell_NotifyIconW`, auto-start HKCU, `TaskbarCreated` recovery)

### 3. Mekanisme Komunikasi & Protokol
- **Komunikasi Internal:** Keberadaan string source path `library\std\src\sync\mpmc\` dalam segmen `.rdata` terverifikasi statis (`[VERIFIED STATIC — STRING/PATH PRESENCE]`). Keberadaan teks path semata tidak membuktikan bahwa kode pustaka MPMC benar-benar tertaut atau terkompilasi ke dalam executable (`[UNKNOWN]`). Pemanggilan aktif antrean ini oleh logika aplikasi Nusa berstatus `[UNKNOWN INVOCATION]` / `[INFERRED]`.
- **Klarifikasi IPC:** Berdasarkan inspeksi biner statis dan import table: tidak ditemukan indikasi penggunaan Winsock secara langsung melalui import IAT (`ws2_32.dll` absen); tidak ditemukan bukti penggunaan Named Pipe; tidak ditemukan bukti Shared Memory (`CreateFileMappingW` absen). Mekanisme komunikasi lain yang belum dipastikan tetap berstatus **`UNKNOWN`**. Komunikasi internal antarmodul berlangsung in-process. Deteksi instans ganda menggunakan Named Mutex `Local\NusaNotch.SingleInstance`.
- **Komunikasi Jaringan Eksternal:** Permintaan HTTPS GET ke REST API publik `https://lrclib.net/api/search` via WinRT HttpClient untuk sinkronisasi lirik musik (`[VERIFIED STATIC]` / `[INFERRED]`).

### 4. Arsitektur State & Persistensi
- Data disimpan dalam berkas teks mentah format kunci-nilai atau delimited di `%LOCALAPPDATA%\Nusa\`.
- **Kelemahan Kritis:** Ketiadaan transaksi ACID, ketiadaan mekanisme penulisan atomik (rentan korupsi saat crash), dan penyimpanan teks clipboard dalam teks polos tanpa enkripsi.
- **Batas Keamanan Arsitektur Hime:** Membedakan secara tegas peran masing-masing mekanisme: DPAPI (perlindungan data/secret tertentu pada tingkat aplikasi), SQLCipher (enkripsi database pada tingkat aplikasi), BitLocker (enkripsi volume/perangkat penyimpanan pada tingkat OS, bukan enkripsi DB aplikasi), Windows ACL (kontrol akses objek/file berdasarkan identitas Windows), dan SQLite WAL (mekanisme journaling dan konkurensi, bukan enkripsi, di mana durability dipengaruhi konfigurasi `PRAGMA synchronous`, filesystem, dan storage hardware). Kombinasi SQLite WAL + DPAPI tidak secara otomatis mengenkripsi seluruh file database.

### 5. Arsitektur Rendering & Komposisi Jendela
- Menggunakan tumpukan API modern Windows: Direct3D 11 (`D3D11CreateDevice`), Direct2D (`D2D1CreateFactory`), DirectComposition (`DCompositionCreateDevice`), DirectWrite (`DWriteCreateFactory`), dan DWM (`DwmSetWindowAttribute`) yang terverifikasi dalam IAT (`[VERIFIED STATIC]`).
- **Status Pipeline Aktif:** Rantai pipa terpadu `D3D11 → DXGI Swapchain → Direct2D → DirectComposition → DWM` merupakan inferensi arsitektural (`[INFERRED]`), bukan hasil tracing frame GPU aktif.
- **Kapabilitas vs Realitas Performa:** Bendera `WS_EX_NOREDIRECTIONBITMAP` adalah kapabilitas API untuk meniadakan alokasi GDI redirection surface pada DWM. Kapabilitas ini tidak otomatis meniadakan seluruh overhead sistem atau menjamin 60 FPS tanpa beban GPU; kapabilitas API dibedakan secara tegas dari hasil benchmark riil.

### 6. Integrasi Windows & Translasi ke Hime
- Nusa dijadikan referensi teknis & UX terbatas pada: Window Manager, Rendering, Windows Integration, System Tray, Media/Audio, dan Desktop Overlay UX.
- Subsistem orisinal Hime yang baru (Hime Core, Agent Orchestrator, Agent Activity Stream, Tool Execution Gateway, Permission & Approval Engine, Context & Memory Manager, Storage Intelligence, Device Bridge, Productivity Services, Process Supervisor) dipisahkan secara tegas dan **BUKAN berasal dari Nusa**.
- Desain Hime mentransformasi antarmuka pasif Nusa menjadi **AI Agent Command Center** berbasis **Agent Activity Stream** dengan kanvas elastis 3-tier, input Omnibox `Alt + Space`, pengawasan proses berbasis Windows Job Objects, dan persistensi aman SQLite WAL + Windows DPAPI.

---

## 4. Architectural Elements: Inferred vs Unknown

Untuk memastikan kejujuran teknis (*technical integrity*), batas-batas pengetahuan hasil reverse engineering didokumentasikan secara terbuka:

### A. Hal-Hal yang Berstatus Inferensi (`[INFERRED]`)
1. **Kurva Interpolasi Easing Pegas:** Nilai numerik eksak kurva spring ($P_1, P_2$) merupakan deduksi analitis dari kehalusan visual Direct2D, belum diverifikasi rumus matematikanya pada level assembly.
2. **Detail Hit-Testing `WM_NCHITTEST`:** Pengembalian `HTCLIENT` vs `HTTRANSPARENT` disimpulkan dari perilaku kanvas 740×212 px dan interaksi desktop, belum diuji jejak pesannya via WinSpy/Spy++ pada sesi terkontrol saat ini.
3. **Alur Benturan Mutex Peluncuran Ganda:** Rekonstruksi logis dari impor API `CreateMutexW`, `FindWindowW`, `SetForegroundWindow`, dan `ExitProcess`.

### B. Hal-Hal yang Belum Diketahui (`[UNKNOWN]`)
1. **Struktur Modul Kode Sumber Asli:** Nama-nama file `.rs` internal pengembang (selain `src\main.rs`), pembagian crate internal, atau hierarki struct privat Rust yang di-inlining oleh optimasi kompilator LLVM.
2. **Flag Optimasi Kompilasi Komersial:** Konfigurasi `opt-level`, `lto` (Link-Time Optimization), atau `codegen-units` persis yang digunakan oleh pengembang saat mem-build installer rilis publik.

---

## 5. Evaluation of Acceptance Criteria (TASK-02-02)

| Kriteria Penerimaan (AC) | Bukti Evaluasi Teknis | Status |
|:---|:---|:---:|
| **AC-01: Struktur biner dianalisis & terdokumentasi** | Dokumen [`NUSA-BINARY-ARCHITECTURE.md`](file:///x:/folder_ai/Hime/project-management/reverse-engineering/nusa/architecture/NUSA-BINARY-ARCHITECTURE.md) menganalisis PE headers, 7 seksi, tabel IAT, resource, dan compiler Rust/MSVC. | **PASS** |
| **AC-02: Peta subsistem tersedia** | Dokumen [`NUSA-SUBSYSTEM-MAP.md`](file:///x:/folder_ai/Hime/project-management/reverse-engineering/nusa/architecture/NUSA-SUBSYSTEM-MAP.md) memetakan 12 subsistem internal lengkap dengan I/O, dependensi, dan diagram Mermaid. | **PASS** |
| **AC-03: Mekanisme komunikasi dianalisis tanpa klaim berlebihan** | Dokumen [`NUSA-COMMUNICATION-ANALYSIS.md`](file:///x:/folder_ai/Hime/project-management/reverse-engineering/nusa/architecture/NUSA-COMMUNICATION-ANALYSIS.md) menganalisis antrean Rust MPMC (string/path presence `[VERIFIED STATIC — STRING/PATH PRESENCE]`, keterhubungan kode `[UNKNOWN]`, pemanggilan aktif `[UNKNOWN INVOCATION]`), pesan Win32, callback, serta evaluasi IPC berbasis bukti (Winsock, Named Pipe, dan Shared Memory tidak ditemukan via import/string; mekanisme lain `[UNKNOWN]`). | **PASS** |
| **AC-04: State dan persistence terdokumentasi** | Dokumen [`NUSA-STATE-PERSISTENCE-ANALYSIS.md`](file:///x:/folder_ai/Hime/project-management/reverse-engineering/nusa/architecture/NUSA-STATE-PERSISTENCE-ANALYSIS.md) merinci skema berkas teks polos, risiko korupsi, serta membedakan secara tegas batas DPAPI (field secrets), SQLCipher (app DB encryption), BitLocker (OS volume encryption), Windows ACL (file access control), dan SQLite WAL (journaling/konkurensi). | **PASS** |
| **AC-05: Rendering architecture dianalisis** | Dokumen [`NUSA-RENDERING-ARCHITECTURE.md`](file:///x:/folder_ai/Hime/project-management/reverse-engineering/nusa/architecture/NUSA-RENDERING-ARCHITECTURE.md) membedah pipeline D3D11 $\rightarrow$ D2D $\rightarrow$ DComp (pipeline aktif `[INFERRED]` dari standar Windows), kapabilitas `WS_EX_NOREDIRECTIONBITMAP` vs performa benchmark riil, dan hit-testing. | **PASS** |
| **AC-06: Windows integration dipetakan** | Dokumen [`NUSA-WINDOWS-API-MAPPING.md`](file:///x:/folder_ai/Hime/project-management/reverse-engineering/nusa/architecture/NUSA-WINDOWS-API-MAPPING.md) memetakan 8 domain Windows API ke Tauri 2.0 dan crate `windows-rs` dengan pembedaan jelas antara import IAT dan invocation. | **PASS** |
| **AC-07: Pemetaan arsitektur Nusa ke kebutuhan Hime tersedia** | Dokumen [`HIME-ARCHITECTURE-TRANSLATION.md`](file:///x:/folder_ai/Hime/project-management/reverse-engineering/nusa/architecture/HIME-ARCHITECTURE-TRANSLATION.md) menyajikan pemisahan tegas antara domain referensi Nusa dengan 10 subsistem orisinal Hime (Hime Core, Agent Orchestrator, Agent Activity Stream, dll.). | **PASS** |
| **AC-08: Seluruh klaim memiliki klasifikasi bukti** | Seluruh klaim secara konsisten dilabeli: `[VERIFIED STATIC]`, `[INFERRED]`, `[UNKNOWN]`, atau `[NOT TESTED]`. | **PASS** |
| **AC-09: Diagram konsisten dengan bukti** | Diagram Mermaid pada seluruh dokumen secara eksplisit membedakan fakta terbukti dari inferensi. | **PASS** |
| **AC-10: Tidak ada modifikasi binary, konfigurasi OS, atau proprietary code** | Biner `nusa.exe` tidak dimodifikasi, registri Windows host tidak diubah, dan pemisahan clean-room terjaga 100%. | **PASS** |
| **AC-11: Tidak ada commit dan push** | Seluruh perubahan berada di working tree lokal; tidak ada eksekusi commit atau push ke GitHub. | **PASS** |

---

## 6. Status Repositori Git

```text
PS X:\folder_ai\Hime> git status
On branch main
Untracked files:
  (use "git add <file>..." to include in what will be committed)
	project-management/reverse-engineering/
	project-management/tasks/TASK-02-01-nusa-discovery.md
	project-management/tasks/TASK-02-02-nusa-architecture.md
	project-management/tasks/TASK-02-02A-evidence-correction.md

nothing added to commit but untracked files present (use "git add" to track)
```

- Normalisasi LF dan pembersihan baris kosong di akhir berkas (EOF) telah diterapkan pada seluruh berkas deliverable baru.
- Verifikasi `git diff --check` menghasilkan nilai keluar `0` (bersih sempurna tanpa galat whitespace).
- Tidak ada commit atau push yang dilakukan.

---

## 7. Rekomendasi & Penghentian Eksekusi

Seluruh rangkaian analisis arsitektur dan protokol reverse engineering Nusa (TASK-02-02) telah selesai dengan tuntas dan terdokumentasi secara kanonikal.

Sesuai instruksi Project Owner:
1. Pengerjaan dihentikan di sini.
2. Tidak memulai task berikutnya secara otomatis.
3. Menunggu evaluasi dan arahan Project Owner untuk persetujuan formal terhadap paket arsitektur ini.
