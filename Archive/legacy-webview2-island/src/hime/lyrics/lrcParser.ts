import type { LyricLine, ParsedLyrics } from "./types";

const TIMESTAMP_REGEX = /\[(\d{1,3}):(\d{2})(?:\.(\d{1,3}))?\]/g;
const OFFSET_REGEX = /\[offset:\s*([+-]?\d+)\s*\]/i;

/**
 * Parses raw LRC string into a chronologically sorted array of LyricLine.
 * Handles centisecond [mm:ss.xx], millisecond [mm:ss.xxx], multi-timestamp lines,
 * and global [offset:+/-ms] tags.
 */
export function parseLrc(input: string): ParsedLyrics {
    let offsetMs = 0;
    const rawLines: { ms: number; text: string }[] = [];
    let hasSync = false;
    let isInstrumental = false;

    if (!input || typeof input !== "string") {
        return { lines: [], offsetMs: 0, isInstrumental: false, hasSync: false };
    }

    const lowerInput = input.toLowerCase();
    if (lowerInput.includes("[instrumental]") || lowerInput.includes("instrumental.")) {
        isInstrumental = true;
    }

    const lines = input.split(/\r?\n/);
    for (const line of lines) {
        const trimmed = line.trim();
        if (!trimmed) continue;

        // Check offset tag
        const offsetMatch = trimmed.match(OFFSET_REGEX);
        if (offsetMatch && offsetMatch[1]) {
            const parsed = parseInt(offsetMatch[1], 10);
            if (!Number.isNaN(parsed)) {
                offsetMs = parsed;
            }
            continue;
        }

        // Parse all timestamp tags in the line
        const timestamps: number[] = [];
        let match: RegExpExecArray | null;
        let lastIndex = 0;

        TIMESTAMP_REGEX.lastIndex = 0;
        while ((match = TIMESTAMP_REGEX.exec(trimmed)) !== null) {
            const minutes = parseInt(match[1], 10) || 0;
            const seconds = parseInt(match[2], 10) || 0;
            const subsecStr = match[3] || "0";

            let millis = 0;
            if (subsecStr.length === 1) {
                millis = parseInt(subsecStr, 10) * 100;
            } else if (subsecStr.length === 2) {
                millis = parseInt(subsecStr, 10) * 10;
            } else {
                millis = parseInt(subsecStr.slice(0, 3), 10) || 0;
            }

            const lineMs = (minutes * 60 + seconds) * 1000 + millis;
            timestamps.push(lineMs);
            lastIndex = match.index + match[0].length;
        }

        if (timestamps.length > 0) {
            hasSync = true;
            const text = trimmed.slice(lastIndex).trim();
            for (const ts of timestamps) {
                rawLines.push({ ms: ts, text });
            }
        }
    }

    // Apply offset and clamp to 0
    const finalLines: LyricLine[] = rawLines.map((item) => ({
        ms: Math.max(0, item.ms + offsetMs),
        text: item.text,
    }));

    // Sort ascending by ms
    finalLines.sort((a, b) => a.ms - b.ms);

    return {
        lines: finalLines,
        offsetMs,
        isInstrumental,
        hasSync,
        plainText: !hasSync ? input : undefined,
    };
}

/**
 * Performs an O(log N) binary search over sorted lyric lines to find the active index.
 * Returns -1 if currentMs is before the first line.
 */
export function findActiveLyricIndex(lines: LyricLine[], currentMs: number): number {
    if (!lines || lines.length === 0) return -1;
    if (currentMs < lines[0].ms) return -1;
    if (currentMs >= lines[lines.length - 1].ms) return lines.length - 1;

    let low = 0;
    let high = lines.length - 1;
    let best = 0;

    while (low <= high) {
        const mid = (low + high) >> 1;
        if (lines[mid].ms <= currentMs) {
            best = mid;
            low = mid + 1; // search right for latest valid line
        } else {
            high = mid - 1;
        }
    }

    return best;
}

/**
 * Calculates current monotonic playback position taking into account GSMTC position update time
 * and user offset adjustments.
 */
export function calculateEffectivePosition(
    positionMs: number,
    positionUpdatedAt: number,
    isPlaying: boolean,
    userOffsetMs: number = 0,
): number {
    const elapsed = isPlaying && positionUpdatedAt > 0 ? Math.max(0, Date.now() - positionUpdatedAt) : 0;
    return Math.max(0, positionMs + elapsed + userOffsetMs);
}

/**
 * Calculates milliseconds delay until the onset of the next lyric line for zero-polling scheduling.
 * Returns null if currently on the final line or no lines exist.
 */
export function calculateNextLineDelay(
    lines: LyricLine[],
    currentIndex: number,
    currentMs: number,
): number | null {
    if (!lines || lines.length === 0) return null;
    const nextIndex = currentIndex + 1;
    if (nextIndex >= lines.length) return null;

    const nextOnset = lines[nextIndex].ms;
    const diff = nextOnset - currentMs;
    return Math.max(16, diff);
}
