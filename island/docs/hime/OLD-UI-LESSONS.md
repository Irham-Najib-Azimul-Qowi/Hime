# Lessons Learned from Legacy Desktop Shell (Phase 01–15 UI)

> Date: 2026-11-16  
> Author: Principal Engineer / Tech Lead  
> Scope: Distillation of behavioral requirements, security workflows, and IPC contracts from `apps/hime-desktop` prior to purge.

---

## 1. Why the Legacy UI Failed (Root Cause Analysis)

1. **Standard Windows Window Paradigm instead of Dynamic Island**:
   - The desktop shell was built using standard Tauri window styling (`decorations: true`, rectangular viewport `1200x800` or framed `460x44` box).
   - The "Dynamic Island" was merely a CSS `div` centered inside an opaque/translucent desktop window card with purple glow (`rgba(139, 92, 246, 0.2)`), not a borderless, docked AppBar.
   - When the user pressed `Win + D` or minimized windows, the island disappeared because it was not registered as a desktop AppBar or floating top-level utility window.
2. **"AI Slop" Design Aesthetic**:
   - Monolithic dashboard grid with generic dark card layouts, neon status badges, decorative stats, and standard titlebar minimize/maximize/close icons directly on top of the island.
   - Did not morph smoothly with spring physics; instead used generic CSS `transition: 220ms cubic-bezier(...)` causing visual jumpiness and border-radius snapping.
3. **Bloom Native Solution**:
   - Bloom runs as a dedicated transparent, frameless, topmost AppBar (`transparent: true`, `decorations: false`, AppBar DLL).
   - Island uses Framer Motion spring physics with verified parameters (`width`: stiffness 400, damping 31; `height`: stiffness 450, damping 29; `borderRadius`: stiffness 1000, damping 40) on a single morphing `<motion.div>`.
   - Never use window frames, card chrome, or decorative glows.

---

## 2. Core Behavioral Requirements Harvested

### 2.1 Action Approval Lifecycle (Security PEP/PDP)
- **Trigger**: When an agent/tool attempts a risky operation (e.g. system file access, remote shell execution, sensitive API key access), Core returns:
  ```json
  {
    "decision": "pending_approval",
    "request_id": "appr-...",
    "description": "Delete 4 redundant files in X:/Downloads",
    "expires_at": 1794651234567
  }
  ```
- **TTL (Time to Live)**: Exactly 60 seconds (`expires_at = now + 60_000`).
- **One-Time Token**: The `request_id` is a cryptographic one-time ticket. Once resolved (approved or denied), Core immediately invalidates it.
- **Fail-Closed**: If TTL expires without explicit user resolution, the action is automatically denied by Core.
- **Anti-Replay**: Any repeated submission of an already-resolved or expired `request_id` is rejected by Core.
- **UI State**: In the Bloom Island, approvals should surface as an urgent Live Activity Bubble with clear countdown and one-tap Approve / Deny actions.

### 2.2 Activity Event Stream Model
- **Event Structure**:
  - `id`: Monotonic sequence integer (used for deduplication).
  - `timestamp`: Unix epoch milliseconds.
  - `event_type`: e.g., `tool_execution`, `security_violation`, `ai_turn_start`, `task_created`.
  - `source`: e.g., `core`, `hermes`, `tool_gateway`, `vault`.
  - `payload`: Structured JSON string or details.
- **Deduplication**: UI maintains a rolling window (default 30–100 events) and ignores any event where `id <= last_seen_id`.

### 2.3 IPC Communication Protocol
- **Transport**: Win32 Named Pipe `\\.\pipe\hime-ipc` with session token authentication.
- **Framing**: 12-byte header (`[HIME 4B][version 2B][flags 2B][payload_len 4B]`) + JSON-RPC 2.0 payload.
- **Asynchronous**: Zero blocking calls on the UI thread.
- **Decoupled Fallback**: If Core process is not running or pipe is disconnected, the shell stays 100% operational as standard Bloom, showing a clean, honest "Offline" status without crashing or hanging.

---

## 3. Required UI States for Bloom Integration

1. **Collapsed Pill (Idle)**:
   - Geometry: 140px × 34px.
   - Shows: Subtle status dot (green when Core online, muted when offline), current time/now-playing or minimal Hime indicator.
2. **Live Activity Bubble (Active Agent Turn / Approval / Audio / Pomodoro)**:
   - Width: ~280px–340px, Height: ~44px–60px.
   - Shows active thinking spinner, tool name, or approval countdown.
3. **Assistant Panel (Expanded)**:
   - Geometry: 380px × 280px–340px.
   - Clean, compact chat input and response card seamlessly blending into Bloom's backdrop blur and typography.
4. **Hime Settings (Settings Tab)**:
   - Native tab inside Bloom's `Settings.tsx` (`TabsContent value="hime"`).
   - Core connection status, AI model selector, vault status, telemetry toggle.
