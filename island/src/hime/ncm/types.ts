import { EyeExpression } from "../ui/HimeEyes";

/**
 * Priority ladder per 06-NOTCH-EXPERIENCE-AND-PRIORITY.md §3.1.
 * Lower number = higher priority.
 */
export type Priority = 0 | 1 | 2 | 3 | 4 | 5;

export const PriorityLevel = {
	P0_CRITICAL: 0 as Priority, // Approval aksi berisiko, keamanan, enforcement
	P1_TIME_SENSITIVE: 1 as Priority, // Rutinitas mulai, deadline hari-H, timer fokus selesai
	P2_ACTION: 2 as Priority, // Unduhan ganda, konfirmasi simpan, pairing
	P3_PROGRESS: 3 as Priority, // Agent bekerja, transfer file, build
	P4_CONTEXTUAL: 4 as Priority, // Terjemahan, saran, clipboard
	P5_AMBIENT: 5 as Priority // Mata ekspresi, lirik, jam
} as const;

export type PresentationKind = "ambient" | "bubble" | "alert" | "panel-hint" | "card";

export interface PresentationAction {
	id: string;
	label: string; // Hard constraint: <= 10 chars
	action: string;
}

export interface Presentation {
	id: string;
	module: string;
	kind: PresentationKind;
	priority: Priority;
	ttl?: number; // Duration in ms
	interruptibility?: boolean;
	userInitiated?: boolean; // Bumps priority by 1 class (except P0)
	coalesceKey?: string;
	payload?: any;
	actions?: PresentationAction[];
	expression?: EyeExpression;
	createdAt: number;
}

export interface SlotState {
	/** Left slot content type in closed pill */
	leftSlot: "clock" | "eyes" | "status-icon";
	/** Active mascot expression for mini HimeEyes */
	mascotExpression: EyeExpression;
	/** Center slot 1-line info/title (null when closed clock) */
	centerSlot: string | null;
	/** Right slot indicators (task count, progress ring, device dot) */
	rightSlot: { count?: number; progress?: number; icon?: string } | null;
	/** Satellite activity bubbles (strictly max 2 flanking or vertical) */
	bubbles: Presentation[];
	/** Active P0 critical presentation requiring priority display */
	activeAlert: Presentation | null;
	/** Invariant: true when notch rests at closed pill clock */
	fallbackToClock: boolean;
}
