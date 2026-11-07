# Hime Asset Register (Daftar Aset Proyek)

- **Dokumen:** Indeks Utama Aset Proyek Hime Assistant
- **Fase:** Phase 01 — Discovery & Environment Audit
- **Task ID:** TASK-01-02
- **Terakhir Diperbarui:** 2026-11-07
- **Otoritas Pemelihara:** AI Engineering Infrastructure Architect & Repository Specialist

---

## 1. Pendahuluan & Klasifikasi Aset

Dokumen ini merupakan indeks kanonikal seluruh aset berharga dalam ekosistem Hime Assistant, mencakup aplikasi referensi, repositori pihak ketiga, artefak riset rekayasa terbalik (*reverse engineering*), pustaka skill kecerdasan buatan, dokumen tata kelola, dan spesifikasi arsitektur.

### Kategori Status Aset (*Lifecycle Status*):
- **ACTIVE / READY:** Aset terverifikasi lengkap, utuh, dan siap digunakan langsung dalam alur kerja rekayasa.
- **INTEGRATION TARGET:** Komponen eksternal yang akan diintegrasikan melalui antarmuka atau sub-proses tanpa modifikasi kode sumber inti (*integrate, do not rebuild*).
- **REFERENCE ONLY:** Aset proprietary atau eksternal yang berfungsi murni sebagai cetak biru desain dan referensi perilaku pengguna (*reimplementation source*).
- **PLANNED / SKELETON:** Direktori terstruktur yang disiapkan untuk tahap implementasi berikutnya (memuat `.gitkeep`).
- **PENDING VERIFICATION:** Aset yang membutuhkan audit lanjutan sebelum dapat dioperasikan.

---

## 2. Inventaris Aset Repositori Utama & Infrastruktur

| Asset ID | Nama Aset | Kategori / Tipe | Lokasi Relatif | Status | Lisensi / Asal | Deskripsi Singkat |
|:---|:---|:---|:---|:---:|:---:|:---|
| **AST-INF-01** | Root Git Repository | Version Control | `.git/` | ACTIVE | Internal Hime | Repositori Git root dengan baseline commit pelindung dan pelacakan riwayat rekayasa. |
| **AST-INF-02** | Git Ignore Rules | Configuration | `.gitignore` | ACTIVE | Internal Hime | Aturan pengecualian komprehensif (3.7 KB) untuk database SQLite, nested repo, binary, dan secrets. |
| **AST-INF-03** | Agent Operational Rules | Governance | `AGENTS.md` | ACTIVE | Internal Hime | Aturan permanen coding agent: evidence-first, minimal implementation, least privilege, security policy. |
| **AST-INF-04** | Project Status Tracker | Governance | `PROJECT_STATUS.md` | ACTIVE | Internal Hime | Pelacak kemajuan fase proyek resmi berbasis 16 fase Master Plan. |
| **AST-INF-05** | Engineering Backlog | Project Mgmt | `project-management/backlog/BACKLOG.md` | ACTIVE | Internal Hime | Daftar item backlog granular per fase dari Phase 00 hingga Phase 11. |
| **AST-INF-06** | Risk Register | Security / Risk | `project-management/risks/RISK_REGISTER.md` | ACTIVE | Internal Hime | Register risiko komprehensif mencakup RSK-01 hingga RSK-12. |
| **AST-INF-07** | Decision Log | Architecture | `project-management/decisions/DECISION_LOG.md` | ACTIVE | Internal Hime | Log keputusan arsitektur dan tata kelola yang disetujui. |
| **AST-INF-08** | Governance Policies | Governance | `docs/governance/` | ACTIVE | Internal Hime | 10 Dokumen tata kelola resmi (Workflow, DoD, Coding Standard, Security, Skills, Audit). |
| **AST-INF-09** | Engineering Templates | Operations | `docs/` & `project-management/` | ACTIVE | Internal Hime | Template baku untuk Task, Phase, ADR, Code Review, QA, dan Security Review. |
| **AST-INF-10** | Projects Source Workspace | Source Code | `Projects/` | PLANNED | Internal Hime | Direktori kanonikal kode sumber masa depan (Hime Desktop & Core) yang dilindungi `.gitkeep`. |
| **AST-INF-11** | Tools & Automation | Scripts / Utilities | `tools/` & `scripts/` | PLANNED | Internal Hime | Direktori script build, packaging, dan tooling yang dilindungi `.gitkeep`. |
| **AST-INF-12** | Documentation & Archive | Historical Docs | `Documentation/` & `Archive/` | PLANNED | Internal Hime | Direktori arsip dokumen lama dan catatan transisi yang dilindungi `.gitkeep`. |
| **AST-INF-13** | License Register | Governance / Legal | `project-management/inventory/LICENSE-REGISTER.md` | ACTIVE | Internal Hime | Register lisensi kanonikal mencakup 87 dependensi langsung, 356 paket terkunci `uv.lock`, dan pemetaan risiko lisensi. |
| **AST-INF-14** | Security Threat Model | Security / STRIDE | `project-management/security/THREAT-MODEL.md` | ACTIVE | Internal Hime | Model ancaman formal STRIDE mencakup trust boundaries, data flow, dan 10 skenario serangan mendalam. |
| **AST-INF-15** | Security Baseline Specification | Security / Governance | `project-management/security/SECURITY-BASELINE.md` | ACTIVE | Internal Hime | Spesifikasi 10 kontrol keamanan rekayasa wajib (SEC-01 s/d SEC-10) untuk implementasi Hime. |
| **AST-INF-16** | Engineering Foundation Reference | Engineering / Architecture | `project-management/engineering/ENGINEERING-FOUNDATION.md` | ACTIVE | Internal Hime | Rujukan kanonikal terpadu: hardware baseline, profil aset, arsitektur sistem, kontrol keamanan, dan batasan rekayasa. |
| **AST-INF-17** | Consolidated Decision Register | Governance / Decisions | `project-management/engineering/DECISION-REGISTER.md` | ACTIVE | Internal Hime | Register pelacak 12 keputusan teknis, arsitektural, dan risiko residu Project Owner dengan rincian opsi dan trade-off. |
| **AST-INF-18** | Implementation Roadmap | Project Mgmt / Roadmap | `project-management/engineering/IMPLEMENTATION-ROADMAP.md` | ACTIVE | Internal Hime | Peta jalan implementasi 11 workstream sadar dependensi dan batas hardware riil dari Phase 02 hingga Phase 15. |
| **AST-INF-19** | Foundation Navigation Index | Documentation / Index | `project-management/engineering/FOUNDATION-INDEX.md` | ACTIVE | Internal Hime | Direktori navigasi cepat ke seluruh artefak fondasi rekayasa dan laporan audit kanonikal Phase 01. |
| **AST-INF-20** | Phase 01 Acceptance Report | Governance / Acceptance | `project-management/PHASE-01-ACCEPTANCE.md` | ACTIVE | Internal Hime | Laporan penutupan formal dan penerimaan Phase 01 (Discovery & Technical Validation) untuk sign-off Project Owner. |

---

## 3. Inventaris Aset Aplikasi Referensi (Nusa Dynamic Island)

| Asset ID | Nama Aset | Kategori / Tipe | Lokasi Relatif | Status | Lisensi / Asal | Deskripsi Singkat |
|:---|:---|:---|:---|:---:|:---:|:---|
| **AST-NSA-01** | Nusa Installer Executable | Binary Package | `RE/Nusa/00-source-materials/NusaSetup.exe` | REFERENCE ONLY | Proprietary (Rama) | Installer biner Inno Setup v6.7.0 (2.94 MB, SHA-256 terverifikasi) untuk inspeksi dan unpacking. |
| **AST-NSA-02** | Nusa Distribution Archive | Compressed Zip | `RE/Nusa/00-source-materials/NusaSetup.zip` | REFERENCE ONLY | Proprietary (Rama) | Arsip kompresi installer distribusi Nusa v1.2.0.0 (2.43 MB). |
| **AST-NSA-03** | Nusa Static Executable | Native Binary | `RE/Nusa/00-source-materials/extracted/{app}/nusa.exe` | REFERENCE ONLY | Proprietary (Rama) | Biner native Rust PE32+ (1.26 MB) yang diekstraksi untuk analisis statis seksi biner dan disassembli. |
| **AST-NSA-04** | Inno Setup Manifest | Setup Script | `RE/Nusa/00-source-materials/extracted/install_script.iss` | REFERENCE ONLY | Proprietary (Rama) | Script setup dekompilasi `innounp 2.70.1` yang memetakan Registry, privilege, mutex, dan file layout. |
| **AST-NSA-05** | Nusa Binary Strings Table | Raw Dump | `RE/Nusa/00-source-materials/extracted/rdata_strings.txt` | ACTIVE | Internal Research | Dump 49.6 KB string ASCII/Unicode dari seksi `.rdata` biner `nusa.exe` yang memetakan seluruh API dan konfigurasi. |
| **AST-NSA-06** | Nusa PE Extraction Map | Analysis Log | `RE/Nusa/02-binary-analysis/extracted-structure.txt` | ACTIVE | Internal Research | Catatan struktur seksi PE, ukuran, virtual address, dan tata letak penyimpanan lokal Nusa. |
| **AST-NSA-07** | Nusa RE Research Suite | Documentation | `RE/Nusa/` (8 laporan) | ACTIVE | Internal Research | 8 Laporan investigasi teknis mendalam mencakup biner, arsitektur window, UI/UX, fitur, dan feasibility. |

---

## 4. Inventaris Aset Komponen Eksternal: Hermes Agent (Nous Research)

| Asset ID | Nama Aset | Kategori / Tipe | Lokasi Relatif | Status | Lisensi / Asal | Deskripsi Singkat |
|:---|:---|:---|:---|:---:|:---:|:---|
| **AST-HRM-01** | Hermes Cloned Codebase | Cloned Git Repo | `RE/Hermes/00-source-materials/hermes-agent/` | INTEGRATION TARGET | MIT (Nous Research) | Repositori resmi `NousResearch/hermes-agent` commit `c5f347eb`, branch `main`. Diisolasi dari Git root. |
| **AST-HRM-02** | Hermes Package Manifests | Dependencies | `.../hermes-agent/pyproject.toml` & `uv.lock` | INTEGRATION TARGET | MIT (Nous Research) | Konfigurasi runtime Python 3.11–3.14 dan file kunci lockfile Astral `uv` (1.25 MB). |
| **AST-HRM-03** | Hermes Agent Core Loop | Python Code | `.../hermes-agent/agent/` & `conversation_loop.py` | INTEGRATION TARGET | MIT (Nous Research) | Mesin status loop penalaran 10 fase (*10-phase reasoning loop*) dan kelas orchestrator `AIAgent`. |
| **AST-HRM-04** | Hermes Windows Pipe IPC | Python Code | `.../hermes-agent/gateway/windows_pipe.py` | INTEGRATION TARGET | MIT (Nous Research) | Transport IPC Win32 Named Pipe (`\\.\pipe\hermes-gateway-<hash>`) dengan tiket sesi satu kali (TTL 30 detik). |
| **AST-HRM-05** | Hermes State & FTS5 DB | Python Code | `.../hermes-agent/hermes_state*.py` | INTEGRATION TARGET | MIT (Nous Research) | Lapisan persistensi memori SQLite WAL bertingkat dengan pencarian teks penuh FTS5 dan kompresi trajektori. |
| **AST-HRM-06** | Hermes Dynamic Skills | Skill Modules | `.../hermes-agent/skills/` & `plugin-catalog/` | INTEGRATION TARGET | MIT / Various | Pustaka skill built-in dan plugin catalog yang dapat dimuat dinamis oleh runtime agen. |
| **AST-HRM-07** | Hermes RE Research Suite | Documentation | `RE/Hermes/reports/` (5 laporan) | ACTIVE | Internal Research | 5 Laporan analisis mendalam mencakup overview, arsitektur internal, integrasi IPC, feasibility, dan blueprint. |

---

## 5. Inventaris Aset Komponen Eksternal: 9Router AI Gateway (decolua)

| Asset ID | Nama Aset | Kategori / Tipe | Lokasi Relatif | Status | Lisensi / Asal | Deskripsi Singkat |
|:---|:---|:---|:---|:---:|:---:|:---|
| **AST-9R-01** | 9Router Cloned Codebase | Cloned Git Repo | `RE/9ROUTER/00-source-materials/9router/` | INTEGRATION TARGET | MIT (decolua) | Repositori resmi `decolua/9router` versi `v0.5.112`, commit `1fe586bb`, branch `master`. Diisolasi dari Git root. |
| **AST-9R-02** | 9Router Package Manifest | Dependencies | `.../9router/package.json` | INTEGRATION TARGET | MIT (decolua) | Definisi monorepo Next.js 16, React 19, custom server port 20127/20128, dan library pendukung. |
| **AST-9R-03** | 9Router Core Proxy Engine | Node.js Code | `.../9router/open-sse/` | INTEGRATION TARGET | MIT (decolua) | Engine proxy streaming berkecepatan tinggi, format intermediate pivot, adapter 140+ provider, fallback routing. |
| **AST-9R-04** | 9Router Auth & Token Auth | Node.js Code | `.../9router/src/dashboardGuard.js` & `auth.js` | INTEGRATION TARGET | MIT (decolua) | Modul autentikasi bcrypt web dashboard dan token mesin CLI `x-9r-cli-token` (terkait risiko RSK-08). |
| **AST-9R-05** | 9Router SQLite Storage | Node.js Code | `.../9router/src/shared/database.js` | INTEGRATION TARGET | MIT (decolua) | Lapisan penyimpanan SQLite (`sql.js` / `better-sqlite3`) yang menyimpan kredensial dan log di `%USERPROFILE%\.9router`. |
| **AST-9R-06** | 9Router RE Research Suite | Documentation | `RE/9ROUTER/` (14 laporan) | ACTIVE | Internal Research | 14 Laporan investigasi teknis komprehensif mencakup routing, adapters, auth, database, integrasi, dan sintesis. |

---

## 6. Inventaris Pustaka Curated Agent Skills (`.agent/skills/`)

| Asset ID | Nama Skill | Path Kanonikal | Format | Status | Lisensi | Deskripsi & Cakupan |
|:---|:---|:---|:---:|:---:|:---:|:---|
| **AST-SKL-01** | `animate` | `.agent/skills/animate/` | Markdown + Recipes | ACTIVE | MIT | Panduan rekayasa animasi UI tingkat lanjut (<300ms, kurva pegas alami, interruptibility). |
| **AST-SKL-02** | `emil-design-eng` | `.agent/skills/emil-design-eng/` | Markdown | ACTIVE | MIT | Filosofi polish visual antarmuka dan detail mikro interaksi perangkat lunak berkualitas tinggi. |
| **AST-SKL-03** | `ponytail` | `.agent/skills/ponytail/` | Markdown | ACTIVE (Restricted) | MIT | Prinsip lazy senior dev & 7-rung solution ladder untuk implementasi minimal tanpa over-engineering. |
| **AST-SKL-04** | `rea` | `.agent/skills/rea/` | Markdown + 7 Refs | ACTIVE | MIT | Metodologi Reverse Engineering Anything untuk inspeksi native, web, biner, dan sistem tertutup. |
| **AST-SKL-05** | `systematic-debugging` | `.agent/skills/systematic-debugging/` | Markdown | ACTIVE | MIT | Metodologi investigasi akar masalah 4 fase sebelum mengajukan atau mengimplementasikan perbaikan. |
| **AST-SKL-06** | `verification-before-completion` | `.agent/skills/verification-before-completion/` | Markdown | ACTIVE | MIT | Gerbang verifikasi berbasis bukti empiris sebelum mengklaim pekerjaan selesai atau passing. |
| **AST-SKL-07** | `web-design-guidelines` | `.agent/skills/web-design-guidelines/` | Markdown + Cache | ACTIVE | MIT | Audit antarmuka berbasis Web Interface Guidelines (aksesibilitas, tipografi, keyboard focus). |

---

## 7. Catatan Pemeliharaan Aset

1. **Integritas Material Sumber:** Seluruh berkas di `RE/Nusa/00-source-materials/`, `RE/Hermes/00-source-materials/`, dan `RE/9ROUTER/00-source-materials/` bersifat *read-only* bagi alur kerja Hime. Tidak boleh ada modifikasi atau komit pada direktori tersebut.
2. **Kanonikalitas Skill:** Folder `.agents` dipertahankan sebagai Windows Directory Junction ke `.agent` semata-mata untuk kompatibilitas runner antarmuka agen tertentu, namun tidak di-track dalam repositori Git root.
3. **Penyimpanan Kredensial:** Repositori ini tidak menyimpan kredensial aktif atau berkas basis data operasional. Seluruh basis data pengujian dan token disimpan di luar pohon repositori (`%USERPROFILE%\.9router` dan `%APPDATA%\9router`).
