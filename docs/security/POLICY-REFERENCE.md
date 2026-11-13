# Hime Security Policy Reference (PEP / PDP)

**Version:** 1.0.0  
**Phase:** Phase 09 — Intelligence Core  
**Enforcer:** `hime-security::policy::PolicyEngine`  
**Security Model:** Fail-Closed, Least Privilege, Zero Trust Localhost

---

## 1. Risk Classification Levels (L0 to L4)

| Level | Identifier | Scope | Evaluation Policy | Example Operations |
|:---|:---|:---|:---|:---|
| **L0** | `L0ReadOnly` | Informational, metadata inspection | **Allow** if path/resource is safe | `system.info`, `system.ping`, `files.stat` |
| **L1** | `L1CreateSafe` | Non-destructive creations in safe areas | **Allow** | `tasks.create`, `notify.send` |
| **L2** | `L2Modify` | Safe updates on owned workspace objects | **Allow** | `tasks.update`, updating task progress |
| **L3** | `L3Destructive` | Deletions, overwrites, terminations | **Require User Approval** (Single-use ticket, 60s TTL) | `files.delete`, destructive file overwrite |
| **L4** | `L4System` | Process execution, OS system config | **Require Explicit Confirmation** | Shell execution (`cmd.exe`, `powershell.exe`) |

---

## 2. Path Validation & Hard Deny Rules

The Policy Engine rigorously canonicalizes and checks all filesystem paths against multiple threat vectors:

### 2.1 Directory Traversal (`..`)
Any path string containing `..` or leading to parent directory escape is **instantly rejected** before any filesystem access occurs.

### 2.2 NTFS Alternate Data Streams (`:`)
Paths containing `:` (outside of the standard single-letter drive root, e.g. `C:\`) are rejected to prevent hidden stream attacks on NTFS.

### 2.3 Windows Reserved Device Names
Files named after MS-DOS legacy devices are rejected:
- `CON`, `PRN`, `AUX`, `NUL`
- `COM1` through `COM9`
- `LPT1` through `LPT9`

### 2.4 System Directory Denylist
Access to operating system and program installation roots is strictly blocked:
- `C:\Windows`
- `C:\Program Files`
- `C:\Program Files (x86)`
- `C:\ProgramData\Microsoft`
- `C:\Recovery`
- `/etc`, `/usr`, `/bin` (Unix cross-platform compatibility)

### 2.5 Sensitive Credential Protection
Paths containing user credential directories are unconditionally blocked from automated tool access:
- `.ssh`
- `.aws`
- `.gnupg`
- `id_rsa`, `id_ed25519`

---

## 3. Forbidden Destructive Command Patterns

The Policy Engine scans external process command lines and blocks destructive patterns:
- `rmdir /s`
- `del /f /s /q`
- `powershell stop-computer`
- `powershell restart-computer`
- `format`
- `diskpart`

---

## 4. Audit Trail Integrity (Hash Chaining)

All audit events recorded in SQLite `audit_events` form an append-only cryptographic hash chain:

$$\text{hash}_n = \text{SHA256}(\text{hash}_{n-1} \parallel \text{timestamp}_n \parallel \text{event\_type}_n \parallel \text{source}_n \parallel \text{payload}_n)$$

The genesis event uses:
$$\text{hash}_0 = \text{"0000000000000000000000000000000000000000000000000000000000000000"}$$

If any row in the SQLite database is modified, inserted out of order, or deleted, `verify_audit_trail()` immediately detects the corruption and flags `tamper_detected: true`.
