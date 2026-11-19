# Bloom Code & Design Conventions (Host Standards)

> **Authority**: This document defines the engineering and design conventions of the host Bloom codebase (`v3.10.2`, SHA `0ab9f2f08b75cfeffd3040330e312c2244d7726a`), verified by direct static analysis of the source code. All Hime Shell code (`src/hime/**`, `src-tauri/src/hime/**`) must adhere strictly to these standards.

---

## 1. Toolchain & Environment

- **Host Platform**: Windows 10/11 x64 (MSVC).
- **Package Manager & Script Runner**: **Bun** (v1.4.x).
  - Install dependencies: `bun install --frozen-lockfile`.
  - **Prohibition**: Never run `npm install` or generate `package-lock.json` in root.
- **Frontend Build**: Vite 7 + React 19.3 + TypeScript 5.9.
  - Strict mode enabled (`noUnusedLocals: true`, `noUnusedParameters: true`).
  - Vite dev server runs strictly on port `1420` (`vite.config.ts:16`). Do not run Vite separately from `tauri dev`.
  - Typecheck command: `bun run build` (`tsc && vite build`).
- **Code Formatter**: `oxfmt` (configured in `.oxfmtrc.json`) for frontend, `cargo fmt` for Rust. Run via `bun run format`.
- **Rust Toolchain**: Rust stable MSVC (2021 edition), Cargo monorepo in `src-tauri`.
  - Quality gates: `cargo check --locked`, `cargo clippy --locked --all-targets`, `cargo test --locked`.

---

## 2. Dynamic Island Animation & Spring Physics (`src/App.tsx`)

Bloom's Dynamic Island is **one single continuous shape** that morphs its geometry dynamically. It is **never** a standard rectangular desktop window with borders or titlebars.

### 2.1 Authoritative Spring Parameters (`src/App.tsx:3270-3278`)
Every geometric property animates independently to achieve fluid fluid-dynamics morphing:

| Property | Spring Type | Stiffness | Damping | Mass | Rest Delta | Notes |
|---|---|---|---|---|---|---|
| **`width`** | `spring` | `400` | `31` | — | — | Smooth horizontal expansion/contraction |
| **`height`** | `spring` | `450` | `29` | — | — | Vertical panel opening and closing |
| **`y`** (Position) | `spring` | `550` | `45` | `0.8` | `0.001` | Snappy dock/undock from top screen edge |
| **`borderTopLeftRadius`** | `spring` | `1000` | `40` | — | — | Corner morphing without visual seams |
| **`borderTopRightRadius`**| `spring` | `1000` | `40` | — | — | Corner morphing without visual seams |
| **`borderBottomLeftRadius`** | `spring` | `1000` | `40` | — | — | Corner morphing without visual seams |
| **`borderBottomRightRadius`**| `spring` | `1000` | `40` | — | — | Corner morphing without visual seams |
| **`default`** (Content) | `spring` | `500` | `30` | `1.0` | — | Internal element transitions |

### 2.2 Standard Geometry & Dimensions Table (`src/App.tsx:3088-3140`)

| Mode / Panel | Width (px) | Height (px) | Form Factor |
|---|---|---|---|
| **Collapsed Pill** | `140` (or `30.6` mini) | `28.9` (compact) / `34` – `44.2` | Minimal top notch |
| **Impacted / Active** | `170` – `200` | `34` | Pill pulsing with status/now-playing |
| **Command Center** | `375` | `230` | Quick controls (volume, brightness, toggles) |
| **Calendar & Pomodoro** | `480` | `112 + rows*30` (~262–292) | Month grid + countdown controls |
| **Shelf** | `380` | `130` (empty) / `176` – `190` | File drop zone & clipboard shelf |
| **Connected Devices** | `350` | `150` – `300` (auto-fit) | Battery levels and peripheral status |
| **Wi-Fi / Bluetooth** | `375` | `300` | Network selection view |
| **Media Player** | `300` (compact) / `340` (full) | Expanded | Album art, progress, playback buttons |
| **Hime Assistant (Target)** | `380` | `280` – `320` | Chat, turn execution, tool approval |
| **Hime Translation (Target)**| `340` | `130` – `150` | Selected-text instant translation card |

---

## 3. Styling, Theme & Design System

1. **No External UI Component Libraries**:
   - Upstream Bloom does **not** use shadcn/ui, MUI, Chakra, AntD, or Radix themes.
   - All styling is authored with **Tailwind CSS v4** utility classes and custom CSS (`App.css`, `Dock.css`, `Settings.css`).
2. **Color Tokens & Theme Engine (`src/theme.ts`)**:
   - The theme is driven by CSS variables (`--bg`, `--fg`, `--border`, `--accent`).
   - Initialized via `initTheme()` which listens for Windows system theme changes (Dark/Light mode).
   - Glassmorphism is achieved using Windows 11 **Mica / Acrylic** (`window-vibrancy` crate in Rust and `appWindow.setEffects({ effects: ["mica"], state: "active" })` in frontend).
3. **Typography**:
   - Primary font: Inter / Segoe UI Variable / System font stack.
   - Compact, high-density line heights, crisp anti-aliasing.
4. **Iconography**:
   - Exclusively `lucide-react` (size `16` to `20`).
   - Custom SVG icons for specific hardware or brand logos are defined as pure SVG components in `src/icons.tsx`.
   - **Zero generic emojis** are used in the UI.

---

## 4. Settings Architecture (`SETTINGS.md`, `src/hooks/useSettingsSync.ts`)

- **Location**: `%APPDATA%/bloom/settings.json`.
- **Format**: Flat JSON key-value map.
- **Value Invariant**: **All values are strings** (booleans are `"true"` / `"false"`, numbers are serialized as strings).
- **Read Path**: Cached in memory by Rust backend (`src-tauri/src/utils.rs:get_setting_str`), synchronized to frontend via `useSettingsSync`.
- **Write Path**: Frontend updates `localStorage` and calls `invoke("save_setting", { key, value })`. Rust backend persists to disk and broadcasts `settings-changed` to all webview windows.
- **Hime Convention**: All new Hime settings must use the prefix `hime-` (e.g., `hime-model-name`, `hime-auto-translate`), documented in `SETTINGS.md`.

---

## 5. Rust Backend Architecture

- **Entry Point**: `src-tauri/src/main.rs`.
- **Command Dispatcher**: Every `#[tauri::command]` must be registered in the `generate_handler![...]` macro in `main.rs:160-230`.
- **IPC Modules**:
  - `commands.rs`: Frontend IPC invocation targets.
  - `services.rs`: Background Win32 hooks (low-level keyboard hooks, audio visualizer WASAPI loop, taskbar AppBar registration, window z-order watchdog).
  - `utils.rs`: Settings caching, file system IO, taskbar visibility APIs.
  - `state.rs` / `types.rs`: Thread-safe statics (`AtomicBool`, `Mutex`) and shared IPC structs.
- **Event Architecture**:
  - Backend emits events via `app.emit("event-name", payload)`.
  - Frontend listens via `listen("event-name", callback)`.
- **Taskbar Safety Contract**:
  - Win32 API `ShowWindow(tray_hwnd, SW_HIDE)` hides the taskbar.
  - Every application exit, crash handler, or watchdog must call `ShowWindow(tray_hwnd, SW_SHOW)` and `SHAppBarMessage(ABM_REMOVE)` to guarantee the user's desktop taskbar is restored.

