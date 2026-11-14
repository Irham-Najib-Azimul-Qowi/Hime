export interface KanbanBoard {
  id: string;
  name: string;
  created_at: number;
  updated_at: number;
}

export interface KanbanColumn {
  id: string;
  board_id: string;
  name: string;
  position: number;
  is_default: boolean;
  created_at: number;
}

export interface ProductivityTask {
  id: string;
  board_id: string;
  column_id: string;
  title: string;
  description: string | null;
  priority: 'low' | 'medium' | 'high' | 'urgent';
  due_date: number | null;
  tags: string | null;
  subtasks: string | null;
  created_at: number;
  updated_at: number;
}

export interface ParsedNaturalTask {
  title: string;
  due_date: number | null;
  priority: 'low' | 'medium' | 'high' | 'urgent';
  tags: string[];
}

export interface StudySchedule {
  id: string;
  title: string;
  subject: string;
  start_time: number;
  end_time: number;
  priority: string;
  is_completed: boolean;
  rescheduled_from: string | null;
  created_at: number;
}

export interface AccountabilitySettings {
  id: string;
  mode: 'santai' | 'biasa' | 'strict';
  opt_in_screen_time: boolean;
  checkin_time: string;
  updated_at: number;
}

export interface ScreenTimeSession {
  id: string;
  app_name: string;
  window_title: string;
  duration_seconds: number;
  is_distraction: boolean;
  session_date: string;
  created_at: number;
}

export interface StartupCheckinSummary {
  mode: 'santai' | 'biasa' | 'strict';
  greeting: string;
  urgent_tasks: ProductivityTask[];
  today_schedules: StudySchedule[];
  snooze_available: boolean;
}

