import React, { useEffect, useRef } from "react";
import { motion, AnimatePresence } from "framer-motion";
import type { LyricLine, LyricStatus } from "./types";

export interface LyricsViewProps {
    lines: LyricLine[];
    plainLyrics?: string;
    activeLineIndex: number;
    status: LyricStatus;
    userOffsetMs: number;
    onAdjustOffset: (deltaMs: number) => void;
    onResetOffset: () => void;
    onRetry?: () => void;
    onSeekLine?: (ms: number) => void;
}

export const LyricsView: React.FC<LyricsViewProps> = ({
    lines,
    plainLyrics,
    activeLineIndex,
    status,
    userOffsetMs,
    onAdjustOffset,
    onResetOffset,
    onRetry,
    onSeekLine,
}) => {
    const containerRef = useRef<HTMLDivElement>(null);
    const activeLineRef = useRef<HTMLDivElement>(null);
    const hasMountedScrollRef = useRef(false);

    // Snap immediately on first mount, then smooth auto-scroll centering the active line
    useEffect(() => {
        if (!containerRef.current || !activeLineRef.current) return;
        const container = containerRef.current;
        const line = activeLineRef.current;

        const targetScroll =
            line.offsetTop - container.clientHeight / 2 + line.clientHeight / 2;

        container.scrollTo({
            top: Math.max(0, targetScroll),
            behavior: hasMountedScrollRef.current ? "smooth" : "auto",
        });
        hasMountedScrollRef.current = true;
    }, [activeLineIndex]);

    if (status === "loading") {
        return (
            <div
                style={{
                    display: "flex",
                    flexDirection: "column",
                    alignItems: "center",
                    justifyContent: "center",
                    height: "180px",
                    gap: "8px",
                    color: "rgba(255, 255, 255, 0.72)",
                    fontSize: "13px",
                    textShadow: "0 1px 3px rgba(0, 0, 0, 0.85)",
                }}
            >
                <motion.div
                    animate={{ scale: [1, 1.15, 1], opacity: [0.6, 1, 0.6] }}
                    transition={{ repeat: Infinity, duration: 1.5, ease: "easeInOut" }}
                    style={{
                        width: "8px",
                        height: "8px",
                        borderRadius: "50%",
                        background: "#00D2FF",
                        boxShadow: "0 0 10px rgba(0, 210, 255, 0.8)",
                    }}
                />
                <span>Mencari lirik...</span>
            </div>
        );
    }

    if (status === "instrumental") {
        return (
            <div
                style={{
                    display: "flex",
                    flexDirection: "column",
                    alignItems: "center",
                    justifyContent: "center",
                    height: "180px",
                    gap: "6px",
                    color: "rgba(255, 255, 255, 0.72)",
                    fontSize: "13px",
                    textShadow: "0 1px 3px rgba(0, 0, 0, 0.85)",
                }}
            >
                <span style={{ fontSize: "20px" }}>🎵</span>
                <span style={{ fontWeight: 500 }}>Lagu instrumental</span>
            </div>
        );
    }

    if (status === "not_found" || (lines.length === 0 && !plainLyrics)) {
        return (
            <div
                style={{
                    display: "flex",
                    flexDirection: "column",
                    alignItems: "center",
                    justifyContent: "center",
                    height: "180px",
                    gap: "10px",
                    color: "rgba(255, 255, 255, 0.72)",
                    fontSize: "13px",
                    textShadow: "0 1px 3px rgba(0, 0, 0, 0.85)",
                }}
            >
                <span>Lirik tak ditemukan</span>
                {onRetry && (
                    <button
                        type="button"
                        onClick={onRetry}
                        style={{
                            padding: "4px 12px",
                            background: "rgba(0, 210, 255, 0.12)",
                            border: "1px solid rgba(0, 210, 255, 0.3)",
                            borderRadius: "12px",
                            color: "#FFFFFF",
                            fontSize: "12px",
                            cursor: "pointer",
                            transition: "all 0.2s ease",
                            boxShadow: "0 0 8px rgba(0, 210, 255, 0.15)",
                        }}
                    >
                        Coba Lagi
                    </button>
                )}
            </div>
        );
    }

    if (status === "unsynced" && plainLyrics) {
        return (
            <div
                style={{
                    display: "flex",
                    flexDirection: "column",
                    height: "220px",
                    width: "100%",
                }}
            >
                <div
                    style={{
                        flex: 1,
                        overflowY: "auto",
                        overflowX: "hidden",
                        scrollbarWidth: "none",
                        msOverflowStyle: "none",
                        padding: "10px 16px",
                        fontSize: "13px",
                        lineHeight: "1.6",
                        color: "rgba(255, 255, 255, 0.85)",
                        textShadow: "0 1px 3px rgba(0, 0, 0, 0.85)",
                        whiteSpace: "pre-wrap",
                        textAlign: "center",
                    }}
                >
                    {plainLyrics}
                </div>
            </div>
        );
    }

    return (
        <div
            style={{
                display: "flex",
                flexDirection: "column",
                height: "100%",
                minHeight: "160px",
                width: "100%",
                position: "relative",
            }}
        >
            {/* Scrollable Lyric Lines List */}
            <div
                ref={containerRef}
                style={{
                    flex: 1,
                    overflowY: "auto",
                    overflowX: "hidden",
                    scrollbarWidth: "none",
                    msOverflowStyle: "none",
                    padding: "36px 12px",
                    textAlign: "center",
                    scrollBehavior: "smooth",
                    display: "flex",
                    flexDirection: "column",
                    gap: "10px",
                    position: "relative",
                }}
            >
                <AnimatePresence>
                    {lines.map((line, index) => {
                        const isActive = index === activeLineIndex;
                        const isUpcoming = index === activeLineIndex + 1;

                        return (
                            <motion.div
                                key={`${index}-${line.ms}`}
                                ref={isActive ? activeLineRef : null}
                                onClick={() => onSeekLine && onSeekLine(line.ms)}
                                initial={{ opacity: 0.3, y: 4 }}
                                animate={{
                                    opacity: isActive ? 1 : isUpcoming ? 0.88 : 0.72,
                                    scale: isActive ? 1.05 : isUpcoming ? 1.0 : 0.98,
                                    color: isActive
                                        ? "#FFFFFF"
                                        : isUpcoming
                                        ? "rgba(255, 255, 255, 0.88)"
                                        : "rgba(255, 255, 255, 0.72)",
                                }}
                                transition={{
                                    type: "spring",
                                    stiffness: 450,
                                    damping: 32,
                                }}
                                style={{
                                    fontSize: isActive ? "16px" : isUpcoming ? "13.5px" : "13px",
                                    fontWeight: isActive ? 700 : isUpcoming ? 500 : 400,
                                    cursor: onSeekLine ? "pointer" : "default",
                                    padding: "5px 12px",
                                    borderRadius: "8px",
                                    textShadow: isActive
                                        ? "0 0 12px #00D2FF, 0 0 24px #38D6F5, 0 1px 3px rgba(0, 0, 0, 0.9)"
                                        : "0 1px 3px rgba(0, 0, 0, 0.85)",
                                    background: isActive
                                        ? "rgba(0, 210, 255, 0.12)"
                                        : "transparent",
                                    border: isActive
                                        ? "1px solid rgba(0, 210, 255, 0.35)"
                                        : "1px solid transparent",
                                    boxShadow: isActive
                                        ? "0 0 18px rgba(0, 210, 255, 0.2)"
                                        : "none",
                                }}
                            >
                                {line.text || "♪ ♪ ♪"}
                            </motion.div>
                        );
                    })}
                </AnimatePresence>
            </div>

            {/* Bottom Offset Controls Toolbar */}
            <div
                style={{
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "space-between",
                    padding: "6px 12px",
                    background: "rgba(0, 0, 0, 0.55)",
                    backdropFilter: "blur(10px)",
                    borderTop: "1px solid rgba(255, 255, 255, 0.08)",
                    fontSize: "11px",
                    color: "rgba(255, 255, 255, 0.72)",
                    borderRadius: "0 0 12px 12px",
                }}
            >
                <div style={{ display: "flex", alignItems: "center", gap: "6px" }}>
                    <span>Offset:</span>
                    <span
                        style={{
                            color: userOffsetMs !== 0 ? "#00D2FF" : "inherit",
                            fontFamily: "monospace",
                            fontWeight: userOffsetMs !== 0 ? 600 : 400,
                            textShadow:
                                userOffsetMs !== 0 ? "0 0 8px rgba(0, 210, 255, 0.4)" : "none",
                        }}
                    >
                        {userOffsetMs > 0 ? `+${userOffsetMs}ms` : `${userOffsetMs}ms`}
                    </span>
                </div>

                <div style={{ display: "flex", alignItems: "center", gap: "3px", flexWrap: "wrap", justifyContent: "flex-end" }}>
                    <button
                        type="button"
                        onClick={() => onAdjustOffset(-500)}
                        title="Mundurkan 500ms"
                        style={{
                            padding: "2px 6px",
                            background: "rgba(255, 255, 255, 0.06)",
                            border: "1px solid rgba(255, 255, 255, 0.12)",
                            borderRadius: "6px",
                            color: "#F4F6FA",
                            fontSize: "10.5px",
                            cursor: "pointer",
                            transition: "all 0.15s ease",
                        }}
                    >
                        -0.5s
                    </button>
                    <button
                        type="button"
                        onClick={() => onAdjustOffset(-200)}
                        title="Mundurkan 200ms"
                        style={{
                            padding: "2px 6px",
                            background: "rgba(255, 255, 255, 0.06)",
                            border: "1px solid rgba(255, 255, 255, 0.12)",
                            borderRadius: "6px",
                            color: "#F4F6FA",
                            fontSize: "10.5px",
                            cursor: "pointer",
                            transition: "all 0.15s ease",
                        }}
                    >
                        -0.2s
                    </button>
                    <button
                        type="button"
                        onClick={() => onAdjustOffset(-50)}
                        title="Mundurkan presisi 50ms"
                        style={{
                            padding: "2px 6px",
                            background: "rgba(0, 210, 255, 0.08)",
                            border: "1px solid rgba(0, 210, 255, 0.25)",
                            borderRadius: "6px",
                            color: "#00D2FF",
                            fontSize: "10px",
                            fontWeight: 600,
                            cursor: "pointer",
                            transition: "all 0.15s ease",
                        }}
                    >
                        -50ms
                    </button>
                    <button
                        type="button"
                        onClick={() => onAdjustOffset(50)}
                        title="Majukan presisi 50ms"
                        style={{
                            padding: "2px 6px",
                            background: "rgba(0, 210, 255, 0.08)",
                            border: "1px solid rgba(0, 210, 255, 0.25)",
                            borderRadius: "6px",
                            color: "#00D2FF",
                            fontSize: "10px",
                            fontWeight: 600,
                            cursor: "pointer",
                            transition: "all 0.15s ease",
                        }}
                    >
                        +50ms
                    </button>
                    <button
                        type="button"
                        onClick={() => onAdjustOffset(200)}
                        title="Majukan 200ms"
                        style={{
                            padding: "2px 6px",
                            background: "rgba(255, 255, 255, 0.06)",
                            border: "1px solid rgba(255, 255, 255, 0.12)",
                            borderRadius: "6px",
                            color: "#F4F6FA",
                            fontSize: "10.5px",
                            cursor: "pointer",
                            transition: "all 0.15s ease",
                        }}
                    >
                        +0.2s
                    </button>
                    <button
                        type="button"
                        onClick={() => onAdjustOffset(500)}
                        title="Majukan 500ms"
                        style={{
                            padding: "2px 6px",
                            background: "rgba(255, 255, 255, 0.06)",
                            border: "1px solid rgba(255, 255, 255, 0.12)",
                            borderRadius: "6px",
                            color: "#F4F6FA",
                            fontSize: "10.5px",
                            cursor: "pointer",
                            transition: "all 0.15s ease",
                        }}
                    >
                        +0.5s
                    </button>
                    <button
                        type="button"
                        onClick={onResetOffset}
                        disabled={userOffsetMs === 0}
                        style={{
                            padding: "2px 8px",
                            background:
                                userOffsetMs !== 0
                                    ? "rgba(0, 210, 255, 0.12)"
                                    : "rgba(255, 255, 255, 0.04)",
                            border:
                                userOffsetMs !== 0
                                    ? "1px solid rgba(0, 210, 255, 0.3)"
                                    : "1px solid rgba(255, 255, 255, 0.08)",
                            borderRadius: "6px",
                            color: userOffsetMs !== 0 ? "#00D2FF" : "rgba(255, 255, 255, 0.35)",
                            fontSize: "10.5px",
                            fontWeight: 500,
                            cursor: userOffsetMs !== 0 ? "pointer" : "default",
                            transition: "all 0.15s ease",
                        }}
                    >
                        Reset
                    </button>
                </div>
            </div>
        </div>
    );
};
