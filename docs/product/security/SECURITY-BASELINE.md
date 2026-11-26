# Hime Security Baseline & Engineering Controls

- **Document:** Canonical Security Baseline & Implementation Requirements
- **Phase:** Phase 01 — Discovery & Technical Validation
- **Task ID:** TASK-01-04
- **Date:** 2026-11-07
- **Authority:** Security Architect & AI Infrastructure Specialist
- **Status:** APPROVED BASELINE SPECIFICATION

---

## 1. Core Security Engineering Principles

Hime follows five non-negotiable security principles across its desktop and mobile architecture:

1. **Least Privilege (*PoLP*):** Components operate with the absolute minimum operating system privileges required. No administrative elevation or UAC bypass shall ever be requested.
2. **Defense-in-Depth:** Multiple redundant controls protect critical assets. Relying solely on one layer (e.g. DPAPI alone without NTFS ACLs, or loopback binding without authentication) is prohibited.
3. **Complete Mediation:** All tool invocations, filesystem modifications, and subprocess executions initiated by AI agents must pass through Hime Core's central policy gate. No autonomous direct OS execution is permitted.
4. **Secure by Default & Active Verification:** All network listeners bind exclusively to loopback (`127.0.0.1`); startup configuration must be validated at runtime via active socket inspection. Permissive default passwords or unauthenticated endpoints are strictly eliminated upon bootstrap.
5. **Fail-Safe Defaults & Clean Rollback:** If a security check, hash validation, socket verification, or integrity check fails, the operation aborts immediately into a safe, non-destructive state (*fail-closed*).

---

## 2. Mandatory Security Controls (SEC-01 through SEC-10)

### SEC-01: Windows NTFS ACL Hardening on Data Directories
- **Security Objective:** Neutralize unauthorized read and write access by other local Windows user accounts and isolate Hime/9Router data directories from standard inherited permissions.
- **Threats Addressed:** Cross-account data theft on multi-user Windows machines, unauthorized tampering by unprivileged local accounts.
- **Responsible Component:** Hime Core Bootstrap Supervisor (Rust).
- **Technical Mechanism:**
  - Hime Core creates dedicated directories under `%LOCALAPPDATA%\Hime` and `%APPDATA%\9router`.
  - Executes Windows icacls lockdown to strip inheritance (`/inheritance:r`) and grant explicit FullControl strictly to the executing user SID (`%USERNAME%`) and `SYSTEM`:
    ```cmd
    icacls "%APPDATA%\9router" /inheritance:r /grant:r "%USERNAME%":(OI)(CI)F /grant:r "SYSTEM":(OI)(CI)F
    icacls "%LOCALAPPDATA%\Hime" /inheritance:r /grant:r "%USERNAME%":(OI)(CI)F /grant:r "SYSTEM":(OI)(CI)F
    ```
  - **EFS (Encrypting File System) on Windows 11 Pro:** EFS (`cipher /e`) is treated as an *optional defense-in-depth layer* against offline disk theft (e.g. physical hard drive extraction). EFS is only applied after runtime verification confirms that EFS certificate generation and backup succeed on the host OS.
- **Verification Criteria:** Audit script executes `(Get-Acl $path).Access` and asserts that only current user SID and SYSTEM possess rights; inherited flags are strictly absent.
- **Security Boundary & Residual Risk:**
  - **Security Boundary:** Isolates against *other* Windows user accounts and offline disk extraction.
  - **Residual Risk:** NTFS ACLs and EFS **DO NOT protect against malware running under the same Windows user session** (`XATOSHI\Satoshi`). Because the current user possesses `FullControl`, any process running under that user's identity can access these files. Status: **`PROPOSED RESIDUAL RISK — OWNER DECISION REQUIRED`**.

---

### SEC-02: Cryptographic Secret Storage (DPAPI with Documented Entropy Limits)
- **Security Objective:** Provide cryptographic protection at rest for master API keys, provider OAuth tokens, and internal session secrets.
- **Threats Addressed:** Offline credential theft, plaintext database scraping by other user accounts, file inspection via backups or disk imaging.
- **Responsible Component:** Hime Core Secret Vault (`vault` module in Rust).
- **Technical Mechanism:**
  - Utilizes Windows DPAPI (`CryptProtectData` via `windows-sys` crate) with `DataProtectionScope.CurrentUser`.
  - **Secondary Entropy Reality:** Hime Core passes an additional entropy buffer (`optionalEntropy`) to `CryptProtectData`.
    * *Realistic Security Assessment:* Additional entropy is NOT an independent security boundary against same-user processes if the entropy is stored where processes of the same user can read it (e.g., Windows Credential Manager or registry). Hardcoded entropy in binaries is strictly forbidden and provides zero security.
    * The entropy buffer serves primarily to prevent cross-application decryption by generic DPAPI decoders and binds encrypted blobs to Hime's application context.
- **Technical Qualification on Same-User Memory Access & Zeroization:**
  - *Same-User Process Memory Access:* Malware berjalan di bawah akun pengguna yang sama (`XATOSHI\Satoshi` pada Medium Integrity) merupakan ancaman serius. Namun, kemampuan proses tersebut untuk membaca memori Hime (`PROCESS_VM_READ` 0x0010 / `PROCESS_QUERY_INFORMATION` 0x0400) secara teknis dievaluasi oleh Windows Security Reference Monitor terhadap *Security Descriptor* (DACL) objek proses Hime dan integritas token akses pemanggil. Hime Core dapat memperkuat perlindungan dengan menetapkan DACL proses restriktif (`SetKernelObjectSecurity`) dan mengaktifkan mitigasi proses (`PROCESS_MITIGATION_DYNAMIC_CODE_POLICY`). Kendati demikian, DPAPI, NTFS ACL, dan zeroization sendiri **TIDAK memberikan isolasi proses yang lengkap**.
  - *Memory Zeroization Reality:* Secrets dimusnahkan dari RAM via crate `zeroize` segera setelah keluar dari scope untuk memperpendek exposure window di RAM. Zeroization **TIDAK mencegah live memory dumping** selama secret sedang aktif digunakan, dan tidak berlaku pada V8 heap 9Router.
- **Verification Criteria:** Automated unit test writes encrypted blob to disk, confirms ciphertext lacks plaintext entropy/keys, and verifies successful round-trip decryption.
- **Residual Risk:** Malware pada sesi user yang sama dapat mencoba membaca memori jika lolos access check process security descriptor. Status: **`PROPOSED RESIDUAL RISK — OWNER DECISION REQUIRED`**.

---

### SEC-03: Process Supervisor Environment Injection (Zero Disk/Shell Leak)
- **Security Objective:** Mitigate RSK-04 (default `123456` dashboard password) and eliminate plaintext credential writing to disk/shell history.
- **Threats Addressed:** Unauthorized dashboard takeover via default password (CWE-1188), credential leakage via shell command history or process argument inspection.
- **Responsible Component:** Hime Process Supervisor (Rust).
- **Technical Mechanism:**
  - Supervisor generates a cryptographically random 32-character CSPRNG token using `ring::rand::SystemRandom`.
  - Injects the token directly into the child process environment block via `std::process::Command::env("INITIAL_PASSWORD", &token)`.
  - Also injects `HOST="127.0.0.1"` and dynamic `PORT`.
  - Zero disk writing, zero shell script execution, zero command-line argument passing (`lpCommandLine` does not contain the password).
- **Verification Criteria:** Supervisor spawns 9Router; test script verifies `http://127.0.0.1:<port>/api/auth/status` rejects default `123456` and accepts only the injected CSPRNG token; process argument inspection via `Get-WmiObject Win32_Process` confirms password is not visible in `CommandLine`.
- **Residual Risk:** Other processes under the same user session could query the PEB environment block using `NtQueryInformationProcess` unless restricted by Windows integrity policies.

---

### SEC-04: Strict Loopback Binding & Runtime Socket Inspection
- **Security Objective:** Prevent LAN exposure (RSK-12) and guarantee that 9Router and internal services never listen on public or local network interfaces.
- **Threats Addressed:** Unauthenticated LAN access to AI endpoints (CWE-1327), accidental exposure via multi-homed network adapters (Wi-Fi/Ethernet).
- **Responsible Component:** Hime Process Supervisor & Network Guard (Rust).
- **Technical Mechanism:**
  - Launch 9Router with explicit argument `--host 127.0.0.1` (`[RECOMMENDATION]`).
  - **Runtime Socket Verification (`[PLANNED CONTROL]`):**
    * Supervisor does NOT assume CLI arguments succeeded. Immediately after spawning 9Router, Supervisor actively queries the listening sockets via Win32 `GetExtendedTcpTable` for both IPv4 (`AF_INET`) and IPv6 (`AF_INET6`).
    * Verifies that all listening sockets are strictly bound to `127.0.0.1` or `::1`, and NEVER `0.0.0.0`, `::`, or any LAN interface IPv4.
    * Verifies port number.
    * Verifies that the socket owning PID (`dwOwningPid`) strictly matches the PID of the 9Router child process spawned by the Supervisor.
    * If binding verification fails (e.g. bound to `0.0.0.0`, LAN IP, or PID mismatch):
      1. Supervisor marks startup as **FAILED** (Security Policy Violation).
      2. Immediately and safely terminates the child process via Windows Job Object (`JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`) / `TerminateProcess`.
      3. Records the security violation and port details to `AUDIT.jsonl`.
      4. Fails closed (refuses to start Hime).
- **Verification Criteria:** Unit test forces simulated `0.0.0.0` binding; Supervisor detects policy breach within 500ms, kills process, and logs alert.
- **Mitigation Status:** **`NOT YET VERIFIED`** (menunggu implementasi dan pengujian runtime pada Phase 06).
- **Address RSK-10 (Implementation Blocker):**
  - *Critical Clarification:* Loopback binding is transport isolation, **NOT authentication**.
  - In 9Router v0.5.112, `src/dashboardGuard.js:161-165` contains `if (isLocalRequest(request)) return true;`. Because this check short-circuits before validating API keys or tokens, **virtual bearer tokens are completely bypassed**.
  - Any local process can connect directly to `http://127.0.0.1:<port>/v1/*` bypassing Hime Core.
  - *Required Resolution:* This is an **`IMPLEMENTATION BLOCKER & PROPOSED RESIDUAL RISK — OWNER DECISION REQUIRED`**. Before Hime can rely on 9Router for quota-controlled or multi-tenant operations, 9Router must be patched (upstream PR or maintained fork) to remove loopback authentication bypass, or 9Router must run under an isolated OS account with Windows Filtering Platform (WFP) firewall rules.

---

### SEC-05: Authenticated & DACL-Protected Named Pipes IPC
- **Security Objective:** Provide secure, authenticated, and high-performance inter-process communication between Hime Desktop, Hime Core, and Hermes Agent.
- **Threats Addressed:** Local IPC hijacking, synthetic command injection by unauthorized local software, impersonation attacks (CWE-285).
- **Responsible Component:** Hime Core IPC Broker (Rust) & Hime Desktop Shell (Tauri).
- **Technical Mechanism:**
  - All local IPC utilizes Windows Named Pipes (`\\.\pipe\hime-{subsystem}-{uuid}`).
  - Hime Core builds a Win32 `SECURITY_ATTRIBUTES` structure with an explicit DACL granting `FILE_GENERIC_READ | FILE_GENERIC_WRITE` strictly to the current user's SID (`[System.Security.Principal.WindowsIdentity]::GetCurrent().User`).
  - Prohibits `SECURITY_IMPERSONATION` or `SECURITY_DELEGATION` (enforces `SECURITY_IDENTIFICATION` only).
  - Enforces a 256-bit ephemeral handshake token exchanged via secure child process spawn pipes before message frames are accepted.
  - Maximum frame size capped at 16 MB with length-prefix framing.
- **Verification Criteria:** Integration test attempts connection to pipe from a simulated unauthorized process / secondary token; server rejects connection with `Access Denied`.
- **Residual Risk:** Same-user processes with administrative elevation could theoretically attach to the pipe, mitigated by standard Medium Integrity execution.

---

### SEC-06: Central Policy & Tool Permission Gate in Hime Core
- **Security Objective:** Prevent autonomous prompt injection attacks against Hermes Agent from executing destructive host commands (RSK-11).
- **Threats Addressed:** Remote code execution via indirect prompt injection (CWE-78), confused deputy attacks, arbitrary file overwriting.
- **Responsible Component:** Hime Core Policy Engine (Rust).
- **Technical Mechanism:**
  - `HERMES_YOLO_MODE` is permanently disabled in the Hime supervisor profile.
  - Hermes Agent's direct command execution capabilities (`terminal_tool`, subprocess execution) are intercepted and mediated by Hime Core.
  - Three-tier permission model:
    1. *Read-Only Operations (Search, Workspace File Read):* Auto-approved if target path lies within approved workspace boundaries.
    2. *State-Altering Operations (Git branch, test execution):* Pre-approved per user session policy.
    3. *Destructive Operations (File write/delete, shell command execution, network requests):* **MANDATORY HUMAN CONFIRMATION MODAL** displayed on Hime Desktop HUD. Modal cannot be dismissed or accepted programmatically by the agent.
- **Verification Criteria:** Test suite simulates indirect prompt injection payload attempting `rmdir`; Hime Core intercepts call, blocks execution, and emits approval request to HUD.
- **Residual Risk:** User fatigue approving legitimate actions leading to accidental approval of malicious commands (mitigated by clear, syntax-highlighted diffs in HUD).

---

### SEC-07: Filesystem Sandboxing & Path Canonicalization
- **Security Objective:** Confine agent read/write operations strictly to designated workspace directories, preventing directory traversal and system tampering.
- **Threats Addressed:** Path traversal / Zip Slip (`..\..`), arbitrary file overwrite, access to sensitive credentials (`.ssh`, `.gnupg`, cookies).
- **Responsible Component:** Hime Core Storage Manager (Rust).
- **Technical Mechanism:**
  - Every path parameter received from agents or tools is resolved using `std::fs::canonicalize`.
  - Verifies `canonical_target.starts_with(workspace_root_canonical)`.
  - **Hardline Deny List (Kernel/System/Credential Paths):**
    * `%WINDIR%` (`C:\Windows`)
    * `%USERPROFILE%\.ssh`
    * `%USERPROFILE%\.gnupg`
    * `%LOCALAPPDATA%\Google\Chrome\User Data`
    * `%APPDATA%\Mozilla\Firefox\Profiles`
    * Windows Startup directories (`AppData\Roaming\Microsoft\Windows\Start Menu\Programs\Startup`)
  - All file write operations utilize atomic write-replace (`tempfile` + `std::fs::rename`).
- **Verification Criteria:** Path traversal test suite passes `../../Windows/System32/drivers/etc/hosts` and verifies immediate rejection with `AccessDenied` error.
- **Residual Risk:** Symbolic links within workspace pointing outside workspace (mitigated by `canonicalize` resolving symlinks prior to prefix check).

---

### SEC-08: Windows Job Objects Process Lifecycle & Supervision
- **Security Objective:** Guarantee deterministic process termination, eliminate orphan/zombie background daemons, and enforce resource bounds on child processes.
- **Threats Addressed:** Process lingering, resource exhaustion DoS, runaway memory consumption by Node.js or Python runtimes.
- **Responsible Component:** Hime Process Supervisor (Rust).
- **Technical Mechanism:**
  - Hime Core creates a Win32 Job Object (`CreateJobObjectW`).
  - Sets `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE` on `JOBOBJECT_EXTENDED_LIMIT_INFORMATION`.
  - Assigns 9Router and Hermes Agent processes to the Job Object (`AssignProcessToJobObject`) upon spawn.
  - If Hime Core exits normally, crashes, or is killed via Task Manager, the Windows kernel deterministically terminates all assigned child processes.
  - Enforces memory quotas: 9Router capped at 512 MB, Hermes capped at 1024 MB.
- **Verification Criteria:** Supervisor spawns 9Router; Hime Core process is killed abruptly with `taskkill /F`; test verifies 9Router process terminates simultaneously.
- **Residual Risk:** None. Windows kernel enforces Job Object lifecycle guarantees.

---

### SEC-09: Sensitive Data Redaction in Structured Logging
- **Security Objective:** Prevent secret exposure, API keys, or private tokens from being written to disk in diagnostic logs.
- **Threats Addressed:** Credential harvesting from log files, information disclosure in error reports (CWE-532).
- **Responsible Component:** Hime Logging Infrastructure (Rust `tracing` crate & Python `logging`).
- **Technical Mechanism:**
  - Structured JSON Lines logging (`AUDIT.jsonl`, `hime.log`).
  - Custom regex redactor layer intercepts log records before output:
    * `Bearer\s+[A-Za-z0-9_\-\.]{16,}` -> `Bearer [REDACTED]`
    * `(sk-[A-Za-z0-9_\-]{20,})` -> `[REDACTED_API_KEY]`
    * `(x-9r-cli-token:\s*)[A-Za-z0-9]{16}` -> `$1[REDACTED]`
    * Basic auth headers: `Basic\s+[A-Za-z0-9+/=]{16,}` -> `Basic [REDACTED]`
  - Log retention capped at 7 days with automatic 50 MB rotation.
- **Verification Criteria:** Unit test logs dummy API keys; asserts log file contains `[REDACTED_API_KEY]` and zero occurrences of raw secret.
- **Residual Risk:** Non-standard secret formats not matching regex patterns could theoretically bypass filters.

---

### SEC-10: Authenticode Code Signing & Deterministic Update Verification
- **Security Objective:** Ensure binary authenticity, prevent malicious update tampering, and protect software supply chain.
- **Threats Addressed:** Malicious binary substitution, man-in-the-middle update injection, untrusted installer execution.
- **Responsible Component:** Hime Release Engineering & Update Client (Phase 15).
- **Technical Mechanism:**
  - All Windows executables (`hime.exe`, installer `.msi`) digitally signed using an Authenticode certificate (`signtool.exe`).
  - Update manifests served exclusively over HTTPS and signed with an Ed25519 publisher key.
  - Update client validates binary SHA-256 digest against the signed manifest prior to execution.
  - Automatic rollback: retains previous stable version in `%LOCALAPPDATA%\Hime\versions\backup` with automatic rollback if post-update health check fails.
- **Verification Criteria:** Update runner rejects unsigned manifest, mismatched SHA-256 digest, or corrupt payload.
- **Residual Risk:** Compromise of release signing private key (mitigated by hardware security key storage for signing keys).

---

## 3. Control Traceability & Status Matrix

| Control ID | Control Name | Target Phase | Implementation Status | Dependency & Type |
|:---|:---|:---:|:---:|:---|
| **SEC-01** | NTFS ACL Hardening & EFS | Phase 06 | PLANNED | Hime Core Rust shell (`icacls`) |
| **SEC-02** | DPAPI + Entropy Vault | Phase 06/09 | PLANNED | Windows `CryptProtectData` |
| **SEC-03** | Supervisor Env Injection | Phase 06 | PLANNED | 9Router `INITIAL_PASSWORD` |
| **SEC-04** | Loopback Binding & Socket Verifier | Phase 06 | PLANNED | Win32 `GetExtendedTcpTable` |
| **SEC-04 (B)**| **Localhost Auth Patch (RSK-10)** | Phase 05/06 | **BLOCKER** | **Upstream 9Router PR or Fork Required** |
| **SEC-05** | Authenticated Named Pipes | Phase 07 | PLANNED | Windows `PipeSecurity` DACL |
| **SEC-06** | Central Policy & Tool Gate | Phase 08 | PLANNED | Hermes approval hooks & Named Pipe |
| **SEC-07** | Filesystem Sandboxing | Phase 09 | PLANNED | Rust `std::fs::canonicalize` |
| **SEC-08** | Windows Job Objects | Phase 06 | PLANNED | Win32 Job Object API |
| **SEC-09** | Log Redaction & Rotation | Phase 06 | PLANNED | Rust `tracing` filter |
| **SEC-10** | Authenticode & Updates | Phase 15 | PLANNED | Windows SDK `signtool` |

---

## 4. Proposed Residual Risks & Implementation Blockers

### A. Proposed Residual Risks — Owner Decision Required
1. **Same-User Process Memory Access:**
   - *Technical Explanation:* Malware atau proses yang berjalan di bawah akun Windows yang sama (`XATOSHI\Satoshi` pada Medium Integrity) merupakan ancaman serius. Keberhasilan membaca memori proses Hime (`PROCESS_VM_READ` 0x0010 dan `PROCESS_QUERY_INFORMATION` 0x0400) bergantung pada evaluasi Windows Security Reference Monitor terhadap *Security Descriptor* (DACL) objek proses Hime dan integritas token akses pemanggil. Hime Core dapat memperkuat perlindungan dengan menetapkan DACL proses restriktif (`SetKernelObjectSecurity`) dan mengaktifkan mitigasi proses (`PROCESS_MITIGATION_DYNAMIC_CODE_POLICY`). Kendati demikian, DPAPI, NTFS ACL, dan zeroization sendiri **TIDAK memberikan isolasi proses yang lengkap**.
   - *Impact & Options:* Jika malware pada akun yang sama berhasil menginjeksi atau membaca memori Hime Core saat secret sedang aktif, token dapat terekspos. Pilihan mitigasi lebih lanjut (misal: Windows AppContainer sandbox atau dedicated service user) memerlukan evaluasi arsitektur di Phase 05. Status: **`PROPOSED RESIDUAL RISK — OWNER DECISION REQUIRED`**.
2. **Upstream 9Router Plaintext Persistence (RSK-03):**
   - *Technical Explanation:* 9Router v0.5.112 menulis API key dan OAuth token dalam teks polos JSON ke SQLite (`%APPDATA%\9router\db\data.sqlite`).
   - *Impact & Options:* Hime Core memitigasi melalui NTFS ACL lockdown (SEC-01) dan EFS diska offline, namun integritas data terhadap malware satu akun tetap terbuka. Opsi penanganan: (a) patch memory-only storage pada Phase 05, atau (b) menerima risiko at-rest SQLite terenkripsi OS. Status: **`PROPOSED RESIDUAL RISK — OWNER DECISION REQUIRED`**.

### B. Implementation Blockers
1. **RSK-10 (9Router Localhost Unauthenticated LLM Access):**
   - *Technical Explanation:* In 9Router v0.5.112, `src/dashboardGuard.js:161-165` short-circuits authentication for all localhost connections. Loopback binding is transport-level only and provides zero authentication. Virtual bearer tokens are bypassed before evaluation.
   - *Impact & Options:* Seluruh proses lokal dapat langsung mengonsumsi kuota inferensi 9Router tanpa otorisasi. Sebelum Hime mengandalkan 9Router untuk operasional multi-tenant atau kuota berbayar sensitif, Project Owner wajib memutuskan:
     - Opsi A: Upstream PR atau maintained fork pada 9Router untuk menghapus short-circuit loopback auth bypass.
     - Opsi B: Menjalankan 9Router di bawah akun service OS terisolasi dengan filter firewall WFP.
     - Status: **`IMPLEMENTATION BLOCKER & PROPOSED RESIDUAL RISK — OWNER DECISION REQUIRED`**.
