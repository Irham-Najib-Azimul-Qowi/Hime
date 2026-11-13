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

