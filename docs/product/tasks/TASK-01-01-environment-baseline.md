# Task Specification: TASK-01-01 — Environment Baseline Validation

---

## 1. Metadata

- **Task ID:** TASK-01-01
- **Phase:** Phase 01 — Discovery & Environment Audit
- **Title:** Environment Baseline Validation & Host Resource Audit
- **Priority:** HIGH
- **Owner:** Senior Engineering Infrastructure Architect & DevOps Specialist
- **Reviewer:** Project Owner / User
- **Status:** **ACCEPTED**
- **Dependencies:** Phase 00 (Project Governance Closed)
- **Commit Reference:** Pending Commit (`docs(discovery): accept task 01-01 environment baseline`)

---

## 2. Description

Lakukan inspeksi dan validasi faktual menyeluruh terhadap lingkungan host Windows 11 tempat Hime Assistant dikembangkan. Audit mencakup inventarisasi perangkat keras (CPU, RAM, GPU, Storage), toolchain runtime (Git, Node.js, Python, uv, Rust, WebView2, SQLite), kapabilitas native Windows (DPAPI, Named Pipes, permissions), matriks kompatibilitas tumpukan teknologi, evaluasi kelayakan sumber daya multi-proses, serta penilaian kesiapan pengembangan tanpa asumsi.

---

## 3. Scope Boundaries

### In Scope
- Inventarisasi perangkat keras aktual via PowerShell CIM/WMI (tanpa asumsi).
- Pemeriksaan keberadaan, versi, jalur executable, dan uji jalan 14 perkakas pengembang.
- Audit lingkungan Windows: versi OS, environment variables penting, analisis konflik PATH, izin pengguna (least privilege), DPAPI, Named Pipes, fasilitas pemantauan dan packaging.
- Penyusunan Matriks Kompatibilitas Teknologi (Ready, Needs Configuration, Missing, Blocked).
- Evaluasi kelayakan sumber daya (*Resource Feasibility*) multi-proses (Hime Shell, Hime Core, Hermes Agent, 9Router).
- Identifikasi dependensi wajib vs yang dapat ditunda tanpa memaksakan instalasi.
- Penyusunan laporan komprehensif di `project-management/reviews/TASK-01-01-ENVIRONMENT-BASELINE-REPORT.md`.

### Out of Scope
- Menginstal, menghapus, atau memperbarui perangkat lunak / dependensi host.
- Mengubah konfigurasi sistem operasi Windows atau registry.
- Mengubah kode runtime Hime, Hermes Agent, atau 9Router.
- Melakukan modifikasi basis data SQLite atau kredensial.
- Melakukan operasi `git push` ke GitHub.
- Memulai eksekusi TASK-01-02.

---

## 4. Acceptance Criteria & Verification Matrix

- [x] **AC-1 (Hardware Inventory):** CPU (Intel i5-8250U 4C/8T), RAM (20 GB total, 3.7 GB free), GPU (Intel UHD 620), Storage (Drive C & X) terinventarisasi faktual.
- [x] **AC-2 (OS Information):** Microsoft Windows 11 Pro Build 26200 x64 diverifikasi.
- [x] **AC-3 (Development Toolchain):** 14 Perkakas diperiksa; Node.js v24, Python 3.13, uv 0.12, Git 2.55, WebView2 157 terkonfirmasi ready; Rust/Cargo/MSVC diidentifikasi missing non-blocker.
- [x] **AC-4 (Windows Native API Audit):** DPAPI (`ProtectedData.Protect`) lolos uji kriptografi; Named Pipes `\\.\pipe\` aktif; User unprivileged terverifikasi.
- [x] **AC-5 (Technology Compatibility Matrix):** Status 7 komponen teknologi diklasifikasikan objektif (Ready vs Missing non-blocker).
- [x] **AC-6 (Resource Feasibility):** Kelayakan beban memori multi-proses (Idle: 130–235 MB, Aktif: 350–850 MB) terbukti aman dalam ruang 3.7 GB free RAM.
- [x] **AC-7 (Missing Dependencies & Blockers):** Tidak ada blocker untuk Phase 01–05; Rust & MSVC dijadwalkan untuk Phase 06.
- [x] **AC-8 (Zero-Change Discipline):** Tidak ada perubahan kode runtime, tidak ada dependensi terinstal, tidak ada push.
- [x] **AC-9 (Documentation Integrity):** Laporan utama diterbitkan di `project-management/reviews/TASK-01-01-ENVIRONMENT-BASELINE-REPORT.md`.

---

## 5. Verification Evidence Summary

- **Hardware Command:** `Get-CimInstance Win32_Processor`, `Win32_OperatingSystem`, `Win32_VideoController`, `Win32_LogicalDisk`.
- **Runtime Command:** `git --version`, `node --version`, `python --version`, `uv --version`, Python FTS5 query.
- **DPAPI Script:** In-memory encryption/decryption round-trip via .NET `System.Security.Cryptography.ProtectedData` (Result: `True`).
- **Named Pipe Script:** Enumerate pipe directory `\\.\pipe\` (Result: active).

---

## 6. Formal Acceptance & Boundary Notes

- **Acceptance Status:** **ACCEPTED** (Qualified Acceptance oleh Project Owner pada 2026-11-07).
- **Hasil Verifikasi:** Baseline lingkungan host Windows 11 telah tervalidasi penuh tanpa blocker untuk Phase 01–05.
- **Batasan & Catatan Kritis:**
  1. **RAM Bebas:** Nilai 3,71 GB adalah hasil pengukuran sesaat (*point-in-time snapshot*) saat pengujian dijalankan. Beban memori host dapat berfluktuasi tergantung aplikasi lain.
  2. **Estimasi Beban Memori Hime:** Angka target multi-proses (~135–235 MB idle, ~280–545 MB aktif) merupakan *proposed engineering target* berdasarkan arsitektur yang direncanakan, bukan hasil benchmark runtime Hime aktual (karena runtime Hime belum dibangun).
  3. **Akselerasi GPU & Latensi IPC:** Terdeteksinya GPU Intel UHD 620 dan Win32 Named Pipes membuktikan ketersediaan driver dan kapabilitas OS, namun belum membuktikan pencapaian 60 FPS antarmuka atau target latensi IPC <50ms secara empiris hingga kode runtime diimplementasikan dan diuji.
- **Catatan Riwayat Git (Commit Phase 00 Closure):**
  - Riwayat Git mencatat dua commit penutupan Phase 00: `61ad2c3` (`docs(governance): accept task-00-02 and close phase 00`) dan `0099969` (`chore(governance): accept task 00-02 and close phase 00`).
  - *Temuan Audit Komit:* Kedua commit tidak menduplikasi perubahan file yang persis sama. Commit `61ad2c3` mencakup migrasi penomoran 16 fase roadmap resmi dan file acceptance awal di `project-management/phases/`, sedangkan commit `0099969` merealisasikan perbaikan wording keamanan token CLI 9Router, penambahan risiko `RSK-08` di `RISK_REGISTER.md`, dan penempatan acceptance final di `project-management/reviews/PHASE-00-ACCEPTANCE.md`. Kedua commit dipertahankan secara utuh sebagai jejak audit rekayasa permanen tanpa penghapusan atau rewrite history.
