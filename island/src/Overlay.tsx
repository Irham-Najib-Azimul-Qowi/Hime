import { StrictMode, useState, useEffect, useRef, useCallback, useLayoutEffect, useMemo } from "react";
import { createRoot } from "react-dom/client";
import { motion, AnimatePresence } from "framer-motion";
import { listen, emit } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { useSettingsSync } from "./hooks/useSettingsSync";
import { useTrailingThrottle } from "./hooks/useTrailingThrottle";
import { getVersion } from "@tauri-apps/api/app";
import { reloadIfMirrorWasStale } from "./hooks/settingsMirror";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { MixerIcon, SpeakerIcon } from "./icons";
import "./Overlay.css";
import { EdgeSurface } from "./components/EdgeSurface";
import { initTheme } from "./theme";
import { createRegionSender, measureElements, type RegionRect } from "./windowRegion";

// ─── App Volume Mixer ───────────────────────────────────────────────────────

interface AudioSession {
	pid: number;
	name: string;
	process_path: string | null;
	volume: number;
	is_muted: boolean;
}

// Sliders can unmount or lose focus while a drag is in progress. Always drop
// their document listeners so an old slider cannot keep sending changes.
function useMouseSliderDrag(onMove: (clientX: number, clientY: number) => void, onEnd?: () => void) {
	const moveRef = useRef(onMove);
	const endRef = useRef(onEnd);
	const cleanupRef = useRef<(() => void) | null>(null);
	moveRef.current = onMove;
	endRef.current = onEnd;

	useEffect(() => () => cleanupRef.current?.(), []);

	return (clientX: number, clientY: number) => {
		cleanupRef.current?.();
		moveRef.current(clientX, clientY);
		const handleMove = (event: MouseEvent) => moveRef.current(event.clientX, event.clientY);
		const cleanup = () => {
			document.removeEventListener("mousemove", handleMove);
			document.removeEventListener("mouseup", cleanup);
			window.removeEventListener("blur", cleanup);
			cleanupRef.current = null;
			endRef.current?.();
		};
		cleanupRef.current = cleanup;
		document.addEventListener("mousemove", handleMove);
		document.addEventListener("mouseup", cleanup);
		window.addEventListener("blur", cleanup);
	};
}

function MixerTile({
	session,
	onVolumeChange,
	onMuteToggle,
	draggingRef
}: {
	session: AudioSession;
	onVolumeChange: (pid: number, volume: number) => void;
	onMuteToggle: (pid: number) => void;
	draggingRef: { current: number | null };
}) {
	const [icon, setIcon] = useState<string | null>(null);
	const [showPercentage, setShowPercentage] = useState(false);
	const barRef = useRef<HTMLDivElement>(null);
	const percentageTimeoutRef = useRef<any>(null);
	const lastPercentageRef = useRef<number | null>(null);

	useEffect(() => {
		if (!session.process_path) return;
		let alive = true;
		invoke<string | null>("get_app_icon", {
			path: session.process_path,
			name: session.name,
			hwnd: null
		})
			.then((data) => {
				if (alive) setIcon(data);
			})
			.catch(() => {});
		return () => {
			alive = false;
		};
	}, [session.process_path, session.name]);

	const percentage = Math.round((session.is_muted ? 0 : session.volume) * 100);

	// Flash the percentage in place of the app logo while its volume changes,
	// matching the main notch's action button.
	useEffect(() => {
		const previous = lastPercentageRef.current;
		lastPercentageRef.current = percentage;
		if (previous === null || previous === percentage) return;
		setShowPercentage(true);
		if (percentageTimeoutRef.current) clearTimeout(percentageTimeoutRef.current);
		percentageTimeoutRef.current = setTimeout(() => setShowPercentage(false), 1200);
		return () => {
			if (percentageTimeoutRef.current) clearTimeout(percentageTimeoutRef.current);
		};
	}, [percentage]);

	const volumeFromY = (_clientX: number, clientY: number) => {
		if (!barRef.current) return;
		const rect = barRef.current.getBoundingClientRect();
		const relativeY = rect.bottom - clientY;
		onVolumeChange(session.pid, Math.max(0, Math.min(1, relativeY / rect.height)));
	};
	const startDrag = useMouseSliderDrag(volumeFromY, () => {
		draggingRef.current = null;
	});

	const handleMouseDown = (e: React.MouseEvent) => {
		startDrag(e.clientX, e.clientY);
		draggingRef.current = session.pid;
	};

	const handleWheel = (e: React.WheelEvent) => {
		e.stopPropagation();
		const delta = e.deltaY < 0 ? 0.04 : -0.04;
		const newVol = Math.max(0, Math.min(1, Math.round((session.volume + delta) * 100) / 100));
		onVolumeChange(session.pid, newVol);
	};

	return (
		<div className="volume-mixer-tile" onWheel={handleWheel}>
			<div
				ref={barRef}
				className="volume-mixer-bar"
				onMouseDown={handleMouseDown}
				onTouchStart={(e) => volumeFromY(e.touches[0].clientX, e.touches[0].clientY)}
				onWheel={handleWheel}
				title={`${session.name} — ${percentage}%`}
				style={{ cursor: "pointer" }}
			>
				<motion.div
					className="volume-mixer-fill"
					initial={false}
					animate={{ height: `${percentage}%` }}
					transition={{ type: "spring", stiffness: 300, damping: 35 }}
				/>
			</div>
			<button
				className={`volume-mixer-action ${session.is_muted ? "muted" : ""} ${
					showPercentage ? "showing-percent" : ""
				}`}
				onClick={() => onMuteToggle(session.pid)}
				title={session.is_muted ? "Unmute" : "Mute"}
			>
				{icon ? (
					<img src={icon} alt="" draggable={false} />
				) : (
					<span className="volume-mixer-action-fallback">
						{session.name.charAt(0).toUpperCase()}
					</span>
				)}
				<span className="volume-mixer-action-pct">{percentage}%</span>
			</button>
		</div>
	);
}

// ─── Volume Notch ───────────────────────────────────────────────────────────

// Mixer panel geometry. The expanded card grows with the number of apps (one
// narrow column each) up to a max, so a single app doesn't open a wide card.
// `MIXER_PANEL_PADDING` must match the horizontal padding on .volume-mixer-list.
const MIXER_COLUMN_WIDTH = 44;
const MIXER_COLUMN_GAP = 4;
const MIXER_PANEL_PADDING = 8;
const MIXER_MAX_COLUMNS = 4;
const MIXER_EMPTY_COLUMNS = 2;
// The widest the mixer panel gets, for the overlay's window region.
const MIXER_MAX_PANEL_WIDTH =
	MIXER_PANEL_PADDING * 2 +
	MIXER_MAX_COLUMNS * MIXER_COLUMN_WIDTH +
	(MIXER_MAX_COLUMNS - 1) * MIXER_COLUMN_GAP;
// A HUD card's size in CSS px before `bloom-scale`; the backend's edge
// hit-testing uses the same 42x196.
const HUD_CARD_WIDTH = 42;
const HUD_CARD_HEIGHT = 196;
const HUD_HORIZONTAL_WIDTH = 196;
const HUD_HORIZONTAL_HEIGHT = 42;
const MIXER_TOP_PANEL_HEIGHT = 160;
const EDGE_CLAMP_MARGIN = 12;

type OverlayEdge = "top" | "left" | "right";
interface OverlayPosition {
	x: number;
	y: number;
	edge?: OverlayEdge;
}

function clamp(value: number, min: number, max: number): number {
	return Math.max(min, Math.min(max, value));
}

function resolveOverlayEdge(
	position: OverlayPosition | null | undefined,
	defaultEdge: OverlayEdge,
	screenW: number
): OverlayEdge {
	if (!position) return defaultEdge;
	if (position.edge === "top" || position.edge === "left" || position.edge === "right") {
		return position.edge;
	}
	if (position.y <= 40) return "top";
	return position.x > screenW / 2 ? "right" : "left";
}

function computeDockedOverlayGeometry(
	edge: OverlayEdge,
	position: OverlayPosition | null | undefined,
	shellW: number,
	shellH: number,
	screenW: number,
	screenH: number
): { x: number; y: number } {
	if (edge === "top") {
		const maxX = Math.max(EDGE_CLAMP_MARGIN, screenW - shellW - EDGE_CLAMP_MARGIN);
		const rawX = position ? position.x : (screenW - shellW) / 2;
		return { x: Math.round(clamp(rawX, EDGE_CLAMP_MARGIN, maxX)), y: 0 };
	}
	const maxY = Math.max(EDGE_CLAMP_MARGIN, screenH - shellH - EDGE_CLAMP_MARGIN);
	const rawY = position ? position.y : (screenH - shellH) / 2;
	const y = Math.round(clamp(rawY, EDGE_CLAMP_MARGIN, maxY));
	const x = edge === "right" ? Math.round(Math.max(0, screenW - shellW)) : 0;
	return { x, y };
}

function snapReleaseToEdge(
	releaseX: number,
	releaseY: number,
	dragW: number,
	screenW: number,
	screenH: number
): OverlayPosition {
	const distTop = Math.max(0, releaseY);
	const distLeft = Math.max(0, releaseX);
	const distRight = Math.max(0, screenW - (releaseX + dragW));

	// Never snap to bottom edge; pick closest among top, left, right.
	if (distTop <= distLeft && distTop <= distRight) {
		const maxX = Math.max(EDGE_CLAMP_MARGIN, screenW - HUD_HORIZONTAL_WIDTH - EDGE_CLAMP_MARGIN);
		return {
			x: Math.round(clamp(releaseX, EDGE_CLAMP_MARGIN, maxX)),
			y: 0,
			edge: "top"
		};
	}
	const maxY = Math.max(EDGE_CLAMP_MARGIN, screenH - HUD_CARD_HEIGHT - EDGE_CLAMP_MARGIN);
	const snappedY = Math.round(clamp(releaseY, EDGE_CLAMP_MARGIN, maxY));
	if (distLeft <= distRight) {
		return { x: 0, y: snappedY, edge: "left" };
	}
	return {
		x: Math.round(Math.max(0, screenW - HUD_CARD_WIDTH)),
		y: snappedY,
		edge: "right"
	};
}

function useViewportSize(): { width: number; height: number } {
	const [size, setSize] = useState(() => ({
		width: typeof window !== "undefined" ? window.innerWidth : 1920,
		height: typeof window !== "undefined" ? window.innerHeight : 1080
	}));
	useEffect(() => {
		const onResize = () => setSize({ width: window.innerWidth, height: window.innerHeight });
		window.addEventListener("resize", onResize);
		return () => window.removeEventListener("resize", onResize);
	}, []);
	return size;
}

function useOverlayCardDrag({
	enabled,
	scale,
	screenW,
	screenH,
	dockedGeometry,
	dragW,
	dragH,
	ignoreSelector,
	onDragStart,
	onPositionChange,
	onDragEnd
}: {
	enabled?: boolean;
	scale: number;
	screenW: number;
	screenH: number;
	dockedGeometry: { x: number; y: number };
	dragW: number;
	dragH: number;
	ignoreSelector: string;
	onDragStart?: () => void;
	onPositionChange?: (pos: OverlayPosition) => void;
	onDragEnd?: () => void;
}) {
	const [isDragging, setIsDragging] = useState(false);
	const [dragPos, setDragPos] = useState<{ x: number; y: number } | null>(null);
	const dragCleanupRef = useRef<(() => void) | null>(null);

	useEffect(() => () => dragCleanupRef.current?.(), []);

	const handleCardPointerDown = (e: React.PointerEvent) => {
		if (!enabled || e.button !== 0) return;
		const target = e.target as HTMLElement;
		if (target.closest(ignoreSelector)) return;

		dragCleanupRef.current?.();
		const safeScale = scale > 0 ? scale : 1;
		const startX = e.clientX;
		const startY = e.clientY;
		const initialPos = { ...dockedGeometry };
		let latestPos = { ...dockedGeometry };
		let hasMoved = false;

		const onPointerMove = (moveEv: PointerEvent) => {
			const rawDx = moveEv.clientX - startX;
			const rawDy = moveEv.clientY - startY;
			if (!hasMoved && (Math.abs(rawDx) > 3 || Math.abs(rawDy) > 3)) {
				hasMoved = true;
				setIsDragging(true);
				onDragStart?.();
			}
			if (hasMoved) {
				const dx = rawDx / safeScale;
				const dy = rawDy / safeScale;
				latestPos = {
					x: Math.round(clamp(initialPos.x + dx, 0, Math.max(0, screenW - dragW))),
					y: Math.round(clamp(initialPos.y + dy, 0, Math.max(0, screenH - dragH)))
				};
				setDragPos(latestPos);
			}
		};

		const finishDrag = (endEv?: PointerEvent) => {
			window.removeEventListener("pointermove", onPointerMove);
			window.removeEventListener("pointerup", finishDrag);
			window.removeEventListener("pointercancel", finishDrag);
			window.removeEventListener("blur", onBlur);
			dragCleanupRef.current = null;
			if (hasMoved) {
				const releaseX = endEv
					? clamp(initialPos.x + (endEv.clientX - startX) / safeScale, 0, Math.max(0, screenW - dragW))
					: latestPos.x;
				const releaseY = endEv
					? clamp(initialPos.y + (endEv.clientY - startY) / safeScale, 0, Math.max(0, screenH - dragH))
					: latestPos.y;
				const snapped = snapReleaseToEdge(releaseX, releaseY, dragW, screenW, screenH);
				setIsDragging(false);
				setDragPos(null);
				onPositionChange?.(snapped);
				onDragEnd?.();
			}
		};

		const onBlur = () => finishDrag();

		dragCleanupRef.current = () => {
			window.removeEventListener("pointermove", onPointerMove);
			window.removeEventListener("pointerup", finishDrag);
			window.removeEventListener("pointercancel", finishDrag);
			window.removeEventListener("blur", onBlur);
		};

		window.addEventListener("pointermove", onPointerMove);
		window.addEventListener("pointerup", finishDrag);
		window.addEventListener("pointercancel", finishDrag);
		window.addEventListener("blur", onBlur);
	};

	return { isDragging, dragPos, handleCardPointerDown };
}

function buildWrapperPosStyle(
	activeEdge: OverlayEdge,
	isDragging: boolean,
	dragPos: { x: number; y: number } | null,
	docked: { x: number; y: number },
	hasPosition: boolean
): React.CSSProperties {
	if (isDragging && dragPos) {
		return {
			transformOrigin: "center center",
			position: "absolute",
			left: dragPos.x,
			right: "auto",
			top: dragPos.y
		};
	}
	if (activeEdge === "top") {
		return {
			transformOrigin: "top center",
			position: "absolute",
			left: docked.x,
			right: "auto",
			top: 0
		};
	}
	if (activeEdge === "right") {
		return {
			transformOrigin: "right center",
			position: hasPosition ? "absolute" : undefined,
			left: "auto",
			right: 0,
			top: hasPosition ? docked.y : undefined
		};
	}
	return {
		transformOrigin: "left center",
		position: hasPosition ? "absolute" : undefined,
		left: 0,
		right: "auto",
		top: hasPosition ? docked.y : undefined
	};
}

function getContentSlideOffset(activeEdge: OverlayEdge): { x: number; y: number } {
	if (activeEdge === "top") return { x: 0, y: -10 };
	if (activeEdge === "right") return { x: 10, y: 0 };
	return { x: -10, y: 0 };
}

function useOverlayRectSync(
	command: "update_volume_overlay_rect" | "update_brightness_overlay_rect",
	x: number,
	y: number,
	width: number,
	height: number,
	edge: OverlayEdge
) {
	useLayoutEffect(() => {
		invoke(command, {
			rect: { x, y, width, height, edge }
		}).catch(() => {});
	}, [command, x, y, width, height, edge]);

	useEffect(() => {
		return () => {
			invoke(command, { rect: null }).catch(() => {});
		};
	}, [command]);
}

// Last fetched session list, so reopening the mixer renders at the right width
// immediately instead of growing out of the empty-state width.
let lastKnownSessions: AudioSession[] = [];

function VolumeNotch({
	volume,
	isMuted,
	onVolumeChange,
	mixerExpanded,
	onMixerExpandedChange,
	freeDragEnabled,
	scale,
	position,
	onPositionChange,
	onDragStart,
	onDragEnd
}: {
	volume: number;
	isMuted: boolean;
	onVolumeChange: (vol: number) => void;
	mixerExpanded: boolean;
	onMixerExpandedChange: (expanded: boolean) => void;
	freeDragEnabled?: boolean;
	scale: number;
	position?: OverlayPosition | null;
	onPositionChange?: (pos: OverlayPosition) => void;
	onDragStart?: () => void;
	onDragEnd?: () => void;
}) {
	const percentage = Math.round(volume * 100);
	const barRef = useRef<HTMLDivElement>(null);
	const shellRef = useRef<HTMLDivElement>(null);
	const draggingRef = useRef<number | null>(null);

	const viewport = useViewportSize();
	const safeScale = scale > 0 ? scale : 1;
	const screenW = viewport.width / safeScale;
	const screenH = viewport.height / safeScale;
	const activeEdge = resolveOverlayEdge(position, "left", screenW);
	const isHorizontal = activeEdge === "top";

	const [sessions, setSessions] = useState<AudioSession[]>(lastKnownSessions);
	const [sessionsLoaded, setSessionsLoaded] = useState(false);
	const [sessionsError, setSessionsError] = useState(false);
	const [showPercentage, setShowPercentage] = useState(false);
	const percentageTimeoutRef = useRef<any>(null);

	// Panel width follows the app count (empty state keeps room for the message).
	const mixerColumns =
		sessions.length === 0 ? MIXER_EMPTY_COLUMNS : Math.min(sessions.length, MIXER_MAX_COLUMNS);
	const mixerPanelWidth =
		MIXER_PANEL_PADDING * 2 +
		mixerColumns * MIXER_COLUMN_WIDTH +
		(mixerColumns - 1) * MIXER_COLUMN_GAP;
	const shellW = isHorizontal
		? mixerExpanded
			? Math.max(HUD_HORIZONTAL_WIDTH, mixerPanelWidth)
			: HUD_HORIZONTAL_WIDTH
		: mixerExpanded
			? HUD_CARD_WIDTH + mixerPanelWidth
			: HUD_CARD_WIDTH;
	const shellH = isHorizontal
		? mixerExpanded
			? HUD_HORIZONTAL_HEIGHT + MIXER_TOP_PANEL_HEIGHT
			: HUD_HORIZONTAL_HEIGHT
		: HUD_CARD_HEIGHT;

	const docked = computeDockedOverlayGeometry(
		activeEdge,
		position,
		shellW,
		shellH,
		screenW,
		screenH
	);

	const { isDragging, dragPos, handleCardPointerDown } = useOverlayCardDrag({
		enabled: freeDragEnabled,
		scale: safeScale,
		screenW,
		screenH,
		dockedGeometry: docked,
		dragW: shellW,
		dragH: shellH,
		ignoreSelector: ".volume-notch-bar, .volume-mixer-bar, .volume-mixer-tile, .volume-mixer-panel, button",
		onDragStart,
		onPositionChange,
		onDragEnd
	});

	const renderX = isDragging && dragPos ? dragPos.x : docked.x;
	const renderY = isDragging && dragPos ? dragPos.y : docked.y;

	// Flash the percentage in place of the icon whenever the volume changes.
	useEffect(() => {
		setShowPercentage(true);
		if (percentageTimeoutRef.current) clearTimeout(percentageTimeoutRef.current);
		percentageTimeoutRef.current = setTimeout(() => setShowPercentage(false), 1200);
		return () => {
			if (percentageTimeoutRef.current) clearTimeout(percentageTimeoutRef.current);
		};
	}, [volume]);

	// A slider drag can end anywhere (including outside the panel); clear the
	// drag marker globally so the polling refresh stops preserving its local value.
	useEffect(() => {
		const clearDrag = () => {
			draggingRef.current = null;
		};
		window.addEventListener("pointerup", clearDrag);
		window.addEventListener("pointercancel", clearDrag);
		return () => {
			window.removeEventListener("pointerup", clearDrag);
			window.removeEventListener("pointercancel", clearDrag);
		};
	}, []);

	const refreshSessions = useCallback(() => {
		invoke<AudioSession[]>("get_audio_sessions")
			.then((list) => {
				lastKnownSessions = list;
				setSessions((prev) => {
					const prevByPid = new Map(prev.map((s) => [s.pid, s]));
					return list.map((session) => {
						if (draggingRef.current === session.pid) {
							const local = prevByPid.get(session.pid);
							if (local) {
								return { ...session, volume: local.volume, is_muted: local.is_muted };
							}
						}
						return session;
					});
				});
				setSessionsLoaded(true);
				setSessionsError(false);
			})
			.catch(() => {
				setSessionsLoaded(true);
				setSessionsError(true);
			});
	}, []);

	useEffect(() => {
		if (!mixerExpanded) return;
		refreshSessions();
		const interval = setInterval(refreshSessions, 2000);
		return () => clearInterval(interval);
	}, [mixerExpanded, refreshSessions]);

	// Report the shell bounds (notch + expanded panel) so the mouse hook keeps
	// the whole card clickable and closes it as soon as the cursor leaves.
	const expandedRef = useRef(mixerExpanded);
	useEffect(() => {
		expandedRef.current = mixerExpanded;
	}, [mixerExpanded]);

	const reportPanelBounds = useCallback(() => {
		if (!expandedRef.current) return;
		const el = shellRef.current;
		if (!el) return;
		const rect = el.getBoundingClientRect();
		invoke("set_volume_mixer_rect", {
			x: rect.x,
			y: rect.y,
			width: rect.width,
			height: rect.height
		}).catch(() => {});
	}, []);

	useOverlayRectSync("update_volume_overlay_rect", renderX, renderY, shellW, shellH, activeEdge);

	useLayoutEffect(() => {
		if (!mixerExpanded) return;
		reportPanelBounds();
		const el = shellRef.current;
		const observer = el ? new ResizeObserver(reportPanelBounds) : null;
		if (el && observer) observer.observe(el);
		return () => {
			observer?.disconnect();
			invoke("clear_volume_mixer_rect").catch(() => {});
		};
	}, [mixerExpanded, reportPanelBounds, renderX, renderY, shellW, shellH, activeEdge]);

	// Slider drags fire many times per second. Coalesce into one backend call
	// per ~60ms and always flush the final value, so releasing quickly doesn't
	// leave the backend (and the next poll) snapped back to a stale volume.
	const pendingAppVolumes = useRef<Map<number, number>>(new Map());
	const appVolumeFlushRef = useRef<any>(null);

	const flushAppVolumes = useCallback(() => {
		appVolumeFlushRef.current = null;
		const pending = pendingAppVolumes.current;
		pendingAppVolumes.current = new Map();
		pending.forEach((volume, pid) => {
			invoke("set_app_volume", { pid, volume }).catch(() => {});
		});
	}, []);

	const handleAppVolumeChange = useCallback(
		(pid: number, newVol: number) => {
			setSessions((prev) =>
				prev.map((s) =>
					s.pid === pid ? { ...s, volume: newVol, is_muted: newVol === 0 ? s.is_muted : false } : s
				)
			);
			pendingAppVolumes.current.set(pid, newVol);
			if (appVolumeFlushRef.current === null) {
				appVolumeFlushRef.current = setTimeout(flushAppVolumes, 60);
			}
		},
		[flushAppVolumes]
	);

	// Send anything still queued when the panel goes away.
	useEffect(
		() => () => {
			if (appVolumeFlushRef.current !== null) {
				clearTimeout(appVolumeFlushRef.current);
				flushAppVolumes();
			}
		},
		[flushAppVolumes]
	);

	const handleAppMuteToggle = useCallback(
		(pid: number) => {
			const session = sessions.find((s) => s.pid === pid);
			const muted = !(session?.is_muted ?? false);
			setSessions((prev) => prev.map((s) => (s.pid === pid ? { ...s, is_muted: muted } : s)));
			invoke("set_app_mute", { pid, muted }).catch(() => {});
		},
		[sessions]
	);

	const handleBarInteraction = (e: React.MouseEvent | React.TouchEvent) => {
		if (!barRef.current) return;
		const rect = barRef.current.getBoundingClientRect();
		if (isHorizontal) {
			const clientX = "touches" in e ? e.touches[0].clientX : e.clientX;
			const relativeX = clientX - rect.left;
			const newVolume = Math.max(0, Math.min(1, relativeX / rect.width));
			onVolumeChange(newVolume);
		} else {
			const clientY = "touches" in e ? e.touches[0].clientY : e.clientY;
			const relativeY = rect.bottom - clientY;
			const newVolume = Math.max(0, Math.min(1, relativeY / rect.height));
			onVolumeChange(newVolume);
		}
	};

	const startDrag = useMouseSliderDrag((clientX, clientY) => {
		if (!barRef.current) return;
		const rect = barRef.current.getBoundingClientRect();
		if (isHorizontal) {
			const relativeX = clientX - rect.left;
			const newVolume = Math.max(0, Math.min(1, relativeX / rect.width));
			onVolumeChange(newVolume);
		} else {
			const relativeY = rect.bottom - clientY;
			const newVolume = Math.max(0, Math.min(1, relativeY / rect.height));
			onVolumeChange(newVolume);
		}
	});
	const handleMouseDown = (e: React.MouseEvent) => {
		startDrag(e.clientX, e.clientY);
	};

	const handleWheel = (e: React.WheelEvent) => {
		e.stopPropagation();
		const delta = e.deltaY < 0 ? 0.04 : -0.04;
		const newVolume = Math.max(0, Math.min(1, Math.round((volume + delta) * 100) / 100));
		onVolumeChange(newVolume);
	};

	const wrapperPosStyle = buildWrapperPosStyle(
		activeEdge,
		isDragging,
		dragPos,
		docked,
		Boolean(position)
	);

	const restY: number | string = isDragging || isHorizontal || position ? 0 : "-50%";
	const exitAnim = isHorizontal
		? mixerExpanded
			? { y: -48, opacity: 0, filter: "blur(8px)", transition: { duration: 0.22, ease: [0.16, 1, 0.3, 1] as const } }
			: { scaleY: 0, opacity: 0, filter: "blur(8px)", y: 0, transition: { duration: 0.2 } }
		: mixerExpanded
			? {
					// Retract the expanded card as one surface. Scaling X would
					// squash the app list into the edge.
					x: activeEdge === "right" ? 64 : -64,
					opacity: 0,
					filter: "blur(8px)",
					y: restY,
					transition: { duration: 0.24, ease: [0.16, 1, 0.3, 1] as const }
				}
			: {
					scaleX: 0,
					scaleY: 0.8,
					opacity: 0,
					filter: "blur(12px)",
					y: restY,
					transition: { duration: 0.22, ease: [0.16, 1, 0.3, 1] as const }
				};

	const contentSlideOffset = getContentSlideOffset(activeEdge);

	const mixerExitAnim = isHorizontal
		? { opacity: 0, y: -18, transition: { y: { duration: 0.28, ease: [0.32, 0.72, 0, 1] as const }, opacity: { duration: 0.18, delay: 0.1 } } }
		: {
				opacity: 0,
				x: activeEdge === "right" ? 18 : -18,
				transition: {
					x: { duration: 0.28, ease: [0.32, 0.72, 0, 1] as const },
					opacity: { duration: 0.18, delay: 0.1 }
				}
			};

	return (
		<motion.div
			className={`volume-notch-wrapper ${isDragging ? "is-dragging" : `is-edge-${activeEdge}`} ${isHorizontal ? "is-horizontal" : ""}`}
			style={wrapperPosStyle}
			initial={{
				scaleX: isHorizontal ? 1 : 0,
				scaleY: isHorizontal ? 0 : 0.5,
				opacity: 0,
				filter: "blur(12px)",
				y: restY
			}}
			animate={{
				scaleX: 1,
				scaleY: 1,
				opacity: 1,
				filter: "blur(0px)",
				y: restY
			}}
			onPointerDown={handleCardPointerDown}
			exit={exitAnim}
			transition={{ type: "spring", stiffness: 320, damping: 28, mass: 0.75 }}
		>
			<EdgeSurface edge={activeEdge} flares={!isDragging} resolutionScale={safeScale} />
			<motion.div
				ref={shellRef}
				className={`volume-notch ${freeDragEnabled ? "free-drag-enabled" : ""} ${isHorizontal ? "is-horizontal" : ""}`}
				animate={{
					width: shellW,
					height: shellH
				}}
				transition={{ type: "spring", bounce: 0, duration: 0.35 }}
				onAnimationComplete={reportPanelBounds}
				onWheel={handleWheel}
			>
				{freeDragEnabled && (
					<div
						className="overlay-drag-handle"
						title="Click-hold to drag"
					/>
				)}
				<motion.div
					className={`volume-notch-content-group ${isHorizontal ? "is-horizontal" : ""}`}
					initial={{ opacity: 0, ...contentSlideOffset }}
					animate={{ opacity: 1, x: 0, y: 0 }}
					exit={{
						opacity: 0,
						x: contentSlideOffset.x / 2,
						y: contentSlideOffset.y / 2,
						transition: { duration: 0.15 }
					}}
					transition={{ delay: 0.05, duration: 0.2, ease: "easeOut" }}
				>
					{isHorizontal ? (
						<>
							<button
								className={`volume-notch-action ${mixerExpanded ? "active" : ""} ${
									showPercentage ? "showing-percent" : ""
								}`}
								onClick={(e) => {
									e.stopPropagation();
									onMixerExpandedChange(!mixerExpanded);
								}}
								title="App volume mixer"
							>
								<span
									className="volume-notch-action-icon"
									style={mixerExpanded ? { opacity: 0, pointerEvents: "none" } : undefined}
									aria-hidden={mixerExpanded}
								>
									<SpeakerIcon size={16} muted={isMuted} />
								</span>
								<span
									className="volume-notch-action-mixer"
									style={!mixerExpanded ? { opacity: 0, pointerEvents: "none" } : undefined}
									aria-hidden={!mixerExpanded}
								>
									<MixerIcon size={16} />
								</span>
								<span className="volume-notch-action-percent">{isMuted ? "0%" : `${percentage}%`}</span>
							</button>

							<div
								ref={barRef}
								className="volume-notch-bar"
								onMouseDown={handleMouseDown}
								onTouchStart={handleBarInteraction}
								onWheel={handleWheel}
								style={{ cursor: "pointer" }}
							>
								<motion.div
									className="volume-notch-fill"
									initial={false}
									animate={{ width: isMuted ? "0%" : `${percentage}%`, height: "100%" }}
									transition={{ type: "spring", stiffness: 300, damping: 35 }}
								/>
							</div>
						</>
					) : (
						<>
							<div
								ref={barRef}
								className="volume-notch-bar"
								onMouseDown={handleMouseDown}
								onTouchStart={handleBarInteraction}
								onWheel={handleWheel}
								style={{ cursor: "pointer" }}
							>
								<motion.div
									className="volume-notch-fill"
									initial={false}
									animate={{ height: isMuted ? "0%" : `${percentage}%` }}
									transition={{ type: "spring", stiffness: 300, damping: 35 }}
								/>
							</div>
							<button
								className={`volume-notch-action ${mixerExpanded ? "active" : ""} ${
									showPercentage ? "showing-percent" : ""
								}`}
								onClick={(e) => {
									e.stopPropagation();
									onMixerExpandedChange(!mixerExpanded);
								}}
								title="App volume mixer"
							>
								<span
									className="volume-notch-action-icon"
									style={mixerExpanded ? { opacity: 0, pointerEvents: "none" } : undefined}
									aria-hidden={mixerExpanded}
								>
									<SpeakerIcon size={16} muted={isMuted} />
								</span>
								<span
									className="volume-notch-action-mixer"
									style={!mixerExpanded ? { opacity: 0, pointerEvents: "none" } : undefined}
									aria-hidden={!mixerExpanded}
								>
									<MixerIcon size={16} />
								</span>
								<span className="volume-notch-action-percent">{isMuted ? "0%" : `${percentage}%`}</span>
							</button>
						</>
					)}
				</motion.div>

				<AnimatePresence>
					{mixerExpanded && (
						<motion.div
							className="volume-mixer-panel"
							style={isHorizontal ? undefined : { width: mixerPanelWidth }}
							initial={{ opacity: 0 }}
							animate={{ opacity: 1 }}
							exit={mixerExitAnim}
						>
							<div
								className="volume-mixer-list"
								style={{ gridTemplateColumns: `repeat(${mixerColumns}, 1fr)` }}
							>
								{sessions.length === 0 ? (
									<div className="volume-mixer-empty">
										{!sessionsLoaded
											? "Loading..."
											: sessionsError
												? "Couldn't read audio sessions"
												: "No apps with audio"}
									</div>
								) : (
									sessions.map((session) => (
										<MixerTile
											key={session.pid}
											session={session}
											onVolumeChange={handleAppVolumeChange}
											onMuteToggle={handleAppMuteToggle}
											draggingRef={draggingRef}
										/>
									))
								)}
							</div>
						</motion.div>
					)}
				</AnimatePresence>
			</motion.div>
		</motion.div>
	);
}

// ─── Brightness Notch ───────────────────────────────────────────────────────

const SunIcon = ({ brightness }: { brightness: number }) => (
	<motion.div
		className="brightness-icon-container"
		animate={{ rotate: (brightness / 100) * 180 }}
		transition={{ type: "spring", stiffness: 200, damping: 25 }}
	>
		<svg width="20" height="20" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
			<circle cx="12" cy="12" r="5" stroke="currentColor" strokeWidth="2.5" />
			<path
				d="M12 2V4M12 20V22M4.93 4.93L6.34 6.34M17.66 17.66L19.07 19.07M2 12H4M20 12H22M4.93 19.07L6.34 17.66M17.66 6.34L19.07 4.93"
				stroke="currentColor"
				strokeWidth="2.5"
				strokeLinecap="round"
			/>
		</svg>
	</motion.div>
);

function BrightnessNotch({
	brightness,
	onBrightnessChange,
	freeDragEnabled,
	scale,
	position,
	onPositionChange,
	onDragStart,
	onDragEnd
}: {
	brightness: number;
	onBrightnessChange: (val: number) => void;
	freeDragEnabled?: boolean;
	scale: number;
	position?: OverlayPosition | null;
	onPositionChange?: (pos: OverlayPosition) => void;
	onDragStart?: () => void;
	onDragEnd?: () => void;
}) {
	const barRef = useRef<HTMLDivElement>(null);
	const [showPercentage, setShowPercentage] = useState(false);
	const percentageTimeoutRef = useRef<any>(null);

	const percentage = Math.round(brightness);
	const viewport = useViewportSize();
	const safeScale = scale > 0 ? scale : 1;
	const screenW = viewport.width / safeScale;
	const screenH = viewport.height / safeScale;
	const activeEdge = resolveOverlayEdge(position, "right", screenW);
	const isHorizontal = activeEdge === "top";
	const shellW = isHorizontal ? HUD_HORIZONTAL_WIDTH : HUD_CARD_WIDTH;
	const shellH = isHorizontal ? HUD_HORIZONTAL_HEIGHT : HUD_CARD_HEIGHT;

	const docked = computeDockedOverlayGeometry(
		activeEdge,
		position,
		shellW,
		shellH,
		screenW,
		screenH
	);

	const { isDragging, dragPos, handleCardPointerDown } = useOverlayCardDrag({
		enabled: freeDragEnabled,
		scale: safeScale,
		screenW,
		screenH,
		dockedGeometry: docked,
		dragW: shellW,
		dragH: shellH,
		ignoreSelector: ".brightness-notch-bar, button",
		onDragStart,
		onPositionChange,
		onDragEnd
	});

	const renderX = isDragging && dragPos ? dragPos.x : docked.x;
	const renderY = isDragging && dragPos ? dragPos.y : docked.y;
	useOverlayRectSync("update_brightness_overlay_rect", renderX, renderY, shellW, shellH, activeEdge);

	// Flash the percentage in place of the icon whenever the brightness changes.
	useEffect(() => {
		setShowPercentage(true);
		if (percentageTimeoutRef.current) clearTimeout(percentageTimeoutRef.current);
		percentageTimeoutRef.current = setTimeout(() => setShowPercentage(false), 1200);
		return () => {
			if (percentageTimeoutRef.current) clearTimeout(percentageTimeoutRef.current);
		};
	}, [brightness]);

	const handleBarInteraction = (e: React.MouseEvent | React.TouchEvent) => {
		if (!barRef.current) return;
		const rect = barRef.current.getBoundingClientRect();
		if (isHorizontal) {
			const clientX = "touches" in e ? e.touches[0].clientX : e.clientX;
			const relativeX = clientX - rect.left;
			const newBrightness = Math.max(0, Math.min(100, (relativeX / rect.width) * 100));
			onBrightnessChange(newBrightness);
		} else {
			const clientY = "touches" in e ? e.touches[0].clientY : e.clientY;
			const relativeY = rect.bottom - clientY;
			const newBrightness = Math.max(0, Math.min(100, (relativeY / rect.height) * 100));
			onBrightnessChange(newBrightness);
		}
	};

	const startDrag = useMouseSliderDrag((clientX, clientY) => {
		if (!barRef.current) return;
		const rect = barRef.current.getBoundingClientRect();
		if (isHorizontal) {
			const relativeX = clientX - rect.left;
			const newBrightness = Math.max(0, Math.min(100, (relativeX / rect.width) * 100));
			onBrightnessChange(newBrightness);
		} else {
			const relativeY = rect.bottom - clientY;
			const newBrightness = Math.max(0, Math.min(100, (relativeY / rect.height) * 100));
			onBrightnessChange(newBrightness);
		}
	});
	const handleMouseDown = (e: React.MouseEvent) => {
		startDrag(e.clientX, e.clientY);
	};

	const handleWheel = (e: React.WheelEvent) => {
		e.stopPropagation();
		const delta = e.deltaY < 0 ? 4 : -4;
		const newBrightness = Math.max(0, Math.min(100, Math.round(brightness + delta)));
		onBrightnessChange(newBrightness);
	};

	const wrapperPosStyle = buildWrapperPosStyle(
		activeEdge,
		isDragging,
		dragPos,
		docked,
		Boolean(position)
	);
	const restY: number | string = isDragging || isHorizontal || position ? 0 : "-50%";
	const contentSlideOffset = getContentSlideOffset(activeEdge);

	return (
		<motion.div
			className={`brightness-notch-wrapper ${isDragging ? "is-dragging" : `is-edge-${activeEdge}`} ${isHorizontal ? "is-horizontal" : ""}`}
			style={wrapperPosStyle}
			initial={{
				scaleX: isHorizontal ? 1 : 0,
				scaleY: isHorizontal ? 0 : 0.5,
				opacity: 0,
				filter: "blur(12px)",
				y: restY
			}}
			animate={{
				scaleX: 1,
				scaleY: 1,
				opacity: 1,
				filter: "blur(0px)",
				y: restY
			}}
			onPointerDown={handleCardPointerDown}
			exit={
				isHorizontal
					? { scaleY: 0, opacity: 0, filter: "blur(8px)", y: 0, transition: { duration: 0.2 } }
					: {
							scaleX: 0,
							scaleY: 0.8,
							opacity: 0,
							filter: "blur(12px)",
							y: restY,
							transition: { duration: 0.22, ease: [0.16, 1, 0.3, 1] }
						}
			}
			transition={{ type: "spring", stiffness: 320, damping: 28, mass: 0.75 }}
		>
			<EdgeSurface edge={activeEdge} flares={!isDragging} resolutionScale={safeScale} />
			<div
				className={`brightness-notch ${freeDragEnabled ? "free-drag-enabled" : ""} ${isHorizontal ? "is-horizontal" : ""}`}
				onWheel={handleWheel}
			>
				{freeDragEnabled && (
					<div
						className="overlay-drag-handle"
						title="Click-hold to drag"
					/>
				)}
				<motion.div
					className={`brightness-notch-content-group ${isHorizontal ? "is-horizontal" : ""}`}
					initial={{ opacity: 0, ...contentSlideOffset }}
					animate={{ opacity: 1, x: 0, y: 0 }}
					exit={{
						opacity: 0,
						x: contentSlideOffset.x / 2,
						y: contentSlideOffset.y / 2,
						transition: { duration: 0.15 }
					}}
					transition={{ delay: 0.05, duration: 0.2, ease: "easeOut" }}
				>
					{isHorizontal ? (
						<>
							<div
								className={`brightness-notch-action ${showPercentage ? "showing-percent" : ""}`}
							>
								<span className="brightness-notch-action-icon">
									<SunIcon brightness={brightness} />
								</span>
								<span className="brightness-notch-action-percent">{percentage}%</span>
							</div>

							<div
								ref={barRef}
								className="brightness-notch-bar"
								onMouseDown={handleMouseDown}
								onTouchStart={handleBarInteraction}
								onWheel={handleWheel}
								style={{ cursor: "pointer" }}
							>
								<motion.div
									className="brightness-notch-fill"
									initial={false}
									animate={{ width: `${brightness}%`, height: "100%" }}
									transition={{ type: "spring", stiffness: 300, damping: 35 }}
								/>
							</div>
						</>
					) : (
						<>
							<div
								ref={barRef}
								className="brightness-notch-bar"
								onMouseDown={handleMouseDown}
								onTouchStart={handleBarInteraction}
								onWheel={handleWheel}
								style={{ cursor: "pointer" }}
							>
								<motion.div
									className="brightness-notch-fill"
									initial={false}
									animate={{ height: `${brightness}%` }}
									transition={{ type: "spring", stiffness: 300, damping: 35 }}
								/>
							</div>
							<div
								className={`brightness-notch-action ${showPercentage ? "showing-percent" : ""}`}
							>
								<span className="brightness-notch-action-icon">
									<SunIcon brightness={brightness} />
								</span>
								<span className="brightness-notch-action-percent">{percentage}%</span>
							</div>
						</>
					)}
				</motion.div>
			</div>
		</motion.div>
	);
}

// ─── Main Overlay Component ─────────────────────────────────────────────────

function OverlayApp() {
	useEffect(() => {
		return initTheme();
	}, []);

	type Mode = "idle" | "volume" | "brightness" | "splash" | "updating";
	const [mode, setMode] = useState<Mode>("idle");
	const [updateStatus, setUpdateStatus] = useState<string>("checking");
	const [updateProgress, setUpdateProgress] = useState(0);

	// Volume state
	const [volume, setVolume] = useState(0.5);
	const [isMuted, setIsMuted] = useState(false);
	const [mixerExpanded, setMixerExpanded] = useState(false);
	const [volumeOverlayEnabled, setVolumeOverlayEnabled] = useState(
		() => localStorage.getItem("bloom-volume-overlay-enabled") !== "false"
	);
	const [volumeEdgeEnabled, setVolumeEdgeEnabled] = useState(
		() => localStorage.getItem("bloom-volume-edge-enabled") !== "false"
	);

	// Brightness state
	const [brightness, setBrightness] = useState(50);
	const [brightnessOverlayEnabled, setBrightnessOverlayEnabled] = useState(
		() => localStorage.getItem("bloom-brightness-overlay-enabled") !== "false"
	);
	const [brightnessEdgeEnabled, setBrightnessEdgeEnabled] = useState(
		() => localStorage.getItem("bloom-brightness-edge-enabled") !== "false"
	);

	// Free Drag state
	const [freeDragEnabled, setFreeDragEnabled] = useState(
		() => localStorage.getItem("bloom-free-drag-enabled") === "true"
	);
	const [volumePosition, setVolumePosition] = useState<OverlayPosition | null>(() => {
		if (localStorage.getItem("bloom-free-drag-enabled") !== "true") return null;
		try {
			const raw = localStorage.getItem("bloom-volume-overlay-position");
			if (raw) {
				const p = JSON.parse(raw);
				if (typeof p?.x === "number" && typeof p?.y === "number") return p;
			}
		} catch {}
		return null;
	});
	const [brightnessPosition, setBrightnessPosition] = useState<OverlayPosition | null>(() => {
		if (localStorage.getItem("bloom-free-drag-enabled") !== "true") return null;
		try {
			const raw = localStorage.getItem("bloom-brightness-overlay-position");
			if (raw) {
				const p = JSON.parse(raw);
				if (typeof p?.x === "number" && typeof p?.y === "number") return p;
			}
		} catch {}
		return null;
	});

	// Shared state
	const [scale, setScale] = useState(() =>
		parseFloat(localStorage.getItem("bloom-scale") || "1.0")
	);
	const volumePositionRef = useRef(volumePosition);
	const brightnessPositionRef = useRef(brightnessPosition);
	useEffect(() => {
		volumePositionRef.current = volumePosition;
	}, [volumePosition]);
	useEffect(() => {
		brightnessPositionRef.current = brightnessPosition;
	}, [brightnessPosition]);

	const timeoutRef = useRef<any>(null);
	const hideWindowTimeoutRef = useRef<any>(null);
	const splashActiveRef = useRef(false);
	const mixerExpandedRef = useRef(false);

	const resetHideTimeout = useCallback(() => {
		// Keep the notch pinned while the mixer panel is open; it closes on
		// mouse leave via the edge-hover timeout instead.
		if (mixerExpandedRef.current) return;
		if (timeoutRef.current) clearTimeout(timeoutRef.current);
		timeoutRef.current = setTimeout(() => {
			if (!splashActiveRef.current) setMode("idle");
		}, 2500);
	}, []);

	// Load scale from settings
	useEffect(() => {
		invoke("load_settings")
			.then(async (settings: any) => {
				if (await reloadIfMirrorWasStale(settings)) return;
				if (settings && settings["bloom-scale"] !== undefined) {
					setScale(parseFloat(settings["bloom-scale"]));
				}
				if (settings && settings["bloom-brightness-overlay-enabled"] !== undefined) {
					setBrightnessOverlayEnabled(settings["bloom-brightness-overlay-enabled"] === "true");
				}
				if (settings && settings["bloom-volume-overlay-enabled"] !== undefined) {
					setVolumeOverlayEnabled(settings["bloom-volume-overlay-enabled"] === "true");
				}
				if (settings && settings["bloom-free-drag-enabled"] !== undefined) {
					setFreeDragEnabled(settings["bloom-free-drag-enabled"] === "true");
				}
				if (settings && settings["bloom-volume-overlay-position"]) {
					try {
						const pos = typeof settings["bloom-volume-overlay-position"] === "string"
							? JSON.parse(settings["bloom-volume-overlay-position"])
							: settings["bloom-volume-overlay-position"];
						if (typeof pos?.x === "number" && typeof pos?.y === "number") {
							setVolumePosition(pos);
							volumePositionRef.current = pos;
						}
					} catch {}
				}
				if (settings && settings["bloom-brightness-overlay-position"]) {
					try {
						const pos = typeof settings["bloom-brightness-overlay-position"] === "string"
							? JSON.parse(settings["bloom-brightness-overlay-position"])
							: settings["bloom-brightness-overlay-position"];
						if (typeof pos?.x === "number" && typeof pos?.y === "number") {
							setBrightnessPosition(pos);
							brightnessPositionRef.current = pos;
						}
					} catch {}
				}
			})
			.catch(console.error);
	}, []);

	// Seed the HUD with the live system volume; the system worker's only
	// startup event fires before this webview can register listeners.
	useEffect(() => {
		invoke<{ volume: number; is_muted: boolean }>("get_volume_state")
			.then((state) => {
				setVolume(state.volume);
				setIsMuted(state.is_muted);
			})
			.catch(() => {});
	}, []);

	// ── Splash Detection ──
	useEffect(() => {
		const firstRun = localStorage.getItem("bloom-first-run") === null;
		const storedVersion = localStorage.getItem("bloom-app-version");

		const showSplash = (version?: string) => {
			splashActiveRef.current = true;
			setMode("splash");
			invoke("set_splash_fullscreen", { fullscreen: true });
			if (version) localStorage.setItem("bloom-app-version", version);
			setTimeout(() => emit("splash-done"), 2800);
		};

		// First run or old version without version key — splash immediately
		if (firstRun || storedVersion === null) {
			showSplash();
			// Still try to store the version in the background
			getVersion()
				.then((v) => localStorage.setItem("bloom-app-version", v))
				.catch(() => {});
			return;
		}

		// Has version key — check if it matches
		getVersion()
			.then((currentVersion) => {
				if (storedVersion !== currentVersion) {
					showSplash(currentVersion);
				} else {
					emit("splash-done");
				}
			})
			.catch(() => {
				// Version check failed — let bloom start
				emit("splash-done");
			});
	}, []);

	const onSplashComplete = useCallback(() => {
		localStorage.setItem("bloom-first-run", "done");
		setTimeout(() => {
			splashActiveRef.current = false;
			setMode("idle");
			invoke("set_splash_fullscreen", { fullscreen: false });
		}, 300);
	}, []);

	// ── Event Listeners ──
	useEffect(() => {
		const preventContext = (e: MouseEvent) => e.preventDefault();
		document.addEventListener("contextmenu", preventContext);

		const volPromise = listen("volume-change", (event: any) => {
			if (splashActiveRef.current || !volumeOverlayEnabled) return;
			invoke("hide_native_osd");
			const { volume: newVolume, is_muted } = event.payload;
			setVolume(newVolume);
			setIsMuted(is_muted);
			setMode("volume");
			resetHideTimeout();
		});

		const volEdgePromise = listen<boolean>("volume-edge-hover", (event) => {
			if (splashActiveRef.current || !volumeOverlayEnabled || !volumeEdgeEnabled) return;
			if (event.payload) {
				setMode("volume");
				if (timeoutRef.current) clearTimeout(timeoutRef.current);
			} else {
				if (timeoutRef.current) clearTimeout(timeoutRef.current);
				timeoutRef.current = setTimeout(
					() => setMode("idle"),
					mixerExpandedRef.current ? 700 : 2500
				);
			}
		});

		const brightPromise = listen("brightness-change", (event: any) => {
			if (splashActiveRef.current || !brightnessOverlayEnabled) return;
			invoke("hide_native_osd");
			const { brightness: newBrightness } = event.payload;
			setBrightness(newBrightness);
			setMode("brightness");
			resetHideTimeout();
		});

		const brightEdgePromise = listen<boolean>("brightness-edge-hover", (event) => {
			if (splashActiveRef.current || !brightnessOverlayEnabled || !brightnessEdgeEnabled) return;
			if (event.payload) {
				setMode("brightness");
				if (timeoutRef.current) clearTimeout(timeoutRef.current);
			} else {
				if (timeoutRef.current) clearTimeout(timeoutRef.current);
				timeoutRef.current = setTimeout(() => setMode("idle"), 2500);
			}
		});

		const autoUpdatePromise = listen<{ status: string; progress?: number }>(
			"auto-update-status",
			(event) => {
				const { status, progress } = event.payload;
				setUpdateStatus(status);
				if (progress !== undefined) setUpdateProgress(progress);

				if (status === "checking" || status === "downloading" || status === "installing") {
					splashActiveRef.current = true;
					setMode("updating");
					invoke("set_splash_fullscreen", { fullscreen: true });
				} else if (status === "done") {
					splashActiveRef.current = false;
					setMode("idle");
					invoke("set_splash_fullscreen", { fullscreen: false });
				}
			}
		);

		return () => {
			volPromise.then((fn) => fn());
			volEdgePromise.then((fn) => fn());
			brightPromise.then((fn) => fn());
			brightEdgePromise.then((fn) => fn());
			autoUpdatePromise.then((fn) => fn());
			document.removeEventListener("contextmenu", preventContext);
		};
	}, [
		volumeOverlayEnabled,
		volumeEdgeEnabled,
		brightnessOverlayEnabled,
		brightnessEdgeEnabled,
		resetHideTimeout
	]);

	useSettingsSync({
		"bloom-volume-overlay-enabled": setVolumeOverlayEnabled,
		"bloom-volume-edge-enabled": setVolumeEdgeEnabled,
		"bloom-brightness-overlay-enabled": setBrightnessOverlayEnabled,
		"bloom-brightness-edge-enabled": setBrightnessEdgeEnabled,
		"bloom-scale": setScale,
		"bloom-free-drag-enabled": setFreeDragEnabled,
		"bloom-volume-overlay-position": (val: any) => {
			if (!val) {
				setVolumePosition(null);
				volumePositionRef.current = null;
				return;
			}
			try {
				const p = typeof val === "string" ? JSON.parse(val) : val;
				if (typeof p?.x === "number" && typeof p?.y === "number") {
					setVolumePosition(p);
					volumePositionRef.current = p;
				}
			} catch {}
		},
		"bloom-brightness-overlay-position": (val: any) => {
			if (!val) {
				setBrightnessPosition(null);
				brightnessPositionRef.current = null;
				return;
			}
			try {
				const p = typeof val === "string" ? JSON.parse(val) : val;
				if (typeof p?.x === "number" && typeof p?.y === "number") {
					setBrightnessPosition(p);
					brightnessPositionRef.current = p;
				}
			} catch {}
		}
	});

	// Side effects: reset overlay mode to idle when overlay is disabled
	useEffect(() => {
		if (splashActiveRef.current) return;
		if (!volumeOverlayEnabled && mode === "volume") setMode("idle");
	}, [volumeOverlayEnabled, mode]);

	useEffect(() => {
		if (splashActiveRef.current) return;
		if (!brightnessOverlayEnabled && mode === "brightness") setMode("idle");
	}, [brightnessOverlayEnabled, mode]);

	// Clear overlay hit-test rects in Rust as soon as each mode leaves active state.
	useEffect(() => {
		if (mode !== "volume") {
			invoke("update_volume_overlay_rect", { rect: null }).catch(() => {});
		}
		if (mode !== "brightness") {
			invoke("update_brightness_overlay_rect", { rect: null }).catch(() => {});
		}
	}, [mode]);

	// Global mouse hook dismissal when clicking outside volume/brightness overlay
	useEffect(() => {
		const unlisten = listen("overlay-dismiss", () => {
			if ((window as any).__isFileDialogOpen || (window as any).isFileDialogOpen || (window as any).__himeFileDialogOpen) return;
			if (timeoutRef.current) clearTimeout(timeoutRef.current);
			setMode("idle");
		});
		return () => {
			unlisten.then((f) => f());
		};
	}, []);

	// ── Window Visibility Management ──
	useEffect(() => {
		const syncWindow = async () => {
			try {
				const appWindow = getCurrentWebviewWindow();
				if (hideWindowTimeoutRef.current) {
					clearTimeout(hideWindowTimeoutRef.current);
					hideWindowTimeoutRef.current = null;
				}

				if (mode === "idle") {
					// Wait for exit animation to finish before hiding
					hideWindowTimeoutRef.current = setTimeout(async () => {
						await appWindow.hide();
					}, 400);
				} else {
					// Position the window first, then show
					await invoke("sync_overlay_position");
					await appWindow.show();
				}
			} catch (e) {
				console.error("Window management error:", e);
			}
		};

		// Don't manage window visibility during splash — Rust handles it
		if (mode !== "splash" && !splashActiveRef.current) {
			syncWindow();
		}

		return () => {
			if (hideWindowTimeoutRef.current) clearTimeout(hideWindowTimeoutRef.current);
		};
	}, [mode]);

	// ── Volume Controls ──
	const sendVolume = useTrailingThrottle((newVol: number) => {
		invoke("set_volume", { volume: newVol }).catch(() => {});
	}, 50);
	const handleVolumeChange = useCallback(
		(newVol: number) => {
			setVolume(newVol);
			setIsMuted(newVol === 0);
			setMode("volume");
			resetHideTimeout();
			sendVolume(newVol);
		},
		[resetHideTimeout, sendVolume]
	);

	const handleMixerExpandedChange = useCallback(
		(expanded: boolean) => {
			mixerExpandedRef.current = expanded;
			setMixerExpanded(expanded);
			if (expanded) {
				// Cancel a hide that was scheduled before the mixer opened (e.g.
				// the edge-hover timeout) so opening it can't close it mid-click.
				if (timeoutRef.current) {
					clearTimeout(timeoutRef.current);
					timeoutRef.current = null;
				}
			} else {
				resetHideTimeout();
			}
		},
		[resetHideTimeout]
	);

	// Collapse the mixer only after the notch's exit animation finishes (see
	// onExitComplete on AnimatePresence) so an open panel retracts as one
	// surface. Splash/update modes unmount the overlay without an exit, so
	// reset there instead; otherwise the mixer would reopen expanded.
	const collapseMixer = useCallback(() => {
		mixerExpandedRef.current = false;
		setMixerExpanded(false);
	}, []);

	useEffect(() => {
		if ((mode === "splash" || mode === "updating") && mixerExpanded) {
			collapseMixer();
		}
	}, [mode, mixerExpanded, collapseMixer]);

	// ── Brightness Controls ──
	const sendBrightness = useTrailingThrottle((newBrightness: number) => {
		invoke("set_brightness", { brightness: Math.round(newBrightness) }).catch(() => {});
	}, 50);
	const handleBrightnessChange = useCallback(
		(newBrightness: number) => {
			setBrightness(newBrightness);
			setMode("brightness");
			resetHideTimeout();
			sendBrightness(newBrightness);
		},
		[resetHideTimeout, sendBrightness]
	);

	const handleDragStart = useCallback(() => {
		if (timeoutRef.current) {
			clearTimeout(timeoutRef.current);
			timeoutRef.current = null;
		}
	}, []);

	const handleVolumePositionChange = useCallback((pos: OverlayPosition) => {
		volumePositionRef.current = pos;
		setVolumePosition(pos);
	}, []);

	const handleVolumeDragEnd = useCallback(() => {
		const pos = volumePositionRef.current;
		if (pos) {
			localStorage.setItem("bloom-volume-overlay-position", JSON.stringify(pos));
			invoke("save_setting", { key: "bloom-volume-overlay-position", value: JSON.stringify(pos) }).catch(() => {});
		}
		resetHideTimeout();
	}, [resetHideTimeout]);

	const handleBrightnessPositionChange = useCallback((pos: OverlayPosition) => {
		brightnessPositionRef.current = pos;
		setBrightnessPosition(pos);
	}, []);

	const handleBrightnessDragEnd = useCallback(() => {
		const pos = brightnessPositionRef.current;
		if (pos) {
			localStorage.setItem("bloom-brightness-overlay-position", JSON.stringify(pos));
			invoke("save_setting", { key: "bloom-brightness-overlay-position", value: JSON.stringify(pos) }).catch(() => {});
		}
		resetHideTimeout();
	}, [resetHideTimeout]);

	const activeVolumePosition = freeDragEnabled ? volumePosition : null;
	const activeBrightnessPosition = freeDragEnabled ? brightnessPosition : null;

	const viewport = useViewportSize();
	const safeScale = scale > 0 ? scale : 1;
	const screenW = viewport.width / safeScale;
	const screenH = viewport.height / safeScale;
	const activeVolumeEdge = resolveOverlayEdge(activeVolumePosition, "left", screenW);
	const activeBrightnessEdge = resolveOverlayEdge(activeBrightnessPosition, "right", screenW);
	const activeModeEdge = mode === "volume" ? activeVolumeEdge : activeBrightnessEdge;

	const sendOverlayRegion = useMemo(
		() =>
			createRegionSender((rects) => {
				invoke("set_window_region", { rects }).catch(() => {});
			}),
		[]
	);

	// Clip the full-screen overlay to what it draws (see window_region.rs).
	// The splashes cover the screen. Otherwise each HUD has a home zone at its
	// edge, the full card size, so a card scaling in from the edge is never
	// clipped; with the mixer open the left zone spans the widest mixer. The
	// HUDs and the mixer panel are measured on top, padded for their shadows.
	useEffect(() => {
		const width = viewport.width;
		const height = viewport.height;
		if (mode === "splash" || mode === "updating") {
			sendOverlayRegion([{ x: 0, y: 0, width, height }]);
			return;
		}
		const pad = 24 * safeScale;
		const cardWidth = HUD_CARD_WIDTH * safeScale;
		const cardHeight = HUD_CARD_HEIGHT * safeScale;
		const leftWidth =
			(mixerExpanded ? HUD_CARD_WIDTH + MIXER_MAX_PANEL_WIDTH : HUD_CARD_WIDTH) * safeScale;
		const zone = (x: number, w: number): RegionRect => ({
			x,
			y: (height - cardHeight) / 2 - pad,
			width: w,
			height: cardHeight + pad * 2
		});
		const customRects: RegionRect[] = [];
		if (mode === "volume" && activeVolumePosition) {
			const isVolTop = activeVolumeEdge === "top";
			const w = isVolTop
				? mixerExpanded
					? Math.max(HUD_HORIZONTAL_WIDTH, MIXER_MAX_PANEL_WIDTH)
					: HUD_HORIZONTAL_WIDTH
				: mixerExpanded
					? HUD_CARD_WIDTH + MIXER_MAX_PANEL_WIDTH
					: HUD_CARD_WIDTH;
			const h = isVolTop
				? mixerExpanded
					? HUD_HORIZONTAL_HEIGHT + MIXER_TOP_PANEL_HEIGHT
					: HUD_HORIZONTAL_HEIGHT
				: HUD_CARD_HEIGHT;
			const dockedVol = computeDockedOverlayGeometry(
				activeVolumeEdge,
				activeVolumePosition,
				w,
				h,
				screenW,
				screenH
			);
			customRects.push({
				x: Math.max(0, dockedVol.x * safeScale - pad),
				y: Math.max(0, dockedVol.y * safeScale - pad),
				width: w * safeScale + pad * 2,
				height: h * safeScale + pad * 2
			});
		} else if (mode === "brightness" && activeBrightnessPosition) {
			const isBrightTop = activeBrightnessEdge === "top";
			const w = isBrightTop ? HUD_HORIZONTAL_WIDTH : HUD_CARD_WIDTH;
			const h = isBrightTop ? HUD_HORIZONTAL_HEIGHT : HUD_CARD_HEIGHT;
			const dockedBright = computeDockedOverlayGeometry(
				activeBrightnessEdge,
				activeBrightnessPosition,
				w,
				h,
				screenW,
				screenH
			);
			customRects.push({
				x: Math.max(0, dockedBright.x * safeScale - pad),
				y: Math.max(0, dockedBright.y * safeScale - pad),
				width: w * safeScale + pad * 2,
				height: h * safeScale + pad * 2
			});
		}
		sendOverlayRegion([
			zone(0, leftWidth + pad),
			zone(width - cardWidth - pad, cardWidth + pad),
			...customRects,
			...measureElements([
				[".volume-notch-wrapper", 24],
				[".volume-mixer-panel", 24],
				[".brightness-notch-wrapper", 24]
			])
		]);
	});

	// Click-outside dismissal: when volume or brightness slider is active, clicking outside immediately hides it
	useEffect(() => {
		if (mode === "idle" || mode === "splash" || mode === "updating") return;
		const handleOverlayOutsideClick = (e: MouseEvent | PointerEvent) => {
			if ((window as any).__isFileDialogOpen || (window as any).isFileDialogOpen || (window as any).__himeFileDialogOpen) return;
			const overlayEl =
				document.querySelector(".volume-notch-wrapper") ||
				document.querySelector(".brightness-notch-wrapper") ||
				document.querySelector(".volume-mixer-panel");
			if (overlayEl && !overlayEl.contains(e.target as Node)) {
				if (timeoutRef.current) {
					clearTimeout(timeoutRef.current);
					timeoutRef.current = null;
				}
				setMode("idle");
			}
		};
		window.addEventListener("pointerdown", handleOverlayOutsideClick, true);
		return () => {
			window.removeEventListener("pointerdown", handleOverlayOutsideClick, true);
		};
	}, [mode]);

	return (
		<div className="overlay-container">
			{/* Splash Screen */}
			<AnimatePresence>
				{mode === "splash" && (
					<motion.div
						className="splash-screen"
						initial={{ opacity: 1 }}
						exit={{ opacity: 0 }}
						transition={{ duration: 0.3 }}
					>
						<motion.img
							src="/hime.png"
							className="splash-logo"
							draggable={false}
							initial={{ scale: 0, opacity: 0, rotate: 0 }}
							animate={{
								scale: [0, 1.1, 1, 1.2, 1, 1, 0.2],
								opacity: [0, 1, 1, 1, 1, 1, 0],
								rotate: [0, 0, 0, 0, 0, 540, 1080]
							}}
							transition={{
								duration: 3.2,
								times: [0, 0.17, 0.3, 0.43, 0.56, 0.78, 1],
								ease: ["easeOut", "easeInOut", "easeInOut", "easeInOut", "linear", "linear"]
							}}
							onAnimationComplete={onSplashComplete}
						/>
					</motion.div>
				)}
			</AnimatePresence>

			{/* Auto-Update Splash */}
			<AnimatePresence>
				{mode === "updating" && (
					<motion.div
						className="update-splash"
						initial={{ opacity: 0 }}
						animate={{ opacity: 1 }}
						exit={{ opacity: 0 }}
						transition={{ duration: 0.3 }}
					>
						<img src="/hime.png" className="update-splash-logo" alt="Hime Island" />
						<p className="update-splash-text">
							{updateStatus === "checking" && "Checking for updates..."}
							{updateStatus === "downloading" && `Downloading update... ${updateProgress}%`}
							{updateStatus === "installing" && "Installing update..."}
						</p>
						{updateStatus === "downloading" && (
							<div className="update-progress-bar">
								<div className="update-progress-fill" style={{ width: `${updateProgress}%` }} />
							</div>
						)}
					</motion.div>
				)}
			</AnimatePresence>

			{/* Volume / Brightness Overlay */}
			{mode !== "splash" && mode !== "updating" && (
				<div
					style={{
						zoom: scale,
						height: "100%",
						display: "flex",
						alignItems: activeModeEdge === "top" ? "flex-start" : "center",
						justifyContent:
							activeModeEdge === "left"
								? "flex-start"
								: activeModeEdge === "right"
									? "flex-end"
									: "center",
						width: "100%",
						position: "relative"
					}}
					onMouseEnter={() => {
						// While hovering over volume/brightness sliders, cancel auto-hide
						if (timeoutRef.current) {
							clearTimeout(timeoutRef.current);
							timeoutRef.current = null;
						}
					}}
					onMouseLeave={() => {
						// When unhovering, start 2.5s auto-hide countdown
						if (!mixerExpandedRef.current && !splashActiveRef.current) {
							if (timeoutRef.current) clearTimeout(timeoutRef.current);
							timeoutRef.current = setTimeout(() => setMode("idle"), 2500);
						}
					}}
				>
					<AnimatePresence
						mode="wait"
						onExitComplete={() => {
							if (mixerExpandedRef.current) collapseMixer();
						}}
					>
						{mode === "volume" && (
							<VolumeNotch
								volume={volume}
								isMuted={isMuted}
								onVolumeChange={handleVolumeChange}
								mixerExpanded={mixerExpanded}
								onMixerExpandedChange={handleMixerExpandedChange}
								freeDragEnabled={freeDragEnabled}
								scale={scale}
								position={activeVolumePosition}
								onPositionChange={handleVolumePositionChange}
								onDragStart={handleDragStart}
								onDragEnd={handleVolumeDragEnd}
								key="volume-island"
							/>
						)}
						{mode === "brightness" && (
							<BrightnessNotch
								brightness={brightness}
								onBrightnessChange={handleBrightnessChange}
								freeDragEnabled={freeDragEnabled}
								scale={scale}
								position={activeBrightnessPosition}
								onPositionChange={handleBrightnessPositionChange}
								onDragStart={handleDragStart}
								onDragEnd={handleBrightnessDragEnd}
								key="brightness-island"
							/>
						)}
					</AnimatePresence>
				</div>
			)}
		</div>
	);
}

createRoot(document.getElementById("root")!).render(
	<StrictMode>
		<OverlayApp />
	</StrictMode>
);
