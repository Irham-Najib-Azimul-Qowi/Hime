export interface LyricLine {
    ms: number;
    text: string;
}

export interface ParsedLyrics {
    lines: LyricLine[];
    offsetMs: number;
    isInstrumental: boolean;
    hasSync: boolean;
    plainText?: string;
}

export interface LyricsResponse {
    cache_key: string;
    title: string;
    artist: string;
    album?: string;
    duration_sec: number;
    source: string;
    synced_lyrics?: string;
    plain_lyrics?: string;
    lines: LyricLine[];
    is_synced: boolean;
    is_instrumental: boolean;
    user_offset_ms: number;
}

export type LyricStatus =
    | "idle"
    | "loading"
    | "synced"
    | "unsynced"
    | "instrumental"
    | "not_found"
    | "error";

export interface LyricsState {
    status: LyricStatus;
    lyrics: LyricsResponse | null;
    activeLineIndex: number;
    activeLineText: string;
    userOffsetMs: number;
    error?: string;
}
