import { invoke } from '@tauri-apps/api/core';
import {
  AiModel,
  AiStatus,
  ChatMessage,
  Conversation,
  CredentialInfo,
  PingResult,
  SystemStatus,
  ToolExecutionResult,
} from '../types/ipc';
import { AuditEvent } from '../types/events';
import { HimeTask } from '../types/tasks';

export const isTauriEnvironment = (): boolean => {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
};

export const pingCore = async (): Promise<PingResult> => {
  if (!isTauriEnvironment()) {
    return {
      success: false,
      message: 'Running in non-Tauri browser preview. Hime Core IPC requires native Tauri shell.',
      latency_ms: 0,
      core_response: null,
    };
  }
  return await invoke<PingResult>('ping_core');
};

export const getSystemStatus = async (): Promise<SystemStatus> => {
  if (!isTauriEnvironment()) {
    return {
      status: 'offline',
      core: 'disconnected',
      version: '0.1.0-preview',
      uptime_secs: 0,
      db_status: 'preview',
      named_pipe: 'none',
      active_tasks: 0,
      total_tasks: 0,
    };
  }
  return await invoke<SystemStatus>('get_system_status');
};

export const getActivityEvents = async (limit: number = 30): Promise<AuditEvent[]> => {
  if (!isTauriEnvironment()) {
    return [];
  }
  return await invoke<AuditEvent[]>('get_activity_events', { limit });
};

export const executeTool = async (
  actionType: string,
  path: string
): Promise<ToolExecutionResult> => {
  if (!isTauriEnvironment()) {
    throw new Error('Native IPC requires Tauri desktop environment.');
  }
  return await invoke<ToolExecutionResult>('execute_tool', { actionType, path });
};

export const resolveApproval = async (
  requestId: string,
  decision: 'approve' | 'deny'
): Promise<any> => {
  if (!isTauriEnvironment()) {
    throw new Error('Native IPC requires Tauri desktop environment.');
  }
  return await invoke('resolve_approval', { requestId, decision });
};

export const createCoreTask = async (
  name: string,
  description: string,
  priority: string
): Promise<HimeTask> => {
  if (!isTauriEnvironment()) {
    throw new Error('Native IPC requires Tauri desktop environment.');
  }
  return await invoke<HimeTask>('create_task', { name, description, priority });
};

export const getCoreTasks = async (): Promise<HimeTask[]> => {
  if (!isTauriEnvironment()) {
    return [];
  }
  return await invoke<HimeTask[]>('get_tasks');
};

export const cancelCoreTask = async (id: string): Promise<HimeTask> => {
  if (!isTauriEnvironment()) {
    throw new Error('Native IPC requires Tauri desktop environment.');
  }
  return await invoke<HimeTask>('cancel_task', { id });
};

export const windowSetAlwaysOnTop = async (alwaysOnTop: boolean): Promise<void> => {
  if (isTauriEnvironment()) {
    await invoke('window_set_always_on_top', { alwaysOnTop });
  }
};

export const windowSetSizeMode = async (mode: 'compact' | 'expanded'): Promise<void> => {
  if (isTauriEnvironment()) {
    await invoke('window_set_size_mode', { mode });
  }
};

export const windowMinimize = async (): Promise<void> => {
  if (isTauriEnvironment()) {
    await invoke('window_minimize');
  }
};

export const windowMaximize = async (): Promise<void> => {
  if (isTauriEnvironment()) {
    await invoke('window_maximize');
  }
};

export const windowClose = async (): Promise<void> => {
  if (isTauriEnvironment()) {
    await invoke('window_close');
  }
};

// -----------------------------------------------------------------------------
// Phase 08 — AI Runtime Integration IPC APIs
// -----------------------------------------------------------------------------

export const getAiStatus = async (): Promise<AiStatus> => {
  if (!isTauriEnvironment()) {
    return {
      core_orchestration_active: true,
      hermes_status: {
        is_available: true,
        state: 'idle',
        base_url: 'http://127.0.0.1:20129/v1',
        active_model: 'hermes-3-llama-3.1-8b',
        active_sessions_count: 1,
        total_turns_executed: 0,
      },
      router_status: {
        is_running: false,
        endpoint: '127.0.0.1:20128',
        port: 20128,
        latency_ms: null,
        direct_access_authenticated: false,
        ib_02_status: 'OPEN (Localhost loopback bypass in 9Router dashboardGuard.js)',
      },
      loopback_adapter_port: 20129,
      configured_providers: ['openai'],
      ib_01_status: 'RESOLVED (Hime-Owned Provider Adapter + DPAPI Vault)',
      ib_02_status: 'OPEN (Localhost loopback bypass in 9Router)',
    };
  }
  return await invoke<AiStatus>('ai_get_status');
};

export const getAiModels = async (): Promise<AiModel[]> => {
  if (!isTauriEnvironment()) {
    return [
      {
        id: 'hermes-3-llama-3.1-8b',
        name: 'Nous Hermes 3 (Llama 3.1 8B)',
        provider: 'nous-research',
        context_length: 131072,
        is_available: true,
      },
      {
        id: 'gpt-4o',
        name: 'OpenAI GPT-4o',
        provider: 'openai',
        context_length: 128000,
        is_available: true,
      },
      {
        id: 'claude-3-5-sonnet',
        name: 'Anthropic Claude 3.5 Sonnet',
        provider: 'anthropic',
        context_length: 200000,
        is_available: true,
      },
    ];
  }
  return await invoke<AiModel[]>('ai_get_models');
};

export const storeCredential = async (provider: string, secret: string): Promise<any> => {
  if (!isTauriEnvironment()) {
    return { status: 'stored', provider, vault: 'DPAPI' };
  }
  return await invoke('credentials_store', { provider, secret });
};

export const listCredentials = async (): Promise<CredentialInfo[]> => {
  if (!isTauriEnvironment()) {
    return [{ provider: 'openai', configured: true, protected_by: 'Windows DPAPI' }];
  }
  return await invoke<CredentialInfo[]>('credentials_list');
};

export const createConversation = async (
  title: string,
  model: string
): Promise<Conversation> => {
  if (!isTauriEnvironment()) {
    return {
      id: `conv-preview-${Date.now()}`,
      title,
      model,
      created_at: Math.floor(Date.now() / 1000),
      updated_at: Math.floor(Date.now() / 1000),
    };
  }
  return await invoke<Conversation>('conversation_create', { title, model });
};

export const listConversations = async (): Promise<Conversation[]> => {
  if (!isTauriEnvironment()) {
    return [];
  }
  return await invoke<Conversation[]>('conversation_list');
};

export const getConversationMessages = async (
  conversationId: string
): Promise<ChatMessage[]> => {
  if (!isTauriEnvironment()) {
    return [];
  }
  return await invoke<ChatMessage[]>('conversation_get_messages', { conversationId });
};

export const sendConversationMessage = async (
  conversationId: string,
  prompt: string
): Promise<{ status: string; response: string }> => {
  if (!isTauriEnvironment()) {
    return {
      status: 'completed',
      response: `[Preview Mode] Salam dari Hermes Agent! Pesan Anda: "${prompt}" berhasil diterima di lingkungan simulasi.`,
    };
  }
  return await invoke<{ status: string; response: string }>('conversation_send_message', {
    conversationId,
    prompt,
  });
};

export const cancelConversation = async (): Promise<{ cancelled: boolean; status: string }> => {
  if (!isTauriEnvironment()) {
    return { cancelled: true, status: 'generation_stopped' };
  }
  return await invoke<{ cancelled: boolean; status: string }>('conversation_cancel');
};

