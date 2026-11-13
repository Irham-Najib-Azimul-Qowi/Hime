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

