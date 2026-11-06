# Task Specification: [TASK-ID] — [Task Title]

---

## 1. Metadata

- **Task ID:** TASK-[PHASE]-[NUMBER] (e.g., TASK-00-01)
- **Phase:** [Phase ID & Name]
- **Title:** [Brief descriptive title]
- **Priority:** [CRITICAL | HIGH | MEDIUM | LOW]
- **Owner:** [Assigned Engineer / Agent]
- **Reviewer:** [Assigned Reviewer / Agent]
- **Status:** BACKLOG
- **Dependencies:** [List predecessor Task IDs or "None"]
- **Commit Reference:** [Git commit hash when completed]

---

## 2. Description

[Detailed narrative explaining what needs to be accomplished and why it is necessary.]

---

## 3. Scope Boundaries

### In Scope
- [File/module 1]
- [File/module 2]
- [Specific capability to be implemented]

### Out of Scope
- [Explicitly excluded modules or functionalities]
- [Features deferred to later tasks]

---

## 4. Acceptance Criteria

- [ ] **AC-1:** [Concrete, testable criterion]
- [ ] **AC-2:** [Concrete, testable criterion]
- [ ] **AC-3:** [All relevant tests pass without regressions]
- [ ] **AC-4:** [Documentation and CHANGELOG updated if applicable]

---

## 5. Risk Assessment

| Risk Description | Severity | Likelihood | Mitigation Strategy |
|:---|:---:|:---:|:---|
| [Identified technical or operational risk] | [High/Med/Low] | [High/Med/Low] | [Concrete countermeasure] |

---

## 6. Implementation Notes & Workflow Log

### Status Transitions
- `BACKLOG` → `READY`: [Date, Author]
- `READY` → `IN PROGRESS`: [Date, Owner]
- `IN PROGRESS` → `IMPLEMENTED`: [Date, Owner]
- `IMPLEMENTED` → `SELF-VERIFIED`: [Date, Owner]
- `SELF-VERIFIED` → `IN REVIEW`: [Date, Owner]
- `IN REVIEW` → `QA VERIFIED`: [Date, Reviewer]
- `QA VERIFIED` → `ACCEPTED`: [Date, Project Owner]
- `ACCEPTED` → `DONE`: [Date, Project Owner]

### Evidence of Completion
```text
[Paste terminal test outputs, build outputs, or verification commands here]
```

### Notes & Discovered Insights
[Record edge cases, deviations, or follow-up recommendations here]

