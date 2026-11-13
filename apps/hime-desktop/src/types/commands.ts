export type CommandCategory = 'system' | 'diagnostics' | 'tools' | 'navigation' | 'settings' | 'ai-runtime';

export interface CommandItem {
  id: string;
  title: string;
  description: string;
  category: CommandCategory;
  shortcut?: string;
  badge?: string;
  isPlanned?: boolean;
  action: () => void | Promise<void>;
}

