# ADR-016: Dual Rust Workspace Architecture (`core` and `island/src-tauri`)

**Status:** Accepted  
**Date:** 2026-11-19  
**Authors:** Hime Architecture & Monorepo Engineering Team  
**Deciders:** Project Owner, Engineering Team  

---

## 1. Context

In `02b-MONOREPO-LAYOUT-AND-MANAGEMENT.md §5`, the initial plan evaluated whether to unify all Rust crates across the monorepo into a single virtual root workspace (`Cargo.toml` at repo root with `members = ["core/crates/*", "island/src-tauri"]`).

During implementation and dependency auditing (Survey Report 2, Milestone 4), significant dependency version discrepancies were identified between the two subsystems:
- `core/` (Hime Core virtual workspace) depends on Microsoft's `windows = "0.58.0"`.
- `island/src-tauri/` (Bloom-derived Tauri v2 shell) depends on Microsoft's `windows = "0.61"`, `windows-core = "0.61"`, and `windows-collections = "0.2"`.

Attempting to force both subsystems into a single root workspace immediately leads to crate version lock conflicts across the Windows API bindings, breaking compilation in one or both codebases unless a large-scale, high-risk refactoring of either Bloom or Hime Core Win32 calls is performed.

---

## 2. Decision

1. **Ratify Dual Rust Workspaces**: Maintain two independent, isolated Rust workspaces within the monorepo:
   - **Kernel Workspace**: `core/Cargo.toml` managing `hime-core`, `hime-ipc`, `hime-pal`, `hime-security`, `hime-storage`, and `hime-modsdk`.
   - **Shell Workspace**: `island/src-tauri/Cargo.toml` managing the Tauri v2 Desktop Island application (`hime-island.exe`) and the global hotkey daemon (`hime_hotkey.exe`).
2. **IPC-Only Coupling**: The two workspaces NEVER link directly via compile-time Rust `path` or `git` dependencies. Communication occurs exclusively at runtime via standard Win32 Local Named Pipe (`\\.\pipe\hime-ipc`) with JSON-RPC payloads and the Win32 Named Event `HimeQuit`.
3. **Single CLI Entry Point**: Unified lifecycle management is provided by `scripts/hime.ps1` (`build`, `test`, `dev`, `check`), invoking the appropriate `cargo` commands with their respective `--manifest-path` targets (`core/Cargo.toml` and `island/src-tauri/Cargo.toml`).

---

## 3. Alternatives Considered

- **Alternative A: Force Single Root Workspace with forced `windows = "0.61"` upgrade on Core**:
  - *Rejected:* Modifying Core's Windows crate version requires extensive re-auditing of `hime-pal` (window management, process token privileges, job object containment), which is stabilized and tested with 124+ unit tests. Risk of regression is unacceptably high.
- **Alternative B: Downgrade Bloom to `windows = "0.58.0"`**:
  - *Rejected:* Upstream Bloom v3.10.2 is tightly integrated with Tauri v2.11 and Windows 0.61 features (e.g. AppBar, Dwm, WinRT collections). Downgrading would break shell integration.

---

## 4. Consequences

- **Positive**:
  - Subsystems build and test reliably without dependency conflicts.
  - Zero regression risk to Core's stabilized Win32 PAL or Island's shell physics.
  - Preserves clean process and licensing separation between the GPL-3.0 desktop shell and independent background daemons.
- **Negative**:
  - Dependencies are fetched into two separate `target/` directories (`core/target` and `island/src-tauri/target`), consuming slightly more disk space during development.
