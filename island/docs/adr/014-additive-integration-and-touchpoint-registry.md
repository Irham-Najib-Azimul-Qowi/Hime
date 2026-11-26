# ADR-014: Additive Integration Architecture and Mandatory Touchpoint Registry

**Status:** Accepted  
**Date:** 2026-11-15  
**Authors:** Principal Engineer + Technical Lead  
**Deciders:** Project Owner, Hime Engineering Team  

---

## 1. Context
Integrating Hime capabilities (Assistant panel, live activity bubbles, selected-text translation, device sync, approval workflows) into Bloom carries the risk of destabilizing Bloom's core shell features (window docking, taskbar replacement, audio visualizer, spring animations) if changes are scattered or if Bloom files are refactored.

A disciplined architectural constraint is required to ensure Bloom remains the stable host while Hime code stays isolated, auditable, and easily maintainable across future updates.

---

## 2. Decision
1. **Isolated Code Directories:**
   - All frontend Hime components, hooks, and stores are isolated in `src/hime/**`.
   - All backend Rust Hime bridges, IPC handlers, and state proxies are isolated in `src-tauri/src/hime/**`.
2. **Minimal Touchpoints:**
   - Existing Bloom files may only be modified to **register** Hime elements:
     - Registering the Hime panel in `App.tsx` panel router.
     - Registering the Hime live activity bubble in the bubble list.
     - Registering IPC commands in `generate_handler!` in `main.rs`.
     - Registering the Hime settings tab in `Settings.tsx`.
3. **Mandatory Touchpoint Registry:**
   - Every touched line in an upstream Bloom file must be logged in `docs/hime/TOUCHPOINTS.md` with file path, line numbers, technical reason, and explicit revert instructions.
4. **No Refactoring of Upstream Bloom:**
   - Upstream Bloom code must not be refactored, restructured, or formatted differently from Bloom's conventions.

---

## 3. Alternatives Considered
- **Alternative A: Fork and Free-Form Edit Bloom:**
  - *Rejected:* Quickly leads to an unmaintainable codebase where Bloom fixes cannot be compared, bugs become impossible to attribute, and regressions in taskbar safety cannot be isolated.
- **Alternative B: Pure Webview Injection / Overlay:**
  - *Rejected:* Cannot integrate into Bloom's notch spring-physics morphing hierarchy and would create separate uncoordinated windows.

---

## 4. Consequences
- **Positive:** Clear architectural boundaries; zero ambiguity on whether a bug originates in Bloom or Hime; easy auditing of upstream diff.
- **Negative:** Requires strict discipline when wiring new UI interactions.

