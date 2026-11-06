# Security Review Report: [Component or Feature Name]

- **Review Target:** [Feature / PR / Architecture Scope]
- **Reviewer:** [Security Engineer / Agent]
- **Date:** YYYY-MM-DD
- **Target Risk Level:** [HIGH | MEDIUM | LOW]
- **Approval Status:** [APPROVED | APPROVED WITH MITIGATIONS | REJECTED]

---

## 1. Threat Model & Attack Surface Analysis

### 1.1 Evaluated Interfaces
- Network Endpoints (e.g., `127.0.0.1:20128`, WebSocket `/api/ws`)
- IPC Boundaries (Tauri `invoke`, Named Pipe `\\.\pipe\...`)
- Data Storage (SQLite WAL files, JSON configs)

### 1.2 Data Flow & Trust Boundaries
[Describe how data crosses between untrusted user input, the LLM, local daemons, and OS resources.]

---

## 2. Security Checklist

| Check | Pass/Fail | Evidence / Notes |
|:---|:---:|:---|
| **No Hardcoded Secrets** (API keys, session tickets, private keys) | | |
| **Localhost Isolation** (All daemons strictly bind to `127.0.0.1`) | | |
| **Input Sanitization** (Validation against shell injection, path traversal) | | |
| **Least Privilege File I/O** (Restricted paths, safe file naming, no ADS) | | |
| **Memory & Concurrency Safety** (Safe Rust boundaries, bounded buffers) | | |
| **Credential Encryption** (Windows DPAPI applied for disk credentials) | | |

---

## 3. Discovered Vulnerabilities

| Vulnerability ID | Severity (Critical / High / Medium / Low) | Title & Affected Component | CVSS / Impact Description | Required Remediation | Status |
|:---:|:---:|:---|:---|:---|:---:|
| SEC-01 | | | | | OPEN |

---

## 4. Sign-off & Conditions

[State whether the component is approved for deployment/integration or if changes are required.]

