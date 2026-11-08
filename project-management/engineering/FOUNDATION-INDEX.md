# Hime Engineering Foundation — Quick Navigation Index

- **Document ID:** FOUNDATION-INDEX
- **Phase:** Phase 01 — Discovery & Technical Validation
- **Authority:** Senior Infrastructure Architect & Project Governance Specialist
- **Date:** 2026-11-08
- **Status:** **CANONICAL REFERENCE INDEX**

---

## 1. Overview

Dokumen ini menyediakan peta navigasi ringkas menuju seluruh dokumen sumber kanonikal dan artefak fondasi rekayasa yang telah divalidasi dan disetujui selama pelaksanaan **Phase 01: Discovery & Technical Validation**.

Setiap dokumen di bawah ini berfungsi sebagai satu-satunya sumber kebenaran (*single source of truth*) untuk domain teknisnya masing-masing.

### Status Penerimaan Task Phase 01 (Governance Baseline):
| Task ID | Nama Task | Status Resmi |
|:---:|:---|:---:|
| **TASK-01-01** | Environment Baseline Validation | **ACCEPTED** |
| **TASK-01-02** | Existing Project & Research Asset Inventory | **ACCEPTED** |
| **TASK-01-03** | Dependency and License Audit | **QUALIFIED ACCEPTANCE** |
| **TASK-01-04** | Security & Runtime Environment Audit | **QUALIFIED ACCEPTANCE** |
| **TASK-01-05** | Discovery Consolidation & Phase Acceptance | **QUALIFIED ACCEPTANCE** |
| **PHASE-01** | Discovery & Technical Validation | **COMPLETED** |

---

## 2. Directory & Document Navigation Map

```text
project-management/
├── PHASE-01-ACCEPTANCE.md                        # Laporan Penutupan & Penerimaan Formal Phase 01
├── engineering/
│   ├── FOUNDATION-INDEX.md                       # Indeks Navigasi Fondasi Rekayasa (Dokumen Ini)
│   ├── ENGINEERING-FOUNDATION.md                 # Rujukan Utama Fondasi Rekayasa Terpadu
│   ├── DECISION-REGISTER.md                      # Register Keputusan Teknis & Keamanan Project Owner
│   └── IMPLEMENTATION-ROADMAP.md                 # Peta Jalan Implementasi Sadar Dependensi (Phase 02–15)
├── inventory/
│   ├── ASSET-REGISTER.md                         # Indeks Kanonikal Seluruh Aset Proyek & Riset
│   └── LICENSE-REGISTER.md                       # Indeks Lisensi Pihak Ketiga & Kewajiban Hukum
├── security/
│   ├── THREAT-MODEL.md                           # Model Ancaman Komprehensif (Metodologi STRIDE)
│   └── SECURITY-BASELINE.md                      # 10 Kontrol Keamanan Wajib (SEC-01 s/d SEC-10)
├── reviews/
│   ├── TASK-01-01-ENVIRONMENT-BASELINE-REPORT.md # Laporan Audit Lingkungan Host Windows 11
│   ├── TASK-01-02-ASSET-INVENTORY-REPORT.md      # Laporan Inventarisasi Aset Proyek & Riset
│   ├── TASK-01-03-DEPENDENCY-LICENSE-AUDIT.md    # Laporan Audit Dependensi & Lisensi
│   └── TASK-01-04-SECURITY-RUNTIME-AUDIT.md      # Laporan Audit Keamanan & Lingkungan Runtime
└── risks/
    └── RISK_REGISTER.md                          # Register Risiko Terkonsolidasi (RSK-01 s/d RSK-12)
```

---

## 3. Topical Source Document Links

### A. Environment Baseline & Hardware Capacity
- **Laporan Audit Utama:** [`project-management/reviews/TASK-01-01-ENVIRONMENT-BASELINE-REPORT.md`](file:///x:/folder_ai/Hime/project-management/reviews/TASK-01-01-ENVIRONMENT-BASELINE-REPORT.md)
- **Ringkasan Fondasi:** [`project-management/engineering/ENGINEERING-FOUNDATION.md#3-environment-baseline--hardware-capacity`](file:///x:/folder_ai/Hime/project-management/engineering/ENGINEERING-FOUNDATION.md)
- **Topik Kunci:** CPU Intel Core i5-8250U, RAM 20.8 GB (free ~3.71 GB), Intel UHD Graphics 620, strategi inferensi awal Cloud AI, ketersediaan Node.js / Python / uv / Git, prasyarat Rust compiler untuk Phase 06.

### B. Asset Inventory & Reverse Engineering Provenance
- **Register Aset Kanonikal:** [`project-management/inventory/ASSET-REGISTER.md`](file:///x:/folder_ai/Hime/project-management/inventory/ASSET-REGISTER.md)
- **Laporan Audit Inventarisasi:** [`project-management/reviews/TASK-01-02-ASSET-INVENTORY-REPORT.md`](file:///x:/folder_ai/Hime/project-management/reviews/TASK-01-02-ASSET-INVENTORY-REPORT.md)
- **Topik Kunci:** Clean-room reimplementation Nusa v1.2.0.0, integritas repositori Hermes Agent (commit `c5f347eb`), 9Router monorepo (commit `1fe586bb`), 7 Curated Skills (.agent/skills), dan aset dokumentasi legacy.

### C. Dependencies, Supply Chain & Third-Party Licensing
- **Register Lisensi Kanonikal:** [`project-management/inventory/LICENSE-REGISTER.md`](file:///x:/folder_ai/Hime/project-management/inventory/LICENSE-REGISTER.md)
- **Laporan Audit Dependensi:** [`project-management/reviews/TASK-01-03-DEPENDENCY-LICENSE-AUDIT.md`](file:///x:/folder_ai/Hime/project-management/reviews/TASK-01-03-DEPENDENCY-LICENSE-AUDIT.md)
- **Topik Kunci:** Hermes (46 direct, 356 locked/resolved packages di `uv.lock`, transitif murni `NOT SEPARATELY COUNTED`), 9Router (34 direct dari manifes, ketiadaan lockfile), kepatuhan lisensi MIT, pencegahan lisensi copyleft (GPL/AGPL).

### D. Security Architecture, Threat Model & Risk Register
- **Baseline Keamanan (SEC-01 s/d SEC-10):** [`project-management/security/SECURITY-BASELINE.md`](file:///x:/folder_ai/Hime/project-management/security/SECURITY-BASELINE.md)
- **Model Ancaman STRIDE (11 Skenario Serangan):** [`project-management/security/THREAT-MODEL.md`](file:///x:/folder_ai/Hime/project-management/security/THREAT-MODEL.md)
- **Laporan Audit Keamanan:** [`project-management/reviews/TASK-01-04-SECURITY-RUNTIME-AUDIT.md`](file:///x:/folder_ai/Hime/project-management/reviews/TASK-01-04-SECURITY-RUNTIME-AUDIT.md)
- **Register Risiko Terkonsolidasi:** [`project-management/risks/RISK_REGISTER.md`](file:///x:/folder_ai/Hime/project-management/risks/RISK_REGISTER.md)
- **Topik Kunci:** RSK-10 (Localhost Auth Bypass Blocker), RSK-12 (0.0.0.0 LAN Exposure & Runtime Socket Verification Planned Control), RSK-03 (Plaintext SQLite), kualifikasi teknis akses memori satu pengguna (`PROCESS_VM_READ` & DACL), larangan menerima residual risk secara sepihak.

### E. Governance Decisions & Implementation Roadmap
- **Register Keputusan Project Owner:** [`project-management/engineering/DECISION-REGISTER.md`](file:///x:/folder_ai/Hime/project-management/engineering/DECISION-REGISTER.md)
- **Peta Jalan Implementasi Sadar Dependensi:** [`project-management/engineering/IMPLEMENTATION-ROADMAP.md`](file:///x:/folder_ai/Hime/project-management/engineering/IMPLEMENTATION-ROADMAP.md)
- **Laporan Penutupan & Penerimaan Phase 01:** [`project-management/PHASE-01-ACCEPTANCE.md`](file:///x:/folder_ai/Hime/project-management/PHASE-01-ACCEPTANCE.md)
- **Status Proyek Terkini:** [`PROJECT_STATUS.md`](file:///x:/folder_ai/Hime/PROJECT_STATUS.md)
- **Backlog Rekayasa:** [`project-management/backlog/BACKLOG.md`](file:///x:/folder_ai/Hime/project-management/backlog/BACKLOG.md)
