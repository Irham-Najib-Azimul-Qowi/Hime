# Development Workflow
**Project:** Hime Assistant  
**Version:** 1.0  

---

## 1. Task Lifecycle

```text
BACKLOG → READY → IN PROGRESS → IMPLEMENTED → SELF-VERIFIED → IN REVIEW → QA VERIFIED → ACCEPTED → DONE
```

### State Definitions

| State | Entry Criteria | Responsible |
|:---|:---|:---|
| BACKLOG | Idea or requirement identified | Anyone |
| READY | Task spec written, dependencies met, acceptance criteria defined | Task Author |
| IN PROGRESS | Work actively being performed | Assigned Owner |
| IMPLEMENTED | Code/docs written, self-review complete | Owner |
| SELF-VERIFIED | Owner has tested and verified against acceptance criteria with evidence | Owner |
| IN REVIEW | Submitted for peer/agent review | Reviewer |
| QA VERIFIED | Reviewer confirms quality and correctness | Reviewer |
| ACCEPTED | Project owner approves the deliverable | Project Owner |
| DONE | Merged/deployed, no further action needed | — |

---

## 2. Branch Strategy

Once Git is initialized:

- `main` — stable, reviewed code only.
- `dev` — integration branch for work in progress.
- `feature/<task-id>-<short-name>` — individual task branches.
- `fix/<task-id>-<short-name>` — bug fix branches.
- `docs/<topic>` — documentation-only branches.

---

## 3. Commit Convention

```text
<type>(<scope>): <description>

[optional body]

[optional footer]
```

Types: `feat`, `fix`, `docs`, `refactor`, `test`, `chore`, `security`  
Scope: component name (e.g., `desktop`, `core`, `hermes`, `9router`, `governance`)

---

## 4. Agent Workflow

AI agents follow this procedure:

1. **Read task specification** — understand scope, acceptance criteria, and constraints.
2. **Check workspace state** — `git status`, existing files, running processes.
3. **Plan changes** — identify files to create, modify, or delete.
4. **Implement** — make changes within scope.
5. **Self-verify** — run tests, check outputs, verify file creation.
6. **Report** — provide completion report with evidence.
7. **Wait for review** — do not proceed to next task without approval.

---

## 5. Review Checklist

- [ ] Code follows project standards.
- [ ] No unnecessary changes outside scope.
- [ ] Tests pass with evidence.
- [ ] Documentation updated.
- [ ] No security issues introduced.
- [ ] No breaking changes to existing APIs.
- [ ] Commit messages follow convention.

