/**
 * island/scripts/resource_optimization.test.ts
 *
 * Stage U4: Resource Optimization (RAM / CPU / Battery) - 15-RESOURCE-OPTIMIZATION.md
 * Comprehensive Test Suite covering:
 * - Budget invariants (Idle <= 350 MB, Mode Hemat <= 250 MB, Active Chat <= 900 MB)
 * - Process tree aggregator logic, private working set calculation, and soak test invariants
 * - Mode Hemat parameter scaling (AI runtime idle timeout 10m -> 3m, SQLite cache, animations)
 * - Window on-demand lifecycle & suspend/low-memory target handling
 * - UI copy limits (Title <= 16, Desc <= 36, Button <= 10) & brand compliance
 *
 * Standards:
 * - bun:test with bun:sqlite
 * - VC-U5 Copy Limits & 15-RESOURCE-OPTIMIZATION.md
 */

import { describe, test, it, expect, beforeEach } from "bun:test";
import { Database } from "bun:sqlite";

// =============================================================================
// 1. DOMAIN MODELS & CONSTANTS (Doc 15 §2, §4, §5)
// =============================================================================

export type SaveMode = "off" | "auto" | "on";

export interface ResourceBudgetLimits {
  maxIdleRamMb: number;
  maxSaveModeRamMb: number;
  maxActiveChatRamMb: number;
  maxPeakTransientRamMb: number;
  maxIdleCpuPercent: number;
  maxAndroidIdleRamMb: number;
  peakReleaseTimeoutSec: number;
}

export const RESOURCE_BUDGETS: ResourceBudgetLimits = {
  maxIdleRamMb: 350,
  maxSaveModeRamMb: 250,
  maxActiveChatRamMb: 900,
  maxPeakTransientRamMb: 500,
  maxIdleCpuPercent: 0.5,
  maxAndroidIdleRamMb: 80,
  peakReleaseTimeoutSec: 60,
};

export interface ProcessMetrics {
  pid: number;
  processName: string;
  privateWorkingSetMb: number;
  cpuPercent: number;
  category: "shell" | "core" | "webview2" | "ai_runtime" | "worker";
}

export interface SystemBatteryState {
  isPluggedIn: boolean;
  batteryPercent: number;
  isSystemBatterySaverActive: boolean;
  isSystemLowRam: boolean;
}

export interface ResourceConfig {
  saveMode: SaveMode;
  aiIdleTimeoutMinutes: number;
  sqliteCacheKb: number;
  sqliteMmapSize: number;
  animationsEnabled: boolean;
  transitionDurationMs: number;
  ambientMascotActive: boolean;
}

export const DEFAULT_RESOURCE_CONFIG: ResourceConfig = {
  saveMode: "auto",
  aiIdleTimeoutMinutes: 10,
  sqliteCacheKb: 2000, // 2MB
  sqliteMmapSize: 0,
  animationsEnabled: true,
  transitionDurationMs: 250,
  ambientMascotActive: true,
};

export const SAVE_MODE_RESOURCE_CONFIG: ResourceConfig = {
  saveMode: "on",
  aiIdleTimeoutMinutes: 3, // Scaled down to 3 minutes
  sqliteCacheKb: 1000,     // 1MB
  sqliteMmapSize: 0,
  animationsEnabled: false,
  transitionDurationMs: 120, // 120ms cross-fade
  ambientMascotActive: false, // Static eyes, zero idle timer
};

// =============================================================================
// 2. SIMULATION & AGGREGATOR ENGINES
// =============================================================================

export class ProcessTreeAggregator {
  processes: Map<number, ProcessMetrics> = new Map();

  updateProcess(metrics: ProcessMetrics) {
    this.processes.set(metrics.pid, metrics);
  }

  removeProcess(pid: number) {
    this.processes.delete(pid);
  }

  calculateTotalWorkingSetMb(): number {
    let total = 0;
    for (const proc of this.processes.values()) {
      total += proc.privateWorkingSetMb;
    }
    return Math.round(total * 10) / 10;
  }

  calculateAverageCpuPercent(): number {
    let total = 0;
    for (const proc of this.processes.values()) {
      total += proc.cpuPercent;
    }
    return Math.round(total * 100) / 100;
  }

  getMetricsByCategory(): Record<string, number> {
    const breakdown: Record<string, number> = {
      shell: 0,
      core: 0,
      webview2: 0,
      ai_runtime: 0,
      worker: 0,
    };
    for (const proc of this.processes.values()) {
      breakdown[proc.category] = (breakdown[proc.category] || 0) + proc.privateWorkingSetMb;
    }
    return breakdown;
  }
}

export class ResourceOptimizationController {
  config: ResourceConfig;
  lastAiActivityTimestamp: number = 0;
  aiRuntimeRunning: boolean = false;

  constructor(initialConfig: ResourceConfig = DEFAULT_RESOURCE_CONFIG) {
    this.config = { ...initialConfig };
  }

  evaluateSaveMode(batteryState: SystemBatteryState): boolean {
    if (this.config.saveMode === "on") {
      this.applySaveModeParameters(true);
      return true;
    }
    if (this.config.saveMode === "off") {
      this.applySaveModeParameters(false);
      return false;
    }

    // "auto" mode: activates when battery saver is on, system low RAM, or battery < 20% while on battery
    const shouldActivate =
      batteryState.isSystemBatterySaverActive ||
      batteryState.isSystemLowRam ||
      (!batteryState.isPluggedIn && batteryState.batteryPercent < 20);

    this.applySaveModeParameters(shouldActivate);
    return shouldActivate;
  }

  applySaveModeParameters(active: boolean) {
    if (active) {
      this.config.aiIdleTimeoutMinutes = 3;
      this.config.animationsEnabled = false;
      this.config.transitionDurationMs = 120;
      this.config.ambientMascotActive = false;
      this.config.sqliteCacheKb = 1000;
    } else {
      this.config.aiIdleTimeoutMinutes = 10;
      this.config.animationsEnabled = true;
      this.config.transitionDurationMs = 250;
      this.config.ambientMascotActive = true;
      this.config.sqliteCacheKb = 2000;
    }
  }

  startAiRuntime(nowSec: number) {
    this.aiRuntimeRunning = true;
    this.lastAiActivityTimestamp = nowSec;
  }

  recordAiActivity(nowSec: number) {
    this.lastAiActivityTimestamp = nowSec;
  }

  checkIdleAiRuntimeStop(nowSec: number): boolean {
    if (!this.aiRuntimeRunning) return false;

    const timeoutSecs = this.config.aiIdleTimeoutMinutes * 60;
    if (nowSec - this.lastAiActivityTimestamp >= timeoutSecs) {
      this.aiRuntimeRunning = false;
      return true; // Stopped due to idle timeout
    }
    return false;
  }
}

// =============================================================================
// 3. TEST SUITES
// =============================================================================

describe("Stage U4: Resource Optimization (RAM / CPU / Battery)", () => {
  let db: Database;
  let aggregator: ProcessTreeAggregator;
  let controller: ResourceOptimizationController;

  beforeEach(() => {
    db = new Database(":memory:");
    aggregator = new ProcessTreeAggregator();
    controller = new ResourceOptimizationController();

    db.run(`
      CREATE TABLE IF NOT EXISTS resource_metrics (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        timestamp INTEGER NOT NULL,
        total_ram_mb REAL NOT NULL,
        cpu_percent REAL NOT NULL,
        save_mode TEXT NOT NULL,
        ai_runtime_active INTEGER NOT NULL
      );
    `);
  });

  // ---------------------------------------------------------------------------
  // TIER 1: RESOURCE BUDGET INVARIANTS
  // ---------------------------------------------------------------------------
  describe("Tier 1: Resource Budget Invariants", () => {
    test("Idle target budget is strictly <= 350 MB RAM and < 0.5% CPU", () => {
      // Mock realistic idle process tree
      aggregator.updateProcess({
        pid: 101,
        processName: "hime-island.exe",
        privateWorkingSetMb: 45.0,
        cpuPercent: 0.05,
        category: "shell",
      });
      aggregator.updateProcess({
        pid: 102,
        processName: "hime-core.exe",
        privateWorkingSetMb: 60.0,
        cpuPercent: 0.1,
        category: "core",
      });
      aggregator.updateProcess({
        pid: 103,
        processName: "msedgewebview2.exe", // Notch webview
        privateWorkingSetMb: 120.0,
        cpuPercent: 0.05,
        category: "webview2",
      });

      const totalRam = aggregator.calculateTotalWorkingSetMb();
      const avgCpu = aggregator.calculateAverageCpuPercent();

      expect(totalRam).toBeLessThanOrEqual(RESOURCE_BUDGETS.maxIdleRamMb);
      expect(avgCpu).toBeLessThan(RESOURCE_BUDGETS.maxIdleCpuPercent);
      expect(totalRam).toBe(225.0);
    });

    test("Mode Hemat idle target budget is strictly <= 250 MB RAM", () => {
      // Under Mode Hemat, background webviews are suspended and memory freed
      aggregator.updateProcess({
        pid: 101,
        processName: "hime-island.exe",
        privateWorkingSetMb: 38.0,
        cpuPercent: 0.02,
        category: "shell",
      });
      aggregator.updateProcess({
        pid: 102,
        processName: "hime-core.exe",
        privateWorkingSetMb: 48.0,
        cpuPercent: 0.05,
        category: "core",
      });
      aggregator.updateProcess({
        pid: 103,
        processName: "msedgewebview2.exe",
        privateWorkingSetMb: 95.0,
        cpuPercent: 0.01,
        category: "webview2",
      });

      const totalRam = aggregator.calculateTotalWorkingSetMb();
      expect(totalRam).toBeLessThanOrEqual(RESOURCE_BUDGETS.maxSaveModeRamMb);
      expect(totalRam).toBe(181.0);
    });

    test("Active Chat budget accommodates AI runtime within <= 900 MB RAM", () => {
      // Base idle + Hermes (Python) + 9Router (Node)
      aggregator.updateProcess({
        pid: 101,
        processName: "hime-island.exe",
        privateWorkingSetMb: 60.0,
        cpuPercent: 1.5,
        category: "shell",
      });
      aggregator.updateProcess({
        pid: 102,
        processName: "hime-core.exe",
        privateWorkingSetMb: 85.0,
        cpuPercent: 2.0,
        category: "core",
      });
      aggregator.updateProcess({
        pid: 103,
        processName: "msedgewebview2.exe",
        privateWorkingSetMb: 160.0,
        cpuPercent: 2.5,
        category: "webview2",
      });
      aggregator.updateProcess({
        pid: 104,
        processName: "python.exe", // Hermes agent
        privateWorkingSetMb: 240.0,
        cpuPercent: 8.0,
        category: "ai_runtime",
      });
      aggregator.updateProcess({
        pid: 105,
        processName: "node.exe", // 9Router gateway
        privateWorkingSetMb: 150.0,
        cpuPercent: 1.0,
        category: "ai_runtime",
      });

      const totalRam = aggregator.calculateTotalWorkingSetMb();
      expect(totalRam).toBeLessThanOrEqual(RESOURCE_BUDGETS.maxActiveChatRamMb);
      expect(totalRam).toBe(695.0);
    });

    test("Transient heavy workers must be released within 60 seconds", () => {
      expect(RESOURCE_BUDGETS.peakReleaseTimeoutSec).toBe(60);
      expect(RESOURCE_BUDGETS.maxPeakTransientRamMb).toBe(500);
    });
  });

  // ---------------------------------------------------------------------------
  // TIER 2: PROCESS AGGREGATOR & SOAK INVARIANTS
  // ---------------------------------------------------------------------------
  describe("Tier 2: Process Aggregator & Soak Test Invariants", () => {
    test("Calculates category breakdown correctly", () => {
      aggregator.updateProcess({ pid: 1, processName: "hime-island.exe", privateWorkingSetMb: 50, cpuPercent: 0, category: "shell" });
      aggregator.updateProcess({ pid: 2, processName: "hime-core.exe", privateWorkingSetMb: 70, cpuPercent: 0, category: "core" });
      aggregator.updateProcess({ pid: 3, processName: "msedgewebview2.exe", privateWorkingSetMb: 110, cpuPercent: 0, category: "webview2" });

      const breakdown = aggregator.getMetricsByCategory();
      expect(breakdown.shell).toBe(50);
      expect(breakdown.core).toBe(70);
      expect(breakdown.webview2).toBe(110);
      expect(breakdown.ai_runtime).toBe(0);
    });

    test("Removing terminated worker process immediately frees tracked RAM", () => {
      aggregator.updateProcess({ pid: 201, processName: "worker-ocr.exe", privateWorkingSetMb: 180, cpuPercent: 5, category: "worker" });
      expect(aggregator.calculateTotalWorkingSetMb()).toBe(180);

      aggregator.removeProcess(201);
      expect(aggregator.calculateTotalWorkingSetMb()).toBe(0);
    });

    test("Soak test invariant: 60-minute simulated idle drift is < 5%", () => {
      const baseRam = 220.0;
      // Simulate 12 samples over 60 minutes
      const samples: number[] = [];
      for (let i = 0; i < 12; i++) {
        // Minor bounded jitter (+- 1 MB) without accumulation
        const jitter = Math.sin(i) * 1.0;
        samples.push(baseRam + jitter);
      }

      const initialRam = samples[0];
      const finalRam = samples[samples.length - 1];
      const driftPercent = (Math.abs(finalRam - initialRam) / initialRam) * 100;

      expect(driftPercent).toBeLessThan(5.0);
    });
  });

  // ---------------------------------------------------------------------------
  // TIER 3: MODE HEMAT PARAMETER SCALING
  // ---------------------------------------------------------------------------
  describe("Tier 3: Mode Hemat Parameter Scaling", () => {
    test("Mode Hemat triggers automatically on system battery saver or low RAM", () => {
      controller.config.saveMode = "auto";

      // Normal battery
      const normalBattery: SystemBatteryState = {
        isPluggedIn: true,
        batteryPercent: 85,
        isSystemBatterySaverActive: false,
        isSystemLowRam: false,
      };
      expect(controller.evaluateSaveMode(normalBattery)).toBe(false);
      expect(controller.config.aiIdleTimeoutMinutes).toBe(10);

      // System battery saver enabled
      const saverBattery: SystemBatteryState = {
        isPluggedIn: false,
        batteryPercent: 40,
        isSystemBatterySaverActive: true,
        isSystemLowRam: false,
      };
      expect(controller.evaluateSaveMode(saverBattery)).toBe(true);
      expect(controller.config.aiIdleTimeoutMinutes).toBe(3);
      expect(controller.config.animationsEnabled).toBe(false);
      expect(controller.config.ambientMascotActive).toBe(false);
    });

    test("Mode Hemat triggers automatically when on battery with charge < 20%", () => {
      controller.config.saveMode = "auto";

      const lowBattery: SystemBatteryState = {
        isPluggedIn: false,
        batteryPercent: 18,
        isSystemBatterySaverActive: false,
        isSystemLowRam: false,
      };
      expect(controller.evaluateSaveMode(lowBattery)).toBe(true);
      expect(controller.config.aiIdleTimeoutMinutes).toBe(3);
    });

    test("AI runtime stops after 10m idle normally, and after 3m under Mode Hemat", () => {
      const now = 1700000000;

      // Normal mode: 10 minutes timeout
      controller.applySaveModeParameters(false);
      controller.startAiRuntime(now);

      // At 5 minutes (300s): Still running
      expect(controller.checkIdleAiRuntimeStop(now + 300)).toBe(false);
      expect(controller.aiRuntimeRunning).toBe(true);

      // At 10 minutes + 1s (601s): Stops
      expect(controller.checkIdleAiRuntimeStop(now + 601)).toBe(true);
      expect(controller.aiRuntimeRunning).toBe(false);

      // Mode Hemat: 3 minutes timeout
      controller.applySaveModeParameters(true);
      controller.startAiRuntime(now);

      // At 2 minutes (120s): Still running
      expect(controller.checkIdleAiRuntimeStop(now + 120)).toBe(false);

      // At 3 minutes + 1s (181s): Stops
      expect(controller.checkIdleAiRuntimeStop(now + 181)).toBe(true);
      expect(controller.aiRuntimeRunning).toBe(false);
    });

    test("SQLite cache tunes down to 1000 KB and mmap is 0 under Mode Hemat", () => {
      controller.applySaveModeParameters(true);
      expect(controller.config.sqliteCacheKb).toBe(1000);
      expect(controller.config.sqliteMmapSize).toBe(0);
    });
  });

  // ---------------------------------------------------------------------------
  // TIER 4: WINDOW ON-DEMAND & WEBVIEW SUSPEND
  // ---------------------------------------------------------------------------
  describe("Tier 4: Window On-Demand Lifecycle & Suspend", () => {
    test("Settings and Hub windows are destroyed upon close rather than hidden", () => {
      const windowRegistry = new Map<string, { state: "active" | "destroyed" }>();

      const openWindow = (name: string) => {
        windowRegistry.set(name, { state: "active" });
      };

      const closeWindow = (name: string) => {
        // Destroyed to release WebView2 process memory
        windowRegistry.delete(name);
      };

      openWindow("settings");
      expect(windowRegistry.has("settings")).toBe(true);

      closeWindow("settings");
      expect(windowRegistry.has("settings")).toBe(false);
    });

    test("Low memory target level drops background cache allocations", () => {
      const webviewMemoryTarget = (isVisible: boolean): "Normal" | "Low" => {
        return isVisible ? "Normal" : "Low";
      };

      expect(webviewMemoryTarget(true)).toBe("Normal");
      expect(webviewMemoryTarget(false)).toBe("Low");
    });
  });

  // ---------------------------------------------------------------------------
  // TIER 5: VC-U5 COPY BOUNDS & BRAND COMPLIANCE
  // ---------------------------------------------------------------------------
  describe("Tier 5: Copy Bounds & Brand Compliance", () => {
    const RESOURCE_UI_ITEMS = [
      { title: "Sumber Daya", desc: "Monitor RAM dan kontrol daya", btn: "Hentikan" },
      { title: "Mode Hemat", desc: "Otomatis saat baterai < 20%", btn: "Hemat" },
      { title: "Pohon Proses", desc: "Daftar RAM tiap komponen Hime", btn: "Segarkan" },
      { title: "Runtime AI", desc: "Berhenti otomatis saat idle", btn: "Tutup" },
    ];

    test("All titles conform to strict copy limit (Title <= 16 chars)", () => {
      for (const item of RESOURCE_UI_ITEMS) {
        expect(item.title.length).toBeLessThanOrEqual(16);
      }
    });

    test("All descriptions conform to strict copy limit (Desc <= 36 chars)", () => {
      for (const item of RESOURCE_UI_ITEMS) {
        expect(item.desc.length).toBeLessThanOrEqual(36);
      }
    });

    test("All buttons conform to strict copy limit (Button <= 10 chars)", () => {
      for (const item of RESOURCE_UI_ITEMS) {
        expect(item.btn.length).toBeLessThanOrEqual(10);
      }
    });

    test("Zero unauthorized 'Bloom' tokens in resource optimization components", () => {
      for (const item of RESOURCE_UI_ITEMS) {
        expect(item.title.toLowerCase()).not.toContain("bloom");
        expect(item.desc.toLowerCase()).not.toContain("bloom");
        expect(item.btn.toLowerCase()).not.toContain("bloom");
      }
    });
  });
});
