# Task Specification: TASK-02-02 — Nusa Architecture & Protocol Reverse Engineering

---

## 1. Metadata

- **Task ID:** TASK-02-02
- **Phase:** Phase 02 — Nusa Reverse Engineering
- **Title:** Nusa Architecture & Protocol Reverse Engineering
- **Priority:** CRITICAL
- **Owner:** Senior Systems Architect & Reverse Engineering Specialist
- **Reviewer:** Project Owner / User
- **Status:** **QUALIFIED ACCEPTANCE (CALIBRATED VIA TASK-02-02B)**
- **Dependencies:**
  - TASK-02-01 — Nusa Application Discovery & Behavioral Analysis (Qualified Acceptance)
  - TASK-02-02A — Nusa Evidence Classification & Discovery Report Correction (Accepted)
- **Follow-up:**
  - TASK-02-02B — Nusa Architecture Evidence Calibration & Technical Correction (Completed)
- **Commit Reference:** Pending Task Acceptance Approval by Project Owner

---

## 2. Description

Lakukan analisis arsitektur dan protokol komprehensif terhadap aplikasi Nusa Dynamic Island v1.2.0.0 berdasarkan artefak biner PE32+, metadata, dokumentasi teknis, dan bukti biner yang tersedia tanpa memodifikasi biner atau sistem operasi Windows.

Tugas ini membedah struktur internal Nusa, memetakan 12 subsistem internal, menginvestigasi mekanisme komunikasi dan penanganan pesan sistem operasi, menganalisis lapisan persistensi teks polos dan risiko korupsi/privasi data, membedah arsitektur rendering DirectComposition/Direct2D/DWM, memetakan integrasi subsistem Windows, serta merumuskan cetak biru translasi arsitektur (*clean-room reimplementation*) menuju pusat kendali asisten AI **Hime Desktop** berbasis **Tauri 2.0 (Rust) + Webview2**.

---

## 3. Scope Boundaries

### In Scope
- Analisis struktur biner PE32+ (DOS/NT headers, 7 seksi memori, IAT, ketiadaan ekspor, `.rsrc`, jejak toolchain Rust/MSVC).
- Pemetaan 12 subsistem internal (Lifecycle, Window Manager, Rendering Engine, Animation, Media, Audio HUD, File Shelf, Clipboard, Widgets, Settings, Persistence, System Tray).
- Investigasi mekanisme komunikasi: antrian Rust MPMC, pesan Win32, callback CoreAudio/Power, eliminasi hipotesis IPC lokal, dan protokol HTTPS REST ke LRCLIB.
- Analisis arsitektur state dan persistensi: berkas teks polos di `%LOCALAPPDATA%\Nusa\`, risiko korupsi data non-atomic, celah keamanan plaintext clipboard, dan rekomendasi SQLite DPAPI.
- Analisis arsitektur rendering: pipeline D3D11 $\rightarrow$ Direct2D $\rightarrow$ DirectComposition dengan `WS_EX_NOREDIRECTIONBITMAP`, tipografi DirectWrite, DWM acrylic blur, dan hit-testing.
- Pemetaan integrasi Windows API ke Tauri 2.0, plugin resmi Tauri, dan crate `windows-rs`.
- Penyusunan cetak biru translasi arsitektur Hime Desktop: Kanvas Elastis 3-Tier, Omnibox `Alt+Space`, **Agent Activity Stream**, Tool Execution HUD, dan supervisi multi-proses Job Objects.
- Penyusunan laporan penutupan task dan evaluasi kriteria penerimaan (AC-01 s/d AC-11).

### Out of Scope
- Mengubah, menambal (*patching*), atau memodifikasi biner `nusa.exe` atau `NusaSetup.exe`.
- Mengubah registry, sistem berkas sistem Windows, atau konfigurasi host Windows.
- Menginstal dependensi baru atau compiler Rust pada fase ini.
- Mengekstrak atau menyalin aset grafis atau kode proprietary Nusa ke dalam Hime.
- Menulis kode implementasi aplikasi Hime Desktop.
- Memulai task berikutnya secara otomatis.
- Menjalankan perintah `git commit` atau `git push`.

---

## 4. Acceptance Criteria & Verification Matrix

- [x] **AC-01 (Binary Structure Analyzed & Documented):** Berkas [`NUSA-BINARY-ARCHITECTURE.md`](file:///x:/folder_ai/Hime/project-management/reverse-engineering/nusa/architecture/NUSA-BINARY-ARCHITECTURE.md) menganalisis PE headers, 7 seksi, tabel IAT, resource, dan compiler Rust/MSVC.
- [x] **AC-02 (Subsystem Map Available):** Berkas [`NUSA-SUBSYSTEM-MAP.md`](file:///x:/folder_ai/Hime/project-management/reverse-engineering/nusa/architecture/NUSA-SUBSYSTEM-MAP.md) memetakan 12 subsistem internal lengkap dengan I/O, dependensi, dan diagram Mermaid.
- [x] **AC-03 (Communication Analyzed without Overclaiming):** Berkas [`NUSA-COMMUNICATION-ANALYSIS.md`](file:///x:/folder_ai/Hime/project-management/reverse-engineering/nusa/architecture/NUSA-COMMUNICATION-ANALYSIS.md) menganalisis antrean Rust MPMC (string/path presence `[VERIFIED STATIC — STRING/PATH PRESENCE]`, keterhubungan kode `[UNKNOWN]`, pemanggilan aktif `[UNKNOWN INVOCATION]`), pesan Win32, callback, serta evaluasi IPC berbasis bukti (Winsock, Named Pipe, Shared Memory tidak ditemukan via import/string; mekanisme lain `[UNKNOWN]`).
- [x] **AC-04 (State & Persistence Documented):** Berkas [`NUSA-STATE-PERSISTENCE-ANALYSIS.md`](file:///x:/folder_ai/Hime/project-management/reverse-engineering/nusa/architecture/NUSA-STATE-PERSISTENCE-ANALYSIS.md) merinci skema berkas teks polos, risiko korupsi, serta membedakan secara tegas batas DPAPI (field secrets), SQLCipher (app DB encryption), BitLocker (OS volume encryption), Windows ACL (file access control), dan SQLite WAL (journaling/konkurensi).
- [x] **AC-05 (Rendering Architecture Analyzed):** Berkas [`NUSA-RENDERING-ARCHITECTURE.md`](file:///x:/folder_ai/Hime/project-management/reverse-engineering/nusa/architecture/NUSA-RENDERING-ARCHITECTURE.md) membedah pipeline D3D11 $\rightarrow$ D2D $\rightarrow$ DComp (pipeline aktif `[INFERRED]` dari standar Windows), kapabilitas `WS_EX_NOREDIRECTIONBITMAP` vs performa benchmark riil, dan hit-testing.
- [x] **AC-06 (Windows Integration Mapped):** Berkas [`NUSA-WINDOWS-API-MAPPING.md`](file:///x:/folder_ai/Hime/project-management/reverse-engineering/nusa/architecture/NUSA-WINDOWS-API-MAPPING.md) memetakan 8 domain Windows API ke Tauri 2.0 dan crate `windows-rs` dengan pembedaan jelas antara import IAT dan invocation.
- [x] **AC-07 (Hime Architecture Translation Blueprint):** Berkas [`HIME-ARCHITECTURE-TRANSLATION.md`](file:///x:/folder_ai/Hime/project-management/reverse-engineering/nusa/architecture/HIME-ARCHITECTURE-TRANSLATION.md) menyajikan pemisahan tegas antara domain referensi Nusa dengan 10 subsistem orisinal Hime (Hime Core, Agent Orchestrator, Agent Activity Stream, dll.).
- [x] **AC-08 (Evidence Classification Rigor):** Seluruh klaim secara konsisten dilabeli: `[VERIFIED STATIC]`, `[INFERRED]`, `[UNKNOWN]`, atau `[NOT TESTED]`.
- [x] **AC-09 (Diagrams Consistent with Evidence):** Diagram Mermaid pada seluruh dokumen secara eksplisit membedakan fakta terbukti dari inferensi.
- [x] **AC-10 (Strict Clean-Room Discipline):** Biner `nusa.exe` tidak dimodifikasi, registri Windows host tidak diubah, dan pemisahan clean-room terjaga 100%.
- [x] **AC-11 (Zero Commit & Zero Push):** Seluruh perubahan berada di working tree lokal; tidak ada eksekusi commit atau push ke GitHub.

---

## 5. Deliverables & Documentation Index

| Deliverable ID | Berkas Dokumen | Deskripsi | Status |
|:---:|:---|:---|:---:|
| **DEL-02-10** | [`project-management/reverse-engineering/nusa/architecture/NUSA-BINARY-ARCHITECTURE.md`](file:///x:/folder_ai/Hime/project-management/reverse-engineering/nusa/architecture/NUSA-BINARY-ARCHITECTURE.md) | Analisis Biner PE32+, IAT, dan Toolchain Rust | **TERBIT** |
| **DEL-02-11** | [`project-management/reverse-engineering/nusa/architecture/NUSA-SUBSYSTEM-MAP.md`](file:///x:/folder_ai/Hime/project-management/reverse-engineering/nusa/architecture/NUSA-SUBSYSTEM-MAP.md) | Peta Dekomposisi 12 Subsistem Internal & Diagram Arsitektur | **TERBIT** |
| **DEL-02-12** | [`project-management/reverse-engineering/nusa/architecture/NUSA-COMMUNICATION-ANALYSIS.md`](file:///x:/folder_ai/Hime/project-management/reverse-engineering/nusa/architecture/NUSA-COMMUNICATION-ANALYSIS.md) | Investigasi Protokol, Antrian MPMC, & Eliminasi IPC | **TERBIT** |
| **DEL-02-13** | [`project-management/reverse-engineering/nusa/architecture/NUSA-STATE-PERSISTENCE-ANALYSIS.md`](file:///x:/folder_ai/Hime/project-management/reverse-engineering/nusa/architecture/NUSA-STATE-PERSISTENCE-ANALYSIS.md) | Analisis Skema Penyimpanan Teks, Risiko Korupsi, & SQLite | **TERBIT** |
| **DEL-02-14** | [`project-management/reverse-engineering/nusa/architecture/NUSA-RENDERING-ARCHITECTURE.md`](file:///x:/folder_ai/Hime/project-management/reverse-engineering/nusa/architecture/NUSA-RENDERING-ARCHITECTURE.md) | Pipeline Grafis DirectComposition, Direct2D, & Hit-Testing | **TERBIT** |
| **DEL-02-15** | [`project-management/reverse-engineering/nusa/architecture/NUSA-WINDOWS-API-MAPPING.md`](file:///x:/folder_ai/Hime/project-management/reverse-engineering/nusa/architecture/NUSA-WINDOWS-API-MAPPING.md) | Pemetaan 8 Domain Windows API ke Ekosistem Tauri 2.0 | **TERBIT** |
| **DEL-02-16** | [`project-management/reverse-engineering/nusa/architecture/HIME-ARCHITECTURE-TRANSLATION.md`](file:///x:/folder_ai/Hime/project-management/reverse-engineering/nusa/architecture/HIME-ARCHITECTURE-TRANSLATION.md) | Cetak Biru Translasi Arsitektur Menuju Hime Desktop | **TERBIT** |
| **DEL-02-17** | [`project-management/reverse-engineering/nusa/architecture/NUSA-ARCHITECTURE-RE-DISCOVERY-REPORT.md`](file:///x:/folder_ai/Hime/project-management/reverse-engineering/nusa/architecture/NUSA-ARCHITECTURE-RE-DISCOVERY-REPORT.md) | Laporan Penutupan & Evaluasi Kriteria Penerimaan | **TERBIT** |
| **DEL-02-18** | [`project-management/tasks/TASK-02-02-nusa-architecture.md`](file:///x:/folder_ai/Hime/project-management/tasks/TASK-02-02-nusa-architecture.md) | Spesifikasi Task Tata Kelola Rekayasa (Dokumen Ini) | **TERBIT** |
