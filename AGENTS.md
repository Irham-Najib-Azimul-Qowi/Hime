# Hime — Agent Engineering Instructions

> This document serves as the permanent, authoritative set of engineering instructions
> for all AI coding agents working on the Hime project. Every agent session must read
> and follow these rules before making any changes.

---

## A. Project Identity

### Vision

Hime is a personal AI ecosystem spanning desktop (Windows) and mobile (Android) that
integrates a floating Dynamic Island interface with an autonomous AI agent runtime.

### Core Components

| Component | Technology | Status |
|:---|:---|:---|
| **Hime Desktop** | Tauri 2.0 (Rust + WebView) | Planned — reimplements Nusa Dynamic Island |
| **Hime Mobile** | Android (Kotlin) | Planned |
| **Hime Core** | Shared logic layer | Planned |
| **Hermes Agent** | Python runtime (NousResearch) | External — integrate, do not rebuild |
| **9Router** | Node.js AI gateway (decolua) | External — integrate, do not rebuild |
| **Device Management** | Cross-device sync & remote execution | Planned |
| **Specialist Agents** | Task-specific AI agents | Planned |

### Agreed Architectural Decisions

1. Nusa source code is unavailable. Hime Desktop is a **reimplementation**, not a fork.
2. Hermes Agent and 9Router are **external dependencies**. They will be audited, configured, and integrated — not rebuilt from scratch.
3. Hime Desktop uses **Tauri 2.0** (Rust native shell + web frontend).
4. 9Router operates as a **Managed Background Service** using the **Dual-Control Plane** topology (see RE/9ROUTER/13-HIME-INTEGRATION-FEASIBILITY.md).
5. Hermes connects to 9Router via `OPENAI_BASE_URL=http://127.0.0.1:20128/v1`.

### Agent Scope Boundaries

- Agents work within the boundaries of their assigned task.
- Agents do NOT make architectural decisions that affect the overall system without explicit approval.
- Agents do NOT modify external dependency source code (Hermes, 9Router) without a documented reason and approval.
- Agents do NOT create competing implementations when a component already exists.

---

## B. Evidence-First Engineering

### Verify Before Acting

- **Check the repository** before assuming something doesn't exist.
- **Read existing code** before writing new code in the same area.
- **Verify APIs and dependencies** before using them — check actual documentation, not assumptions.
- **Run tests** before claiming they pass.

### Classification of Claims

Every technical claim must be classifiable as:

| Label | Meaning |
|:---|:---|
| `[VERIFIED]` | Directly confirmed through code, test, or command output |
| `[OBSERVED]` | Seen in runtime or logs but not independently validated |
| `[INFERRED]` | Logically deduced from available evidence |
| `[UNKNOWN]` | Cannot be determined from current information |

### Prohibited Behaviors

- Do NOT fabricate test results or build outputs.
- Do NOT claim a feature works without evidence.
- Do NOT assume an API exists without checking documentation.
- Do NOT claim a file was created without verifying it on disk.
- Do NOT assume a dependency is available without checking `package.json`, `Cargo.toml`, or equivalent.

---

## C. Minimal and Maintainable Implementation

### Core Principles

1. **Do the simplest thing that works correctly.** Avoid over-engineering.
2. **Use existing implementations** when they are suitable. Don't rebuild what already works.
3. **Avoid duplication.** Check if similar code, utilities, or patterns already exist before adding new ones.
4. **No dead code.** Don't leave commented-out code, unused imports, or unreachable branches.
5. **No ghost code.** Don't add placeholder implementations "for later" unless explicitly requested.
6. **No unnecessary abstractions.** Add indirection layers only when there is a concrete, present need.
7. **No speculative dependencies.** Don't add packages, libraries, or tools without a technical justification tied to the current task.

### Code Quality Standards

- Every function should have a clear, single purpose.
- Prefer explicit over implicit behavior.
- Name variables, functions, and files descriptively.
- Keep functions short enough to understand without scrolling.
- Prefer composition over inheritance unless inheritance clearly fits the domain.
- Handle errors explicitly — no silent swallowing of exceptions.

---

## D. Architecture Governance

### Domain Boundaries

- Respect the component boundaries defined in the architecture.
- Each component (Hime Desktop, Hime Core, Hermes integration, 9Router integration) has its own domain.
- Cross-domain changes require understanding of both sides.

### Structural Rules

- **No circular dependencies** between modules or packages.
- **No silent API contract changes.** If you change a function signature, interface, or protocol, document it and update all callers.
- **Record significant decisions** in an Architecture Decision Record (ADR) at `docs/adr/`.
- **No large-scale refactors** outside the scope of the current task. If a refactor is needed, propose it as a separate task.

### Dependency Management

- Pin dependency versions explicitly.
- Document why each dependency was added.
- Prefer well-maintained, widely-used libraries over obscure alternatives.
- Evaluate security advisories before adding new dependencies.

---

## E. Security

### Principles

1. **Least privilege.** Request only the permissions actually needed.
2. **Secret protection.** Never hardcode API keys, tokens, passwords, or credentials in source code. Use environment variables or secure storage (Windows DPAPI, credential manager).
3. **Input validation.** Validate and sanitize all external input — user input, API responses, file contents, environment variables.
4. **Safe file operations.** Validate paths. Prevent path traversal. Don't write to system directories without explicit need.
5. **No unrestricted shell access.** Don't execute arbitrary user-provided strings as shell commands.
6. **No unauthorized privilege escalation.** Don't request admin/root access unless specifically required and documented.
7. **Security review for risky changes.** Changes involving authentication, authorization, cryptography, network exposure, or file system access to sensitive paths require a documented security review.

### Specific Requirements for Hime

- 9Router database (`9router.db`) contains **plaintext credentials**. Apply Windows ACL restrictions to the `.9router` directory.
- 9Router must bind to **`127.0.0.1` only**, never `0.0.0.0`.
- 9Router default password `INITIAL_PASSWORD="123456"` must be replaced with a cryptographically random string during Hime initialization.
- Hermes Agent Named Pipe connections use one-time session tickets for authentication.
- All IPC between Hime Desktop and Hermes/9Router must use localhost-only transports.

---

## F. Testing

### Required Test Types

| Type | When Required | Purpose |
|:---|:---|:---|
| Unit Tests | Every new module or function with logic | Verify individual units in isolation |
| Integration Tests | Cross-module or cross-component interactions | Verify components work together |
| Contract Tests | API boundaries (REST, IPC, WebSocket) | Verify protocol compliance |
| End-to-End Tests | User-facing workflows | Verify complete user scenarios |
| Regression Tests | Every bug fix | Prevent recurrence |
| Security Tests | Auth, crypto, file access, network | Verify security properties |

### Testing Rules

- Write tests before claiming code works.
- Tests must be runnable by another developer or agent without manual setup.
- Tests must have clear assertions — not just "runs without errors."
- Mock external services (Hermes, 9Router, AI providers) in unit tests.
- Use real instances in integration tests when feasible.
- Document test prerequisites and environment requirements.

---

## G. Change Management

### Before Making Changes

1. **Check Git status** — are there uncommitted changes?
2. **Check for work in progress** — is someone else (human or agent) working in the same area?
3. **Understand the scope** — what files and modules will be affected?

### While Making Changes

4. **Work within the approved scope.** Don't expand the task beyond what was agreed.
5. **Track every file changed** — additions, modifications, and deletions.
6. **Commit with meaningful messages** following conventional commit format:
   - `feat:` new feature
   - `fix:` bug fix
   - `docs:` documentation
   - `refactor:` code restructuring
   - `test:` test additions or changes
   - `chore:` maintenance, build, or tooling
   - `security:` security-related change

### After Making Changes

7. **Report all changes** — list every file added, modified, or deleted with reasons.
8. **Run relevant tests** — provide evidence of results.
9. **Do NOT delete files** without performing impact analysis first.
10. **Do NOT modify files outside your task scope** without documenting the reason.

---

## H. Completion Verification

### Definition of Done

A task is not complete until:

1. All acceptance criteria from the task specification are met.
2. All changed/added code has been reviewed (self-review minimum).
3. Relevant tests pass (provide command output as evidence).
4. Documentation is updated if behavior changed.
5. No new warnings or errors introduced.
6. All files are accounted for (listed in the completion report).

### Evidence Requirements

| Claim | Required Evidence |
|:---|:---|
| "It builds" | Actual build command output |
| "Tests pass" | Actual test runner output with pass/fail counts |
| "UI works" | Screenshot or described verification steps |
| "API works" | Request/response examples or test output |
| "File created" | `ls` or `Get-ChildItem` output showing the file |
| "Performance improved" | Before/after metrics |

### Prohibited Completion Claims

- "Should work" without testing.
- "I believe it's correct" without running it.
- "Tests would pass" without actually running them.
- "Compatible with X" without verifying against X.

---

## I. Workspace Structure

```text
Hime/
├── AGENTS.md                    # This file — agent engineering instructions
├── README.md                    # Project overview
├── CHANGELOG.md                 # Change log
├── PROJECT_STATUS.md            # Milestone tracking
│
├── docs/                        # All project documentation
│   ├── master-plan/             # Master roadmap and vision documents
│   ├── architecture/            # System architecture documents
│   ├── adr/                     # Architecture Decision Records
│   ├── governance/              # Engineering governance policies
│   ├── security/                # Security policies and reviews
│   ├── standards/               # Coding and design standards
│   ├── testing/                 # Testing strategy and plans
│   ├── skills/                  # Skill registry and documentation
│   └── reports/                 # Engineering reports
│
├── .agent/                      # Agent skills directory
│   └── skills/                  # Installed agent skills (SKILL.md format)
│
├── RE/                          # Reverse Engineering research (completed)
│   ├── Nusa/                    # Nusa Dynamic Island analysis
│   ├── Hermes/                  # Hermes Agent analysis
│   ├── 9ROUTER/                 # 9Router analysis
│   └── Shared/                  # Cross-cutting research artifacts
│
├── project-management/          # Task and project tracking
│   ├── phases/                  # Phase specifications
│   ├── tasks/                   # Individual task specifications
│   ├── backlog/                 # Backlog items
│   ├── risks/                   # Risk register
│   └── decisions/               # Decision log
│
├── scripts/                     # Build, test, and utility scripts
├── tools/                       # Development tools and utilities
│
├── Documentation/               # Legacy documentation (from RE phase)
├── Projects/                    # Future: implementation source code
├── Resources/                   # Design assets, mockups, icons
└── Archive/                     # Archived artifacts
```

---

## J. File Naming Conventions

- Use `kebab-case` for directories and general files.
- Use `UPPER-KEBAB-CASE` for governance and policy documents (e.g., `CODING_STANDARD.md`).
- Use `PascalCase` for component/module directories when they map to code namespaces.
- Never use colons (`:`) in filenames (NTFS ADS conflict).
- Never use spaces in filenames for new files (existing files with spaces are acceptable).
- Use `.md` for documentation, `.toml` for Rust config, `.json` for structured data.

---

## K. Communication Protocol

When completing a task, agents must provide:

1. **Summary** — What was done and why.
2. **Files Changed** — Complete list with add/modify/delete status.
3. **Evidence** — Command outputs, test results, or screenshots.
4. **Outstanding Issues** — What wasn't completed and why.
5. **Risks** — Any new risks introduced or discovered.
6. **Next Steps** — Recommended follow-up work.

---

*This document is the authoritative source of engineering rules for the Hime project.
All agents must follow these instructions. Changes to this document require explicit
approval from the project owner.*

