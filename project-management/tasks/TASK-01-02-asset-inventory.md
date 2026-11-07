# Task Specification: TASK-01-02 — Existing Project & Research Asset Inventory

---

## 1. Metadata

- **Task ID:** TASK-01-02
- **Phase:** Phase 01 — Discovery & Environment Audit
- **Title:** Existing Project & Research Asset Inventory
- **Priority:** HIGH
- **Owner:** Senior Engineering Infrastructure Architect & Repository Specialist
- **Reviewer:** Project Owner / User
- **Status:** **ACCEPTED**
- **Dependencies:** TASK-01-01 (Accepted)
- **Commit Reference:** `docs(discovery): accept task 01-02 asset inventory`

---

## 2. Description

Lakukan inventarisasi menyeluruh terhadap seluruh aset fisik dan intelektual yang tersedia di ruang kerja Hime (`X:\folder_ai\Hime`), meliputi struktur repositori Git, aplikasi referensi Nusa Dynamic Island, repositori dan runtime Hermes Agent, gateway model AI 9Router, seluruh artefak dan laporan reverse engineering, dokumen tata kelola rekayasa, pustaka skill, serta lisensi perangkat lunak pihak ketiga. Susun peta hubungan aset (*Asset Relationship Mapping*) menggunakan diagram visual, identifikasi kesenjangan (*gap analysis*), dan terbitkan indeks aset kanonikal (`ASSET-REGISTER.md`).

---

## 3. Scope Boundaries

### In Scope
- Inventarisasi struktur direktori repositori Git root dan fungsi aktual setiap folder penting.
- Inventarisasi aplikasi referensi Nusa (biner, installer Inno Setup, string `.rdata`, bukti ketiadaan kode sumber, dan matriks fitur).
- Inventarisasi repositori Hermes Agent (commit `c5f347eb`, lisensi MIT, Python 3.13 / `uv`, arsitektur loop penalaran, IPC Named Pipe).
- Inventarisasi repositori 9Router (v0.5.112 commit `1fe586bb`, lisensi MIT, Node.js v24, pemetaan 14 laporan riset, dan analisis risiko token mesin RSK-08).
- Audit dokumentasi dan pemetaan redundansi (Single Source of Truth untuk acceptance Phase 00).
- Audit lisensi pihak ketiga (Nusa proprietary vs Hermes/9Router/Skills MIT).
- Visualisasi hubungan aset menggunakan diagram Mermaid.
- Analisis kesenjangan (menjawab 10 pertanyaan konsolidasi dan menyusun tabel GAP).
- Penyusunan dokumen indeks aset: `project-management/inventory/ASSET-REGISTER.md`.
- Penyusunan laporan utama: `project-management/reviews/TASK-01-02-ASSET-INVENTORY-REPORT.md`.

### Out of Scope
- Mengubah kode runtime Hime, Hermes Agent, atau 9Router.
- Menginstal, menghapus, atau memperbarui dependensi host atau paket npm/pip.
- Menghapus, memindahkan, atau mengubah materi sumber di `RE/`.
- Memulai implementasi kode produk Hime Desktop atau Hime Core.
- Mengulang pengujian reverse engineering yang sudah selesai.
- Melakukan operasi `git push` ke GitHub remote.
- Memulai eksekusi TASK-01-03 sebelum persetujuan pengguna.

---

## 4. Acceptance Criteria & Verification Matrix

- [x] **AC-1 (Repository Inventory):** Seluruh direktori, source code, docs, config, dan script terinventarisasi dengan fungsi aktualnya.
- [x] **AC-2 (Nusa Asset Inventory):** Biner installer `NusaSetup.exe`, `install_script.iss`, dan string `.rdata` dipetakan; ketiadaan kode sumber dinyatakan tegas.
- [x] **AC-3 (Nusa Research Inventory):** 8 Laporan riset dipetakan dan Matriks Fitur Nusa disusun lengkap (`Feature | Evidence | Current Understanding | Confidence | Follow-up`).
- [x] **AC-4 (Hermes Agent Inventory):** Repositori commit `c5f347eb`, lisensi MIT, 10-phase loop, Named Pipe IPC, dan pembagian peran Hermes vs Hime Core dipetakan.
- [x] **AC-5 (9Router Inventory):** Repositori v0.5.112 commit `1fe586bb`, lisensi MIT, 14 laporan riset, dan risiko RSK-08 didokumentasikan.
- [x] **AC-6 (Documentation Audit):** Seluruh dokumen dipetakan; single source of truth untuk acceptance Phase 00 ditetapkan di `reviews/PHASE-00-ACCEPTANCE.md`.
- [x] **AC-7 (Dependency & License Inventory):** Status lisensi Nusa (Proprietary), Hermes (MIT), 9Router (MIT), dan Skills (MIT) diverifikasi.
- [x] **AC-8 (Asset Relationship Mapping):** Diagram Mermaid hubungan antar komponen dibuat (rebuild, retain, integrate, build).
- [x] **AC-9 (Consolidation & Gap Analysis):** 10 Pertanyaan dijawab dan tabel matriks gap disusun (GAP-01 s/d GAP-05).
- [x] **AC-10 (Asset Register Published):** Indeks aset kanonikal diterbitkan di `project-management/inventory/ASSET-REGISTER.md`.
- [x] **AC-11 (Zero Code Changes):** Tidak ada perubahan kode runtime atau aset asli di `RE/`.
- [x] **AC-12 (Documentation Integrity):** Laporan utama diterbitkan di `project-management/reviews/TASK-01-02-ASSET-INVENTORY-REPORT.md`.

---

## 5. Verification Evidence Summary

- **Struktur Repositori:** Query PowerShell `Get-ChildItem -Path . -Force` dan inspeksi Python traversal.
- **Biner Nusa:** SHA-256 installer `NusaSetup.exe` (`BC4133...`), seksi PE biner `nusa.exe`, dan script `install_script.iss`.
- **Hermes Agent:** `git -C ... log -1` (`c5f347eb`), berkas `LICENSE` (MIT), dan inspeksi `pyproject.toml`.
- **9Router:** `git -C ... log -1` (`1fe586bb`), berkas `LICENSE` (MIT), dan inspeksi `package.json`.
- **Curated Skills:** 7 pustaka skill di `.agent/skills/` terkonfirmasi aktif dan berlisensi MIT.
- **Git History:** Baseline intact (`8cf0070`), commit acceptance penutupan Phase 00 (`0099969`), commit TASK-01-01 acceptance (`85ec774`).

---

## 6. Formal Acceptance & Asset Designation

- **Acceptance Status:** **ACCEPTED** (Qualified Acceptance oleh Project Owner pada 2026-11-07).
- **Penetapan Indeks Utama Aset:** Dokumen [`project-management/inventory/ASSET-REGISTER.md`](file:///x:/folder_ai/Hime/project-management/inventory/ASSET-REGISTER.md) secara resmi ditetapkan sebagai **Indeks Utama Aset Proyek Hime** yang bersifat permanen dan terus dipelihara seiring perkembangan proyek.
- **Integritas Artefak:** Laporan komprehensif [`project-management/reviews/TASK-01-02-ASSET-INVENTORY-REPORT.md`](file:///x:/folder_ai/Hime/project-management/reviews/TASK-01-02-ASSET-INVENTORY-REPORT.md) dan seluruh artefak pendukung di `RE/` tersimpan utuh tanpa modifikasi atau penghapusan materi asli.
- **Tindak Lanjut Rekayasa:** Audit lisensi formal dan penelusuran dependensi rinci (direct & transitive) dilanjutkan ke **TASK-01-03 (Dependency and License Audit)**.
