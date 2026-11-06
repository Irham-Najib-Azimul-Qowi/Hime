# Code Review Report: [Task ID or PR Title]

- **Task ID:** [TASK-ID]
- **Author:** [Author Name / Agent]
- **Reviewer:** [Reviewer Name / Agent]
- **Date:** YYYY-MM-DD
- **Target Branch:** [main / dev / feature-branch]
- **Commit Reviewed:** [Git Commit Hash]

---

## 1. Executive Summary

[Brief high-level summary of the reviewed changes, overall code quality, and whether the implementation faithfully follows the task specification.]

---

## 2. Review Checklist

| Dimension | Question | Status (PASS / WARN / FAIL) | Notes |
|:---|:---|:---:|:---|
| **Scope** | Are all modifications within the approved task scope? | | |
| **Correctness** | Does the logic fulfill all acceptance criteria? | | |
| **Standards** | Does the code adhere to `CODING_STANDARD.md`? | | |
| **Simplicity** | Is it minimal? Any dead code, ghost code, or over-engineering? | | |
| **Security** | Any hardcoded secrets, unsafe shell calls, or input injection risks? | | |
| **Testing** | Are unit/integration tests present with verifiable assertions? | | |
| **Documentation** | Are docstrings, README, or CHANGELOG updated? | | |

---

## 3. Detailed Findings

| # | Severity (Critical / Important / Suggestion) | File Path & Line | Description of Finding | Required Action |
|:---:|:---:|:---|:---|:---|
| 1 | | `path/to/file.ext#L10` | | |
| 2 | | `path/to/file.ext#L45` | | |

---

## 4. Final Verdict

- [ ] **APPROVED** — Ready to merge without further changes.
- [ ] **APPROVED WITH MINOR SUGGESTIONS** — Author may merge after addressing non-blocking notes.
- [ ] **CHANGES REQUESTED** — Critical or Important issues must be fixed and re-reviewed.
- [ ] **REJECTED** — Severe violation of architectural principles or security policy.

