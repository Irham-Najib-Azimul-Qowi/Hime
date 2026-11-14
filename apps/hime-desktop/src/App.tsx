import React, { useState, useEffect, useCallback } from 'react';
import { DynamicIsland } from './features/island/DynamicIsland';
import { OmniboxModal } from './features/omnibox/OmniboxModal';
import { OverviewView } from './features/overview/OverviewView';
import { ConversationView } from './features/conversation/ConversationView';
import { ActivityStream } from './features/activity/ActivityStream';
import { TasksView } from './features/tasks/TasksView';
import { ProductivityView } from './features/productivity/ProductivityView';
import { DocumentsView } from './features/documents/DocumentsView';
import { DeveloperWorkspaceView } from './features/developer/DeveloperWorkspaceView';
import { BrowserIntegrationsView } from './features/browser/BrowserIntegrationsView';
import { SpecialistsView } from './features/specialists/SpecialistsView';
import { StorageView } from './features/storage/StorageView';
import { DevicesView } from './features/devices/DevicesView';
import { NotificationCenter } from './features/notifications/NotificationCenter';
import { SettingsView } from './features/settings/SettingsView';
import { Badge } from './components/ui/Badge';
import { createCommandRegistry } from './services/commandRegistry';
import {
  pingCore,
  getSystemStatus,
  getActivityEvents,
  executeTool,
  resolveApproval,
  createCoreTask,
  getCoreTasks,
  cancelCoreTask,
  windowSetAlwaysOnTop,
  windowSetSizeMode,
  windowMinimize,
  windowMaximize,
  windowClose,
} from './services/ipcService';
import { SystemStatus, PingResult, ToolExecutionResult } from './types/ipc';
import { AuditEvent } from './types/events';
import { HimeTask, TaskPriority } from './types/tasks';
import { NotificationItem } from './types/notifications';
import { DesktopView, ThemeMode, IslandPresentationMode } from './types/theme';

export const App: React.FC = () => {
  // Navigation & Theme State - Default to Island-First AI Assistant
  const [currentView, setCurrentView] = useState<DesktopView>('assistant');
  const [theme, setTheme] = useState<ThemeMode>(() => {
    return (localStorage.getItem('hime_theme') as ThemeMode) || 'dark';
  });
  const [islandMode, setIslandMode] = useState<IslandPresentationMode>(() => {
    return (localStorage.getItem('hime_island_mode') as IslandPresentationMode) || 'floating';
  });
  const [alwaysOnTop, setAlwaysOnTop] = useState(true);
  const [isOmniboxOpen, setIsOmniboxOpen] = useState(false);
  const [windowSizeMode, setWindowSizeMode] = useState<'compact' | 'expanded'>('compact');

  // Core & Data State
  const [status, setStatus] = useState<SystemStatus | null>(null);
  const [isCoreOnline, setIsCoreOnline] = useState(false);
  const [activeActivity, setActiveActivity] = useState<string | null>(null);
  const [translationResult, setTranslationResult] = useState<{ original: string; translated: string } | null>(null);
  const [events, setEvents] = useState<AuditEvent[]>([]);
  const [isEventsLoading, setIsEventsLoading] = useState(false);
  const [tasks, setTasks] = useState<HimeTask[]>([]);
  const [isTasksLoading, setIsTasksLoading] = useState(false);
  const [notifications, setNotifications] = useState<NotificationItem[]>([
    {
      id: 'notif-1',
      title: 'Hime Desktop Initialized',
      message: 'Desktop presentation shell connected to local system bus.',
      category: 'system',
      priority: 'normal',
      timestamp: Date.now(),
      read: false,
    },
  ]);

  // Apply theme to document
  useEffect(() => {
    document.documentElement.setAttribute('data-theme', theme);
    localStorage.setItem('hime_theme', theme);
  }, [theme]);

  // Apply island presentation mode
  useEffect(() => {
    localStorage.setItem('hime_island_mode', islandMode);
  }, [islandMode]);

  // Fetch Core telemetry
  const refreshCoreStatus = useCallback(async () => {
    try {
      const res = await getSystemStatus();
      setStatus(res);
      setIsCoreOnline(res.core === 'online');
    } catch {
      setIsCoreOnline(false);
    }
  }, []);

  const refreshEvents = useCallback(async () => {
    setIsEventsLoading(true);
    try {
      const evs = await getActivityEvents(40);
      setEvents((prev) => {
        const map = new Map<number, AuditEvent>();
        for (const e of evs) map.set(e.id, e);
        for (const e of prev) {
          if (!map.has(e.id)) map.set(e.id, e);
        }
        return Array.from(map.values()).sort((a, b) => b.id - a.id).slice(0, 50);
      });
    } catch {
      // Core might be offline
    } finally {
      setIsEventsLoading(false);
    }
  }, []);

  const refreshTasks = useCallback(async () => {
    setIsTasksLoading(true);
    try {
      const ts = await getCoreTasks();
      setTasks(ts);
    } catch {
      // Core offline
    } finally {
      setIsTasksLoading(false);
    }
  }, []);

  // Polling loop
  useEffect(() => {
    refreshCoreStatus();
    refreshEvents();
    refreshTasks();

    const interval = setInterval(() => {
      refreshCoreStatus();
    }, 4000);

    return () => clearInterval(interval);
  }, [refreshCoreStatus, refreshEvents, refreshTasks]);

  // Interactive Ping handler
  const handlePing = async (): Promise<PingResult> => {
    setActiveActivity('Pinging Core IPC...');
    try {
      const res = await pingCore();
      await refreshCoreStatus();
      await refreshEvents();
      return res;
    } finally {
      setActiveActivity(null);
    }
  };

  // Interactive Policy Test handler
  const handlePolicyTest = async (): Promise<ToolExecutionResult> => {
    setActiveActivity('Evaluating PEP/PDP policy...');
    try {
      const res = await executeTool('file_write', 'workspace/build-output.log');
      await refreshEvents();

      if (res.decision === 'pending_approval' && res.request_id) {
        setNotifications((prev) => [
          {
            id: `notif-appr-${Date.now()}`,
            title: 'Policy Approval Required',
            message: `Tool requests authorization: ${res.description || 'write file'}. (Expires in 60s)`,
            category: 'permission',
            priority: 'high',
            timestamp: Date.now(),
            read: false,
            actionRequired: true,
            actionType: 'approval',
            contextData: {
              requestId: res.request_id,
              expiresAt: res.expires_at,
            },
          },
          ...prev,
        ]);
      }
      return res;
    } finally {
      setActiveActivity(null);
    }
  };

  // Interactive Action Approval handler
  const handleApproveAction = async (requestId: string) => {
    setActiveActivity(`Submitting approval for ${requestId}...`);
    try {
      await resolveApproval(requestId, 'approve');
      await refreshEvents();
      await refreshCoreStatus();
      setNotifications((prev) =>
        prev.map((n) =>
          n.contextData?.requestId === requestId
            ? { ...n, read: true, message: `${n.message} [APPROVED BY USER VIA CORE IPC]` }
            : n
        )
      );
    } catch (err) {
      console.error('Core approval error:', err);
    } finally {
      setActiveActivity(null);
    }
  };

  // Interactive Action Denial handler
  const handleDenyAction = async (requestId: string) => {
    setActiveActivity(`Submitting denial for ${requestId}...`);
    try {
      await resolveApproval(requestId, 'deny');
      await refreshEvents();
      await refreshCoreStatus();
      setNotifications((prev) =>
        prev.map((n) =>
          n.contextData?.requestId === requestId
            ? { ...n, read: true, message: `${n.message} [DENIED BY USER VIA CORE IPC]` }
            : n
        )
      );
    } catch (err) {
      console.error('Core denial error:', err);
    } finally {
      setActiveActivity(null);
    }
  };

  // Create Task handler
  const handleCreateTask = async (name: string, description: string, priority: TaskPriority) => {
    setActiveActivity(`Registering task: ${name}...`);
    try {
      const newTask = await createCoreTask(name, description, priority);
      setTasks((prev) => [newTask, ...prev]);
      await refreshEvents();
      await refreshCoreStatus();
      setNotifications((prev) => [
        {
          id: `notif-task-${Date.now()}`,
          title: 'Task Registered',
          message: `Task '${name}' (${priority}) submitted to Job Supervisor.`,
          category: 'task',
          priority: 'normal',
          timestamp: Date.now(),
          read: false,
        },
        ...prev,
      ]);
    } finally {
      setActiveActivity(null);
    }
  };

  // Cancel Task handler
  const handleCancelTask = async (id: string) => {
    try {
      await cancelCoreTask(id);
      await refreshTasks();
      await refreshEvents();
    } catch (err) {
      console.error('Failed to cancel task:', err);
    }
  };

  // Toggle Always On Top
  const handleToggleAlwaysOnTop = async (enabled: boolean) => {
    setAlwaysOnTop(enabled);
    await windowSetAlwaysOnTop(enabled);
  };

  // Toggle Window Size
  const handleToggleWindowSize = async (mode: 'compact' | 'expanded') => {
    setWindowSizeMode(mode);
    await windowSetSizeMode(mode);
  };

  // Global Keyboard Shortcuts
  useEffect(() => {
    const handleGlobalKeyDown = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'k') {
        e.preventDefault();
        setIsOmniboxOpen((prev) => !prev);
      } else if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'p') {
        e.preventDefault();
        handlePing();
      } else if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 't') {
        e.preventDefault();
        setTheme((prev) => (prev === 'dark' ? 'light' : 'dark'));
      } else if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'm') {
        e.preventDefault();
        const nextIslandMode = islandMode === 'floating' ? 'integrated' : 'floating';
        setIslandMode(nextIslandMode);
        handleToggleWindowSize(nextIslandMode === 'floating' ? 'compact' : 'expanded');
      } else if (e.altKey) {
        if (e.key === '1') setCurrentView('assistant');
        if (e.key === '2') setCurrentView('tasks');
        if (e.key === '3') setCurrentView('productivity');
        if (e.key === '4') setCurrentView('storage');
        if (e.key === '5') setCurrentView('documents');
        if (e.key === '6') setCurrentView('developer');
        if (e.key === '7') setCurrentView('browser');
        if (e.key === '8') setCurrentView('specialists');
        if (e.key === '9') setCurrentView('activity');
        if (e.key === '0') setCurrentView('settings');
      }
    };

    window.addEventListener('keydown', handleGlobalKeyDown);
    return () => window.removeEventListener('keydown', handleGlobalKeyDown);
  }, [windowSizeMode, islandMode]);

  // Command Registry Context
  const commandRegistry = createCommandRegistry({
    navigate: (v) => {
      setCurrentView(v);
      if (islandMode === 'floating') {
        setIslandMode('integrated');
        handleToggleWindowSize('expanded');
      }
    },
    toggleTheme: () => setTheme((prev) => (prev === 'dark' ? 'light' : 'dark')),
    triggerPing: async () => {
      await handlePing();
    },
    triggerPolicyTest: async () => {
      await handlePolicyTest();
    },
    toggleWindowMode: async () => {
      const nextMode = windowSizeMode === 'compact' ? 'expanded' : 'compact';
      await handleToggleWindowSize(nextMode);
    },
    theme,
  });

  const unreadCount = notifications.filter((n) => !n.read).length;

  // =========================================================================
  // PRESENTATION MODE 1: FLOATING DYNAMIC ISLAND (DEFAULT HERO VIEW)
  // =========================================================================
  if (islandMode === 'floating') {
    return (
      <div
        style={{
          width: '100vw',
          height: '100vh',
          backgroundColor: 'transparent',
          display: 'flex',
          flexDirection: 'column',
          alignItems: 'center',
          justifyContent: 'flex-start',
          padding: '6px 12px',
          overflow: 'hidden',
        }}
      >
        <DynamicIsland
          status={status}
          isCoreOnline={isCoreOnline}
          activeActivity={activeActivity}
          translationResult={translationResult}
          onClearTranslation={() => setTranslationResult(null)}
          onOpenOmnibox={() => setIsOmniboxOpen(true)}
          onOpenAssistant={() => {
            setIslandMode('integrated');
            setCurrentView('assistant');
            handleToggleWindowSize('expanded');
          }}
          onOpenWorkstation={() => {
            setIslandMode('integrated');
            handleToggleWindowSize('expanded');
          }}
          onMinimizeWindow={() => windowMinimize()}
          onCloseWindow={() => windowClose()}
          unreadCount={unreadCount}
          isFloatingMode={true}
        />

        {/* Omnibox Modal in Island Mode */}
        <OmniboxModal
          isOpen={isOmniboxOpen}
          onClose={() => setIsOmniboxOpen(false)}
          commands={commandRegistry}
        />
      </div>
    );
  }

  // =========================================================================
  // PRESENTATION MODE 2: FULL WORKSTATION ENVIRONMENT (ACCESSED ON DEMAND)
  // =========================================================================
  const navItems = [
    { id: 'assistant', label: 'AI Assistant', icon: '💬', shortcut: 'Alt+1' },
    { id: 'tasks', label: 'Papan Tugas', icon: '📋', shortcut: 'Alt+2' },
    { id: 'productivity', label: 'Produktivitas', icon: '🎯', shortcut: 'Alt+3' },
    { id: 'storage', label: 'Penyimpanan', icon: '💾', shortcut: 'Alt+4' },
    { id: 'documents', label: 'Dokumen & Akademik', icon: '📄', shortcut: 'Alt+5' },
    { id: 'developer', label: 'Developer Workspace', icon: '💻', shortcut: 'Alt+6' },
    { id: 'browser', label: 'Browser & Web', icon: '🌐', shortcut: 'Alt+7' },
    { id: 'specialists', label: 'Spesialis Data & AI', icon: '🤖', shortcut: 'Alt+8' },
    { id: 'activity', label: 'Log Aktivitas', icon: '⚡', shortcut: 'Alt+9' },
    { id: 'devices', label: 'Perangkat (Mobile)', icon: '📱' },
    { id: 'notifications', label: 'Notifikasi', icon: '🔔', badge: unreadCount > 0 ? unreadCount : undefined },
    { id: 'settings', label: 'Pengaturan', icon: '⚙️', shortcut: 'Alt+0' },
    { id: 'overview', label: 'Diagnostik Sistem', icon: '📊' },
  ];

  return (
    <div
      style={{
        display: 'flex',
        flexDirection: 'column',
        width: '100vw',
        height: '100vh',
        backgroundColor: 'var(--bg-app)',
        color: 'var(--text-primary)',
        overflow: 'hidden',
      }}
    >
      {/* Titlebar & Window Header with Return to Island Button */}
      <div
        className="app-drag-region"
        style={{
          height: '48px',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          padding: '0 16px',
          borderBottom: '1px solid var(--border-subtle)',
          backgroundColor: 'var(--bg-surface-1)',
          zIndex: 100,
        }}
      >
        <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
          <span style={{ fontSize: 'var(--text-sm)', fontWeight: 700, letterSpacing: '0.04em', color: '#f8fafc' }}>
            ✨ HIME WORKSTATION
          </span>
          <Badge variant={isCoreOnline ? 'success' : 'danger'} size="xs" dot>
            {isCoreOnline ? 'Core Online' : 'Core Offline'}
          </Badge>

          {/* Button to return to Floating Dynamic Island */}
          <button
            onClick={() => {
              setIslandMode('floating');
              handleToggleWindowSize('compact');
            }}
            className="app-no-drag btn-pressable"
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: '6px',
              backgroundColor: 'rgba(139, 92, 246, 0.15)',
              border: '1px solid rgba(139, 92, 246, 0.35)',
              borderRadius: 'var(--radius-pill)',
              padding: '4px 12px',
              color: '#c4b5fd',
              fontSize: '11px',
              fontWeight: 600,
              cursor: 'pointer',
            }}
            title="Kembali ke Mode Dynamic Island Mengambang di Atas Layar"
          >
            ✨ Mode Island Mengambang
          </button>
        </div>

        {/* Window action buttons */}
        <div className="app-no-drag" style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
          <button
            onClick={() => windowMinimize()}
            className="btn-pressable"
            style={{
              background: 'transparent',
              border: 'none',
              color: 'var(--text-muted)',
              cursor: 'pointer',
              padding: '4px 8px',
              borderRadius: 'var(--radius-xs)',
              fontSize: 'var(--text-xs)',
            }}
            title="Minimize"
          >
            —
          </button>
          <button
            onClick={() => windowMaximize()}
            className="btn-pressable"
            style={{
              background: 'transparent',
              border: 'none',
              color: 'var(--text-muted)',
              cursor: 'pointer',
              padding: '4px 8px',
              borderRadius: 'var(--radius-xs)',
              fontSize: 'var(--text-xs)',
            }}
            title="Maximize / Restore"
          >
            ◻
          </button>
          <button
            onClick={() => windowClose()}
            className="btn-pressable"
            style={{
              background: 'transparent',
              border: 'none',
              color: 'var(--text-muted)',
              cursor: 'pointer',
              padding: '4px 8px',
              borderRadius: 'var(--radius-xs)',
              fontSize: 'var(--text-xs)',
            }}
            title="Close"
          >
            ✕
          </button>
        </div>
      </div>

      {/* Main Content Layout with Sidebar Navigation */}
      <div style={{ display: 'flex', flex: 1, overflow: 'hidden' }}>
        {/* Navigation Sidebar */}
        <div
          style={{
            width: '220px',
            backgroundColor: 'var(--bg-surface-1)',
            borderRight: '1px solid var(--border-subtle)',
            display: 'flex',
            flexDirection: 'column',
            justifyContent: 'space-between',
            padding: '14px 10px',
            overflowY: 'auto',
          }}
        >
          {/* Navigation Items */}
          <div style={{ display: 'flex', flexDirection: 'column', gap: '3px' }}>
            {navItems.map((item) => {
              const isActive = currentView === item.id;
              return (
                <button
                  key={item.id}
                  onClick={() => setCurrentView(item.id as DesktopView)}
                  className="btn-pressable"
                  style={{
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'space-between',
                    padding: '8px 12px',
                    borderRadius: 'var(--radius-sm)',
                    backgroundColor: isActive ? 'var(--bg-surface-3)' : 'transparent',
                    color: isActive ? '#f8fafc' : 'var(--text-secondary)',
                    border: 'none',
                    fontWeight: isActive ? 600 : 500,
                    fontSize: '12px',
                    cursor: 'pointer',
                    textAlign: 'left',
                    transition: 'background-color 150ms ease-out',
                  }}
                >
                  <div style={{ display: 'flex', alignItems: 'center', gap: '10px' }}>
                    <span style={{ fontSize: '14px' }}>{item.icon}</span>
                    <span>{item.label}</span>
                  </div>
                  {item.badge !== undefined && (
                    <Badge variant="warning" size="xs">
                      {item.badge}
                    </Badge>
                  )}
                </button>
              );
            })}
          </div>

          {/* Quick status in sidebar footer */}
          <div
            style={{
              padding: '10px 12px',
              backgroundColor: 'var(--bg-surface-2)',
              borderRadius: 'var(--radius-sm)',
              border: '1px solid var(--border-subtle)',
              fontSize: '11px',
              display: 'flex',
              flexDirection: 'column',
              gap: '4px',
              marginTop: '12px',
            }}
          >
            <div style={{ color: 'var(--text-muted)' }}>Status Sistem</div>
            <div style={{ fontWeight: 600, color: isCoreOnline ? 'var(--accent-success)' : 'var(--accent-danger)' }}>
              {isCoreOnline ? '● Hime Core Aktif' : '○ Core Terputus'}
            </div>
            <div style={{ color: 'var(--text-dim)', fontSize: '10px' }}>
              Tekan Ctrl+K untuk perintah
            </div>
          </div>
        </div>

        {/* Main Work Area */}
        <div
          style={{
            flex: 1,
            padding: '24px',
            overflowY: 'auto',
            backgroundColor: 'var(--bg-app)',
          }}
        >
          {/* Offline Warning Banner if Core is disconnected */}
          {!isCoreOnline && (
            <div
              className="animate-slide-down"
              style={{
                marginBottom: '16px',
                padding: '12px 16px',
                backgroundColor: 'var(--accent-danger-subtle)',
                border: '1px solid rgba(239, 68, 68, 0.3)',
                borderRadius: 'var(--radius-md)',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'space-between',
              }}
            >
              <div style={{ display: 'flex', alignItems: 'center', gap: '10px' }}>
                <span style={{ fontSize: 'var(--text-lg)' }}>⚠️</span>
                <div>
                  <div style={{ fontWeight: 600, fontSize: 'var(--text-xs)', color: 'var(--accent-danger)' }}>
                    HIME CORE SEDANG OFFLINE ATAU TIDAK DAPAT DIHUBUNGI
                  </div>
                  <div style={{ fontSize: 'var(--text-2xs)', color: 'var(--text-secondary)' }}>
                    Windows Named Pipe `\\.\pipe\hime-ipc` tidak merespon. Pastikan daemon `hime-core.exe` berjalan.
                  </div>
                </div>
              </div>
              <button
                onClick={refreshCoreStatus}
                className="btn-pressable"
                style={{
                  backgroundColor: 'var(--bg-surface-1)',
                  border: '1px solid var(--border-medium)',
                  borderRadius: 'var(--radius-sm)',
                  padding: '4px 12px',
                  fontSize: 'var(--text-xs)',
                  color: 'var(--text-primary)',
                  cursor: 'pointer',
                }}
              >
                Coba Hubungkan Ulang
              </button>
            </div>
          )}

          {/* View Dispatcher - Assistant is the Hero View */}
          {currentView === 'assistant' && (
            <ConversationView
              isCoreOnline={isCoreOnline}
              onSendSystemCommand={async (cmd) => {
                if (cmd === 'system.ping') {
                  const res = await pingCore();
                  return `Ping latency: ${res.latency_ms}ms • Response: ${JSON.stringify(res.core_response)}`;
                } else if (cmd === 'system.status') {
                  const st = await getSystemStatus();
                  return `Status: ${st.status} • Core: ${st.core} • Tasks: ${st.active_tasks} active / ${st.total_tasks} total`;
                } else if (cmd.includes('policy') || cmd.includes('file_write')) {
                  const res = await handlePolicyTest();
                  return `Policy test result: ${JSON.stringify(res)}`;
                } else {
                  return `Perintah '${cmd}' diterima dan diakui oleh Hime Core.`;
                }
              }}
            />
          )}

          {currentView === 'tasks' && (
            <TasksView
              tasks={tasks}
              isLoading={isTasksLoading}
              onCreateTask={handleCreateTask}
              onCancelTask={handleCancelTask}
              onRefresh={refreshTasks}
            />
          )}

          {currentView === 'productivity' && (
            <ProductivityView />
          )}

          {currentView === 'storage' && (
            <StorageView />
          )}

          {currentView === 'documents' && (
            <DocumentsView />
          )}

          {currentView === 'developer' && (
            <DeveloperWorkspaceView />
          )}

          {currentView === 'browser' && (
            <BrowserIntegrationsView />
          )}

          {currentView === 'specialists' && (
            <SpecialistsView />
          )}

          {currentView === 'activity' && (
            <ActivityStream
              events={events}
              isLoading={isEventsLoading}
              onRefresh={refreshEvents}
            />
          )}

          {currentView === 'devices' && (
            <DevicesView />
          )}

          {currentView === 'notifications' && (
            <NotificationCenter
              notifications={notifications}
              onDismiss={(id) => setNotifications((prev) => prev.filter((n) => n.id !== id))}
              onMarkAllRead={() => setNotifications((prev) => prev.map((n) => ({ ...n, read: true })))}
              onApproveAction={handleApproveAction}
              onDenyAction={handleDenyAction}
            />
          )}

          {currentView === 'settings' && (
            <SettingsView
              theme={theme}
              onToggleTheme={() => setTheme((prev) => (prev === 'dark' ? 'light' : 'dark'))}
              islandMode={islandMode}
              onToggleIslandMode={(m) => {
                setIslandMode(m);
                handleToggleWindowSize(m === 'floating' ? 'compact' : 'expanded');
              }}
              alwaysOnTop={alwaysOnTop}
              onToggleAlwaysOnTop={handleToggleAlwaysOnTop}
              onToggleWindowSize={handleToggleWindowSize}
              onOpenOmnibox={() => setIsOmniboxOpen(true)}
            />
          )}

          {/* System Diagnostics - Moved away from the front page */}
          {currentView === 'overview' && (
            <OverviewView
              status={status}
              isCoreOnline={isCoreOnline}
              onPing={handlePing}
              onTestPolicy={handlePolicyTest}
            />
          )}
        </div>
      </div>

      {/* Omnibox Modal */}
      <OmniboxModal
        isOpen={isOmniboxOpen}
        onClose={() => setIsOmniboxOpen(false)}
        commands={commandRegistry}
      />
    </div>
  );
};

export default App;
