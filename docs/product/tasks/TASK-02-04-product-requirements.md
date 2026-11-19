# Task Specification: TASK-02-04 — Hime Product Requirements & Experience Architecture

- **Task ID:** TASK-02-04
- **Phase:** Phase 02 — Nusa Reverse Engineering & Architecture Discovery (Product Specification)
- **Title:** Hime Product Requirements & Experience Architecture
- **Priority:** CRITICAL
- **Owner:** Lead Systems Architect & Principal Product Manager
- **Reviewer:** Project Owner / User
- **Status:** **COMPLETE — PENDING OWNER REVIEW**
- **Dependencies:**
  - TASK-02-01 — Nusa Application Discovery & Behavioral Analysis (Qualified Acceptance)
  - TASK-02-02A — Nusa Evidence Classification & Discovery Report Correction (Accepted)
  - TASK-02-02 — Nusa Architecture & Protocol Reverse Engineering (Qualified Acceptance)
  - TASK-02-02B & TASK-02-02B-FINAL — Architecture Evidence Calibration & Precision Correction (Accepted)
  - TASK-02-03 & TASK-02-03B — Nusa UX & Interaction Behavior Reverse Engineering & Calibration (Accepted / Documented Limitations)
- **Commit Reference:** Pending Task Acceptance Approval by Project Owner (Zero Commit / Zero Push Discipline)

---

## 1. Description & Context

Menyusun Product Requirements & Experience Architecture yang lengkap, konsisten, realistis, dan berorientasi jangka panjang sebagai fondasi perancangan arsitektur sistem (Phase 05) dan implementasi rekayasa (Phase 06-07).

Hime diposisikan sebagai personal AI desktop environment mandiri (*Windows-first, cross-platform by design*), bukan klon Nusa dan bukan wrapper chatbot sederhana. Tugas ini mengintegrasikan pembelajaran dari reverse engineering Nusa sebagai inspirasi UX, merumuskan 14 modul produk, interaksi 3-Tier Canvas, spesifikasi pengalaman Hermes Agent, manajemen penyimpanan lokal dan Android, serta matriks kapabilitas lintas platform Windows/Linux.

---

## 2. Acceptance Criteria & Verification Matrix

- [x] **AC-01 (Product Identity & Philosophy):** Dokumen [`HIME-PRODUCT-VISION.md`](file:///x:/folder_ai/Hime/project-management/product-requirements/HIME-PRODUCT-VISION.md) mendefinisikan visi, masalah pengguna, diferensiasi dari chatbot/Nusa, 6 prinsip UX, dan batas produk.
- [x] **AC-02 (User Context & Use Cases):** Dokumen [`HIME-USER-CONTEXT-AND-USE-CASES.md`](file:///x:/folder_ai/Hime/project-management/product-requirements/HIME-USER-CONTEXT-AND-USE-CASES.md) merinci 11 domain use cases dengan trigger, context, perilaku, izin, dan failure handling.
- [x] **AC-03 (Functional Requirements):** Dokumen [`HIME-FUNCTIONAL-REQUIREMENTS.md`](file:///x:/folder_ai/Hime/project-management/product-requirements/HIME-FUNCTIONAL-REQUIREMENTS.md) mengatalogkan 33 kebutuhan fungsional unik dalam 13 domain dengan kriteria MoSCoW (24 Must, 5 Should, 4 Could).
- [x] **AC-04 (Non-Functional Requirements):** Dokumen [`HIME-NON-FUNCTIONAL-REQUIREMENTS.md`](file:///x:/folder_ai/Hime/project-management/product-requirements/HIME-NON-FUNCTIONAL-REQUIREMENTS.md) merumuskan 18 NFR terukur berbasis profil hardware riil (Intel i5-8250U, ~3.7GB RAM bebas).
- [x] **AC-05 (Product Module Architecture):** Dokumen [`HIME-PRODUCT-MODULE-ARCHITECTURE.md`](file:///x:/folder_ai/Hime/project-management/product-requirements/HIME-PRODUCT-MODULE-ARCHITECTURE.md) memetakan 14 modul produk, keterkaitan Hime Core, dan dependensi OS.
- [x] **AC-06 (UX & Interaction Architecture):** Dokumen [`HIME-UX-AND-INTERACTION-ARCHITECTURE.md`](file:///x:/folder_ai/Hime/project-management/product-requirements/HIME-UX-AND-INTERACTION-ARCHITECTURE.md) mendefinisikan Kanvas 3-Tier, aturan transisi, dan interaksi keyboard-first.
- [x] **AC-07 (Agent Experience Specification):** Dokumen [`HIME-AGENT-EXPERIENCE-SPECIFICATION.md`](file:///x:/folder_ai/Hime/project-management/product-requirements/HIME-AGENT-EXPERIENCE-SPECIFICATION.md) mengatur siklus hidup Hermes, Activity Stream transparan, larangan private CoT, dan izin Human-in-the-Loop.
- [x] **AC-08 (File & Storage Experience):** Dokumen [`HIME-FILE-AND-STORAGE-EXPERIENCE.md`](file:///x:/folder_ai/Hime/project-management/product-requirements/HIME-FILE-AND-STORAGE-EXPERIENCE.md) mengatur deteksi duplikat 5 dimensi, resolusi unduhan, manajemen laptop & Android, serta Undo.
- [x] **AC-09 (Cross-Platform Architecture):** Dokumen [`HIME-CROSS-PLATFORM-EXPERIENCE-ARCHITECTURE.md`](file:///x:/folder_ai/Hime/project-management/product-requirements/HIME-CROSS-PLATFORM-EXPERIENCE-ARCHITECTURE.md) menetapkan strategi Windows-first Linux-second, PAL adapter, Wayland/X11, dan matriks kapabilitas.
- [x] **AC-10 (MVP & Product Roadmap):** Dokumen [`HIME-MVP-AND-PRODUCT-ROADMAP.md`](file:///x:/folder_ai/Hime/project-management/product-requirements/HIME-MVP-AND-PRODUCT-ROADMAP.md) mendefinisikan MVP realistis, kriteria pembobotan, dan peta jalan 4 fase.
- [x] **AC-11 (Risks & Open Decisions):** Dokumen [`HIME-PRODUCT-RISKS-AND-OPEN-DECISIONS.md`](file:///x:/folder_ai/Hime/project-management/product-requirements/HIME-PRODUCT-RISKS-AND-OPEN-DECISIONS.md) mencatat keputusan disetujui, usulan, pertanyaan terbuka Project Owner, dan risk register.
- [x] **AC-12 (Task Completion Report):** Dokumen [`TASK-02-04-COMPLETION-REPORT.md`](file:///x:/folder_ai/Hime/project-management/product-requirements/TASK-02-04-COMPLETION-REPORT.md) merangkum sintesis eksekutif penutupan task.
- [x] **AC-13 (Review & Calibration Report):** Dokumen [`TASK-02-04B-REVIEW-AND-CALIBRATION-REPORT.md`](file:///x:/folder_ai/Hime/project-management/product-requirements/TASK-02-04B-REVIEW-AND-CALIBRATION-REPORT.md) merangkum hasil peninjauan dan kalibrasi terarah TASK-02-04B sesuai evaluasi Project Owner.
- [x] **AC-14 (Zero Implementation Code):** Tidak ada kode implementasi aplikasi Hime yang dibuat.
- [x] **AC-15 (Zero Commit & Zero Push):** Tidak ada staging `git add`, tidak ada commit lokal, dan tidak ada push ke origin GitHub.

---

## 3. Deliverables & Documentation Index

| Deliverable ID | Berkas Dokumen | Deskripsi Konten | Status |
|:---:|:---|:---|:---:|
| **DEL-02-30** | [`project-management/product-requirements/HIME-PRODUCT-VISION.md`](file:///x:/folder_ai/Hime/project-management/product-requirements/HIME-PRODUCT-VISION.md) | Visi, filosofi, 6 prinsip UX, diferensiasi, dan batas produk | **TERBIT (Terkalibrasi)** |
| **DEL-02-31** | [`project-management/product-requirements/HIME-USER-CONTEXT-AND-USE-CASES.md`](file:///x:/folder_ai/Hime/project-management/product-requirements/HIME-USER-CONTEXT-AND-USE-CASES.md) | 11 domain use cases konkret dengan trigger, context, dan alur | **TERBIT (Terkalibrasi)** |
| **DEL-02-32** | [`project-management/product-requirements/HIME-FUNCTIONAL-REQUIREMENTS.md`](file:///x:/folder_ai/Hime/project-management/product-requirements/HIME-FUNCTIONAL-REQUIREMENTS.md) | 33 Functional Requirements ber-ID unik dan kriteria MoSCoW | **TERBIT (Terkalibrasi)** |
| **DEL-02-33** | [`project-management/product-requirements/HIME-NON-FUNCTIONAL-REQUIREMENTS.md`](file:///x:/folder_ai/Hime/project-management/product-requirements/HIME-NON-FUNCTIONAL-REQUIREMENTS.md) | 18 NFR terukur berbasis profil hardware riil (pemisahan 4 proses, target vs ambang batas) | **TERBIT (Terkalibrasi)** |
| **DEL-02-34** | [`project-management/product-requirements/HIME-PRODUCT-MODULE-ARCHITECTURE.md`](file:///x:/folder_ai/Hime/project-management/product-requirements/HIME-PRODUCT-MODULE-ARCHITECTURE.md) | Arsitektur 14 modul produk, relasi Hime Core, dan dependensi OS | **TERBIT (Terkalibrasi)** |
| **DEL-02-35** | [`project-management/product-requirements/HIME-UX-AND-INTERACTION-ARCHITECTURE.md`](file:///x:/folder_ai/Hime/project-management/product-requirements/HIME-UX-AND-INTERACTION-ARCHITECTURE.md) | Model Kanvas Elastis 3-Tier, aturan transisi, dan keyboard-first (Click default DIR-01) | **TERBIT (Terkalibrasi)** |
| **DEL-02-36** | [`project-management/product-requirements/HIME-AGENT-EXPERIENCE-SPECIFICATION.md`](file:///x:/folder_ai/Hime/project-management/product-requirements/HIME-AGENT-EXPERIENCE-SPECIFICATION.md) | Siklus hidup Hermes, Activity Stream transparan, izin Human-in-the-Loop, Hime Core PEP/PDP | **TERBIT (Terkalibrasi)** |
| **DEL-02-37** | [`project-management/product-requirements/HIME-FILE-AND-STORAGE-EXPERIENCE.md`](file:///x:/folder_ai/Hime/project-management/product-requirements/HIME-FILE-AND-STORAGE-EXPERIENCE.md) | Deteksi duplikat 5 dimensi, safe file move, laptop & Android, Undo 30 hari, AC-FAIL-01 s/d 07 | **TERBIT (Terkalibrasi)** |
| **DEL-02-38** | [`project-management/product-requirements/HIME-CROSS-PLATFORM-EXPERIENCE-ARCHITECTURE.md`](file:///x:/folder_ai/Hime/project-management/product-requirements/HIME-CROSS-PLATFORM-EXPERIENCE-ARCHITECTURE.md) | Strategi Windows-first Linux-second, PAL adapter, Wayland/X11, fallback anggun DIR-03 | **TERBIT (Terkalibrasi)** |
| **DEL-02-39** | [`project-management/product-requirements/HIME-MVP-AND-PRODUCT-ROADMAP.md`](file:///x:/folder_ai/Hime/project-management/product-requirements/HIME-MVP-AND-PRODUCT-ROADMAP.md) | Lingkup MVP 12 pilar fondasi, kriteria pembobotan, dan peta jalan 4 fase | **TERBIT (Terkalibrasi)** |
| **DEL-02-40** | [`project-management/product-requirements/HIME-PRODUCT-RISKS-AND-OPEN-DECISIONS.md`](file:///x:/folder_ai/Hime/project-management/product-requirements/HIME-PRODUCT-RISKS-AND-OPEN-DECISIONS.md) | Risiko 9Router Phase 01, klasifikasi kontrol, penerapan DIR-01 s/d 04, dan register OQ-05 s/d 07 | **TERBIT (Terkalibrasi)** |
| **DEL-02-41** | [`project-management/product-requirements/TASK-02-04-COMPLETION-REPORT.md`](file:///x:/folder_ai/Hime/project-management/product-requirements/TASK-02-04-COMPLETION-REPORT.md) | Laporan Sintesis Eksekutif Akhir TASK-02-04 & TASK-02-04B | **TERBIT (Terkalibrasi)** |
| **DEL-02-42** | [`project-management/product-requirements/TASK-02-04B-REVIEW-AND-CALIBRATION-REPORT.md`](file:///x:/folder_ai/Hime/project-management/product-requirements/TASK-02-04B-REVIEW-AND-CALIBRATION-REPORT.md) | Laporan Komprehensif Peninjauan Arsitektur & Kalibrasi Spesifikasi TASK-02-04B | **TERBIT (Terkalibrasi)** |
| **DEL-02-43** | [`project-management/product-requirements/TASK-02-04C-FINAL-CONSISTENCY-REPORT.md`](file:///x:/folder_ai/Hime/project-management/product-requirements/TASK-02-04C-FINAL-CONSISTENCY-REPORT.md) | Laporan Konsistensi Akhir & Kesiapan Penerimaan TASK-02-04C | **TERBIT (Terkalibrasi)** |
| **DEL-02-44** | [`project-management/tasks/TASK-02-04-product-requirements.md`](file:///x:/folder_ai/Hime/project-management/tasks/TASK-02-04-product-requirements.md) | Dokumen Spesifikasi Formal Tata Kelola Task Ini | **TERBIT (Terkalibrasi)** |
| **DEL-02-45** | [`project-management/product-requirements/TASK-02-04D-FINAL-CORRECTION-REPORT.md`](file:///x:/folder_ai/Hime/project-management/product-requirements/TASK-02-04D-FINAL-CORRECTION-REPORT.md) | Laporan Koreksi Presisi Teknis & Abstraksi IPC Lintas-Platform Akhir TASK-02-04D | **TERBIT (Baru)** |
