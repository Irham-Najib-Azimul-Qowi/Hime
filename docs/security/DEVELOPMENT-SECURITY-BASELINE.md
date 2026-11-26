# Development Security Baseline & Secrets Management Policy

- **Document ID:** HIME-SEC-05-01
- **Task ID:** TASK-05-05 (Development Security Baseline & Secrets Git-Exclusion)
- **Phase:** Phase 05 — Development Environment & Implementation Readiness
- **Authority:** System Architect & Technical Lead
- **Execution Role:** Senior Security Architect & DevOps Engineer
- **Status:** **APPROVED SECURITY BASELINE — IB-01 & IB-02 REMAIN OPEN**
- **Date:** 2026-11-10

---

## 1. Executive Summary & Security Philosophy

Kebijakan ini menetapkan batas-batas keamanan mutlak (*Security Baseline*) untuk seluruh alur kerja pengembangan, scripting otomatisasi, pengujian lokal, dan pengelolaan berkas pada proyek Hime.

### Prinsip Keamanan Utama:
1. **Zero Secret in Version Control:** Kunci API komersial, bearer token, sertifikat privat, file `.env` lokal, atau basis data lokal **DILARANG KERAS MASUK KE GIT**.
2. **Larangan Penggunaan Kunci Komersial pada Development:** Seluruh pengujian dan pengembangan lokal wajib menggunakan *mock responses* atau *dummy keys* (`hime-managed-key`).
3. **Pemisahan Data Transien & Residual:** Data runtime dipisahkan ke `%LOCALAPPDATA%\Hime\dev\` dan diproteksi dengan DACL Current User SID.
4. **Fail-Closed Scripting:** Skrip build, test, dan dev wajib berhenti seketika (*fail-closed*) jika menemukan konfigurasi yang tidak aman atau hilangnya batas isolasi.
5. **Pemblokir Implementasi IB-01 & IB-02:** Phase 05 **TIDAK MENGUBAH STATUS** IB-01 maupun IB-02. Keduanya tetap berstatus **`OPEN IMPLEMENTATION BLOCKERS`**.

---

## 2. Aturan Git Exclusion & Verifikasi Pra-Commit

### Daftar Pola Berkas yang Dikecualikan Mutlak:
- **Environment & Kredensial:** `.env`, `.env.local`, `.env.*.local`, `*.pem`, `*.key`, `*.cert`, `*.pfx`, `*.token`, `*session_tickets*`.
- **Basis Data Runtime:** `*.db`, `*.db-wal`, `*.db-shm`, `*.sqlite`, `*.sqlite3`, `.9router/`, `.hermes/`.
- **Artefak Kompilasi:** `target/`, `node_modules/`, `dist/`, `out/`, `*.pdb`.
- **Log & Crash Dumps:** `*.log`, `*.dmp`, `*.mdmp`, `transcript*.jsonl`, `scratch/`.
- **Staging & Temporary Files:** `.hime_staging/`, `dev/`, `*.tmp`.

---

## 3. Kebijakan Output Skrip & Larangan Logging Rahasia

1. **Larangan Mencetak Rahasia ke Terminal:**
   - Skrip PowerShell (`check-env.ps1`, `build.ps1`, `test.ps1`, `dev.ps1`) **DILARANG** mencetak variabel yang memuat token, password, atau master key ke `stdout` atau `stderr`.
2. **Redaksi Otomatis:**
   - Jika skrip mendeteksi string yang berformat `sk-[a-zA-Z0-9]{20,}` atau `Bearer ...`, string tersebut wajib disensor menjadi `[REDACTED_SECRET]`.
3. **Izin Berkas Lokal (Least Privilege):**
   - Direktori `%LOCALAPPDATA%\Hime\dev\` dibuat secara eksklusif dengan izin baca/tulis hanya untuk User SID pengguna interaktif.

---

## 4. Status Pemblokir Implementasi (IB-01 & IB-02)

- **IB-01 (9Router Credential Storage):** Tetap berstatus **OPEN**. Tidak ada kunci API komersial yang disimpan di file `.env` atau database 9Router lokal.
- **IB-02 (9Router Localhost Auth Bypass):** Tetap berstatus **OPEN**. Seluruh skrip pengembangan lokal hanya mengikat 9Router pada loopback `127.0.0.1` dan menolak eksposur ke interface jaringan publik `0.0.0.0`.

