import { CommandItem } from '../types/commands';
import { DesktopView, ThemeMode } from '../types/theme';

export interface CommandContext {
  navigate: (view: DesktopView) => void;
  toggleTheme: () => void;
  triggerPing: () => Promise<void>;
  triggerPolicyTest: () => Promise<void>;
  toggleWindowMode: () => Promise<void>;
  theme: ThemeMode;
}

export const createCommandRegistry = (ctx: CommandContext): CommandItem[] => {
  return [
    // Navigation Category
    {
      id: 'nav-overview',
      title: 'Go to Overview',
      description: 'System health, telemetry, and architecture status',
      category: 'navigation',
      shortcut: 'Alt+1',
      action: () => ctx.navigate('overview'),
    },
    {
      id: 'nav-assistant',
      title: 'Go to Assistant',
      description: 'Hime personal conversation interface and system interaction',
      category: 'navigation',
      shortcut: 'Alt+2',
      action: () => ctx.navigate('assistant'),
    },
    {
      id: 'nav-activity',
      title: 'Go to Activity Stream',
      description: 'Inspect real-time Hime Core and tool execution audit events',
      category: 'navigation',
      shortcut: 'Alt+3',
      action: () => ctx.navigate('activity'),
    },
    {
      id: 'nav-tasks',
      title: 'Go to Tasks',
      description: 'View and manage system tasks and background jobs',
      category: 'navigation',
      shortcut: 'Alt+4',
      action: () => ctx.navigate('tasks'),
    },
    {
      id: 'nav-notifications',
      title: 'Go to Notifications',
      description: 'Review alerts, permission requests, and audit notices',
      category: 'navigation',
      shortcut: 'Alt+5',
      action: () => ctx.navigate('notifications'),
    },
    {
      id: 'nav-settings',
      title: 'Go to Settings',
      description: 'Configure appearance, Dynamic Island, and shortcuts',
      category: 'navigation',
      shortcut: 'Alt+6',
      action: () => ctx.navigate('settings'),
    },

    // System & Diagnostics Category (Real IPC)
    {
      id: 'sys-ping',
      title: 'Ping Hime Core',
      description: 'Send 12-byte binary framed JSON-RPC heartbeat over Named Pipe',
      category: 'diagnostics',
      shortcut: 'Ctrl+P',
      badge: 'Live IPC',
      action: async () => {
        await ctx.triggerPing();
      },
    },
    {
      id: 'sys-policy-test',
      title: 'Run Policy PEP/PDP Test',
      description: 'Attempt access to protected system path to verify policy boundary',
      category: 'tools',
      badge: 'Live IPC',
      action: async () => {
        await ctx.triggerPolicyTest();
      },
    },
    {
      id: 'sys-toggle-theme',
      title: 'Toggle Theme',
      description: `Switch current theme (currently ${ctx.theme})`,
      category: 'settings',
      shortcut: 'Ctrl+T',
      action: () => ctx.toggleTheme(),
    },
    {
      id: 'sys-window-mode',
      title: 'Toggle Compact HUD / Full Window',
      description: 'Switch between floating Dynamic Island HUD and full desktop window',
      category: 'settings',
      shortcut: 'Ctrl+M',
      badge: 'Window',
      action: async () => {
        await ctx.toggleWindowMode();
      },
    },

    // AI Runtime Category (Phase 08 Active)
    {
      id: 'ai-hermes-chat',
      title: 'Chat with Hermes Agent',
      description: 'Start conversation with Nous Hermes 3 autonomous agent runtime',
      category: 'ai-runtime',
      badge: 'Active AI',
      action: () => {
        ctx.navigate('assistant');
      },
    },
    {
      id: 'ai-models-list',
      title: 'List Available AI Models',
      description: 'Query discovered models from Hime Core and Provider Adapter',
      category: 'ai-runtime',
      badge: 'Active AI',
      action: () => {
        ctx.navigate('overview');
      },
    },
    {
      id: 'ai-gateway-status',
      title: 'AI Runtime & DPAPI Vault Status',
      description: 'Inspect status of Hermes, 9Router, and IB-01/IB-02 boundaries',
      category: 'ai-runtime',
      badge: 'Live IPC',
      action: () => {
        ctx.navigate('overview');
      },
    },
    {
      id: 'ai-agent-task',
      title: 'Launch Autonomous Agent Task',
      description: 'Hermes tool calling and task orchestration under Core PEP policy',
      category: 'ai-runtime',
      badge: 'Active AI',
      action: () => {
        ctx.navigate('tasks');
      },
    },
  ];
};

