# Task Specification: TASK-00-02 — Git Initialization, Repository Protection & Toolchain Verification

---

## 1. Metadata

- **Task ID:** TASK-00-02
- **Phase:** PHASE 0 — Workspace Infrastructure & Governance
- **Title:** Workspace Git Initialization, Root .gitignore, and Toolchain Verification
- **Priority:** CRITICAL
- **Owner:** Senior DevOps Engineer & Repository Specialist (AI Agent)
- **Reviewer:** Project Owner / User
- **Status:** **READY FOR INDEPENDENT REVIEW**
- **Dependencies:** TASK-00-01 (Workspace Setup & Governance)
- **Commit Reference:** `fec28f4` (Baseline commit on branch `main`)

---

## 2. Description

Initialize the root Git repository for the Hime project at `X:\folder_ai\Hime`, configure a comprehensive `.gitignore` protecting credentials, local SQLite databases, external upstream repositories, and build artifacts, unlink duplicate junctions, verify skill compatibility, and establish toolchain readiness without blocking architecture research.

---

## 3. Scope Boundaries

### In Scope
- Running `git init -b main` at `X:\folder_ai\Hime`.
- Configuring remote `origin` to `https://github.com/Irham-Najib-Azimul-Qowi/Hime.git` (no push).
- Authoring production-grade root `.gitignore` covering OS, IDE, Node.js, Rust, Python, Android, secrets, SQLite, logs, and external source materials.
- Unlinking duplicate REA junction (`.agent/skills/reverse-engineer-anything`) safely without deleting canonical skill data.
- Auditing skill registry interaction between `ponytail`, `systematic-debugging`, and `verification-before-completion`.
- Performing toolchain audit and classifying readiness by project phase (documenting Rust as non-blocker for Phase 01).
- Reclassifying risk register (`RISK_REGISTER.md`) with observable evidence and mitigation strategies.
- Creating baseline local commit `chore: initialize Hime project governance baseline`.

### Out of Scope
- Performing `git push` or modifying remote branch history.
- Auto-installing OS-level toolchains or packages.
- Modifying internal commits or code of nested repositories (`hermes-agent`, `9router`).
- Modifying 9Router's internal SQLite database.

---

## 4. Acceptance Criteria & Verification Matrix

- [x] **AC-1 (Pre-Flight Audit):** Workspace scanned for hidden `.git`, secrets, databases, large files; pre-flight report authored at [`docs/reports/REPOSITORY_PREFLIGHT_AUDIT.md`](file:///x:/folder_ai/Hime/docs/reports/REPOSITORY_PREFLIGHT_AUDIT.md).
- [x] **AC-2 (Root Git Init):** Repositori Git diinisialisasi pada root dengan default branch `main`; remote `origin` diset ke URL pengguna; nested repositori dipertahankan utuh.
- [x] **AC-3 (Root .gitignore):** File `.gitignore` dibuat komprehensif, mengecualikan nested repositori dan biner installer tanpa mengabaikan laporan riset `RE/`.
- [x] **AC-4 (REA Junction Resolution):** Junction duplikat `.agent/skills/reverse-engineer-anything` di-unlink dengan aman; folder kanonikal `.agent/skills/rea/` terbukti 100% utuh.
- [x] **AC-5 (Skill Verification):** Interaksi `ponytail`, `systematic-debugging`, dan `verification-before-completion` dianalisis tanpa konflik; matriks pemanggilan didokumentasikan di `SKILL_REGISTRY.md`.
- [x] **AC-6 (Toolchain Verification):** Status 12 tools diperiksa; klasifikasi fase (Required Now vs Phase 04/06 vs Phase 07/11) didokumentasikan; panduan instalasi `rustup` disediakan; Rust ditegaskan bukan blocker Phase 01.
- [x] **AC-7 (Security Risk Reclassification):** Seluruh 7 risiko dievaluasi ulang dengan label bukti terstandar di `RISK_REGISTER.md`.
- [x] **AC-8 (Initial Baseline Commit):** Staged diff diperiksa bersih; commit lokal `chore: initialize Hime project governance baseline` dibuat.
- [x] **AC-9 (Documentation Update):** `PROJECT_STATUS.md`, `RISK_REGISTER.md`, `DECISION_LOG.md`, `SKILL_REGISTRY.md` diperbarui konsisten.

---

## 5. Verification Evidence

```powershell
# Git Init & Remote Verification
Initialized empty Git repository in X:/folder_ai/Hime/.git/
origin  https://github.com/Irham-Najib-Azimul-Qowi/Hime.git (fetch)
origin  https://github.com/Irham-Najib-Azimul-Qowi/Hime.git (push)

# Junction Unlink Verification
cmd /c rmdir "x:\folder_ai\Hime\.agent\skills\reverse-engineer-anything" -> ExitCode: 0
Canonical rea intact: 37,433 bytes SKILL.md + 7 reference files.

# Clean Untracked Status
git status -u shows only governance docs, templates, research reports, and .gitkeeps.
```
