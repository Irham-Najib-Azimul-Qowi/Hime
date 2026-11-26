# AGENTS.md

Windows-only Tauri v2 app: Rust backend + React 19/TypeScript frontend. Replaces the native Windows taskbar (dock) and adds a notch/island UI. Bun is the package manager. CI runs on `windows-latest`.

## Commands

Run from the repo root unless noted. Bun 1.4.0, Rust stable.

- `bun install` — deps (CI uses `bun install --frozen-lockfile`; never create a root `package-lock.json`)
- `bun run tauri dev` — full dev loop. Vite is on strict port 1420; do not start `bun run dev` separately
- `bun run build` — `tsc && vite build`; this is the frontend typecheck. `strict` + `noUnusedLocals`/`noUnusedParameters` fail the build. There is no root lint or frontend test script
- `cargo check --locked` and `cargo clippy --locked --all-targets` in `src-tauri` — CI's Rust gates
- `cargo test --locked` in `src-tauri` — pure-function unit tests. Runs fast on Windows but CI does **not** run it, so run it yourself
- `bun run bump <x.y.z>` — bumps `package.json`, `tauri.conf.json`, `Cargo.toml`, `Cargo.lock` together
- `bun run release <x.y.z>` — bump + commit + tag + push, must be on `main`. The release workflow fails unless all four version files match the tag. Before/after releasing, follow `RELEASE_CHECKLIST.md` (updater manifest verification, N‑1 install smoke test)
- `website/` is a separate static site: use `npm install` / `npm run dev` there. Root scripts and CI do not build it

## Architecture

- Four webview windows, each a separate Vite/HTML entry: `main` = notch (`index.html` → `src/main.tsx` → `src/App.tsx`), `dock` (`dock.html` → `src/Dock.tsx`), `overlay` (`overlay.html` → `src/Overlay.tsx`), `settings` (`settings.html` → `src/Settings.tsx`). Adding or renaming a window means editing `src-tauri/tauri.conf.json`, `vite.config.ts` inputs, the HTML file, and `src-tauri/capabilities/default.json`
- Rust entrypoint is `src-tauri/src/main.rs`; `lib.rs` is an empty stub. Every `#[tauri::command]` must be registered in the `generate_handler!` list in `main.rs`
- `commands.rs` = IPC surface; `services.rs` = long-lived Win32 hooks/workers (keyboard hook, thumbnail capture, audio visualizer, AppBar registration); `utils.rs` = settings cache and file IO; `state.rs`/`types.rs` = shared statics/types
- Backend→frontend events use `app.emit`: `settings-changed`, `settings-external-changed`, `media-update`, `volume-change`, `brightness-change`, `dock-overlap`, `notch-overlap`, `windows-changed`, `update-available`, `auto-update-status`. Listen via `@tauri-apps/api/event`

## Settings

- Stored in `%APPDATA%/bloom/settings.json` as a flat object of `bloom-`-prefixed keys; all values are strings (`"true"`/`"false"` for booleans). `SETTINGS.md` documents every key
- UI write path: set `localStorage`, then `invoke("save_setting")` → backend caches it and emits `settings-changed`. External file edits are watched and emitted as `settings-external-changed`
- `src/hooks/useSettingsSync.ts` is the single sync hook for both events and normalizes string booleans/numbers
- The CSP in `tauri.conf.json` allow-lists network origins (weather APIs only); add new origins there. New Tauri plugins also need permissions in `capabilities/default.json`

## Gotchas

- The app hides the native Windows taskbar and registers AppBars while running. Any new exit/restart path must restore the taskbar and unregister AppBars, like the tray menu in `main.rs` does. A marker file restores it after a crash
- The `windows` crate is pinned to 0.61 with an explicit feature list in `Cargo.toml`; add features there when using new Win32 APIs
- Commits follow conventional commits (`feat:`, `fix:`); release notes are generated from them and commits touching only `website/*`, `.github/*`, or `*.md` are filtered out

---

## Hime Engineering Rules & Boundaries

Hime is built as an additive integration on top of Bloom. Bloom is the host environment; Hime conforms to Bloom's architecture, conventions, and styling.

### 1. Code Placement & Isolation
- **Frontend**: All Hime React components and logic live strictly in `src/hime/**`.
- **Shell Backend**: All Hime Tauri Rust logic lives strictly in `src-tauri/src/hime/**`.
- **Hime Core**: Independent standalone Rust process living in `core/` (`core/crates/hime-*`).
- **Touchpoints**: Any minimal edit to upstream Bloom files (`src/App.tsx`, `src-tauri/src/main.rs`, etc.) is strictly additive and must be registered in `docs/hime/TOUCHPOINTS.md`. Never refactor Bloom files.

### 2. Design Principles (Zero AI Slop)
- Dynamic Island is a single morphing `<motion.div>` driven by Framer Motion spring physics (`width`: stiffness 400, damping 31; `height`: stiffness 450, damping 29; `borderRadius`: stiffness 1000, damping 40).
- Never render desktop titlebars, window chrome, or generic cards inside the island.
- Follow Tailwind v4 utility patterns and existing Bloom design tokens.

### 3. Build & Test Commands for Hime
- `powershell scripts/build-core.ps1` — Builds standalone Hime Core (`core/`).
- `powershell scripts/run-safe.ps1` — Runs Bloom shell with watchdog protection and taskbar auto-restore.
- `powershell scripts/dev-hime.ps1` — Builds Core and launches full development environment safely.
- In `core/`: `cargo check --locked` and `cargo test --locked`.
- In `src-tauri/`: `cargo check --locked` and `cargo test --locked`.
- In root: `bun run build` (`tsc && vite build`).

### 4. Zero Push Constraint
- Under no circumstances run `git push` or `bun run release`. Remote `upstream` is read-only.
- All commits are strictly local Conventional Commits.

