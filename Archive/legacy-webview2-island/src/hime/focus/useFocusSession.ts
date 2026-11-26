import { useState, useEffect, useCallback, useRef } from "react";
import type { FocusRoutine, SessionState, ReasonPill } from "./types";
import {
	listRoutines,
	startRoutineSession,
	completeRoutineSession,
	skipRoutineSession,
	snoozeRoutineSession,
} from "../api";

const DEFAULT_ROUTINE: FocusRoutine = {
	id: "rtn-default",
	title: "Belajar",
	category: "Kuliah",
	schedule: {
		days: [1, 2, 3, 4, 5],
		time: "19:00",
		duration_min: 30,
	},
	enforcement: "tegas",
	start_grace_min: 10,
	streak: {
		count: 5,
		best: 14,
	},
	active: true,
};

export function useFocusSession() {
	const [routines, setRoutines] = useState<FocusRoutine[]>([DEFAULT_ROUTINE]);
	const [activeRoutine, setActiveRoutine] = useState<FocusRoutine>(DEFAULT_ROUTINE);
	const [sessionState, setSessionState] = useState<SessionState>("idle");
	const [sessionId, setSessionId] = useState<string | null>(null);
	const [elapsedSecs, setElapsedSecs] = useState(0);
	const [durationMin, setDurationMin] = useState(30);
	const [snoozeCount, setSnoozeCount] = useState(0);
	const [isSelectingReason, setIsSelectingReason] = useState<"skip" | "snooze" | null>(null);
	const timerRef = useRef<ReturnType<typeof setInterval> | null>(null);

	// Load routines from Core if available
	useEffect(() => {
		listRoutines(true)
			.then((items) => {
				if (items && items.length > 0) {
					const mapped: FocusRoutine[] = items.map((r) => ({
						id: r.id,
						title: r.title,
						category: r.category,
						schedule: r.schedule,
						enforcement: r.enforcement,
						allowed_apps: r.allowed_apps,
						distraction_rules: r.distraction_rules,
						start_grace_min: r.start_grace_min,
						streak: r.streak,
						active: r.active,
					}));
					setRoutines(mapped);
					setActiveRoutine(mapped[0]);
					setDurationMin(mapped[0].schedule.duration_min);
				}
			})
			.catch(() => {
				// Fallback to default routine
			});
	}, []);

	// Active countdown ticker
	useEffect(() => {
		if (sessionState === "active") {
			timerRef.current = setInterval(() => {
				setElapsedSecs((prev) => {
					const total = durationMin * 60;
					if (prev + 1 >= total) {
						clearInterval(timerRef.current!);
						setSessionState("completed");
						return total;
					}
					return prev + 1;
				});
			}, 1000);
		} else {
			if (timerRef.current) {
				clearInterval(timerRef.current);
				timerRef.current = null;
			}
		}

		return () => {
			if (timerRef.current) {
				clearInterval(timerRef.current);
			}
		};
	}, [sessionState, durationMin]);

	// Hotkey listener: Ctrl+Alt+F to start manual focus session
	useEffect(() => {
		const handleKeyDown = (e: KeyboardEvent) => {
			if (e.ctrlKey && e.altKey && e.key.toLowerCase() === "f") {
				e.preventDefault();
				if (sessionState === "idle" || sessionState === "prompted") {
					startSession();
				}
			}
		};
		window.addEventListener("keydown", handleKeyDown);
		return () => window.removeEventListener("keydown", handleKeyDown);
	}, [sessionState, activeRoutine]);

	const startSession = useCallback(async () => {
		const now = Math.floor(Date.now() / 1000);
		try {
			const res = await startRoutineSession(activeRoutine.id, now);
			if (res?.session?.id) {
				setSessionId(res.session.id);
			}
		} catch {
			setSessionId(`sess-${Date.now()}`);
		}
		setSessionState("active");
		setElapsedSecs(0);
		setIsSelectingReason(null);
	}, [activeRoutine]);

	const completeSession = useCallback(async () => {
		if (sessionId) {
			try {
				await completeRoutineSession(sessionId);
			} catch {
				// Local completion
			}
		}
		setSessionState("completed");
		setIsSelectingReason(null);
	}, [sessionId]);

	const promptSnooze = useCallback(() => {
		if (activeRoutine.enforcement === "tegas" && snoozeCount >= 3) {
			return; // Capped to max 3 snoozes
		}
		setIsSelectingReason("snooze");
	}, [activeRoutine.enforcement, snoozeCount]);

	const confirmSnooze = useCallback(async (reason: ReasonPill) => {
		if (sessionId) {
			try {
				await snoozeRoutineSession(sessionId, 10, reason);
			} catch {
				// Local snooze
			}
		}
		setSnoozeCount((prev) => prev + 1);
		setSessionState("snoozed");
		setIsSelectingReason(null);
	}, [sessionId]);

	const promptSkip = useCallback(() => {
		setIsSelectingReason("skip");
	}, []);

	const confirmSkip = useCallback(async (reason: ReasonPill) => {
		if (sessionId) {
			try {
				await skipRoutineSession(sessionId, reason);
			} catch {
				// Local skip
			}
		}
		setSessionState("skipped");
		setIsSelectingReason(null);
	}, [sessionId]);

	const resetSession = useCallback(() => {
		setSessionState("idle");
		setElapsedSecs(0);
		setIsSelectingReason(null);
	}, []);

	const totalSecs = durationMin * 60;
	const remainingSecs = Math.max(0, totalSecs - elapsedSecs);
	const progressPct = Math.min(100, Math.round((elapsedSecs / (totalSecs || 1)) * 100));

	return {
		routines,
		activeRoutine,
		setActiveRoutine,
		sessionState,
		elapsedSecs,
		remainingSecs,
		durationMin,
		progressPct,
		snoozeCount,
		maxSnoozes: activeRoutine.enforcement === "tegas" ? 3 : 99,
		isSelectingReason,
		startSession,
		completeSession,
		promptSnooze,
		confirmSnooze,
		promptSkip,
		confirmSkip,
		resetSession,
		setIsSelectingReason,
	};
}
