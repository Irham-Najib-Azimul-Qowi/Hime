# Change Management Policy
**Project:** Hime Assistant  
**Version:** 1.0  
**Effective Date:** 2026-11-06  

---

## 1. Scope & Objective

This policy governs how modifications to code, configuration, architecture, and project documentation are proposed, implemented, verified, and merged. The goal is to ensure predictability, traceability, and prevent breaking changes or unintentional code overwrites.

---

## 2. Change Classification

| Change Class | Description | Required Approvals | Required Verification |
|:---|:---|:---|:---|
| **Class 1: Architectural** | Cross-component interfaces (Hime ↔ Hermes ↔ 9Router), new dependencies, database schema changes | Project Owner + ADR | Contract tests, security review |
| **Class 2: Feature / Component** | New UI widgets, tool implementations, module additions | Code Reviewer | Unit tests, integration tests, DoD checklist |
| **Class 3: Bug Fix / Refactor** | Code corrections, performance optimizations, non-breaking refactoring | Peer / Self-review with evidence | Regression tests, before/after metrics |
| **Class 4: Documentation / Chore** | Governance updates, typos, task documentation, skill additions | Self-review | Linters, link checks |

---

## 3. Pre-Change Verification Checklist

Before writing code or editing files, an agent or engineer MUST:
1. `git status` — Ensure no uncommitted or untracked changes from other sessions exist.
2. Verify active branch — Confirm work is being performed on the proper task or feature branch.
3. Review Task Specification — Confirm the file paths to be touched fall strictly within the task's stated scope.
4. Impact Assessment — If modifying shared interfaces or utilities, inspect all dependent callers before proceeding.

---

## 4. Post-Change Verification & Reporting

After implementing changes, an agent or engineer MUST:
1. Run local test suites, builds, or typecheckers and capture the terminal output as evidence.
2. Perform a self-review of `git diff` to ensure no accidental whitespace noise, stray logs, or dead code were added.
3. Record all modified, created, or deleted files in the completion report.
4. If the change alters architectural assumptions or system contracts, author an Architecture Decision Record (ADR) in `docs/adr/`.

