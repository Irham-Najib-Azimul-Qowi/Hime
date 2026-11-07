# Task Specification: TASK-00-01 — Initial Workspace Setup & Engineering Governance

---

## 1. Metadata

- **Task ID:** TASK-00-01
- **Phase:** Phase 00 — Project Governance
- **Title:** Initial Workspace Setup & Engineering Governance Baseline
- **Priority:** CRITICAL
- **Owner:** Senior Engineering Infrastructure Architect (AI Agent)
- **Reviewer:** Project Owner / User
- **Status:** **COMPLETED**
- **Dependencies:** None
- **Commit Reference:** `8cf0070` (Baseline commit on branch `main`)

---

## 2. Description

Establish the permanent development foundation, engineering rules, security guidelines, task tracking systems, and curated agent skills library for the Hime Assistant project before any product feature code is written.

---

## 3. Scope Boundaries

### In Scope
- Comprehensive environment audit (`ENVIRONMENT_AUDIT.md`) classifying host OS, runtimes, package managers, agent platform, and existing assets.
- Creation of authoritative agent engineering instructions (`AGENTS.md`) covering project identity, evidence-first standards, minimal implementation, architecture governance, security, testing, and change management.
- Creation of core engineering governance policies in `docs/governance/`:
  - `DEVELOPMENT_WORKFLOW.md`
  - `CODING_STANDARD.md`
  - `REVIEW_STANDARD.md`
  - `DEFINITION_OF_DONE.md`
  - `AI_AGENT_POLICY.md`
  - `SECURITY_POLICY.md`
  - `SKILL_POLICY.md`
  - `CHANGE_MANAGEMENT.md`
- Governance and management templates in `docs/` and `project-management/`.
- Curated skill library evaluation in `docs/skills/SKILL_REGISTRY.md` and installation of approved skills into `.agent/skills/`.
- Project tracking baseline (`BACKLOG.md`, `RISK_REGISTER.md`, `DECISION_LOG.md`).

### Out of Scope
- Implementing Hime product features (Tauri desktop code, Android code).
- Modifying Hermes Agent or 9Router source code.
- Modifying user global configurations or installing host software without approval.

---

## 4. Acceptance Criteria

- [x] **AC-1:** Environment audit conducted with every claim classified (`[VERIFIED]`, `[OBSERVED]`, `[INFERRED]`, `[UNKNOWN]`).
- [x] **AC-2:** `AGENTS.md` created at workspace root with all 8 mandatory sections.
- [x] **AC-3:** Complete governance policy suite and standard templates authored.
- [x] **AC-4:** External agent skills researched, evaluated in `SKILL_REGISTRY.md`, and approved skills installed and verified in `.agent/skills/`.
- [x] **AC-5:** Task tracking, risk register, and backlog established.

---

## 5. Verification Evidence

- Environment inspection commands completed (Node.js 24.18, Python 3.13, uv 0.12, Android SDK verified; Rust and Git absence documented).
- All governance files and templates created and validated on disk.
- Approved skills installed into `.agent/skills/` (`animate`, `emil-design-eng`, `ponytail`, `systematic-debugging`, `verification-before-completion`, `web-design-guidelines`) with offline cache.
- Symlink `.agents/skills` reflects all installed skills.

