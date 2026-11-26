export type EnforcementTier = "ingatkan" | "tegas" | "paksa";

export type SessionState = "idle" | "ambient" | "prompted" | "active" | "snoozed" | "completed" | "skipped";

export type ReasonPill = "Sibuk" | "Sakit" | "Libur" | "Lainnya";

export interface RoutineSchedule {
	days: number[];
	time: string;
	duration_min: number;
	window_min?: number;
}

export interface FocusRoutine {
	id: string;
	title: string;
	category: string;
	schedule: RoutineSchedule;
	enforcement: EnforcementTier;
	allowed_apps?: string[];
	distraction_rules?: Record<string, any>;
	start_grace_min: number;
	streak?: {
		count: number;
		best: number;
		last_completed_date?: string;
		frozen_on_date?: string;
	};
	active: boolean;
}

export interface FocusSession {
	id: string;
	routine_id: string;
	routine_title: string;
	state: SessionState;
	duration_min: number;
	elapsed_secs: number;
	remaining_secs: number;
	snooze_count: number;
	max_snoozes: number;
	skip_reason?: ReasonPill;
}
