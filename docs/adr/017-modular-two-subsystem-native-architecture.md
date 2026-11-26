# ADR-017: Modular Two-Subsystem Native Rust Architecture (`Hime Island` + `Hime AI`)

- **Status:** PROPOSED
- **Date:** 2026-11-26
- **Author(s):** Antigravity Engineering Agent & Project Owner
- **Deciders:** Project Owner
- **Informed:** All Hime Desktop & Hime Core Contributors

---

## 1. Context and Problem Statement

Live profiling of the current Hime Desktop ecosystem on Windows revealed a total footprint of **~765.5 MB Working Set RAM (~843.7 MB Private Memory)** across 16 processes:

- **WebView2 (3 simultaneous Chromium renderers: Main Notch, Dock, Overlay + GPU/Browser/Utility):** ~598.5 MB (~78% of total RAM) and highest cumulative CPU.
- **9Router (Node.js gateway):** ~119.4 MB (~16% of total RAM).
- **Native Rust binaries (`hime-island.exe`, `hime-core.exe`, `hime_hotkey.exe`, watchdog):** ~33.5 MB (~4% of total RAM).

Furthermore, tightly coupling AI features, multi-window React/Framer Motion UI, and OS shell hooks inside the Chromium WebView2 shell increases regression risk whenever UI layout or AI states change.

The Project Owner wants to refactor Hime into a **clean, modular 2-subsystem native Rust architecture** while **leaving `9Router` untouched as an optional external module**:

1. **Subsystem 1 (`hime-island`)**: Ultra-lightweight pure-Rust desktop shell with **zero AI and zero WebView2** (preserving the existing UI/UX design), supporting Notch, Dock, Overlay, Settings, and drag-and-drop File Shelf.
2. **Subsystem 2 (`hime-ai`)**: Optional modular AI assistant downloaded from the repository and installed locally when enabled in Settings — supporting adaptive multi-provider AI APIs (including `9Router`, Google Gemini API, and any OpenAI-compatible endpoint), extensible Skills, continuous learning Memory, MCP (Model Context Protocol), and autonomous laptop/task/document/device assistance.

---

## 2. Decision Drivers

- **Extreme Idle Efficiency:** Reduce always-on desktop memory footprint from ~765 MB to **~15–40 MB RAM** and near-0% idle CPU.
- **Zero Visual Regression:** Preserve Hime's current visual identity (3-edge Dynamic Island, `EdgeSurface` ears/flares, `HimeEyes` expressions, spring animations, synced LRC lyrics, Dock, Overlay) by replacing the rendering engine, not redesigning the UI.
- **Fault Isolation & Modularity:** AI runtime updates, downloads, or crashes must never affect core Notch/Dock/Taskbar stability.
- **Provider Agnosticism without Bloat:** Support any AI model API cleanly without rebuilding `9Router`'s 130+ bespoke provider transforms.
- **Extensibility:** Allow adding new assistant capabilities without recompiling core binaries via **Skills** and **MCP**.
- **9Router Boundary:** Do NOT modify `9Router` source code; keep it strictly external/optional (local or remote).

---

## 3. Architecture Overview (2 Modular Subsystems + Optional External 9Router)

```text
+-----------------------------------------------------------------------------+
| SUBSYSTEM 1: Hime Island Shell (Always-On, Pure Rust Native, ~15-40 MB)     |
|                                                                             |
|  - Embedded Hotkey & Lifecycle (absorbs hime_hotkey + PS1 supervisor roles) |
|  - Native UI Renderer: Direct2D + DirectWrite + DirectComposition (Win32)   |
|    * 3-Edge Notch (Clock, Net, Battery, Music+LRC, Calendar, Timer, NCM)    |
|    * Floating Dock (Taskbar replacement, Win+1..9, System Tray mirror)      |
|    * Left/Right Edge Overlay (Volume, Audio Devices, Brightness HUD)        |
|    * File Shelf (Drag & Drop local temporary file stash & previews)         |
|    * Native Settings (includes "Enable Hime AI" module installer toggle)    |
|  - Module Manager: Downloads verified `hime-ai` release (SHA-256 check)     |
+--------------------------------------+--------------------------------------+
                                       |
         Win32 Named Pipe (\\.\pipe\hime-ipc) + JSON-RPC + CSPRNG Token
         (Subsystem 2 runs only when enabled in Settings)
                                       v
+-----------------------------------------------------------------------------+
| SUBSYSTEM 2: Hime AI Runtime (Optional Modular Add-on, ~15-35 MB)           |
|                                                                             |
|  1. Adaptive AI Gateway (3 core adapters + JSON provider presets + failover)|
|     * OpenAI-Compatible (9Router, OpenAI, Groq, OpenRouter, CF, NVIDIA, etc)|
|     * Google Gemini Native (`v1beta` REST/SSE + multimodal + tools)         |
|     * Anthropic Messages (`v1/messages` lightweight compat)                 |
|  2. Self-Learning Memory Engine (SQLite + FTS5 + Post-Turn Auto-Extractor)  |
|  3. Dynamic Skill Runtime (`hime-modsdk` + folder `SKILL.md` / `skill.json`)|
|  4. MCP Host Client (`stdio` local servers + `HTTP/SSE` remote servers)     |
|  5. Autonomous Assistants (Tasks/Deadlines, File/Storage Ops, Dev/Web,      |
|     Docs/Academic PDF, Device Activity Watcher, App Control)                |
|  6. Security Gate (`PolicyEngine` PEP, DPAPI Vault, Approval Cards, Audit)  |
+--------------------------------------+--------------------------------------+
                                       |
                 Optional HTTP/HTTPS OpenAI-Compatible API
                                       v
+-----------------------------------------------------------------------------+
| EXTERNAL MODULE (Untouched): 9Router (Local 127.0.0.1:20128 or Azure VM)    |
+-----------------------------------------------------------------------------+
```

---

## 4. Detailed Subsystem Design

### 4.1 Subsystem 1: `hime-island` (Lightweight Core Shell — Zero AI)

1. **Process Consolidation:**
   - Merge PowerShell supervisor responsibilities (`hime-supervisor.ps1`), taskbar safety restoration (`restore-taskbar` on panic/exit/crash marker), and `hime_hotkey` global hotkey listener into a single native Rust binary (`hime-island.exe`) with an optional minimal sleep-tray mode (`Ctrl+Shift+Q` hides/unloads UI surfaces while keeping hotkey listener alive under ~5 MB, or exits cleanly).
2. **Pure Native Rendering Engine (Zero WebView2):**
   - Replace Tauri WebView2 renderers with **Direct2D + DirectWrite + DirectComposition + `windows-rs`** (layered/transparent borderless surfaces with per-pixel alpha and spring physics timer).
   - **Visual Parity Guarantee:** Replicate the exact existing design language—corner radii, `EdgeSurface` concave ears/shine, `HimeEyes` vector eye geometry/blinking, dark glass contrast, vertical left/right stacking, Dock pill spacing, and LRC lyric animations.
   - **Click-Through Precision:** Native Win32 hit-testing (`WM_NCHITTEST` / `DirectComposition` visual bounds) replaces polling `SetWindowRgn` over WebView2, eliminating Chromium blanking bugs and GPU process overhead.
3. **Core Non-AI Features:**
   - 3-Edge Draggable Notch (Top / Left / Right, auto-hide, smart peek).
   - System telemetry: Wi-Fi vs Ethernet detection, internet probe, speed monitor, battery, Bluetooth/Buds, USB, 3.5mm jack, audio visualizer + SMTC media playback & LRC lyrics.
   - Windows Native Suppression & Redirect: `Win+N` calendar toggle, Windows toast capture via `IUIAutomation` displayed in Main Notch center slot.
   - Dock (app launcher, running window thumbnails, system tray mirror, `Win+1..9`).
   - Overlay HUD (Volume, output switcher, Brightness).
   - **File Shelf (Drag & Drop):** Native `IDropTarget` accepting files dropped onto the Notch, stored safely in local shelf metadata with quick preview, copy, drag-out, or attach-to-AI (when Subsystem 2 is active).
   - Native Settings window + **Hime AI Module Manager** (downloads, verifies SHA-256 checksum, installs, starts/stops, and updates `hime-ai` from the configured repository).

### 4.2 Subsystem 2: `hime-ai` (Modular Assistant Add-on)

Subsystem 2 builds upon the existing `core/` workspace (`hime-core`, `hime-ipc`, `hime-security`, `hime-storage`, `hime-modsdk`, `hime-pal`) and extends it with five key capabilities:

#### A. Adaptive AI Model Gateway (Lightweight Multi-Provider)
Rather than re-implementing 130+ bespoke provider quirks (which stays `9Router`'s job), `hime-ai` implements **3 lean Rust HTTP/SSE protocol adapters** driven by an externalizable JSON preset registry (`providers.json`):

1. **`openai-compat` Adapter:** Connects to any OpenAI-compatible `/v1/chat/completions` & `/v1/models` endpoint (`Authorization: Bearer <key>`). A single code path immediately supports **9Router** (`http://127.0.0.1:20128/v1` or remote HTTPS), **OpenAI**, **Groq**, **OpenRouter**, **NVIDIA NIM**, **Cloudflare Workers AI**, **Mistral**, **DeepSeek**, **Cerebras**, **Ollama**, and **LM Studio** by merely supplying `(base_url, api_key, model_id)`.
2. **`gemini-native` Adapter:** Native `generativelanguage.googleapis.com/v1beta` SSE/JSON adapter supporting Google AI Studio keys, system instructions, tool declarations, and multimodal inputs (images/files from File Shelf).
3. **`anthropic-compat` Adapter (thin):** Lightweight `/v1/messages` (`x-api-key`) adapter for direct Claude API usage.

- **Adaptive Capability Discovery & Failover:** Queries `/v1/models` (or preset metadata) to auto-detect context window, vision, and tool-calling support; maintains an ordered **Priority Profile Chain** in SQLite (with keys encrypted via **Windows DPAPI** and zeroized on drop) that fails over cleanly on `429`/`5xx`/timeout.

#### B. Self-Learning Memory Engine
Extends `core/crates/hime-core/src/ai/memory.rs` and `hime-storage` (`hime_core.db`) from passive storage to continuous learning:

- **Three Memory Layers:**
  1. *Semantic (User Profile & Preferences):* Workspace paths, formatting standards, preferred apps, tone, language.
  2. *Episodic (Tasks, Deadlines & Context):* Ongoing projects, recent file moves, reminders, past errors/fixes.
  3. *Procedural (Learned Habits & Corrections):* Explicit user corrections (*"when I ask to open X, use app Y"*), injected with high priority into matching prompts.
- **Post-Turn Extractor:** Lightweight background step after completed turns extracts candidate memories with source provenance, deduplicates against existing records via SQLite **FTS5 + recency/frequency decay**, and enforces strict token budgets when building prompt context.
- **User Sovereignty:** Every learned memory displays its provenance in Settings and can be inspected, pinned, edited, or deleted at any time.

#### C. Dynamic Skill System (`hime-modsdk` + Declarative `SKILL.md`)
- **Folder-Based Extensibility:** Users or agents can drop new skills into `%APPDATA%\ai.hime.desktop\skills\<skill-id>\` containing:
  - `SKILL.md` (instructions, examples, persona/workflow rules), and
  - `manifest.json` (trigger keywords/intents, required tool permissions, optional local script/executable entry point, risk level).
- **Hot Reload:** `hime-ai` watches the skills directory and registers/updates skills without restarting `hime-island`.
- **Built-in & Extensible Domains:** Task & deadline reminders, proactive laptop check-ins, application launch/close control, device activity monitoring, smart file/folder organization, web/document/academic PDF creation, and developer git/command runner.

#### D. MCP (Model Context Protocol) Host Client
- **`mcp_servers.json` Configuration:** Supports both **`stdio`** (local child processes wrapped in `JobSupervisor` Win32 Job Object) and **`Streamable HTTP / SSE`** (remote servers).
- **Protocol Lifecycle:** Performs JSON-RPC `initialize`, `notifications/initialized`, `tools/list`, `resources/list`, and `prompts/list`.
- **Unified Tool & Policy Routing:** Dynamically exposes discovered MCP tools into `ToolGateway` under `mcp.<server>.<tool_name>`, forcing every call through `PolicyEngine`, parameter schema validation, `redact_api_keys`, and Notch **Approval Cards** for side-effecting operations.

#### E. UI Bridge for Hime AI
- **IPC-Driven Native Panels + Optional Rich Artifact Drawer:**
  - Quick Chat, Approval Bubble/Card, Tasks, Reminders, and Proactive Alerts render inside Subsystem 1's native Notch UI via typed `hime-ipc` events (`stream_delta`, `tool_status`, `approval_request`, `proactive_alert`).
  - *Improvisation for Complex Artifacts:* For heavy HTML/KaTeX/interactive previews (such as generated web apps or math-heavy reports), Subsystem 2 can open an on-demand preview window or hand off to the user's browser—keeping Subsystem 1's steady-state memory pure native (~15–40 MB).

---

## 5. Additional Engineering Improvisations

1. **Atomic Module Install & Self-Healing Rollback:** When Subsystem 1 downloads a new `hime-ai` package from GitHub Releases:
   - Verify SHA-256 hash against release manifest before unpacking into `modules/hime-ai/<version>/`.
   - Keep `previous/` version intact; if the new binary fails IPC handshake within 10s, auto-rollback to `previous/` and surface a clear status alert in Notch.
2. **Attention Budget & Context-Aware Proactivity:** Device activity watcher and deadline reminders check foreground fullscreen state (games/presentations/meetings) and user attention budget (`pendiam` / `seimbang` / `cerewet`) before popping Notch alerts.
3. **Zero-Config `9Router` Auto-Detection:** On first enable of `hime-ai`, probe `http://127.0.0.1:20128/v1/models` automatically; if active, offer one-click binding (e.g., `hime-combo`) alongside direct API key entry (Gemini / OpenAI / Groq / Custom).
4. **Emergency Kill-Switch & Crash Recovery:** `Ctrl+Alt+Shift+P` immediately halts active AI tool loops and background MCP `stdio` child processes via Win32 Job Object termination without touching Subsystem 1 UI.

---

## 6. Phased Migration Plan (Zero-Breakage Path)

| Phase | Target | Deliverable | Exit Criteria |
| :--- | :--- | :--- | :--- |
| **Phase 1: Core AI Upgrade (`hime-ai` backend)** | `core/` | Upgrade `provider_adapter.rs` (`openai-compat` + `gemini-native` + `anthropic-compat` + presets + failover), add MCP Client Host (`stdio` + HTTP/SSE), add `SKILL.md` hot-loader, upgrade `MemoryEngine` with FTS5 & post-turn extraction. | All `core` tests pass; CLI/pipe test chats with Gemini API, 9Router, and an MCP server succeed. |
| **Phase 2: IPC & Module Installer Contract** | `hime-ipc` + `hime-modsdk` | Finalize versioned IPC messages for Chat, Skills, Memories, MCP Approvals, and File Shelf handoff; implement SHA-256 verified module downloader/launcher. | Subsystem 1 can download, verify, start, health-check, and cleanly stop `hime-ai`. |
| **Phase 3: Native Rust Shell (`hime-island` UI)** | `island/` | Build native Direct2D/DirectComposition shell incrementally: (3a) Overlay HUD -> (3b) Dock + Tray -> (3c) 3-Edge Notch + File Shelf + Music/LRC -> (3d) Settings & AI Drawer; absorb `hime_hotkey`. | Visual parity across 3 edges, all hotkey/taskbar/notification tests pass, steady-state RAM <= 45 MB. |
| **Phase 4: Cutover & Archive WebView2** | Repo-wide | Replace Tauri multi-WebView shell with native `hime-island.exe` + modular `hime-ai.exe`; keep `9Router` external/optional. | Full E2E desktop verification, cold start < 0.5s, zero WebView2 processes when AI module is off. |
