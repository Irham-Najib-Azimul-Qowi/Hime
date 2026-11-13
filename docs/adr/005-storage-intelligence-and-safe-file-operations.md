# ADR-005: Storage Intelligence, Journaled Safe File Operations & Hime Recycle

**Status:** Accepted  
**Date:** 2026-11-13  
**Authors:** Principal Engineer + Technical Lead  
**Deciders:** Hime Engineering Team  

---

## 1. Context
Phase 10 tasks Hime with becoming an intelligent, safe, and proactive laptop storage manager (UR-01, UR-03, UR-04). Desktop operating environments are prone to catastrophic data loss from accidental agent actions, unintended deletions, interrupted network transfers, or unsynced write operations. 

To enable autonomous organization, duplicate cleanup, and download interception, Hime requires a strict guarantee of **zero unintended data loss**, reversible file operations, and crash resilience across all file lifecycle transitions.

---

## 2. Decision

We implement the **Storage Intelligence Engine** in `crates/hime-core/src/storage` and deepen `crates/hime-storage` with the following architectural commitments:

1. **Deterministic Safe Operations Pipeline:**
   - Every file move, deletion, or copy executes through staging in `.hime/staging`, verified with SHA-256 pre- and post-hashes.
   - Pre-mutation archiving: files to be overwritten or deleted are moved into **Hime Recycle** (`.hime/recycle`), never permanently unlinked.
   - All operations are journaled in SQLite table `file_operations` before execution.
2. **Startup Crash Recovery:**
   - `SafeFileOperationsEngine::new()` automatically scans uncommitted operations (`staged`, `in_progress`).
   - Orphaned staging files are purged and incomplete records are marked `rolled_back`, ensuring crash resilience across system power loss.
3. **One-Click Reversible Recovery:**
   - Any completed operation in the journal can be reverted with `restore_file()`, transferring the file back from Hime Recycle to its original path after hash verification.
4. **Policy-Enforced Path Sanitization:**
   - Every path is inspected by `PolicyEngine::validate_path_safety`: path traversal (`..`), NTFS Alternate Data Streams (`:stream`), Windows reserved device names (`CON`, `NUL`), sensitive folders (`.ssh`, `.aws`), and OS system roots are denied.
5. **Multi-Stage Duplicate & Version Intelligence:**
   - Duplicates are detected through SHA-256 content hashes.
   - Version candidates are detected via regex/pattern heuristics on file stems, allowing Hime to propose archiving older drafts while preserving new downloads.

---

## 3. Alternatives Considered

### Alternative A: Relying on the Windows OS Recycle Bin API (`SHFileOperation` / `IFileOperation`)
- **Description:** Delegate deletions and moves to the Windows Shell API.
- **Reason for Rejection:** Windows Recycle Bin does not provide programmatic hash verification, cross-volume atomicity guarantees, transaction journaling, or integration with Hime's multi-step agent plans and policy engine.

### Alternative B: Direct Filesystem Deletions with Undo Prompts
- **Description:** Delete files directly with a temporary UI toast undo notification.
- **Reason for Rejection:** Once a process crashes or the notification expires, data is lost permanently. This violates G9, G13, and the zero data loss mandate.

---

## 4. Consequences

### Positive
- Zero risk of unrecoverable data loss during agent operations.
- Full auditability and timeline inspection of all file actions via SQLite WAL journal.
- Safe download interception and duplicate resolution directly from the desktop UI.

### Negative
- Temporary disk overhead while files reside in Hime Recycle (mitigated by 30-day retention notifications and capacity bounds).

---

## 5. Validation Plan & Evidence
- Tested on synthetic directory trees via automated integration suite `tests/storage_integration.rs` (10 passing tests).
- Verified crash recovery rollback of uncommitted staging files.
- Verified one-click restoration of moved and deleted files from Hime Recycle with identical SHA-256 hash match.

