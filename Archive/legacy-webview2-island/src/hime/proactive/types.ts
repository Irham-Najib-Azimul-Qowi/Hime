import { EyeExpression } from "../ui/HimeEyes";

/**
 * 4 Interaction Levels per 11-PROACTIVE-INTERACTION.md §2:
 * - Bisik: Perubahan halus (ekspresi mata, titik di pill)
 * - Colek: Bubble ringkas tanpa aksi wajib, auto-hilang
 * - Tanya: Bubble dengan <= 3 aksi, default first, remember rule option
 * - Penting: Ekspansi + suara/getar pendek
 */
export type InteractionLevel = "bisik" | "colek" | "tanya" | "penting";

/**
 * Attention Budget modes:
 * - Pendiam: 1 Colek/hr, 1 Tanya/hr
 * - Seimbang (default): 3 Colek/hr, 3 Tanya/hr
 * - Cerewet: 6 Colek/hr, 5 Tanya/hr
 */
export type AttentionBudget = "pendiam" | "seimbang" | "cerewet";

export interface AttentionLimits {
	colekPerHour: number;
	tanyaPerHour: number;
}

export const ATTENTION_LIMITS: Record<AttentionBudget, AttentionLimits> = {
	pendiam: { colekPerHour: 1, tanyaPerHour: 1 },
	seimbang: { colekPerHour: 3, tanyaPerHour: 3 },
	cerewet: { colekPerHour: 6, tanyaPerHour: 5 }
};

export interface ProactiveAction {
	id: string;
	label: string; // Hard constraint: <= 10 chars
	action?: string;
	isPrimary?: boolean;
	isDestructive?: boolean;
}

export interface ProactiveItem {
	id: string;
	situationId: string;
	level: InteractionLevel;
	title: string; // Hard constraint: <= 16 chars
	description: string; // Hard constraint: <= 36 chars
	bubbleText: string; // Hard constraint: <= 40 chars
	expression: EyeExpression;
	actions?: ProactiveAction[]; // <= 3 actions, first is default
	hasRememberOption?: boolean; // "Selalu..." option
	autoDismissSecs?: number; // for Colek, e.g. 6s
	createdAt: number;
	payload?: any;
}

export interface ProactiveSettings {
	budget: AttentionBudget;
	quietHoursEnabled: boolean;
	quietHoursStart: string; // e.g. "23:00"
	quietHoursEnd: string; // e.g. "06:00"
	rememberedRules: Record<string, string>; // situationId -> chosen actionId
	dismissCounts: Record<string, number>; // situationId -> number of dismissals for backoff
	backoffMultipliers: Record<string, number>; // situationId -> exponential backoff factor
}

export const DEFAULT_PROACTIVE_SETTINGS: ProactiveSettings = {
	budget: "seimbang",
	quietHoursEnabled: true,
	quietHoursStart: "23:00",
	quietHoursEnd: "06:00",
	rememberedRules: {},
	dismissCounts: {},
	backoffMultipliers: {}
};
