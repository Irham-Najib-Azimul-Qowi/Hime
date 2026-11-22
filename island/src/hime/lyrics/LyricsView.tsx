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

    // Smooth spring auto-scroll centering the active line
    useEffect(() => {
        if (!containerRef.current || !activeLineRef.current) return;
        const container = containerRef.current;
        const line = activeLineRef.current;

        const targetScroll =
            line.offsetTop - container.clientHeight / 2 + line.clientHeight / 2;

        container.scrollTo({
            top: Math.max(0, targetScroll),
            behavior: "smooth",
        });
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
                    color: "var(--text-2, #A7AEBB)",
                    fontSize: "13px",
                }}
            >
                <motion.div
                    animate={{ scale: [1, 1.15, 1], opacity: [0.6, 1, 0.6] }}
                    transition={{ repeat: Infinity, duration: 1.5, ease: "easeInOut" }}
                    style={{
                        width: "8px",
                        height: "8px",
                        borderRadius: "50%",
                        background: "var(--accent-mint, #34E3A6)",
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
                    color: "var(--text-2, #A7AEBB)",
                    fontSize: "13px",
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
                    color: "var(--text-2, #A7AEBB)",
                    fontSize: "13px",
                }}
            >
                <span>Lirik tak ditemukan</span>
                {onRetry && (
                    <button
                        type="button"
                        onClick={onRetry}
                        style={{
                            padding: "4px 12px",
                            background: "rgba(255, 255, 255, 0.08)",
                            border: "1px solid rgba(255, 255, 255, 0.15)",
                            borderRadius: "12px",
                            color: "var(--text-1, #F4F6FA)",
                            fontSize: "12px",
                            cursor: "pointer",
                            transition: "all 0.2s ease",
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
                        padding: "10px 16px",
                        fontSize: "13px",
                        lineHeight: "1.6",
                        color: "var(--text-1, #F4F6FA)",
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
                height: "240px",
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
                    padding: "60px 12px",
                    textAlign: "center",
                    scrollBehavior: "smooth",
                    display: "flex",
                    flexDirection: "column",
                    gap: "12px",
                }}
            >
                <AnimatePresence>
                    {lines.map((line, index) => {
                        const isActive = index === activeLineIndex;
                        const isPast = index < activeLineIndex;

                        return (
                            <motion.div
                                key={`${index}-${line.ms}`}
                                ref={isActive ? activeLineRef : null}
                                onClick={() => onSeekLine && onSeekLine(line.ms)}
                                initial={{ opacity: 0.3, y: 4 }}
                                animate={{
                                    opacity: isActive ? 1 : isPast ? 0.35 : 0.45,
                                    scale: isActive ? 1.04 : 0.98,
                                    color: isActive
                                        ? "var(--text-1, #F4F6FA)"
                                        : "var(--text-2, #A7AEBB)",
                                }}
                                transition={{
                                    type: "spring",
                                    stiffness: 450,
                                    damping: 32,
                                }}
                                style={{
                                    fontSize: isActive ? "15px" : "13px",
                                    fontWeight: isActive ? 600 : 400,
                                    cursor: onSeekLine ? "pointer" : "default",
                                    padding: "3px 8px",
                                    borderRadius: "6px",
                                    transition: "color 0.2s ease",
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
                    background: "rgba(0, 0, 0, 0.4)",
                    backdropFilter: "blur(8px)",
                    borderTop: "1px solid rgba(255, 255, 255, 0.08)",
                    fontSize: "11px",
                    color: "var(--text-2, #A7AEBB)",
                    borderRadius: "0 0 12px 12px",
                }}
            >
                <div style={{ display: "flex", alignItems: "center", gap: "6px" }}>
                    <span>Offset:</span>
                    <span
                        style={{
                            color:
                                userOffsetMs !== 0
                                    ? "var(--accent-mint, #34E3A6)"
                                    : "inherit",
                            fontFamily: "monospace",
                        }}
                    >
                        {userOffsetMs > 0 ? `+${userOffsetMs}ms` : `${userOffsetMs}ms`}
                    </span>
                </div>

                <div style={{ display: "flex", alignItems: "center", gap: "6px" }}>
                    <button
                        type="button"
                        onClick={() => onAdjustOffset(-500)}
                        style={{
                            padding: "2px 8px",
                            background: "rgba(255, 255, 255, 0.06)",
                            border: "1px solid rgba(255, 255, 255, 0.12)",
                            borderRadius: "8px",
                            color: "var(--text-1, #F4F6FA)",
                            fontSize: "11px",
                            cursor: "pointer",
                        }}
                    >
                        -0.5s
                    </button>
                    <button
                        type="button"
                        onClick={() => onAdjustOffset(500)}
                        style={{
                            padding: "2px 8px",
                            background: "rgba(255, 255, 255, 0.06)",
                            border: "1px solid rgba(255, 255, 255, 0.12)",
                            borderRadius: "8px",
                            color: "var(--text-1, #F4F6FA)",
                            fontSize: "11px",
                            cursor: "pointer",
                        }}
                    >
                        +0.5s
                    </button>
                    {userOffsetMs !== 0 && (
                        <button
                            type="button"
                            onClick={onResetOffset}
                            style={{
                                padding: "2px 8px",
                                background: "rgba(255, 255, 255, 0.06)",
                                border: "1px solid rgba(255, 255, 255, 0.12)",
                                borderRadius: "8px",
                                color: "var(--text-1, #F4F6FA)",
                                fontSize: "11px",
                                cursor: "pointer",
                            }}
                        >
                            Reset
                        </button>
                    )}
                </div>
            </div>
        </div>
    );
};
