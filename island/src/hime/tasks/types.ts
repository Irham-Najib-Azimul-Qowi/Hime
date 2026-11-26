/** Priority levels matching Core's ProductivityTaskRecord. */
export type TaskPriority = "low" | "medium" | "high" | "critical";

/** Status values matching Core columns. */
export type TaskStatus = "todo" | "in_progress" | "review" | "done";

/** Task record from Core's productivity_tasks table. */
export interface TaskItem {
	id: string;
	board_id: string;
	column_id: string;
	title: string;
	description: string | null;
	priority: TaskPriority;
	status: TaskStatus;
	due_date: number | null;
	estimated_minutes: number | null;
	actual_minutes: number | null;
	labels: string | null;
	parent_task_id: string | null;
	created_at: number;
	updated_at: number;
	completed_at: number | null;
}

/** Urgency chip color derived from due_date vs now. */
export type UrgencyLevel = "overdue" | "today" | "soon" | "normal" | "none";

/** Result from parse_task_natural. */
export interface ParsedTaskInput {
	title: string;
	priority: TaskPriority;
	due_date: number | null;
}

/** Default category list with vibrant, luminous accent colors. */
export const CATEGORIES = [
	{ id: "kuliah", name: "Kuliah", color: "#8B5CF6" },
	{ id: "proyek", name: "Proyek", color: "#06B6D4" },
	{ id: "organisasi", name: "Organisasi", color: "#10B981" },
	{ id: "pribadi", name: "Pribadi", color: "#F43F5E" },
	{ id: "lainnya", name: "Lainnya", color: "#F59E0B" },
] as const;

/** Tab filter for the segmented control. */
export type TaskTab = "today" | "upcoming" | "done";
