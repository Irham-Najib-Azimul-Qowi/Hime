# Review Standard
**Project:** Hime Assistant  
**Version:** 1.0  

---

## 1. Review Scope

Every change (code, documentation, configuration) must be reviewed before acceptance.

Review may be performed by:
- A human team member.
- An AI agent acting as reviewer (different from the agent that authored the change).
- Self-review (minimum bar when no other reviewer is available — must include evidence).

---

## 2. Review Severity Levels

| Level | Description | Action Required |
|:---|:---|:---|
| **Critical** | Security vulnerabilities, data loss risks, breaking API contracts, production crashes | Must fix before merge |
| **Important** | Architecture violations, missing tests, performance regressions, incorrect logic | Should fix before merge |
| **Suggestion** | Naming improvements, code style, readability, minor optimizations | Fix at author's discretion |
| **Nitpick** | Personal preference, cosmetic | Informational only |

---

## 3. Review Checklist

### Correctness
- [ ] Does the code do what the task specification requires?
- [ ] Are edge cases handled?
- [ ] Are error conditions handled appropriately?

### Security
- [ ] No hardcoded secrets or credentials.
- [ ] Input validation present for external data.
- [ ] No new attack surfaces introduced.

### Architecture
- [ ] Follows domain boundaries.
- [ ] No circular dependencies.
- [ ] No unnecessary coupling.

### Quality
- [ ] Tests exist and pass.
- [ ] No dead code, ghost code, or duplicated logic.
- [ ] Documentation is updated.
- [ ] Naming is clear and consistent.

### Scope
- [ ] Changes are within the approved task scope.
- [ ] No unrelated changes bundled in.

---

## 4. Review Report Template

```markdown
# Code Review Report

**Task:** [Task ID]
**Author:** [Author]
**Reviewer:** [Reviewer]
**Date:** [Date]

## Summary
[Brief description of what was reviewed]

## Findings

| # | Severity | File | Line | Description | Recommendation |
|---|----------|------|------|-------------|----------------|
| 1 | Critical | ... | ... | ... | ... |

## Verdict
- [ ] APPROVED
- [ ] APPROVED WITH CHANGES (list required changes)
- [ ] CHANGES REQUESTED (must resubmit)
- [ ] REJECTED (with reason)

## Notes
[Additional context or recommendations]
```

