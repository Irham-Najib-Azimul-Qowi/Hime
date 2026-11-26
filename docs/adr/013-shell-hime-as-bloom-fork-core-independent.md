# ADR-013: Hime Desktop Shell as an Additive Bloom Fork with Decoupled Hime Core

**Status:** Accepted  
**Date:** 2026-11-15  
**Authors:** Principal Engineer + Technical Lead  
**Deciders:** Project Owner, Hime Engineering Team  

---

## 1. Context
In Phase 07, a custom desktop user interface built from scratch with Tauri and React failed visual acceptance: it rendered as a standard rectangular desktop window with native OS borders and titlebar controls, failing to achieve the floating, morphing Dynamic Island form factor of Nusa.

Rather than continuously fixing custom window hacks, the project owner directed an architectural pivot: build the Hime Desktop Shell on top of **Bloom** (`https://github.com/little-systems/bloom.git`, GPL-3.0), a mature, high-polish open-source Windows shell replacement featuring a spring-physics Dynamic Island (notch), taskbar dock, HUD overlay, and settings manager built on Tauri v2, React 19, TypeScript, and Rust.

At the same time, Hime Core's architectural responsibilities (Policy Enforcement Point, secret vault, process supervisor for Hermes Agent and 9Router) must remain decoupled from the visual shell.

---

## 2. Decision
1. **Host Adoption:** Hime Shell is established as a direct fork of Bloom pinned to release tag `v3.10.2` (commit `0ab9f2f08b75cfeffd3040330e312c2244d7726a`).
2. **Core Independence:** Hime Core (`hime-core.exe`) remains a completely separate, dedicated Rust background process.
3. **IPC Boundary:** The Shell communicates with Core exclusively over a Win32 Local Named Pipe (`\\.\pipe\hime-ipc`) using JSON-RPC requests and event streaming.
4. **Resilience Principle:** If Hime Core is stopped or absent, the Shell continues running with full native Bloom functionality, displaying an honest "Core Offline" state without crashing, hanging, or degrading shell performance.
5. **GPL-3.0 Compliance:** The Shell codebase is governed by GPL-3.0. The IPC separation between the Shell and Core preserves clean architectural and licensing boundaries.

---

## 3. Alternatives Considered
- **Alternative A: Monolithic Merged Process (Link Core directly into Bloom backend):**
  - *Rejected:* Destroys the Policy Enforcement Point boundary, tightly couples AI daemon lifecycles with the desktop UI, creates potential crash cascades where model failures bring down the Windows taskbar, and complicates GPL licensing.
- **Alternative B: Keep Custom Tauri UI and patch window properties:**
  - *Rejected:* Failed visual acceptance repeatedly; custom window borders and z-order handling on Windows do not match the physics and multi-monitor AppBar integration already proven in Bloom.

---

## 4. Consequences
- **Positive:** Instantly inherits Bloom's native Dynamic Island morphing, live activity bubbles, dock, and spring physics. Hime UI integrates seamlessly as native Bloom panels and bubbles.
- **Negative:** Requires maintaining upstream tracking, monitoring delta memory/CPU footprint against pristine Bloom, and enforcing strict additive touchpoint discipline.

