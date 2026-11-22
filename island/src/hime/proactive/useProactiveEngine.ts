import { useState, useEffect, useCallback, useRef } from "react";
import { listen } from "@tauri-apps/api/event";
import {
	ProactiveItem,
	ProactiveSettings,
	DEFAULT_PROACTIVE_SETTINGS,
	AttentionBudget,
	ATTENTION_LIMITS,
	InteractionLevel
} from "./types";
import { validateProactiveItem, CATALOG_TEMPLATES } from "./copyLint";
import { playProactiveChime } from "./chime";
import { Presentation, PriorityLevel } from "../ncm/types";
import { EyeExpression } from "../ui/HimeEyes";

export interface UseProactiveEngineOptions {
	initialSettings?: Partial<ProactiveSettings>;
	onPresentationSubmit?: (p: Presentation) => void;
	onPresentationRetract?: (id: string) => void;
	onExpressionOverride?: (expr: EyeExpression | null) => void;
	isQuietHoursMock?: boolean;
}

export interface HourlyUsage {
	windowStart: number;
	colekCount: number;
	tanyaCount: number;
}

const SETTINGS_STORAGE_KEY = "hime-proactive-settings";

export function loadProactiveSettings(): ProactiveSettings {
	try {
		const raw = localStorage.getItem(SETTINGS_STORAGE_KEY);
		if (raw) {
			const parsed = JSON.parse(raw);
			return { ...DEFAULT_PROACTIVE_SETTINGS, ...parsed };
		}
	} catch {
		// Fallback
	}
	return DEFAULT_PROACTIVE_SETTINGS;
}

export function saveProactiveSettingsToStorage(settings: ProactiveSettings): void {
	try {
		localStorage.setItem(SETTINGS_STORAGE_KEY, JSON.stringify(settings));
	} catch {
		// Ignore
	}
}

/**
 * Checks if the current time falls inside quiet hours (e.g. 23:00 to 06:00).
 */
export function isCurrentlyQuietHours(
	startStr = "23:00",
	endStr = "06:00",
	currentDate: Date = new Date()
): boolean {
	const [startHour, startMin] = startStr.split(":").map(Number);
	const [endHour, endMin] = endStr.split(":").map(Number);

	const curHour = currentDate.getHours();
	const curMin = currentDate.getMinutes();
	const curTotal = curHour * 60 + curMin;

	const startTotal = startHour * 60 + (startMin || 0);
	const endTotal = endHour * 60 + (endMin || 0);

	if (startTotal > endTotal) {
		// Crosses midnight (e.g. 23:00 to 06:00)
		return curTotal >= startTotal || curTotal < endTotal;
	} else {
		return curTotal >= startTotal && curTotal < endTotal;
	}
}

export function useProactiveEngine(options: UseProactiveEngineOptions = {}) {
	const [settings, setSettings] = useState<ProactiveSettings>(() => loadProactiveSettings());
	const [activeItem, setActiveItem] = useState<ProactiveItem | null>(null);
	const [activeCard, setActiveCard] = useState<ProactiveItem | null>(null);
	const [deferredItems, setDeferredItems] = useState<ProactiveItem[]>([]);
	const [pillDot, setPillDot] = useState(false);

	const hourlyUsageRef = useRef<HourlyUsage>({
		windowStart: Date.now(),
		colekCount: 0,
		tanyaCount: 0
	});

	// Update settings
	const updateSettings = useCallback((updates: Partial<ProactiveSettings>) => {
		setSettings((prev) => {
			const next = { ...prev, ...updates };
			saveProactiveSettingsToStorage(next);
			return next;
		});
	}, []);

	// Reset hourly budget if 1 hour has elapsed
	const checkAndResetHourlyWindow = useCallback(() => {
		const now = Date.now();
		if (now - hourlyUsageRef.current.windowStart >= 3600 * 1000) {
			hourlyUsageRef.current = {
				windowStart: now,
				colekCount: 0,
				tanyaCount: 0
			};
		}
	}, []);

	// Handle proactive submission
	const submitProactive = useCallback(
		(item: ProactiveItem): boolean => {
			validateProactiveItem(item);
			checkAndResetHourlyWindow();

			// 1. Check remembered rule: if user already set "Selalu..." for this situation, auto-resolve!
			if (settings.rememberedRules[item.situationId]) {
				const autoAction = settings.rememberedRules[item.situationId];
				console.info(
					`[Proactive] Auto-resolved situation "${item.situationId}" with remembered rule: ${autoAction}`
				);
				return false;
			}

			// 2. Check backoff: after 3 dismissals, apply exponential backoff (drop or defer)
			const dismissals = settings.dismissCounts[item.situationId] || 0;
			if (dismissals >= 3) {
				const multiplier = settings.backoffMultipliers[item.situationId] || 1;
				// Dropped unless level is Penting
				if (item.level !== "penting") {
					console.warn(
						`[Proactive] Situation "${item.situationId}" is in backoff (${dismissals} dismissals, x${multiplier}). Suppressed.`
					);
					return false;
				}
			}

			// 3. Quiet Hours Check (default 23:00 - 06:00)
			const isQuiet =
				options.isQuietHoursMock ??
				(settings.quietHoursEnabled &&
					isCurrentlyQuietHours(settings.quietHoursStart, settings.quietHoursEnd));

			if (isQuiet && item.level !== "penting") {
				// Quiet hours: Only P0 / Penting allowed. Eyes turn ngantuk.
				options.onExpressionOverride?.("ngantuk");
				setDeferredItems((prev) => [...prev, item]);
				return false;
			}

			// 4. Attention Budget Verification
			const limits = ATTENTION_LIMITS[settings.budget];
			if (item.level === "colek") {
				if (hourlyUsageRef.current.colekCount >= limits.colekPerHour) {
					console.warn(
						`[Proactive] Hourly Colek budget exceeded (${limits.colekPerHour}/hr). Deferred.`
					);
					setDeferredItems((prev) => [...prev, item]);
					return false;
				}
				hourlyUsageRef.current.colekCount += 1;
			} else if (item.level === "tanya") {
				if (hourlyUsageRef.current.tanyaCount >= limits.tanyaPerHour) {
					console.warn(
						`[Proactive] Hourly Tanya budget exceeded (${limits.tanyaPerHour}/hr). Deferred.`
					);
					setDeferredItems((prev) => [...prev, item]);
					return false;
				}
				hourlyUsageRef.current.tanyaCount += 1;
			}

			// 5. Interaction Level Dispatch
			if (item.level === "bisik") {
				// Bisik: Eye curiosity / pill dot without loud bubble
				options.onExpressionOverride?.(item.expression || "penasaran");
				setPillDot(true);
				setTimeout(() => {
					setPillDot(false);
					options.onExpressionOverride?.(null);
				}, 6000);
				return true;
			}

			if (item.level === "penting") {
				// Penting: Sound chime + immediate expansion / alert
				playProactiveChime();
				setActiveCard(item);
				setActiveItem(item);

				// Bridge to NCM presentation
				options.onPresentationSubmit?.({
					id: `proactive-${item.id}`,
					module: "proactive",
					kind: "alert",
					priority: PriorityLevel.P0_CRITICAL,
					expression: item.expression,
					createdAt: Date.now(),
					payload: {
						title: item.title,
						description: item.description,
						text: item.bubbleText
					},
					actions: item.actions?.map((a) => ({
						id: a.id,
						label: a.label,
						action: a.action || a.id
					}))
				});
				return true;
			}

			// Colek & Tanya: display bubble
			setActiveItem(item);
			options.onExpressionOverride?.(item.expression);

			// Bridge to NCM presentation
			options.onPresentationSubmit?.({
				id: `proactive-${item.id}`,
				module: "proactive",
				kind: "bubble",
				priority: item.level === "tanya" ? PriorityLevel.P2_ACTION : PriorityLevel.P4_CONTEXTUAL,
				expression: item.expression,
				ttl: item.autoDismissSecs ? item.autoDismissSecs * 1000 : 10000,
				createdAt: Date.now(),
				payload: {
					title: item.title,
					description: item.description,
					text: item.bubbleText
				},
				actions: item.actions?.map((a) => ({
					id: a.id,
					label: a.label,
					action: a.action || a.id
				}))
			});

			return true;
		},
		[settings, checkAndResetHourlyWindow, options]
	);

	// User responds to an action
	const respondProactive = useCallback(
		(itemId: string, actionId: string, remember = false) => {
			const item = activeItem?.id === itemId ? activeItem : activeCard;
			if (!item) return;

			if (remember && item.situationId) {
				updateSettings({
					rememberedRules: {
						...settings.rememberedRules,
						[item.situationId]: actionId
					}
				});
			}

			// Clear presentations and states
			options.onPresentationRetract?.(`proactive-${item.id}`);
			options.onExpressionOverride?.(null);
			setActiveItem(null);
			setActiveCard(null);
		},
		[activeItem, activeCard, settings, updateSettings, options]
	);

	// User dismisses / closes an item
	const dismissProactive = useCallback(
		(itemId?: string) => {
			const target = activeItem || activeCard;
			if (!target) return;

			const situationId = target.situationId;
			if (situationId) {
				const currentCount = (settings.dismissCounts[situationId] || 0) + 1;
				const currentMultiplier = settings.backoffMultipliers[situationId] || 1;
				const newMultiplier = currentCount >= 3 ? currentMultiplier * 2 : currentMultiplier;

				updateSettings({
					dismissCounts: {
						...settings.dismissCounts,
						[situationId]: currentCount
					},
					backoffMultipliers: {
						...settings.backoffMultipliers,
						[situationId]: newMultiplier
					}
				});
			}

			options.onPresentationRetract?.(`proactive-${target.id}`);
			options.onExpressionOverride?.(null);
			setActiveItem(null);
			setActiveCard(null);
		},
		[activeItem, activeCard, settings, updateSettings, options]
	);

	// Reset backoff for a specific situation or all
	const resetBackoff = useCallback(
		(situationId?: string) => {
			if (situationId) {
				const newCounts = { ...settings.dismissCounts };
				delete newCounts[situationId];
				const newMults = { ...settings.backoffMultipliers };
				delete newMults[situationId];
				updateSettings({ dismissCounts: newCounts, backoffMultipliers: newMults });
			} else {
				updateSettings({ dismissCounts: {}, backoffMultipliers: {} });
			}
		},
		[settings, updateSettings]
	);

	// Set Attention Budget: "pendiam" | "seimbang" | "cerewet"
	const setBudget = useCallback(
		(budget: AttentionBudget) => {
			updateSettings({ budget });
		},
		[updateSettings]
	);

	// Listen to Tauri events for proactive triggers
	useEffect(() => {
		let unlistenProactive: (() => void) | undefined;
		let unlistenDuplicate: (() => void) | undefined;
		let unlistenStorage: (() => void) | undefined;

		const setupListeners = async () => {
			try {
				unlistenProactive = await listen<ProactiveItem>("hime-proactive-event", (event) => {
					if (event.payload) submitProactive(event.payload);
				});

				unlistenDuplicate = await listen<any>("hime-download-duplicate", (event) => {
					const tmpl = CATALOG_TEMPLATES.fileDuplicate;
					submitProactive({
						id: `dup-${Date.now()}`,
						situationId: tmpl.situationId,
						level: tmpl.level,
						title: tmpl.title,
						description: tmpl.description,
						bubbleText: tmpl.bubbleText,
						expression: tmpl.expression,
						actions: tmpl.actions,
						hasRememberOption: tmpl.hasRememberOption,
						createdAt: Date.now(),
						payload: event.payload
					});
				});

				unlistenStorage = await listen<any>("hime-storage-low", (event) => {
					const tmpl = CATALOG_TEMPLATES.storageLow;
					submitProactive({
						id: `storage-${Date.now()}`,
						situationId: tmpl.situationId,
						level: tmpl.level,
						title: tmpl.title,
						description: tmpl.description,
						bubbleText: tmpl.bubbleText,
						expression: tmpl.expression,
						actions: tmpl.actions,
						hasRememberOption: tmpl.hasRememberOption,
						createdAt: Date.now(),
						payload: event.payload
					});
				});
			} catch {
				// Headless or non-Tauri environment
			}
		};

		setupListeners();

		return () => {
			if (unlistenProactive) unlistenProactive();
			if (unlistenDuplicate) unlistenDuplicate();
			if (unlistenStorage) unlistenStorage();
		};
	}, [submitProactive]);

	return {
		settings,
		activeItem,
		activeCard,
		deferredItems,
		pillDot,
		submitProactive,
		respondProactive,
		dismissProactive,
		resetBackoff,
		setBudget,
		setActiveCard,
		updateSettings
	};
}
