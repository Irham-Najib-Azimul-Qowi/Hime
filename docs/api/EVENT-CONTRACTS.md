# Hime Core Event Contracts

**Version:** 1.0.0  
**Phase:** Phase 09 — Intelligence Core  
**Subsystem:** Event Engine (`hime-core::ai::event_engine`)

---

## 1. Event Envelope Specification

All events emitted across the Hime ecosystem follow a versioned schema contract:

```json
{
  "schema_version": 1,
  "event_id": 142,
  "correlation_id": "corr-uuid-42",
  "timestamp_ms": 1731510420000,
  "event_type": "task.status_changed",
  "payload": {
    "task_id": "task-101",
    "status": "running",
    "progress": 0.5
  }
}
```

### Fields

| Field | Type | Description |
|:---|:---|:---|
| `schema_version` | `u32` | Protocol schema version (currently `1`) |
| `event_id` | `u64` | Monotonically increasing unique event sequence identifier |
| `correlation_id` | `Option<String>` | Request or plan trace correlation token |
| `timestamp_ms` | `u64` | Unix epoch timestamp in milliseconds |
| `event_type` | `String` | Namespaced event category |
| `payload` | `JSON Object` | Domain-specific structured event payload |

---

## 2. Event Catalog

### `agent.state_changed`
Emitted whenever the Hermes Agent transition occurs (`idle`, `thinking`, `executing_tool`, `streaming`).
- **Payload:** `{ "new_state": "thinking" }`

### `agent.loop_detected`
Emitted when quality control heuristics detect a repetitive loop of tool invocations.
- **Payload:** `{ "step_id": "s-3", "tool_id": "system.info" }`

### `task.status_changed`
Emitted upon task state transitions in the Task Engine.
- **Payload:** `{ "task_id": "task-101", "status": "succeeded", "progress": 1.0 }`

### `tool.executed`
Emitted upon completion of a tool execution.
- **Payload:** `{ "tool": "files.stat", "success": true, "params": { ... } }`

### `policy.approval_requested`
Emitted when a tool or operation requires single-use user approval (L3/L4).
- **Payload:** `{ "request_id": "appr-abc123", "action": "file_write", "expires_at": 1731510480 }`

### `selection_translated`
Emitted when the Island translation service successfully captures and translates text.
- **Payload:** `{ "original": "hello", "translated": "Halo", "target_lang": "id" }`

---

## 3. Delivery & Offline Catchup Semantics

- **Rolling Buffer:** The Event Engine maintains an in-memory ring buffer (default capacity: 200 events).
- **Offline UI Reconnection:** When Hime Desktop reconnects or wakes from sleep, it queries `events.recent` passing `since_id: <last_seen_event_id>`. The engine returns all events emitted in the interim, ensuring at-least-once delivery with client-side deduplication.
