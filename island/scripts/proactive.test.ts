/**
 * island/scripts/proactive.test.ts
 *
 * Stage 10R: Storage Intelligence & Hime Proaktif
 * Comprehensive Test Suite for Proactive Interaction Engine:
 * - Attention budget rates (Pendiam, Seimbang, Cerewet)
 * - Quiet hours enforcement (23:00-06:00 default & custom ranges)
 * - Backoff after 3 dismissals & Golden Rule 6 (no repeat today)
 * - Catalog situations 1–22 & strict VC-U5 copy bounds
 * - SQLite preference persistence & "Selalu begini" rules
 *
 * Standards:
 * - bun:test with bun:sqlite
 * - VC-U5 / Doc 11 Copy Bounds: Title <= 16, Desc <= 36, Button <= 10, Bubble <= 40
 * - Brand compliance: zero unauthorized "Bloom" leaks
 */

import { describe, test, it, expect, beforeEach } from "bun:test";
import { Database } from "bun:sqlite";

// =============================================================================
// 1. DOMAIN TYPES & ENUMS (Docs 11 §2–§4)
// =============================================================================

export type AttentionMode = "Pendiam" | "Seimbang" | "Cerewet";

export type InteractionLevel = "Bisik" | "Colek" | "Tanya" | "Penting";

export type EyeExpression =
  | "penasaran"
  | "khawatir"
  | "netral"
  | "fokus"
  | "semangat"
  | "sedih"
  | "kesal"
  | "marah"
  | "senang"
  | "tenang"
  | "bangga"
  | "ngantuk"
  | "pusing"
  | "waspada";

export interface RateLimits {
  colekPerHour: number;
  tanyaPerHour: number;
  pentingPerHour: number; // typically Infinity
}

export const ATTENTION_BUDGETS: Record<AttentionMode, RateLimits> = {
  Pendiam: {
    colekPerHour: 1,
    tanyaPerHour: 1,
    pentingPerHour: Infinity,
  },
  Seimbang: {
    colekPerHour: 3,
    tanyaPerHour: 3,
    pentingPerHour: Infinity,
  },
  Cerewet: {
    colekPerHour: 6,
    tanyaPerHour: 5,
    pentingPerHour: Infinity,
  },
};

export interface QuietHoursConfig {
  enabled: boolean;
  startHour: number; // e.g. 23
  endHour: number;   // e.g. 6
}

export const DEFAULT_QUIET_HOURS: QuietHoursConfig = {
  enabled: true,
  startHour: 23,
  endHour: 6,
};

export interface InteractionCatalogItem {
  id: number;
  title: string;
  situation: string;
  level: InteractionLevel;
  isP0: boolean;
  eye: EyeExpression;
  bubbleText: string;
  buttons: string[];
}

export interface InteractionHistoryEntry {
  situationId: number;
  level: InteractionLevel;
  timestamp: number; // Unix seconds
  response?: "accepted" | "dismissed" | "ignored" | "muted";
}

export interface InteractionPreferenceRule {
  situationId: number;
  rule: "always_accept" | "always_dismiss" | "custom";
  revoked: boolean;
  updatedAt: number;
}

export interface BackoffRecord {
  situationId: number;
  dismissCount: number;
  backoffHours: number;
  lastDismissedAt: number;
}

// =============================================================================
// 2. COMPLETE CATALOG SITUATIONS 1–22 (Docs 11 §3)
// =============================================================================

export const CATALOG_SITUATIONS: InteractionCatalogItem[] = [
  {
    id: 1,
    title: "File Ganda",
    situation: "File ganda terdeteksi",
    level: "Tanya",
    isP0: false,
    eye: "penasaran",
    bubbleText: "File sudah ada. Ganti?",
    buttons: ["Ganti", "Simpan dua", "Hapus baru"],
  },
  {
    id: 2,
    title: "Unduhan Baru",
    situation: "File baru di Unduhan",
    level: "Tanya",
    isP0: false,
    eye: "penasaran",
    bubbleText: "Simpan ke mana?",
    buttons: ["Dokumen", "Lainnya", "Biarkan"],
  },
  {
    id: 3,
    title: "Storage Penuh",
    situation: "Storage hampir penuh",
    level: "Tanya",
    isP0: false,
    eye: "khawatir",
    bubbleText: "C: tersisa 8%",
    buttons: ["Lihat", "Nanti"],
  },
  {
    id: 4,
    title: "Tenggat Dekat",
    situation: "Tenggat dekat",
    level: "Penting",
    isP0: false,
    eye: "netral",
    bubbleText: "Laporan SKPL besok 19:00",
    buttons: ["Mulai", "Tunda", "Selesai"],
  },
  {
    id: 5,
    title: "Rutinitas Mulai",
    situation: "Rutinitas mulai",
    level: "Penting",
    isP0: false,
    eye: "fokus",
    bubbleText: "Waktunya belajar 30 mnt",
    buttons: ["Mulai", "Tunda 10m", "Lewati"],
  },
  {
    id: 6,
    title: "Rutin Terlewat",
    situation: "Rutinitas terlewat",
    level: "Tanya",
    isP0: false,
    eye: "sedih",
    bubbleText: "Belajar terlewat. Jadwal ulang?",
    buttons: ["Jam 21:00", "Besok", "Lewati"],
  },
  {
    id: 7,
    title: "Batas Pengalih",
    situation: "Pengalih melewati batas",
    level: "Penting",
    isP0: false,
    eye: "kesal",
    bubbleText: "Sudah 45 mnt di luar belajar",
    buttons: ["Lanjut", "Tunda 5m", "Jeda"],
  },
  {
    id: 8,
    title: "Ringkasan Pagi",
    situation: "Ringkasan pagi",
    level: "Colek",
    isP0: false,
    eye: "senang",
    bubbleText: "3 tugas hari ini. Belajar 19:00",
    buttons: ["Lihat", "Tutup"],
  },
  {
    id: 9,
    title: "Rekap Malam",
    situation: "Rekap malam",
    level: "Colek",
    isP0: false,
    eye: "tenang",
    bubbleText: "2 selesai, 1 geser besok",
    buttons: ["Lihat", "Tutup"],
  },
  {
    id: 10,
    title: "Tugas Selesai",
    situation: "Tugas selesai",
    level: "Colek",
    isP0: false,
    eye: "senang",
    bubbleText: "Selesai. Mantap.",
    buttons: [],
  },
  {
    id: 11,
    title: "Status Belajar",
    situation: "Idle saat blok belajar",
    level: "Tanya",
    isP0: false,
    eye: "netral",
    bubbleText: "Masih belajar?",
    buttons: ["Ya", "Istirahat", "Selesai"],
  },
  {
    id: 12,
    title: "Teks Asing",
    situation: "Teks asing",
    level: "Colek",
    isP0: false,
    eye: "penasaran",
    bubbleText: "Terjemahkan?",
    buttons: ["Terjemah", "Abaikan"],
  },
  {
    id: 13,
    title: "Baterai Lemah",
    situation: "Baterai rendah",
    level: "Tanya",
    isP0: false,
    eye: "khawatir",
    bubbleText: "Baterai 15%",
    buttons: ["Hemat", "Tutup"],
  },
  {
    id: 14,
    title: "HP Terhubung",
    situation: "HP terhubung/terputus; transfer selesai",
    level: "Colek",
    isP0: false,
    eye: "netral",
    bubbleText: "HP terhubung. 3 file siap.",
    buttons: ["Lihat"],
  },
  {
    id: 15,
    title: "Tugas AI Siap",
    situation: "Agent selesai tugas panjang",
    level: "Colek",
    isP0: false,
    eye: "senang",
    bubbleText: "Ringkasan siap",
    buttons: ["Buka"],
  },
  {
    id: 16,
    title: "Aksi Berisiko",
    situation: "Approval aksi berisiko",
    level: "Penting",
    isP0: true, // P0 Critical Approval
    eye: "waspada",
    bubbleText: "Pindahkan 12 file?",
    buttons: ["Setujui", "Tolak", "Detail"],
  },
  {
    id: 17,
    title: "Hime Offline",
    situation: "Runtime AI mati",
    level: "Tanya",
    isP0: false,
    eye: "khawatir",
    bubbleText: "Hime offline",
    buttons: ["Coba lagi"],
  },
  {
    id: 18,
    title: "Larut Malam",
    situation: "Larut malam",
    level: "Tanya",
    isP0: false,
    eye: "ngantuk",
    bubbleText: "Sudah larut. Tidur?",
    buttons: ["Bentar", "Oke"],
  },
  {
    id: 19,
    title: "Selamat Datang",
    situation: "Kembali setelah lama pergi",
    level: "Bisik",
    isP0: false,
    eye: "senang",
    bubbleText: "Selamat datang kembali.",
    buttons: [],
  },
  {
    id: 20,
    title: "Kategori App",
    situation: "Aplikasi baru sering dipakai",
    level: "Tanya",
    isP0: false,
    eye: "penasaran",
    bubbleText: "Kategori untuk Discord?",
    buttons: ["Belajar", "Netral", "Pengalih"],
  },
  {
    id: 21,
    title: "Clipboard Link",
    situation: "Clipboard berisi tautan/teks panjang",
    level: "Colek",
    isP0: false,
    eye: "netral",
    bubbleText: "Simpan sebagai tugas?",
    buttons: ["Ya", "Abaikan"],
  },
  {
    id: 22,
    title: "Pemberitahuan",
    situation: "Pembaruan/kesalahan penting",
    level: "Tanya",
    isP0: false,
    eye: "pusing",
    bubbleText: "Ada masalah. Lihat?",
    buttons: ["Lihat", "Nanti"],
  },
];

// =============================================================================
// 3. ENGINE LOGIC IMPLEMENTATION (Attention Budget, Quiet Hours, Backoff)
// =============================================================================

export interface DeliveryDecision {
  allowed: boolean;
  reason?: "quiet_hours" | "budget_exceeded" | "backoff_active" | "rejected_today" | "remembered_rule";
  suggestedAction?: string; // e.g. "bundle" or "auto_apply"
  summaryBundle?: string;
}

export class ProactiveEngineLogic {
  mode: AttentionMode;
  quietHours: QuietHoursConfig;
  history: InteractionHistoryEntry[] = [];
  backoffMap: Map<number, BackoffRecord> = new Map();
  ruleMap: Map<number, InteractionPreferenceRule> = new Map();

  constructor(mode: AttentionMode = "Seimbang", quietHours: QuietHoursConfig = DEFAULT_QUIET_HOURS) {
    this.mode = mode;
    this.quietHours = quietHours;
  }

  setMode(mode: AttentionMode) {
    this.mode = mode;
  }

  setQuietHours(config: QuietHoursConfig) {
    this.quietHours = config;
  }

  isQuietHour(unixSec: number): boolean {
    if (!this.quietHours.enabled) return false;
    const date = new Date(unixSec * 1000);
    const hour = date.getHours();

    if (this.quietHours.startHour > this.quietHours.endHour) {
      // Midnight span, e.g. 23:00 to 06:00
      return hour >= this.quietHours.startHour || hour < this.quietHours.endHour;
    } else {
      // Same day span, e.g. 13:00 to 15:00
      return hour >= this.quietHours.startHour && hour < this.quietHours.endHour;
    }
  }

  isRejectedToday(lastDismissedAt: number, nowSec: number): boolean {
    const d1 = new Date(lastDismissedAt * 1000);
    const d2 = new Date(nowSec * 1000);
    return (
      d1.getFullYear() === d2.getFullYear() &&
      d1.getMonth() === d2.getMonth() &&
      d1.getDate() === d2.getDate()
    );
  }

  recordDismissal(situationId: number, nowSec: number): BackoffRecord {
    const existing = this.backoffMap.get(situationId) || {
      situationId,
      dismissCount: 0,
      backoffHours: 0,
      lastDismissedAt: 0,
    };

    existing.dismissCount += 1;
    existing.lastDismissedAt = nowSec;

    // Exponential backoff after 3 dismissals:
    // 3 dismissals -> 2 hours
    // 4 dismissals -> 4 hours
    // 5 dismissals -> 8 hours
    // 6+ dismissals -> 16+ hours (max 72 hours)
    if (existing.dismissCount >= 3) {
      const exponent = existing.dismissCount - 3;
      existing.backoffHours = Math.min(2 * Math.pow(2, exponent), 72);
    }

    this.backoffMap.set(situationId, existing);
    return existing;
  }

  resetBackoff(situationId: number) {
    this.backoffMap.delete(situationId);
  }

  setRememberRule(situationId: number, rule: "always_accept" | "always_dismiss" | "custom", nowSec: number) {
    this.ruleMap.set(situationId, {
      situationId,
      rule,
      revoked: false,
      updatedAt: nowSec,
    });
  }

  revokeRememberRule(situationId: number) {
    const r = this.ruleMap.get(situationId);
    if (r) {
      r.revoked = true;
    }
  }

  evaluate(situation: InteractionCatalogItem, nowSec: number): DeliveryDecision {
    // 1. Check permanent remember rule
    const rule = this.ruleMap.get(situation.id);
    if (rule && !rule.revoked) {
      return {
        allowed: false,
        reason: "remembered_rule",
        suggestedAction: rule.rule,
      };
    }

    // 2. Quiet hours check: only P0 allowed during quiet hours
    if (this.isQuietHour(nowSec)) {
      if (!situation.isP0) {
        return {
          allowed: false,
          reason: "quiet_hours",
        };
      }
    }

    // 3. Backoff & Golden Rule 6 check
    const backoff = this.backoffMap.get(situation.id);
    if (backoff) {
      // Golden Rule 6: do not repeat something rejected today
      if (this.isRejectedToday(backoff.lastDismissedAt, nowSec)) {
        return {
          allowed: false,
          reason: "rejected_today",
        };
      }

      // Check exponential backoff window
      if (backoff.backoffHours > 0) {
        const cooldownSecs = backoff.backoffHours * 3600;
        if (nowSec - backoff.lastDismissedAt < cooldownSecs) {
          return {
            allowed: false,
            reason: "backoff_active",
          };
        }
      }
    }

    // 4. Attention budget check (past 1 hour = 3600 seconds)
    const budget = ATTENTION_BUDGETS[this.mode];
    const oneHourAgo = nowSec - 3600;
    const recentEvents = this.history.filter((h) => h.timestamp >= oneHourAgo);

    if (situation.level === "Colek") {
      const colekCount = recentEvents.filter((h) => h.level === "Colek").length;
      if (colekCount >= budget.colekPerHour) {
        return {
          allowed: false,
          reason: "budget_exceeded",
          suggestedAction: "bundle",
          summaryBundle: `${colekCount + 1} hal menunggu`,
        };
      }
    } else if (situation.level === "Tanya") {
      const tanyaCount = recentEvents.filter((h) => h.level === "Tanya").length;
      if (tanyaCount >= budget.tanyaPerHour) {
        return {
          allowed: false,
          reason: "budget_exceeded",
          suggestedAction: "bundle",
          summaryBundle: `${tanyaCount + 1} hal menunggu`,
        };
      }
    }

    return { allowed: true };
  }

  deliver(situation: InteractionCatalogItem, nowSec: number) {
    this.history.push({
      situationId: situation.id,
      level: situation.level,
      timestamp: nowSec,
    });
  }
}

// =============================================================================
// 4. TEST SUITE
// =============================================================================

describe("Stage 10R: Proactive Interaction Engine", () => {
  let engine: ProactiveEngineLogic;
  let db: Database;

  beforeEach(() => {
    engine = new ProactiveEngineLogic("Seimbang");
    db = new Database(":memory:");

    // Initialize proactive storage tables
    db.run(`
      CREATE TABLE IF NOT EXISTS interaction_prefs (
        situation_id INTEGER PRIMARY KEY,
        rule TEXT,
        dismiss_count INTEGER DEFAULT 0,
        backoff_hours INTEGER DEFAULT 0,
        last_dismissed_at INTEGER,
        revoked INTEGER DEFAULT 0,
        updated_at INTEGER
      );
    `);

    db.run(`
      CREATE TABLE IF NOT EXISTS proactive_settings (
        id TEXT PRIMARY KEY,
        mode TEXT DEFAULT 'Seimbang',
        quiet_hours_enabled INTEGER DEFAULT 1,
        quiet_hours_start INTEGER DEFAULT 23,
        quiet_hours_end INTEGER DEFAULT 6,
        is_enabled INTEGER DEFAULT 1,
        updated_at INTEGER
      );
    `);
  });

  // ---------------------------------------------------------------------------
  // TIER 1: ATTENTION BUDGET RATES
  // ---------------------------------------------------------------------------
  describe("Tier 1: Attention Budget Rates", () => {
    beforeEach(() => {
      // Disable quiet hours in Tier 1 to isolate attention budget rate testing
      engine.setQuietHours({ enabled: false, startHour: 23, endHour: 6 });
    });

    test("Pendiam mode strictly enforces 1 Colek and 1 Tanya per hour", () => {
      engine.setMode("Pendiam");
      const baseTime = 1700000000; // Reference timestamp (12:00 UTC)

      const colekItem = CATALOG_SITUATIONS.find((s) => s.id === 8)!; // Colek: Ringkasan pagi
      const tanyaItem = CATALOG_SITUATIONS.find((s) => s.id === 1)!; // Tanya: File ganda

      // 1st Colek -> Allowed
      const d1 = engine.evaluate(colekItem, baseTime);
      expect(d1.allowed).toBe(true);
      engine.deliver(colekItem, baseTime);

      // 2nd Colek within same hour -> Blocked
      const d2 = engine.evaluate(colekItem, baseTime + 1800);
      expect(d2.allowed).toBe(false);
      expect(d2.reason).toBe("budget_exceeded");
      expect(d2.suggestedAction).toBe("bundle");

      // 1st Tanya -> Allowed
      const t1 = engine.evaluate(tanyaItem, baseTime + 1800);
      expect(t1.allowed).toBe(true);
      engine.deliver(tanyaItem, baseTime + 1800);

      // 2nd Tanya within same hour -> Blocked
      const t2 = engine.evaluate(tanyaItem, baseTime + 2400);
      expect(t2.allowed).toBe(false);
      expect(t2.reason).toBe("budget_exceeded");
    });

    test("Seimbang mode allows 3 Colek and 3 Tanya per hour", () => {
      engine.setMode("Seimbang");
      const baseTime = 1700000000;
      const colekItem = CATALOG_SITUATIONS.find((s) => s.id === 8)!;

      // Deliver 3 Coleks
      for (let i = 0; i < 3; i++) {
        const res = engine.evaluate(colekItem, baseTime + i * 600);
        expect(res.allowed).toBe(true);
        engine.deliver(colekItem, baseTime + i * 600);
      }

      // 4th Colek exceeds budget
      const overflow = engine.evaluate(colekItem, baseTime + 2000);
      expect(overflow.allowed).toBe(false);
      expect(overflow.reason).toBe("budget_exceeded");
      expect(overflow.summaryBundle).toBe("4 hal menunggu");
    });

    test("Cerewet mode allows 6 Colek and 5 Tanya per hour", () => {
      engine.setMode("Cerewet");
      const baseTime = 1700000000;
      const tanyaItem = CATALOG_SITUATIONS.find((s) => s.id === 1)!;

      // Deliver 5 Tanya items
      for (let i = 0; i < 5; i++) {
        const res = engine.evaluate(tanyaItem, baseTime + i * 300);
        expect(res.allowed).toBe(true);
        engine.deliver(tanyaItem, baseTime + i * 300);
      }

      // 6th Tanya exceeds budget
      const overflow = engine.evaluate(tanyaItem, baseTime + 1600);
      expect(overflow.allowed).toBe(false);
      expect(overflow.reason).toBe("budget_exceeded");
    });

    test("Penting (critical) interactions are never restricted by hourly limits", () => {
      engine.setMode("Pendiam");
      const baseTime = 1700000000;
      const pentingItem = CATALOG_SITUATIONS.find((s) => s.id === 16)!; // Aksi berisiko (P0)

      // Even with 10 Penting alerts in 10 minutes, all are allowed
      for (let i = 0; i < 10; i++) {
        const res = engine.evaluate(pentingItem, baseTime + i * 60);
        expect(res.allowed).toBe(true);
        engine.deliver(pentingItem, baseTime + i * 60);
      }
    });

    test("Sliding 1-hour window recovers capacity after 3600 seconds", () => {
      engine.setMode("Pendiam");
      const baseTime = 1700000000;
      const colekItem = CATALOG_SITUATIONS.find((s) => s.id === 8)!;

      engine.deliver(colekItem, baseTime);

      // Blocked at 3500s
      expect(engine.evaluate(colekItem, baseTime + 3500).allowed).toBe(false);

      // Unblocked at 3601s (past the 1-hour window)
      expect(engine.evaluate(colekItem, baseTime + 3601).allowed).toBe(true);
    });
  });

  // ---------------------------------------------------------------------------
  // TIER 2: QUIET HOURS ENFORCEMENT
  // ---------------------------------------------------------------------------
  describe("Tier 2: Quiet Hours Enforcement", () => {
    test("Identifies quiet hours spanning midnight correctly (23:00 - 06:00)", () => {
      const midnightSpan: QuietHoursConfig = { enabled: true, startHour: 23, endHour: 6 };
      engine.setQuietHours(midnightSpan);

      // Construct dates at specific local hours
      const testHour = (h: number, m: number = 0) => {
        const d = new Date(2026, 10, 22, h, m, 0);
        return Math.floor(d.getTime() / 1000);
      };

      expect(engine.isQuietHour(testHour(23, 0))).toBe(true);
      expect(engine.isQuietHour(testHour(23, 59))).toBe(true);
      expect(engine.isQuietHour(testHour(0, 0))).toBe(true);
      expect(engine.isQuietHour(testHour(3, 30))).toBe(true);
      expect(engine.isQuietHour(testHour(5, 59))).toBe(true);

      // Active day hours
      expect(engine.isQuietHour(testHour(6, 0))).toBe(false);
      expect(engine.isQuietHour(testHour(12, 0))).toBe(false);
      expect(engine.isQuietHour(testHour(22, 59))).toBe(false);
    });

    test("During quiet hours, non-P0 interactions are suppressed", () => {
      const quietTimestamp = Math.floor(new Date(2026, 10, 22, 2, 0, 0).getTime() / 1000);
      const normalItem = CATALOG_SITUATIONS.find((s) => s.id === 1)!; // File ganda (non-P0)

      const res = engine.evaluate(normalItem, quietTimestamp);
      expect(res.allowed).toBe(false);
      expect(res.reason).toBe("quiet_hours");
    });

    test("During quiet hours, P0 critical items (Approval aksi berisiko) pass through", () => {
      const quietTimestamp = Math.floor(new Date(2026, 10, 22, 3, 0, 0).getTime() / 1000);
      const p0Item = CATALOG_SITUATIONS.find((s) => s.id === 16)!; // Aksi berisiko (P0)

      const res = engine.evaluate(p0Item, quietTimestamp);
      expect(res.allowed).toBe(true);
    });

    test("Eye expression is 'ngantuk' during late night situations without aggressive text", () => {
      const lateNightItem = CATALOG_SITUATIONS.find((s) => s.id === 18)!;
      expect(lateNightItem.eye).toBe("ngantuk");
      expect(lateNightItem.bubbleText.length).toBeLessThanOrEqual(40);
    });
  });

  // ---------------------------------------------------------------------------
  // TIER 3: BACKOFF AFTER 3 DISMISSALS & GOLDEN RULES
  // ---------------------------------------------------------------------------
  describe("Tier 3: Backoff & Learning From Rejection", () => {
    test("Dismissals 1 and 2 do not trigger exponential backoff", () => {
      const now = 1700000000;
      const b1 = engine.recordDismissal(1, now);
      expect(b1.dismissCount).toBe(1);
      expect(b1.backoffHours).toBe(0);

      const b2 = engine.recordDismissal(1, now + 100);
      expect(b2.dismissCount).toBe(2);
      expect(b2.backoffHours).toBe(0);
    });

    test("3rd dismissal triggers exponential backoff (2 hours minimum)", () => {
      const now = 1700000000;
      engine.recordDismissal(1, now);
      engine.recordDismissal(1, now + 100);
      const b3 = engine.recordDismissal(1, now + 200);

      expect(b3.dismissCount).toBe(3);
      expect(b3.backoffHours).toBe(2);
    });

    test("4th and 5th dismissals scale backoff exponentially (4h, 8h)", () => {
      const now = 1700000000;
      engine.recordDismissal(1, now);
      engine.recordDismissal(1, now);
      engine.recordDismissal(1, now);

      const b4 = engine.recordDismissal(1, now);
      expect(b4.backoffHours).toBe(4);

      const b5 = engine.recordDismissal(1, now);
      expect(b5.backoffHours).toBe(8);
    });

    test("Golden Rule 6: Items dismissed today cannot be repeated today", () => {
      // Create date today at 10:00
      const today10 = Math.floor(new Date(2026, 10, 22, 10, 0, 0).getTime() / 1000);
      const today14 = Math.floor(new Date(2026, 10, 22, 14, 0, 0).getTime() / 1000);
      const tomorrow10 = Math.floor(new Date(2026, 10, 23, 10, 0, 0).getTime() / 1000);

      engine.recordDismissal(2, today10);
      const item = CATALOG_SITUATIONS.find((s) => s.id === 2)!;

      // Same day at 14:00 -> Blocked by rejected_today
      const resToday = engine.evaluate(item, today14);
      expect(resToday.allowed).toBe(false);
      expect(resToday.reason).toBe("rejected_today");

      // Next day -> Unblocked by rejected_today
      const resTomorrow = engine.evaluate(item, tomorrow10);
      expect(resTomorrow.allowed).toBe(true);
    });

    test("Reset backoff restores default clean state", () => {
      const now = 1700000000;
      for (let i = 0; i < 5; i++) {
        engine.recordDismissal(3, now);
      }
      expect(engine.backoffMap.get(3)?.backoffHours).toBe(8);

      engine.resetBackoff(3);
      expect(engine.backoffMap.has(3)).toBe(false);
    });

    test("'Selalu begini' preference rule overrides prompts and can be revoked", () => {
      const now = 1700000000;
      const item = CATALOG_SITUATIONS.find((s) => s.id === 1)!;

      // Set remember rule
      engine.setRememberRule(1, "always_accept", now);
      const decision1 = engine.evaluate(item, now);
      expect(decision1.allowed).toBe(false);
      expect(decision1.reason).toBe("remembered_rule");
      expect(decision1.suggestedAction).toBe("always_accept");

      // Revoke rule
      engine.revokeRememberRule(1);
      const decision2 = engine.evaluate(item, now + 100);
      expect(decision2.allowed).toBe(true);
    });
  });

  // ---------------------------------------------------------------------------
  // TIER 4: CATALOG SITUATIONS 1–22 & VC-U5 COPY BOUNDS
  // ---------------------------------------------------------------------------
  describe("Tier 4: Catalog Situations 1–22 & Copy Bounds", () => {
    test("Catalog contains exactly 22 situations", () => {
      expect(CATALOG_SITUATIONS.length).toBe(22);
      for (let i = 1; i <= 22; i++) {
        const found = CATALOG_SITUATIONS.find((s) => s.id === i);
        expect(found).toBeDefined();
        expect(found!.id).toBe(i);
      }
    });

    test("All titles adhere to strict copy limit (Title <= 16 chars)", () => {
      for (const item of CATALOG_SITUATIONS) {
        expect(item.title.length).toBeLessThanOrEqual(16);
      }
    });

    test("All bubble texts adhere to strict copy limit (Bubble <= 40 chars)", () => {
      for (const item of CATALOG_SITUATIONS) {
        expect(item.bubbleText.length).toBeLessThanOrEqual(40);
      }
    });

    test("All action buttons adhere to strict copy limit (Button <= 10 chars)", () => {
      for (const item of CATALOG_SITUATIONS) {
        for (const btn of item.buttons) {
          expect(btn.length).toBeLessThanOrEqual(10);
        }
      }
    });

    test("All 'Tanya' situations have <= 3 action choices", () => {
      const tanyaItems = CATALOG_SITUATIONS.filter((s) => s.level === "Tanya");
      expect(tanyaItems.length).toBeGreaterThan(0);
      for (const item of tanyaItems) {
        expect(item.buttons.length).toBeLessThanOrEqual(3);
      }
    });

    test("Situation #16 is marked as P0 Critical Approval", () => {
      const s16 = CATALOG_SITUATIONS.find((s) => s.id === 16)!;
      expect(s16.isP0).toBe(true);
      expect(s16.level).toBe("Penting");
      expect(s16.eye).toBe("waspada");
      expect(s16.buttons).toEqual(["Setujui", "Tolak", "Detail"]);
    });

    test("Zero unauthorized 'Bloom' references in catalog copy", () => {
      for (const item of CATALOG_SITUATIONS) {
        expect(item.title.toLowerCase()).not.toContain("bloom");
        expect(item.situation.toLowerCase()).not.toContain("bloom");
        expect(item.bubbleText.toLowerCase()).not.toContain("bloom");
        for (const btn of item.buttons) {
          expect(btn.toLowerCase()).not.toContain("bloom");
        }
      }
    });
  });

  // ---------------------------------------------------------------------------
  // TIER 5: SQLITE PERSISTENCE CONTRACT
  // ---------------------------------------------------------------------------
  describe("Tier 5: SQLite Database Persistence", () => {
    test("Persists and updates proactive settings record", () => {
      const now = Math.floor(Date.now() / 1000);
      db.run(
        `INSERT INTO proactive_settings (id, mode, quiet_hours_enabled, quiet_hours_start, quiet_hours_end, is_enabled, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?)`,
        ["default", "Cerewet", 1, 22, 7, 1, now]
      );

      const row: any = db.query("SELECT * FROM proactive_settings WHERE id = ?").get("default");
      expect(row).toBeDefined();
      expect(row.mode).toBe("Cerewet");
      expect(row.quiet_hours_start).toBe(22);
      expect(row.quiet_hours_end).toBe(7);
      expect(row.is_enabled).toBe(1);
    });

    test("Persists backoff and preference rules in interaction_prefs", () => {
      const now = Math.floor(Date.now() / 1000);
      db.run(
        `INSERT INTO interaction_prefs (situation_id, rule, dismiss_count, backoff_hours, last_dismissed_at, revoked, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?)`,
        [1, "always_accept", 3, 2, now, 0, now]
      );

      const row: any = db.query("SELECT * FROM interaction_prefs WHERE situation_id = ?").get(1);
      expect(row).toBeDefined();
      expect(row.rule).toBe("always_accept");
      expect(row.dismiss_count).toBe(3);
      expect(row.backoff_hours).toBe(2);
      expect(row.revoked).toBe(0);

      // Update revocation
      db.run("UPDATE interaction_prefs SET revoked = 1 WHERE situation_id = ?", [1]);
      const updated: any = db.query("SELECT revoked FROM interaction_prefs WHERE situation_id = ?").get(1);
      expect(updated.revoked).toBe(1);
    });
  });
});
