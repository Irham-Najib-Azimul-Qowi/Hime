# Runbook: File Recovery & Hime Recycle Operations

## 1. Overview
Hime strictly adheres to a **zero-unintended-data-loss** policy. All destructive operations (file deletions, relocations, or overwrites) are pre-empted by archiving the source file into **Hime Recycle** (`.hime/recycle`) before any modification occurs.

This runbook guides operators and users through file restoration, crash recovery, and disaster response.

---

## 2. Standard One-Click File Restoration

### Via Hime Desktop GUI
1. Open Hime Desktop and navigate to the **Storage** view (click **Storage** on the sidebar or press `Alt+5`).
2. Select the **Journal & Hime Recycle** tab.
3. Locate the operation in the timeline table by:
   - Operation ID (e.g. `op-move-101`, `del-op-001`)
   - Original source path
   - Timestamp and actor
4. Click the **1-Click Restore** button next to the operation record.
5. Hime Core will:
   - Verify the presence and SHA-256 integrity of the archived file in `.hime/recycle/`.
   - Ensure the destination original path does not have an conflicting file.
   - Atomically move the file from Recycle back to its original location.
   - Update the operation record in `file_operations` to reflect the completed restoration.

### Via IPC / Tool Gateway
Invoke the `storage.restore_file` tool or the `storage.operations.restore` RPC method:
```json
{
  "jsonrpc": "2.0",
  "id": "req-restore-1",
  "method": "storage.operations.restore",
  "params": {
    "original_op_id": "op-move-101",
    "actor": "user"
  }
}
```

---

## 3. Manual Disaster Recovery (Offline Core)

If Hime Core is offline or inaccessible, files can be recovered directly from the local filesystem:

1. Navigate to the Hime data directory:
   - Windows default: `%APPDATA%\Hime\recycle\` (or `.hime/recycle/` within custom workspace root).
2. The files in the recycle directory are preserved in the format:
   `<timestamp>_<original_filename>`
3. To inspect which file corresponds to an operation, inspect the SQLite database:
   ```bash
   sqlite3 "%APPDATA%\Hime\hime_core.db"
   SELECT id, op_type, source_path, backup_path, status, datetime(started_at, 'unixepoch') FROM file_operations;
   ```
4. Copy the file from the `backup_path` back to the desired `source_path`.

---

## 4. Crash Recovery Protocol

If the host machine experienced a sudden power loss or process kill during a file operation:

1. When Hime Core boots up, `SafeFileOperationsEngine::new()` automatically runs `recover_uncommitted_operations()`.
2. Hime Core queries `file_operations` for operations stuck in `staged` or `in_progress`.
3. For each incomplete operation:
   - Any dangling staging files in `.hime/staging/` matching the operation ID are purged.
   - The operation status is marked as `rolled_back` in the database.
   - An audit event is recorded: `storage.crash_recovery_completed`.
4. The user's original source file remains untouched or safe in Hime Recycle.

---

## 5. Storage Retention Policy & Recycle Maintenance

- **Default Retention:** 30 days in Hime Recycle.
- **Capacity Limits:** When Hime Recycle exceeds 10% of total disk space, Hime alerts the user via the Dynamic Island and Notification Center with a proposal to purge items older than 30 days.
- **Explicit User Consent:** Automatic silent deletion is prohibited. The user must approve recycle purge operations.
