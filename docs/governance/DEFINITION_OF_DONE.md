# Definition of Done
**Project:** Hime Assistant  
**Version:** 1.0  

---

## Universal Definition of Done

A task transitions to DONE only when ALL of the following are satisfied:

### 1. Acceptance Criteria Met
- [ ] Every acceptance criterion from the task specification has been addressed.
- [ ] Each criterion has associated evidence (test output, screenshot, command result).

### 2. Code Quality
- [ ] Code follows the project coding standard.
- [ ] No compiler warnings or linting errors introduced.
- [ ] No dead code, ghost code, or unused imports.
- [ ] Functions and modules have appropriate documentation.

### 3. Tests
- [ ] Relevant tests exist and pass (evidence provided).
- [ ] No existing tests broken by the change.
- [ ] Test coverage is adequate for the change scope.

### 4. Documentation
- [ ] Code documentation updated (doc comments, README).
- [ ] Architecture documentation updated if design changed.
- [ ] CHANGELOG updated for user-visible changes.

### 5. Security
- [ ] No credentials, secrets, or tokens in code.
- [ ] No new security risks introduced without documented mitigation.

### 6. Review
- [ ] Change has been reviewed (self-review minimum).
- [ ] All critical and important review findings addressed.

### 7. Integration
- [ ] Change does not break existing functionality.
- [ ] Change is compatible with the target branch.
- [ ] Dependencies are properly declared and pinned.

### 8. Traceability
- [ ] Commit references the task ID.
- [ ] All files changed are listed in the completion report.
- [ ] Completion report includes evidence for every claim.

---

## Phase-Level Definition of Done

A project phase (e.g., Phase 0, Phase 1) is DONE when:

- [ ] All tasks in the phase are individually DONE.
- [ ] Phase acceptance report written and approved.
- [ ] No blocking issues remain.
- [ ] Dependencies for the next phase are satisfied.
- [ ] Project owner has explicitly approved phase completion.

