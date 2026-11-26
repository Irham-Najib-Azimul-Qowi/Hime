# Shell ⇄ Core IPC Protocol Contract

> Document: `docs/api/SHELL-CORE-CONTRACT.md`  
> Author: Principal Engineer / Tech Lead  
> Version: 1.0.0  
> Transport: Windows Named Pipe `\\.\pipe\hime-ipc`  
> Framing: 12-byte binary header + JSON-RPC 2.0 payload

---

## 1. Binary Framing Protocol

Every frame sent across the Named Pipe consists of a 12-byte big-endian header followed by a variable-length payload.

```text
 0                   1                   2                   3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                          MAGIC ("HIME")                       |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|    VERSION    |   MSG_TYPE    |           RESERVED (0)        |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                          PAYLOAD_LEN                          |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                        PAYLOAD BYTES ...                      |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
```

| Field         | Type      |  Size   | Valid Values                                                                                                          | Description                                 |
| :------------ | :-------- | :-----: | :-------------------------------------------------------------------------------------------------------------------- | :------------------------------------------ |
| `MAGIC`       | `[u8; 4]` | 4 bytes | `b"HIME"` (`0x48 0x49 0x4D 0x45`)                                                                                     | Protocol sync magic identifier              |
| `VERSION`     | `u8`      | 1 byte  | `0x01`                                                                                                                | Protocol version                            |
| `MSG_TYPE`    | `u8`      | 1 byte  | `0x01` (Request), `0x02` (Response), `0x03` (Notification), `0x04` (StreamChunk), `0x05` (StreamEnd), `0x06` (Cancel) | Message frame type                          |
| `RESERVED`    | `u16`     | 2 bytes | `0x0000`                                                                                                              | Big-endian reserved field (must be 0)       |
| `PAYLOAD_LEN` | `u32`     | 4 bytes | `0` to `16,777,216` (16 MiB)                                                                                          | Big-endian length of following JSON payload |

---

## 2. Authentication & Handshake Lifecycle

Before issuing any command, the Shell must execute `system.handshake` with a valid session authentication token and a monotonically increasing nonce.

### Handshake Request:

```json
{
	"jsonrpc": "2.0",
	"id": 1,
	"method": "system.handshake",
	"params": {
		"client_id": "bloom-shell-bridge",
		"auth_token": "<32-byte CSPRNG token>",
		"nonce": 1
	}
}
```

### Handshake Response (Success):

```json
{
	"jsonrpc": "2.0",
	"id": 1,
	"result": {
		"status": "authenticated",
		"core_version": "0.1.0",
		"session_id": "sess-..."
	}
}
```

---

## 3. Core RPC Endpoints

| RPC Method          | Request Params                                         | Success Result                                                             | Error Codes                                         |
| :------------------ | :----------------------------------------------------- | :------------------------------------------------------------------------- | :-------------------------------------------------- |
| `system.ping`       | `None`                                                 | `{"status": "pong", "core_version": "0.1.0", "server_time_ms": 179465...}` | -                                                   |
| `system.handshake`  | `{"client_id": str, "auth_token": str, "nonce": u64}`  | `{"status": "authenticated", "core_version": str}`                         | `-32001` (Unauthorized), `-32002` (Replay Detected) |
| `approvals.resolve` | `{"request_id": str, "decision": "approve" \| "deny"}` | `{"request_id": str, "status": "approved" \| "denied"}`                    | `-32004` (Expired / Not Found)                      |
| `tasks.create`      | `{"name": str, "description": str, "priority": str}`   | Task object                                                                | -                                                   |
| `tasks.list`        | `None`                                                 | Array of Task objects                                                      | -                                                   |

---

## 4. Frontend Tauri Commands

| Command                     | Arguments                                                | Return Type                    | Description                                                   |
| :-------------------------- | :------------------------------------------------------- | :----------------------------- | :------------------------------------------------------------ |
| `hime_core_status`          | `None`                                                   | `HimeCoreStatus`               | Returns real-time connection state, latency, active approvals |
| `hime_ping`                 | `None`                                                   | `Result<u64, String>`          | Sends ping to Core and returns roundtrip latency in ms        |
| `hime_approval_list`        | `None`                                                   | `Vec<ApprovalItem>`            | Returns all pending approvals with TTL expiry                 |
| `hime_approval_resolve`     | `request_id: String, decision: String`                   | `Result<String, String>`       | Resolves an approval and emits resolution event               |
| `hime_dev_trigger_approval` | `action_type: String, path: String, description: String` | `Result<ApprovalItem, String>` | Debug trigger for testing approval bubble                     |

---

## 5. Frontend Tauri Events

- `hime-core-status`: Emitted whenever Core connection state changes (`online`, `offline`, `reconnecting`).
- `hime-approval-pending`: Emitted when an agent or tool requests user approval (TTL 60s).
- `hime-approval-resolved`: Emitted when an approval is resolved or expires.
