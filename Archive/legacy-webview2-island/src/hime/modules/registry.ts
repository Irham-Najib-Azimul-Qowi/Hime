/**
 * Hime Island Frontend Module Registry (island/src/hime/modules/registry.ts)
 *
 * Implements zero-cost modular extensions per 14-MODULARITY-AND-CODE-QUALITY.md.
 * Modules are loaded dynamically only when enabled; disabled modules have zero
 * runtime listeners, zero timers, and zero memory overhead.
 */

export interface ModuleManifestInfo {
  id: string;
  version: string;
  title: string;
  desc: string;
  icon: string;
  category: string;
  enabled: boolean;
}

export interface HimeShellModule {
  id: string;
  register: () => Promise<void> | void;
  dispose: () => Promise<void> | void;
}

class ShellModuleRegistry {
  private activeModules: Map<string, HimeShellModule> = new Map();
  private enabledStates: Map<string, boolean> = new Map();

  constructor() {
    // Initial default modules from manifests
    this.enabledStates.set("eyes", true);
    this.enabledStates.set("keybinds", true);
  }

  public isEnabled(moduleId: string): boolean {
    return this.enabledStates.get(moduleId) ?? false;
  }

  public async setEnabled(moduleId: string, enabled: boolean): Promise<void> {
    const prev = this.isEnabled(moduleId);
    if (prev === enabled) return;

    this.enabledStates.set(moduleId, enabled);

    if (!enabled) {
      // Clean teardown for zero-cost when OFF
      const mod = this.activeModules.get(moduleId);
      if (mod) {
        await mod.dispose();
        this.activeModules.delete(moduleId);
      }
    } else {
      // Lazy load and register
      await this.loadAndActivate(moduleId);
    }
  }

  private async loadAndActivate(moduleId: string): Promise<void> {
    if (this.activeModules.has(moduleId)) return;

    try {
      if (moduleId === "eyes") {
        const mod: HimeShellModule = {
          id: "eyes",
          register: () => {
            // Eyes module registration
          },
          dispose: () => {
            // Teardown listeners
          }
        };
        await mod.register();
        this.activeModules.set(moduleId, mod);
      } else if (moduleId === "keybinds") {
        const mod: HimeShellModule = {
          id: "keybinds",
          register: () => {
            // Global keybind registration
          },
          dispose: () => {
            // Unregister shortcuts
          }
        };
        await mod.register();
        this.activeModules.set(moduleId, mod);
      }
    } catch (err) {
      console.error(`Failed to activate module '${moduleId}':`, err);
    }
  }

  public async initializeAll(): Promise<void> {
    for (const [id, enabled] of this.enabledStates.entries()) {
      if (enabled) {
        await this.loadAndActivate(id);
      }
    }
  }
}

export const moduleRegistry = new ShellModuleRegistry();
