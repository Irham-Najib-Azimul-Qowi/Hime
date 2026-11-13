# Hime Desktop Experience — IPC Frontend Integration

- **Document ID:** DOCS-DESKTOP-IPC-INTEGRATION
- **Phase:** Phase 07 — Hime Desktop Experience
- **Authority:** System Architect & Technical Lead
- **Status:** APPROVED IMPLEMENTATION SPECIFICATION

---

## 1. IPC Protocol Contract

Communication between the React frontend and Hime Core traverses the native Windows Named Pipe transport via Tauri commands.

```text
React Frontend  --(invoke)-->  Tauri Backend  --(12-Byte HIME Frame)-->  Hime Core Named Pipe
```

### 12-Byte Header Format
```text
+-----------------------+----------+----------+----------+----------+-----------------------+
| Magic (4 bytes)       | Ver (1B) | Type(1B) | Flags(2B)| Rsrv(2B) | Payload Len (4 bytes) |
| 0x48 0x49 0x4D 0x45   | 0x01     | 0x01..06 | 0x0000   | 0x0000   | Big-Endian u32        |
+-----------------------+----------+----------+----------+----------+-----------------------+
```

---

## 2. Implemented RPC Methods

| RPC Method | Request Parameters | Response Payload | Description |
|:---|:---|:---|:---|
| `system.ping` | `None` or `{"source":"desktop"}` | `{"status":"pong", "core_version":"0.1.0", "server_time_ms": u128}` | Heartbeat latency verification |
| `system.status` | `None` | `{"status":"running", "core":"online", "version":"0.1.0", "uptime_secs": u64, "db_status":"connected", "active_tasks": usize, "total_tasks": usize}` | Real-time kernel telemetry |
| `events.list` | `{"limit": usize}` | `[{"id": i64, "timestamp": i64, "event_type": str, "source": str, "payload": str}]` | Queries audit events from SQLite WAL |
| `tasks.list` | `None` | `[{"id": str, "name": str, "description": str, "priority": str, "status": str, "created_at": i64}]` | Retrieves active supervised tasks |
| `tasks.create` | `{"name": str, "description": str, "priority": str}` | `{"id": str, "name": str, "priority": str, "status":"queued"}` | Creates and audits new task |
| `tasks.cancel` | `{"id": str}` | `{"id": str, "status":"cancelled", "completed_at": i64}` | Cancels active task and audits cancellation |
| `tool.execute` | `{"action_type": str, "path": str}` | `{"decision":"allowed"}` or Error `-32003 Policy Violation` | PEP/PDP security boundary check |

---

## 3. Graceful Failure & Error Boundary

When Hime Core is unavailable:
1. `connect_pipe_client` returns `std::io::Error` (Pipe not found / 2).
2. The Tauri command formats this into a descriptive Rust `Err("Hime Core is unavailable on pipe ...")`.
3. The TypeScript service `ipcService.ts` catches this and sets `isCoreOnline = false`.
4. React components display clean fallback UI states with retry buttons rather than throwing uncaught exceptions.

