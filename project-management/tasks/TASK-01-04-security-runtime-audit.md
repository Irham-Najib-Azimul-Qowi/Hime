# Task Specification: TASK-01-04 — Security & Runtime Environment Audit

---

## 1. Metadata

- **Task ID:** TASK-01-04
- **Phase:** Phase 01 — Discovery & Environment Audit
- **Title:** Security & Runtime Environment Audit, Threat Modeling & Security Baseline
- **Priority:** CRITICAL
- **Owner:** Senior Security Architect & AI Infrastructure Specialist
- **Reviewer:** Project Owner / User
- **Status:** **ACCEPTED (Qualified Acceptance)**
- **Dependencies:** TASK-01-01 (Accepted), TASK-01-02 (Accepted), TASK-01-03 (Accepted)
- **Commit Reference:** Pending Review Approval

---

## 2. Description

Lakukan audit keamanan dan lingkungan runtime secara menyeluruh berbasis bukti terhadap ekosistem Hime Assistant, mencakup kontrol keamanan Windows host (privileges, UAC, integrity level, NTFS ACLs, DPAPI, Credential Manager, Named Pipes, Defender, Firewall), arsitektur rahasia & kredensial pihak ketiga (9Router SQLite plaintext CWE-312 RSK-03, `x-9r-cli-token` RSK-08, Hermes config & env scrubbing), keamanan jaringan (paparan port, binding loopback vs 0.0.0.0 RSK-12, bypass localhost RSK-10, tunnels), IPC trust boundaries, keamanan eksekusi tool & prompt injection (RSK-11), sistem berkas, logging, pembaruan aplikasi, serta penyusunan Threat Model formal (STRIDE) dan Baseline Keamanan Rekayasa (SEC-01 s/d SEC-10).

---

## 3. Scope Boundaries

### In Scope
- Audit lingkungan keamanan Windows 11 host (non-invasif, tanpa eskalasi hak istimewa).
- Evaluasi penanganan rahasia dan kredensial (Hermes Agent, 9Router, Hime Core, Hime Desktop).
- Audit paparan jaringan dan rute autentikasi 9Router dan Hermes.
- Analisis batas kepercayaan dan protokol IPC (Windows Named Pipes, Localhost HTTP, WebSocket).
- Audit keamanan proses, pengawasan daur hidup proses anak, dan risiko prompt injection.
- Evaluasi keamanan sistem berkas, traversal path, dan penyimpanan percakapan.
- Perumusan persyaratan jejak audit dan pembersihan log dari data sensitif.
- Evaluasi keamanan installer dan mekanisme update (Authenticode, checksum pinning).
- Penyusunan STRIDE Threat Model komprehensif (`THREAT-MODEL.md`).
- Perumusan Baseline Keamanan Rekayasa (`SECURITY-BASELINE.md`).
- Rekonsiliasi risiko lama (RSK-01 s/d RSK-09) dan penambahan risiko terverifikasi baru (RSK-10, RSK-11, RSK-12).

### Out of Scope
- Mengubah kode sumber aplikasi Hermes Agent atau 9Router.
- Menginstal atau memperbarui dependensi host atau paket pihak ketiga.
- Mengubah pengaturan keamanan Windows atau aturan Windows Defender/Firewall.
- Mengubah kredensial, token rahasia, atau mereset basis data.
- Menjalankan uji penetrasi destruktif.
- Melakukan operasi `git commit` atau `git push`.
- Memulai eksekusi TASK-01-05 sebelum persetujuan pengguna.

---

## 4. Acceptance Criteria & Verification Matrix

- [x] **AC-1 (Windows Security Environment):** Konteks pengguna (`XATOSHI\Satoshi`, Medium integrity, filtered token UAC), izin direktori NTFS, DPAPI, Named Pipes DACL, dan Credential Manager dievaluasi dengan bukti empiris.
- [x] **AC-2 (Secret & Credential Handling):** Penyimpanan rahasia 9Router (RSK-03, RSK-04, RSK-08) dan Hermes Agent dianalisis baris demi baris dari kode sumber aktual.
- [x] **AC-3 (Network Exposure Assessed):** Paparan port dievaluasi; kelemahan default host binding `0.0.0.0` (RSK-12) didokumentasikan (`[VERIFIED]`) dengan rencana mitigasi runtime socket verification IPv4/IPv6 & PID matching (`[PLANNED CONTROL]`, efektivitas `[NOT YET VERIFIED]`); bypass localhost `/v1/*` (RSK-10) ditandai sebagai `IMPLEMENTATION BLOCKER & PROPOSED RESIDUAL RISK — OWNER DECISION REQUIRED` (upstream modification / process isolation required).
- [x] **AC-4 (IPC Trust Boundaries):** 5 Batas IPC dianalisis; Windows Named Pipes dengan `PipeSecurity` DACL dan token handshake direkomendasikan.
- [x] **AC-5 (Process & Runtime Security):** Daur hidup proses dan risiko eksekusi tool Hermes (RSK-11) didokumentasikan bersama solusi Windows Job Objects.
- [x] **AC-6 (Filesystem Security):** Kontrol anti-traversal path (`canonicalize`), deny-list folder sensitif, dan operasi write-replace atomik dirumuskan.
- [x] **AC-7 (Logging & Audit Trail):** Ketentuan pembersihan regex rahasia dan format log terstruktur `AUDIT.jsonl` dirumuskan.
- [x] **AC-8 (Installer & Update Security):** Persyaratan Authenticode code signing, pinning SHA-256, dan mitigasi runtime download (RSK-09) ditentukan.
- [x] **AC-9 (STRIDE Threat Model Produced):** Threat model terstruktur diterbitkan di `project-management/security/THREAT-MODEL.md` mencakup 10 skenario serangan wajib + RSK-10 & RSK-12.
- [x] **AC-10 (Security Baseline Actionable):** Baseline 10 kontrol rekayasa (SEC-01 s/d SEC-10) diterbitkan di `project-management/security/SECURITY-BASELINE.md` dengan kualifikasi realistis DPAPI, ACL, EFS, zeroization, serta kualifikasi teknis DACL akses memori satu pengguna dan penegasan `PROPOSED RESIDUAL RISK — OWNER DECISION REQUIRED`.
- [x] **AC-11 (Risk Reconciliation):** Register risiko diperbarui secara harmonis mencakup RSK-01 s/d RSK-12 tanpa duplikasi.
- [x] **AC-12 (Evidence-Based Claims):** Setiap temuan utama memiliki label bukti (`VERIFIED`, `RECOMMENDATION`, `INFERRED`).
- [x] **AC-13 (Git Diff Clean):** `git diff --check` bersih tanpa error whitespace.
- [x] **AC-14 (Working Tree Documented):** Seluruh berkas baru dan modifikasi dicatat secara transparan.
- [x] **AC-15 (Zero Host Modification):** Tidak ada kode sumber, dependensi, atau konfigurasi keamanan host yang diubah.

---

## 5. Verification Evidence Summary

- **Laporan Audit Utama:** [`project-management/reviews/TASK-01-04-SECURITY-RUNTIME-AUDIT.md`](file:///x:/folder_ai/Hime/project-management/reviews/TASK-01-04-SECURITY-RUNTIME-AUDIT.md)
- **Model Ancaman STRIDE:** [`project-management/security/THREAT-MODEL.md`](file:///x:/folder_ai/Hime/project-management/security/THREAT-MODEL.md)
- **Baseline Keamanan:** [`project-management/security/SECURITY-BASELINE.md`](file:///x:/folder_ai/Hime/project-management/security/SECURITY-BASELINE.md)
- **Register Risiko Terbarui:** [`project-management/risks/RISK_REGISTER.md`](file:///x:/folder_ai/Hime/project-management/risks/RISK_REGISTER.md) (Penambahan RSK-10, RSK-11, RSK-12)
