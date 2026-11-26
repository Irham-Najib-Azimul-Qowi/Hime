# Bloom Touchpoint Candidates for Hime Integration

This document defines the planned, minimal touchpoints in Bloom required to integrate Hime features. Every touchpoint is strictly additive and designed to preserve host stability.

---

## 1. Candidate Integration Points

### TC-01: Island Panel Registration
- **Target File**: `src/App.tsx:86-89`
- **Location**: `type NotchPanel = "calendar" | "command-center" | "devices" | "assistant";`
- **Integration**:
  - Adds `"assistant"` as an available user tool panel in the notch router.
  - Sizing rules in `getDynamicWidth` (`380px`) and `getDynamicHeight` (`280–320px`) matching `command-center` proportions.
  - Component rendered: `<HimeAssistantPanel />` from `src/hime/components/HimeAssistantPanel.tsx`.

### TC-02: Live Activity Bubble Registration
- **Target File**: `src/App.tsx:3400-3600`
- **Location**: Bubble container alongside pill.
- **Integration**:
  - When Hime Agent is actively executing tasks or awaiting L3/L4 approval tickets, a live activity bubble appears beside the pill.
  - Morphing and spring physics use Bloom's standard `{ type: "spring", stiffness: 400, damping: 25 }`.
  - Component rendered: `<HimeActivityBubble />` from `src/hime/components/HimeActivityBubble.tsx`.

### TC-03: Selected-Text Translation View
- **Target File**: `src/App.tsx:3090-3120`
- **Location**: Mode routing inside Dynamic Island (`bloomMode === "translation"`).
- **Integration**:
  - Activated by global hotkey (`Ctrl + Shift + T`) or clipboard text selection event via Hime Core.
  - Morphs pill to `340px × 140px` with source text and target translation.
  - Dismisses on click-outside or Escape key.

### TC-04: Settings Tab Integration
- **Target File**: `src/Settings.tsx:25-32` and `src/settings/index.ts`
- **Location**: `TABS` array in `Settings.tsx`.
- **Integration**:
  - Adds `{ id: "hime", label: "Hime AI", icon: Sparkles }`.
  - Renders `<HimeSettingsTab />` from `src/hime/settings/HimeSettingsTab.tsx`.
  - Configures model selection, Core connection status, 9Router ports, and translation language defaults.

### TC-05: Rust IPC Command Registration
- **Target File**: `src-tauri/src/main.rs:160-230`
- **Location**: `tauri::generate_handler![...]`
- **Integration**:
  - Registers additive commands defined in `src-tauri/src/hime/mod.rs`:
    - `hime_get_core_status`
    - `hime_send_prompt`
    - `hime_resolve_approval`
    - `hime_trigger_translation`
    - `hime_open_workspace`

### TC-06: On-Demand Workspace Window
- **Target File**: `src-tauri/tauri.conf.json:35`
- **Location**: `app.windows` array.
- **Integration**:
  - Adds window entry with `"label": "hime-workspace"`, `"create": false`, `"url": "hime.html"`.
  - Created only on-demand when user expands into full agent canvas / workflow inspector.
  - Zero resource consumption when closed.

