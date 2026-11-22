import { useState, useEffect, useCallback, useMemo, useRef } from "react";
import { Presentation, SlotState, PriorityLevel } from "./types";
import { ApprovalItem } from "../types";
import { EyeExpression } from "../ui/HimeEyes";
import { arbitrateSlotState } from "./arbiter";

export interface UseNCMOptions {
	isOnline?: boolean;
	approvals?: ApprovalItem[];
	approvalJustResolved?: boolean;
	onApprovalExpired?: (requestId: string) => void;
	freq?: "Mati" | "Jarang" | "Sedang" | "Sering";
	isFocusMode?: boolean;
	isFullscreen?: boolean;
	isTyping?: boolean;
	isDnd?: boolean;
	isBatterySaver?: boolean;
	mode?: "normal" | "dnd" | "focus" | "fullscreen";
	hasStatusIcon?: boolean;
}

const EMPTY_APPROVALS: ApprovalItem[] = [];

/**
 * Structural comparison helper for ApprovalItem to guarantee idempotent synchronization.
 */
function areApprovalsShallowEqual(a: ApprovalItem, b: ApprovalItem): boolean {
	if (a === b) return true;
	if (!a || !b) return false;
	if (
		a.request_id !== b.request_id ||
		a.action_type !== b.action_type ||
		a.description !== b.description ||
		a.expires_at !== b.expires_at ||
		a.created_at !== b.created_at ||
		a.status !== b.status ||
		a.path !== b.path
	) {
		return false;
	}
	if ((a as any).source !== (b as any).source || (a as any).risk_level !== (b as any).risk_level) {
		return false;
	}
	return true;
}

/**
 * Compares two approval arrays by identity, length, IDs, and payload data.
 */
function areApprovalListsEqual(
	prev: ApprovalItem[] | null,
	next: ApprovalItem[]
): boolean {
	if (prev === null) return false;
	if (prev === next) return true;
	if (prev.length !== next.length) return false;
	if (prev.length === 0) return true;

	// In-order fast-path
	let inOrderMatch = true;
	for (let i = 0; i < prev.length; i++) {
		if (!areApprovalsShallowEqual(prev[i], next[i])) {
			inOrderMatch = false;
			break;
		}
	}
	if (inOrderMatch) return true;

	// Unordered comparison
	const prevMap = new Map<string, ApprovalItem>();
	for (const item of prev) {
		prevMap.set(item.request_id, item);
	}
	for (const nextItem of next) {
		const prevItem = prevMap.get(nextItem.request_id);
		if (!prevItem || !areApprovalsShallowEqual(prevItem, nextItem)) {
			return false;
		}
	}
	return true;
}

/**
 * Weighted contextual picker for Idle Director ambient expressions.
 * Respects time of day, activity status, and avoids consecutive duplicates.
 */
export function pickAmbientExpression(
	options: {
		hour?: number;
		isDaytimeIdle?: boolean;
		taskJustFinished?: boolean;
		isBatteryLow?: boolean;
		lastExpression?: EyeExpression;
	} = {}
): EyeExpression {
	const hour = options.hour ?? new Date().getHours();
	const pool: { expr: EyeExpression; weight: number }[] = [];

	if (options.taskJustFinished) {
		pool.push({ expr: "senang", weight: 80 });
		pool.push({ expr: "bangga", weight: 20 });
	} else if (options.isBatteryLow) {
		pool.push({ expr: "khawatir", weight: 100 });
	} else if (hour >= 23 || hour < 5) {
		// Late night: 23:00 - 04:59
		pool.push({ expr: "ngantuk", weight: 50 });
		pool.push({ expr: "menguap", weight: 30 });
		pool.push({ expr: "tidur", weight: 20 });
	} else if (options.isDaytimeIdle) {
		// Daytime idle > 20 min
		pool.push({ expr: "bosan", weight: 60 });
		pool.push({ expr: "netral", weight: 30 });
		pool.push({ expr: "kedip-sebelah", weight: 10 });
	} else {
		// Standard ambient daytime
		pool.push({ expr: "netral", weight: 70 });
		pool.push({ expr: "penasaran", weight: 20 });
		pool.push({ expr: "kedip-sebelah", weight: 10 });
	}

	let filtered = pool;
	if (options.lastExpression && pool.length > 1) {
		const withoutLast = pool.filter((p) => p.expr !== options.lastExpression);
		if (withoutLast.length > 0) filtered = withoutLast;
	}

	const totalWeight = filtered.reduce((sum, item) => sum + item.weight, 0);
	let rand = Math.random() * totalWeight;
	for (const item of filtered) {
		if (rand < item.weight) return item.expr;
		rand -= item.weight;
	}
	return filtered[0]?.expr ?? "netral";
}

export function useNCM(options: UseNCMOptions = {}) {
	const {
		isOnline = true,
		approvals = EMPTY_APPROVALS,
		approvalJustResolved = false,
		onApprovalExpired,
		freq = "Sedang",
		isFocusMode = false,
		isFullscreen = false,
		isTyping = false,
		isDnd = false,
		isBatterySaver = false,
		mode,
		hasStatusIcon = false
	} = options;

	const [presentations, setPresentations] = useState<Presentation[]>([]);
	const presentationsRef = useRef<Presentation[]>([]);
	presentationsRef.current = presentations;

	const [pausedIds, setPausedIds] = useState<Set<string>>(new Set());
	const pausedIdsRef = useRef<Set<string>>(new Set());
	pausedIdsRef.current = pausedIds;

	const lastAmbientExprRef = useRef<EyeExpression>("netral");
	const prevApprovalsRef = useRef<ApprovalItem[] | null>(null);

	// Pause TTL countdown (e.g. mouse hover over contextual bubble)
	const pausePresentation = useCallback((id: string) => {
		setPausedIds((prev) => {
			const next = new Set(prev);
			next.add(id);
			return next;
		});
	}, []);

	// Resume TTL countdown (e.g. mouse pointer leave)
	const resumePresentation = useCallback((id: string) => {
		setPausedIds((prev) => {
			const next = new Set(prev);
			next.delete(id);
			return next;
		});
	}, []);

	// Submit a new presentation or coalesce with matching coalesceKey
	const submitPresentation = useCallback(
		(p: Omit<Presentation, "createdAt"> & { createdAt?: number }) => {
			const item: Presentation = {
				...p,
				createdAt: p.createdAt ?? Date.now()
			};

			setPresentations((prev) => {
				// Coalesce rule: merge item with matching coalesceKey if present
				if (item.coalesceKey) {
					const existingIdx = prev.findIndex((x) => x.coalesceKey === item.coalesceKey);
					if (existingIdx >= 0) {
						const existing = prev[existingIdx];
						const prevCount = existing.payload?.count ?? 1;
						const addedCount = item.payload?.count ?? 1;
						const newCount = prevCount + addedCount;
						const updated = [...prev];
						updated[existingIdx] = {
							...item,
							id: existing.id,
							createdAt: Date.now(),
							payload: {
								...existing.payload,
								...item.payload,
								count: newCount,
								description: item.payload?.description ?? `${newCount} file ganda`
							}
						};
						return updated;
					}
				}

				// Otherwise match by ID or append
				const existingIdIdx = prev.findIndex((x) => x.id === item.id);
				if (existingIdIdx >= 0) {
					const updated = [...prev];
					updated[existingIdIdx] = item;
					return updated;
				}
				return [...prev, item];
			});
		},
		[]
	);

	// Retract a presentation by ID
	const retractPresentation = useCallback((id: string) => {
		setPresentations((prev) => prev.filter((p) => p.id !== id));
	}, []);

	// Clear all presentations
	const clearPresentations = useCallback(() => {
		setPresentations([]);
	}, []);

	// Global event bridge for cross-component NCM presentations
	useEffect(() => {
		if (typeof window === "undefined") return;
		const handleNcmSubmit = (e: Event) => {
			const detail = (e as CustomEvent<Presentation>).detail;
			if (detail) submitPresentation(detail);
		};
		const handleNcmRetract = (e: Event) => {
			const detail = (e as CustomEvent<{ id: string }>).detail;
			if (detail?.id) retractPresentation(detail.id);
		};
		window.addEventListener("hime:ncm-submit", handleNcmSubmit);
		window.addEventListener("hime:ncm-retract", handleNcmRetract);
		return () => {
			window.removeEventListener("hime:ncm-submit", handleNcmSubmit);
			window.removeEventListener("hime:ncm-retract", handleNcmRetract);
		};
	}, [submitPresentation, retractPresentation]);

	// Idle Director: trigger an ambient peek with soft spring duration (4-7s)
	const triggerAmbientPeek = useCallback(
		(overrideExpr?: EyeExpression, durationMs = 5000) => {
			if (
				isFullscreen ||
				isFocusMode ||
				isTyping ||
				isDnd ||
				isBatterySaver ||
				freq === "Mati"
			) {
				return;
			}
			const chosen =
				overrideExpr ??
				pickAmbientExpression({
					lastExpression: lastAmbientExprRef.current
				});
			lastAmbientExprRef.current = chosen;

			submitPresentation({
				id: `ambient-peek-${Date.now()}`,
				module: "idle-director",
				kind: "ambient",
				priority: PriorityLevel.P5_AMBIENT,
				expression: chosen,
				ttl: durationMs
			});
		},
		[
			isFullscreen,
			isFocusMode,
			isTyping,
			isDnd,
			isBatterySaver,
			freq,
			submitPresentation
		]
	);

	// Auto sync incoming approvals from Hime Core into P0 presentations (fully idempotent)
	useEffect(() => {
		// Compare incoming approval IDs/data before scheduling or updating presentations state
		if (areApprovalListsEqual(prevApprovalsRef.current, approvals)) {
			return;
		}
		prevApprovalsRef.current = approvals.map((app) => ({ ...app }));

		const activeIds = new Set(approvals.map((app) => `approval-${app.request_id}`));

		if (approvals.length > 0) {
			setPresentations((prev) => {
				let hasChanges = false;

				// 1. Prune resolved or retracted approval presentations
				const pruned = prev.filter((p) => {
					if (p.module === "approvals" && !activeIds.has(p.id)) {
						hasChanges = true;
						return false;
					}
					return true;
				});

				// 2. Add or update active approvals
				for (const app of approvals) {
					const presentationId = `approval-${app.request_id}`;
					const existingIdx = pruned.findIndex((p) => p.id === presentationId);

					if (existingIdx >= 0) {
						const existing = pruned[existingIdx];
						// Compare incoming approval data with existing presentation payload
						if (existing.payload && areApprovalsShallowEqual(existing.payload, app)) {
							// Unchanged: preserve existing presentation (keeps createdAt / timer)
							continue;
						}
						// Approval data changed: update in place
						hasChanges = true;
						pruned[existingIdx] = {
							...existing,
							payload: app,
							ttl: Math.max(0, app.expires_at - Date.now())
						};
					} else {
						// New approval presentation: append
						hasChanges = true;
						pruned.push({
							id: presentationId,
							module: "approvals",
							kind: "alert",
							priority: PriorityLevel.P0_CRITICAL,
							ttl: Math.max(0, app.expires_at - Date.now()),
							createdAt: Date.now(),
							expression: "berpikir",
							payload: app,
							actions: [
								{ id: "deny", label: "Tolak", action: "deny" },
								{ id: "approve", label: "Setujui", action: "approve" }
							]
						});
					}
				}

				if (!hasChanges) return prev;
				return pruned;
			});
		} else {
			// Prune any remaining approval presentations if approvals is empty
			setPresentations((prev) => {
				const hasStale = prev.some((p) => p.module === "approvals");
				if (!hasStale) return prev;
				return prev.filter((p) => p.module !== "approvals");
			});
		}
	}, [approvals]);

	// TTL countdown & Preemption Resumption timer (runs every 1s)
	// When a presentation is preempted by a higher-priority alert or is paused by hover,
	// its elapsed time is frozen by shifting createdAt forward so its TTL resumes upon release.
	useEffect(() => {
		const interval = setInterval(() => {
			const now = Date.now();
			let changed = false;

			// Determine if a P0 critical alert is currently preempting lower tiers
			const hasActiveP0 = presentationsRef.current.some(
				(p) => p.priority === PriorityLevel.P0_CRITICAL
			);

			const valid = presentationsRef.current.filter((p) => {
				// If paused by hover or preempted by higher-tier P0, freeze countdown
				const isPreempted = hasActiveP0 && p.priority > PriorityLevel.P0_CRITICAL;
				const isPaused = pausedIdsRef.current.has(p.id) || isPreempted;

				if (isPaused) {
					// Shift createdAt by 1000ms to pause TTL countdown
					p.createdAt += 1000;
					return true;
				}

				if (p.ttl && now - p.createdAt >= p.ttl) {
					changed = true;
					if (p.module === "approvals" && p.payload?.request_id && onApprovalExpired) {
						onApprovalExpired(p.payload.request_id);
					}
					return false;
				}
				return true;
			});

			if (changed) {
				setPresentations(valid);
			}
		}, 1000);

		return () => clearInterval(interval);
	}, [onApprovalExpired]);

	// Idle Director periodic ambient peek scheduler
	useEffect(() => {
		if (freq === "Mati") return;

		// Average intervals: Sering (5m), Sedang (14m), Jarang (30m)
		const intervalMs =
			freq === "Sering"
				? 5 * 60 * 1000
				: freq === "Jarang"
				? 30 * 60 * 1000
				: 14 * 60 * 1000;

		const timer = setInterval(() => {
			triggerAmbientPeek();
		}, intervalMs);

		return () => clearInterval(timer);
	}, [freq, triggerAmbientPeek]);

	// Effective mode computation
	const effectiveMode =
		mode ??
		(isFullscreen
			? "fullscreen"
			: isFocusMode
			? "focus"
			: isDnd
			? "dnd"
			: "normal");

	// Arbiter: Sort and schedule presentations based on effective priority
	const slotState: SlotState = useMemo(() => {
		return arbitrateSlotState(presentations, {
			isOnline,
			approvals,
			approvalJustResolved,
			mode: effectiveMode,
			hasStatusIcon
		});
	}, [
		presentations,
		isOnline,
		approvals,
		approvalJustResolved,
		effectiveMode,
		hasStatusIcon
	]);

	return {
		presentations,
		slotState,
		submitPresentation,
		retractPresentation,
		clearPresentations,
		pausePresentation,
		resumePresentation,
		triggerAmbientPeek
	};
}
