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


