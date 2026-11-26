# ADR 008: Browser Service and Developer Workspace Architecture

## Status
Accepted

## Date
2026-11-14

## Context
Phase 13 requires adding web browsing capabilities, external integrations (GitHub, Google Drive), and software engineering workspace tools. 

These capabilities introduce substantial security challenges:
1. Web browsing is vulnerable to SSRF (Server-Side Request Forgery) attacks against loopback services (`127.0.0.1`, `9Router`, `Hermes`, local IPC) and cloud instance metadata (`169.254.169.254`).
2. Web text passed into LLM prompts can contain prompt injection attacks designed to break out of reasoning boundaries.
3. Arbitrary shell command execution (`cmd.exe /c` or `powershell.exe`) can lead to arbitrary code execution, command chaining (`;`, `&&`), and environment compromise.
4. Git operations might inadvertently commit credentials, API keys, or private SSH keys.

## Decisions

1. **SSRF Defense at Policy Layer**:
   - `BrowserSecurityPolicy` enforces strict scheme filtering (`http`, `https` only) and IP resolution checks.
   - Loopback (`127.0.0.0/8`, `::1`), RFC 1918 private subnets, link-local addresses, and cloud metadata endpoints are rejected before socket connection.

2. **Prompt Injection Tag Encapsulation**:
   - Web DOM readability text is encapsulated in `<untrusted_web_content>` tags. Any rogue closing tag within the extracted payload is escaped to `&lt;/untrusted_web_content&gt;`.

3. **Isolated Quarantined Downloads**:
   - Web downloads do not write directly to the user's Downloads or Desktop folders. They are isolated in `.hime/quarantine/` with SHA-256 calculation and SQLite tracking.

4. **Zero Raw Shell Strings**:
   - Developer commands are executed by invoking allowed binaries (`git`, `cargo`, `pnpm`, etc.) with arguments passed directly as `&[String]` vectors via `std::process::Command`, without shell interpreters.

5. **Pre-Commit Secret Scanner**:
   - Commits initiated via Hime trigger a secret audit over modified and staged files. Detected secrets halt the commit pipeline.

6. **Windows UNC Path Normalization**:
   - Windows verbatim UNC prefixes (`\\?\`) are stripped before path boundary checking to prevent false-positive traversal alerts.

## Consequences

- Web automation is strictly confined to external public pages; local network probes are blocked.
- LLM reasoning cannot be hijacked by web page DOM content.
- Developers have a unified, secure workspace toolchain with automatic secret defense.

