# Task Specification: TASK-02-02A — Nusa Evidence Classification & Discovery Report Correction

---

## 1. Metadata

- **Task ID:** TASK-02-02A
- **Phase:** Phase 02 — Nusa Reverse Engineering
- **Title:** Nusa Evidence Classification & Discovery Report Correction
- **Priority:** CRITICAL
- **Owner:** Senior Reverse Engineering Specialist & Windows Systems Analyst
- **Reviewer:** Project Owner / User
- **Status:** **PENDING FINAL OWNER ACCEPTANCE**
- **Dependencies:**
  - TASK-02-01 — Nusa Application Discovery & Behavioral Analysis (Qualified Acceptance)
- **Commit Reference:** Pending Task Acceptance Approval by Project Owner

---

## 2. Description

Lakukan satu putaran koreksi terarah terhadap ketepatan klasifikasi bukti dan dokumentasi hasil penemuan Nusa Dynamic Island v1.2.0.0 pada direktori `project-management/reverse-engineering/nusa/`.

Tugas ini mengimplementasikan skema klasifikasi 5 kategori secara konsisten (`VERIFIED STATIC`, `VERIFIED RUNTIME`, `INFERRED`, `UNKNOWN`, `NOT TESTED`), mengkualifikasi klaim performa (RAM ~58–61 MB, CPU 0.0%) sebagai temuan observasi laporan awal yang belum diuji empiris dalam sesi saat ini, serta menyelaraskan rekomendasi antarmuka asisten AI pada Hime Desktop ke arah konsep **Agent Activity Stream** (bukan private chain-of-thought).

---

## 3. Scope Boundaries

### In Scope
- Menerapkan skema 5 kategori klasifikasi bukti pada seluruh dokumen di `project-management/reverse-engineering/nusa/`.
- Memisahkan secara tegas bukti statis (`[VERIFIED STATIC]`) dan inferensi (`[INFERRED]`).
- Mengaudit klaim UI dan perilaku runtime (spring physics, geometri, mouse/keyboard, fullscreen auto-hide, DPI scaling, sleep/resume, startup).
- Mengoreksi klaim performa: menandai RAM ~58–61 MB dan CPU 0.0% sebagai `[NOT TESTED IN THIS SESSION]` (berasal dari laporan riset awal).
- Menggantikan seluruh rekomendasi konsep "Thought Stream" / penalaran internal mentah dengan konsep **Agent Activity Stream** (status agen, ringkasan rencana, tool aktif, progres eksekusi, hasil tindakan, alasan keputusan ringkas, modal konfirmasi).
- Memperbarui ketujuh dokumen reverse engineering Nusa dan laporan penutupan task.

### Out of Scope
- Mengulang reverse engineering dari awal.
- Memodifikasi berkas executable `nusa.exe` atau `NusaSetup.exe`.
- Mengubah konfigurasi registry atau sistem operasi Windows host.
- Menginstal dependensi baru atau compiler Rust.
- Menulis kode implementasi Hime Desktop.
- Menyalin aset proprietary Nusa ke repositori Hime.
- Memulai TASK-02-02 (arsitektur mendalam) atau TASK-02-03.
- Menjalankan perintah `git commit` atau `git push`.

---

## 4. Acceptance Criteria & Verification Matrix

- [x] **AC-01 (Appropriate Evidence Classification):** Seluruh klaim penting memiliki klasifikasi bukti yang tepat (`VERIFIED STATIC`, `VERIFIED RUNTIME`, `INFERRED`, `UNKNOWN`, `NOT TESTED`).
- [x] **AC-02 (Static vs Runtime Evidence Distinction):** Bukti statis (header PE, IAT, string biner) dan perilaku runtime dibedakan secara tegas.
- [x] **AC-03 (Performance Claims Qualification):** Angka RAM 58–61 MB dan CPU 0.0% dikualifikasikan sebagai temuan historis dari laporan awal yang belum diuji pada sesi terkontrol saat ini.
- [x] **AC-04 (UI & Behavioral Claims Calibrated):** Klaim kurva spring physics, kanvas 740×212 px, dan hit-testing diturunkan statusnya menjadi `[INFERRED]`.
- [x] **AC-05 (Agent Activity Stream Paradigm):** Konsep Agent Activity Stream menggantikan Thought Stream di seluruh dokumen.
- [x] **AC-06 (Cross-Document Consistency):** Seluruh 7 dokumen reverse engineering Nusa konsisten dan selaras.
- [x] **AC-07 (No Fabricated Observations):** Tidak ada klaim observasi atau pengujian runtime yang dibuat-buat; batasan pengujian didokumentasikan transparan.
- [x] **AC-08 (Clean Git Diff):** `git diff --check` bersih dari galat whitespace dan baris baru.
- [x] **AC-09 (Zero Commit & Zero Push):** Perubahan berada dalam working tree lokal tanpa eksekusi commit atau push.

---

## 5. Deliverables & Documentation Index

| Deliverable ID | Berkas Dokumen | Deskripsi | Status |
|:---:|:---|:---|:---:|
| **DEL-02-01** | [`project-management/reverse-engineering/nusa/NUSA-APPLICATION-INVENTORY.md`](file:///x:/folder_ai/Hime/project-management/reverse-engineering/nusa/NUSA-APPLICATION-INVENTORY.md) | Inventarisasi Biner & Metadata (Koreksi Klasifikasi Statis) | **TERKALIBRASI** |
| **DEL-02-02** | [`project-management/reverse-engineering/nusa/NUSA-UI-UX-ANALYSIS.md`](file:///x:/folder_ai/Hime/project-management/reverse-engineering/nusa/NUSA-UI-UX-ANALYSIS.md) | Analisis UI/UX & Matriks Audit Bukti UI/Runtime | **TERKALIBRASI** |
| **DEL-02-03** | [`project-management/reverse-engineering/nusa/NUSA-FEATURE-CATALOG.md`](file:///x:/folder_ai/Hime/project-management/reverse-engineering/nusa/NUSA-FEATURE-CATALOG.md) | Katalog 10 Fitur Terverifikasi Statis & Relevansi Hime | **TERKALIBRASI** |
| **DEL-02-04** | [`project-management/reverse-engineering/nusa/NUSA-BEHAVIORAL-ANALYSIS.md`](file:///x:/folder_ai/Hime/project-management/reverse-engineering/nusa/NUSA-BEHAVIORAL-ANALYSIS.md) | Analisis Perilaku Runtime & Kualifikasi Metrik Performa | **TERKALIBRASI** |
| **DEL-02-05** | [`project-management/reverse-engineering/nusa/NUSA-WINDOWS-INTEGRATION.md`](file:///x:/folder_ai/Hime/project-management/reverse-engineering/nusa/NUSA-WINDOWS-INTEGRATION.md) | Integrasi Windows & Peta Translasi Tauri 2.0 | **TERKALIBRASI** |
| **DEL-02-06** | [`project-management/reverse-engineering/nusa/NUSA-FEATURE-GAP-ANALYSIS.md`](file:///x:/folder_ai/Hime/project-management/reverse-engineering/nusa/NUSA-FEATURE-GAP-ANALYSIS.md) | Analisis Kesenjangan 4 Kuadran & Agent Activity Stream | **TERKALIBRASI** |
| **DEL-02-07** | [`project-management/reverse-engineering/nusa/NUSA-RE-DISCOVERY-REPORT.md`](file:///x:/folder_ai/Hime/project-management/reverse-engineering/nusa/NUSA-RE-DISCOVERY-REPORT.md) | Laporan Penutupan & Evaluasi Kriteria Penerimaan | **TERKALIBRASI** |
| **DEL-02-09** | [`project-management/tasks/TASK-02-02A-evidence-correction.md`](file:///x:/folder_ai/Hime/project-management/tasks/TASK-02-02A-evidence-correction.md) | Spesifikasi Task Tata Kelola Rekayasa (Dokumen Ini) | **TERBIT** |
