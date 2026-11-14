# Hime Integration: GitHub Adapter

## 1. Overview & Purpose

The GitHub Adapter (`hime-core::integrations::github`) enables Hime to interface with GitHub repositories, pull requests, and issues on behalf of the user, while adhering to Hime's core principle of **Zero Unauthorized Write Operations**:

- **Read Operations (Autonomous)**: Repository status inspection, branch tracking, open issue listings, pull request reviews.
- **Write Operations (Guarded by PEP)**: Creating issues, opening pull requests, pushing commits, and merging code require explicit Human Approval through Hime Desktop / Dynamic Island.

---

## 2. Authentication & Credential Storage

GitHub Personal Access Tokens (PATs) and OAuth tokens are stored exclusively in the Windows DPAPI Credential Vault (`crates/hime-security::dpapi`), protected by hardware-backed user master keys. Tokens are never logged or returned in plain text over IPC or RPC.

---

## 3. Integration Capabilities & Method Signatures

| RPC Endpoint | Operation | Security Tier | Approval Required |
|:---|:---|:---|:---:|
| `integrations.github.inspect` | Inspect repo metadata, branches, PRs | Low | No |
| `integrations.github.issues.list` | List open/closed issues | Low | No |
| `integrations.github.issues.create` | Open new tracking issue | Medium | Yes |
| `integrations.github.pr.create` | Open a pull request against upstream | High | Yes |

---

## 4. Prompt & Output Sanitization

All issue bodies, descriptions, and comments retrieved from GitHub are sanitized before delivery to Hermes Agent. High-entropy strings matching GitHub PAT patterns (`ghp_`, `github_pat_`) are automatically scrubbed via `hime_security::redaction::redact_sensitive_text`.
