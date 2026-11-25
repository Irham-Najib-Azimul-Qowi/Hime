import { useState, useEffect, useCallback, useRef } from "react";
import { motion, AnimatePresence, useAnimation } from "framer-motion";
import { invoke } from "@tauri-apps/api/core";
import { Visualizer } from "./App";
import { useLyrics, LyricsView } from "./hime/lyrics";
import {
	PlayIcon,
	PauseIcon,
	SkipBackIcon,
	SkipForwardIcon,
	MusicNoteIcon,
	SpeakerIcon,
	HeadphonesIcon,
	VolumeLowIcon,
	VolumeHighIcon
} from "./icons";

interface MediaInfo {
	title: string;
	artist: string;
	album?: string;
	is_playing: boolean;
	has_media: boolean;
	artwork?: string[];
	position_ms?: number;
	duration_ms?: number;
	seek_enabled?: boolean;
	position_updated_at?: number;
}

interface CompactMediaPlayerProps {
	mediaInfo: MediaInfo;
	albumArtUrl: string | null;
	albumArtKey: number;
	isPlaying: boolean;
	volume: number;
	volumeExpanded: boolean;
	onVolumeExpandedChange: (expanded: boolean) => void;
	onTogglePlayPause: () => void;
	onVolumeChange: (vol: number) => void;
	prevFront: ReturnType<typeof useAnimation>;
	prevBack: ReturnType<typeof useAnimation>;
	nextFront: ReturnType<typeof useAnimation>;
	nextBack: ReturnType<typeof useAnimation>;
	onAnimatePrev: () => void;
	onAnimateNext: () => void;
	onLayoutChange: (layout: "classic" | "compact") => void;
	lyricsHook?: ReturnType<typeof useLyrics>;
	lyricsExpanded?: boolean;
	onLyricsExpandedChange?: (expanded: boolean) => void;
}

const MARQUEE_SPEED = 30;
const MARQUEE_MIN_DURATION = 5;

function TitleMarquee({ title }: { title: string }) {
	const textRef = useRef<HTMLSpanElement>(null);
	const containerRef = useRef<HTMLDivElement>(null);
	const [isOverflowing, setIsOverflowing] = useState(false);
	const [scrollDistance, setScrollDistance] = useState(0);
	const [scrollDuration, setScrollDuration] = useState(8);

	useEffect(() => {
		const check = () => {
			if (textRef.current && containerRef.current) {
				const textWidth = textRef.current.scrollWidth;
				const containerWidth = containerRef.current.clientWidth;
				const overflow = textWidth > containerWidth;
				setIsOverflowing(overflow);
				if (overflow) {
					const distance = textWidth - containerWidth;
					setScrollDistance(distance);
					setScrollDuration(Math.max(distance / MARQUEE_SPEED, MARQUEE_MIN_DURATION));
				}
			}
		};
		check();
		const observer = new ResizeObserver(check);
		if (containerRef.current) observer.observe(containerRef.current);
		return () => observer.disconnect();
	}, [title]);

	return (
		<div ref={containerRef} className="cmp-marquee-wrap">
			<span
				ref={textRef}
				className={`cmp-title ${isOverflowing ? "cmp-marquee" : ""}`}
				style={
					{
						"--scroll-distance": isOverflowing ? `-${scrollDistance}px` : undefined,
						"--scroll-duration": `${scrollDuration}s`
					} as React.CSSProperties
				}
			>
				{title}
			</span>
		</div>
	);
}

function formatTime(ms: number): string {
	if (ms <= 0) return "0:00";
	const totalSeconds = Math.floor(ms / 1000);
	const minutes = Math.floor(totalSeconds / 60);
	const seconds = totalSeconds % 60;
	return `${minutes}:${seconds.toString().padStart(2, "0")}`;
}

function formatRemaining(elapsed: number, duration: number): string {
	const remaining = Math.max(0, duration - elapsed);
	const totalSeconds = Math.floor(remaining / 1000);
	const minutes = Math.floor(totalSeconds / 60);
	const seconds = totalSeconds % 60;
	return `-${minutes}:${seconds.toString().padStart(2, "0")}`;
}

export function CompactMediaPlayer({
	mediaInfo,
	albumArtUrl,
	albumArtKey,
	isPlaying,
	volume,
	volumeExpanded,
	onVolumeExpandedChange,
	onTogglePlayPause,
	onVolumeChange,
	prevFront,
	prevBack,
	nextFront,
	nextBack,
	onAnimatePrev,
	onAnimateNext,
	onLayoutChange,
	lyricsHook: passedLyricsHook,
	lyricsExpanded,
	onLyricsExpandedChange
}: CompactMediaPlayerProps) {
	const seekRef = useRef<HTMLInputElement>(null);

	const positionMs = mediaInfo.position_ms ?? 0;
	const durationMs = mediaInfo.duration_ms ?? 0;
	const seekEnabled = mediaInfo.seek_enabled ?? false;
	const positionUpdatedAt = mediaInfo.position_updated_at ?? 0;

	// Single source of truth: last known backend position + when it was received
	const backendPosRef = useRef(positionMs);
	const backendTimeRef = useRef(positionUpdatedAt > 0 ? positionUpdatedAt : Date.now());
	const isSeekingRef = useRef(false);
	const lastTrackKeyRef = useRef(`${mediaInfo.artist}:${mediaInfo.title}`);

	// displayPosition is the live, extrapolated position
	const [displayPosition, setDisplayPosition] = useState(() => {
		const updatedAt = positionUpdatedAt > 0 ? positionUpdatedAt : Date.now();
		const elapsed = isPlaying ? Math.max(0, Date.now() - updatedAt) : 0;
		return Math.max(0, Math.min(positionMs + elapsed, durationMs || Infinity));
	});

	// Update refs when backend sends new position (ignore during seek, avoid jitter on 1s polls)
	useEffect(() => {
		if (isSeekingRef.current) return;
		const now = Date.now();
		const updatedAt = positionUpdatedAt > 0 ? positionUpdatedAt : now;
		const incomingLivePos = Math.max(
			0,
			Math.min(positionMs + (isPlaying ? Math.max(0, now - updatedAt) : 0), durationMs || Infinity)
		);
		const trackKey = `${mediaInfo.artist}:${mediaInfo.title}`;
		const trackChanged = trackKey !== lastTrackKeyRef.current;
		lastTrackKeyRef.current = trackKey;

		const currentLivePos = Math.max(
			0,
			Math.min(
				backendPosRef.current + (isPlaying ? Math.max(0, now - backendTimeRef.current) : 0),
				durationMs || Infinity
			)
		);
		const drift = incomingLivePos - currentLivePos;

		if (trackChanged || !isPlaying || Math.abs(drift) > 850 || drift > 250) {
			backendPosRef.current = positionMs;
			backendTimeRef.current = updatedAt;
			setDisplayPosition(incomingLivePos);
		}
	}, [positionMs, positionUpdatedAt, isPlaying, durationMs, mediaInfo.artist, mediaInfo.title]);

	const [localShowLyrics, setLocalShowLyrics] = useState(false);
	const showLyrics = lyricsExpanded !== undefined ? lyricsExpanded : localShowLyrics;
	const setShowLyrics = useCallback((updater: boolean | ((prev: boolean) => boolean)) => {
		const nextVal = typeof updater === "function" ? updater(showLyrics) : updater;
		if (onLyricsExpandedChange) {
			onLyricsExpandedChange(nextVal);
		}
		setLocalShowLyrics(nextVal);
	}, [showLyrics, onLyricsExpandedChange]);

	// Automatic Lyrics Fetch: instant online fetch, never prompts for user permission
	const internalLyricsHook = useLyrics({
		title: mediaInfo.title,
		artist: mediaInfo.artist,
		album: mediaInfo.album,
		isPlaying,
		hasMedia: mediaInfo.has_media,
		positionMs: displayPosition,
		durationMs,
		positionUpdatedAt: Date.now(),
		enabled: true,
		allowOnline: true,
		showInPill: true,
	});

	const lyricsHook = passedLyricsHook || internalLyricsHook;

	// Extract active line and upcoming line for dedicated stream containers
	const lines = lyricsHook.lines || [];
	const activeIdx = lyricsHook.activeLineIndex;
	const hasSyncedLines = lines.length > 0;

	let currentActiveText = "";
	let upcomingText = "";

	if (hasSyncedLines) {
		if (activeIdx >= 0 && lines[activeIdx]) {
			currentActiveText = lines[activeIdx].text;
		} else if (activeIdx === -1) {
			// Pre-roll before first line
			currentActiveText = isPlaying ? "♪ ♪ ♪" : (mediaInfo.title ? `♪ ${mediaInfo.title}` : "");
		}
		if (activeIdx + 1 < lines.length) {
			upcomingText = lines[activeIdx + 1]?.text || "";
		}
	} else if (lyricsHook.status === "instrumental") {
		currentActiveText = "🎵 Lagu instrumental";
	} else if (lyricsHook.status === "unsynced" && lyricsHook.plainLyrics) {
		currentActiveText = lyricsHook.plainLyrics.split("\n").find((l) => l.trim()) || "Lirik tersedia";
		upcomingText = "Klik untuk membuka lirik";
	} else if (lyricsHook.activeLineText) {
		currentActiveText = lyricsHook.activeLineText;
	}

	useEffect(() => {
		const handleKeyDown = (e: KeyboardEvent) => {
			if (e.ctrlKey && e.altKey && e.key.toLowerCase() === "l") {
				e.preventDefault();
				setShowLyrics((prev) => !prev);
			} else if (showLyrics && !e.ctrlKey && !e.altKey) {
				if (e.key === "[") {
					e.preventDefault();
					lyricsHook.adjustOffset(e.shiftKey ? -50 : -200);
				} else if (e.key === "]") {
					e.preventDefault();
					lyricsHook.adjustOffset(e.shiftKey ? 50 : 200);
				}
			}
		};
		window.addEventListener("keydown", handleKeyDown);
		return () => window.removeEventListener("keydown", handleKeyDown);
	}, [showLyrics, lyricsHook.adjustOffset, setShowLyrics]);

	// Tick: extrapolate position every 50ms for smooth progress & lyrics sync
	useEffect(() => {
		const tick = () => {
			if (isSeekingRef.current) return; // don't update during seek
			const elapsed = isPlaying ? Math.max(0, Date.now() - backendTimeRef.current) : 0;
			const pos = Math.max(0, Math.min(backendPosRef.current + elapsed, durationMs || Infinity));
			setDisplayPosition(pos);
		};
		tick();
		const interval = setInterval(tick, 50);
		return () => clearInterval(interval);
	}, [isPlaying, durationMs]);

	// Seek handlers
	const handleSeekStart = useCallback(() => {
		isSeekingRef.current = true;
	}, []);

	const handleSeekChange = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
		setDisplayPosition(Number(e.target.value));
	}, []);

	const handleSeekEnd = useCallback(() => {
		const seekPos = displayPosition;
		isSeekingRef.current = false;
		// Update backend refs to the seek position so extrapolation continues from there
		backendPosRef.current = seekPos;
		backendTimeRef.current = Date.now();
		invoke("media_seek", { positionMs: seekPos }).catch(() => {});
	}, [displayPosition]);

	const progressPercent = durationMs > 0 ? (displayPosition / durationMs) * 100 : 0;

	return (
		<div className="cmp">
			{showLyrics ? (
				/* EXPANDED MUSIC VIEW: Dedicated Lyrics Container & Stream */
				<div className="cmp-expanded-view">
					{/* Top Header Row */}
					<div className="cmp-row cmp-row-info cmp-expanded-header">
						<div className="cmp-track">
							<TitleMarquee title={mediaInfo.title} />
							<span className="cmp-artist">{mediaInfo.artist}</span>
						</div>
						<div className="cmp-visualizer">
							<Visualizer isPlaying={isPlaying} bars={4} height={16} />
						</div>
					</div>

					{/* Dedicated Lyrics Stream Banner (Karaoke Active + Subtle Upcoming) */}
					<div
						className="cmp-lyrics-stream cmp-lyrics-stream--expanded"
						onClick={(e) => e.stopPropagation()}
					>
						<AnimatePresence>
							<motion.div
								key={currentActiveText || `active-${activeIdx}`}
								initial={{ opacity: 0, y: 4 }}
								animate={{ opacity: 1, y: 0 }}
								exit={{ opacity: 0, y: -4 }}
								transition={{ duration: 0.12, ease: "easeOut" }}
								className="cmp-lyrics-active-line"
							>
								{currentActiveText ? (
									<span className="cmp-lyrics-karaoke-text">
										{currentActiveText}
									</span>
								) : lyricsHook.status === "loading" ? (
									<span className="cmp-lyrics-loading">
										<span className="cmp-lyrics-dot" /> Mencari lirik...
									</span>
								) : (
									<span className="cmp-lyrics-placeholder">
										♪ {mediaInfo.artist || "Hime Music"}
									</span>
								)}
							</motion.div>
						</AnimatePresence>

						{upcomingText ? (
							<AnimatePresence mode="wait">
								<motion.div
									key={upcomingText}
									initial={{ opacity: 0, y: 4 }}
									animate={{ opacity: 0.55, y: 0 }}
									exit={{ opacity: 0, y: -4 }}
									transition={{ duration: 0.25, ease: "easeOut" }}
									className="cmp-lyrics-upcoming-line"
								>
									{upcomingText}
								</motion.div>
							</AnimatePresence>
						) : null}
					</div>

					{/* Scrollable Lyrics Stream */}
					<div
						className="cmp-lyrics-container"
						style={{
							scrollbarWidth: "none",
							msOverflowStyle: "none",
							overflowX: "hidden",
							overflowY: "hidden",
						}}
					>
						<LyricsView
							lines={lyricsHook.lines}
							plainLyrics={lyricsHook.plainLyrics}
							activeLineIndex={lyricsHook.activeLineIndex}
							status={lyricsHook.status}
							userOffsetMs={lyricsHook.userOffsetMs}
							onAdjustOffset={lyricsHook.adjustOffset}
							onResetOffset={lyricsHook.resetOffset}
							onRetry={() => {}}
							onSeekLine={(ms) => {
								setDisplayPosition(ms);
								backendPosRef.current = ms;
								backendTimeRef.current = Date.now();
								invoke("media_seek", { positionMs: ms }).catch(() => {});
							}}
						/>
					</div>

					{/* Compact playback controls bar at bottom of expanded lyrics */}
					<div className="cmp-row cmp-row-controls cmp-expanded-controls">
						<motion.button
							className="cmp-btn cmp-btn-side"
							onClick={(e) => {
								e.stopPropagation();
								onVolumeExpandedChange(!volumeExpanded);
							}}
							whileTap={{ scale: 0.9 }}
							title="Volume"
						>
							<SpeakerIcon size={18} muted={volume === 0} />
						</motion.button>

						<div className="cmp-main-controls">
							<motion.button
								className="cmp-btn"
								onClick={(e) => {
									e.stopPropagation();
									onAnimatePrev();
								}}
								whileTap={{ scale: 0.9 }}
							>
								<SkipBackIcon size={24} />
							</motion.button>

							<motion.button
								className="cmp-btn cmp-play"
								onClick={(e) => {
									e.stopPropagation();
									onTogglePlayPause();
								}}
								whileTap={{ scale: 0.95 }}
							>
								{isPlaying ? <PauseIcon size={22} /> : <PlayIcon size={22} />}
							</motion.button>

							<motion.button
								className="cmp-btn"
								onClick={(e) => {
									e.stopPropagation();
									onAnimateNext();
								}}
								whileTap={{ scale: 0.9 }}
							>
								<SkipForwardIcon size={24} />
							</motion.button>
						</div>

						<span className="cmp-time" style={{ fontSize: "11px", minWidth: "40px", textAlign: "right" }}>
							{formatTime(displayPosition)}
						</span>
					</div>
				</div>
			) : (
				/* COMPACT MUSIC VIEW: Standard Player with Dedicated Lyrics Stream */
				<>
					{/* Row 1: Album art + Title/Artist + Visualizer & Lyrics Toggle */}
					<div className="cmp-row cmp-row-info">
						<div
							className="cmp-art"
							onClick={(e) => {
								e.stopPropagation();
								onLayoutChange("classic");
							}}
							style={{ cursor: "pointer" }}
						>
							<AnimatePresence mode="wait" initial={false}>
								{albumArtUrl ? (
									<motion.img
										key={`art-${albumArtKey}`}
										src={albumArtUrl}
										alt="Art"
										draggable={false}
										initial={{ rotateY: 90, opacity: 0 }}
										animate={{ rotateY: 0, opacity: 1 }}
										exit={{ rotateY: -90, opacity: 0 }}
										transition={{ duration: 0.3, ease: "easeInOut" }}
									/>
								) : (
									<motion.div
										key="placeholder"
										className="cmp-art-placeholder"
										initial={{ rotateY: 90, opacity: 0 }}
										animate={{ rotateY: 0, opacity: 1 }}
										exit={{ rotateY: -90, opacity: 0 }}
										transition={{ duration: 0.3, ease: "easeInOut" }}
									>
										<MusicNoteIcon size={22} />
									</motion.div>
								)}
							</AnimatePresence>
						</div>
						<div className="cmp-track">
							<TitleMarquee title={mediaInfo.title} />
							<span className="cmp-artist">{mediaInfo.artist}</span>
						</div>
						<div className="cmp-visualizer" style={{ display: "flex", alignItems: "center", gap: "6px" }}>
							<button
								type="button"
								className={`cmp-lyrics-toggle ${showLyrics ? "active" : ""}`}
								onClick={(e) => {
									e.stopPropagation();
									setShowLyrics(true);
								}}
								style={{
									padding: "2px 8px",
									borderRadius: "10px",
									fontSize: "11px",
									fontWeight: 600,
									background: "rgba(255, 255, 255, 0.08)",
									color: "var(--text-2, #A7AEBB)",
									border: "none",
									cursor: "pointer",
									transition: "all 0.2s ease",
								}}
								title="Buka lirik lengkap (Ctrl+Alt+L)"
							>
								Lirik
							</button>
							<Visualizer isPlaying={isPlaying} bars={5} height={16} />
						</div>
					</div>

					{/* Row 2: Dedicated Lyrics Stream Container (Active karaoke glow + upcoming subtle) */}
					<div
						className="cmp-lyrics-stream"
						onClick={(e) => {
							e.stopPropagation();
							setShowLyrics(true);
						}}
						title="Klik untuk membuka lirik lengkap (Ctrl+Alt+L)"
					>
						<AnimatePresence>
							<motion.div
								key={currentActiveText || `status-${lyricsHook.status}`}
								initial={{ opacity: 0, y: 4 }}
								animate={{ opacity: 1, y: 0 }}
								exit={{ opacity: 0, y: -4 }}
								transition={{ duration: 0.12, ease: "easeOut" }}
								className="cmp-lyrics-active-line"
							>
								{currentActiveText ? (
									<span className="cmp-lyrics-karaoke-text">
										{currentActiveText}
									</span>
								) : lyricsHook.status === "loading" ? (
									<span className="cmp-lyrics-loading">
										<span className="cmp-lyrics-dot" /> Mencari lirik...
									</span>
								) : lyricsHook.status === "instrumental" ? (
									<span className="cmp-lyrics-instrumental">
										🎵 Lagu instrumental
									</span>
								) : (
									<span className="cmp-lyrics-placeholder">
										♪ {mediaInfo.artist || "Hime Music"}
									</span>
								)}
							</motion.div>
						</AnimatePresence>

						{upcomingText ? (
							<AnimatePresence mode="wait">
								<motion.div
									key={upcomingText}
									initial={{ opacity: 0, y: 4 }}
									animate={{ opacity: 0.5, y: 0 }}
									exit={{ opacity: 0, y: -4 }}
									transition={{ duration: 0.24, ease: "easeOut" }}
									className="cmp-lyrics-upcoming-line"
								>
									{upcomingText}
								</motion.div>
							</AnimatePresence>
						) : null}
					</div>

					{/* Row 3: Progress Bar */}
					{durationMs > 0 && (
						<div className="cmp-row cmp-row-progress">
							<span className="cmp-time">{formatTime(displayPosition)}</span>
							<div className="cmp-track-bar">
								<div className="cmp-track-fill" style={{ width: `${progressPercent}%` }} />
								{seekEnabled && (
									<input
										ref={seekRef}
										type="range"
										className="cmp-seek"
										min={0}
										max={durationMs}
										step={100}
										value={displayPosition}
										onMouseDown={handleSeekStart}
										onTouchStart={handleSeekStart}
										onChange={handleSeekChange}
										onMouseUp={handleSeekEnd}
										onTouchEnd={handleSeekEnd}
										onClick={(e) => e.stopPropagation()}
									/>
								)}
							</div>
							<span className="cmp-time">{formatRemaining(displayPosition, durationMs)}</span>
						</div>
					)}

					{/* Row 4: Controls */}
					<div className="cmp-row cmp-row-controls">
						<motion.button
							className="cmp-btn cmp-btn-side"
							onClick={(e) => {
								e.stopPropagation();
								onVolumeExpandedChange(!volumeExpanded);
							}}
							whileTap={{ scale: 0.9 }}
							title="Volume"
						>
							<SpeakerIcon size={22} muted={volume === 0} />
						</motion.button>

						<div className="cmp-main-controls">
							<motion.button
								className="cmp-btn"
								onClick={(e) => {
									e.stopPropagation();
									onAnimatePrev();
								}}
								whileTap={{ scale: 0.9 }}
							>
								<div className="cmp-slide-wrap">
									<motion.div animate={prevBack} className="cmp-slide-layer">
										<SkipBackIcon size={30} />
									</motion.div>
									<motion.div animate={prevFront} className="cmp-slide-layer">
										<SkipBackIcon size={30} />
									</motion.div>
								</div>
							</motion.button>

							<motion.button
								className="cmp-btn cmp-play"
								onClick={(e) => {
									e.stopPropagation();
									onTogglePlayPause();
								}}
								whileTap={{ scale: 0.95 }}
							>
								<AnimatePresence mode="wait" initial={false}>
									<motion.div
										key={isPlaying ? "pause" : "play"}
										initial={{ opacity: 0, scale: 0.7 }}
										animate={{ opacity: 1, scale: 1 }}
										exit={{ opacity: 0, scale: 0.7 }}
										transition={{ duration: 0.12 }}
										style={{ display: "flex", alignItems: "center", justifyContent: "center" }}
									>
										{isPlaying ? <PauseIcon size={26} /> : <PlayIcon size={26} />}
									</motion.div>
								</AnimatePresence>
							</motion.button>

							<motion.button
								className="cmp-btn"
								onClick={(e) => {
									e.stopPropagation();
									onAnimateNext();
								}}
								whileTap={{ scale: 0.9 }}
							>
								<div className="cmp-slide-wrap">
									<motion.div animate={nextBack} className="cmp-slide-layer">
										<SkipForwardIcon size={30} />
									</motion.div>
									<motion.div animate={nextFront} className="cmp-slide-layer">
										<SkipForwardIcon size={30} />
									</motion.div>
								</div>
							</motion.button>
						</div>

						<motion.button
							className="cmp-btn cmp-btn-side"
							onClick={(e) => {
								e.stopPropagation();
								invoke("open_sound_settings").catch(() => {});
							}}
							whileTap={{ scale: 0.9 }}
							title="Audio Output"
						>
							<HeadphonesIcon size={22} />
						</motion.button>
					</div>

					{/* Volume expansion */}
					<AnimatePresence>
						{volumeExpanded && (
							<motion.div
								className="cmp-expand"
								initial={{ height: 0, opacity: 0 }}
								animate={{ height: 36, opacity: 1 }}
								exit={{ height: 0, opacity: 0 }}
								transition={{ type: "spring", stiffness: 500, damping: 30 }}
							>
								<div
									className="cmp-volume-row"
									onWheel={(e) => {
										e.stopPropagation();
										const delta = e.deltaY < 0 ? 0.04 : -0.04;
										const newVol = Math.max(0, Math.min(1, Math.round((volume + delta) * 100) / 100));
										onVolumeChange(newVol);
									}}
								>
									<VolumeLowIcon size={12} style={{ opacity: 0.5, flexShrink: 0 }} />
									<div
										className="cmp-volume-track"
										onWheel={(e) => {
											e.stopPropagation();
											const delta = e.deltaY < 0 ? 0.04 : -0.04;
											const newVol = Math.max(0, Math.min(1, Math.round((volume + delta) * 100) / 100));
											onVolumeChange(newVol);
										}}
									>
										<div className="cmp-volume-fill" style={{ width: `${volume * 100}%` }} />
										<input
											type="range"
											className="cmp-volume-slider"
											min={0}
											max={1}
											step={0.01}
											value={volume}
											onChange={(e) => onVolumeChange(parseFloat(e.target.value))}
											onClick={(e) => e.stopPropagation()}
										/>
									</div>
									<VolumeHighIcon size={14} style={{ opacity: 0.5, flexShrink: 0 }} />
								</div>
							</motion.div>
						)}
					</AnimatePresence>
				</>
			)}
		</div>
	);
}
