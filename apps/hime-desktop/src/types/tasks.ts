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

