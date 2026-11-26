/**
 * island/scripts/mobile_parity.test.ts
 *
 * Stage 11R: Mobile & Devices Parity + Android Enforcement
 * Comprehensive Test Suite for Android Companion Parity:
 * - 5-tab bottom navigation (Hime, Tugas, Fokus, Simpanan, Lainnya)
 * - Tier A/B/C enforcement logic, cooling-off window, protected apps list
 * - Two-way sync protocol contract (tasks, routines, screen-time, shelf, approvals)
 * - OEM battery optimization intent mappings & recovery handlers
 * - VC-U5 copy bounds & brand compliance (zero "Bloom" leaks)
 *
 * Standards:
 * - bun:test with bun:sqlite
 * - Doc 16 §2–§6 & ADR 006
 */

import { describe, test, it, expect, beforeEach } from "bun:test";
import { Database } from "bun:sqlite";

// =============================================================================
// 1. DOMAIN TYPES & CONSTANTS (Doc 16 §2–§6)
// =============================================================================

export type MobileTabId = "chat" | "tasks" | "focus" | "shelf" | "more";

export interface MobileTabDefinition {
  id: MobileTabId;
  label: string;
  icon: string;
  route: string;
  description: string;
}

export const MOBILE_NAVIGATION_TABS: MobileTabDefinition[] = [
  {
    id: "chat",
    label: "Hime",
    icon: "message-circle",
    route: "/chat",
    description: "Chat satu thread & skill",
  },
  {
    id: "tasks",
    label: "Tugas",
    icon: "check-square",
    route: "/tasks",
    description: "Tugas & tenggat hari ini",
  },
  {
    id: "focus",
    label: "Fokus",
    icon: "target",
    route: "/focus",
    description: "Rutinitas & sesi aktif",
  },
  {
    id: "shelf",
    label: "Simpanan",
    icon: "bookmark",
    route: "/shelf",
    description: "Teks & file sementara",
  },
  {
    id: "more",
    label: "Lainnya",
    icon: "more-horizontal",
    route: "/more",
    description: "Perangkat, storage, setelan",
  },
];

export type MobileEnforcementTier = "A_INGATKAN" | "B_TEGAS" | "C_PAKSA";

export interface ProtectedAppEntry {
  packageName: string;
  category: "phone" | "emergency" | "navigation" | "system_ui" | "self";
  displayName: string;
}

export const PROTECTED_MOBILE_APPS: ProtectedAppEntry[] = [
  { packageName: "com.android.phone", category: "phone", displayName: "Telepon Sistem" },
  { packageName: "com.google.android.dialer", category: "phone", displayName: "Google Dialer" },
  { packageName: "com.samsung.android.dialer", category: "phone", displayName: "Samsung Dialer" },
  { packageName: "com.android.emergency", category: "emergency", displayName: "Kontak Darurat" },
  { packageName: "com.google.android.apps.safetyhub", category: "emergency", displayName: "Safety Hub" },
  { packageName: "com.google.android.apps.maps", category: "navigation", displayName: "Google Maps" },
  { packageName: "com.waze", category: "navigation", displayName: "Waze" },
  { packageName: "com.android.systemui", category: "system_ui", displayName: "System UI" },
  { packageName: "com.google.android.apps.nexuslauncher", category: "system_ui", displayName: "Pixel Launcher" },
  { packageName: "com.sec.android.app.launcher", category: "system_ui", displayName: "One UI Home" },
  { packageName: "ai.hime.mobile", category: "self", displayName: "Hime Mobile" },
];

export interface OemIntentResolution {
  manufacturer: string;
  targetPackage: string;
  targetActivity?: string;
  action: string;
  isCustomOemIntent: boolean;
}

export const OEM_BATTERY_INTENT_MAP: Record<string, { pkg: string; activity?: string; action: string }> = {
  xiaomi: {
    pkg: "com.miui.securitycenter",
    activity: "com.miui.permcenter.autostart.AutoStartManagementActivity",
    action: "miui.intent.action.OP_AUTO_START",
  },
  redmi: {
    pkg: "com.miui.securitycenter",
    activity: "com.miui.permcenter.autostart.AutoStartManagementActivity",
    action: "miui.intent.action.OP_AUTO_START",
  },
  poco: {
    pkg: "com.miui.securitycenter",
    activity: "com.miui.permcenter.autostart.AutoStartManagementActivity",
    action: "miui.intent.action.OP_AUTO_START",
  },
  oppo: {
    pkg: "com.coloros.safecenter",
    activity: "com.coloros.safecenter.permission.startup.StartupAppListActivity",
    action: "android.intent.action.MAIN",
  },
  realme: {
    pkg: "com.coloros.safecenter",
    activity: "com.coloros.safecenter.permission.startup.StartupAppListActivity",
    action: "android.intent.action.MAIN",
  },
  vivo: {
    pkg: "com.iqoo.secure",
    activity: "com.iqoo.secure.ui.phoneoptimize.AddWhiteListActivity",
    action: "android.intent.action.MAIN",
  },
  samsung: {
    pkg: "com.samsung.android.lool",
    activity: "com.samsung.android.sm.ui.battery.BatteryActivity",
    action: "android.settings.IGNORE_BATTERY_OPTIMIZATION_SETTINGS",
  },
  huawei: {
    pkg: "com.huawei.systemmanager",
    activity: "com.huawei.systemmanager.startupmgr.ui.StartupNormalAppListActivity",
    action: "android.intent.action.MAIN",
  },
};

export interface SyncTaskItem {
  id: string;
  title: string;
  category: string;
  completed: boolean;
  dueAt: number | null;
  updatedAt: number;
}

export interface SyncRoutineItem {
  id: string;
  title: string;
  scheduledTime: string;
  days: string[];
  active: boolean;
  updatedAt: number;
}

export interface SyncShelfItem {
  id: string;
  kind: "text" | "file";
  content: string;
  pinned: boolean;
  createdAt: number;
}

export interface MobileScreenTimeEntry {
  packageName: string;
  category: "belajar" | "netral" | "pengalih";
  durationSeconds: number;
  dateStr: string;
  timestamp: number;
}

export interface DeviceSyncPayload {
  deviceId: string;
  protocolVersion: number;
  timestamp: number;
  tasks: SyncTaskItem[];
  routines: SyncRoutineItem[];
  shelfItems: SyncShelfItem[];
  pendingApprovals: Array<{ id: string; action: string; risk: string }>;
}

export interface ScreenTimeSyncPayload {
  deviceId: string;
  timestamp: number;
  entries: MobileScreenTimeEntry[];
}

// =============================================================================
// 2. SIMULATION & DOMAIN ENGINES
// =============================================================================

export class MobileEnforcementEngine {
  activeRoutine: { id: string; title: string; tier: MobileEnforcementTier } | null = null;
  distractionPackages: Set<string> = new Set();
  emergencyBreakUntil: number = 0;
  coolingOffUntil: number = 0;

  setActiveRoutine(routine: { id: string; title: string; tier: MobileEnforcementTier } | null) {
    this.activeRoutine = routine;
  }

  setDistractionPackages(packages: string[]) {
    this.distractionPackages = new Set(packages);
  }

  isProtected(pkg: string): boolean {
    return PROTECTED_MOBILE_APPS.some((app) => app.packageName.toLowerCase() === pkg.toLowerCase());
  }

  applyEmergencyBreak(holdDurationSec: number, reason: string, nowSec: number, breakMinutes: number = 10): boolean {
    // Requires >= 3.0 seconds hold and non-empty reason
    if (holdDurationSec < 3.0) return false;
    if (!reason || reason.trim().length === 0) return false;

    this.emergencyBreakUntil = nowSec + breakMinutes * 60;
    return true;
  }

  applyCoolingOff(seconds: number, nowSec: number) {
    this.coolingOffUntil = nowSec + seconds;
  }

  evaluateForegroundApp(
    currentPackage: string,
    nowSec: number
  ): {
    actionRequired: boolean;
    tier?: MobileEnforcementTier;
    actionType?: "NOTIFY" | "OVERLAY" | "HOME_ACTION";
    reason?: string;
  } {
    // 1. Protected applications are NEVER enforced
    if (this.isProtected(currentPackage)) {
      return { actionRequired: false, reason: "protected_app" };
    }

    // 2. Check emergency break
    if (nowSec < this.emergencyBreakUntil) {
      return { actionRequired: false, reason: "emergency_break_active" };
    }

    // 3. Check cooling-off window
    if (nowSec < this.coolingOffUntil) {
      return { actionRequired: false, reason: "cooling_off_active" };
    }

    // 4. Check if active routine is enforcing
    if (!this.activeRoutine) {
      return { actionRequired: false, reason: "no_active_routine" };
    }

    // 5. Check if the app is a designated distraction
    if (!this.distractionPackages.has(currentPackage)) {
      return { actionRequired: false, reason: "allowed_app" };
    }

    // Enforcement applies based on active routine's tier
    switch (this.activeRoutine.tier) {
      case "A_INGATKAN":
        return {
          actionRequired: true,
          tier: "A_INGATKAN",
          actionType: "NOTIFY",
          reason: "distraction_reminder",
        };
      case "B_TEGAS":
        return {
          actionRequired: true,
          tier: "B_TEGAS",
          actionType: "OVERLAY",
          reason: "distraction_overlay_block",
        };
      case "C_PAKSA":
        return {
          actionRequired: true,
          tier: "C_PAKSA",
          actionType: "HOME_ACTION",
          reason: "distraction_return_home",
        };
    }
  }
}

export class OemReliabilityHelper {
  static resolveBatteryIntent(manufacturer: string): OemIntentResolution {
    const key = manufacturer.trim().toLowerCase();
    const mapping = OEM_BATTERY_INTENT_MAP[key];

    if (mapping) {
      return {
        manufacturer,
        targetPackage: mapping.pkg,
        targetActivity: mapping.activity,
        action: mapping.action,
        isCustomOemIntent: true,
      };
    }

    // Standard Android fallback
    return {
      manufacturer,
      targetPackage: "com.android.settings",
      action: "android.settings.IGNORE_BATTERY_OPTIMIZATION_SETTINGS",
      isCustomOemIntent: false,
    };
  }
}

// =============================================================================
// 3. TEST SUITES
// =============================================================================

describe("Stage 11R: Mobile Parity & Android Companion Suite", () => {
  let db: Database;
  let enforcement: MobileEnforcementEngine;

  beforeEach(() => {
    db = new Database(":memory:");
    enforcement = new MobileEnforcementEngine();

    // Setup local mobile test tables
    db.run(`
      CREATE TABLE IF NOT EXISTS mobile_tasks (
        id TEXT PRIMARY KEY,
        title TEXT NOT NULL,
        category TEXT NOT NULL,
        completed INTEGER DEFAULT 0,
        due_at INTEGER,
        updated_at INTEGER NOT NULL
      );
    `);

    db.run(`
      CREATE TABLE IF NOT EXISTS mobile_shelf (
        id TEXT PRIMARY KEY,
        kind TEXT NOT NULL,
        content TEXT NOT NULL,
        pinned INTEGER DEFAULT 0,
        created_at INTEGER NOT NULL
      );
    `);

    db.run(`
      CREATE TABLE IF NOT EXISTS mobile_screentime (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        package_name TEXT NOT NULL,
        category TEXT NOT NULL,
        duration_seconds INTEGER NOT NULL,
        date_str TEXT NOT NULL,
        timestamp INTEGER NOT NULL
      );
    `);
  });

  // ---------------------------------------------------------------------------
  // TIER 1: 5-TAB NAVIGATION & ROUTES
  // ---------------------------------------------------------------------------
  describe("Tier 1: 5-Tab Navigation & Route Contracts", () => {
    test("Navigation bar strictly defines at most 5 tabs", () => {
      expect(MOBILE_NAVIGATION_TABS.length).toBeLessThanOrEqual(5);
      expect(MOBILE_NAVIGATION_TABS.length).toBe(5);
    });

    test("All 5 canonical tabs exist in exact sequence (Hime, Tugas, Fokus, Simpanan, Lainnya)", () => {
      const expectedIds: MobileTabId[] = ["chat", "tasks", "focus", "shelf", "more"];
      const expectedLabels = ["Hime", "Tugas", "Fokus", "Simpanan", "Lainnya"];

      expect(MOBILE_NAVIGATION_TABS.map((t) => t.id)).toEqual(expectedIds);
      expect(MOBILE_NAVIGATION_TABS.map((t) => t.label)).toEqual(expectedLabels);
    });

    test("Each tab route starts with forward slash and has a valid icon", () => {
      for (const tab of MOBILE_NAVIGATION_TABS) {
        expect(tab.route.startsWith("/")).toBe(true);
        expect(tab.icon.length).toBeGreaterThan(0);
        expect(tab.description.length).toBeGreaterThan(0);
      }
    });
  });

  // ---------------------------------------------------------------------------
  // TIER 2: TIER A/B/C ENFORCEMENT & SAFETY INVARIANTS
  // ---------------------------------------------------------------------------
  describe("Tier 2: Android Tier A/B/C Enforcement & Safety Invariants", () => {
    test("Protected applications are never intercepted regardless of tier", () => {
      enforcement.setActiveRoutine({
        id: "routine-belajar",
        title: "Belajar UTBK",
        tier: "C_PAKSA",
      });
      enforcement.setDistractionPackages(["com.android.phone", "com.google.android.apps.maps"]);

      const now = 1700000000;
      // Phone dialer
      const resPhone = enforcement.evaluateForegroundApp("com.android.phone", now);
      expect(resPhone.actionRequired).toBe(false);
      expect(resPhone.reason).toBe("protected_app");

      // Maps
      const resMaps = enforcement.evaluateForegroundApp("com.google.android.apps.maps", now);
      expect(resMaps.actionRequired).toBe(false);
      expect(resMaps.reason).toBe("protected_app");

      // Emergency safety hub
      const resSafety = enforcement.evaluateForegroundApp("com.google.android.apps.safetyhub", now);
      expect(resSafety.actionRequired).toBe(false);

      // Self companion package
      const resSelf = enforcement.evaluateForegroundApp("ai.hime.mobile", now);
      expect(resSelf.actionRequired).toBe(false);
    });

    test("Tier A (Ingatkan) emits NOTIFY action without blocking overlay", () => {
      enforcement.setActiveRoutine({
        id: "routine-santai",
        title: "Membaca Buku",
        tier: "A_INGATKAN",
      });
      enforcement.setDistractionPackages(["com.instagram.android"]);

      const res = enforcement.evaluateForegroundApp("com.instagram.android", 1700000000);
      expect(res.actionRequired).toBe(true);
      expect(res.tier).toBe("A_INGATKAN");
      expect(res.actionType).toBe("NOTIFY");
    });

    test("Tier B (Tegas) emits OVERLAY action without terminating process", () => {
      enforcement.setActiveRoutine({
        id: "routine-fokus",
        title: "Coding Project",
        tier: "B_TEGAS",
      });
      enforcement.setDistractionPackages(["com.zhiliaoapp.musically"]); // TikTok

      const res = enforcement.evaluateForegroundApp("com.zhiliaoapp.musically", 1700000000);
      expect(res.actionRequired).toBe(true);
      expect(res.tier).toBe("B_TEGAS");
      expect(res.actionType).toBe("OVERLAY");
    });

    test("Tier C (Paksa) emits HOME_ACTION (return home) without killing app", () => {
      enforcement.setActiveRoutine({
        id: "routine-ujian",
        title: "Persiapan Sidang",
        tier: "C_PAKSA",
      });
      enforcement.setDistractionPackages(["com.twitter.android"]);

      const res = enforcement.evaluateForegroundApp("com.twitter.android", 1700000000);
      expect(res.actionRequired).toBe(true);
      expect(res.tier).toBe("C_PAKSA");
      expect(res.actionType).toBe("HOME_ACTION");
    });

    test("Jeda Darurat requires >= 3.0 seconds hold and non-empty reason", () => {
      enforcement.setActiveRoutine({
        id: "r1",
        title: "Belajar",
        tier: "B_TEGAS",
      });
      enforcement.setDistractionPackages(["com.twitter.android"]);
      const now = 1700000000;

      // Hold < 3.0s -> Rejected
      const rejected1 = enforcement.applyEmergencyBreak(2.9, "Panggilan darurat", now);
      expect(rejected1).toBe(false);

      // Empty reason -> Rejected
      const rejected2 = enforcement.applyEmergencyBreak(3.5, "   ", now);
      expect(rejected2).toBe(false);

      // Valid hold + reason -> Accepted (pauses enforcement for 10 minutes)
      const accepted = enforcement.applyEmergencyBreak(3.2, "Mengecek pesan penting ortu", now, 10);
      expect(accepted).toBe(true);

      // Evaluated at now + 300s (5 mins into 10 min break) -> Suppressed
      const evalDuringBreak = enforcement.evaluateForegroundApp("com.twitter.android", now + 300);
      expect(evalDuringBreak.actionRequired).toBe(false);
      expect(evalDuringBreak.reason).toBe("emergency_break_active");

      // Evaluated at now + 601s (after 10 mins) -> Re-enforces
      const evalAfterBreak = enforcement.evaluateForegroundApp("com.twitter.android", now + 601);
      expect(evalAfterBreak.actionRequired).toBe(true);
      expect(evalAfterBreak.actionType).toBe("OVERLAY");
    });

    test("Cooling-off window temporarily suspends enforcement", () => {
      enforcement.setActiveRoutine({ id: "r1", title: "Belajar", tier: "B_TEGAS" });
      enforcement.setDistractionPackages(["com.twitter.android"]);
      const now = 1700000000;

      enforcement.applyCoolingOff(120, now); // 2 minutes cooling off

      const evalCooling = enforcement.evaluateForegroundApp("com.twitter.android", now + 60);
      expect(evalCooling.actionRequired).toBe(false);
      expect(evalCooling.reason).toBe("cooling_off_active");

      const evalExpired = enforcement.evaluateForegroundApp("com.twitter.android", now + 121);
      expect(evalExpired.actionRequired).toBe(true);
    });
  });

  // ---------------------------------------------------------------------------
  // TIER 3: SYNC PROTOCOL & PAYLOAD CONTRACTS
  // ---------------------------------------------------------------------------
  describe("Tier 3: Two-Way Device Sync Protocol", () => {
    test("DeviceSyncPayload correctly deserializes and inserts tasks and shelf items", () => {
      const now = 1700000000;
      const syncPayload: DeviceSyncPayload = {
        deviceId: "android-pixel8-01",
        protocolVersion: 1,
        timestamp: now,
        tasks: [
          {
            id: "task-001",
            title: "Belajar Kalkulus",
            category: "belajar",
            completed: false,
            dueAt: now + 3600,
            updatedAt: now,
          },
          {
            id: "task-002",
            title: "Beli Token Listrik",
            category: "pribadi",
            completed: true,
            dueAt: null,
            updatedAt: now,
          },
        ],
        routines: [],
        shelfItems: [
          {
            id: "shelf-001",
            kind: "text",
            content: "https://arxiv.org/abs/2301.00001",
            pinned: true,
            createdAt: now,
          },
        ],
        pendingApprovals: [],
      };

      // Process and insert into SQLite
      for (const t of syncPayload.tasks) {
        db.run(
          `INSERT INTO mobile_tasks (id, title, category, completed, due_at, updated_at)
           VALUES (?, ?, ?, ?, ?, ?)`,
          [t.id, t.title, t.category, t.completed ? 1 : 0, t.dueAt, t.updatedAt]
        );
      }

      for (const s of syncPayload.shelfItems) {
        db.run(
          `INSERT INTO mobile_shelf (id, kind, content, pinned, created_at)
           VALUES (?, ?, ?, ?, ?)`,
          [s.id, s.kind, s.content, s.pinned ? 1 : 0, s.createdAt]
        );
      }

      const tasksCount = db.query("SELECT COUNT(*) as count FROM mobile_tasks").get() as any;
      expect(tasksCount.count).toBe(2);

      const shelfRow = db.query("SELECT * FROM mobile_shelf WHERE id = ?").get("shelf-001") as any;
      expect(shelfRow).toBeDefined();
      expect(shelfRow.content).toBe("https://arxiv.org/abs/2301.00001");
      expect(shelfRow.pinned).toBe(1);
    });

    test("ScreenTimeSyncPayload aggregates mobile usage into database", () => {
      const now = 1700000000;
      const screenTimePayload: ScreenTimeSyncPayload = {
        deviceId: "android-galaxy-s24",
        timestamp: now,
        entries: [
          {
            packageName: "com.duolingo",
            category: "belajar",
            durationSeconds: 1200,
            dateStr: "2026-11-22",
            timestamp: now,
          },
          {
            packageName: "com.whatsapp",
            category: "netral",
            durationSeconds: 600,
            dateStr: "2026-11-22",
            timestamp: now,
          },
        ],
      };

      for (const e of screenTimePayload.entries) {
        db.run(
          `INSERT INTO mobile_screentime (package_name, category, duration_seconds, date_str, timestamp)
           VALUES (?, ?, ?, ?, ?)`,
          [e.packageName, e.category, e.durationSeconds, e.dateStr, e.timestamp]
        );
      }

      const totalStudySecs = db
        .query("SELECT SUM(duration_seconds) as total FROM mobile_screentime WHERE category = 'belajar'")
        .get() as any;
      expect(totalStudySecs.total).toBe(1200);
    });
  });

  // ---------------------------------------------------------------------------
  // TIER 4: OEM BATTERY OPTIMIZATION & RECOVERY CONTRACTS
  // ---------------------------------------------------------------------------
  describe("Tier 4: OEM Battery Optimization & Recovery Contracts", () => {
    test("Resolves Xiaomi / Redmi / Poco to AutoStartManagementActivity", () => {
      const xiaomiRes = OemReliabilityHelper.resolveBatteryIntent("Xiaomi");
      expect(xiaomiRes.isCustomOemIntent).toBe(true);
      expect(xiaomiRes.targetPackage).toBe("com.miui.securitycenter");
      expect(xiaomiRes.targetActivity).toContain("AutoStartManagementActivity");

      const pocoRes = OemReliabilityHelper.resolveBatteryIntent("POCO");
      expect(pocoRes.isCustomOemIntent).toBe(true);
      expect(pocoRes.targetPackage).toBe("com.miui.securitycenter");
    });

    test("Resolves Samsung to BatteryActivity / IGNORE_BATTERY_OPTIMIZATION_SETTINGS", () => {
      const res = OemReliabilityHelper.resolveBatteryIntent("Samsung");
      expect(res.isCustomOemIntent).toBe(true);
      expect(res.targetPackage).toBe("com.samsung.android.lool");
    });

    test("Resolves Oppo / Realme to ColorOS StartupAppListActivity", () => {
      const oppoRes = OemReliabilityHelper.resolveBatteryIntent("Oppo");
      expect(oppoRes.isCustomOemIntent).toBe(true);
      expect(oppoRes.targetPackage).toBe("com.coloros.safecenter");

      const realmeRes = OemReliabilityHelper.resolveBatteryIntent("Realme");
      expect(realmeRes.isCustomOemIntent).toBe(true);
      expect(realmeRes.targetPackage).toBe("com.coloros.safecenter");
    });

    test("Resolves Vivo to iQOO secure whitelist activity", () => {
      const vivoRes = OemReliabilityHelper.resolveBatteryIntent("Vivo");
      expect(vivoRes.isCustomOemIntent).toBe(true);
      expect(vivoRes.targetPackage).toBe("com.iqoo.secure");
    });

    test("Falls back to standard Android settings for unknown manufacturers", () => {
      const genericRes = OemReliabilityHelper.resolveBatteryIntent("Google");
      expect(genericRes.isCustomOemIntent).toBe(false);
      expect(genericRes.targetPackage).toBe("com.android.settings");
      expect(genericRes.action).toBe("android.settings.IGNORE_BATTERY_OPTIMIZATION_SETTINGS");
    });
  });

  // ---------------------------------------------------------------------------
  // TIER 5: VC-U5 COPY BOUNDS & BRAND COMPLIANCE
  // ---------------------------------------------------------------------------
  describe("Tier 5: Copy Bounds & Brand Compliance", () => {
    test("All tab labels conform to Title length limit (Title <= 16 chars)", () => {
      for (const tab of MOBILE_NAVIGATION_TABS) {
        expect(tab.label.length).toBeLessThanOrEqual(16);
      }
    });

    test("All tab descriptions conform to Desc limit (Desc <= 36 chars)", () => {
      for (const tab of MOBILE_NAVIGATION_TABS) {
        expect(tab.description.length).toBeLessThanOrEqual(36);
      }
    });

    test("Protected app display names conform to Title limit (Title <= 16 chars)", () => {
      for (const app of PROTECTED_MOBILE_APPS) {
        expect(app.displayName.length).toBeLessThanOrEqual(16);
      }
    });

    test("Zero unauthorized 'Bloom' tokens in all mobile definitions", () => {
      for (const tab of MOBILE_NAVIGATION_TABS) {
        expect(tab.label.toLowerCase()).not.toContain("bloom");
        expect(tab.description.toLowerCase()).not.toContain("bloom");
      }
      for (const app of PROTECTED_MOBILE_APPS) {
        expect(app.displayName.toLowerCase()).not.toContain("bloom");
      }
    });
  });
});
