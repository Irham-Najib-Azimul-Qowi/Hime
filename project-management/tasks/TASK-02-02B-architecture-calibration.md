# Task Specification: TASK-02-02B — Nusa Architecture Evidence Calibration & Technical Correction

---

## 1. Metadata

- **Task ID:** TASK-02-02B
- **Phase:** Phase 02 — Nusa Reverse Engineering
- **Title:** Nusa Architecture Evidence Calibration & Technical Correction
- **Priority:** CRITICAL
- **Owner:** Senior Systems Architect & Evidence Calibration Specialist
- **Reviewer:** Project Owner / User
- **Status:** **COMPLETE — PENDING OWNER REVIEW**
- **Dependencies:**
  - TASK-02-02 — Nusa Architecture & Protocol Reverse Engineering (Qualified Acceptance)
- **Commit Reference:** Pending Approval by Project Owner (Zero Commit / Zero Push Discipline)

---

## 2. Description & Context

Tugas ini merupakan putaran koreksi dan kalibrasi bukti teknis terarah terhadap dokumentasi arsitektur hasil TASK-02-02 berdasarkan evaluasi Qualified Acceptance dari Project Owner.

Fokus kalibrasi adalah memastikan tidak ada kesimpulan mekanisme internal aplikasi yang ditarik hanya berdasarkan keberadaan API, string, atau pustaka standar dalam biner, melainkan dibedakan secara tegas antara bukti statis (kompilasi/impor), inferensi arsitektural, dan hal-hal yang belum diketahui (*unknown*).

Tugas ini **tidak mengulang seluruh reverse engineering**, **tidak memodifikasi biner atau sistem operasi**, dan **tidak menulis kode implementasi aplikasi**.

---

## 3. Scope of Calibrations

1. **Rust MPMC & Komunikasi Antar-Thread:**
   - Membedakan keberadaan string/source path `library\std\src\sync\mpmc\` dalam `.rdata` (`[VERIFIED STATIC — STRING/PATH PRESENCE]`) dari keterhubungan kode MPMC dengan executable (`[UNKNOWN]`) dan bukti pemanggilan aktif oleh logika aplikasi Nusa (`[UNKNOWN INVOCATION]` / `[INFERRED]`).
   - Tidak menyimpulkan kompilasi/penautan pustaka hanya dari keberadaan source path string.
   - Mengkualifikasikan peran `PostMessageW` sebagai mekanisme dispatch inferensial.
2. **IPC & Komunikasi Jaringan:**
   - Menghindari klaim mutlak bahwa Nusa "pasti tidak menggunakan" pipe atau socket.
   - Menggunakan rumusan berbasis bukti terverifikasi: tidak ditemukan indikasi langsung Winsock melalui IAT, tidak ditemukan bukti Named Pipe pada string/biner, tidak ditemukan bukti Shared Memory pada IAT; mekanisme lain berstatus `[UNKNOWN]`.
   - Membedakan tegas antara API import, API invocation, implementasi protokol, dan aktivitas jaringan runtime.
3. **Rendering Pipeline & DirectX / DirectComposition:**
   - Membedakan IAT impor, metadata/string, dan bukti keterhubungan antar-komponen.
   - Menegaskan bahwa pipeline aktif `D3D11 → DXGI Swapchain → Direct2D → DirectComposition → DWM` merupakan inferensi dari pola standar Windows, bukan hasil tracing frame GPU aktif.
   - Mengoreksi klaim `WS_EX_NOREDIRECTIONBITMAP` sebagai kapabilitas API untuk meniadakan alokasi GDI redirection bitmap pada DWM; bukan jaminan otomatis 60 FPS atau bebas beban GPU; membedakan kapabilitas API dengan hasil benchmark riil.
4. **Keamanan State, Persistensi, & Enkripsi:**
   - Mengklasifikasikan secara terpisah dan presisi:
     - **Windows DPAPI:** Perlindungan data atau secret tertentu melalui Windows Data Protection API pada tingkat aplikasi.
     - **SQLCipher:** Enkripsi database pada tingkat aplikasi (*application-level database encryption*).
     - **BitLocker:** Enkripsi volume atau perangkat penyimpanan pada tingkat sistem operasi (*OS-level volume / storage device encryption*). BUKAN enkripsi database pada tingkat aplikasi.
     - **Windows File ACLs:** Kontrol akses terhadap objek dan berkas berdasarkan identitas serta izin Windows (*OS-level object/file access control*).
     - **SQLite WAL:** Mekanisme journaling dan konkurensi (BUKAN enkripsi).
   - Memperjelas batas SQLite WAL: memungkinkan pembacaan bersamaan dengan transaksi penulisan dalam kondisi yang didukung SQLite; durability dipengaruhi konfigurasi `PRAGMA synchronous`, filesystem, dan write-caching hardware; kemampuan concurrent read tidak sama dengan jaminan bebas blocking dalam seluruh kondisi.
   - Menegaskan bahwa kombinasi SQLite WAL + DPAPI **tidak secara otomatis mengenkripsi seluruh file database**.
5. **Translasi Arsitektur ke Hime:**
   - Menjaga arsitektur Hime agar tidak terlalu bergantung pada pembagian subsistem Nusa.
   - Menetapkan Nusa strictly sebagai referensi untuk: Window Manager, Rendering, Windows Integration, System Tray, Media/Audio, dan Desktop Overlay UX.
   - Memisahkan secara tegas 10 subsistem orisinal Hime yang **BUKAN berasal dari Nusa**: Hime Core, Agent Orchestrator, Agent Activity Stream, Tool Execution Gateway, Permission & Approval Engine, Context & Memory Manager, Storage Intelligence, Device Bridge, Productivity Services, dan Process Supervisor.

---

## 4. Acceptance Criteria & Verification Matrix

| Kriteria Penerimaan (AC) | Bukti Evaluasi Teknis | Status |
|:---|:---|:---:|
| **AC-01: Bukti MPMC dikalibrasi** | Keberadaan string `library\std\src\sync\mpmc\` di segmen `.rdata` dilabeli `[VERIFIED STATIC — STRING/PATH PRESENCE]`, keterhubungan kode MPMC dengan executable `[UNKNOWN]`, dan pemanggilan aktif oleh logika aplikasi dilabeli `[UNKNOWN INVOCATION]`. | **PASS** |
| **AC-02: Klaim IPC dikalibrasi dengan rumusan berbasis bukti** | Dirumuskan secara tegas: tidak ditemukan indikasi langsung Winsock via IAT, tidak ditemukan bukti Named Pipe atau Shared Memory; mekanisme lain `[UNKNOWN]`. Dibedakan import, invocation, protocol, runtime. | **PASS** |
| **AC-03: Komponen pipeline rendering didokumentasikan terpisah** | IAT imports, strings/metadata, dan keterhubungan antar-komponen dipisahkan secara eksplisit; pipeline aktif dinyatakan sebagai `[INFERRED]` dari standar Windows. | **PASS** |
| **AC-04: Klaim `WS_EX_NOREDIRECTIONBITMAP` dikalibrasi** | Dijelaskan sebagai kapabilitas API untuk meniadakan alokasi GDI redirection surface; dibedakan tegas dari performa benchmark riil; klaim jaminan 60 FPS dihapus/dikualifikasi. | **PASS** |
| **AC-05: Peran DPAPI, SQLCipher, BitLocker, ACL, dan SQLite WAL dipisahkan** | DPAPI (field secrets), SQLCipher (app DB encryption), BitLocker (OS volume encryption, bukan app DB encryption), Windows ACL (file access control), dan SQLite WAL (journaling & concurrency mechanism) diklasifikasikan secara independen. | **PASS** |
| **AC-06: Penegasan SQLite WAL + DPAPI tidak mengenkripsi seluruh DB** | Pernyataan eksplisit dicantumkan bahwa WAL bukan enkripsi, durability WAL dipengaruhi `PRAGMA synchronous`, dan kombinasi WAL + DPAPI tidak otomatis mengenkripsi seluruh file DB. | **PASS** |
| **AC-07: Batas subsistem orisinal Hime dipisahkan secara tegas** | Nusa ditetapkan sebagai referensi windowing/rendering/UX; 10 subsistem orisinal Hime dinyatakan BUKAN turunan Nusa pada bab terpisah. | **PASS** |
| **AC-08: Seluruh dokumen di direktori architecture diperbarui** | Kedelapan dokumen arsitektur di `project-management/reverse-engineering/nusa/architecture/` diperbarui secara konsisten. | **PASS** |
| **AC-09: Diagram arsitektur dikoreksi** | Diagram Mermaid pada `NUSA-SUBSYSTEM-MAP.md`, `NUSA-COMMUNICATION-ANALYSIS.md`, dan `NUSA-RENDERING-ARCHITECTURE.md` disesuaikan tingkat kepastian buktinya. | **PASS** |
| **AC-10: Tata kelola & batas rekayasa dipatuhi** | Tidak ada modifikasi binary, tidak ada modifikasi OS/registry, tidak ada penulisan kode implementasi, zero commit, dan zero push. | **PASS** |

---

## 5. Deliverables Package Summary

| Berkas Dokumen | Perubahan Kalibrasi yang Diterapkan | Status |
|:---|:---|:---:|
| [`NUSA-BINARY-ARCHITECTURE.md`](file:///x:/folder_ai/Hime/project-management/reverse-engineering/nusa/architecture/NUSA-BINARY-ARCHITECTURE.md) | Kalibrasi MPMC (Section 7.B) dan DirectComposition (Section 4.A). | **TERBIT** |
| [`NUSA-SUBSYSTEM-MAP.md`](file:///x:/folder_ai/Hime/project-management/reverse-engineering/nusa/architecture/NUSA-SUBSYSTEM-MAP.md) | Diagram RenderEngine & CPU idle observation disesuaikan ke inferensi standar. | **TERBIT** |
| [`NUSA-COMMUNICATION-ANALYSIS.md`](file:///x:/folder_ai/Hime/project-management/reverse-engineering/nusa/architecture/NUSA-COMMUNICATION-ANALYSIS.md) | Diagram bus MPMC dikalibrasi; rumusan evaluasi IPC berbasis bukti diterapkan. | **TERBIT** |
| [`NUSA-STATE-PERSISTENCE-ANALYSIS.md`](file:///x:/folder_ai/Hime/project-management/reverse-engineering/nusa/architecture/NUSA-STATE-PERSISTENCE-ANALYSIS.md) | Penegasan batas DPAPI, SQLCipher/BitLocker, Windows ACL, dan SQLite WAL. | **TERBIT** |
| [`NUSA-RENDERING-ARCHITECTURE.md`](file:///x:/folder_ai/Hime/project-management/reverse-engineering/nusa/architecture/NUSA-RENDERING-ARCHITECTURE.md) | Rincian komponen IAT vs metadata vs linkage; kalibrasi `WS_EX_NOREDIRECTIONBITMAP`. | **TERBIT** |
| [`NUSA-WINDOWS-API-MAPPING.md`](file:///x:/folder_ai/Hime/project-management/reverse-engineering/nusa/architecture/NUSA-WINDOWS-API-MAPPING.md) | Diharmonisasikan dengan pembedaan API import vs invocation dan batas keamanan. | **TERBIT** |
| [`HIME-ARCHITECTURE-TRANSLATION.md`](file:///x:/folder_ai/Hime/project-management/reverse-engineering/nusa/architecture/HIME-ARCHITECTURE-TRANSLATION.md) | Pemisahan tegas domain referensi Nusa vs 10 subsistem orisinal Hime. | **TERBIT** |
| [`NUSA-ARCHITECTURE-RE-DISCOVERY-REPORT.md`](file:///x:/folder_ai/Hime/project-management/reverse-engineering/nusa/architecture/NUSA-ARCHITECTURE-RE-DISCOVERY-REPORT.md) | Pembaruan ringkasan eksekutif dan evaluasi AC-01 s/d AC-11. | **TERBIT** |
| [`TASK-02-02-nusa-architecture.md`](file:///x:/folder_ai/Hime/project-management/tasks/TASK-02-02-nusa-architecture.md) | Pembaruan status Qualified Acceptance dan kriteria penerimaan terkalibrasi. | **TERBIT** |
| [`TASK-02-02B-architecture-calibration.md`](file:///x:/folder_ai/Hime/project-management/tasks/TASK-02-02B-architecture-calibration.md) | Dokumen spesifikasi task kalibrasi ini. | **TERBIT** |
