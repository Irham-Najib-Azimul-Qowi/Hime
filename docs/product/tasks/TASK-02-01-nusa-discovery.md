# Task Specification: TASK-02-01 — Nusa Application Discovery & Behavioral Analysis

---

## 1. Metadata

- **Task ID:** TASK-02-01
- **Phase:** Phase 02 — Nusa Reverse Engineering
- **Title:** Nusa Application Discovery & Behavioral Analysis
- **Priority:** HIGH
- **Owner:** Senior Reverse Engineering Specialist & Windows Systems Analyst
- **Reviewer:** Project Owner / User
- **Status:** **QUALIFIED ACCEPTANCE** (Dikalibrasi via TASK-02-02A)
- **Dependencies:**
  - Phase 01 — Discovery & Technical Validation (Completed & Formally Accepted)
- **Commit Reference:** Pending Task Acceptance Approval by Project Owner

---

## 2. Description

Lakukan discovery dan analisis perilaku aplikasi Nusa Dynamic Island v1.2.0.0 secara sistematis, non-destruktif, dan berbasis bukti (*evidence-based*) untuk menghasilkan dokumentasi teknis yang menjadi fondasi desain antarmuka, interaksi visual, dan integrasi Windows bagi reimplementasi mandiri (*clean-room reimplementation*) pada komponen Hime Desktop (Tauri 2.0 + Rust + Web Frontend).

---

## 3. Scope Boundaries

### In Scope
- Inventarisasi biner `nusa.exe` dan `NusaSetup.exe` (PE32+ headers, SHA-256, ukuran, compiler, imports, strings `.rdata`, resources `.rsrc`).
- Analisis visual, UI/UX, geometri jendela, kurva animasi spring, palet warna, tipografi, dan maskot.
- Katalogisasi 10 modul fitur fungsional beserta mekanisme internal, input-output, batasan, dan relevansi Hime.
- Analisis siklus hidup proses, Named Mutex, extended styles (`WS_EX_NOREDIRECTIONBITMAP`, dll.), penanganan multi-monitor, resolusi/DPI, deteksi fullscreen, dan penanganan kegagalan API.
- Analisis integrasi subsistem Windows (DWM, DirectComposition, WinRT GSMTC, Core Audio, OLE Drag-Drop, storage isolation) dan peta translasi ke Tauri 2.0.
- Analisis kesenjangan strategis (*quadrant gap analysis*) antara Nusa dan Hime Desktop.
- Penyusunan laporan penutupan task dan pemenuhan seluruh Acceptance Criteria (AC-01 s/d AC-10).

### Out of Scope
- Melakukan modifikasi, penambalan (*patching*), atau injeksi DLL ke biner `nusa.exe` atau `NusaSetup.exe`.
- Mengubah konfigurasi sistem operasi Windows host, file sistem Windows, atau registry.
- Menginstal dependensi baru atau compiler Rust pada fase ini.
- Menulis kode implementasi Tauri 2.0 atau kode sumber aplikasi Hime.
- Menyalin aset grafis proprietary atau kode sumber Nusa ke dalam repositori Hime (menjaga prinsip clean-room).
- Mengubah konfigurasi runtime Hermes Agent atau 9Router.
- Melakukan eksekusi `git commit` atau `git push` sebelum instruksi resmi Project Owner.
- Melanjutkan secara otomatis ke TASK-02-02 sebelum memperoleh persetujuan task ini.

---

## 4. Acceptance Criteria & Verification Matrix

- [x] **AC-01 (Canonical Application Inventory):** Berkas [`NUSA-APPLICATION-INVENTORY.md`](file:///x:/folder_ai/Hime/project-management/reverse-engineering/nusa/NUSA-APPLICATION-INVENTORY.md) diterbitkan dengan metadata PE32+, checksum hash SHA-256, struktur instalasi, dan skema berkas data.
- [x] **AC-02 (Visual & UI/UX Analysis):** Berkas [`NUSA-UI-UX-ANALYSIS.md`](file:///x:/folder_ai/Hime/project-management/reverse-engineering/nusa/NUSA-UI-UX-ANALYSIS.md) memetakan kanvas 740×212 px, kapsul 200×34 px, gaya Classic/Floating, palet warna OLED, tipografi Segoe UI, animasi spring, dan fitur maskot.
- [x] **AC-03 (Verified Feature Catalog):** Berkas [`NUSA-FEATURE-CATALOG.md`](file:///x:/folder_ai/Hime/project-management/reverse-engineering/nusa/NUSA-FEATURE-CATALOG.md) merinci 10 modul fitur terverifikasi lengkap dengan mekanisme internal, model input-output, batasan teknis, dan relevansi Hime.
- [x] **AC-04 (Runtime Behavioral Analysis):** Berkas [`NUSA-BEHAVIORAL-ANALYSIS.md`](file:///x:/folder_ai/Hime/project-management/reverse-engineering/nusa/NUSA-BEHAVIORAL-ANALYSIS.md) membedah siklus hidup proses, Mutex `Local\NusaNotch.SingleInstance`, `SetWinEventHook`, `WM_DPICHANGED`, dan deteksi fullscreen.
- [x] **AC-05 (Windows Subsystem Integration):** Berkas [`NUSA-WINDOWS-INTEGRATION.md`](file:///x:/folder_ai/Hime/project-management/reverse-engineering/nusa/NUSA-WINDOWS-INTEGRATION.md) menganalisis DWM zero-redirection bitmap, DirectComposition, WinRT GSMTC, Core Audio, dan pemetaan lengkap ke Tauri 2.0.
- [x] **AC-06 (Strategic Feature Gap Analysis):** Berkas [`NUSA-FEATURE-GAP-ANALYSIS.md`](file:///x:/folder_ai/Hime/project-management/reverse-engineering/nusa/NUSA-FEATURE-GAP-ANALYSIS.md) menyajikan diagram 4 kuadran, evaluasi kanvas elastis untuk AI chat, transformasi File Shelf, dan fitur unik Hime.
- [x] **AC-07 (Evidence Classification Rigor):** Seluruh klaim diberi label eksplisit: `[VERIFIED]`, `[INFERRED]`, `[OBSERVED]`, atau `[UNKNOWN]`.
- [x] **AC-08 (Strict Scope Boundaries):** Tidak ada penambalan biner, tidak ada modifikasi registri OS, tidak ada kode sumber Tauri prematur, dan clean-room terjaga.
- [x] **AC-09 (Task Specification Produced):** Berkas spesifikasi task ini diterbitkan dengan metadata terstruktur dan indeks artefak lengkap.
- [x] **AC-10 (Clean Repository State & Zero Commit/Push):** `git diff --check` bersih dari galat whitespace; status working tree terjaga tanpa commit atau push.

---

## 5. Deliverables & Documentation Index

| Deliverable ID | Berkas Dokumen | Deskripsi | Status |
|:---:|:---|:---|:---:|
| **DEL-02-01** | [`project-management/reverse-engineering/nusa/NUSA-APPLICATION-INVENTORY.md`](file:///x:/folder_ai/Hime/project-management/reverse-engineering/nusa/NUSA-APPLICATION-INVENTORY.md) | Inventarisasi Biner & Metadata Aplikasi Nusa | **TERBIT** |
| **DEL-02-02** | [`project-management/reverse-engineering/nusa/NUSA-UI-UX-ANALYSIS.md`](file:///x:/folder_ai/Hime/project-management/reverse-engineering/nusa/NUSA-UI-UX-ANALYSIS.md) | Analisis Desain Visual, Geometri, & Animasi UI/UX | **TERBIT** |
| **DEL-02-03** | [`project-management/reverse-engineering/nusa/NUSA-FEATURE-CATALOG.md`](file:///x:/folder_ai/Hime/project-management/reverse-engineering/nusa/NUSA-FEATURE-CATALOG.md) | Katalog 10 Modul Fitur Terverifikasi & Mekanisme Internal | **TERBIT** |
| **DEL-02-04** | [`project-management/reverse-engineering/nusa/NUSA-BEHAVIORAL-ANALYSIS.md`](file:///x:/folder_ai/Hime/project-management/reverse-engineering/nusa/NUSA-BEHAVIORAL-ANALYSIS.md) | Analisis Mesin Status Runtime, Siklus Hidup, & Quirks | **TERBIT** |
| **DEL-02-05** | [`project-management/reverse-engineering/nusa/NUSA-WINDOWS-INTEGRATION.md`](file:///x:/folder_ai/Hime/project-management/reverse-engineering/nusa/NUSA-WINDOWS-INTEGRATION.md) | Analisis Integrasi Mendalam Subsistem Windows & Peta Tauri | **TERBIT** |
| **DEL-02-06** | [`project-management/reverse-engineering/nusa/NUSA-FEATURE-GAP-ANALYSIS.md`](file:///x:/folder_ai/Hime/project-management/reverse-engineering/nusa/NUSA-FEATURE-GAP-ANALYSIS.md) | Analisis Kesenjangan Strategis 4 Kuadran (Nusa vs Hime) | **TERBIT** |
| **DEL-02-07** | [`project-management/reverse-engineering/nusa/NUSA-RE-DISCOVERY-REPORT.md`](file:///x:/folder_ai/Hime/project-management/reverse-engineering/nusa/NUSA-RE-DISCOVERY-REPORT.md) | Laporan Penutupan & Evaluasi Kriteria Penerimaan | **TERBIT** |
| **DEL-02-08** | [`project-management/tasks/TASK-02-01-nusa-discovery.md`](file:///x:/folder_ai/Hime/project-management/tasks/TASK-02-01-nusa-discovery.md) | Spesifikasi Task Tata Kelola Rekayasa (Dokumen Ini) | **TERBIT** |
