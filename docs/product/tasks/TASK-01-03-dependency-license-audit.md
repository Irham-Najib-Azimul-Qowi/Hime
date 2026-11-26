# Task Specification: TASK-01-03 — Dependency and License Audit

---

## 1. Metadata

- **Task ID:** TASK-01-03
- **Phase:** Phase 01 — Discovery & Environment Audit
- **Title:** Dependency and License Audit & Supply Chain Security Review
- **Priority:** HIGH
- **Owner:** Senior Infrastructure Architect & Compliance Specialist
- **Reviewer:** Project Owner / User
- **Status:** **QUALIFIED ACCEPTANCE**
- **Dependencies:** TASK-01-02 (Accepted)
- **Commit Reference:** Pending Review Approval

---

## 2. Description

Lakukan audit dependensi dan lisensi secara menyeluruh terhadap seluruh komponen pihak ketiga dalam ekosistem Hime Assistant, mencakup dependensi langsung dan transitif Hermes Agent (Python / `uv.lock`), dependensi runtime 9Router (Node.js / `package.json`), 7 pustaka Curated Agent Skills, batas hak cipta aplikasi referensi Nusa, serta baseline dependensi terencana untuk Hime Desktop dan Hime Core. Evaluasi kompatibilitas lisensi terhadap rencana distribusi biner Windows, identifikasi kewajiban redistribusi, audit risiko rantai pasok (*supply chain*), dan terbitkan register lisensi kanonikal (`LICENSE-REGISTER.md`).

---

## 3. Scope Boundaries

### In Scope
- Audit dependensi langsung (46 paket) dan paket terkunci di `uv.lock` (356 locked/resolved packages; direct + transitive; transitif murni: `NOT SEPARATELY COUNTED`) Hermes Agent.
- Audit dependensi produksi (30 paket), opsional (4 paket), dan development 9Router.
- Verifikasi lisensi dan atribusi 7 Curated Agent Skills di `.agent/skills/`.
- Perumusan baseline dependensi terencana Hime Desktop (Tauri 2, React 19, Tailwind) dan Hime Core (Tokio, Rusqlite, Windows DPAPI).
- Audit batasan aset proprietary Nusa (Clean-room boundary enforcement).
- Klasifikasi lisensi (Permissive, Weak Copyleft, Strong Copyleft, Proprietary).
- Analisis kompatibilitas distribusi biner desktop Windows (.msi / .exe).
- Penilaian risiko rantai pasok (lazy runtime downloads, post-install scripts, version pinning).
- Penyusunan dokumen register lisensi: `project-management/inventory/LICENSE-REGISTER.md`.
- Penyusunan laporan teknis utama: `project-management/reviews/TASK-01-03-DEPENDENCY-LICENSE-AUDIT.md`.

### Out of Scope
- Menginstal, memperbarui, atau menghapus dependensi host, npm, atau Python.
- Mengubah berkas manifes atau lockfiles pada `hermes-agent` atau `9router`.
- Mengunduh paket biner eksternal atau menjalankan skrip pihak ketiga yang belum dipercaya.
- Memulai eksekusi kode atau mengimplementasikan packaging.
- Melakukan operasi `git push` ke GitHub remote.
- Memulai eksekusi TASK-01-04 sebelum persetujuan pengguna.

---

## 4. Acceptance Criteria & Verification Matrix

- [x] **AC-1 (Hermes Dependency Audit):** 46 Dependensi langsung diperiksa lisensinya; 356 locked/resolved packages di `uv.lock` teridentifikasi (transitif murni: `NOT SEPARATELY COUNTED`; lisensi transitif diklasifikasikan sebagai `UNVERIFIED / NOT FULLY AUDITED`).
- [x] **AC-2 (9Router Dependency Audit):** 34 Dependensi 9Router teridentifikasi dari manifes; ketiadaan lockfile dan sifat unverified dependensi transitif dicatat.
- [x] **AC-3 (Agent Skills Audit):** 7 Curated Skills diperiksa berkas fisiknya; 100% berkas Markdown murni berlisensi MIT.
- [x] **AC-4 (Planned Baseline):** Baseline dependensi terencana Hime Desktop & Core diklasifikasikan terpisah dari dependensi aktif.
- [x] **AC-5 (Nusa Asset Boundaries):** Larangan redistribusi biner proprietary Nusa dan legalitas clean-room didokumentasikan tanpa asumsi provenance yang belum terbukti.
- [x] **AC-6 (License Classification):** 87 Paket dependensi langsung open-source diklasifikasikan (56 MIT, 14 Apache-2.0, 10 BSD, 4 Weak Copyleft, 3 Permissive Khusus, 0 Copyleft pada paket terperiksa).
- [x] **AC-7 (Distribution Compatibility):** 10 Pertanyaan model distribusi biner Windows dijawab tanpa klaim absolut; `node-forge` dan `libheif` ditandai memerlukan peninjauan hukum.
- [x] **AC-8 (Supply Chain Risk):** Dekomposisi RSK-09 membedakan secara rinci mekanisme dynamic npm install on-demand (`lazyRuntime.js`), npm install saat startup (`sqliteRuntime.js`), dan download biner native systray dengan verifikasi hash (`trayRuntime.js`, bypassed pada Windows via PowerShell `NotifyIcon`).
- [x] **AC-9 (License Register Published):** Register lisensi kanonikal diterbitkan di `project-management/inventory/LICENSE-REGISTER.md`.
- [x] **AC-10 (Zero Runtime Modification):** Tidak ada dependensi yang diinstal atau diubah selama audit.
- [x] **AC-11 (Documentation Integrity):** Laporan utama diterbitkan di `project-management/reviews/TASK-01-03-DEPENDENCY-LICENSE-AUDIT.md`.

---

## 5. Verification Evidence Summary

- **Hermes Agent Manifes:** `RE/Hermes/00-source-materials/hermes-agent/pyproject.toml` & `uv.lock`.
- **Hermes License:** `RE/Hermes/00-source-materials/hermes-agent/LICENSE` (MIT).
- **9Router Manifes:** `RE/9ROUTER/00-source-materials/9router/package.json` & `cli/package.json`.
- **9Router License:** `RE/9ROUTER/00-source-materials/9router/LICENSE` (MIT).
- **Skills Inspeksi Fisik:** 16 berkas Markdown di `.agent/skills/`.
- **Nusa Biner Metadata:** `RE/Nusa/00-source-materials/extracted/install_script.iss` (Copyright Rama).
- **Git Baseline Log:** `06dd996 docs(discovery): accept task 01-02 asset inventory`.
