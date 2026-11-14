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
import { BrowserSession, BrowserHistoryItem, BrowserDownload, BrowserExtractionResult } from '../types/browser';
import { DeveloperWorkspace, DeveloperCommandOutput, GitStatusResult, SeTemplateItem } from '../types/developer';
import { IntegrationAccount, GitHubRepoSummary, GitHubIssue, GoogleDriveFile } from '../types/integrations';
import {
  SpecialistDefinition,
  DelegationResult,
  DatasetProfile,
  ExperimentDesign,
  ProactiveSuggestionItem,
  ProactiveSettings,
  EvaluationReport,
} from '../types/specialists';

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

// -----------------------------------------------------------------------------
// Phase 10: Storage Intelligence IPC APIs
// -----------------------------------------------------------------------------

import {
  StorageAnalysisReport,
  DuplicateGroup,
  FileOperation,
  FileRule,
  IndexedFile,
} from '../types/storage';

export const getStorageAnalysis = async (topN: number = 10): Promise<StorageAnalysisReport> => {
  if (!isTauriEnvironment()) {
    return {
      base_report: {
        total_files: 42,
        total_bytes: 52428800,
        by_extension: [
          { extension: 'pdf', count: 18, total_bytes: 25000000 },
          { extension: 'docx', count: 12, total_bytes: 15000000 },
          { extension: 'rs', count: 8, total_bytes: 4000000 },
          { extension: 'png', count: 4, total_bytes: 8428800 },
        ],
        largest_files: [
          {
            path: 'C:\\Users\\Satoshi\\Documents\\Annual_Report_2026.pdf',
            parent_dir: 'C:\\Users\\Satoshi\\Documents',
            file_name: 'Annual_Report_2026.pdf',
            extension: 'pdf',
            size_bytes: 14200000,
            modified_at: Math.floor(Date.now() / 1000) - 86400,
            created_at: Math.floor(Date.now() / 1000) - 172800,
            is_dir: false,
            content_hash: 'a1b2c3d4e5f6',
            mime_type: 'application/pdf',
            indexed_at: Math.floor(Date.now() / 1000),
          },
        ],
        old_downloads: [
          {
            path: 'C:\\Users\\Satoshi\\Downloads\\installer_setup.exe',
            parent_dir: 'C:\\Users\\Satoshi\\Downloads',
            file_name: 'installer_setup.exe',
            extension: 'exe',
            size_bytes: 85000000,
            modified_at: Math.floor(Date.now() / 1000) - 35 * 86400,
            created_at: Math.floor(Date.now() / 1000) - 40 * 86400,
            is_dir: false,
            content_hash: 'deadbeef1234',
            mime_type: 'application/octet-stream',
            indexed_at: Math.floor(Date.now() / 1000),
          },
        ],
      },
      cleanable_suggestions: [
        {
          category: 'Old Download Installer/Archive',
          path: 'C:\\Users\\Satoshi\\Downloads\\installer_setup.exe',
          size_bytes: 85000000,
          reason: 'Installer (.exe) in Downloads older than 30 days',
        },
      ],
      cleanable_bytes: 85000000,
    };
  }
  return await invoke<StorageAnalysisReport>('storage_get_analysis', { topN });
};

export const getStorageDuplicates = async (): Promise<DuplicateGroup[]> => {
  if (!isTauriEnvironment()) {
    return [
      {
        content_hash: 'sha256-abc123dup',
        size_bytes: 2048576,
        files: [
          {
            path: 'C:\\Users\\Satoshi\\Documents\\Draft_Proposal.pdf',
            parent_dir: 'C:\\Users\\Satoshi\\Documents',
            file_name: 'Draft_Proposal.pdf',
            extension: 'pdf',
            size_bytes: 2048576,
            modified_at: Math.floor(Date.now() / 1000) - 7200,
            created_at: Math.floor(Date.now() / 1000) - 14400,
            is_dir: false,
            content_hash: 'sha256-abc123dup',
            mime_type: 'application/pdf',
            indexed_at: Math.floor(Date.now() / 1000),
          },
          {
            path: 'C:\\Users\\Satoshi\\Downloads\\Draft_Proposal (1).pdf',
            parent_dir: 'C:\\Users\\Satoshi\\Downloads',
            file_name: 'Draft_Proposal (1).pdf',
            extension: 'pdf',
            size_bytes: 2048576,
            modified_at: Math.floor(Date.now() / 1000) - 3600,
            created_at: Math.floor(Date.now() / 1000) - 3600,
            is_dir: false,
            content_hash: 'sha256-abc123dup',
            mime_type: 'application/pdf',
            indexed_at: Math.floor(Date.now() / 1000),
          },
        ],
      },
    ];
  }
  return await invoke<DuplicateGroup[]>('storage_get_duplicates');
};

export const getStorageOperations = async (limit: number = 50): Promise<FileOperation[]> => {
  if (!isTauriEnvironment()) {
    return [
      {
        id: 'op-preview-01',
        op_type: 'move',
        source_path: 'C:\\Users\\Satoshi\\Downloads\\paper.pdf',
        destination_path: 'C:\\Users\\Satoshi\\Documents\\Research\\paper.pdf',
        backup_path: 'C:\\Users\\Satoshi\\AppData\\Local\\Hime\\recycle\\paper.pdf',
        source_hash: 'hash-preview',
        dest_hash: 'hash-preview',
        actor: 'smart_organizer',
        approval_id: null,
        status: 'committed',
        started_at: Math.floor(Date.now() / 1000) - 600,
        completed_at: Math.floor(Date.now() / 1000) - 598,
        error_message: null,
      },
    ];
  }
  return await invoke<FileOperation[]>('storage_get_operations', { limit });
};

export const restoreStorageOperation = async (originalOpId: string): Promise<any> => {
  if (!isTauriEnvironment()) {
    return { status: 'committed', restored: true, original_op_id: originalOpId };
  }
  return await invoke('storage_restore_operation', { originalOpId });
};

export const getStorageRules = async (): Promise<FileRule[]> => {
  if (!isTauriEnvironment()) {
    return [
      {
        id: 'rule-pdf-archive',
        name: 'Auto-Organize PDF Reports',
        pattern: '*.pdf',
        target_dir: 'C:\\Users\\Satoshi\\Documents\\PDFs',
        naming_template: '{name}_{date}.{ext}',
        action: 'propose',
        is_active: true,
        created_at: Math.floor(Date.now() / 1000) - 86400,
        updated_at: Math.floor(Date.now() / 1000) - 86400,
      },
    ];
  }
  return await invoke<FileRule[]>('storage_get_rules');
};

export const toggleStorageRule = async (id: string, isActive: boolean): Promise<any> => {
  if (!isTauriEnvironment()) {
    return { updated: true, id, isActive };
  }
  return await invoke('storage_toggle_rule', { id, isActive });
};

export const searchStorageFiles = async (query: string, limit: number = 20): Promise<IndexedFile[]> => {
  if (!isTauriEnvironment()) {
    return [];
  }
  return await invoke<IndexedFile[]>('storage_search', { query, limit });
};

export const scanStorageDirectory = async (rootPath: string): Promise<{ indexed_count: number }> => {
  if (!isTauriEnvironment()) {
    return { indexed_count: 42 };
  }
  return await invoke<{ indexed_count: number }>('storage_scan', { rootPath });
};

// -----------------------------------------------------------------------------
// Phase 11: Mobile & Devices Service Endpoints
// -----------------------------------------------------------------------------

export const getDevicesList = async (): Promise<any[]> => {
  if (!isTauriEnvironment()) {
    return [
      {
        id: 'dev-pixel-preview',
        device_name: 'Pixel 8 Pro (Simulated)',
        device_type: 'android',
        cert_fingerprint: 'SHA256:PREVIEW-DEVICE-CERT-FINGERPRINT',
        paired_at: Math.floor(Date.now() / 1000) - 3600,
        last_seen_at: Math.floor(Date.now() / 1000) - 60,
        status: 'active',
      },
    ];
  }
  return await invoke<any[]>('devices_list');
};

export const startDevicePairing = async (ttlSecs: number = 300): Promise<any> => {
  if (!isTauriEnvironment()) {
    return {
      server_address: '127.0.0.1',
      server_port: 20130,
      pairing_token: 'tok-preview-12345',
      confirmation_code: '849201',
      ca_fingerprint: 'SHA256:PREVIEW-CA-FINGERPRINT',
      expires_at: Math.floor(Date.now() / 1000) + ttlSecs,
    };
  }
  return await invoke<any>('devices_pairing_start', { ttlSecs });
};

export const revokeDevice = async (deviceId: string): Promise<any> => {
  if (!isTauriEnvironment()) {
    return { device_id: deviceId, revoked: true };
  }
  return await invoke<any>('devices_revoke', { deviceId });
};

export const getDeviceStorage = async (deviceId: string): Promise<any> => {
  if (!isTauriEnvironment()) {
    return {
      device_id: deviceId,
      total_bytes: 128_000_000_000,
      free_bytes: 45_000_000_000,
      indexed_items_count: 1420,
      updated_at: Math.floor(Date.now() / 1000),
    };
  }
  return await invoke<any>('devices_storage_get', { deviceId });
};

export const getDeviceTransfers = async (deviceId?: string): Promise<any[]> => {
  if (!isTauriEnvironment()) {
    return [
      {
        id: 'xfer-001',
        device_id: deviceId || 'dev-pixel-preview',
        file_name: 'quarterly_report.pdf',
        file_size: 4_500_000,
        bytes_transferred: 4_500_000,
        direction: 'incoming',
        status: 'completed',
        sha256: 'e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855',
        started_at: Math.floor(Date.now() / 1000) - 120,
        completed_at: Math.floor(Date.now() / 1000) - 90,
      },
    ];
  }
  return await invoke<any[]>('devices_transfers_list', { deviceId });
};

// =========================================================================
// Phase 12 Productivity & Documents Service
// =========================================================================

export const getKanbanBoard = async (): Promise<any> => {
  if (!isTauriEnvironment()) {
    return {
      id: 'board-default',
      name: 'Hime Master Workspace',
      created_at: Math.floor(Date.now() / 1000) - 86400,
      updated_at: Math.floor(Date.now() / 1000),
    };
  }
  return await invoke<any>('kanban_board_get');
};

export const getKanbanColumns = async (boardId: string = 'board-default'): Promise<any[]> => {
  if (!isTauriEnvironment()) {
    return [
      { id: 'col-todo', board_id: boardId, name: 'To Do', position: 0, is_default: true, created_at: 0 },
      { id: 'col-progress', board_id: boardId, name: 'In Progress', position: 1, is_default: false, created_at: 0 },
      { id: 'col-review', board_id: boardId, name: 'Review', position: 2, is_default: false, created_at: 0 },
      { id: 'col-done', board_id: boardId, name: 'Done', position: 3, is_default: false, created_at: 0 },
    ];
  }
  return await invoke<any[]>('kanban_columns_list', { boardId });
};

export const createProductivityTask = async (
  boardId: string,
  columnId: string,
  title: string,
  description?: string,
  priority?: string,
  dueDate?: number,
  tags?: string,
  subtasks?: string
): Promise<any> => {
  if (!isTauriEnvironment()) {
    return {
      id: `task-${Date.now()}`,
      board_id: boardId,
      column_id: columnId,
      title,
      description: description || null,
      priority: priority || 'medium',
      due_date: dueDate || null,
      tags: tags || null,
      subtasks: subtasks || null,
      created_at: Math.floor(Date.now() / 1000),
      updated_at: Math.floor(Date.now() / 1000),
    };
  }
  return await invoke<any>('productivity_tasks_create', {
    boardId,
    columnId,
    title,
    description,
    priority,
    dueDate,
    tags,
    subtasks,
  });
};

export const listProductivityTasks = async (boardId?: string, columnId?: string): Promise<any[]> => {
  if (!isTauriEnvironment()) {
    return [
      {
        id: 'task-preview-1',
        board_id: boardId || 'board-default',
        column_id: columnId || 'col-todo',
        title: 'Review Skripsi Bab 4 Methodology',
        description: 'Pastikan format sitasi IEEE dan margin 4-3-3-3 terpenuhi.',
        priority: 'high',
        due_date: Math.floor(Date.now() / 1000) + 86400 * 2,
        tags: 'academic,thesis',
        subtasks: '[{"title":"Cek sitasi Mendeley","done":true},{"title":"Format margin 4-3-3-3","done":false}]',
        created_at: Math.floor(Date.now() / 1000) - 3600,
        updated_at: Math.floor(Date.now() / 1000) - 1800,
      },
      {
        id: 'task-preview-2',
        board_id: boardId || 'board-default',
        column_id: columnId || 'col-progress',
        title: 'Generate Modern Quarterly Report',
        description: 'Kompilasi log audit dan status sistem ke format DOCX.',
        priority: 'medium',
        due_date: null,
        tags: 'report',
        subtasks: null,
        created_at: Math.floor(Date.now() / 1000) - 7200,
        updated_at: Math.floor(Date.now() / 1000) - 3600,
      },
    ];
  }
  return await invoke<any[]>('productivity_tasks_list', { boardId, columnId });
};

export const moveProductivityTask = async (id: string, columnId: string): Promise<any> => {
  if (!isTauriEnvironment()) {
    return { id, column_id: columnId, updated_at: Math.floor(Date.now() / 1000) };
  }
  return await invoke<any>('productivity_tasks_move', { id, columnId });
};

export const parseNaturalTask = async (text: string): Promise<any> => {
  if (!isTauriEnvironment()) {
    return {
      title: text,
      due_date: Math.floor(Date.now() / 1000) + 86400,
      priority: 'high',
      tags: ['natural-parsed'],
    };
  }
  return await invoke<any>('productivity_tasks_parse_natural', { text });
};

export const planStudySchedule = async (
  title: string,
  subject: string,
  startTime: number,
  durationMinutes: number,
  priority?: string
): Promise<any> => {
  if (!isTauriEnvironment()) {
    return {
      id: `sched-${Date.now()}`,
      title,
      subject,
      start_time: startTime,
      end_time: startTime + durationMinutes * 60,
      priority: priority || 'medium',
      is_completed: false,
      rescheduled_from: null,
      created_at: Math.floor(Date.now() / 1000),
    };
  }
  return await invoke<any>('productivity_schedules_plan', {
    title,
    subject,
    startTime,
    durationMinutes,
    priority,
  });
};

export const listStudySchedules = async (isCompleted?: boolean): Promise<any[]> => {
  if (!isTauriEnvironment()) {
    const now = Math.floor(Date.now() / 1000);
    return [
      {
        id: 'sched-1',
        title: 'Sesi Belajar Algoritma & Struktur Data',
        subject: 'Computer Science',
        start_time: now + 3600,
        end_time: now + 7200,
        priority: 'high',
        is_completed: false,
        rescheduled_from: null,
        created_at: now,
      },
      {
        id: 'sched-2',
        title: 'Review Jurnal AI Agentic Reasoning',
        subject: 'Artificial Intelligence',
        start_time: now + 9000,
        end_time: now + 12600,
        priority: 'medium',
        is_completed: false,
        rescheduled_from: null,
        created_at: now,
      },
    ];
  }
  return await invoke<any[]>('productivity_schedules_list', { isCompleted });
};

export const getAccountabilitySettings = async (): Promise<any> => {
  if (!isTauriEnvironment()) {
    return {
      id: 'default',
      mode: 'biasa',
      opt_in_screen_time: false,
      checkin_time: '08:00',
      updated_at: Math.floor(Date.now() / 1000),
    };
  }
  return await invoke<any>('productivity_accountability_get');
};

export const setAccountabilityMode = async (
  mode: string,
  optInScreenTime?: boolean
): Promise<any> => {
  if (!isTauriEnvironment()) {
    return {
      id: 'default',
      mode,
      opt_in_screen_time: optInScreenTime ?? false,
      checkin_time: '08:00',
      updated_at: Math.floor(Date.now() / 1000),
    };
  }
  return await invoke<any>('productivity_accountability_set_mode', {
    mode,
    optInScreenTime,
  });
};

export const getAccountabilityCheckin = async (): Promise<any> => {
  if (!isTauriEnvironment()) {
    return {
      mode: 'biasa',
      greeting: 'Selamat pagi! Ada 1 tugas prioritas tinggi dan 2 jadwal belajar hari ini.',
      urgent_tasks: [],
      today_schedules: [],
      snooze_available: true,
    };
  }
  return await invoke<any>('productivity_accountability_checkin');
};

export const createDocumentReport = async (
  templateId: string,
  title: string,
  author: string,
  institution?: string,
  abstractText?: string,
  sections: any[] = [],
  targetFormat: string = 'docx',
  outputPath?: string
): Promise<any> => {
  if (!isTauriEnvironment()) {
    return {
      id: `doc-${Date.now()}`,
      template_id: templateId,
      target_format: targetFormat,
      output_path: outputPath || `C:\\Users\\Satoshi\\Documents\\${title}.${targetFormat}`,
      status: 'completed',
      error_message: null,
      created_at: Math.floor(Date.now() / 1000),
      completed_at: Math.floor(Date.now() / 1000),
    };
  }
  return await invoke<any>('documents_create_report', {
    templateId,
    title,
    author,
    institution,
    abstractText,
    sections,
    targetFormat,
    outputPath,
  });
};

export const mergePdfs = async (
  inputPaths: string[],
  outputPath?: string
): Promise<any> => {
  if (!isTauriEnvironment()) {
    return {
      id: `merge-${Date.now()}`,
      template_id: 'pdf-merge',
      target_format: 'pdf',
      output_path: outputPath || 'C:\\Users\\Satoshi\\Documents\\merged_bundle.pdf',
      status: 'completed',
      error_message: null,
      created_at: Math.floor(Date.now() / 1000),
      completed_at: Math.floor(Date.now() / 1000),
    };
  }
  return await invoke<any>('documents_merge_pdfs', { inputPaths, outputPath });
};

export const listDocumentJobs = async (limit: number = 20): Promise<any[]> => {
  if (!isTauriEnvironment()) {
    return [
      {
        id: 'job-prev-1',
        template_id: 'academic-id',
        target_format: 'docx',
        output_path: 'C:\\Users\\Satoshi\\Documents\\Skripsi_Final_Proposal.docx',
        status: 'completed',
        error_message: null,
        created_at: Math.floor(Date.now() / 1000) - 3600,
        completed_at: Math.floor(Date.now() / 1000) - 3590,
      },
    ];
  }
  return await invoke<any[]>('documents_jobs_list', { limit });
};

export const discoverArtcraft = async (): Promise<any> => {
  if (!isTauriEnvironment()) {
    return {
      wordcraft_available: false,
      figcraft_available: false,
      wordcraft_path: null,
      figcraft_path: null,
      supported_interchanges: ['docx', 'pptx', 'pdf', 'svg'],
    };
  }
  return await invoke<any>('documents_artcraft_discover');
};

// ---------------------------------------------------------------------------
// Phase 13: Browser, Integrations & Developer Workspace Service Calls
// ---------------------------------------------------------------------------

// Browser API
export const browserOpen = async (url: string): Promise<BrowserSession> => {
  if (!isTauriEnvironment()) {
    return {
      id: `session-prev-${Date.now()}`,
      url,
      title: 'Preview Web Page',
      status: 'active',
      started_at: Math.floor(Date.now() / 1000),
      last_active_at: Math.floor(Date.now() / 1000),
    };
  }
  return await invoke<BrowserSession>('browser_open', { url });
};

export const browserExtract = async (sessionId: string): Promise<BrowserExtractionResult> => {
  if (!isTauriEnvironment()) {
    return {
      session_id: sessionId,
      url: 'https://example.com/research',
      title: 'Preview Page Title',
      extracted_markdown: '<untrusted_web_content source="https://example.com/research">\n# Research Content\nThis is preview extracted content from web DOM.\n</untrusted_web_content>',
      links: ['https://example.com/about', 'https://example.com/docs'],
      timestamp: Math.floor(Date.now() / 1000),
    };
  }
  return await invoke<BrowserExtractionResult>('browser_extract', { sessionId });
};

export const browserDownload = async (url: string, targetPath: string): Promise<BrowserDownload> => {
  if (!isTauriEnvironment()) {
    return {
      id: `dl-prev-${Date.now()}`,
      source_url: url,
      target_path: targetPath,
      file_size_bytes: 1048576,
      sha256: 'e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855',
      quarantined: true,
      quarantine_reason: 'quarantined for security scan',
      downloaded_at: Math.floor(Date.now() / 1000),
    };
  }
  return await invoke<BrowserDownload>('browser_download', { url, targetPath });
};

export const browserClose = async (sessionId: string): Promise<any> => {
  if (!isTauriEnvironment()) {
    return { success: true };
  }
  return await invoke<any>('browser_close', { sessionId });
};

export const listBrowserHistory = async (limit: number = 50): Promise<BrowserHistoryItem[]> => {
  if (!isTauriEnvironment()) {
    return [
      {
        id: 'hist-prev-1',
        url: 'https://docs.rs/tauri/2.0',
        title: 'Tauri 2.0 Docs',
        visited_at: Math.floor(Date.now() / 1000) - 1800,
        metadata: '{}',
      },
      {
        id: 'hist-prev-2',
        url: 'https://github.com/rust-lang/rust',
        title: 'Rust Repository',
        visited_at: Math.floor(Date.now() / 1000) - 3600,
        metadata: '{}',
      },
    ];
  }
  return await invoke<BrowserHistoryItem[]>('browser_history_list', { limit });
};

export const listBrowserDownloads = async (limit: number = 50): Promise<BrowserDownload[]> => {
  if (!isTauriEnvironment()) {
    return [
      {
        id: 'dl-prev-1',
        source_url: 'https://example.com/dataset.csv',
        target_path: '.hime/quarantine/dataset.csv',
        file_size_bytes: 450200,
        sha256: '9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08',
        quarantined: true,
        quarantine_reason: 'Automated download quarantine defense',
        downloaded_at: Math.floor(Date.now() / 1000) - 7200,
      },
    ];
  }
  return await invoke<BrowserDownload[]>('browser_downloads_list', { limit });
};

// Integrations API
export const githubInspect = async (owner: string, repo: string): Promise<GitHubRepoSummary> => {
  if (!isTauriEnvironment()) {
    return {
      owner,
      repo,
      default_branch: 'main',
      open_issues_count: 5,
      open_pr_count: 2,
      is_private: false,
      description: 'Mock GitHub repository summary in preview mode',
    };
  }
  return await invoke<GitHubRepoSummary>('integrations_github_inspect', { owner, repo });
};

export const githubListIssues = async (owner: string, repo: string, state: string = 'open'): Promise<GitHubIssue[]> => {
  if (!isTauriEnvironment()) {
    return [
      {
        id: 101,
        title: 'Fix path boundary check in UNC Windows paths',
        state: 'open',
        created_at: new Date(Date.now() - 86400000).toISOString(),
        html_url: `https://github.com/${owner}/${repo}/issues/101`,
      },
      {
        id: 102,
        title: 'Support opt-in screen time tracking',
        state: 'open',
        created_at: new Date(Date.now() - 172800000).toISOString(),
        html_url: `https://github.com/${owner}/${repo}/issues/102`,
      },
    ];
  }
  return await invoke<GitHubIssue[]>('integrations_github_issues_list', { owner, repo, state });
};

export const githubCreateIssue = async (owner: string, repo: string, title: string, body: string): Promise<any> => {
  if (!isTauriEnvironment()) {
    return { id: 103, number: 103, title, state: 'open' };
  }
  return await invoke<any>('integrations_github_issues_create', { owner, repo, title, body });
};

export const githubCreatePr = async (
  owner: string,
  repo: string,
  title: string,
  head: string,
  base: string,
  body: string
): Promise<any> => {
  if (!isTauriEnvironment()) {
    return { id: 201, number: 201, title, state: 'open', head, base };
  }
  return await invoke<any>('integrations_github_pr_create', { owner, repo, title, head, base, body });
};

export const driveListFiles = async (parentId?: string): Promise<GoogleDriveFile[]> => {
  if (!isTauriEnvironment()) {
    return [
      {
        id: 'file-drv-1',
        name: 'Research_Notes_2026.gdoc',
        mime_type: 'application/vnd.google-apps.document',
        size_bytes: 0,
        modified_at: new Date().toISOString(),
      },
      {
        id: 'file-drv-2',
        name: 'Architecture_Diagrams.pdf',
        mime_type: 'application/pdf',
        size_bytes: 2400000,
        modified_at: new Date(Date.now() - 86400000).toISOString(),
      },
    ];
  }
  return await invoke<GoogleDriveFile[]>('integrations_drive_files_list', { parentId });
};

export const driveShareFile = async (fileId: string, email: string, role: string = 'reader'): Promise<any> => {
  if (!isTauriEnvironment()) {
    return { file_id: fileId, email, role, shared_at: Math.floor(Date.now() / 1000) };
  }
  return await invoke<any>('integrations_drive_files_share', { fileId, email, role });
};

export const listIntegrationAccounts = async (): Promise<IntegrationAccount[]> => {
  if (!isTauriEnvironment()) {
    return [
      {
        provider: 'github',
        account_id: 'satoshidk',
        display_name: 'Satoshi (GitHub)',
        scopes: 'repo,read:user',
        connected_at: Math.floor(Date.now() / 1000) - 86400 * 7,
        last_used_at: Math.floor(Date.now() / 1000) - 3600,
      },
      {
        provider: 'google_drive',
        account_id: 'satoshi.work@gmail.com',
        display_name: 'Satoshi Drive Workspace',
        scopes: 'drive.readonly,drive.file',
        connected_at: Math.floor(Date.now() / 1000) - 86400 * 5,
        last_used_at: Math.floor(Date.now() / 1000) - 1800,
      },
    ];
  }
  return await invoke<IntegrationAccount[]>('integrations_accounts_list');
};

// Developer Workspace API
export const registerDeveloperWorkspace = async (name: string, rootPath: string): Promise<DeveloperWorkspace> => {
  if (!isTauriEnvironment()) {
    return {
      id: `ws-prev-${Date.now()}`,
      name,
      root_path: rootPath,
      is_active: true,
      registered_at: Math.floor(Date.now() / 1000),
    };
  }
  return await invoke<DeveloperWorkspace>('developer_workspaces_register', { name, rootPath });
};

export const listDeveloperWorkspaces = async (): Promise<DeveloperWorkspace[]> => {
  if (!isTauriEnvironment()) {
    return [
      {
        id: 'ws-hime-default',
        name: 'Hime Project Root',
        root_path: 'X:\\folder_ai\\Hime',
        is_active: true,
        registered_at: Math.floor(Date.now() / 1000) - 86400 * 10,
      },
    ];
  }
  return await invoke<DeveloperWorkspace[]>('developer_workspaces_list');
};

export const runDeveloperCommand = async (
  workspaceId: string,
  command: string,
  args: string[]
): Promise<DeveloperCommandOutput> => {
  if (!isTauriEnvironment()) {
    return {
      command_id: `cmd-prev-${Date.now()}`,
      exit_code: 0,
      stdout: `[preview] ${command} ${args.join(' ')}\nDone in 0.42s`,
      stderr: '',
      duration_ms: 420,
      truncated: false,
    };
  }
  return await invoke<DeveloperCommandOutput>('developer_command_run', { workspaceId, command, args });
};

export const getGitStatus = async (workspaceId: string): Promise<GitStatusResult> => {
  if (!isTauriEnvironment()) {
    return {
      branch: 'phase/13-browser-dev-workspace',
      staged: [],
      modified: ['crates/hime-core/src/lib.rs', 'crates/hime-storage/src/database.rs'],
      untracked: ['crates/hime-core/src/browser/', 'crates/hime-core/src/developer/'],
      is_clean: false,
    };
  }
  return await invoke<GitStatusResult>('developer_git_status', { workspaceId });
};

export const getGitDiff = async (workspaceId: string, stagedOnly: boolean = false): Promise<{ diff: string }> => {
  if (!isTauriEnvironment()) {
    return {
      diff: 'diff --git a/crates/hime-core/src/lib.rs b/crates/hime-core/src/lib.rs\n+ pub mod browser;\n+ pub mod integrations;\n+ pub mod developer;',
    };
  }
  return await invoke<{ diff: string }>('developer_git_diff', { workspaceId, stagedOnly });
};

export const commitGitChanges = async (
  workspaceId: string,
  message: string
): Promise<{ success: boolean; commit_hash: string }> => {
  if (!isTauriEnvironment()) {
    return {
      success: true,
      commit_hash: 'a1b2c3d4e5f6',
    };
  }
  return await invoke<{ success: boolean; commit_hash: string }>('developer_git_commit', { workspaceId, message });
};

export const listSeTemplates = async (): Promise<SeTemplateItem[]> => {
  if (!isTauriEnvironment()) {
    return [
      {
        id: 'skpl-ieee830',
        title: 'Spesifikasi Kebutuhan Perangkat Lunak (SKPL / SRS)',
        description: 'Standar rekayasa perangkat lunak IEEE 830 / ISO 29148 berbahasa Indonesia.',
        standard: 'IEEE 830',
        category: 'requirements',
      },
      {
        id: 'dppl-sad',
        title: 'Deskripsi Perancangan Perangkat Lunak (DPPL / SAD)',
        description: 'Standar perancangan arsitektur, modul, ERD, dan antarmuka perangkat lunak.',
        standard: 'IEEE 1016',
        category: 'architecture',
      },
      {
        id: 'test-plan-ieee829',
        title: 'Master Test Plan & Verification Matrix',
        description: 'Standar perencanaan pengujian IEEE 829.',
        standard: 'IEEE 829',
        category: 'testing',
      },
      {
        id: 'code-review-checklist',
        title: 'Rigorous Code Review & Pre-Commit Audit Checklist',
        description: 'Daftar periksa keamanan, batas memori, rahasia, dan sanitasi input.',
        standard: 'Hime Standard',
        category: 'quality',
      },
    ];
  }
  return await invoke<SeTemplateItem[]>('developer_templates_list');
};

export const renderSeTemplate = async (
  templateId: string,
  projectName: string,
  author: string
): Promise<{ template_id: string; rendered_markdown: string }> => {
  if (!isTauriEnvironment()) {
    return {
      template_id: templateId,
      rendered_markdown: `# SPESIFIKASI KEBUTUHAN PERANGKAT LUNAK (SKPL)\n## ${projectName}\nOleh: ${author}\n\n### 1. Pendahuluan\nDokumen ini mendefinisikan kebutuhan perangkat lunak...`,
    };
  }
  return await invoke<{ template_id: string; rendered_markdown: string }>('developer_templates_render', {
    templateId,
    projectName,
    author,
  });
};

// ---------------------------------------------------------------------------
// Phase 14: Specialist Agents, Skills & Proactive Assistance Service Calls
// ---------------------------------------------------------------------------

export const listSpecialists = async (activeOnly?: boolean): Promise<SpecialistDefinition[]> => {
  if (!isTauriEnvironment()) {
    return [
      {
        id: 'specialist.software_engineer',
        name: 'Hime Software Engineer',
        role_title: 'Senior Systems & Software Engineer',
        domain: 'developer',
        responsibilities: [
          'Analisis kebutuhan perangkat lunak dan arsitektur (IEEE 830/1016)',
          'Eksekusi perintah build, lint, dan test sandboxed',
          'Inspeksi Git, safe pre-commit secret scan, dan refactoring aman',
        ],
        system_prompt: 'You are the Hime Software Engineering Specialist.',
        allowed_tools: ['dev.run_command', 'dev.git_status', 'files.stat'],
        max_permission_level: 'L2',
        resource_budget: { max_tokens: 6144, timeout_secs: 90, max_concurrency: 1 },
        is_active: true,
        evaluation_score: 96.5,
        version: '1.0.0',
      },
      {
        id: 'specialist.data_ai',
        name: 'Hime Data & AI Engineer',
        role_title: 'Data Analyst, Scientist & ML Engineer',
        domain: 'data_ai',
        responsibilities: [
          'Profil dataset, statistik deskriptif, dan deteksi nilai hilang',
          'Pembersihan data dan validasi integritas dataset',
          'Perancangan eksperimen ML tanpa kebocoran data dan model card',
        ],
        system_prompt: 'You are the Hime Data & AI Specialist.',
        allowed_tools: ['data_ai.profile_dataset', 'data_ai.design_experiment', 'files.stat'],
        max_permission_level: 'L1',
        resource_budget: { max_tokens: 4096, timeout_secs: 60, max_concurrency: 1 },
        is_active: true,
        evaluation_score: 95.0,
        version: '1.0.0',
      },
      {
        id: 'specialist.research',
        name: 'Hime Web Researcher',
        role_title: 'Autonomous Web & Documentation Researcher',
        domain: 'browser',
        responsibilities: [
          'Riset web publik dengan proteksi SSRF dan anti-prompt injection',
          'Ekstraksi teks keterbacaan DOM dan sitasi sumber terpercaya',
        ],
        system_prompt: 'You are the Hime Web Research Specialist.',
        allowed_tools: ['browser.open_page', 'browser.extract', 'browser.download'],
        max_permission_level: 'L1',
        resource_budget: { max_tokens: 5120, timeout_secs: 60, max_concurrency: 1 },
        is_active: true,
        evaluation_score: 94.0,
        version: '1.0.0',
      },
      {
        id: 'specialist.document',
        name: 'Hime Document Specialist',
        role_title: 'Office Document & Academic Formatting Specialist',
        domain: 'documents',
        responsibilities: [
          'Sintesis dokumen standar akademik Indonesia (TNR 12, margin 4-3-3-3)',
          'Penyusunan laporan modern DOCX/HTML dan verifikasi visual',
        ],
        system_prompt: 'You are the Hime Document Specialist.',
        allowed_tools: ['docs.create_report', 'docs.merge_pdfs'],
        max_permission_level: 'L2',
        resource_budget: { max_tokens: 4096, timeout_secs: 60, max_concurrency: 1 },
        is_active: true,
        evaluation_score: 97.0,
        version: '1.0.0',
      },
    ];
  }
  return await invoke<SpecialistDefinition[]>('specialists_list', { activeOnly });
};

export const getSpecialist = async (id: string): Promise<SpecialistDefinition> => {
  return await invoke<SpecialistDefinition>('specialists_get', { id });
};

export const delegateToSpecialist = async (
  specialistId: string,
  taskId: string,
  inputPrompt: string
): Promise<DelegationResult> => {
  if (!isTauriEnvironment()) {
    return {
      delegation_id: `del-prev-${Date.now()}`,
      specialist_id: specialistId,
      task_id: taskId,
      status: 'completed',
      output_summary: `[Preview] Tugas berhasil diproses oleh spesialis '${specialistId}'.`,
      evidence: ['Batas sandbox dipatuhi', 'Resource budget terkontrol'],
      tokens_used: 350,
      duration_ms: 420,
    };
  }
  return await invoke<DelegationResult>('specialists_delegate', {
    specialistId,
    taskId,
    inputPrompt,
  });
};

export const listSpecialistDelegations = async (
  specialistId?: string,
  limit?: number
): Promise<DelegationResult[]> => {
  if (!isTauriEnvironment()) {
    return [
      {
        delegation_id: 'del-sample-1',
        specialist_id: specialistId || 'specialist.software_engineer',
        task_id: 'task-prev-101',
        status: 'completed',
        output_summary: 'Analisis rekayasa selesai. Working tree bersih dan safe commit diverifikasi.',
        evidence: ['Git status OK', 'Pre-commit scanner passed'],
        tokens_used: 410,
        duration_ms: 512,
      },
    ];
  }
  return await invoke<DelegationResult[]>('specialists_delegations_list', {
    specialistId,
    limit,
  });
};

export const evaluateSpecialist = async (specialistId: string): Promise<EvaluationReport> => {
  if (!isTauriEnvironment()) {
    return {
      specialist_id: specialistId,
      scenarios_evaluated: 2,
      scenarios_passed: 2,
      pass_rate: 100.0,
      average_tokens: 380,
      average_duration_ms: 450,
      overall_score: 96.5,
      evaluation_summary: `Evaluasi spesialis '${specialistId}' lulus 100% pada seluruh skenario regresi dan kapabilitas.`,
    };
  }
  return await invoke<EvaluationReport>('specialists_evaluate', { specialistId });
};

export const listProactiveSuggestions = async (
  status?: string,
  limit?: number
): Promise<ProactiveSuggestionItem[]> => {
  if (!isTauriEnvironment()) {
    return [
      {
        id: 'sug-sample-1',
        title: 'Rekomendasi Fokus & Belajar',
        content: 'Ada blok belajar Skripsi Bab 4 dalam 15 menit. Siapkan materi atau aktifkan Mode Strict?',
        category: 'schedule',
        status: 'pending',
        score: 0.95,
        created_at: Math.floor(Date.now() / 1000) - 600,
        responded_at: null,
      },
      {
        id: 'sug-sample-2',
        title: 'Pembersihan Unduhan Tertunda',
        content: 'Ditemukan 5 file baru di folder Downloads. Pindahkan ke arsip proyek?',
        category: 'storage',
        status: 'pending',
        score: 0.88,
        created_at: Math.floor(Date.now() / 1000) - 1800,
        responded_at: null,
      },
    ];
  }
  return await invoke<ProactiveSuggestionItem[]>('proactive_suggestions_list', {
    status,
    limit,
  });
};

export const evaluateProactiveSuggestions = async (): Promise<ProactiveSuggestionItem[]> => {
  if (!isTauriEnvironment()) {
    return await listProactiveSuggestions();
  }
  return await invoke<ProactiveSuggestionItem[]>('proactive_suggestions_evaluate');
};

export const respondProactiveSuggestion = async (
  suggestionId: string,
  action: string
): Promise<any> => {
  if (!isTauriEnvironment()) {
    return { success: true };
  }
  return await invoke<any>('proactive_suggestions_respond', {
    suggestionId,
    action,
  });
};

export const getProactiveSettings = async (): Promise<ProactiveSettings> => {
  if (!isTauriEnvironment()) {
    return {
      is_enabled: true,
      cooldown_minutes: 30,
      muted_until: null,
      updated_at: Math.floor(Date.now() / 1000),
    };
  }
  return await invoke<ProactiveSettings>('proactive_settings_get');
};

export const setProactiveSettings = async (
  isEnabled: boolean,
  cooldownMinutes: number,
  muteHours?: number
): Promise<any> => {
  if (!isTauriEnvironment()) {
    return { success: true };
  }
  return await invoke<any>('proactive_settings_set', {
    isEnabled,
    cooldownMinutes,
    muteHours,
  });
};

export const profileDataset = async (csvData: string): Promise<DatasetProfile> => {
  if (!isTauriEnvironment()) {
    return {
      row_count: 5,
      column_count: 3,
      columns: [
        { name: 'id', inferred_type: 'numeric', missing_count: 0, missing_percentage: 0, min: 1, max: 5, mean: 3 },
        { name: 'score', inferred_type: 'numeric', missing_count: 0, missing_percentage: 0, min: 70, max: 98, mean: 84 },
        { name: 'category', inferred_type: 'text', missing_count: 0, missing_percentage: 0, min: null, max: null, mean: null },
      ],
      summary_markdown: '### Ringkasan Profil Dataset Preview\n- 5 baris, 3 kolom',
    };
  }
  return await invoke<DatasetProfile>('data_ai_profile_dataset', { csvData });
};

export const designExperiment = async (
  targetVariable: string,
  problemType: string,
  numFeatures: number
): Promise<ExperimentDesign> => {
  if (!isTauriEnvironment()) {
    return {
      target_variable: targetVariable,
      problem_type: problemType,
      baseline_models: ['LogisticRegression', 'RandomForestClassifier'],
      recommended_metrics: ['F1-Macro', 'ROC-AUC'],
      validation_strategy: 'Stratified 5-Fold Cross Validation without data leakage',
      uncertainty_and_limitations: 'Predictions must report probabilities and out-of-distribution risks.',
      execution_constraints: 'CPU friendly (Intel i5, max 1GB RAM budget)',
      model_card_markdown: `# MODEL CARD: ${targetVariable}\n- Problem: ${problemType}\n- Features: ${numFeatures}`,
    };
  }
  return await invoke<ExperimentDesign>('data_ai_design_experiment', {
    targetVariable,
    problemType,
    numFeatures,
  });
};


