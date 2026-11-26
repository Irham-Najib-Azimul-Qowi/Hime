# ADR-004: Intelligence Core, Policy Engine & Deterministic Tool Gateway

**Status:** Accepted  
**Date:** 2026-11-13  
**Authors:** Principal Engineer + Technical Lead  
**Deciders:** Hime Engineering Team  

---

## 1. Context

In Phase 08, Hime integrated external AI runtimes (Nous Research Hermes Agent and decolua 9Router). While basic chat streaming between desktop and Hermes via loopback was established, Phase 09 requires transforming Hime into a multi-step agent operating system capable of executing system workflows, managing background tasks, and enforcing fine-grained security policies.

Hermes Agent and 9Router are untrusted runtimes:
- Hermes Agent can generate arbitrary tool call strings or hallucinate capabilities.
- 9Router has known localhost access bypasses (`isLocalRequest(request)` in `dashboardGuard.js`).
- Giving Hermes direct access to shell, filesystem, or operating system APIs would introduce fatal remote code execution and privilege escalation vulnerabilities.

Furthermore, desktop HUD features such as Dynamic Island selected-text translation (UR-16) require interacting with host applications and Windows clipboard without destroying user clipboard data.

---

## 2. Decision

We establish the **Hime Intelligence Core** within `crates/hime-core` with the following architectural commitments:

1. **Sole PEP/PDP:** `hime-security::policy::PolicyEngine` is the single source of truth for Policy Enforcement and Policy Decisions. No tool, shell command, or filesystem path can be accessed without declarative authorization across levels L0–L4.
2. **Deterministic Tool Gateway:**
   - Implements strict validation, single-use approval ticket checks for destructive actions (L3/L4), idempotency caching backed by SQLite (`tool_executions`), SEC-09 output redaction (Chain-of-Thought `<think>` and API secret keys), and append-only cryptographic hash chaining (`audit_events`).
3. **SQLite-Backed Task Engine:**
   - Enforces a formal state machine (`Queued -> Running -> (WaitingApproval | Paused) -> Succeeded | Failed | Cancelled`).
   - Implements startup crash recovery: unfinished `running` tasks are recovered to `queued` on startup.
   - Throttles concurrency based on the active `ResourceManager` mode (`Efficiency`: 1, `Balanced`: 2, `Performance`: 4).
4. **Island Intelligence Translation Service:**
   - Windows text selection is acquired via Hime PAL (`hime-pal::selection::SelectionReader`), which backs up the original clipboard and restores it immediately.
   - Translation is gated behind an explicit user opt-in flag.
   - Sensitive text (API keys, private keys, passwords) is blocked from transmission to protect privacy.

---

## 3. Alternatives Considered

### Alternative A: Allow Hermes Agent Built-in Tool Execution
- **Description:** Permit Hermes to directly run Python subprocesses and shell tools.
- **Reason for Rejection:** Completely breaks the security perimeter. Hermes is not a security authority and would bypass all Hime auditing, approval workflows, and path protection denylists.

### Alternative B: Ephemeral In-Memory Task & Idempotency Store
- **Description:** Store tasks and idempotency keys only in memory without SQLite WAL persistence.
- **Reason for Rejection:** System crashes or Core restarts would cause task state loss and duplicate tool execution on retries, violating G6 and Phase 09 specification.

---

## 4. Consequences

### Positive
- Strict, fail-closed enforcement of host security boundaries.
- All actions are permanently audited in a tamper-evident SHA-256 hash chain.
- Crashes do not corrupt state or leave zombie running tasks.
- User clipboard is fully preserved when using Island translation.

### Negative
- Multi-step execution adds a lightweight SQLite lookup overhead (~0.1ms per tool execution).
- External runtimes must strictly conform to Hime Core Tool Gateway contracts.

---

## 5. Validation Plan & Evidence

- Unit tests in `hime-security`, `hime-storage`, `hime-pal`, and `hime-core` verifying policy decisions, path traversal prevention, hash chaining, and task lifecycle.
- End-to-end integration tests in `crates/hime-core/tests/intelligence_integration.rs` verifying 2-step plan execution, loop detection, tamper detection, task crash recovery, and sensitive text translation blocking.
- 100% pass rate across workspace test suite (76 tests passed).
