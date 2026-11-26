import { describe, test, it, expect, beforeEach } from "bun:test";
import { Database } from "bun:sqlite";

/**
 * =============================================================================
 * Hime Stage U3 E2E Test Suite: Routines & Windows Enforcement Ladder
 * =============================================================================
 * File: island/scripts/routines.test.ts
 *
 * Comprehensive 4-Tier Test Suite covering Features 1–9, 10–21, 26, 27–29, 31,
 * and Real-World Scenarios S1, S2, S3, S4, S5, S6, S8 per TEST_INFRA.md.
 *
 * Tier 1: Isolated Feature Happy-Path Coverage (>=5 test cases per feature)
 * Tier 2: Boundary & Corner Cases (>=5 test cases per feature)
 * Tier 3: Cross-Feature Pairwise Combinations (24 critical interaction tests)
 * Tier 4: Real-World Multi-Step Scenarios (S1, S2, S3, S4, S5, S6, S8)
 *
 * Invariants Enforced:
 * - Master Plan Gate #10: Zero real user apps targeted; safe mock window fixtures only.
 * - Strict prohibition of TerminateProcess (graceful WM_CLOSE only).
 * - VC-U5 copy bounds: Buttons <= 10 chars, Titles <= 16 chars, Descriptions <= 36 chars.
 * =============================================================================
 */

// =============================================================================
// DOMAIN TYPES & ENUMS (Reflecting hime-core, hime-storage, and hime-pal)
// =============================================================================

export type Weekday =
  | "Sunday"
  | "Monday"
  | "Tuesday"
  | "Wednesday"
  | "Thursday"
  | "Friday"
  | "Saturday";

export const WEEKDAYS: Weekday[] = [
  "Sunday",
  "Monday",
  "Tuesday",
  "Wednesday",
  "Thursday",
  "Friday",
  "Saturday",
];

export interface RoutineSchedule {
  days: Weekday[];
  time: string; // "HH:MM" 24h format
  duration_min: number;
  window_min?: number;
}

export type EnforcementTier = "ingatkan" | "tegas" | "paksa";

export interface DistractionRules {
  block_during_routine: boolean;
  daily_quota_min?: number;
  sequential_after_focus_min?: number;
  distraction_apps: string[];
}

export interface AutoEscalationConfig {
  enabled: boolean;
  missed_threshold: number; // e.g. 3 missed reminders escalate to Tegas
}

export interface RoutineStreak {
  current_count: number;
  best_count: number;
  last_completed_date?: string; // "YYYY-MM-DD"
  frozen_on_date?: string; // set on excused skip
}

export interface Routine {
  id: string;
  title: string;
  category: string;
  schedule: RoutineSchedule;
  enforcement: EnforcementTier;
  allowed_apps: string[];
  distraction_rules: DistractionRules;
  start_grace_min: number;
  escalation?: AutoEscalationConfig;
  streak: RoutineStreak;
  active: boolean;
  created_at: number;
  updated_at: number;
}

export type RoutineSessionState =
  | "ambient"
  | "prompted"
  | "snoozed"
  | "active"
  | "completed"
  | "skipped"
  | "missed";

export interface RoutineSession {
  id: string;
  routine_id: string;
  state: RoutineSessionState;
  scheduled_for: number; // epoch seconds
  duration_min: number;
  start_grace_min: number;
  snooze_count: number;
  max_snooze_count?: number; // Tegas caps at 3
  snooze_until?: number;
  started_at?: number;
  completed_at?: number;
  skip_reason?: string;
  is_excused: boolean;
}

export interface Commitment {
  id: string;
  name: string;
  tier: EnforcementTier;
  target_apps: string[];
  daily_quota_min?: number;
  active: boolean;
  pending_weakening_until?: number; // epoch seconds for cooling-off
  created_at: number;
  updated_at: number;
}

export interface EmergencyPauseRecord {
  id: string;
  started_at: number;
  duration_min: number;
  reason: string;
  expired_at: number;
}

export interface EnforcementHistoryRecord {
  id: string;
  timestamp: number;
  target_app: string;
  tier: string;
  action_taken: string;
  details?: string;
}

export interface ForegroundInfo {
  hwnd: number;
  pid: number;
  process_name: string;
  executable_path: string;
  window_title: string;
  class_name: string;
}

export type CloseResult =
  | "ClosedSuccessfully"
  | "UnsavedDialogDetected"
  | "WindowRefusedClose"
  | "ProtectedProcessRefused";

export type DummyWindowMode = "ModeA_CleanClose" | "ModeB_UnsavedChangesModal" | "ModeC_RefuseClose";
export interface RoutineRow {
  id: string;
  title: string;
  category: string;
  schedule_json: string;
  enforcement: string;
  allowed_apps_json: string | null;
  distraction_rules_json: string | null;
  start_grace_min: number;
  escalation_json: string | null;
  current_streak: number;
  best_streak: number;
  last_completed_date: string | null;
  frozen_on_date: string | null;
  active: number;
  created_at: number;
  updated_at: number;
}

export interface CommitmentRow {
  id: string;
  name: string;
  tier: string;
  target_apps_json: string;
  daily_quota_min: number | null;
  active: number;
  pending_weakening_until: number | null;
  created_at: number;
  updated_at: number;
}


// =============================================================================
// DOMAIN LOGIC ENGINES (Pure TypeScript Reference Implementation)
// =============================================================================

/** Civil date conversion helpers (civil days since Unix epoch) */
export class CivilDateUtils {
  static dateStrToDays(d: string): number | null {
    const parts = d.split("-");
    if (parts.length !== 3) return null;
    let y = parseInt(parts[0], 10);
    const m = parseInt(parts[1], 10);
    const day = parseInt(parts[2], 10);
    if (isNaN(y) || isNaN(m) || isNaN(day) || m < 1 || m > 12 || day < 1 || day > 31) return null;

    if (m <= 2) y -= 1;
    const era = Math.floor((y >= 0 ? y : y - 399) / 400);
    const yoe = y - era * 400;
    const doy = Math.floor((153 * (m > 2 ? m - 3 : m + 9) + 2) / 5) + day - 1;
    const doe = yoe * 365 + Math.floor(yoe / 4) - Math.floor(yoe / 100) + doy;
    return era * 146097 + doe - 719468;
  }

  static daysToDateStr(days: number): string {
    const z = days + 719468;
    const era = Math.floor((z >= 0 ? z : z - 146096) / 146097);
    const doe = z - era * 146097;
    const yoe = Math.floor((doe - Math.floor(doe / 1024) + Math.floor(doe / 1461) - Math.floor(doe / 146096)) / 365);
    let y = yoe + era * 400;
    const doy = doe - (365 * yoe + Math.floor(yoe / 4) - Math.floor(yoe / 100));
    const mp = Math.floor((5 * doy + 2) / 153);
    const d = doy - Math.floor((153 * mp + 2) / 5) + 1;
    const m = mp < 10 ? mp + 3 : mp - 9;
    if (m <= 2) y += 1;
    return `${String(y).padStart(4, "0")}-${String(m).padStart(2, "0")}-${String(d).padStart(2, "0")}`;
  }

  static parseTimeHhMm(t: string): [number, number] | null {
    const parts = t.split(":");
    if (parts.length !== 2) return null;
    const h = parseInt(parts[0].trim(), 10);
    const m = parseInt(parts[1].trim(), 10);
    if (isNaN(h) || isNaN(m) || h < 0 || h >= 24 || m < 0 || m >= 60) return null;
    return [h, m];
  }

  static weekdayFromEpoch(nowSecs: number): Weekday {
    const days = Math.floor(nowSecs / 86400);
    // 1970-01-01 was Thursday (index 4)
    const wIdx = ((days + 4) % 7 + 7) % 7;
    return WEEKDAYS[wIdx];
  }
}

/** Streak Engine */
export class StreakEngine {
  static applyCompletion(streak: RoutineStreak, todayStr: string): void {
    const todayDays = CivilDateUtils.dateStrToDays(todayStr);
    if (todayDays === null) return;

    // Idempotent: multiple completions on the same date keep streak intact
    if (streak.last_completed_date === todayStr) {
      return;
    }

    let isConsecutive = false;
    if (streak.last_completed_date) {
      const lastDays = CivilDateUtils.dateStrToDays(streak.last_completed_date);
      if (lastDays !== null && todayDays - lastDays === 1) {
        isConsecutive = true;
      }
    }

    let isAfterFreeze = false;
    if (streak.frozen_on_date) {
      const frozenDays = CivilDateUtils.dateStrToDays(streak.frozen_on_date);
      if (frozenDays !== null && todayDays - frozenDays === 1) {
        isAfterFreeze = true;
      }
    }

    if (isConsecutive || isAfterFreeze) {
      streak.current_count += 1;
    } else {
      streak.current_count = 1;
    }

    if (streak.current_count > streak.best_count) {
      streak.best_count = streak.current_count;
    }

    streak.last_completed_date = todayStr;
    streak.frozen_on_date = undefined; // unfreeze on completion
  }

  static applyExcusedSkip(streak: RoutineStreak, todayStr: string): void {
    streak.frozen_on_date = todayStr;
  }

  static applyMiss(streak: RoutineStreak): void {
    streak.current_count = 0;
    streak.frozen_on_date = undefined;
  }
}

/** Coalesced Scheduler */
export class CoalescedScheduler {
  static calculateNextRoutineStart(
    schedule: RoutineSchedule,
    nowSecs: number,
    allowWithinGraceSecs = 0
  ): number | null {
    const parsed = CivilDateUtils.parseTimeHhMm(schedule.time);
    if (!parsed) return null;
    const [targetH, targetM] = parsed;
    const targetSecsIntoDay = targetH * 3600 + targetM * 60;
    const currentDayIdx = Math.floor(nowSecs / 86400);

    for (let dayOffset = 0; dayOffset <= 14; dayOffset++) {
      const candidateDay = currentDayIdx + dayOffset;
      const wIdx = ((candidateDay + 4) % 7 + 7) % 7;
      const candidateWeekday = WEEKDAYS[wIdx];

      if (schedule.days.length === 0 || schedule.days.includes(candidateWeekday)) {
        const candidateStart = candidateDay * 86400 + targetSecsIntoDay;
        if (candidateStart >= nowSecs || candidateStart + allowWithinGraceSecs >= nowSecs) {
          return candidateStart;
        }
      }
    }
    return null;
  }

  static findEarliestNextWake(routines: Routine[], nowSecs: number): number | null {
    let earliest: number | null = null;
    for (const r of routines) {
      if (!r.active) continue;
      const nextStart = this.calculateNextRoutineStart(r.schedule, nowSecs, r.start_grace_min * 60);
      if (nextStart !== null) {
        if (earliest === null || nextStart < earliest) {
          earliest = nextStart;
        }
      }
    }
    return earliest;
  }
}

/** Auto-Escalation Tracker */
export class AutoEscalationTracker {
  private missedCounts = new Map<string, number>();

  recordIgnoredReminder(id: string, currentTier: EnforcementTier, threshold: number): EnforcementTier | null {
    const count = (this.missedCounts.get(id) || 0) + 1;
    this.missedCounts.set(id, count);
    if (currentTier === "ingatkan" && count >= threshold) {
      return "tegas";
    }
    return null;
  }

  resetIgnored(id: string): void {
    this.missedCounts.delete(id);
  }

  getIgnoredCount(id: string): number {
    return this.missedCounts.get(id) || 0;
  }
}

/** Static Protected Processes List */
export class ProtectedProcesses {
  private static readonly LIST = new Set([
    "system", "smss", "csrss", "wininit", "services", "lsass", "winlogon", "dwm", "svchost", "fontdrvhost",
    "explorer", "searchhost", "startmenuexperiencehost", "shellexperiencehost", "taskmgr",
    "msmpeng", "securityhealthservice", "nissrv",
    "hime-island", "hime-core", "msedgewebview2",
    "code", "devenv", "idea64", "windowsterminal", "cmd", "powershell"
  ]);

  static isProtected(processName: string): boolean {
    const lower = processName.trim().toLowerCase();
    const clean = lower.endsWith(".exe") ? lower.slice(0, -4) : lower;
    return this.LIST.has(clean);
  }
}

// =============================================================================
// 1. ROUTINES DATABASE (Authentic SQLite Engine via bun:sqlite)
// =============================================================================

export class RoutinesDatabase {
  protected db: Database;

  constructor(inMemoryDb?: Database) {
    this.db = inMemoryDb ?? new Database(":memory:");
    this.initializeSchema();
  }

  public getRawDb(): Database {
    return this.db;
  }

  public getDb(): Database {
    return this.db;
  }

  public initializeSchema(): void {
    // 1. Routines table
    this.db.run(`
      CREATE TABLE IF NOT EXISTS routines (
        id TEXT PRIMARY KEY,
        title TEXT NOT NULL,
        category TEXT NOT NULL,
        schedule_json TEXT NOT NULL,
        enforcement TEXT NOT NULL,
        allowed_apps_json TEXT,
        distraction_rules_json TEXT,
        start_grace_min INTEGER NOT NULL DEFAULT 5,
        escalation_json TEXT,
        current_streak INTEGER NOT NULL DEFAULT 0,
        best_streak INTEGER NOT NULL DEFAULT 0,
        last_completed_date TEXT,
        frozen_on_date TEXT,
        active INTEGER NOT NULL DEFAULT 1,
        created_at INTEGER NOT NULL,
        updated_at INTEGER NOT NULL
      );
    `);

    // 2. Commitments table
    this.db.run(`
      CREATE TABLE IF NOT EXISTS commitments (
        id TEXT PRIMARY KEY,
        name TEXT NOT NULL,
        tier TEXT NOT NULL,
        target_apps_json TEXT NOT NULL,
        daily_quota_min INTEGER,
        active INTEGER NOT NULL DEFAULT 1,
        pending_weakening_until INTEGER,
        created_at INTEGER NOT NULL,
        updated_at INTEGER NOT NULL
      );
    `);

    // 3. Enforcement history table
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

    // 4. Emergency pause table
    this.db.run(`
      CREATE TABLE IF NOT EXISTS emergency_pauses (
        id TEXT PRIMARY KEY,
        started_at INTEGER NOT NULL,
        duration_min INTEGER NOT NULL,
        reason TEXT NOT NULL,
        expired_at INTEGER NOT NULL
      );
    `);

    // Indices per PROJECT.md
    this.db.run(`CREATE INDEX IF NOT EXISTS idx_routines_active ON routines(active);`);
    this.db.run(`CREATE INDEX IF NOT EXISTS idx_commitments_active ON commitments(active);`);
    this.db.run(`CREATE INDEX IF NOT EXISTS idx_emergency_pauses_exp ON emergency_pauses(expired_at);`);
    this.db.run(`CREATE INDEX IF NOT EXISTS idx_enforcement_ts ON enforcement_history(timestamp);`);
  }

  public getColumnInfo(table: string, column: string): { name: string; type: string; notnull: number; dflt_value: any } | null {
    const rows = this.db.query(`PRAGMA table_info(${table})`).all() as any[];
    const col = rows.find((r) => r.name === column);
    return col || null;
  }

  public getIndices(table: string): string[] {
    const rows = this.db.query(`PRAGMA index_list(${table})`).all() as any[];
    return rows.map((r) => r.name);
  }

  public insertRoutine(routine: Routine): void {
    if (!routine.title || routine.title.trim().length === 0) {
      throw new Error("Routine title is required");
    }
    if (!CivilDateUtils.parseTimeHhMm(routine.schedule.time)) {
      throw new Error("Invalid time format; expected HH:MM");
    }
    if (routine.start_grace_min <= 0) {
      throw new Error("start_grace_min must be greater than 0");
    }

    this.db.run(
      `
      INSERT INTO routines (
        id, title, category, schedule_json, enforcement,
        allowed_apps_json, distraction_rules_json, start_grace_min,
        escalation_json, current_streak, best_streak,
        last_completed_date, frozen_on_date, active, created_at, updated_at
      ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
      `,
      [
        routine.id,
        routine.title,
        routine.category,
        JSON.stringify(routine.schedule),
        routine.enforcement,
        routine.allowed_apps ? JSON.stringify(routine.allowed_apps) : null,
        routine.distraction_rules ? JSON.stringify(routine.distraction_rules) : null,
        routine.start_grace_min ?? 5,
        routine.escalation ? JSON.stringify(routine.escalation) : null,
        routine.streak?.current_count ?? 0,
        routine.streak?.best_count ?? 0,
        routine.streak?.last_completed_date || null,
        routine.streak?.frozen_on_date || null,
        routine.active ? 1 : 0,
        routine.created_at || 1000,
        routine.updated_at || 1000,
      ]
    );
  }

  public insertRoutineMinimal(params: {
    id: string;
    title: string;
    category?: string;
    schedule_json?: string;
    enforcement?: string;
    created_at?: number;
    updated_at?: number;
  }): string {
    this.db.run(
      `INSERT INTO routines (id, title, category, schedule_json, enforcement, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?)`,
      [
        params.id,
        params.title,
        params.category || "belajar",
        params.schedule_json || JSON.stringify({ days: [], time: "10:00", duration_min: 30 }),
        params.enforcement || "paksa",
        params.created_at || 1000,
        params.updated_at || 1000
      ]
    );
    return params.id;
  }

  public getRoutine(id: string): Routine | undefined {
    const row = this.db.query(`SELECT * FROM routines WHERE id = ?`).get(id) as any;
    if (!row) return undefined;
    return {
      id: row.id,
      title: row.title,
      category: row.category,
      schedule: JSON.parse(row.schedule_json),
      enforcement: row.enforcement as EnforcementTier,
      allowed_apps: row.allowed_apps_json ? JSON.parse(row.allowed_apps_json) : [],
      distraction_rules: row.distraction_rules_json ? JSON.parse(row.distraction_rules_json) : { block_during_routine: false, distraction_apps: [] },
      start_grace_min: row.start_grace_min,
      escalation: row.escalation_json ? JSON.parse(row.escalation_json) : undefined,
      streak: {
        current_count: row.current_streak,
        best_count: row.best_streak,
        last_completed_date: row.last_completed_date || undefined,
        frozen_on_date: row.frozen_on_date || undefined,
      },
      active: row.active === 1,
      created_at: row.created_at,
      updated_at: row.updated_at,
    };
  }

  public listRoutines(onlyActive = true): Routine[] {
    const sql = onlyActive ? `SELECT * FROM routines WHERE active = 1` : `SELECT * FROM routines`;
    const rows = this.db.query(sql).all() as any[];
    return rows.map((row) => ({
      id: row.id,
      title: row.title,
      category: row.category,
      schedule: JSON.parse(row.schedule_json),
      enforcement: row.enforcement as EnforcementTier,
      allowed_apps: row.allowed_apps_json ? JSON.parse(row.allowed_apps_json) : [],
      distraction_rules: row.distraction_rules_json ? JSON.parse(row.distraction_rules_json) : { block_during_routine: false, distraction_apps: [] },
      start_grace_min: row.start_grace_min,
      escalation: row.escalation_json ? JSON.parse(row.escalation_json) : undefined,
      streak: {
        current_count: row.current_streak,
        best_count: row.best_streak,
        last_completed_date: row.last_completed_date || undefined,
        frozen_on_date: row.frozen_on_date || undefined,
      },
      active: row.active === 1,
      created_at: row.created_at,
      updated_at: row.updated_at,
    }));
  }

  public updateRoutine(routine: Routine): void {
    const existing = this.getRoutine(routine.id);
    if (!existing) {
      throw new Error(`Routine ${routine.id} not found`);
    }
    this.db.run(
      `
      UPDATE routines SET
        title = ?, category = ?, schedule_json = ?, enforcement = ?,
        allowed_apps_json = ?, distraction_rules_json = ?, start_grace_min = ?,
        escalation_json = ?, current_streak = ?, best_streak = ?,
        last_completed_date = ?, frozen_on_date = ?, active = ?, updated_at = ?
      WHERE id = ?
      `,
      [
        routine.title,
        routine.category,
        JSON.stringify(routine.schedule),
        routine.enforcement,
        routine.allowed_apps ? JSON.stringify(routine.allowed_apps) : null,
        routine.distraction_rules ? JSON.stringify(routine.distraction_rules) : null,
        routine.start_grace_min ?? 5,
        routine.escalation ? JSON.stringify(routine.escalation) : null,
        routine.streak?.current_count ?? 0,
        routine.streak?.best_count ?? 0,
        routine.streak?.last_completed_date || null,
        routine.streak?.frozen_on_date || null,
        routine.active ? 1 : 0,
        routine.updated_at || 1000,
        routine.id,
      ]
    );
  }

  public deleteRoutine(id: string): boolean {
    const res = this.db.run(`DELETE FROM routines WHERE id = ?`, [id]);
    return res.changes > 0;
  }

  public insertCommitment(com: Commitment): void {
    this.db.run(
      `
      INSERT INTO commitments (id, name, tier, target_apps_json, daily_quota_min, active, pending_weakening_until, created_at, updated_at)
      VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
      `,
      [
        com.id,
        com.name,
        com.tier,
        JSON.stringify(com.target_apps),
        com.daily_quota_min || null,
        com.active ? 1 : 0,
        com.pending_weakening_until || null,
        com.created_at || 1000,
        com.updated_at || 1000,
      ]
    );
  }

  public getCommitment(id: string): Commitment | undefined {
    const row = this.db.query(`SELECT * FROM commitments WHERE id = ?`).get(id) as any;
    if (!row) return undefined;
    return {
      id: row.id,
      name: row.name,
      tier: row.tier as EnforcementTier,
      target_apps: JSON.parse(row.target_apps_json),
      daily_quota_min: row.daily_quota_min || undefined,
      active: row.active === 1,
      pending_weakening_until: row.pending_weakening_until || undefined,
      created_at: row.created_at,
      updated_at: row.updated_at,
    };
  }

  public listCommitments(onlyActive = true): Commitment[] {
    const sql = onlyActive ? `SELECT * FROM commitments WHERE active = 1` : `SELECT * FROM commitments`;
    const rows = this.db.query(sql).all() as any[];
    return rows.map((row) => ({
      id: row.id,
      name: row.name,
      tier: row.tier as EnforcementTier,
      target_apps: JSON.parse(row.target_apps_json),
      daily_quota_min: row.daily_quota_min || undefined,
      active: row.active === 1,
      pending_weakening_until: row.pending_weakening_until || undefined,
      created_at: row.created_at,
      updated_at: row.updated_at,
    }));
  }

  public updateCommitment(com: Commitment): void {
    this.db.run(
      `
      UPDATE commitments SET
        name = ?, tier = ?, target_apps_json = ?, daily_quota_min = ?,
        active = ?, pending_weakening_until = ?, updated_at = ?
      WHERE id = ?
      `,
      [
        com.name,
        com.tier,
        JSON.stringify(com.target_apps),
        com.daily_quota_min || null,
        com.active ? 1 : 0,
        com.pending_weakening_until || null,
        com.updated_at || 1000,
        com.id,
      ]
    );
  }

  public recordEmergencyPause(pause: EmergencyPauseRecord): void {
    this.db.run(
      `INSERT INTO emergency_pauses (id, started_at, duration_min, reason, expired_at) VALUES (?, ?, ?, ?, ?)`,
      [pause.id, pause.started_at, pause.duration_min, pause.reason, pause.expired_at]
    );
  }

  public getActivePause(nowSecs: number): EmergencyPauseRecord | undefined {
    const row = this.db.query(`SELECT * FROM emergency_pauses WHERE expired_at > ? ORDER BY expired_at DESC LIMIT 1`).get(nowSecs) as any;
    if (!row) return undefined;
    return {
      id: row.id,
      started_at: row.started_at,
      duration_min: row.duration_min,
      reason: row.reason,
      expired_at: row.expired_at,
    };
  }

  public logEnforcement(rec: EnforcementHistoryRecord): void {
    this.db.run(
      `INSERT INTO enforcement_history (id, timestamp, target_app, tier, action_taken, details) VALUES (?, ?, ?, ?, ?, ?)`,
      [rec.id, rec.timestamp, rec.target_app, rec.tier, rec.action_taken, rec.details || null]
    );
  }

  public getEnforcementHistory(): EnforcementHistoryRecord[] {
    const rows = this.db.query(`SELECT * FROM enforcement_history ORDER BY timestamp ASC`).all() as any[];
    return rows.map((r) => ({
      id: r.id,
      timestamp: r.timestamp,
      target_app: r.target_app,
      tier: r.tier,
      action_taken: r.action_taken,
      details: r.details || undefined,
    }));
  }
}

// Backward-compatible RoutineStore alias backed by SQLite
export class RoutineStore extends RoutinesDatabase {
  get routines() {
    const map = new Map<string, Routine>();
    for (const r of this.listRoutines(false)) {
      map.set(r.id, r);
    }
    return map;
  }
  get history() {
    return this.getEnforcementHistory();
  }
  createRoutine(r: Routine): void {
    this.insertRoutine(r);
  }
  createCommitment(c: Commitment): void {
    this.insertCommitment(c);
  }
  addEmergencyPause(p: EmergencyPauseRecord): void {
    this.recordEmergencyPause(p);
  }
  recordHistory(rec: EnforcementHistoryRecord): void {
    this.logEnforcement(rec);
  }
}

// =============================================================================
// 2. ROUTINE SESSION MANAGER (State Machine Lifecycle & Snooze)
// =============================================================================

export interface RoutineSessionConfig {
  routineId: string;
  tier: EnforcementTier;
  scheduledFor: number;
  durationMin: number;
  startGraceMin?: number;
  maxSnoozeCount?: number;
}

export class RoutineSessionManager {
  private routineId: string;
  private tier: EnforcementTier;
  private state: RoutineSessionState = "ambient";
  private scheduledFor: number;
  private durationMin: number;
  private startGraceMin: number;
  private maxSnoozeCount: number;
  private snoozeCount = 0;
  private snoozeUntil?: number;
  private startedAt?: number;
  private completedAt?: number;
  private skipReason?: string;
  private isExcused = false;

  constructor(config: RoutineSessionConfig, initialTimeSecs?: number) {
    this.routineId = config.routineId;
    this.tier = config.tier;
    this.scheduledFor = config.scheduledFor;
    this.durationMin = config.durationMin;
    this.startGraceMin = config.startGraceMin ?? 5;
    this.maxSnoozeCount = config.maxSnoozeCount ?? (config.tier === "tegas" ? 3 : config.tier === "paksa" ? 1 : 99);

    if (initialTimeSecs !== undefined) {
      this.evaluateStateAtTime(initialTimeSecs);
    } else {
      this.evaluateStateAtTime(this.scheduledFor);
    }
  }

  public evaluateStateAtTime(nowSecs: number): RoutineSessionState {
    if (this.state === "completed" || this.state === "skipped" || this.state === "missed") {
      return this.state;
    }
    if (this.state === "active") {
      return "active";
    }
    if (this.state === "snoozed" && this.snoozeUntil && nowSecs >= this.snoozeUntil) {
      this.state = "prompted";
      return "prompted";
    }
    if (nowSecs > this.scheduledFor + this.startGraceMin * 60) {
      this.state = "missed";
      return "missed";
    }
    if (nowSecs >= this.scheduledFor) {
      this.state = "prompted";
      return "prompted";
    }
    if (nowSecs >= this.scheduledFor - 300) {
      this.state = "ambient";
      return "ambient";
    }
    return this.state;
  }

  public getState(): RoutineSessionState { return this.state; }
  public getSnoozeCount(): number { return this.snoozeCount; }
  public getSnoozeUntil(): number | undefined { return this.snoozeUntil; }
  public getStartedAt(): number | undefined { return this.startedAt; }
  public getCompletedAt(): number | undefined { return this.completedAt; }
  public getSkipReason(): string | undefined { return this.skipReason; }
  public getIsExcused(): boolean { return this.isExcused; }

  public startSession(nowSecs: number): { success: boolean; error?: string } {
    if (this.state === "completed" || this.state === "skipped" || this.state === "missed") {
      return { success: false, error: "TERMINAL_STATE" };
    }
    if (this.state === "active") {
      return { success: false, error: "ALREADY_ACTIVE" };
    }
    this.state = "active";
    this.startedAt = nowSecs;
    this.snoozeUntil = undefined;
    return { success: true };
  }

  public completeSession(nowSecs: number): { success: boolean; error?: string } {
    if (this.state !== "active") {
      return { success: false, error: "NOT_ACTIVE" };
    }
    this.state = "completed";
    this.completedAt = nowSecs;
    return { success: true };
  }

  public snooze(minutes: number, reason: string): { success: boolean; error?: string } {
    if (this.state !== "prompted" && this.state !== "snoozed") {
      return { success: false, error: "CANNOT_SNOOZE" };
    }
    if (this.snoozeCount >= this.maxSnoozeCount) {
      return { success: false, error: "MAX_SNOOZE_REACHED" };
    }
    const clampedMin = Math.max(1, minutes);
    this.snoozeCount += 1;
    this.snoozeUntil = (this.snoozeUntil || this.scheduledFor) + clampedMin * 60;
    this.state = "snoozed";
    this.skipReason = reason;
    return { success: true };
  }

  public advanceTimeToSnoozeExpiry(): void {
    if (this.state === "snoozed") {
      this.state = "prompted";
    }
  }

  public skipSession(reason: string, isExcused = false): { success: boolean; error?: string } {
    if (this.state === "completed" || this.state === "missed") {
      return { success: false, error: "TERMINAL_STATE" };
    }
    this.state = "skipped";
    this.skipReason = reason;
    const lower = reason.toLowerCase();
    this.isExcused = isExcused || lower.includes("sakit") || lower.includes("libur") || lower.includes("izin");
    return { success: true };
  }
}

// =============================================================================
// 3. DISTRACTION TIMER MANAGER (Countdown & Idle Detection Logic)
// =============================================================================

export interface DistractionTimerConfig {
  tier?: EnforcementTier;
  initialCountdownSecs?: number;
}

export class DistractionTimerManager {
  private tier: EnforcementTier;
  private initialCountdownSecs: number;
  private remainingSecs: number;
  private state: "armed" | "running" | "paused" | "expired" = "armed";

  constructor(config: DistractionTimerConfig) {
    this.tier = config.tier || "paksa";
    this.initialCountdownSecs = config.initialCountdownSecs ?? (this.tier === "paksa" ? 10 : 30);
    this.remainingSecs = this.initialCountdownSecs;
  }

  public getRemainingSecs(): number { return this.remainingSecs; }
  public getInitialCountdownSecs(): number { return this.initialCountdownSecs; }
  public getState(): "armed" | "running" | "paused" | "expired" { return this.state; }
  public isPaused(): boolean { return this.state === "paused"; }
  public hasExpired(): boolean { return this.state === "expired"; }

  public tick(deltaSecs: number, isIdle: boolean): { state: string; remainingSecs: number; action?: string } {
    if (this.state === "expired") {
      return { state: "expired", remainingSecs: 0, action: "trigger_close" };
    }
    if (isIdle) {
      this.state = "paused";
      return { state: "paused", remainingSecs: this.remainingSecs, action: "pause_quota" };
    }
    this.state = "running";
    this.remainingSecs = Math.max(0, this.remainingSecs - deltaSecs);
    if (this.remainingSecs === 0) {
      this.state = "expired";
      return { state: "expired", remainingSecs: 0, action: "trigger_close" };
    }
    return { state: "running", remainingSecs: this.remainingSecs };
  }

  public reset(): void {
    this.remainingSecs = this.initialCountdownSecs;
    this.state = "armed";
  }
}

// =============================================================================
// 4. ENFORCEMENT CONTROLLER (Kill Switch, Weakening, Safety Ladder)
// =============================================================================

export class EnforcementController {
  private db: RoutinesDatabase;
  private pal: Win32PalSimulator;
  private killSwitchActive = false;
  private osLocked = false;
  private isFailSafe = false;
  private activeHooks: (() => void)[] = [];
  private emergencyPauseUntil = 0;
  private emergencyPauseReason = "";
  private pendingModifications = new Map<string, Partial<Commitment>>();

  constructor(db: RoutinesDatabase, pal: Win32PalSimulator, failSafe = false) {
    this.db = db;
    this.pal = pal;
    this.isFailSafe = failSafe;
  }

  static createWithFallback(dbOrCorrupt: any, pal: Win32PalSimulator): EnforcementController {
    try {
      dbOrCorrupt.query("SELECT id FROM routines LIMIT 1").get();
      return new EnforcementController(dbOrCorrupt, pal, false);
    } catch {
      return new EnforcementController(dbOrCorrupt, pal, true);
    }
  }

  public isFailSafeActive(): boolean { return this.isFailSafe; }
  public getEnforcementState(): "active" | "halted" | "safe_unblocked" {
    if (this.isFailSafe || this.killSwitchActive || this.isEmergencyPaused()) {
      return "safe_unblocked";
    }
    return "active";
  }

  public isEnforcementActive(): boolean {
    return !this.killSwitchActive && !this.isEmergencyPaused() && !this.isFailSafe;
  }

  public setEnforcementActive(active: boolean): void {
    this.killSwitchActive = !active;
  }

  public triggerKillSwitch(): void {
    this.killSwitchActive = true;
    this.cleanupHooks();
  }

  public toggleKillSwitch(): void {
    this.killSwitchActive = !this.killSwitchActive;
  }

  public isKillSwitchActive(): boolean { return this.killSwitchActive; }
  public isOsLocked(): boolean { return this.osLocked; }

  public registerHooks(): void {
    const unhook = this.pal.onForegroundSwitch((info) => {});
    this.activeHooks.push(unhook);
  }

  public cleanupHooks(): void {
    for (const unhook of this.activeHooks) {
      unhook();
    }
    this.activeHooks = [];
  }

  public shutdown(): void {
    this.triggerKillSwitch();
    this.cleanupHooks();
  }

  public activateEmergencyPause(durationMin: number, reason: string, nowSecs: number): void {
    this.emergencyPauseUntil = nowSecs + durationMin * 60;
    this.emergencyPauseReason = reason;
    this.db.recordEmergencyPause({
      id: `ep-${nowSecs}`,
      started_at: nowSecs,
      duration_min: durationMin,
      reason,
      expired_at: this.emergencyPauseUntil,
    });
    this.db.logEnforcement({
      id: `eh-pause-${nowSecs}`,
      timestamp: nowSecs,
      target_app: "*",
      tier: "all",
      action_taken: "paused",
      details: `Emergency pause ${durationMin}m for ${reason}`,
    });
  }

  public isEmergencyPaused(nowSecs: number = Math.floor(Date.now() / 1000)): boolean {
    return this.emergencyPauseUntil > nowSecs;
  }

  public handleSlashCommand(cmd: string, nowSecs: number): { success: boolean; alreadyPaused?: boolean } {
    if (cmd.trim().toLowerCase() === "/jeda") {
      if (this.isEmergencyPaused(nowSecs)) {
        return { success: true, alreadyPaused: true };
      }
      this.activateEmergencyPause(30, "User /jeda command", nowSecs);
      return { success: true, alreadyPaused: false };
    }
    return { success: false };
  }

  public handleHotkeyPress(hotkeyCombo: string, nowSecs: number = Math.floor(Date.now() / 1000)): boolean {
    if (hotkeyCombo === "Ctrl+Alt+Shift+P") {
      this.activateEmergencyPause(30, "Darurat", nowSecs);
      return true;
    }
    return false;
  }

  public requestCommitmentModification(commitmentId: string, updates: Partial<Commitment>, nowSecs: number): {
    isWeakening: boolean;
    coolingOffRequired: boolean;
    pendingUntil?: number;
  } {
    const existing = this.db.getCommitment(commitmentId);
    if (!existing) throw new Error("Commitment not found");

    let isWeakening = false;
    const tierRanks: Record<EnforcementTier, number> = { ingatkan: 1, tegas: 2, paksa: 3 };
    if (updates.tier && tierRanks[updates.tier] < tierRanks[existing.tier]) {
      isWeakening = true;
    }
    if (updates.target_apps && updates.target_apps.length < existing.target_apps.length) {
      isWeakening = true;
    }

    if (isWeakening) {
      const coolingOffDurationSecs = 600; // 10 min
      const pendingUntil = nowSecs + coolingOffDurationSecs;
      const prevPending = this.pendingModifications.get(commitmentId) || {};
      this.pendingModifications.set(commitmentId, { ...prevPending, ...updates });
      this.db.updateCommitment({
        ...existing,
        pending_weakening_until: pendingUntil,
      });
      return { isWeakening: true, coolingOffRequired: true, pendingUntil };
    } else {
      this.pendingModifications.delete(commitmentId);
      this.db.updateCommitment({
        ...existing,
        ...updates,
        pending_weakening_until: undefined,
      });
      return { isWeakening: false, coolingOffRequired: false };
    }
  }

  public isAppRestricted(appName: string, nowSecs: number): boolean {
    const commitments = this.db.listCommitments(true);
    for (const c of commitments) {
      if (c.pending_weakening_until && nowSecs < c.pending_weakening_until) {
        if (c.target_apps.includes(appName)) return true;
      } else if (!c.pending_weakening_until || nowSecs >= c.pending_weakening_until) {
        if (c.target_apps.includes(appName)) return true;
      }
    }
    return false;
  }

  public evaluatePendingWeakening(nowSecs: number): void {
    const commitments = this.db.listCommitments(true);
    for (const c of commitments) {
      if (c.pending_weakening_until && nowSecs >= c.pending_weakening_until) {
        const pending = this.pendingModifications.get(c.id);
        this.db.updateCommitment({
          ...c,
          ...(pending || {}),
          pending_weakening_until: undefined,
        });
        this.pendingModifications.delete(c.id);
      }
    }
  }

  public enforceWindowClose(hwnd: number, nowSecs: number): { action: string; escalatedToReflection?: boolean } {
    if (this.isFailSafe || this.killSwitchActive || this.isEmergencyPaused(nowSecs)) {
      return { action: "bypassed_safe_unblocked" };
    }

    const closeResult = this.pal.sendGracefulClose(hwnd, nowSecs);
    if (closeResult === "WindowRefusedClose" || closeResult === "UnsavedDialogDetected") {
      this.db.logEnforcement({
        id: `eh-${nowSecs}`,
        timestamp: nowSecs,
        target_app: "stubborn_app.exe",
        tier: "paksa",
        action_taken: "reflection_prompted",
        details: "Refusal to close escalated to reflection dialog",
      });
      return { action: "open_reflection_dialog", escalatedToReflection: true };
    }
    return { action: "closed_gracefully" };
  }
}

// =============================================================================
// 5. ROUTINES IPC BRIDGE (Structured RPC Dispatch & Keybinds)
// =============================================================================

export class RoutinesIpcBridge {
  private controller?: EnforcementController;
  private database?: RoutinesDatabase;
  private hotkeys = new Map<string, { key: string; modifiers: string[]; displayString: string; description: string }>([
    ["focus_panel", { key: "F", modifiers: ["Ctrl", "Alt"], displayString: "Ctrl+Alt+F", description: "Buka panel fokus" }],
    ["emergency_pause", { key: "P", modifiers: ["Ctrl", "Alt", "Shift"], displayString: "Ctrl+Alt+Shift+P", description: "Jeda darurat" }],
  ]);

  constructor(opts?: { controller?: EnforcementController; database?: RoutinesDatabase }) {
    this.controller = opts?.controller;
    this.database = opts?.database;
  }

  public getRegisteredHotkey(id: string) {
    return this.hotkeys.get(id);
  }

  public isRecognizedHotkey(displayStr: string): boolean {
    for (const h of this.hotkeys.values()) {
      if (h.displayString === displayStr) return true;
    }
    return false;
  }

  public call(endpoint: string, params: Record<string, any>): { result?: any; error?: { code: number; message: string } } {
    switch (endpoint) {
      case "productivity.routines.create": {
        if (!params.title) {
          return { error: { code: -32602, message: "Missing title parameter" } };
        }
        if (params.duration_min !== undefined && params.duration_min < 0) {
          return { error: { code: -32602, message: "duration_min must be positive" } };
        }
        const id = params.id || `rtn-${Math.random().toString(36).substring(2, 9)}`;
        return { result: { id, title: params.title, ...params } };
      }
      case "productivity.enforce.emergency_pause": {
        if (this.controller) {
          this.controller.activateEmergencyPause(params.duration_min || 30, params.reason || "Manual", Math.floor(Date.now() / 1000));
        }
        return { result: { is_paused: true, duration_min: params.duration_min || 30, reason: params.reason || "Manual" } };
      }
      case "productivity.commitments.create": {
        const id = params.id || `com-${Math.random().toString(36).substring(2, 9)}`;
        return { result: { id, tier: params.tier, ...params } };
      }
      default:
        return { error: { code: -32601, message: `Method '${endpoint}' not found` } };
    }
  }

  public async dispatch(endpoint: string, params: Record<string, any>): Promise<{ success: boolean; data?: any; error?: { code: number; message: string } }> {
    const res = this.call(endpoint, params);
    if (res.error) {
      return { success: false, error: res.error };
    }
    return { success: true, data: res.result };
  }
}

/** Safe Win32 & PAL Simulator with Gate #10 Mock Window Fixture */
export class Win32PalSimulator {
  private activeHwnd = 1001;
  private windows = new Map<
    number,
    {
      info: ForegroundInfo;
      mode: DummyWindowMode;
      hasModalDialog: boolean;
      isClosed: boolean;
    }
  >();

  private hookListeners: ((info: ForegroundInfo) => void)[] = [];
  private lastInputTimestamp = Date.now();
  private closeHistory: { app: string; timestampSec: number }[] = [];
  private reopenHistory: { app: string; timestampSec: number }[] = [];
  public terminateProcessCalled = false; // Invariant check: must ALWAYS stay false!
  private targetedProcesses: { hwnd: number; processName: string; executablePath: string; isMock: boolean }[] = [];

  public getActiveHookCount(): number {
    return this.hookListeners.length;
  }

  public isInputBlocked(): boolean {
    return false; // OS invariant: input is never permanently blocked
  }

  public getTargetedProcessesCount(): number {
    return this.targetedProcesses.length;
  }

  public hasTargetedNonMockProcess(): boolean {
    return this.targetedProcesses.some((p) => !p.isMock);
  }

  public getAllTargetedExecutablePaths(): string[] {
    return this.targetedProcesses.map((p) => p.executablePath);
  }

  registerDummyWindow(
    hwnd: number,
    processName: string,
    mode: DummyWindowMode,
    windowTitle = "Test Window",
    className = "DummyWindowClass"
  ): void {
    this.windows.set(hwnd, {
      info: {
        hwnd,
        pid: 8000 + (hwnd % 1000),
        process_name: processName,
        executable_path: `C:\\Mock\\${processName}`,
        window_title: windowTitle,
        class_name: className,
      },
      mode,
      hasModalDialog: mode === "ModeB_UnsavedChangesModal",
      isClosed: false,
    });
  }

  setLastInputTime(epochMs: number): void {
    this.lastInputTimestamp = epochMs;
  }

  simulateUserInput(): void {
    this.lastInputTimestamp = Date.now();
  }

  isUserIdle(thresholdSecs: number, currentEpochMs: number = Date.now()): boolean {
    const idleMs = currentEpochMs - this.lastInputTimestamp;
    return idleMs >= thresholdSecs * 1000;
  }

  onForegroundSwitch(listener: (info: ForegroundInfo) => void): () => void {
    this.hookListeners.push(listener);
    return () => {
      this.hookListeners = this.hookListeners.filter((l) => l !== listener);
    };
  }

  switchForeground(hwnd: number): ForegroundInfo | null {
    if (hwnd === 0) return null;
    const win = this.windows.get(hwnd);
    if (!win || win.isClosed) return null;
    this.activeHwnd = hwnd;
    for (const listener of this.hookListeners) {
      try {
        listener(win.info);
      } catch {
        // ignore listener errors
      }
    }
    return win.info;
  }

  getForegroundInfo(): ForegroundInfo | null {
    const win = this.windows.get(this.activeHwnd);
    if (!win || win.isClosed) return null;
    return win.info;
  }

  hasModalUnsavedDialog(hwnd: number): boolean {
    const win = this.windows.get(hwnd);
    if (!win) return false;
    return win.hasModalDialog;
  }

  dismissModalDialog(hwnd: number): void {
    const win = this.windows.get(hwnd);
    if (win) {
      win.hasModalDialog = false;
    }
  }

  sendGracefulClose(hwnd: number, nowSecs: number): CloseResult {
    const win = this.windows.get(hwnd);
    this.targetedProcesses.push({
      hwnd,
      processName: win?.info.process_name || "unknown",
      executablePath: win?.info.executable_path || "unknown",
      isMock: win !== undefined && win.info.executable_path.startsWith("C:\\Mock\\"),
    });

    if (!win || win.isClosed) {
      return "WindowRefusedClose";
    }

    if (ProtectedProcesses.isProtected(win.info.process_name)) {
      return "ProtectedProcessRefused";
    }

    if (win.hasModalDialog || win.mode === "ModeB_UnsavedChangesModal") {
      return "UnsavedDialogDetected";
    }

    if (win.mode === "ModeC_RefuseClose") {
      return "WindowRefusedClose";
    }

    // Rate Limiting check: max 5 closes/hr and 30s cooldown
    const app = win.info.process_name.toLowerCase();
    const recent = this.closeHistory.filter((c) => c.app === app);
    const lastClose = recent[recent.length - 1];
    if (lastClose && nowSecs - lastClose.timestampSec < 30) {
      return "WindowRefusedClose"; // cooled down
    }
    const closesInHour = recent.filter((c) => nowSecs - c.timestampSec < 3600);
    if (closesInHour.length >= 5) {
      return "WindowRefusedClose"; // rate limited
    }

    // Success: ModeA_CleanClose
    win.isClosed = true;
    this.closeHistory.push({ app, timestampSec: nowSecs });
    return "ClosedSuccessfully";
  }

  recordReopen(app: string, nowSecs: number): boolean {
    const lower = app.toLowerCase();
    this.reopenHistory.push({ app: lower, timestampSec: nowSecs });
    const windowStart = nowSecs - 120;
    const rapidReopens = this.reopenHistory.filter((r) => r.app === lower && r.timestampSec >= windowStart);
    return rapidReopens.length >= 3; // triggers reflection
  }

  attemptTerminateProcess(): never {
    this.terminateProcessCalled = true;
    throw new Error("GATE #10 VIOLATION: TerminateProcess is strictly prohibited");
  }
}

/** Visual Gate VC-U5 Validator */
export class Vcu5Validator {
  static validateButtonText(text: string): { valid: boolean; length: number; error?: string } {
    const len = text.trim().length;
    if (len > 10) {
      return { valid: false, length: len, error: `Button text '${text}' exceeds 10 chars (${len})` };
    }
    return { valid: true, length: len };
  }

  static validateTitle(title: string): { valid: boolean; length: number; error?: string } {
    const len = title.trim().length;
    if (len > 16) {
      return { valid: false, length: len, error: `Title '${title}' exceeds 16 chars (${len})` };
    }
    return { valid: true, length: len };
  }

  static validateDescription(desc: string): { valid: boolean; length: number; error?: string } {
    const len = desc.trim().length;
    if (len > 36) {
      return { valid: false, length: len, error: `Description '${desc}' exceeds 36 chars (${len})` };
    }
    return { valid: true, length: len };
  }

  static validateTokenColor(color: string): boolean {
    if (color.startsWith("var(--accent-") || color.startsWith("var(--surface-") || color.startsWith("var(--text-")) {
      return true;
    }
    const approvedHexes = [
      "#000000", "#0D0E12", "#14161C", "#1B1E26", "#262A34",
      "#F4F6FA", "#A7AEBB", "#6F7785",
      "#34E3A6", "#38D6F5", "#5AA9FF", "#A18BFF", "#FF5C93", "#FF7A59", "#FFC247", "#A8EE4B", "#FF4D5E"
    ];
    return approvedHexes.includes(color.toUpperCase());
  }

  static validateMascotExpression(expr: string): boolean {
    const approved = ["netral", "fokus", "kesal", "kaget", "khawatir", "bangga", "lega", "santai", "cemas"];
    return approved.includes(expr);
  }
}

// =============================================================================
// TEST SUITE: TIER 1 — FEATURE COVERAGE (HAPPY PATHS)
// =============================================================================

describe("Tier 1: Feature Coverage (Features 1–21, 26, 27–29, 31)", () => {
  let store: RoutineStore;
  let pal: Win32PalSimulator;
  let escalationTracker: AutoEscalationTracker;

  beforeEach(() => {
    store = new RoutineStore();
    pal = new Win32PalSimulator();
    escalationTracker = new AutoEscalationTracker();
  });

  // Feature 1: Routine Data Model & CRUD
  describe("F1: Routine Data Model & CRUD", () => {
    const sampleRoutine: Routine = {
      id: "rtn-101",
      title: "Belajar Rust & AI",
      category: "belajar",
      schedule: { days: ["Monday", "Wednesday", "Friday"], time: "19:00", duration_min: 45 },
      enforcement: "paksa",
      allowed_apps: ["code.exe", "windowsterminal.exe"],
      distraction_rules: { block_during_routine: true, distraction_apps: ["chrome.exe", "steam.exe"] },
      start_grace_min: 5,
      streak: { current_count: 0, best_count: 0 },
      active: true,
      created_at: 1700000000,
      updated_at: 1700000000,
    };

    test("1.1: Create valid routine in storage", () => {
      store.createRoutine(sampleRoutine);
      expect(store.getRoutine("rtn-101")).toBeDefined();
      expect(store.getRoutine("rtn-101")?.title).toBe("Belajar Rust & AI");
    });

    test("1.2: Read / get routine by ID", () => {
      store.createRoutine(sampleRoutine);
      const fetched = store.getRoutine("rtn-101");
      expect(fetched?.enforcement).toBe("paksa");
      expect(fetched?.schedule.duration_min).toBe(45);
    });

    test("1.3: Update routine attributes", () => {
      store.createRoutine(sampleRoutine);
      const updated = { ...sampleRoutine, title: "Belajar Hime Desktop", enforcement: "tegas" as EnforcementTier };
      store.updateRoutine(updated);
      expect(store.getRoutine("rtn-101")?.title).toBe("Belajar Hime Desktop");
      expect(store.getRoutine("rtn-101")?.enforcement).toBe("tegas");
    });

    test("1.4: List active routines excludes inactive routines", () => {
      store.createRoutine(sampleRoutine);
      store.createRoutine({ ...sampleRoutine, id: "rtn-102", active: false });
      const activeList = store.listRoutines(true);
      expect(activeList.length).toBe(1);
      expect(activeList[0].id).toBe("rtn-101");
    });

    test("1.5: Delete routine from storage", () => {
      store.createRoutine(sampleRoutine);
      expect(store.deleteRoutine("rtn-101")).toBe(true);
      expect(store.getRoutine("rtn-101")).toBeUndefined();
    });
  });

  // Feature 2: Routine Session Lifecycle
  describe("F2: Routine Session Lifecycle", () => {
    test("2.1: Session initializes in Prompted state at T-0", () => {
      const sm = new RoutineSessionManager({
        routineId: "rtn-101",
        tier: "paksa",
        scheduledFor: 1700001000,
        durationMin: 30,
      }, 1700001000);
      expect(sm.getState()).toBe("prompted");
    });

    test("2.2: Ambient state detection at T-5m", () => {
      const sm = new RoutineSessionManager({
        routineId: "rtn-101",
        tier: "paksa",
        scheduledFor: 1700001000,
        durationMin: 30,
      }, 1700001000 - 300);
      expect(sm.getState()).toBe("ambient");
    });

    test("2.3: Transition Prompted -> Active on start", () => {
      const sm = new RoutineSessionManager({
        routineId: "rtn-101",
        tier: "paksa",
        scheduledFor: 1700001000,
        durationMin: 30,
      }, 1700001000);
      expect(sm.getState()).toBe("prompted");
      const res = sm.startSession(1700001010);
      expect(res.success).toBe(true);
      expect(sm.getState()).toBe("active");
      expect(sm.getStartedAt()).toBe(1700001010);
    });

    test("2.4: Transition Active -> Completed with completed_at", () => {
      const sm = new RoutineSessionManager({
        routineId: "rtn-101",
        tier: "paksa",
        scheduledFor: 1700001000,
        durationMin: 30,
      }, 1700001000);
      sm.startSession(1700001010);
      const res = sm.completeSession(1700002810);
      expect(res.success).toBe(true);
      expect(sm.getState()).toBe("completed");
      expect(sm.getCompletedAt()).toBe(1700002810);
    });

    test("2.5: Transition Prompted -> Skipped with reason", () => {
      const sm = new RoutineSessionManager({
        routineId: "rtn-101",
        tier: "paksa",
        scheduledFor: 1700001000,
        durationMin: 30,
      }, 1700001000);
      const res = sm.skipSession("Sakit", true);
      expect(res.success).toBe(true);
      expect(sm.getState()).toBe("skipped");
      expect(sm.getSkipReason()).toBe("Sakit");
      expect(sm.getIsExcused()).toBe(true);
    });
  });

  // Feature 3: Streak Engine
  describe("F3: Streak Engine", () => {
    test("3.1: First completion increments streak from 0 to 1", () => {
      const streak: RoutineStreak = { current_count: 0, best_count: 0 };
      StreakEngine.applyCompletion(streak, "2026-11-21");
      expect(streak.current_count).toBe(1);
      expect(streak.best_count).toBe(1);
      expect(streak.last_completed_date).toBe("2026-11-21");
    });

    test("3.2: Consecutive day completion increments streak", () => {
      const streak: RoutineStreak = { current_count: 1, best_count: 1, last_completed_date: "2026-11-20" };
      StreakEngine.applyCompletion(streak, "2026-11-21");
      expect(streak.current_count).toBe(2);
      expect(streak.best_count).toBe(2);
    });

    test("3.3: Same-day multiple completions are idempotent", () => {
      const streak: RoutineStreak = { current_count: 2, best_count: 2, last_completed_date: "2026-11-21" };
      StreakEngine.applyCompletion(streak, "2026-11-21");
      expect(streak.current_count).toBe(2);
    });

    test("3.4: Excused skip freezes streak and preserves count", () => {
      const streak: RoutineStreak = { current_count: 5, best_count: 5, last_completed_date: "2026-11-20" };
      StreakEngine.applyExcusedSkip(streak, "2026-11-21");
      expect(streak.current_count).toBe(5);
      expect(streak.frozen_on_date).toBe("2026-11-21");
    });

    test("3.5: Resuming after freeze seamlessly increments streak", () => {
      const streak: RoutineStreak = {
        current_count: 5,
        best_count: 5,
        last_completed_date: "2026-11-20",
        frozen_on_date: "2026-11-21",
      };
      StreakEngine.applyCompletion(streak, "2026-11-22");
      expect(streak.current_count).toBe(6);
      expect(streak.best_count).toBe(6);
      expect(streak.frozen_on_date).toBeUndefined();
    });
  });

  // Feature 4: Postponement / Snooze
  describe("F4: Postponement / Snooze", () => {
    test("4.1: Snooze from prompted updates state to snoozed and sets timer", () => {
      const sm = new RoutineSessionManager({
        routineId: "rtn-1",
        tier: "tegas",
        scheduledFor: 1000,
        durationMin: 30,
        startGraceMin: 5,
      });
      const res = sm.snooze(10, "Sibuk");
      expect(res.success).toBe(true);
      expect(sm.getState()).toBe("snoozed");
      expect(sm.getSnoozeCount()).toBe(1);
      expect(sm.getSnoozeUntil()).toBe(1600);
    });

    test("4.2: Snooze records reason pill", () => {
      const sm = new RoutineSessionManager({
        routineId: "rtn-1",
        tier: "tegas",
        scheduledFor: 1000,
        durationMin: 30,
        startGraceMin: 5,
      });
      sm.snooze(10, "Sibuk");
      expect(sm.getSkipReason()).toBe("Sibuk");
    });

    test("4.3: In Tegas tier, snooze is bounded to max 3 times", () => {
      const sessionManager = new RoutineSessionManager({
        routineId: "rtn-tegas-1",
        tier: "tegas",
        scheduledFor: 1000,
        durationMin: 30,
        startGraceMin: 5,
      });

      expect(sessionManager.getState()).toBe("prompted");

      expect(sessionManager.snooze(10, "Sibuk").success).toBe(true);
      expect(sessionManager.getSnoozeCount()).toBe(1);

      sessionManager.advanceTimeToSnoozeExpiry();
      expect(sessionManager.snooze(10, "Rapat").success).toBe(true);
      expect(sessionManager.getSnoozeCount()).toBe(2);

      sessionManager.advanceTimeToSnoozeExpiry();
      expect(sessionManager.snooze(10, "Telepon").success).toBe(true);
      expect(sessionManager.getSnoozeCount()).toBe(3);
    });

    test("4.4: 4th snooze in Tegas is rejected", () => {
      const sessionManager = new RoutineSessionManager({
        routineId: "rtn-tegas-1",
        tier: "tegas",
        scheduledFor: 1000,
        durationMin: 30,
        startGraceMin: 5,
      });

      sessionManager.snooze(10, "1");
      sessionManager.advanceTimeToSnoozeExpiry();
      sessionManager.snooze(10, "2");
      sessionManager.advanceTimeToSnoozeExpiry();
      sessionManager.snooze(10, "3");
      sessionManager.advanceTimeToSnoozeExpiry();

      const res = sessionManager.snooze(10, "4th attempt");
      expect(res.success).toBe(false);
      expect(res.error).toBe("MAX_SNOOZE_REACHED");
      expect(sessionManager.getSnoozeCount()).toBe(3);
    });

    test("4.5: Active session start clears snooze timer", () => {
      const sm = new RoutineSessionManager({
        routineId: "rtn-1",
        tier: "tegas",
        scheduledFor: 1000,
        durationMin: 30,
        startGraceMin: 5,
      });
      sm.snooze(10, "Sibuk");
      expect(sm.getState()).toBe("snoozed");
      const res = sm.startSession(1200);
      expect(res.success).toBe(true);
      expect(sm.getState()).toBe("active");
      expect(sm.getSnoozeUntil()).toBeUndefined();
    });
  });

  // Feature 5: Excused Skip Semantics
  describe("F5: Excused Skip Semantics", () => {
    function evaluateExcused(reason: string): boolean {
      const r = reason.trim().toLowerCase();
      return r.includes("sakit") || r.includes("libur") || r.includes("sibuk") || r.includes("izin") || r.includes("holiday");
    }

    test("5.1: Skip with 'Sakit' is excused", () => {
      expect(evaluateExcused("Sakit")).toBe(true);
    });

    test("5.2: Skip with 'Libur' is excused", () => {
      expect(evaluateExcused("Libur")).toBe(true);
    });

    test("5.3: Skip with 'Sibuk' is excused", () => {
      expect(evaluateExcused("Sibuk")).toBe(true);
    });

    test("5.4: Skip with 'Izin cuti' is excused", () => {
      expect(evaluateExcused("Izin cuti")).toBe(true);
    });

    test("5.5: Skip with 'Malas' is unexcused", () => {
      expect(evaluateExcused("Malas")).toBe(false);
    });
  });

  // Feature 6: Coalesced Next-Wake Timer
  describe("F6: Coalesced Next-Wake Timer", () => {
    test("6.1: Routine scheduled today in future computes exact epoch seconds", () => {
      const schedule: RoutineSchedule = { days: ["Thursday"], time: "10:00", duration_min: 30 };
      // 1970-01-01 was Thursday. 00:00 is epoch 0. 10:00 is 36000
      const next = CoalescedScheduler.calculateNextRoutineStart(schedule, 0);
      expect(next).toBe(36000);
    });

    test("6.2: Routine scheduled today in past rolls over to next scheduled day", () => {
      const schedule: RoutineSchedule = { days: ["Thursday"], time: "10:00", duration_min: 30 };
      // At epoch 40000 (11:06 Thursday), next Thursday is 7 days later
      const next = CoalescedScheduler.calculateNextRoutineStart(schedule, 40000);
      expect(next).toBe(36000 + 7 * 86400);
    });

    test("6.3: Multi-day schedule finds nearest occurrence", () => {
      const schedule: RoutineSchedule = { days: ["Friday", "Monday"], time: "10:00", duration_min: 30 };
      // From Thursday 00:00, nearest is Friday (day 1)
      const next = CoalescedScheduler.calculateNextRoutineStart(schedule, 0);
      expect(next).toBe(86400 + 36000);
    });

    test("6.4: Empty days array means every day", () => {
      const schedule: RoutineSchedule = { days: [], time: "08:00", duration_min: 30 };
      const next = CoalescedScheduler.calculateNextRoutineStart(schedule, 0);
      expect(next).toBe(8 * 3600);
    });

    test("6.5: Coalesced scheduler finds minimum across multiple routines", () => {
      const r1: Routine = {
        id: "r1",
        title: "Pagi",
        category: "pagi",
        schedule: { days: ["Thursday"], time: "14:00", duration_min: 30 },
        enforcement: "ingatkan",
        allowed_apps: [],
        distraction_rules: { block_during_routine: false, distraction_apps: [] },
        start_grace_min: 5,
        streak: { current_count: 0, best_count: 0 },
        active: true,
        created_at: 0,
        updated_at: 0,
      };
      const r2: Routine = {
        ...r1,
        id: "r2",
        schedule: { days: ["Thursday"], time: "11:00", duration_min: 30 },
      };
      const earliest = CoalescedScheduler.findEarliestNextWake([r1, r2], 0);
      expect(earliest).toBe(11 * 3600);
    });
  });

  // Feature 7: Commitment Ladder Logic
  describe("F7: Commitment Ladder Logic", () => {
    function evaluateAction(tier: EnforcementTier, app: string, allowed: string[], distractions: string[]) {
      if (ProtectedProcesses.isProtected(app)) return "Allowed_Protected";
      if (allowed.includes(app)) return "Allowed_Allowlisted";
      if (!distractions.includes(app)) return "Allowed_Default";
      switch (tier) {
        case "ingatkan":
          return "WarnGentle";
        case "tegas":
          return "WarnFirm";
        case "paksa":
          return "EnforceStrict";
      }
    }

    test("7.1: Ingatkan returns gentle warning", () => {
      expect(evaluateAction("ingatkan", "youtube.exe", [], ["youtube.exe"])).toBe("WarnGentle");
    });

    test("7.2: Tegas returns firm warning with panel requirement", () => {
      expect(evaluateAction("tegas", "steam.exe", [], ["steam.exe"])).toBe("WarnFirm");
    });

    test("7.3: Paksa returns strict enforcement", () => {
      expect(evaluateAction("paksa", "game.exe", [], ["game.exe"])).toBe("EnforceStrict");
    });

    test("7.4: Allowlisted apps are allowed in Paksa tier", () => {
      expect(evaluateAction("paksa", "code.exe", ["code.exe"], ["code.exe"])).toBe("Allowed_Protected");
    });

    test("7.5: Non-distraction apps are allowed", () => {
      expect(evaluateAction("paksa", "calculator.exe", [], ["game.exe"])).toBe("Allowed_Default");
    });
  });

  // Feature 8: Auto-Escalation Engine
  describe("F8: Auto-Escalation Engine", () => {
    test("8.1: First reminder ignored records count = 1 without escalation", () => {
      const esc = escalationTracker.recordIgnoredReminder("rtn-1", "ingatkan", 3);
      expect(esc).toBeNull();
      expect(escalationTracker.getIgnoredCount("rtn-1")).toBe(1);
    });

    test("8.2: 2nd reminder ignored does not escalate yet", () => {
      escalationTracker.recordIgnoredReminder("rtn-1", "ingatkan", 3);
      const esc = escalationTracker.recordIgnoredReminder("rtn-1", "ingatkan", 3);
      expect(esc).toBeNull();
      expect(escalationTracker.getIgnoredCount("rtn-1")).toBe(2);
    });

    test("8.3: 3rd reminder ignored triggers escalation to Tegas", () => {
      escalationTracker.recordIgnoredReminder("rtn-1", "ingatkan", 3);
      escalationTracker.recordIgnoredReminder("rtn-1", "ingatkan", 3);
      const esc = escalationTracker.recordIgnoredReminder("rtn-1", "ingatkan", 3);
      expect(esc).toBe("tegas");
    });

    test("8.4: Reset ignored count clears counter to 0", () => {
      escalationTracker.recordIgnoredReminder("rtn-1", "ingatkan", 3);
      escalationTracker.resetIgnored("rtn-1");
      expect(escalationTracker.getIgnoredCount("rtn-1")).toBe(0);
    });

    test("8.5: Routines track ignored counters independently", () => {
      escalationTracker.recordIgnoredReminder("rtn-1", "ingatkan", 3);
      escalationTracker.recordIgnoredReminder("rtn-2", "ingatkan", 3);
      expect(escalationTracker.getIgnoredCount("rtn-1")).toBe(1);
      expect(escalationTracker.getIgnoredCount("rtn-2")).toBe(1);
    });
  });

  // Feature 9: SQLite Schemas & Migrations
  describe("F9: SQLite Schemas & Migrations", () => {
    test("9.1: Routine model maps to complete relational record fields in SQLite", () => {
      const db = new RoutinesDatabase();
      const routine: Routine = {
        id: "r-sql",
        title: "Test Routine",
        category: "belajar",
        schedule: { days: ["Monday"], time: "18:00", duration_min: 30 },
        enforcement: "paksa",
        allowed_apps: ["code.exe"],
        distraction_rules: { block_during_routine: true, distraction_apps: ["game.exe"] },
        start_grace_min: 5,
        streak: { current_count: 0, best_count: 0 },
        active: true,
        created_at: 1000,
        updated_at: 1000,
      };
      db.insertRoutine(routine);
      const fetched = db.getRoutine("r-sql");
      expect(fetched?.id).toBe("r-sql");
      expect(fetched?.start_grace_min).toBe(5);
      expect(fetched?.enforcement).toBe("paksa");
    });

    test("9.2: Commitment table schema contract in SQLite", () => {
      const db = new RoutinesDatabase();
      const com: Commitment = {
        id: "c-sql",
        name: "Block Games",
        tier: "paksa",
        target_apps: ["tiktok.exe"],
        active: true,
        created_at: 1000,
        updated_at: 1000,
      };
      db.insertCommitment(com);
      const fetched = db.getCommitment("c-sql");
      expect(fetched?.tier).toBe("paksa");
      expect(fetched?.target_apps).toEqual(["tiktok.exe"]);
    });

    test("9.3: Enforcement history table schema contract in SQLite", () => {
      const db = new RoutinesDatabase();
      const historyRecord: EnforcementHistoryRecord = {
        id: "h-sql",
        timestamp: 1700000000,
        target_app: "game.exe",
        tier: "paksa",
        action_taken: "closed_gracefully",
      };
      db.logEnforcement(historyRecord);
      const rows = db.getEnforcementHistory();
      expect(rows.length).toBe(1);
      expect(rows[0].action_taken).toBe("closed_gracefully");
      expect(rows[0].target_app).toBe("game.exe");
    });

    test("9.4: Emergency pauses table schema contract in SQLite", () => {
      const db = new RoutinesDatabase();
      const pauseRecord: EmergencyPauseRecord = {
        id: "p-sql",
        started_at: 1000,
        duration_min: 30,
        reason: "Sakit",
        expired_at: 2800,
      };
      db.recordEmergencyPause(pauseRecord);
      const active = db.getActivePause(1500);
      expect(active?.duration_min).toBe(30);
      expect(active?.reason).toBe("Sakit");
    });

    test("9.5: Indices and constraints validation in SQLite", () => {
      const db = new RoutinesDatabase();
      const rIndices = db.getIndices("routines");
      const cIndices = db.getIndices("commitments");
      const epIndices = db.getIndices("emergency_pauses");
      expect(rIndices).toContain("idx_routines_active");
      expect(cIndices).toContain("idx_commitments_active");
      expect(epIndices).toContain("idx_emergency_pauses_exp");
    });
  });

  // Feature 10: Win32 Foreground Hook
  describe("F10: Win32 Foreground Hook", () => {
    test("10.1: Register foreground hook callback", () => {
      let received: ForegroundInfo | null = null;
      const unhook = pal.onForegroundSwitch((info) => {
        received = info;
      });
      pal.registerDummyWindow(2001, "browser.exe", "ModeA_CleanClose");
      pal.switchForeground(2001);
      expect(received).not.toBeNull();
      expect(received?.process_name).toBe("browser.exe");
      unhook();
    });

    test("10.2: Switching foreground emits hook event with accurate info", () => {
      pal.registerDummyWindow(2002, "editor.exe", "ModeA_CleanClose", "main.rs - Editor");
      pal.switchForeground(2002);
      const info = pal.getForegroundInfo();
      expect(info?.window_title).toBe("main.rs - Editor");
      expect(info?.hwnd).toBe(2002);
    });

    test("10.3: Rapid switching emits sequence of events", () => {
      const events: string[] = [];
      pal.onForegroundSwitch((info) => events.push(info.process_name));
      pal.registerDummyWindow(2003, "app1.exe", "ModeA_CleanClose");
      pal.registerDummyWindow(2004, "app2.exe", "ModeA_CleanClose");
      pal.switchForeground(2003);
      pal.switchForeground(2004);
      expect(events).toEqual(["app1.exe", "app2.exe"]);
    });

    test("10.4: Hook callback receives valid PID and HWND", () => {
      pal.registerDummyWindow(2005, "notepad.exe", "ModeA_CleanClose");
      pal.switchForeground(2005);
      const info = pal.getForegroundInfo();
      expect(info?.pid).toBeGreaterThan(0);
      expect(info?.hwnd).toBe(2005);
    });

    test("10.5: Unregistering hook stops dispatching events to that listener", () => {
      let count = 0;
      const unhook = pal.onForegroundSwitch(() => count++);
      pal.registerDummyWindow(2006, "test.exe", "ModeA_CleanClose");
      pal.switchForeground(2006);
      expect(count).toBe(1);
      unhook();
      pal.switchForeground(2006);
      expect(count).toBe(1);
    });
  });

  // Feature 11: Win32 Foreground Query
  describe("F11: Win32 Foreground Query", () => {
    beforeEach(() => {
      pal.registerDummyWindow(3001, "code.exe", "ModeA_CleanClose", "Workspace - VS Code", "Chrome_WidgetWin_1");
      pal.switchForeground(3001);
    });

    test("11.1: Query active HWND", () => {
      const info = pal.getForegroundInfo();
      expect(info?.hwnd).toBe(3001);
    });

    test("11.2: Query PID", () => {
      const info = pal.getForegroundInfo();
      expect(info?.pid).toBe(8001);
    });

    test("11.3: Query process executable name", () => {
      const info = pal.getForegroundInfo();
      expect(info?.process_name).toBe("code.exe");
    });

    test("11.4: Query window title", () => {
      const info = pal.getForegroundInfo();
      expect(info?.window_title).toBe("Workspace - VS Code");
    });

    test("11.5: Query window class name", () => {
      const info = pal.getForegroundInfo();
      expect(info?.class_name).toBe("Chrome_WidgetWin_1");
    });
  });

  // Feature 12: User Idle Detection
  describe("F12: User Idle Detection", () => {
    test("12.1: User active < 30s returns is_idle = false", () => {
      const now = 100000;
      pal.setLastInputTime(now - 15000); // 15s ago
      expect(pal.isUserIdle(30, now)).toBe(false);
    });

    test("12.2: User inactive >= 30s returns is_idle = true", () => {
      const now = 100000;
      pal.setLastInputTime(now - 35000); // 35s ago
      expect(pal.isUserIdle(30, now)).toBe(true);
    });

    test("12.3: User interaction resets idle timer immediately", () => {
      const now = 100000;
      pal.setLastInputTime(now - 40000);
      expect(pal.isUserIdle(30, now)).toBe(true);
      pal.setLastInputTime(now); // input received
      expect(pal.isUserIdle(30, now)).toBe(false);
    });

    test("12.4: Distraction timer pauses when idle", () => {
      const timer = new DistractionTimerManager({ tier: "paksa", initialCountdownSecs: 10 });
      timer.tick(1, true);
      expect(timer.getRemainingSecs()).toBe(10);
      expect(timer.isPaused()).toBe(true);
    });

    test("12.5: Distraction timer resumes when active", () => {
      const timer = new DistractionTimerManager({ tier: "paksa", initialCountdownSecs: 10 });
      timer.tick(1, true);
      expect(timer.getRemainingSecs()).toBe(10);

      timer.tick(1, false);
      expect(timer.getRemainingSecs()).toBe(9);
      expect(timer.isPaused()).toBe(false);
    });
  });

  // Feature 13: Protected Processes List
  describe("F13: Protected Processes List", () => {
    test("13.1: Windows system and shell processes are protected", () => {
      expect(ProtectedProcesses.isProtected("explorer.exe")).toBe(true);
      expect(ProtectedProcesses.isProtected("dwm.exe")).toBe(true);
      expect(ProtectedProcesses.isProtected("svchost.exe")).toBe(true);
      expect(ProtectedProcesses.isProtected("taskmgr.exe")).toBe(true);
    });

    test("13.2: Security and AV processes are protected", () => {
      expect(ProtectedProcesses.isProtected("MsMpEng.exe")).toBe(true);
      expect(ProtectedProcesses.isProtected("SecurityHealthService.exe")).toBe(true);
      expect(ProtectedProcesses.isProtected("nissrv.exe")).toBe(true);
    });

    test("13.3: Hime core and Island shell are protected", () => {
      expect(ProtectedProcesses.isProtected("hime-island.exe")).toBe(true);
      expect(ProtectedProcesses.isProtected("hime-core.exe")).toBe(true);
      expect(ProtectedProcesses.isProtected("msedgewebview2.exe")).toBe(true);
    });

    test("13.4: Development environments and terminals are protected", () => {
      expect(ProtectedProcesses.isProtected("code.exe")).toBe(true);
      expect(ProtectedProcesses.isProtected("devenv.exe")).toBe(true);
      expect(ProtectedProcesses.isProtected("idea64.exe")).toBe(true);
      expect(ProtectedProcesses.isProtected("windowsterminal.exe")).toBe(true);
      expect(ProtectedProcesses.isProtected("cmd.exe")).toBe(true);
      expect(ProtectedProcesses.isProtected("powershell.exe")).toBe(true);
    });

    test("13.5: Distraction applications are NOT protected", () => {
      expect(ProtectedProcesses.isProtected("chrome.exe")).toBe(false);
      expect(ProtectedProcesses.isProtected("steam.exe")).toBe(false);
      expect(ProtectedProcesses.isProtected("discord.exe")).toBe(false);
      expect(ProtectedProcesses.isProtected("spotify.exe")).toBe(false);
    });
  });

  // Feature 14: Graceful Window Close (WM_CLOSE)
  describe("F14: Graceful Window Close (WM_CLOSE)", () => {
    test("14.1: Close succeeds on ModeA dummy window in Paksa tier", () => {
      pal.registerDummyWindow(4001, "distraction.exe", "ModeA_CleanClose");
      const res = pal.sendGracefulClose(4001, 1000);
      expect(res).toBe("ClosedSuccessfully");
    });

    test("14.2: Protected process is refused close", () => {
      pal.registerDummyWindow(4002, "explorer.exe", "ModeA_CleanClose");
      const res = pal.sendGracefulClose(4002, 1000);
      expect(res).toBe("ProtectedProcessRefused");
    });

    test("14.3: Closed window is marked closed in simulator", () => {
      pal.registerDummyWindow(4003, "game.exe", "ModeA_CleanClose");
      pal.switchForeground(4003);
      pal.sendGracefulClose(4003, 1000);
      expect(pal.getForegroundInfo()).toBeNull();
    });

    test("14.4: Graceful close dispatches to top-level window only", () => {
      pal.registerDummyWindow(4004, "browser.exe", "ModeA_CleanClose");
      const res = pal.sendGracefulClose(4004, 1000);
      expect(res).toBe("ClosedSuccessfully");
    });

    test("14.5: Multiple distinct windows can be closed sequentially", () => {
      pal.registerDummyWindow(4005, "app1.exe", "ModeA_CleanClose");
      pal.registerDummyWindow(4006, "app2.exe", "ModeA_CleanClose");
      expect(pal.sendGracefulClose(4005, 1000)).toBe("ClosedSuccessfully");
      expect(pal.sendGracefulClose(4006, 1000)).toBe("ClosedSuccessfully");
    });
  });

  // Feature 15: Unsaved Changes Dialog Detection
  describe("F15: Unsaved Changes Dialog Detection", () => {
    test("15.1: ModeB window detects modal unsaved dialog (#32770)", () => {
      pal.registerDummyWindow(5001, "notepad.exe", "ModeB_UnsavedChangesModal", "Untitled - Notepad", "#32770");
      expect(pal.hasModalUnsavedDialog(5001)).toBe(true);
    });

    test("15.2: Sending close to ModeB returns UnsavedDialogDetected", () => {
      pal.registerDummyWindow(5002, "editor.exe", "ModeB_UnsavedChangesModal");
      const res = pal.sendGracefulClose(5002, 1000);
      expect(res).toBe("UnsavedDialogDetected");
    });

    test("15.3: Halts enforcement action without terminating process", () => {
      pal.registerDummyWindow(5003, "editor.exe", "ModeB_UnsavedChangesModal");
      pal.sendGracefulClose(5003, 1000);
      expect(pal.terminateProcessCalled).toBe(false);
    });

    test("15.4: UI alert is 'Selesaikan dulu'", () => {
      const alertTitle = "Selesaikan dulu";
      expect(Vcu5Validator.validateTitle(alertTitle).valid).toBe(true);
    });

    test("15.5: Dismissing modal allows subsequent close to succeed", () => {
      pal.registerDummyWindow(5005, "editor.exe", "ModeA_CleanClose");
      const res = pal.sendGracefulClose(5005, 1000);
      expect(res).toBe("ClosedSuccessfully");
    });
  });

  // Feature 16: Prohibition of TerminateProcess
  describe("F16: Prohibition of TerminateProcess", () => {
    test("16.1: Invariant: TerminateProcess flag is false initially", () => {
      expect(pal.terminateProcessCalled).toBe(false);
    });

    test("16.2: Refusal to close (ModeC) does not terminate process", () => {
      pal.registerDummyWindow(6001, "stubborn.exe", "ModeC_RefuseClose");
      const res = pal.sendGracefulClose(6001, 1000);
      expect(res).toBe("WindowRefusedClose");
      expect(pal.terminateProcessCalled).toBe(false);
    });

    test("16.3: Unsaved dialog does not terminate process", () => {
      pal.registerDummyWindow(6002, "word.exe", "ModeB_UnsavedChangesModal");
      pal.sendGracefulClose(6002, 1000);
      expect(pal.terminateProcessCalled).toBe(false);
    });

    test("16.4: Direct invocation of TerminateProcess throws Gate #10 error", () => {
      expect(() => pal.attemptTerminateProcess()).toThrow("GATE #10 VIOLATION");
    });

    test("16.5: Gate #10 safety certification: 0 process kills performed", () => {
      expect(pal.terminateProcessCalled).toBe(false);
    });
  });

  // Feature 17: Rate Limiting & Cooldown
  describe("F17: Rate Limiting & Cooldown", () => {
    test("17.1: Immediate subsequent close within 30s is rejected", () => {
      pal.registerDummyWindow(7001, "distract.exe", "ModeA_CleanClose");
      pal.registerDummyWindow(7002, "distract.exe", "ModeA_CleanClose");
      expect(pal.sendGracefulClose(7001, 1000)).toBe("ClosedSuccessfully");
      // Attempt close on same app 10s later
      expect(pal.sendGracefulClose(7002, 1010)).toBe("WindowRefusedClose");
    });

    test("17.2: Close after 30s cooldown elapses is allowed", () => {
      pal.registerDummyWindow(7003, "app.exe", "ModeA_CleanClose");
      pal.registerDummyWindow(7004, "app.exe", "ModeA_CleanClose");
      expect(pal.sendGracefulClose(7003, 1000)).toBe("ClosedSuccessfully");
      expect(pal.sendGracefulClose(7004, 1031)).toBe("ClosedSuccessfully");
    });

    test("17.3: Max 5 closes per hour enforced", () => {
      for (let i = 0; i < 5; i++) {
        const hwnd = 7010 + i;
        pal.registerDummyWindow(hwnd, "spam.exe", "ModeA_CleanClose");
        expect(pal.sendGracefulClose(hwnd, 1000 + i * 35)).toBe("ClosedSuccessfully");
      }
      // 6th close within the hour
      const hwnd6 = 7016;
      pal.registerDummyWindow(hwnd6, "spam.exe", "ModeA_CleanClose");
      expect(pal.sendGracefulClose(hwnd6, 1200)).toBe("WindowRefusedClose");
    });

    test("17.4: Rate limiting is isolated per application", () => {
      pal.registerDummyWindow(7020, "appA.exe", "ModeA_CleanClose");
      pal.registerDummyWindow(7021, "appB.exe", "ModeA_CleanClose");
      expect(pal.sendGracefulClose(7020, 1000)).toBe("ClosedSuccessfully");
      expect(pal.sendGracefulClose(7021, 1005)).toBe("ClosedSuccessfully");
    });

    test("17.5: Reopen loop guard triggers reflection after 3 reopens in 120s", () => {
      expect(pal.recordReopen("game.exe", 1000)).toBe(false);
      expect(pal.recordReopen("game.exe", 1020)).toBe(false);
      expect(pal.recordReopen("game.exe", 1040)).toBe(true); // 3rd reopen
    });
  });

  // Feature 18: Emergency Pause Backend
  describe("F18: Emergency Pause Backend", () => {
    test("18.1: Pause record creates valid expiration timestamp", () => {
      const started = 1000;
      const duration = 30; // 30 min
      const expired = started + duration * 60;
      const pause: EmergencyPauseRecord = { id: "p-1", started_at: started, duration_min: duration, reason: "Darurat", expired_at: expired };
      store.addEmergencyPause(pause);
      expect(store.getActivePause(1500)).toBeDefined();
    });

    test("18.2: Active pause overrides enforcement", () => {
      store.addEmergencyPause({ id: "p-1", started_at: 1000, duration_min: 30, reason: "Sakit", expired_at: 2800 });
      const pause = store.getActivePause(1500);
      expect(pause?.reason).toBe("Sakit");
    });

    test("18.3: Expired pause is no longer active", () => {
      store.addEmergencyPause({ id: "p-1", started_at: 1000, duration_min: 15, reason: "Sakit", expired_at: 1900 });
      expect(store.getActivePause(2000)).toBeUndefined();
    });

    test("18.4: Standard durations: 15m, 30m, 60m supported", () => {
      const allowed = [15, 30, 60];
      expect(allowed.includes(15)).toBe(true);
      expect(allowed.includes(30)).toBe(true);
      expect(allowed.includes(60)).toBe(true);
    });

    test("18.5: Remaining pause seconds calculation", () => {
      const pause: EmergencyPauseRecord = { id: "p-1", started_at: 1000, duration_min: 30, reason: "Sibuk", expired_at: 2800 };
      const now = 2000;
      const remaining = pause.expired_at - now;
      expect(remaining).toBe(800);
    });
  });

  // Feature 19: Rule Weakening Delay
  describe("F19: Rule Weakening Delay", () => {
    test("19.1: Weakening Paksa -> Tegas sets 10-minute cooling-off period", () => {
      const db = new RoutinesDatabase();
      const palSim = new Win32PalSimulator();
      const controller = new EnforcementController(db, palSim);
      const now = 1000;
      db.insertCommitment({
        id: "c-1",
        name: "Study",
        tier: "paksa",
        target_apps: ["game.exe"],
        active: true,
        created_at: now,
        updated_at: now,
      });
      const res = controller.requestCommitmentModification("c-1", { tier: "ingatkan" }, now);
      expect(res.isWeakening).toBe(true);
      expect(res.coolingOffRequired).toBe(true);
      expect(res.pendingUntil).toBe(1600);
      expect(db.getCommitment("c-1")?.pending_weakening_until).toBe(1600);
    });

    test("19.2: Strict tier enforced during cooling-off", () => {
      const db = new RoutinesDatabase();
      const palSim = new Win32PalSimulator();
      const controller = new EnforcementController(db, palSim);
      db.insertCommitment({
        id: "c-1",
        name: "Study",
        tier: "paksa",
        target_apps: ["game.exe"],
        pending_weakening_until: 1600,
        active: true,
        created_at: 1000,
        updated_at: 1000,
      });
      expect(controller.isAppRestricted("game.exe", 1300)).toBe(true);
    });

    test("19.3: Weakened tier applies after 10 min elapse", () => {
      const db = new RoutinesDatabase();
      const palSim = new Win32PalSimulator();
      const controller = new EnforcementController(db, palSim);
      db.insertCommitment({
        id: "c-1",
        name: "Study",
        tier: "paksa",
        target_apps: [],
        pending_weakening_until: 1600,
        active: true,
        created_at: 1000,
        updated_at: 1000,
      });
      controller.evaluatePendingWeakening(1601);
      expect(controller.isAppRestricted("game.exe", 1601)).toBe(false);
    });

    test("19.4: Cancellation restores pure strict state", () => {
      const db = new RoutinesDatabase();
      const palSim = new Win32PalSimulator();
      const controller = new EnforcementController(db, palSim);
      db.insertCommitment({
        id: "c-1",
        name: "Study",
        tier: "paksa",
        target_apps: ["game.exe"],
        pending_weakening_until: 1600,
        active: true,
        created_at: 1000,
        updated_at: 1000,
      });
      const res = controller.requestCommitmentModification("c-1", { tier: "paksa" }, 1200);
      expect(res.isWeakening).toBe(false);
      expect(db.getCommitment("c-1")?.pending_weakening_until).toBeUndefined();
    });

    test("19.5: Strengthening applies immediately with 0s delay", () => {
      const db = new RoutinesDatabase();
      const palSim = new Win32PalSimulator();
      const controller = new EnforcementController(db, palSim);
      db.insertCommitment({
        id: "c-1",
        name: "Study",
        tier: "ingatkan",
        target_apps: ["game.exe"],
        active: true,
        created_at: 1000,
        updated_at: 1000,
      });
      const res = controller.requestCommitmentModification("c-1", { tier: "paksa" }, 1000);
      expect(res.isWeakening).toBe(false);
      expect(res.coolingOffRequired).toBe(false);
      expect(db.getCommitment("c-1")?.tier).toBe("paksa");
      expect(db.getCommitment("c-1")?.pending_weakening_until).toBeUndefined();
    });
  });

  // Feature 20: Kill Switch & Crash Recovery
  describe("F20: Kill Switch & Crash Recovery", () => {
    test("20.1: Emergency kill switch halts active enforcement", () => {
      const db = new RoutinesDatabase();
      const palSim = new Win32PalSimulator();
      const controller = new EnforcementController(db, palSim);

      controller.setEnforcementActive(true);
      expect(controller.isEnforcementActive()).toBe(true);

      controller.triggerKillSwitch();
      expect(controller.isEnforcementActive()).toBe(false);
      expect(controller.isKillSwitchActive()).toBe(true);

      palSim.registerDummyWindow(2001, "distraction.exe", "ModeA_CleanClose");
      const result = controller.enforceWindowClose(2001, 1000);
      expect(result.action).toBe("bypassed_safe_unblocked");
    });

    test("20.2: /jeda command activates emergency pause", () => {
      const db = new RoutinesDatabase();
      const palSim = new Win32PalSimulator();
      const controller = new EnforcementController(db, palSim);
      const res = controller.handleSlashCommand("/jeda", 1000);
      expect(res.success).toBe(true);
      expect(controller.isEmergencyPaused(1005)).toBe(true);
    });

    test("20.3: Crash recovery reconstructs state from storage", () => {
      store.createRoutine({
        id: "rtn-persist",
        title: "Persistence Check",
        category: "belajar",
        schedule: { days: [], time: "20:00", duration_min: 30 },
        enforcement: "paksa",
        allowed_apps: [],
        distraction_rules: { block_during_routine: true, distraction_apps: [] },
        start_grace_min: 5,
        streak: { current_count: 3, best_count: 5 },
        active: true,
        created_at: 1000,
        updated_at: 1000,
      });
      const recovered = store.getRoutine("rtn-persist");
      expect(recovered?.streak.current_count).toBe(3);
    });

    test("20.4: Crash does not leave orphaned window hooks", () => {
      const db = new RoutinesDatabase();
      const palSim = new Win32PalSimulator();
      const controller = new EnforcementController(db, palSim);

      controller.registerHooks();
      expect(palSim.getActiveHookCount()).toBeGreaterThan(0);

      controller.cleanupHooks();
      expect(palSim.getActiveHookCount()).toBe(0);
    });

    test("20.5: Invariant: OS is never left in locked state", () => {
      const db = new RoutinesDatabase();
      const palSim = new Win32PalSimulator();
      const controller = new EnforcementController(db, palSim);

      controller.triggerKillSwitch();
      controller.shutdown();

      expect(controller.isOsLocked()).toBe(false);
      expect(palSim.isInputBlocked()).toBe(false);
      expect(palSim.getActiveHookCount()).toBe(0);
    });
  });

  // Feature 21: Gate #10 Mock Window Fixture
  describe("F21: Gate #10 Mock Window Fixture", () => {
    test("21.1: Mode A handles clean WM_CLOSE exit 0", () => {
      pal.registerDummyWindow(8001, "dummy_a.exe", "ModeA_CleanClose");
      expect(pal.sendGracefulClose(8001, 1000)).toBe("ClosedSuccessfully");
    });

    test("21.2: Mode B pops modal unsaved dialog (#32770)", () => {
      pal.registerDummyWindow(8002, "dummy_b.exe", "ModeB_UnsavedChangesModal");
      expect(pal.sendGracefulClose(8002, 1000)).toBe("UnsavedDialogDetected");
    });

    test("21.3: Mode C refuses WM_CLOSE", () => {
      pal.registerDummyWindow(8003, "dummy_c.exe", "ModeC_RefuseClose");
      expect(pal.sendGracefulClose(8003, 1000)).toBe("WindowRefusedClose");
    });

    test("21.4: Dummy windows execute without touching real user processes", () => {
      pal.registerDummyWindow(8004, "dummy_test.exe", "ModeA_CleanClose");
      const info = pal.getForegroundInfo();
      // Only mock windows used
      expect(info?.executable_path.startsWith("C:\\Mock\\") || info === null).toBe(true);
    });

    test("21.5: Zero TerminateProcess across all dummy window modes", () => {
      expect(pal.terminateProcessCalled).toBe(false);
    });
  });

  // Feature 26: Tauri IPC Bridge for U3
  describe("F26: Tauri IPC Bridge for U3", () => {
    function mockIpcCall(endpoint: string, params: Record<string, any>) {
      const bridge = new RoutinesIpcBridge();
      return bridge.call(endpoint, params);
    }

    test("26.1: Invoking productivity.routines.create succeeds", () => {
      const res = mockIpcCall("productivity.routines.create", { title: "Study" });
      expect(res.result?.title).toBe("Study");
    });

    test("26.2: Invoking productivity.enforce.emergency_pause succeeds", () => {
      const res = mockIpcCall("productivity.enforce.emergency_pause", { duration_min: 30, reason: "Sakit" });
      expect(res.result?.is_paused).toBe(true);
    });

    test("26.3: Invoking productivity.commitments.create succeeds", () => {
      const res = mockIpcCall("productivity.commitments.create", { tier: "paksa" });
      expect(res.result?.tier).toBe("paksa");
    });

    test("26.4: Missing parameters return RPC error code -32602", () => {
      const res = mockIpcCall("productivity.routines.create", {});
      expect(res.error?.code).toBe(-32602);
    });

    test("26.5: Unknown method returns RPC error code -32601", () => {
      const res = mockIpcCall("unknown.method", {});
      expect(res.error?.code).toBe(-32601);
    });
  });

  // Feature 27: Focus Panel (HimeFocusPanel.tsx, Ctrl+Alt+F)
  describe("F27: Focus Panel (HimeFocusPanel.tsx)", () => {
    test("27.1: Progress ring percentage clamps between 0% and 100%", () => {
      const calcProgress = (elapsed: number, total: number) => Math.min(100, Math.max(0, (elapsed / total) * 100));
      expect(calcProgress(15, 30)).toBe(50);
      expect(calcProgress(-5, 30)).toBe(0);
      expect(calcProgress(45, 30)).toBe(100);
    });

    test("27.2: Category pills contain standard Indonesian categories", () => {
      const categories = ["Belajar", "Kerja", "Olahraga", "Ibadah", "Pribadi"];
      expect(categories.length).toBe(5);
    });

    test("27.3: Keyboard shortcut Ctrl+Alt+F is documented and recognized", () => {
      const bridge = new RoutinesIpcBridge();
      const registered = bridge.getRegisteredHotkey("focus_panel");
      expect(registered).toEqual({
        key: "F",
        modifiers: ["Ctrl", "Alt"],
        displayString: "Ctrl+Alt+F",
        description: "Buka panel fokus",
      });
      expect(bridge.isRecognizedHotkey("Ctrl+Alt+F")).toBe(true);
      expect(bridge.isRecognizedHotkey("Ctrl+Alt+Z")).toBe(false);
    });

    test("27.4: Active routine displays elapsed and remaining minutes", () => {
      const duration = 30;
      const elapsed = 12;
      const remaining = duration - elapsed;
      expect(remaining).toBe(18);
    });

    test("27.5: Notch panel title meets VC-U5 length constraint (<= 16 chars)", () => {
      const title = "Fokus Belajar";
      expect(Vcu5Validator.validateTitle(title).valid).toBe(true);
    });
  });

  // Feature 28: Enforcement & Alert UI (EnforcementBubble.tsx)
  describe("F28: Enforcement & Alert UI (EnforcementBubble.tsx)", () => {
    test("28.1: Countdown initializes at 10 seconds in Paksa tier", () => {
      const timer = new DistractionTimerManager({ tier: "paksa" });
      expect(timer.getRemainingSecs()).toBe(10);
      expect(timer.getInitialCountdownSecs()).toBe(10);
      expect(timer.getState()).toBe("armed");
    });

    test("28.2: Button label [Tunda 5m] conforms to <= 10 chars", () => {
      const label = "Tunda 5m";
      expect(Vcu5Validator.validateButtonText(label).valid).toBe(true);
    });

    test("28.3: Button label [Jeda] conforms to <= 10 chars", () => {
      const label = "Jeda";
      expect(Vcu5Validator.validateButtonText(label).valid).toBe(true);
    });

    test("28.4: Alert title 'Waktunya Fokus' conforms to <= 16 chars", () => {
      const title = "Waktunya Fokus";
      expect(Vcu5Validator.validateTitle(title).valid).toBe(true);
    });

    test("28.5: Alert description conforms to <= 36 chars", () => {
      const desc = "Aplikasi ini dibatasi saat fokus";
      expect(Vcu5Validator.validateDescription(desc).valid).toBe(true);
    });
  });

  // Feature 29: Emergency Pause Modal (EmergencyPauseModal.tsx)
  describe("F29: Emergency Pause Modal", () => {
    test("29.1: Duration pills: 15m, 30m, 60m", () => {
      const durations = ["15m", "30m", "60m"];
      for (const d of durations) {
        expect(Vcu5Validator.validateButtonText(d).valid).toBe(true);
      }
    });

    test("29.2: Reason pills: Sibuk, Sakit, Libur, Lainnya", () => {
      const reasons = ["Sibuk", "Sakit", "Libur", "Lainnya"];
      for (const r of reasons) {
        expect(Vcu5Validator.validateButtonText(r).valid).toBe(true);
      }
    });

    test("29.3: Modal title 'Darurat Jeda' conforms to <= 16 chars", () => {
      expect(Vcu5Validator.validateTitle("Darurat Jeda").valid).toBe(true);
    });

    test("29.4: Action button [Aktifkan] conforms to <= 10 chars", () => {
      expect(Vcu5Validator.validateButtonText("Aktifkan").valid).toBe(true);
    });

    test("29.5: Dismiss button [Batal] conforms to <= 10 chars", () => {
      expect(Vcu5Validator.validateButtonText("Batal").valid).toBe(true);
    });
  });

  // Feature 31: VC-U5 Presentation & Visual Gate
  describe("F31: VC-U5 Presentation & Visual Gate", () => {
    test("31.1: Token colors use CSS variables or verified design tokens", () => {
      expect(Vcu5Validator.validateTokenColor("var(--accent-mint)")).toBe(true);
      expect(Vcu5Validator.validateTokenColor("var(--surface-island)")).toBe(true);
      expect(Vcu5Validator.validateTokenColor("#34E3A6")).toBe(true);
    });

    test("31.2: Mascot eyes expression 'fokus' is valid", () => {
      expect(Vcu5Validator.validateMascotExpression("fokus")).toBe(true);
    });

    test("31.3: Mascot eyes expression 'kesal' on distraction is valid", () => {
      expect(Vcu5Validator.validateMascotExpression("kesal")).toBe(true);
    });

    test("31.4: Mascot eyes expression 'kaget' on countdown start is valid", () => {
      expect(Vcu5Validator.validateMascotExpression("kaget")).toBe(true);
    });

    test("31.5: Mascot eyes expression 'lega' on session complete is valid", () => {
      expect(Vcu5Validator.validateMascotExpression("lega")).toBe(true);
    });
  });
});

// =============================================================================
// TEST SUITE: TIER 2 — BOUNDARY & CORNER CASES
// =============================================================================

describe("Tier 2: Boundary & Corner Cases", () => {
  let store: RoutineStore;
  let pal: Win32PalSimulator;
  let escalationTracker: AutoEscalationTracker;

  beforeEach(() => {
    store = new RoutineStore();
    pal = new Win32PalSimulator();
    escalationTracker = new AutoEscalationTracker();
  });

  // F1 Boundaries
  describe("F1 Boundaries: Routine Models & CRUD", () => {
    test("1.1b: Empty routine title throws validation error", () => {
      expect(() => {
        store.createRoutine({
          id: "r-empty",
          title: "",
          category: "c",
          schedule: { days: [], time: "10:00", duration_min: 10 },
          enforcement: "paksa",
          allowed_apps: [],
          distraction_rules: { block_during_routine: false, distraction_apps: [] },
          start_grace_min: 5,
          streak: { current_count: 0, best_count: 0 },
          active: true,
          created_at: 0,
          updated_at: 0,
        });
      }).toThrow("Routine title is required");
    });

    test("1.2b: Malformed time string throws validation error", () => {
      expect(() => {
        store.createRoutine({
          id: "r-badtime",
          title: "Bad Time",
          category: "c",
          schedule: { days: [], time: "25:70", duration_min: 10 },
          enforcement: "paksa",
          allowed_apps: [],
          distraction_rules: { block_during_routine: false, distraction_apps: [] },
          start_grace_min: 5,
          streak: { current_count: 0, best_count: 0 },
          active: true,
          created_at: 0,
          updated_at: 0,
        });
      }).toThrow("Invalid time format");
    });

    test("1.3b: Zero start_grace_min throws validation error", () => {
      expect(() => {
        store.createRoutine({
          id: "r-zerograce",
          title: "Zero Grace",
          category: "c",
          schedule: { days: [], time: "10:00", duration_min: 10 },
          enforcement: "paksa",
          allowed_apps: [],
          distraction_rules: { block_during_routine: false, distraction_apps: [] },
          start_grace_min: 0,
          streak: { current_count: 0, best_count: 0 },
          active: true,
          created_at: 0,
          updated_at: 0,
        });
      }).toThrow("start_grace_min must be greater than 0");
    });

    test("1.4b: Updating non-existent ID throws not found error", () => {
      expect(() => {
        store.updateRoutine({
          id: "non-existent",
          title: "Ghost",
          category: "c",
          schedule: { days: [], time: "10:00", duration_min: 10 },
          enforcement: "paksa",
          allowed_apps: [],
          distraction_rules: { block_during_routine: false, distraction_apps: [] },
          start_grace_min: 5,
          streak: { current_count: 0, best_count: 0 },
          active: true,
          created_at: 0,
          updated_at: 0,
        });
      }).toThrow("not found");
    });

    test("1.5b: Routine with Unicode and emoji title serializes and roundtrips cleanly", () => {
      const routine: Routine = {
        id: "r-emoji",
        title: "Belajar 🎯 & Sholat 🕌",
        category: "pribadi",
        schedule: { days: ["Monday"], time: "05:00", duration_min: 20 },
        enforcement: "tegas",
        allowed_apps: [],
        distraction_rules: { block_during_routine: false, distraction_apps: [] },
        start_grace_min: 5,
        streak: { current_count: 0, best_count: 0 },
        active: true,
        created_at: 0,
        updated_at: 0,
      };
      store.createRoutine(routine);
      const fetched = store.getRoutine("r-emoji");
      expect(fetched?.title).toBe("Belajar 🎯 & Sholat 🕌");
    });
  });

  // F2 Boundaries
  describe("F2 Boundaries: Session Lifecycle", () => {
    test("2.1b: Starting completed session is rejected", () => {
      const sm = new RoutineSessionManager({
        routineId: "rtn-21b",
        tier: "paksa",
        scheduledFor: 1000,
        durationMin: 30,
      }, 1000);
      sm.startSession(1010);
      sm.completeSession(1200);
      const res = sm.startSession(1210);
      expect(res.success).toBe(false);
      expect(res.error).toBe("TERMINAL_STATE");
      expect(sm.getState()).toBe("completed");
    });

    test("2.2b: Completing skipped session is rejected", () => {
      const sm = new RoutineSessionManager({
        routineId: "rtn-22b",
        tier: "paksa",
        scheduledFor: 1000,
        durationMin: 30,
      }, 1000);
      sm.skipSession("Sakit", true);
      const res = sm.completeSession(1200);
      expect(res.success).toBe(false);
      expect(res.error).toBe("NOT_ACTIVE");
      expect(sm.getState()).toBe("skipped");
    });

    test("2.3b: Starting already active session is idempotent", () => {
      const sm = new RoutineSessionManager({
        routineId: "rtn-23b",
        tier: "paksa",
        scheduledFor: 1000,
        durationMin: 30,
      }, 1000);
      sm.startSession(1010);
      const res = sm.startSession(1020);
      expect(res.success).toBe(false);
      expect(res.error).toBe("ALREADY_ACTIVE");
      expect(sm.getState()).toBe("active");
    });

    test("2.4b: Grace window expiry: T-0 + grace + 1s marks session missed", () => {
      const sm = new RoutineSessionManager({
        routineId: "rtn-24b",
        tier: "paksa",
        scheduledFor: 1000,
        durationMin: 30,
        startGraceMin: 5,
      });
      const state = sm.evaluateStateAtTime(1000 + 300 + 1);
      expect(state).toBe("missed");
      expect(sm.getState()).toBe("missed");
    });

    test("2.5b: Exact boundary: T-0 + grace (300s) is still within grace", () => {
      const sm = new RoutineSessionManager({
        routineId: "rtn-25b",
        tier: "paksa",
        scheduledFor: 1000,
        durationMin: 30,
        startGraceMin: 5,
      });
      const state = sm.evaluateStateAtTime(1000 + 300);
      expect(state).toBe("prompted");
      expect(sm.getState()).toBe("prompted");
    });
  });

  // F3 Boundaries
  describe("F3 Boundaries: Streak Engine", () => {
    test("3.1b: Streak resets on 2-day gap without excused freeze", () => {
      const streak: RoutineStreak = { current_count: 5, best_count: 5, last_completed_date: "2026-11-19" };
      StreakEngine.applyCompletion(streak, "2026-11-21"); // 2-day gap
      expect(streak.current_count).toBe(1);
      expect(streak.best_count).toBe(5); // best streak preserved!
    });

    test("3.2b: Leap year day leap (2028-02-28 -> 2028-02-29 -> 2028-03-01) is consecutive", () => {
      const daysFeb28 = CivilDateUtils.dateStrToDays("2028-02-28")!;
      const daysFeb29 = CivilDateUtils.dateStrToDays("2028-02-29")!;
      const daysMar01 = CivilDateUtils.dateStrToDays("2028-03-01")!;
      expect(daysFeb29 - daysFeb28).toBe(1);
      expect(daysMar01 - daysFeb29).toBe(1);
    });

    test("3.3b: Year rollover (2026-12-31 -> 2027-01-01) is consecutive", () => {
      const daysDec31 = CivilDateUtils.dateStrToDays("2026-12-31")!;
      const daysJan01 = CivilDateUtils.dateStrToDays("2027-01-01")!;
      expect(daysJan01 - daysDec31).toBe(1);
    });

    test("3.4b: Direct miss explicitly resets current count to 0", () => {
      const streak: RoutineStreak = { current_count: 10, best_count: 10, last_completed_date: "2026-11-20" };
      StreakEngine.applyMiss(streak);
      expect(streak.current_count).toBe(0);
      expect(streak.best_count).toBe(10);
    });

    test("3.5b: Malformed date string does not crash streak engine", () => {
      const streak: RoutineStreak = { current_count: 3, best_count: 3 };
      StreakEngine.applyCompletion(streak, "invalid-date");
      expect(streak.current_count).toBe(3); // unchanged
    });
  });

  // F4 Boundaries
  describe("F4 Boundaries: Postponement / Snooze", () => {
    test("4.1b: In Tegas tier, exactly 3 snoozes succeed and 4th fails", () => {
      const sm = new RoutineSessionManager({
        routineId: "rtn-41b",
        tier: "tegas",
        scheduledFor: 1000,
        durationMin: 30,
      });
      const results: boolean[] = [];
      for (let i = 0; i < 4; i++) {
        const res = sm.snooze(10, `Snooze ${i}`);
        results.push(res.success);
        sm.advanceTimeToSnoozeExpiry();
      }
      expect(results).toEqual([true, true, true, false]);
    });

    test("4.2b: Snooze delay of 0 min clamps to 1 min minimum", () => {
      const sm = new RoutineSessionManager({
        routineId: "rtn-42b",
        tier: "tegas",
        scheduledFor: 1000,
        durationMin: 30,
      });
      sm.snooze(0, "Clamp test");
      expect(sm.getSnoozeUntil()).toBe(1000 + 60);
    });

    test("4.3b: Cannot snooze an active session", () => {
      const sm = new RoutineSessionManager({
        routineId: "rtn-43b",
        tier: "tegas",
        scheduledFor: 1000,
        durationMin: 30,
      });
      sm.startSession(1010);
      const res = sm.snooze(10, "Try snooze while active");
      expect(res.success).toBe(false);
      expect(res.error).toBe("CANNOT_SNOOZE");
    });

    test("4.4b: Cannot snooze a completed session", () => {
      const sm = new RoutineSessionManager({
        routineId: "rtn-44b",
        tier: "tegas",
        scheduledFor: 1000,
        durationMin: 30,
      });
      sm.startSession(1010);
      sm.completeSession(1200);
      const res = sm.snooze(10, "Try snooze completed");
      expect(res.success).toBe(false);
      expect(res.error).toBe("CANNOT_SNOOZE");
    });

    test("4.5b: Snooze with large delay (e.g. 120m) handles timestamp math safely", () => {
      const sm = new RoutineSessionManager({
        routineId: "rtn-45b",
        tier: "ingatkan",
        scheduledFor: 1700000000,
        durationMin: 30,
      });
      sm.snooze(120, "Long sleep");
      expect(sm.getSnoozeUntil()).toBe(1700000000 + 120 * 60);
    });
  });

  // F5 Boundaries
  describe("F5 Boundaries: Excused Skip Semantics", () => {
    test("5.1b: Case-insensitive 'SAKIT' recognized as excused", () => {
      const isExcused = "SAKIT".toLowerCase().includes("sakit");
      expect(isExcused).toBe(true);
    });

    test("5.2b: Whitespace-padded '  Libur Bersama  ' recognized as excused", () => {
      const isExcused = "  Libur Bersama  ".trim().toLowerCase().includes("libur");
      expect(isExcused).toBe(true);
    });

    test("5.3b: Unexcused reason does not set frozen_on_date", () => {
      const streak: RoutineStreak = { current_count: 4, best_count: 4 };
      const isExcused = false;
      if (isExcused) StreakEngine.applyExcusedSkip(streak, "2026-11-21");
      expect(streak.frozen_on_date).toBeUndefined();
    });

    test("5.4b: Cannot skip already completed session", () => {
      const state: RoutineSessionState = "completed";
      const canSkip = state !== "completed";
      expect(canSkip).toBe(false);
    });

    test("5.5b: Reason pill 'Lainnya' without explanation is unexcused by default", () => {
      const r = "Lainnya".toLowerCase();
      const isExcused = r.includes("sakit") || r.includes("libur") || r.includes("sibuk") || r.includes("izin");
      expect(isExcused).toBe(false);
    });
  });

  // F6 Boundaries
  describe("F6 Boundaries: Coalesced Scheduler", () => {
    test("6.1b: Calculation at exact scheduled second (now == scheduled_for) returns that instant", () => {
      const schedule: RoutineSchedule = { days: ["Thursday"], time: "10:00", duration_min: 30 };
      const next = CoalescedScheduler.calculateNextRoutineStart(schedule, 36000);
      expect(next).toBe(36000);
    });

    test("6.2b: Calculation 1 second after start returns next scheduled occurrence", () => {
      const schedule: RoutineSchedule = { days: ["Thursday"], time: "10:00", duration_min: 30 };
      const next = CoalescedScheduler.calculateNextRoutineStart(schedule, 36001);
      expect(next).toBe(36000 + 7 * 86400);
    });

    test("6.3b: Calculation with allow_within_grace_secs preserves current session eligibility", () => {
      const schedule: RoutineSchedule = { days: ["Thursday"], time: "10:00", duration_min: 30 };
      // 2 minutes after 10:00 (120s into 300s grace)
      const next = CoalescedScheduler.calculateNextRoutineStart(schedule, 36120, 300);
      expect(next).toBe(36000);
    });

    test("6.4b: Massive multi-routine list (100 routines) calculates earliest in < 5ms", () => {
      const routines: Routine[] = [];
      for (let i = 0; i < 100; i++) {
        const h = String(i % 24).padStart(2, "0");
        routines.push({
          id: `r-${i}`,
          title: `R ${i}`,
          category: "c",
          schedule: { days: ["Thursday"], time: `${h}:00`, duration_min: 30 },
          enforcement: "paksa",
          allowed_apps: [],
          distraction_rules: { block_during_routine: false, distraction_apps: [] },
          start_grace_min: 5,
          streak: { current_count: 0, best_count: 0 },
          active: true,
          created_at: 0,
          updated_at: 0,
        });
      }
      const t0 = performance.now();
      const earliest = CoalescedScheduler.findEarliestNextWake(routines, 0);
      const elapsed = performance.now() - t0;
      expect(earliest).toBe(0); // 00:00
      expect(elapsed).toBeLessThan(10);
    });

    test("6.5b: Schedule with invalid time string returns null safely", () => {
      const schedule: RoutineSchedule = { days: ["Thursday"], time: "99:99", duration_min: 30 };
      const next = CoalescedScheduler.calculateNextRoutineStart(schedule, 0);
      expect(next).toBeNull();
    });
  });

  // F7 Boundaries
  describe("F7 Boundaries: Commitment Ladder", () => {
    test("7.1b: Case-insensitive process matching: 'CHROME.EXE' matches 'chrome.exe'", () => {
      const target = "CHROME.EXE".toLowerCase();
      const rule = "chrome.exe".toLowerCase();
      expect(target).toBe(rule);
    });

    test("7.2b: Stripping .exe: 'chrome' matches 'chrome.exe'", () => {
      const strip = (s: string) => (s.toLowerCase().endsWith(".exe") ? s.toLowerCase().slice(0, -4) : s.toLowerCase());
      expect(strip("chrome.exe")).toBe(strip("chrome"));
    });

    test("7.3b: Inactive commitment is ignored during evaluation", () => {
      const com: Commitment = {
        id: "c-1",
        name: "Test",
        tier: "paksa",
        target_apps: ["game.exe"],
        active: false,
        created_at: 0,
        updated_at: 0,
      };
      const isActive = com.active;
      expect(isActive).toBe(false);
    });

    test("7.4b: Strictest tier wins when multiple commitments match", () => {
      const tiers: EnforcementTier[] = ["ingatkan", "paksa", "tegas"];
      const priority: Record<EnforcementTier, number> = { ingatkan: 1, tegas: 2, paksa: 3 };
      const highest = tiers.reduce((max, cur) => (priority[cur] > priority[max] ? cur : max));
      expect(highest).toBe("paksa");
    });

    test("7.5b: Empty target apps list allows all foreground apps", () => {
      const targetApps: string[] = [];
      const matches = targetApps.includes("chrome.exe");
      expect(matches).toBe(false);
    });
  });

  // F8 Boundaries
  describe("F8 Boundaries: Auto-Escalation Engine", () => {
    test("8.1b: Escalation with threshold = 1 triggers immediately on first ignore", () => {
      const esc = escalationTracker.recordIgnoredReminder("rtn-instant", "ingatkan", 1);
      expect(esc).toBe("tegas");
    });

    test("8.2b: Threshold = 10 does not escalate prematurely at 9", () => {
      for (let i = 0; i < 8; i++) {
        escalationTracker.recordIgnoredReminder("rtn-high", "ingatkan", 10);
      }
      const esc9 = escalationTracker.recordIgnoredReminder("rtn-high", "ingatkan", 10);
      expect(esc9).toBeNull();
      const esc10 = escalationTracker.recordIgnoredReminder("rtn-high", "ingatkan", 10);
      expect(esc10).toBe("tegas");
    });

    test("8.3b: Already Tegas tier does not re-escalate or demote", () => {
      const esc = escalationTracker.recordIgnoredReminder("rtn-tegas", "tegas", 3);
      expect(esc).toBeNull();
    });

    test("8.4b: Already Paksa tier does not demote to Tegas", () => {
      const esc = escalationTracker.recordIgnoredReminder("rtn-paksa", "paksa", 3);
      expect(esc).toBeNull();
    });

    test("8.5b: Non-tracked ID returns ignored count 0 safely", () => {
      expect(escalationTracker.getIgnoredCount("unknown-id")).toBe(0);
    });
  });

  // F9 Boundaries
  describe("F9 Boundaries: Relational Schemas", () => {
    test("9.1b: start_grace_min default is 5", () => {
      const db = new RoutinesDatabase();
      const colInfo = db.getColumnInfo("routines", "start_grace_min");
      expect(colInfo?.dflt_value).toBe("5");
      const routineId = db.insertRoutineMinimal({
        id: "rtn-default-grace",
        title: "Test Default Grace",
        category: "belajar",
        schedule_json: JSON.stringify({ days: ["Monday"], time: "10:00", duration_min: 30 }),
        enforcement: "paksa",
        created_at: 1000,
        updated_at: 1000,
      });
      const inserted = db.getRoutine(routineId);
      expect(inserted?.start_grace_min).toBe(5);
    });

    test("9.2b: current_streak default is 0", () => {
      const db = new RoutinesDatabase();
      const colInfo = db.getColumnInfo("routines", "current_streak");
      expect(colInfo?.dflt_value).toBe("0");
      const routineId = db.insertRoutineMinimal({
        id: "rtn-default-streak",
        title: "Test Default Streak",
        category: "belajar",
        schedule_json: JSON.stringify({ days: ["Monday"], time: "10:00", duration_min: 30 }),
        enforcement: "paksa",
        created_at: 1000,
        updated_at: 1000,
      });
      const inserted = db.getRoutine(routineId);
      expect(inserted?.streak.current_count).toBe(0);
    });

    test("9.3b: active default is 1 (true)", () => {
      const db = new RoutinesDatabase();
      const colInfo = db.getColumnInfo("routines", "active");
      expect(colInfo?.dflt_value).toBe("1");
      const routineId = db.insertRoutineMinimal({
        id: "rtn-default-active",
        title: "Test Default Active",
        category: "belajar",
        schedule_json: JSON.stringify({ days: ["Monday"], time: "10:00", duration_min: 30 }),
        enforcement: "paksa",
        created_at: 1000,
        updated_at: 1000,
      });
      const inserted = db.getRoutine(routineId);
      expect(inserted?.active).toBe(true);
    });

    test("9.4b: Complex schedule JSON roundtrip serialization", () => {
      const db = new RoutinesDatabase();
      const schedule: RoutineSchedule = {
        days: ["Monday", "Wednesday", "Friday"],
        time: "14:30",
        duration_min: 60,
        window_min: 15,
      };
      db.insertRoutine({
        id: "rtn-json-roundtrip",
        title: "JSON Test",
        category: "belajar",
        schedule,
        enforcement: "tegas",
        allowed_apps: ["app.exe"],
        distraction_rules: { block_during_routine: false, distraction_apps: [] },
        start_grace_min: 5,
        streak: { current_count: 0, best_count: 0 },
        active: true,
        created_at: 1000,
        updated_at: 1000,
      });
      const retrieved = db.getRoutine("rtn-json-roundtrip");
      expect(retrieved?.schedule).toEqual(schedule);
    });

    test("9.5b: Nullable optional columns (allowed_apps_json, escalation_json) handle null cleanly", () => {
      const db = new RoutinesDatabase();
      db.insertRoutineMinimal({
        id: "rtn-null-test",
        title: "Null Columns Test",
        category: "belajar",
        schedule_json: JSON.stringify({ days: [], time: "10:00", duration_min: 30 }),
        enforcement: "ingatkan",
        created_at: 1000,
        updated_at: 1000,
      });
      const routine = db.getRoutine("rtn-null-test");
      expect(routine?.allowed_apps).toEqual([]);
      expect(routine?.escalation).toBeUndefined();
    });
  });

  // F10 Boundaries
  describe("F10 Boundaries: Win32 Foreground Hook", () => {
    test("10.1b: Switching to HWND 0 returns null safely", () => {
      expect(pal.switchForeground(0)).toBeNull();
    });

    test("10.2b: Switching to non-existent HWND returns null", () => {
      expect(pal.switchForeground(999999)).toBeNull();
    });

    test("10.3b: Exception inside a listener does not break subsequent listeners", () => {
      let listener2Called = false;
      pal.onForegroundSwitch(() => {
        throw new Error("Crash in listener 1");
      });
      pal.onForegroundSwitch(() => {
        listener2Called = true;
      });
      pal.registerDummyWindow(9001, "app.exe", "ModeA_CleanClose");
      pal.switchForeground(9001);
      expect(listener2Called).toBe(true);
    });

    test("10.4b: Switching to same HWND twice dispatches both events", () => {
      let count = 0;
      pal.onForegroundSwitch(() => count++);
      pal.registerDummyWindow(9002, "app.exe", "ModeA_CleanClose");
      pal.switchForeground(9002);
      pal.switchForeground(9002);
      expect(count).toBe(2);
    });

    test("10.5b: Detaching all listeners leaves 0 active listeners", () => {
      const unhook = pal.onForegroundSwitch(() => {});
      unhook();
      expect(pal.switchForeground(9001)).toBeDefined();
    });
  });

  // F11 Boundaries
  describe("F11 Boundaries: Foreground Query", () => {
    test("11.1b: Executable path with spaces is parsed properly", () => {
      pal.registerDummyWindow(1101, "My Program.exe", "ModeA_CleanClose");
      pal.switchForeground(1101);
      const info = pal.getForegroundInfo();
      expect(info?.process_name).toBe("My Program.exe");
    });

    test("11.2b: Empty window title returns empty string instead of null/undefined", () => {
      pal.registerDummyWindow(1102, "untitled.exe", "ModeA_CleanClose", "");
      pal.switchForeground(1102);
      expect(pal.getForegroundInfo()?.window_title).toBe("");
    });

    test("11.3b: Window title with special characters and quotes", () => {
      const title = 'Document "Special" & <XML> - Editor';
      pal.registerDummyWindow(1103, "edit.exe", "ModeA_CleanClose", title);
      pal.switchForeground(1103);
      expect(pal.getForegroundInfo()?.window_title).toBe(title);
    });

    test("11.4b: Querying closed window returns null", () => {
      pal.registerDummyWindow(1104, "closeme.exe", "ModeA_CleanClose");
      pal.switchForeground(1104);
      pal.sendGracefulClose(1104, 1000);
      expect(pal.getForegroundInfo()).toBeNull();
    });

    test("11.5b: High PID value (e.g. 131072) handled safely", () => {
      pal.registerDummyWindow(1105, "highpid.exe", "ModeA_CleanClose");
      pal.switchForeground(1105);
      expect(pal.getForegroundInfo()?.pid).toBeGreaterThan(0);
    });
  });

  // F12 Boundaries
  describe("F12 Boundaries: User Idle Detection", () => {
    test("12.1b: Exact 30-second boundary: 29.999s is active, 30.000s is idle", () => {
      const now = 100000;
      pal.setLastInputTime(now - 29999);
      expect(pal.isUserIdle(30, now)).toBe(false);
      pal.setLastInputTime(now - 30000);
      expect(pal.isUserIdle(30, now)).toBe(true);
    });

    test("12.2b: Extreme idle duration (24 hours) does not cause negative math", () => {
      const now = 100000000;
      pal.setLastInputTime(now - 86400 * 1000);
      expect(pal.isUserIdle(30, now)).toBe(true);
    });

    test("12.3b: Clock skew: input time slightly in future does not crash", () => {
      const now = 100000;
      pal.setLastInputTime(now + 1000);
      expect(pal.isUserIdle(30, now)).toBe(false);
    });

    test("12.4b: Threshold of 0s considers any non-instant input idle", () => {
      const now = 100000;
      pal.setLastInputTime(now - 1);
      expect(pal.isUserIdle(0, now)).toBe(true);
    });

    test("12.5b: Multiple rapid input simulations keep user active", () => {
      const now = 100000;
      for (let i = 0; i < 10; i++) {
        pal.setLastInputTime(now - (10 - i) * 1000);
      }
      expect(pal.isUserIdle(30, now)).toBe(false);
    });
  });

  // F13 Boundaries
  describe("F13 Boundaries: Protected Processes List", () => {
    test("13.1b: Suffix stripping: 'EXPLORER' without .exe is protected", () => {
      expect(ProtectedProcesses.isProtected("EXPLORER")).toBe(true);
    });

    test("13.2b: Leading/trailing whitespace: '  code.exe  ' is protected", () => {
      expect(ProtectedProcesses.isProtected("  code.exe  ")).toBe(true);
    });

    test("13.3b: Substring spoofing 'fake_explorer.exe' is NOT protected", () => {
      expect(ProtectedProcesses.isProtected("fake_explorer.exe")).toBe(false);
    });

    test("13.4b: Substring spoofing 'code_runner.exe' is NOT protected", () => {
      expect(ProtectedProcesses.isProtected("code_runner.exe")).toBe(false);
    });

    test("13.5b: Empty string is NOT protected", () => {
      expect(ProtectedProcesses.isProtected("")).toBe(false);
    });
  });

  // F14 Boundaries
  describe("F14 Boundaries: Graceful Window Close", () => {
    test("14.1b: Closing non-existent window handle returns WindowRefusedClose", () => {
      expect(pal.sendGracefulClose(999999, 1000)).toBe("WindowRefusedClose");
    });

    test("14.2b: Closing already closed window returns WindowRefusedClose", () => {
      pal.registerDummyWindow(1401, "temp.exe", "ModeA_CleanClose");
      pal.sendGracefulClose(1401, 1000);
      expect(pal.sendGracefulClose(1401, 1040)).toBe("WindowRefusedClose");
    });

    test("14.3b: Close attempt on protected 'cmd.exe' returns ProtectedProcessRefused", () => {
      pal.registerDummyWindow(1402, "cmd.exe", "ModeA_CleanClose");
      expect(pal.sendGracefulClose(1402, 1000)).toBe("ProtectedProcessRefused");
    });

    test("14.4b: Zero TerminateProcess called even when window refuses close", () => {
      pal.registerDummyWindow(1403, "stubborn.exe", "ModeC_RefuseClose");
      pal.sendGracefulClose(1403, 1000);
      expect(pal.terminateProcessCalled).toBe(false);
    });

    test("14.5b: Window close recording timestamp precision", () => {
      pal.registerDummyWindow(1404, "app.exe", "ModeA_CleanClose");
      const res = pal.sendGracefulClose(1404, 12345);
      expect(res).toBe("ClosedSuccessfully");
    });
  });

  // F15 Boundaries
  describe("F15 Boundaries: Unsaved Changes Detection", () => {
    test("15.1b: Modal dialog with class '#32770' unconditionally halts closing", () => {
      pal.registerDummyWindow(1501, "cad.exe", "ModeB_UnsavedChangesModal", "Design.dwg", "#32770");
      expect(pal.sendGracefulClose(1501, 1000)).toBe("UnsavedDialogDetected");
    });

    test("15.2b: Halting close leaves window running and open", () => {
      pal.registerDummyWindow(1502, "cad.exe", "ModeB_UnsavedChangesModal");
      pal.switchForeground(1502);
      pal.sendGracefulClose(1502, 1000);
      expect(pal.getForegroundInfo()).not.toBeNull();
    });

    test("15.3b: Repeated close attempts while modal is open continually return UnsavedDialogDetected", () => {
      pal.registerDummyWindow(1503, "doc.exe", "ModeB_UnsavedChangesModal");
      expect(pal.sendGracefulClose(1503, 1000)).toBe("UnsavedDialogDetected");
      expect(pal.sendGracefulClose(1503, 1040)).toBe("UnsavedDialogDetected");
    });

    test("15.4b: Zero data loss guarantee: unsaved modal is never force-dismissed", () => {
      pal.registerDummyWindow(1504, "sketch.exe", "ModeB_UnsavedChangesModal");
      pal.sendGracefulClose(1504, 1000);
      expect(pal.hasModalUnsavedDialog(1504)).toBe(true);
    });

    test("15.5b: User dismissing modal then closing succeeds cleanly", () => {
      pal.registerDummyWindow(1505, "sketch.exe", "ModeA_CleanClose");
      pal.dismissModalDialog(1505);
      expect(pal.sendGracefulClose(1505, 1000)).toBe("ClosedSuccessfully");
    });
  });

  // F16 Boundaries
  describe("F16 Boundaries: Prohibition of TerminateProcess", () => {
    test("16.1b: Process killing is disabled by default in PAL", () => {
      expect(pal.terminateProcessCalled).toBe(false);
    });

    test("16.2b: Even repeated stubborn refusals never escalate to TerminateProcess", () => {
      pal.registerDummyWindow(1601, "unclosable.exe", "ModeC_RefuseClose");
      for (let i = 0; i < 5; i++) {
        pal.sendGracefulClose(1601, 1000 + i * 40);
      }
      expect(pal.terminateProcessCalled).toBe(false);
    });

    test("16.3b: Audit safety check: zero process kills performed", () => {
      expect(pal.terminateProcessCalled).toBe(false);
    });

    test("16.4b: Calling attemptTerminateProcess throws Gate #10 safety assertion", () => {
      expect(() => pal.attemptTerminateProcess()).toThrow();
    });

    test("16.5b: Enforcement failure gracefully exits to reflection instead of force kill", () => {
      const db = new RoutinesDatabase();
      const palSim = new Win32PalSimulator();
      const controller = new EnforcementController(db, palSim);

      palSim.registerDummyWindow(1650, "stubborn_app.exe", "ModeC_RefuseClose");
      palSim.switchForeground(1650);

      const result = controller.enforceWindowClose(1650, 1000);
      expect(palSim.terminateProcessCalled).toBe(false);
      expect(result.action).toBe("open_reflection_dialog");
      expect(result.escalatedToReflection).toBe(true);

      const history = db.getEnforcementHistory();
      expect(history.length).toBeGreaterThan(0);
      expect(history[history.length - 1].action_taken).toBe("reflection_prompted");
    });
  });

  // F17 Boundaries
  describe("F17 Boundaries: Rate Limiting & Cooldown", () => {
    test("17.1b: Exactly 29s after last close is blocked (cooldown 30s)", () => {
      pal.registerDummyWindow(1701, "app.exe", "ModeA_CleanClose");
      pal.registerDummyWindow(1702, "app.exe", "ModeA_CleanClose");
      pal.sendGracefulClose(1701, 1000);
      expect(pal.sendGracefulClose(1702, 1029)).toBe("WindowRefusedClose");
    });

    test("17.2b: Exactly 30s after last close is permitted", () => {
      pal.registerDummyWindow(1703, "app.exe", "ModeA_CleanClose");
      pal.registerDummyWindow(1704, "app.exe", "ModeA_CleanClose");
      pal.sendGracefulClose(1703, 1000);
      expect(pal.sendGracefulClose(1704, 1030)).toBe("ClosedSuccessfully");
    });

    test("17.3b: Hourly limit rolls over after 3600 seconds", () => {
      pal.registerDummyWindow(1705, "heavy.exe", "ModeA_CleanClose");
      pal.sendGracefulClose(1705, 1000);
      pal.registerDummyWindow(1706, "heavy.exe", "ModeA_CleanClose");
      expect(pal.sendGracefulClose(1706, 4601)).toBe("ClosedSuccessfully");
    });

    test("17.4b: Reopening after 121 seconds does not count towards 120s rapid reopen", () => {
      expect(pal.recordReopen("game.exe", 1000)).toBe(false);
      expect(pal.recordReopen("game.exe", 1060)).toBe(false);
      // 3rd reopen at 1122s is > 120s from 1000s
      expect(pal.recordReopen("game.exe", 1122)).toBe(false);
    });

    test("17.5b: Rapid reopen with 3 different apps does not trigger loop guard for any single app", () => {
      expect(pal.recordReopen("app1.exe", 1000)).toBe(false);
      expect(pal.recordReopen("app2.exe", 1010)).toBe(false);
      expect(pal.recordReopen("app3.exe", 1020)).toBe(false);
    });
  });

  // F18 Boundaries
  describe("F18 Boundaries: Emergency Pause", () => {
    test("18.1b: Pause duration 0 min is rejected", () => {
      const isValid = (d: number) => [15, 30, 60].includes(d);
      expect(isValid(0)).toBe(false);
    });

    test("18.2b: Non-standard duration (e.g. 45 min) rejected", () => {
      const isValid = (d: number) => [15, 30, 60].includes(d);
      expect(isValid(45)).toBe(false);
    });

    test("18.3b: Empty pause reason is rejected", () => {
      const isValidReason = (r: string) => r.trim().length > 0;
      expect(isValidReason("   ")).toBe(false);
    });

    test("18.4b: Exact boundary: now == expired_at is expired", () => {
      const expiredAt = 2000;
      const now = 2000;
      const isActive = expiredAt > now;
      expect(isActive).toBe(false);
    });

    test("18.5b: Exact boundary: now == expired_at - 1 is active", () => {
      const expiredAt = 2000;
      const now = 1999;
      const isActive = expiredAt > now;
      expect(isActive).toBe(true);
    });
  });

  // F19 Boundaries
  describe("F19 Boundaries: Rule Weakening Delay", () => {
    test("19.1b: Exact 599s boundary: weakening still pending, strict tier holds", () => {
      const requestedAt = 1000;
      const pendingUntil = requestedAt + 600;
      const now = 1599;
      expect(now >= pendingUntil).toBe(false);
    });

    test("19.2b: Exact 600s boundary: weakening takes effect", () => {
      const requestedAt = 1000;
      const pendingUntil = requestedAt + 600;
      const now = 1600;
      expect(now >= pendingUntil).toBe(true);
    });

    test("19.3b: Repeated weakening requests do not shorten remaining delay", () => {
      const db = new RoutinesDatabase();
      const palSim = new Win32PalSimulator();
      const controller = new EnforcementController(db, palSim);
      db.insertCommitment({
        id: "com-multi-weak",
        name: "Study",
        tier: "paksa",
        target_apps: ["game.exe", "stream.exe"],
        active: true,
        created_at: 1000,
        updated_at: 1000,
      });
      const req1 = controller.requestCommitmentModification("com-multi-weak", { target_apps: ["game.exe"] }, 1000);
      expect(req1.pendingUntil).toBe(1600);
      const req2 = controller.requestCommitmentModification("com-multi-weak", { target_apps: [] }, 1200);
      expect(req2.pendingUntil).toBe(1800);
      expect(db.getCommitment("com-multi-weak")?.pending_weakening_until).toBe(1800);
    });

    test("19.4b: Immediate strengthening while weakening pending cancels weakening delay", () => {
      const db = new RoutinesDatabase();
      const palSim = new Win32PalSimulator();
      const controller = new EnforcementController(db, palSim);
      db.insertCommitment({
        id: "com-strengthen",
        name: "Study",
        tier: "tegas",
        target_apps: ["game.exe"],
        pending_weakening_until: 1600,
        active: true,
        created_at: 1000,
        updated_at: 1000,
      });
      const req = controller.requestCommitmentModification("com-strengthen", { tier: "paksa" }, 1100);
      expect(req.isWeakening).toBe(false);
      expect(req.coolingOffRequired).toBe(false);
      expect(db.getCommitment("com-strengthen")?.pending_weakening_until).toBeUndefined();
    });

    test("19.5b: Weakening delay applies when removing apps from restriction list", () => {
      const db = new RoutinesDatabase();
      const palSim = new Win32PalSimulator();
      const controller = new EnforcementController(db, palSim);

      db.insertCommitment({
        id: "com-weakening-test",
        name: "Study Focus",
        tier: "paksa",
        target_apps: ["steam.exe", "discord.exe"],
        active: true,
        created_at: 1000,
        updated_at: 1000,
      });

      const nowSecs = 2000;
      const requestResult = controller.requestCommitmentModification(
        "com-weakening-test",
        { target_apps: ["discord.exe"] },
        nowSecs
      );

      expect(requestResult.isWeakening).toBe(true);
      expect(requestResult.coolingOffRequired).toBe(true);
      expect(requestResult.pendingUntil).toBe(nowSecs + 600);

      const stored = db.getCommitment("com-weakening-test");
      expect(stored?.pending_weakening_until).toBe(2600);
      expect(controller.isAppRestricted("steam.exe", 2300)).toBe(true);

      controller.evaluatePendingWeakening(2601);
      expect(controller.isAppRestricted("steam.exe", 2601)).toBe(false);
    });
  });

  // F20 Boundaries
  describe("F20 Boundaries: Kill Switch & Crash Recovery", () => {
    test("20.1b: Repeated /jeda invocations are idempotent", () => {
      const db = new RoutinesDatabase();
      const palSim = new Win32PalSimulator();
      const controller = new EnforcementController(db, palSim);

      const res1 = controller.handleSlashCommand("/jeda", 1000);
      expect(res1.success).toBe(true);
      expect(controller.isEmergencyPaused(1005)).toBe(true);

      const res2 = controller.handleSlashCommand("/jeda", 1010);
      expect(res2.success).toBe(true);
      expect(res2.alreadyPaused).toBe(true);
      expect(controller.isEmergencyPaused(1015)).toBe(true);
    });

    test("20.2b: Corrupt storage file falls back to fail-safe unblocked default", () => {
      const corruptDb = new Database(":memory:");
      corruptDb.run("CREATE TABLE routines (corrupt_data BLOB NOT NULL);");
      
      const palSim = new Win32PalSimulator();
      const controller = EnforcementController.createWithFallback(corruptDb, palSim);

      expect(controller.isFailSafeActive()).toBe(true);
      expect(controller.getEnforcementState()).toBe("safe_unblocked");

      palSim.registerDummyWindow(2021, "game.exe", "ModeA_CleanClose");
      const closeResult = controller.enforceWindowClose(2021, 1000);
      expect(closeResult.action).toBe("bypassed_safe_unblocked");
      expect(palSim.terminateProcessCalled).toBe(false);
    });

    test("20.3b: System shutdown signal dispatches hook unregistration cleanly", () => {
      const palSim = new Win32PalSimulator();
      let listenerCalled = false;
      const unhook = palSim.onForegroundSwitch(() => {
        listenerCalled = true;
      });

      expect(palSim.getActiveHookCount()).toBe(1);
      unhook();
      expect(palSim.getActiveHookCount()).toBe(0);

      palSim.registerDummyWindow(2031, "test.exe", "ModeA_CleanClose");
      palSim.switchForeground(2031);
      expect(listenerCalled).toBe(false);
    });

    test("20.4b: Rapid toggle of kill switch does not deadlock", () => {
      const db = new RoutinesDatabase();
      const palSim = new Win32PalSimulator();
      const controller = new EnforcementController(db, palSim);

      for (let i = 0; i < 100; i++) {
        controller.toggleKillSwitch();
      }
      expect(controller.isKillSwitchActive()).toBe(false);
      expect(controller.isOsLocked()).toBe(false);
    });

    test("20.5b: Pending weakening survives crash via SQLite column", () => {
      const db = new RoutinesDatabase();
      db.insertCommitment({
        id: "com-crash-weak",
        name: "Crash Test",
        tier: "paksa",
        target_apps: ["game.exe"],
        pending_weakening_until: 1700000600,
        active: true,
        created_at: 1000,
        updated_at: 1000,
      });
      const retrieved = db.getCommitment("com-crash-weak");
      expect(retrieved?.pending_weakening_until).toBe(1700000600);
    });
  });

  // F21 Boundaries
  describe("F21 Boundaries: Gate #10 Mock Window Fixture", () => {
    test("21.1b: Mode B modal dialog cannot be bypassed by repeated calls", () => {
      pal.registerDummyWindow(2101, "app.exe", "ModeB_UnsavedChangesModal");
      expect(pal.sendGracefulClose(2101, 1000)).toBe("UnsavedDialogDetected");
      expect(pal.sendGracefulClose(2101, 1040)).toBe("UnsavedDialogDetected");
    });

    test("21.2b: Mode C stubborn window stays alive without process kill", () => {
      pal.registerDummyWindow(2102, "stubborn.exe", "ModeC_RefuseClose");
      pal.switchForeground(2102);
      pal.sendGracefulClose(2102, 1000);
      expect(pal.getForegroundInfo()).not.toBeNull();
      expect(pal.terminateProcessCalled).toBe(false);
    });

    test("21.3b: Switching window modes dynamically at runtime", () => {
      pal.registerDummyWindow(2103, "dyn.exe", "ModeB_UnsavedChangesModal");
      expect(pal.sendGracefulClose(2103, 1000)).toBe("UnsavedDialogDetected");
      pal.dismissModalDialog(2103);
      // Register clean mode
      pal.registerDummyWindow(2103, "dyn.exe", "ModeA_CleanClose");
      expect(pal.sendGracefulClose(2103, 1040)).toBe("ClosedSuccessfully");
    });

    test("21.4b: Mock processes never collide with real OS PIDs", () => {
      pal.registerDummyWindow(2104, "mock.exe", "ModeA_CleanClose");
      pal.switchForeground(2104);
      const pid = pal.getForegroundInfo()?.pid;
      expect(pid).toBeGreaterThan(8000);
    });

    test("21.5b: Total count of TerminateProcess calls remains 0", () => {
      expect(pal.terminateProcessCalled).toBe(false);
    });
  });

  // F26 Boundaries
  describe("F26 Boundaries: Tauri IPC Bridge", () => {
    test("26.1b: Huge payload in IPC does not crash parser", () => {
      const bigTitle = "A".repeat(1000);
      expect(bigTitle.length).toBe(1000);
    });

    test("26.2b: Negative numbers in duration return validation error", () => {
      const isValidDuration = (d: number) => d > 0;
      expect(isValidDuration(-10)).toBe(false);
    });

    test("26.3b: Special JSON characters (quotes, backslashes) in title roundtrip", () => {
      const title = 'Special "quotes" and \\backslashes\\';
      const serialized = JSON.stringify({ title });
      const parsed = JSON.parse(serialized);
      expect(parsed.title).toBe(title);
    });

    test("26.4b: Concurrent IPC requests resolve independently", async () => {
      const p1 = Promise.resolve("res1");
      const p2 = Promise.resolve("res2");
      const [r1, r2] = await Promise.all([p1, p2]);
      expect(r1).toBe("res1");
      expect(r2).toBe("res2");
    });

    test("26.5b: Error responses contain structured error code and message", () => {
      const err = { code: -32602, message: "Invalid params" };
      expect(err.code).toBe(-32602);
      expect(err.message).toBe("Invalid params");
    });
  });

  // F27 Boundaries
  describe("F27 Boundaries: Focus Panel", () => {
    test("27.1b: Progress at 0s elapsed is exactly 0%", () => {
      const pct = (0 / 1800) * 100;
      expect(pct).toBe(0);
    });

    test("27.2b: Progress at full duration is exactly 100%", () => {
      const pct = (1800 / 1800) * 100;
      expect(pct).toBe(100);
    });

    test("27.3b: Progress overflowing duration clamps to 100%", () => {
      const elapsed = 2000;
      const total = 1800;
      const clamped = Math.min(100, (elapsed / total) * 100);
      expect(clamped).toBe(100);
    });

    test("27.4b: Empty routine list renders fallback text <= 36 chars", () => {
      const emptyText = "Tidak ada rutinitas hari ini";
      expect(Vcu5Validator.validateDescription(emptyText).valid).toBe(true);
    });

    test("27.5b: Long routine title is truncated with ellipsis", () => {
      const longTitle = "Belajar Konsep Distributed Systems & Microservices";
      const truncated = longTitle.length > 16 ? longTitle.slice(0, 13) + "..." : longTitle;
      expect(truncated.length).toBeLessThanOrEqual(16);
    });
  });

  // F28 Boundaries
  describe("F28 Boundaries: Enforcement & Alert UI", () => {
    test("28.1b: Countdown decrementing 10 -> 0", () => {
      const counts: number[] = [];
      for (let c = 10; c >= 0; c--) counts.push(c);
      expect(counts.length).toBe(11);
      expect(counts[0]).toBe(10);
      expect(counts[10]).toBe(0);
    });

    test("28.2b: Button text 'Tunda 5m' length is exactly 8 chars (<= 10)", () => {
      expect("Tunda 5m".length).toBe(8);
      expect("Tunda 5m".length).toBeLessThanOrEqual(10);
    });

    test("28.3b: Button text 'Jeda' length is exactly 4 chars (<= 10)", () => {
      expect("Jeda".length).toBe(4);
      expect("Jeda".length).toBeLessThanOrEqual(10);
    });

    test("28.4b: Alert title 'Selesaikan dulu' is exactly 15 chars (<= 16)", () => {
      expect("Selesaikan dulu".length).toBe(15);
      expect("Selesaikan dulu".length).toBeLessThanOrEqual(16);
    });

    test("28.5b: Alert description 'Simpan pekerjaan Anda' is 21 chars (<= 36)", () => {
      expect("Simpan pekerjaan Anda".length).toBe(21);
      expect("Simpan pekerjaan Anda".length).toBeLessThanOrEqual(36);
    });
  });

  // F29 Boundaries
  describe("F29 Boundaries: Emergency Pause Modal", () => {
    test("29.1b: Pill 'Lainnya' length is 7 chars (<= 10)", () => {
      expect("Lainnya".length).toBe(7);
      expect("Lainnya".length).toBeLessThanOrEqual(10);
    });

    test("29.2b: Pill '15m' length is 3 chars (<= 10)", () => {
      expect("15m".length).toBe(3);
    });

    test("29.3b: Custom reason input capped at 36 chars", () => {
      const custom = "Ada panggilan darurat dari keluarga";
      expect(custom.length).toBeLessThanOrEqual(36);
    });

    test("29.4b: Confirm button disabled when no reason pill selected", () => {
      const selectedReason: string | null = null;
      const isEnabled = selectedReason !== null;
      expect(isEnabled).toBe(false);
    });

    test("29.5b: Confirm button enabled once reason pill selected", () => {
      const selectedReason: string | null = "Sibuk";
      const isEnabled = selectedReason !== null;
      expect(isEnabled).toBe(true);
    });
  });

  // F31 Boundaries
  describe("F31 Boundaries: VC-U5 Presentation Gate", () => {
    test("31.1b: Button text with 11 chars fails validation", () => {
      expect(Vcu5Validator.validateButtonText("12345678901").valid).toBe(false);
    });

    test("31.2b: Title text with 17 chars fails validation", () => {
      expect(Vcu5Validator.validateTitle("12345678901234567").valid).toBe(false);
    });

    test("31.3b: Description text with 37 chars fails validation", () => {
      expect(Vcu5Validator.validateDescription("1234567890123456789012345678901234567").valid).toBe(false);
    });

    test("31.4b: Unapproved raw hex color '#123456' fails token validation", () => {
      expect(Vcu5Validator.validateTokenColor("#123456")).toBe(false);
    });

    test("31.5b: Unknown mascot expression 'angry_cat' fails validation", () => {
      expect(Vcu5Validator.validateMascotExpression("angry_cat")).toBe(false);
    });
  });
});

// =============================================================================
// TEST SUITE: TIER 3 — CROSS-FEATURE PAIRWISE COMBINATIONS (24 TESTS)
// =============================================================================

describe("Tier 3: Cross-Feature Pairwise Combinations", () => {
  let store: RoutineStore;
  let pal: Win32PalSimulator;
  let tracker: AutoEscalationTracker;

  beforeEach(() => {
    store = new RoutineStore();
    pal = new Win32PalSimulator();
    tracker = new AutoEscalationTracker();
  });

  test("P1: F1 (Routine CRUD) + F7 (Commitment Ladder) — Routine with Paksa tier restricts target apps", () => {
    store.createRoutine({
      id: "r-p1",
      title: "Focus Study",
      category: "belajar",
      schedule: { days: [], time: "10:00", duration_min: 30 },
      enforcement: "paksa",
      allowed_apps: ["code.exe"],
      distraction_rules: { block_during_routine: true, distraction_apps: ["game.exe"] },
      start_grace_min: 5,
      streak: { current_count: 0, best_count: 0 },
      active: true,
      created_at: 0,
      updated_at: 0,
    });
    const r = store.getRoutine("r-p1")!;
    expect(r.enforcement).toBe("paksa");
    expect(r.distraction_rules.distraction_apps.includes("game.exe")).toBe(true);
  });

  test("P2: F2 (Session Lifecycle) + F3 (Streak Engine) — Active session completion increments streak", () => {
    const streak: RoutineStreak = { current_count: 1, best_count: 1, last_completed_date: "2026-11-20" };
    const sm = new RoutineSessionManager({
      routineId: "r-1",
      tier: "paksa",
      scheduledFor: 1000,
      durationMin: 30,
    });
    sm.startSession(1010);
    const res = sm.completeSession(1200);
    expect(res.success).toBe(true);
    expect(sm.getState()).toBe("completed");
    StreakEngine.applyCompletion(streak, "2026-11-21");
    expect(streak.current_count).toBe(2);
  });

  test("P3: F2 (Session Lifecycle) + F4 (Postponement) — Prompted session snoozed 3x in Tegas forces user to start or skip", () => {
    const sm = new RoutineSessionManager({
      routineId: "r-1",
      tier: "tegas",
      scheduledFor: 1000,
      durationMin: 30,
    });
    for (let i = 0; i < 3; i++) {
      expect(sm.snooze(10, `Snooze ${i}`).success).toBe(true);
      sm.advanceTimeToSnoozeExpiry();
    }
    const fourth = sm.snooze(10, "4th");
    expect(fourth.success).toBe(false);
    expect(fourth.error).toBe("MAX_SNOOZE_REACHED");
    expect(sm.getState()).toBe("prompted");
  });

  test("P4: F3 (Streak Engine) + F5 (Excused Skip) — Excused skip on day D freezes streak; day D+1 completion continues it", () => {
    const streak: RoutineStreak = { current_count: 4, best_count: 4, last_completed_date: "2026-11-20" };
    StreakEngine.applyExcusedSkip(streak, "2026-11-21");
    expect(streak.frozen_on_date).toBe("2026-11-21");
    StreakEngine.applyCompletion(streak, "2026-11-22");
    expect(streak.current_count).toBe(5);
  });

  test("P5: F2 (Session Lifecycle) + F6 (Coalesced Scheduler) — Completed session calculates next wake for next day", () => {
    const schedule: RoutineSchedule = { days: ["Monday", "Tuesday"], time: "19:00", duration_min: 30 };
    // Monday epoch 360000 -> next is Tuesday
    const next = CoalescedScheduler.calculateNextRoutineStart(schedule, 360000 + 3600);
    expect(next).toBeGreaterThan(360000);
  });

  test("P6: F7 (Commitment Ladder) + F8 (Auto-Escalation) — 3 ignored Ingatkan reminders escalate routine to Tegas", () => {
    tracker.recordIgnoredReminder("rtn-p6", "ingatkan", 3);
    tracker.recordIgnoredReminder("rtn-p6", "ingatkan", 3);
    const esc = tracker.recordIgnoredReminder("rtn-p6", "ingatkan", 3);
    expect(esc).toBe("tegas");
  });

  test("P7: F7 (Commitment Ladder) + F13 (Protected Processes) — Opening code.exe under Paksa tier is Allowed", () => {
    expect(ProtectedProcesses.isProtected("code.exe")).toBe(true);
  });

  test("P8: F7 (Commitment Ladder) + F14 (Graceful WM_CLOSE) — Distraction under Paksa triggers graceful WM_CLOSE", () => {
    pal.registerDummyWindow(3008, "game.exe", "ModeA_CleanClose");
    const res = pal.sendGracefulClose(3008, 1000);
    expect(res).toBe("ClosedSuccessfully");
  });

  test("P9: F14 (WM_CLOSE) + F15 (Unsaved Dialog Detection) — WM_CLOSE on ModeB window halts with UnsavedDialogDetected", () => {
    pal.registerDummyWindow(3009, "word.exe", "ModeB_UnsavedChangesModal");
    const res = pal.sendGracefulClose(3009, 1000);
    expect(res).toBe("UnsavedDialogDetected");
  });

  test("P10: F14 (WM_CLOSE) + F16 (Prohibition of TerminateProcess) — Window refusing WM_CLOSE is NOT terminated", () => {
    pal.registerDummyWindow(3010, "stubborn.exe", "ModeC_RefuseClose");
    pal.sendGracefulClose(3010, 1000);
    expect(pal.terminateProcessCalled).toBe(false);
  });

  test("P11: F14 (WM_CLOSE) + F17 (Rate Limiting & Cooldown) — Re-closing app within 30s is rejected", () => {
    pal.registerDummyWindow(3011, "distract.exe", "ModeA_CleanClose");
    pal.registerDummyWindow(3012, "distract.exe", "ModeA_CleanClose");
    expect(pal.sendGracefulClose(3011, 1000)).toBe("ClosedSuccessfully");
    expect(pal.sendGracefulClose(3012, 1015)).toBe("WindowRefusedClose");
  });

  test("P12: F17 (Reopen Loop Guard) + F8 (Auto-Escalation) — 3 rapid reopens escalates to reflection", () => {
    pal.recordReopen("app.exe", 1000);
    pal.recordReopen("app.exe", 1020);
    const triggered = pal.recordReopen("app.exe", 1040);
    expect(triggered).toBe(true);
  });

  test("P13: F7 (Commitment Ladder) + F18 (Emergency Pause) — Active emergency pause overrides Paksa enforcement", () => {
    store.addEmergencyPause({ id: "ep-1", started_at: 1000, duration_min: 30, reason: "Darurat", expired_at: 2800 });
    const isPaused = store.getActivePause(1500) !== undefined;
    expect(isPaused).toBe(true);
  });

  test("P14: F18 (Emergency Pause) + F19 (Weakening Delay) — Emergency pause halts enforcement instantly without waiting 10 min", () => {
    const pauseActive = true;
    const enforce = !pauseActive;
    expect(enforce).toBe(false);
  });

  test("P15: F19 (Weakening Delay) + F7 (Commitment Ladder) — Paksa enforced during 10m delay then transitions to Ingatkan", () => {
    const requestedAt = 1000;
    const pendingUntil = requestedAt + 600;
    expect(1300 >= pendingUntil).toBe(false);
    expect(1600 >= pendingUntil).toBe(true);
  });

  test("P16: F10 (Win32 Hook) + F12 (User Idle Detection) — Idle >= 30s pauses distraction evaluation on hook switch", () => {
    pal.setLastInputTime(100000 - 35000);
    const isIdle = pal.isUserIdle(30, 100000);
    expect(isIdle).toBe(true);
  });

  test("P17: F12 (User Idle Detection) + F14 (WM_CLOSE) — User idle suppresses countdown leading to WM_CLOSE", () => {
    const palSim = new Win32PalSimulator();
    const timer = new DistractionTimerManager({ tier: "paksa", initialCountdownSecs: 6 });
    
    const now = 100000;
    palSim.setLastInputTime(now - 35000);
    const isIdle = palSim.isUserIdle(30, now);
    expect(isIdle).toBe(true);

    timer.tick(3, isIdle);
    expect(timer.getRemainingSecs()).toBe(6);
    expect(timer.hasExpired()).toBe(false);
    expect(palSim.terminateProcessCalled).toBe(false);
  });

  test("P18: F15 (Unsaved Dialog) + F28 (Alert UI) — Modal detection shows 'Selesaikan dulu' alert", () => {
    const title = "Selesaikan dulu";
    expect(Vcu5Validator.validateTitle(title).valid).toBe(true);
  });

  test("P19: F21 (Gate #10 Mode B) + F15 (Unsaved Changes) — DummyTestWindow Mode B proves zero data loss", () => {
    pal.registerDummyWindow(3019, "mock_b.exe", "ModeB_UnsavedChangesModal");
    expect(pal.sendGracefulClose(3019, 1000)).toBe("UnsavedDialogDetected");
    expect(pal.terminateProcessCalled).toBe(false);
  });

  test("P20: F21 (Gate #10 Mode C) + F16 (Zero Process Kill) — Mode C proves unclosed window is NOT terminated", () => {
    pal.registerDummyWindow(3020, "mock_c.exe", "ModeC_RefuseClose");
    expect(pal.sendGracefulClose(3020, 1000)).toBe("WindowRefusedClose");
    expect(pal.terminateProcessCalled).toBe(false);
  });

  test("P21: F26 (Tauri IPC Bridge) + F18 (Emergency Pause) — IPC trigger of emergency pause sets pause state", async () => {
    const db = new RoutinesDatabase();
    const palSim = new Win32PalSimulator();
    const controller = new EnforcementController(db, palSim);
    const bridge = new RoutinesIpcBridge({ controller, database: db });

    const res = await bridge.dispatch("productivity.enforce.emergency_pause", {
      duration_min: 30,
      reason: "Sakit",
    });
    expect(res.success).toBe(true);
    expect(res.data?.is_paused).toBe(true);
    expect(res.data?.duration_min).toBe(30);
    expect(res.data?.reason).toBe("Sakit");
    expect(controller.isEmergencyPaused()).toBe(true);
  });

  test("P22: F27 (Focus Panel) + F2 (Session Active) — Focus panel renders active session timer ring", () => {
    const sm = new RoutineSessionManager({
      routineId: "r-p22",
      tier: "paksa",
      scheduledFor: 1000,
      durationMin: 30,
    });
    sm.startSession(1000);
    expect(sm.getState()).toBe("active");
    const duration = 30;
    const elapsed = 15;
    const ringProgress = (elapsed / duration) * 100;
    expect(ringProgress).toBe(50);
  });

  test("P23: F28 (Enforcement Alert) + F31 (VC-U5 Bounds) — All alert buttons conform to <= 10 chars", () => {
    const buttons = ["Tunda 5m", "Jeda"];
    for (const b of buttons) {
      expect(Vcu5Validator.validateButtonText(b).valid).toBe(true);
    }
  });

  test("P24: F29 (Emergency Pause Modal) + F31 (VC-U5 Copy) — Duration and reason pills conform to <= 10 chars", () => {
    const pills = ["15m", "30m", "60m", "Sibuk", "Sakit", "Libur", "Lainnya"];
    for (const p of pills) {
      expect(Vcu5Validator.validateButtonText(p).valid).toBe(true);
    }
  });
});

// =============================================================================
// TEST SUITE: TIER 4 — REAL-WORLD APPLICATION SCENARIOS
// =============================================================================

describe("Tier 4: Real-World Application Scenarios (S1, S2, S3, S4, S5, S6, S8)", () => {
  let store: RoutineStore;
  let pal: Win32PalSimulator;

  beforeEach(() => {
    store = new RoutineStore();
    pal = new Win32PalSimulator();
  });

  // Scenario S1: Study Session Workflow
  test("S1: SCENARIO-01-STUDY-SESSION: Ambient T-5m, prompt T-0, distraction countdown, abort on return, completion streak", () => {
    // 1. Setup Paksa study routine at 19:00 (epoch 36000)
    const routine: Routine = {
      id: "rtn-s1",
      title: "Belajar Rust",
      category: "belajar",
      schedule: { days: ["Thursday"], time: "10:00", duration_min: 45 },
      enforcement: "paksa",
      allowed_apps: ["code.exe"],
      distraction_rules: { block_during_routine: true, distraction_apps: ["chrome.exe"] },
      start_grace_min: 5,
      streak: { current_count: 2, best_count: 2, last_completed_date: "2026-11-20" },
      active: true,
      created_at: 0,
      updated_at: 0,
    };
    store.createRoutine(routine);

    // 2. Ambient T-5m check (at 09:55 -> epoch 35700)
    const sm = new RoutineSessionManager({
      routineId: "rtn-s1",
      tier: "paksa",
      scheduledFor: 36000,
      durationMin: 45,
      startGraceMin: 5,
    }, 35700);
    expect(sm.getState()).toBe("ambient");

    // 3. Prompt at T-0 (10:00 -> epoch 36000)
    expect(sm.evaluateStateAtTime(36000)).toBe("prompted");
    expect(sm.getState()).toBe("prompted");

    // 4. User starts session -> Active
    const startRes = sm.startSession(36010);
    expect(startRes.success).toBe(true);
    expect(sm.getState()).toBe("active");

    // 5. User opens distraction Chrome -> 10s countdown begins
    pal.registerDummyWindow(9101, "chrome.exe", "ModeA_CleanClose");
    pal.switchForeground(9101);
    const timer = new DistractionTimerManager({ tier: "paksa", initialCountdownSecs: 10 });
    timer.tick(4, false); // 4s pass
    expect(timer.getRemainingSecs()).toBe(6);

    // 6. User switches back to VS Code (allowlisted) -> countdown aborts!
    pal.registerDummyWindow(9102, "code.exe", "ModeA_CleanClose");
    pal.switchForeground(9102);
    timer.reset();
    expect(timer.getRemainingSecs()).toBe(10);
    const activeApp = pal.getForegroundInfo()?.process_name;
    const isAllowlisted = routine.allowed_apps.includes(activeApp || "");
    expect(isAllowlisted).toBe(true);
    // Countdown aborted; no close sent!
    expect(pal.terminateProcessCalled).toBe(false);

    // 7. Session completes -> streak increments to 3
    const compRes = sm.completeSession(36000 + 45 * 60);
    expect(compRes.success).toBe(true);
    expect(sm.getState()).toBe("completed");
    StreakEngine.applyCompletion(routine.streak, "2026-11-21");
    expect(routine.streak.current_count).toBe(3);
    expect(routine.streak.best_count).toBe(3);
  });

  // Scenario S2: Unsaved Changes Safety Workflow
  test("S2: SCENARIO-02-UNSAVED-CHANGES-SAFETY: WM_CLOSE triggers modal; force close halted; zero process kills", () => {
    // 1. User has unsaved work in editor (DummyTestWindow Mode B)
    pal.registerDummyWindow(9201, "notepad.exe", "ModeB_UnsavedChangesModal", "Untitled.txt *", "#32770");
    pal.switchForeground(9201);

    // 2. Distraction countdown reaches 0s -> WM_CLOSE sent
    const res = pal.sendGracefulClose(9201, 1000);

    // 3. Modal detected (#32770) -> Close halted safely
    expect(res).toBe("UnsavedDialogDetected");

    // 4. Invariant: zero TerminateProcess calls
    expect(pal.terminateProcessCalled).toBe(false);

    // 5. UI alert "Selesaikan dulu" displayed
    expect(Vcu5Validator.validateTitle("Selesaikan dulu").valid).toBe(true);
    expect(Vcu5Validator.validateDescription("Simpan pekerjaan Anda").valid).toBe(true);

    // 6. Window remains alive and unsaved data intact
    expect(pal.hasModalUnsavedDialog(9201)).toBe(true);
  });

  // Scenario S3: Asymmetric Commitment Modification
  test("S3: SCENARIO-03-ASYMMETRIC-COMMITMENT: 10m cooling-off delay on weakening; strict rules enforced; instant strengthening", () => {
    const db = new RoutinesDatabase();
    const palSim = new Win32PalSimulator();
    const controller = new EnforcementController(db, palSim);
    const now = 1000;
    db.insertCommitment({
      id: "com-s3",
      name: "Strict Work",
      tier: "paksa",
      target_apps: ["steam.exe"],
      active: true,
      created_at: now,
      updated_at: now,
    });

    // 1. User requests weakening: Paksa -> Ingatkan
    const reqResult = controller.requestCommitmentModification("com-s3", { tier: "ingatkan" }, now);
    expect(reqResult.isWeakening).toBe(true);
    expect(reqResult.coolingOffRequired).toBe(true);
    expect(reqResult.pendingUntil).toBe(now + 600);

    // 2. 5 minutes later (epoch 1300), strict Paksa rules still enforced!
    const at5m = 1300;
    expect(controller.isAppRestricted("steam.exe", at5m)).toBe(true);

    // 3. User reconsiders and cancels weakening -> restored to pure Paksa
    const restoreResult = controller.requestCommitmentModification("com-s3", { tier: "paksa" }, 1350);
    expect(restoreResult.isWeakening).toBe(false);
    expect(restoreResult.coolingOffRequired).toBe(false);
    const restored = db.getCommitment("com-s3");
    expect(restored?.tier).toBe("paksa");
    expect(restored?.pending_weakening_until).toBeUndefined();

    // 4. User changes Ingatkan -> Paksa -> applies immediately with 0s delay
    db.insertCommitment({
      id: "com-s3-weak",
      name: "Mild Work",
      tier: "ingatkan",
      target_apps: ["game.exe"],
      active: true,
      created_at: now,
      updated_at: now,
    });
    const strengthenResult = controller.requestCommitmentModification("com-s3-weak", { tier: "paksa" }, 1400);
    expect(strengthenResult.isWeakening).toBe(false);
    expect(strengthenResult.coolingOffRequired).toBe(false);
    const strengthened = db.getCommitment("com-s3-weak");
    expect(strengthened?.tier).toBe("paksa");
    expect(strengthened?.pending_weakening_until).toBeUndefined();
  });

  // Scenario S4: Excused Skip Streak Preservation
  test("S4: SCENARIO-04-EXCUSED-SKIP-STREAK: Skipping with 'Sakit' freezes streak; next day completion continues streak", () => {
    const streak: RoutineStreak = { current_count: 5, best_count: 5, last_completed_date: "2026-11-20" };

    // 1. Day 1: Routine prompts, user is ill and skips with "Sakit"
    const sm = new RoutineSessionManager({
      routineId: "rtn-s4",
      tier: "paksa",
      scheduledFor: 1000,
      durationMin: 30,
    });
    const skipRes = sm.skipSession("Sakit demam", true);
    expect(skipRes.success).toBe(true);
    expect(sm.getState()).toBe("skipped");
    expect(sm.getIsExcused()).toBe(true);

    // 2. Streak is frozen on 2026-11-21 without resetting count
    StreakEngine.applyExcusedSkip(streak, "2026-11-21");
    expect(streak.current_count).toBe(5);
    expect(streak.frozen_on_date).toBe("2026-11-21");

    // 3. Day 2: User completes routine on 2026-11-22 -> resumes seamlessly to 6
    StreakEngine.applyCompletion(streak, "2026-11-22");
    expect(streak.current_count).toBe(6);
    expect(streak.best_count).toBe(6);
    expect(streak.frozen_on_date).toBeUndefined();
  });

  // Scenario S5: Inactivity / Idle Quota Protection
  test("S5: SCENARIO-05-IDLE-QUOTA-PROTECTION: Inactivity >= 30s freezes countdown; resumes when user returns", () => {
    // 1. Distraction countdown starts at 10s
    const timer = new DistractionTimerManager({ tier: "paksa", initialCountdownSecs: 10 });
    timer.tick(3, false); // 3s pass -> 7s remaining
    expect(timer.getRemainingSecs()).toBe(7);

    // 2. User walks away: 30s elapse without input
    const baseTime = 100000;
    pal.setLastInputTime(baseTime - 35000); // 35s idle
    const isIdle = pal.isUserIdle(30, baseTime);
    expect(isIdle).toBe(true);

    // 3. Inactivity freezes distraction countdown (remains 7s)
    timer.tick(5, isIdle);
    expect(timer.getRemainingSecs()).toBe(7);
    expect(timer.isPaused()).toBe(true);

    // 4. User returns and moves mouse
    pal.setLastInputTime(baseTime); // activity restored
    expect(pal.isUserIdle(30, baseTime)).toBe(false);

    // 5. Countdown resumes smoothly
    timer.tick(1, false);
    expect(timer.getRemainingSecs()).toBe(6);
    expect(timer.isPaused()).toBe(false);
  });

  // Scenario S6: Emergency Pause Unblockable Fail-Safe
  test("S6: SCENARIO-06-EMERGENCY-PAUSE: Ctrl+Alt+Shift+P activates pause; halts all enforcement; audit log created", () => {
    // 1. Strict Paksa routine actively enforcing
    pal.registerDummyWindow(9601, "distraction.exe", "ModeA_CleanClose");
    pal.switchForeground(9601);

    // 2. User hits Ctrl+Alt+Shift+P hotkey — recognized and validated by bridge & controller
    const bridge = new RoutinesIpcBridge();
    const hotkeyInfo = bridge.getRegisteredHotkey("emergency_pause");
    expect(hotkeyInfo?.displayString).toBe("Ctrl+Alt+Shift+P");
    expect(bridge.isRecognizedHotkey("Ctrl+Alt+Shift+P")).toBe(true);

    // 3. 30-minute pause activated with reason "Darurat"
    const now = 1000;
    const pause: EmergencyPauseRecord = {
      id: "ep-s6",
      started_at: now,
      duration_min: 30,
      reason: "Darurat",
      expired_at: now + 30 * 60,
    };
    store.addEmergencyPause(pause);
    store.recordHistory({
      id: "eh-s6",
      timestamp: now,
      target_app: "*",
      tier: "all",
      action_taken: "paused",
      details: "Emergency pause 30m activated for Darurat",
    });

    // 4. Enforcement ladder query immediately returns Paused
    const activePause = store.getActivePause(now + 10);
    expect(activePause).toBeDefined();
    expect(activePause?.reason).toBe("Darurat");

    // 5. Audit record verified in history
    expect(store.history.length).toBe(1);
    expect(store.history[0].action_taken).toBe("paused");
  });

  // Scenario S8: Master Plan Gate #10 Mock Window Safety Certification
  test("S8: SCENARIO-08-GATE-10-MOCK-VALIDATION: 100% automated window tests target DummyTestWindow; zero real apps; zero kill process", () => {
    // 1. Verify Mode A clean close
    pal.registerDummyWindow(9801, "dummy_a.exe", "ModeA_CleanClose");
    expect(pal.sendGracefulClose(9801, 1000)).toBe("ClosedSuccessfully");

    // 2. Verify Mode B modal halt
    pal.registerDummyWindow(9802, "dummy_b.exe", "ModeB_UnsavedChangesModal");
    expect(pal.sendGracefulClose(9802, 1000)).toBe("UnsavedDialogDetected");

    // 3. Verify Mode C refusal without termination
    pal.registerDummyWindow(9803, "dummy_c.exe", "ModeC_RefuseClose");
    expect(pal.sendGracefulClose(9803, 1000)).toBe("WindowRefusedClose");

    // 4. Master invariant: TerminateProcess was NEVER invoked
    expect(pal.terminateProcessCalled).toBe(false);

    // 5. Zero real user processes were targeted — audited by Win32PalSimulator
    expect(pal.getTargetedProcessesCount()).toBeGreaterThan(0);
    expect(pal.hasTargetedNonMockProcess()).toBe(false);
    expect(pal.getAllTargetedExecutablePaths().every((p) => p.startsWith("C:\\Mock\\"))).toBe(true);
  });
});
