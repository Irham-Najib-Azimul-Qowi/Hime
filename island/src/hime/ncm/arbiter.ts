import { Presentation, Priority, SlotState, PriorityLevel } from "./types";
import { EyeExpression } from "../ui/HimeEyes";
import { ApprovalItem } from "../types";

export interface ArbitrateOptions {
	isOnline?: boolean;
	approvals?: ApprovalItem[];
	approvalJustResolved?: boolean;
	mode?: "normal" | "dnd" | "focus" | "fullscreen";
	hasStatusIcon?: boolean;
}

/**
 * Calculates effective priority: userInitiated bumps by 1 class (except P0).
 */
export function calculateEffectivePriority(p: Presentation): Priority {
	return p.userInitiated && p.priority > 0 ? ((p.priority - 1) as Priority) : p.priority;
}

/**
 * Pure arbitration function: sorts presentations by priority and FIFO createdAt,
 * extracts active alert (P0), satellite bubbles (max 2), mascot expression,
 * slot texts/indicators, and fallback-to-clock status.
 */
export function arbitrateSlotState(
	presentations: Presentation[],
	options: ArbitrateOptions = {}
): SlotState {
	const {
		isOnline = true,
		approvals = [],
		approvalJustResolved = false,
		mode = "normal",
		hasStatusIcon = false
	} = options;

	// Mode suppression: filter presentations based on environment
	const filtered = presentations.filter((p) => {
		if (mode === "fullscreen") {
			return p.priority === PriorityLevel.P0_CRITICAL;
		}
		if (mode === "dnd" || mode === "focus") {
			return (
				p.priority === PriorityLevel.P0_CRITICAL ||
				p.priority === PriorityLevel.P1_TIME_SENSITIVE
			);
		}
		return true;
	});

	// Calculate effective priority: userInitiated bumps by 1 class (except P0)
	const sorted = [...filtered].sort((a, b) => {
		const effA = calculateEffectivePriority(a);
		const effB = calculateEffectivePriority(b);
		if (effA !== effB) return effA - effB;
		return a.createdAt - b.createdAt; // FIFO for same priority
	});

	// 1. Critical P0 Alert / Preemption
	const activeAlert = sorted.find((p) => p.priority === PriorityLevel.P0_CRITICAL) ?? null;

	// 2. Satellite Bubbles: filter presentations intended for bubble, max 2
	const bubbles = sorted
		.filter((p) => p.kind === "bubble" && p !== activeAlert)
		.slice(0, 2);

	// 3. Determine Mascot Expression for Closed Notch Pill
	let mascotExpression: EyeExpression = "netral";
	if (!isOnline) {
		mascotExpression = "offline";
	} else if (activeAlert || approvals.length > 0) {
		mascotExpression = "berpikir";
	} else if (approvalJustResolved) {
		mascotExpression = "senang";
	} else if (sorted.length > 0 && sorted[0].expression) {
		mascotExpression = sorted[0].expression;
	}

	// 4. Left slot arbitration: eyes (meaningful) > status-icon > clock
	// Meaningful expressions: offline, thinking (approvals/alert), senang (just resolved),
	// or any non-netral expression / ambient peek in top presentation
	const isMeaningfulExpression =
		!isOnline ||
		activeAlert !== null ||
		approvals.length > 0 ||
		approvalJustResolved ||
		(sorted.length > 0 && sorted[0].expression && sorted[0].expression !== "netral") ||
		(sorted.length > 0 && sorted[0].kind === "ambient");

	let leftSlot: "clock" | "eyes" | "status-icon" = "clock";
	if (isMeaningfulExpression) {
		leftSlot = "eyes";
	} else if (hasStatusIcon) {
		leftSlot = "status-icon";
	} else {
		leftSlot = "clock";
	}

	// 5. Center slot text: dynamic 1-line text (<= 36 characters)
	let centerSlot: string | null = null;
	if (activeAlert?.payload?.description) {
		centerSlot = String(activeAlert.payload.description).slice(0, 36);
	} else if (sorted.length > 0) {
		const top = sorted[0];
		if (top.payload?.description) {
			centerSlot = String(top.payload.description).slice(0, 36);
		} else if (top.payload?.title) {
			centerSlot = String(top.payload.title).slice(0, 36);
		} else if (top.payload?.text) {
			centerSlot = String(top.payload.text).slice(0, 36);
		}
	}

	// 6. Right slot indicator: primary indicator + optional dot
	let rightSlot: SlotState["rightSlot"] = null;
	if (approvals.length > 0) {
		rightSlot = { count: approvals.length };
	} else {
		const withIndicator = sorted.find(
			(p) =>
				p.payload?.progress !== undefined ||
				p.payload?.count !== undefined ||
				p.payload?.icon !== undefined
		);
		if (withIndicator) {
			rightSlot = {
				count: withIndicator.payload.count,
				progress: withIndicator.payload.progress,
				icon: withIndicator.payload.icon
			};
		}
	}

	// 7. Fallback invariant: true when resting at closed pill clock (no active P0 alert)
	const fallbackToClock = activeAlert === null;

	return {
		leftSlot,
		mascotExpression,
		centerSlot,
		rightSlot,
		bubbles,
		activeAlert,
		fallbackToClock
	};
}
