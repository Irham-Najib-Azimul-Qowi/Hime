export interface AuditEvent {
  id: number;
  timestamp: number;
  event_type: string;
  source: string;
  payload: string;
}

export type EventCategory = 'all' | 'core' | 'tool' | 'task' | 'policy';

