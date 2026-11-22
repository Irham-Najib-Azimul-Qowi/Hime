import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
    calculateEffectivePosition,
    calculateNextLineDelay,
    findActiveLyricIndex,
    parseLrc,
} from "./lrcParser";
import type { LyricsResponse, LyricStatus } from "./types";

export interface UseLyricsProps {
    title: string;
    artist: string;
    album?: string;
    isPlaying: boolean;
    hasMedia: boolean;
    positionMs: number;
    durationMs: number;
    positionUpdatedAt: number;
    enabled?: boolean;
    allowOnline?: boolean;
    showInPill?: boolean;
}

export function useLyrics({
    title,
    artist,
    album,
    isPlaying,
    hasMedia,
    positionMs,
    durationMs,
    positionUpdatedAt,
    enabled = true,
    allowOnline = true,
    showInPill = true,
}: UseLyricsProps) {
    const [status, setStatus] = useState<LyricStatus>("idle");
    const [lyrics, setLyrics] = useState<LyricsResponse | null>(null);
    const [activeLineIndex, setActiveLineIndex] = useState<number>(-1);
    const [activeLineText, setActiveLineText] = useState<string>("");
    const [userOffsetMs, setUserOffsetMs] = useState<number>(0);

    const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
    const lastTrackRef = useRef<string>("");
    const isPlayingRef = useRef<boolean>(isPlaying);
    isPlayingRef.current = isPlaying;

    const clearScheduledTimer = useCallback(() => {
        if (timerRef.current !== null) {
            clearTimeout(timerRef.current);
            timerRef.current = null;
        }
    }, []);

    // Broadcast active lyric line to NCM center slot
    const emitToNcm = useCallback((text: string) => {
        if (typeof window === "undefined") return;
        if (!text) {
            window.dispatchEvent(
                new CustomEvent("hime:ncm-retract", { detail: "music-lyrics" }),
            );
            return;
        }
        window.dispatchEvent(
            new CustomEvent("hime:ncm-submit", {
                detail: {
                    id: "music-lyrics",
                    module: "music",
                    kind: "ambient",
                    priority: 5, // PriorityLevel.P5_AMBIENT
                    payload: { text: text.slice(0, 36) },
                },
            }),
        );
    }, []);

    const retractFromNcm = useCallback(() => {
        if (typeof window === "undefined") return;
        window.dispatchEvent(
            new CustomEvent("hime:ncm-retract", { detail: "music-lyrics" }),
        );
    }, []);

    // 1. Fetch lyrics when song metadata changes
    useEffect(() => {
        if (!enabled || !hasMedia || !title.trim()) {
            clearScheduledTimer();
            setLyrics(null);
            setStatus("idle");
            setActiveLineIndex(-1);
            setActiveLineText("");
            retractFromNcm();
            lastTrackRef.current = "";
            return;
        }

        const trackKey = `${artist}:${title}`;
        if (trackKey === lastTrackRef.current && lyrics) {
            return;
        }
        lastTrackRef.current = trackKey;

        let cancelled = false;
        setStatus("loading");
        clearScheduledTimer();

        const durationSec = durationMs > 0 ? Math.round(durationMs / 1000) : 0;

        const fetchLyricsData = async () => {
            try {
                const res = await invoke<LyricsResponse>("hime_lyrics_get", {
                    title,
                    artist,
                    album: album || null,
                    durationSec: durationSec > 0 ? durationSec : null,
                    allowOnline,
                });

                if (cancelled) return;

                if (!res || res.source === "not_found") {
                    setLyrics(null);
                    setStatus("not_found");
                    setActiveLineIndex(-1);
                    setActiveLineText("");
                    retractFromNcm();
                    return;
                }

                // If lines are not pre-parsed or empty, parse syncedLyrics if present
                if ((!res.lines || res.lines.length === 0) && res.synced_lyrics) {
                    const parsed = parseLrc(res.synced_lyrics);
                    res.lines = parsed.lines;
                    res.is_synced = parsed.hasSync;
                    res.is_instrumental = res.is_instrumental || parsed.isInstrumental;
                }

                setLyrics(res);
                setUserOffsetMs(res.user_offset_ms || 0);

                if (res.is_instrumental) {
                    setStatus("instrumental");
                    setActiveLineIndex(-1);
                    setActiveLineText("Instrumental.");
                    if (showInPill) emitToNcm("Instrumental.");
                } else if (res.lines && res.lines.length > 0) {
                    setStatus("synced");
                } else if (res.plain_lyrics) {
                    setStatus("unsynced");
                    setActiveLineIndex(-1);
                    setActiveLineText("");
                } else {
                    setStatus("not_found");
                    retractFromNcm();
                }
            } catch {
                if (cancelled) return;
                setLyrics(null);
                setStatus("not_found");
                retractFromNcm();
            }
        };

        fetchLyricsData();

        return () => {
            cancelled = true;
        };
    }, [
        title,
        artist,
        album,
        hasMedia,
        durationMs,
        enabled,
        allowOnline,
        showInPill,
        clearScheduledTimer,
        emitToNcm,
        retractFromNcm,
    ]);

    // 2. Real-time synchronization loop with zero-polling one-shot timer
    const scheduleNextTick = useCallback(
        (targetPos: number) => {
            clearScheduledTimer();
            if (!lyrics || !lyrics.lines || lyrics.lines.length === 0 || !isPlayingRef.current) {
                return;
            }

            const activeIdx = findActiveLyricIndex(lyrics.lines, targetPos);
            setActiveLineIndex(activeIdx);

            const activeText = activeIdx >= 0 ? lyrics.lines[activeIdx].text : "";
            setActiveLineText(activeText);

            if (showInPill) {
                emitToNcm(activeText);
            }

            const delay = calculateNextLineDelay(lyrics.lines, activeIdx, targetPos);
            if (delay !== null) {
                timerRef.current = setTimeout(() => {
                    if (!isPlayingRef.current) return;
                    const nextEffective = calculateEffectivePosition(
                        positionMs,
                        positionUpdatedAt,
                        true,
                        userOffsetMs,
                    );
                    scheduleNextTick(nextEffective);
                }, delay);
            }
        },
        [
            lyrics,
            positionMs,
            positionUpdatedAt,
            userOffsetMs,
            showInPill,
            clearScheduledTimer,
            emitToNcm,
        ],
    );

    // Trigger on seek, position update, play/pause state change, or offset change
    useEffect(() => {
        if (!lyrics || !lyrics.lines || lyrics.lines.length === 0) {
            clearScheduledTimer();
            return;
        }

        if (!isPlaying) {
            clearScheduledTimer();
            const pos = calculateEffectivePosition(positionMs, positionUpdatedAt, false, userOffsetMs);
            const activeIdx = findActiveLyricIndex(lyrics.lines, pos);
            setActiveLineIndex(activeIdx);
            const activeText = activeIdx >= 0 ? lyrics.lines[activeIdx].text : "";
            setActiveLineText(activeText);
            return;
        }

        const effectivePos = calculateEffectivePosition(
            positionMs,
            positionUpdatedAt,
            isPlaying,
            userOffsetMs,
        );
        scheduleNextTick(effectivePos);

        return () => {
            clearScheduledTimer();
        };
    }, [
        lyrics,
        isPlaying,
        positionMs,
        positionUpdatedAt,
        userOffsetMs,
        scheduleNextTick,
        clearScheduledTimer,
    ]);

    // Cleanup NCM on unmount
    useEffect(() => {
        return () => {
            clearScheduledTimer();
            retractFromNcm();
        };
    }, [clearScheduledTimer, retractFromNcm]);

    // 3. Offset Adjustment Actions
    const adjustOffset = useCallback(
        async (deltaMs: number) => {
            const newOffset = userOffsetMs + deltaMs;
            setUserOffsetMs(newOffset);
            if (lyrics?.cache_key) {
                try {
                    await invoke("hime_lyrics_set_offset", {
                        cacheKey: lyrics.cache_key,
                        offsetMs: newOffset,
                    });
                } catch {
                    // ignore failure
                }
            }
        },
        [userOffsetMs, lyrics],
    );

    const resetOffset = useCallback(async () => {
        setUserOffsetMs(0);
        if (lyrics?.cache_key) {
            try {
                await invoke("hime_lyrics_set_offset", {
                    cacheKey: lyrics.cache_key,
                    offsetMs: 0,
                });
            } catch {
                // ignore failure
            }
        }
    }, [lyrics]);

    const clearCache = useCallback(async () => {
        try {
            await invoke("hime_lyrics_clear_cache");
            setLyrics(null);
            setStatus("not_found");
            retractFromNcm();
        } catch {
            // ignore failure
        }
    }, [retractFromNcm]);

    return {
        status,
        lyrics,
        lines: lyrics?.lines || [],
        plainLyrics: lyrics?.plain_lyrics,
        isInstrumental: lyrics?.is_instrumental || status === "instrumental",
        isSynced: lyrics?.is_synced || status === "synced",
        activeLineIndex,
        activeLineText,
        userOffsetMs,
        adjustOffset,
        resetOffset,
        clearCache,
    };
}
