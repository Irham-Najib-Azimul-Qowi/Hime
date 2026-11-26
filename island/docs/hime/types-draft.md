# TypeScript Types Draft (Ported from Legacy IPC Clients)

> Document: `types-draft.md`  
> Author: Principal Engineer / Tech Lead  
> Date: 2026-11-16  
> Scope: Pure TypeScript DTO interfaces ready for use in `src/hime/types/` during Stage H2 and Phase 08R+.

---

## 1. Core IPC & Diagnostics Types

```typescript
export interface PingResult {
  success: boolean;
  message: string;
  latency_ms: number;
  core_response: {
    status?: string;
    core_version?: string;
    server_time_ms?: number;
    [key: string]: unknown;
  } | null;
}

export interface SystemStatus {
  status: 'running' | 'offline' | 'error';
  core: 'online' | 'disconnected' | 'reconnecting';
  version: string;
  uptime_secs: number;
  db_status: string;
  named_pipe: string;
  active_tasks: number;
  total_tasks: number;
}

export interface ToolExecutionResult {
  decision: 'allowed' | 'pending_approval';
  request_id?: string;
  result?: string;
  description?: string;
  expires_at?: number;
  status?: string;
}

export interface ApprovalResolutionResult {
  request_id: string;
  status: 'approved' | 'denied' | 'expired';
  result?: string;
}
```

---

## 2. AI Runtime Types (Phase 08R)

```typescript
export interface AiModel {
  id: string;
  name: string;
  provider: string;
  context_length: number;
  is_available: boolean;
}

export interface AiStatus {
  core_orchestration_active: boolean;
  hermes_status: {
    is_available: boolean;
    state: string;
    base_url: string;
    active_model: string;
    active_sessions_count: number;
    total_turns_executed: number;
  };
  router_status: {
    is_running: boolean;
    endpoint: string;
    port: number;
    latency_ms: number | null;
    direct_access_authenticated: boolean;
    ib_02_status: string;
  };
  loopback_adapter_port: number;
  configured_providers: string[];
  ib_01_status: string;
  ib_02_status: string;
}

export interface Conversation {
  id: string;
  title: string;
  model: string;
  created_at: number;
  updated_at: number;
}

export interface ChatMessage {
  id: string;
  conversation_id: string;
  role: 'user' | 'assistant' | 'system' | 'tool';
  content: string;
  created_at: number;
  token_count?: number;
}

export interface CredentialInfo {
  provider: string;
  configured: boolean;
  protected_by: string;
}
```

---

## 3. Events & Notifications Types

```typescript
export interface AuditEvent {
  id: number;
  timestamp: number;
  event_type: string;
  source: string;
  payload: string;
}

export type EventCategory = 'all' | 'core' | 'tool' | 'task' | 'policy';

export type NotificationCategory = 'task' | 'agent' | 'system' | 'permission' | 'error';
export type NotificationPriority = 'low' | 'normal' | 'high' | 'urgent';

export interface NotificationItem {
  id: string;
  title: string;
  message: string;
  category: NotificationCategory;
  priority: NotificationPriority;
  timestamp: number;
  read: boolean;
  actionRequired?: boolean;
  actionType?: 'approval' | 'view';
  contextData?: Record<string, unknown>;
}
```

---

## 4. Task & Storage Types (Phase 09R–10R)

```typescript
export type TaskPriority = 'low' | 'normal' | 'high' | 'urgent';
export type TaskStatus = 'queued' | 'running' | 'completed' | 'failed' | 'cancelled';

export interface HimeTask {
  id: string;
  name: string;
  description: string;
  priority: TaskPriority;
  status: TaskStatus;
  created_at: number;
  completed_at?: number | null;
}

export interface IndexedFile {
  path: string;
  parent_dir: string;
  file_name: string;
  extension: string | null;
  size_bytes: number;
  modified_at: number;
  created_at: number;
  is_dir: boolean;
  content_hash: string | null;
  mime_type: string | null;
  indexed_at: number;
}

export interface StorageAnalysisReport {
  base_report: {
    total_files: number;
    total_bytes: number;
    largest_files: IndexedFile[];
    old_downloads: IndexedFile[];
  };
  cleanable_suggestions: {
    category: string;
    path: string;
    size_bytes: number;
    reason: string;
  }[];
  cleanable_bytes: number;
}
```

---

## 5. Mobile & Device Types (Phase 11R)

```typescript
export interface PairedDevice {
  id: string;
  device_name: string;
  device_type: string;
  cert_fingerprint: string;
  paired_at: number;
  last_seen_at: number;
  status: 'active' | 'revoked';
}

export interface PairingSession {
  server_address: string;
  server_port: number;
  pairing_token: string;
  confirmation_code: string;
  ca_fingerprint: string;
  expires_at: number;
}

export interface FileTransfer {
  id: string;
  device_id: string;
  file_name: string;
  file_size: number;
  bytes_transferred: number;
  direction: 'incoming' | 'outgoing';
  status: 'pending' | 'transferring' | 'completed' | 'failed';
  sha256: string;
  started_at: number;
  completed_at?: number;
}
```
