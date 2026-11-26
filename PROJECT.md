# Project: Hime Desktop Milestones 3, 4, 5, 6 (STAGE H2)

## Architecture
- **Desktop Shell (`island/`)**: Tauri 2.0 (Rust native shell `island/src-tauri` + React 19 / TypeScript / Bun frontend `island/src`), rendering the floating Dynamic Island overlay, tray menu, window management, and taskbar integration.
- **Kernel (`core/`)**: Rust virtual workspace (`hime-core`, `hime-pal`, `hime-ipc`, `hime-security`, `hime-storage`, etc.) handling background AI runtimes (Hermes, 9Router) via Win32 Job Object supervision.
- **Global Hotkey Daemon (`island/src-tauri/src/bin/hime_hotkey.rs`)**: Dedicated Win32 background process managing system-wide keyboard shortcuts (`Ctrl + Alt + Space`, `Ctrl + Alt + H`, `Ctrl + Alt + Q`).
- **Supervision & Scripts (`scripts/`)**: PowerShell supervisor watchdog (`hime-supervisor.ps1`), lifecycle scripts (`start-hime.ps1`, `stop-hime.ps1`, `hime.ps1`), and quality gates (`check-brand.ps1`, `check-hygiene.ps1`).
- **Coordination Primitives**:
  - Win32 Named Event `HimeQuit`: Broadcast manual-reset event for instantaneous multi-process shutdown.
  - Marker file `%APPDATA%\ai.hime.desktop\hime.active`: Indicates ecosystem active state, persisting across laptop reboots unless manually stopped.
  - IPC Named Pipe `\\.\pipe\hime-ipc`: 12-byte binary framed JSON-RPC 2.0 transport between Tauri shell and Hime Core.

## Feature Inventory
| # | Feature | Description | Milestone | Source |
|---|---------|-------------|-----------|--------|
| 1 | `HimeQuit` Named Event Broadcast | Win32 manual-reset named event for coordinated clean shutdown across all processes | M3 | survey |
| 2 | Tray Menu Clean Quit | Context menu item "Quit Hime" removes `hime.active`, signals `HimeQuit`, restores taskbars, and cleanly exits | M3 | survey |
| 3 | Global Hotkey `Ctrl + Alt + Q` Quit | Dedicated hotkey removes `hime.active`, signals `HimeQuit`, and coordinates shutdown | M3 | survey |
| 4 | Supervisor Event Wait & Clean Exit | `hime-supervisor.ps1` waits on `HimeQuit` via `EventWaitHandle`, exiting immediately without delay | M3 | survey |
| 5 | Core Kernel `HimeQuit` Listener | `hime-core` listener thread triggers graceful shutdown on `HimeQuit` | M3 | survey |
| 6 | Reboot Persistence Discrimination | `restore_taskbar_and_exit()` checks `GetSystemMetrics(SM_SHUTTINGDOWN)` to preserve `hime.active` across laptop restart | M3 | survey |
| 7 | Graceful Shutdown Coordination | `stop-hime.ps1` signals `HimeQuit` and grants graceful exit window before fallback force-kill | M3 | survey |
| 8 | Hotkey Daemon Single Instance | `hime_hotkey.rs` enforces single-instance mutex to avoid duplicate daemons | M3 | survey |
| 9 | Monorepo 02b Structure Alignment | Move root tracking files (`BACKLOG.md`, `RISK_REGISTER.md`), remove duplicate `island/core`, consolidate `docs/legal/` and `docs/hime/` | M4 | survey |
| 10 | Workspace Standards & Lockfiles | Add `docs/INDEX.md`, `docs/README.md`, `modules/*/README.md`, `runtimes/runtimes.lock.json`, `.gitattributes`, `.editorconfig`, dual-workspace ADR | M4 | survey |
| 11 | Source Brand Compliance | Neutralize all unwhitelisted brand strings in `island/src-tauri` (`commands.rs`, `services.rs`) and rename tools | M4 | survey |
| 12 | Brand Asset Purge & Mascot Generation | Remove 6.6MB dead Bloom PNGs from `island/public/` and ensure Hime mark icons in `src-tauri/icons/` | M4 | survey |
| 13 | Brand Gate Script Enhancement | Expand `scripts/check-brand.ps1` `$targetDirs` to verify `src-tauri` and `tools` with 0 errors | M4 | survey |
| 14 | Code Hygiene Gate Enhancement | Expand `scripts/check-hygiene.ps1` `$scanDirs` to cover `island/src`, `island/src-tauri`, `scripts`, `tools` with 0 errors | M4 | survey |
| 15 | Bloom GPL-3.0 Licensing & About Dialog | Verify root `LICENSE` and `NOTICE`; add dedicated "License & Open Source Notices" section in `AboutTab.tsx` | M4 | survey |
| 16 | Island Unit Test Invariant Fix | Update `services.rs:4810` test assertion to match Hime floating overlay invariant (never reserves work area) | M5 | survey |
| 17 | Automated Full E2E Verification Suite | Script `scripts/verify-e2e.ps1` running brand gate, hygiene gate, Core tests (124+), Frontend Bun tests (65), and Tauri tests (104) | M5 | survey |
| 18 | Live Desktop Application Validation | Automated live test script validating process presence, floating island visual capture, tray interaction, and hotkey shutdown | M5 | survey |
| 19 | Mascot Eyes SVG Mask & Blink Engine | Replace SVG clipPath with SVG `<mask id="...">` using `#FFFFFF` base and `#000000` cutouts, random blinking (3000-7000ms), and micro-saccades in `HimeEyes.tsx` | M6 | survey |
| 20 | Closed Notch Pill Mascot Integration | Replace legacy `<Bot />` icon in `island/src/App.tsx:3884` with `<HimeEyes size={14} expression={himeExpression} />` for states `netral`, `berpikir`, `senang`, `offline` | M6 | survey |
| 21 | Event-Driven IPC Core Status | Remove `setInterval(refresh, 3000)` polling in `useHimeEvents.ts` for zero idle polling; subscribe purely to `hime-core-status` events | M6 | survey |
| 22 | RPC Method Name Alignment | Change `"approvals.resolve"` to `"approval.resolve"` in `island/src-tauri/src/hime/services.rs:263` matching `server.rs:517` | M6 | survey |
| 23 | Vertical Approval Bubble & Text Limits | Anchor approval bubble vertically below notch using `.activity-bubble` (auto click-through); enforce title <= 16 chars ("Izin Operasi") and description <= 36 chars | M6 | survey |
| 24 | Foundational Notch Compose Manager (NCM) | Formalize `Presentation` data model and P0-P5 priority arbiter in `island/src/hime/ncm/` with fallback to closed pill clock | M6 | survey |
| 25 | Full 22 Expressions Catalog & SVG Mask Model | Parametric eye geometry, colors, asymmetric lids, and SVG masks for all 22 states in `HimeEyes.tsx` | M7 | survey |
| 26 | Idle Director & Ambient Engine | Contextual weighted peeks (4–7s), fallback to clock, and suppression rules in `useNCM.ts` | M7 | survey |
| 27 | 8 Mascot Keyframe Motion Presets | CSS keyframes for `bounce-soft`, `pulse-twice`, `wander-slow`, `audio-pulse`, `pop`, `breathing-slow`, `jitter-2px`, `orbit-small` | M7 | survey |
| 28 | Spring Physics CSS Variables | `--anim-spring-snappy: cubic-bezier(0.16, 1, 0.3, 1)` (220ms), stiffness 520, damping 38, mass 1 compiled to `tokens.css` | M7 | survey |
| 29 | Dynamic Tri-State Left Slot Arbitration | Left slot arbitration: `eyes (meaningful) > status-icon > clock` with dynamic center/right slots | M7 | survey |
| 30 | NCM Preemption Resumption & TTL Freeze | Preempted presentations resume upon higher-tier completion; hover freezes 6-10s timer | M7 | survey |
| 31 | Canonical Settings Schema Registry | Single source schema (`src/hime/settings/schema.ts`) driving 17 canonical settings sections | M7 | survey |
| 32 | Mascot Expression Tester UI Component | Interactive previewer for all 22 expressions mounted in Settings (`ExpressionTester.tsx`) | M7 | survey |
| 33 | Global Shortcuts Cheat Sheet Modal | In-notch cheat sheet modal for 12 global and 11 local shortcuts (`CheatSheet.tsx`) | M7 | survey |
| 34 | UI Text Limits Compliance | Full audit and remediation of copy (Titles <= 16 chars, Descriptions <= 36 chars, Buttons <= 10 chars) | M7 | survey |
| 35 | Win32 Hotkey Daemon Instant Wakeup | Named Event `HimeShowIsland` eliminates PowerShell launch overhead when island is active | M7 | survey |
| 36 | Hime Persona & Anti-Slop Injection | `core/prompts/hime-persona.md` template loaded into Hermes adapter; Indonesian santai-sopan, no preamble, no Bloom leak | M8 | survey |
| 37 | Single Global Thread Singleton | Continuous conversation thread `conv-global` in SQLite; rolling summary and virtualized scrolling | M8 | survey |
| 38 | Multi-Run Non-Blocking Scheduler | Input clears immediately (<=1s); parallel execution up to default 2; excess queued as `Antre #n` | M8 | survey |
| 39 | Interactive Process Run Card | Inline card (`RunCard.tsx`) with progress pill, step activity log, cancel `✕`, morphing in-place into assistant markdown reply | M8 | survey |
| 40 | Targeted Run Cancellation | Cancel specific active run by `run_id` via `/stop` or `✕` without interrupting sibling runs | M8 | survey |
| 41 | Universal Multimodal File Attachments | Ingest all file types via `+` picker, drag-and-drop on notch, and clipboard paste; category chips with icon, size, and SHA-256 hash | M8 | survey |
| 42 | Voice Push-to-Talk & Real-Time Transcript | `Ctrl+Alt+M` or mic button; Web Audio API RMS meter driving `HimeEyes` `dengar` pulse <=30Hz; real-time streaming transcript into input | M8 | survey |
| 43 | Slash (`/`) Skill Autocomplete Popover | Keyboard-navigable autocomplete overlay filtered against skill catalog with category colors and descriptions | M8 | survey |
| 44 | Solid Categorized Skill Pills | `<SkillPill />` rendered with dark `#0A0B0E` text on category accents (sky, amber, mint, violet, lime, cyan, pink, coral, text-3) | M8 | survey |
| 45 | Fast Control Commands Interception | Direct interception and execution of `/stop`, `/jeda`, `/cari`, and `/bersih` | M8 | survey |
| 46 | NCM Progress & Mascot Synchronization | Active run emits `P3_PROGRESS` presentation (`userInitiated: true`); left slot shows `berpikir`/`fokus`; auto-restores to clock | M8 | survey |
| 47 | Strict UI Copy Limits Compliance | All STAGE 08R titles <= 16, descriptions <= 36, buttons <= 10, errors <= 44 chars | M8 | survey |
| 48 | Core LRCLIB HTTPS Client & Host Allowlist | Strict Core network rule: allowlist `lrclib.net` only, User-Agent `HimeIsland/<version>`, 5s timeout, 404 negative cache, 429 backoff | M9 | survey |
| 49 | Core SQLite Lyrics Persistence & User Offset | SQLite table `lyrics_cache` in `%APPDATA%\Hime\hime_core.db`, 6h negative TTL, 30d hit TTL, per-track user offset ms | M9 | survey |
| 50 | Rust & TypeScript LRC Format Parsers | Accepts `[mm:ss.xx]` and `[mm:ss.xxx]`, multi-tag expansion, offset shifting, sorted chronologically, tested with property tests | M9 | survey |
| 51 | Windows GSMTC Album Metadata Extension | `island/src-tauri/src/services.rs` extracts `props.AlbumTitle()`, updating `MediaInfo` struct for exact LRCLIB matching | M9 | survey |
| 52 | Lyrics Named Pipe IPC & Tauri Bridge | JSON-RPC methods `lyrics.get`, `lyrics.set_offset`, `lyrics.clear_cache` bridged to Tauri commands | M9 | survey |
| 53 | Zero-Polling Real-Time Synchronization Engine | `useLyrics.ts`, monotonic timeline extrapolation, $O(\log N)$ binary search active line, event-driven one-shot `setTimeout` to next line onset, zero CPU idle | M9 | survey |
| 54 | NCM Closed Pill Ambient Lyric Presentation | `P5_AMBIENT` presentation `music-lyrics` rendering active line <= 36 chars in closed pill center slot, auto-preempted by P0-P4 and cleanly resumed | M9 | survey |
| 55 | Expanded Media Panel Lyrics View & Floating Bubble | `CompactMediaPlayer.tsx` dedicated Lirik tab with Framer Motion spring vertical scroll, offset controls `[-0.5s]` `[+0.5s]`, fallback un-synced and instrumental states | M9 | survey |
| 56 | Lyrics Settings & Shortcuts Integration | `schema.ts` Section 11 9 canonical keys, global hotkey `Ctrl+Alt+L`, local keycaps `[` / `]` | M9 | survey |
| 57 | Core Task Engine & SQLite Persistence | SQLite table `tasks` with CRUD, urgency levels, state machine, and periodic scheduler | M10 | survey |
| 58 | Frontend Tasks Panel & Segmented Tabs | `HimeTasksPanel.tsx` with tabs "Hari ini", "Mendatang", "Selesai", progress ring, and task deletion/completion | M10 | survey |
| 59 | Natural Language Quick Add Bar | `QuickAddBar.tsx` invoking `hime_parse_task_natural` with priority and date preview pills | M10 | survey |
| 60 | Dynamic Island Tasks Mode & Shortcuts | Full wiring into `App.tsx` (`NotchPanel`, `BloomMode`, hotkey `Ctrl+Alt+N`, dimensions 380x380) | M10 | survey |
| 61 | Chat-Task Bidirectional Switching | Header affordances in `ChatHeader.tsx` and `HimeTasksPanel.tsx` for seamless 1-click panel toggling | M10 | survey |

## Milestones
| # | Name | Scope | Dependencies | Status |
|---|------|-------|-------------|--------|
| 3 | Clean Shutdown & Process Persistence | Win32 Named Event `HimeQuit`, tray quit handler, hotkey `Ctrl + Alt + Q`, supervisor event wait, reboot persistence check, graceful stop | None | DONE |
| 4 | Monorepo Compliance, Hygiene, and Licensing | Monorepo 02b layout alignment, brand/hygiene scripts passing 0 errors, Bloom GPL-3.0 licensing in LICENSE, NOTICE, About | M3 | DONE |
| 5 | Automated E2E Verification & Live Desktop Testing | Fix unit test assertion, automated E2E test runner (all tests pass), live desktop visual & functional validation | M3, M4 | DONE |
| 6 | STAGE H2: Bridge + Irisan Vertikal Island | Mascot `HimeEyes` SVG mask & closed pill integration, event-driven IPC bridge, vertical approval bubble, UI text constraints, and foundational NCM | M5 | DONE |
| 7 | STAGE U1: NCM + Mata 22 Ekspresi + Animasi + Pintasan + Settings | Full 22 expressions catalog, Idle Director, 220ms spring transitions, new shortcuts & settings per 13-SETTINGS-DAN-PINTASAN.md | M6 | DONE |
| 8A | STAGE 08R Backend: AI Bridge & Persona Hime | `hime-persona.md`, Hermes adapter, single thread `conv-global`, multi-run cancellation, and IPC streaming | M7 | DONE |
| 8B | STAGE 08R Frontend: Multi-Run & Skills Chat | Modular chat package in `island/src/hime/chat/`, non-blocking input, run cards, attachments, voice, and slash pills | M8A | DONE |
| 9 | STAGE U2: Lirik Real-Time | LRCLIB client integration, local lyric cache, synchronized LRC parser, dynamic Island lyric bubble | M8B | DONE |
| 10 | STAGE 09R: Intelijen Core, Tugas & Rutinitas | System prompt, proactive engine, task management, translation, saved shelf | M9 | DONE |

## Interface Contracts
### `HimeQuit` Win32 Named Event Contract
- **Type**: Win32 Named Event, Manual Reset (`bManualReset = TRUE`, `bInitialState = FALSE`).
- **Name**: `"HimeQuit"` (or `Local\HimeQuit`).
- **Signaler**: Tray Menu Quit, Hotkey `Ctrl + Alt + Q`, In-app Quit, or `stop-hime.ps1`.
- **Listeners**:
  - `hime-island.exe`: Spawns background thread waiting on `HimeQuit`. When signaled, calls `restore_taskbar_and_exit()`.
  - `hime-core.exe`: Background thread connects to tokio broadcast channel, drops `JobSupervisor` (terminating child processes), and exits code 0.
  - `hime-supervisor.ps1`: `EventWaitHandle.WaitOne(10000)` wakes up immediately on signal and exits loop without respawning processes.
- **Marker Synchronization**: Signaler MUST remove `%APPDATA%\ai.hime.desktop\hime.active` prior to or in conjunction with `SetEvent(HimeQuit)` when quit is user-initiated.
- **Reboot Persistence Invariant**: On OS logoff/shutdown (`GetSystemMetrics(SM_SHUTTINGDOWN) != 0`), `hime.active` MUST NOT be deleted.

### STAGE H2 & U1 Mascot & IPC Interface Contracts
- **Mascot Expressions (Full 22 Catalog)**:
  - `netral`: Cyan capsules (`#38D6F5`), default resting, random blinking (3000–7000ms), saccades.
  - `senang`: Mint capsules (`#34E3A6`), curved bottom lid ("^ ^"), 2-4s soft bounce on approval resolution.
  - `semangat`: Mint capsules (`#34E3A6`), pulse-twice motion.
  - `bangga`: Lime capsules (`#A8EE4B`), elevated calm gaze.
  - `fokus`: Lime capsules (`#A8EE4B`), squinted locked gaze.
  - `berpikir`: Violet capsules (`#A18BFF`), gaze top-left, wander-slow motion.
  - `dengar`: Mint capsules (`#34E3A6`), audio-pulse concentric ring <= 30Hz.
  - `penasaran`: Cyan capsules (`#38D6F5`), asymmetric left eye larger + 6° head tilt.
  - `kaget`: Bright cyan (`#38D6F5`), pop expansion.
  - `khawatir`: Amber capsules (`#FFC247`), outer lid droop.
  - `bosan`: Cyan capsules (`#38D6F5`, opac 0.8), lazy side-to-side sweep.
  - `ngantuk`: Cyan capsules (`#38D6F5`, opac 0.7), slow lazy blink 250ms.
  - `menguap`: Cyan capsules (`#38D6F5`), 3-phase yawn.
  - `tidur`: Cyan horizontal slits (`#38D6F5`, opac 0.5), slow breathing.
  - `kesal`: Coral capsules (`#FF7A59`), sideways piercing glare.
  - `marah`: Red capsules (`#FF4D5E`), inner downward tilt + jitter-2px.
  - `sedih`: Sky capsules (`#5AA9FF`), outer downward tilt downcast.
  - `curiga`: Amber capsules (`#FFC247`), asymmetric squint side-eye.
  - `nakal`: Pink capsules (`#FF5C93`), playful tilted glance.
  - `kedip-sebelah`: Mint capsules (`#34E3A6`), right eye winking slit.
  - `pusing`: Violet capsules (`#A18BFF`), counter-orbiting pupils.
  - `offline`: Dim gray line (`#6F7785`), opacity 0.4, when IPC Core is disconnected.
- **NCM Left Slot Precedence**:
  - `eyes (meaningful expression) > status-icon > clock`.
  - Invariant: Clock always returns when no higher priority presentation is active.
- **Spring Physics Contracts**:
  - `--anim-spring-snappy`: `cubic-bezier(0.16, 1, 0.3, 1)`, duration 220ms ($k=520, c=38, m=1$).
  - `soft` spring: $k=200, c=26, m=1$, for mascot eye parameter transitions.
- **UI Text Limits**:
  - Judul / Title: `<= 16` characters. E.g. `"Izin Operasi"` (12 chars), `"Uji Ekspresi"` (12 chars).
  - Deskripsi / Description: `<= 36` characters (1 baris). E.g. `"Pratinjau 22 ekspresi"` (22 chars).
  - Tombol / Buttons: `<= 10` characters ("Setuju", "Tolak", "Uji Mata", "Reset").
- **Win32 Named Event `HimeShowIsland`**:
  - Auto-reset event signaled by `hime_hotkey.exe` on `Ctrl+Alt+Space` or `Ctrl+Alt+H` when `hime-island.exe` is already running to instantly show/focus the window without spawning PowerShell.

### STAGE 08R Chat, Multi-Run & Persona Contracts
- **Global Thread Identifier**: `"conv-global"` (single persistent thread across sessions).
- **Multi-Run Concurrency**: Maximum 2 active parallel runs; excess queued with status `"queued"` and label `"Antre #n"`.
- **Run Object Schema**: `id`, `prompt`, `skills: string[]`, `attachments: AttachmentRef[]`, `status: RunStatus`, `progress: number`, `statusText: string` (<= 36 chars), `steps: string[]`, `resultText?: string`, `error?: string`, `createdAt: number`.
- **NCM Active Run Presentation**:
  - `id: "chat-run-" + run.id`, `module: "chat"`, `kind: "panel-hint"`, `priority: PriorityLevel.P3_PROGRESS` (3), `userInitiated: true` (effective priority 2).
  - Left slot: `HimeEyes` with `expression="berpikir"` (or `"fokus"` during tool execution).
  - Center slot: `title: run.statusText.slice(0, 36)`.
  - Right slot: `{ progress: run.progress }`.
  - On complete: transient `expression="senang"` (2.5s TTL) -> auto-restores to clock.
- **Voice Pulse Contract**:
  - Audio capture via Web Audio API RMS meter (`audioLevel: 0.0 .. 1.0`).
  - Left slot: `HimeEyes` with `expression="dengar"` and concentric pulse ring <= 30Hz.
  - Zero audio persistence (transcription text only).
- **Skill Pill Visual Token Contract**:
  - Solid background using category accent: Tasks=`sky`, Time=`amber`, Storage=`mint`, Docs=`violet`, Dev/Code=`lime`, Web=`cyan`, Translate=`pink`, Device=`coral`, System=`text-3`.
  - Text color: Strictly `#0A0B0E` (dark) for $> 6.8:1$ contrast ratio.
- **Hime Persona Contract**:
  - Name is strictly **Hime**. No reference to "Hermes Agent" or "Bloom" in user-facing responses.
  - Tone: Indonesian santai-sopan (`aku`/`kamu`), concise, no robot preamble or emoji spam.
  - Template located at `core/prompts/hime-persona.md`.

### STAGE U2 Real-Time Lyrics & LRCLIB Contracts
- **Host Allowlist & Security (Decision U7)**: Outbound requests strictly limited to `https://lrclib.net/api/` via `hime-core`; shell remains completely offline/isolated from external web calls.
- **LRCLIB User-Agent**: `HimeIsland/3.10.2` with mandatory 5-second timeout, exponential backoff with jitter on 429, and negative cache on 404 (6-hour TTL).
- **Core SQLite Cache (`lyrics_cache`)**:
  - Key: `hex(sha256(normalize(artist) + ":" + normalize(track) + ":" + bucket_2s))`.
  - Stored fields: `cache_key`, `title`, `artist`, `album`, `duration_sec`, `source`, `synced_lyrics`, `plain_lyrics`, `parsed_json`, `is_instrumental`, `user_offset_ms`, `cached_at`, `expires_at`.
- **JSON-RPC Methods (Named Pipe)**:
  - `lyrics.get`: params `{ title, artist, album, duration_sec, allow_online }` -> returns `{ lines: LyricLine[], plainLyrics, isSynced, isInstrumental, userOffsetMs, source }`.
  - `lyrics.set_offset`: params `{ cacheKey, offsetMs }` -> updates user offset in SQLite.
  - `lyrics.clear_cache`: params `{}` -> flushes `lyrics_cache` table.
- **Timing & Synchronization**:
  - Monotonic timestamp calculation: `pos = position_ms + (isPlaying ? Date.now() - position_updated_at : 0) + user_offset_ms`.
  - Active line index: $O(\log N)$ binary search over sorted `LyricLine` timestamps.
  - Zero-polling scheduling: One-shot `setTimeout` targeted exactly to `nextLine.ms - pos`. Zero `requestAnimationFrame` loops when closed/hidden (0.0% CPU idle).
- **NCM Presentation Contract**:
  - `id: "music-lyrics"`, `module: "music"`, `kind: "ambient"`, `priority: PriorityLevel.P5_AMBIENT` (5).
  - Center slot: Active lyric line clamped strictly to 36 chars.
  - Left slot: `HimeEyes` with `expression="senang"` or `"semangat"` peeking during music; resting returns to clock.
  - Conflict resolution: Preempted cleanly by P0-P4 presentations (e.g. Translation P4, Approval P0), auto-resumes on higher priority completion.
- **UI Copy Limits**: Titles <= 16 chars, Descriptions <= 36 chars, Buttons <= 10 chars.

## Code Layout
- `island/src/hime/ui/HimeEyes.tsx`: SVG mask renderer with `#FFFFFF` base + `#000000` cutouts, 22 expressions, keyframe motions, blink timer, and saccades.
- `island/src/hime/assets/eyes.json`: Parametric eye database for all 22 expressions.
- `island/src/hime/styles/tokens.css`: Compiled design tokens including `--anim-spring-snappy` (220ms).
- `island/src/App.tsx`: Closed notch pill mascot mounting, dynamic slot rendering, activity bubbles.
- `island/src/hime/ncm/types.ts`: Presentation, Slot, and Priority models (P0–P5).
- `island/src/hime/ncm/arbiter.ts`: Priority arbiter with dynamic left slot (`eyes > status-icon > clock`), center slot, right slot.
- `island/src/hime/ncm/useNCM.ts`: React hook with preemption resumption, Idle Director ambient peeks, preserving `EMPTY_APPROVALS` & idempotent pruning.
- `island/src/hime/ncm/index.ts`: Public exports for NCM module.
- `island/src/hime/settings/schema.ts`: Single-source-of-truth schema registry for 17 canonical settings sections.
- `island/src/hime/settings/ExpressionTester.tsx`: Segmented expression selector testing all 22 expressions.
- `island/src/hime/settings/CheatSheet.tsx`: In-notch keyboard shortcuts cheat sheet modal card.
- `island/src/hime/settings/HimeSettingsTab.tsx`: Settings panel compliant with strict UI text length limits (<=16 / <=36 / <=10 chars).
- `island/scripts/ncm.test.ts`: Bun test suite testing decision table scenarios and expressions.
- `island/src-tauri/src/bin/hime_hotkey.rs`: Daemon hotkey listener with `HimeShowIsland` event trigger.
- `island/src-tauri/src/main.rs`: Background listener for `HimeShowIsland` to toggle/focus main window.
- `core/prompts/hime-persona.md`: Canonical Hime persona system prompt template.
- `core/crates/hime-core/src/ai/hermes_adapter.rs`: AI runtime adapter with persona prompt loading, multi-run cancellation map, and 9Router streaming.
- `island/src-tauri/src/hime/services.rs`: Tauri IPC bridge with global thread `conv-global` and streaming event handlers.
- `island/src/hime/chat/types.ts`: TypeScript contracts for ChatMessage, ChatRun, ChatAttachment, and SkillManifest.
- `island/src/hime/chat/skillsCatalog.ts`: Central skill definitions with category token accents and quick commands.
- `island/src/hime/chat/useChatSession.ts`: Multi-run queue management (max 2 parallel), NCM progress synchronization, and IPC messaging.
- `island/src/hime/chat/useVoiceInput.ts`: Web Audio API audio recording, RMS volume calculation, and HimeEyes `dengar` sync.
- `island/src/hime/chat/useFileAttachment.ts`: Drag-and-drop, native file dialog, clipboard paste, and category chips.
- `island/src/hime/chat/RunCard.tsx`: Non-blocking process card (running, queued, cancel button, and morph to markdown).
- `island/src/hime/chat/ChatPromptBar.tsx`: Auto-expanding 1-6 line textarea, `/` autocomplete popover, and `<SkillPill>` rendering.
- `island/src/hime/chat/ChatMessageItem.tsx`: Markdown message rendering with sanitized syntax and code copy button.
- `island/src/hime/chat/ChatHeader.tsx`: Header component with animated mascot eyes, title "Hime", and quick action menu.
- `island/src/hime/chat/index.ts`: Public module exports for Hime Chat.
- `island/src/hime/panels/HimeAssistantPanel.tsx`: Composed chat panel mounting the modular chat package.
- `island/scripts/chat.test.ts`: Bun test suite testing multi-run queue, skill autocomplete, and UI copy constraints.
- `core/crates/hime-storage/src/database.rs`: SQLite database with `lyrics_cache` table and storage methods.
- `core/crates/hime-core/src/lyrics/mod.rs`: Lyrics module exporting client, parser, and normalizer.
- `core/crates/hime-core/src/lyrics/client.rs`: HTTPS client querying `https://lrclib.net/api/` with allowlist and User-Agent.
- `core/crates/hime-core/src/lyrics/parser.rs`: Rust LRC parser handling timestamps, multi-tags, and offsets with `proptest`.
- `core/crates/hime-core/src/lyrics/normalizer.rs`: Song title cleaner stripping video/remaster suffixes.
- `island/src/hime/lyrics/types.ts`: TypeScript contracts for LyricLine, ParsedLyrics, and LyricsResponse.
- `island/src/hime/lyrics/lrcParser.ts`: TypeScript LRC parsing utility and multi-tag expander.
- `island/src/hime/lyrics/useLyrics.ts`: Hook for monotonic sync, $O(\log N)$ binary search, one-shot timer, and NCM sync.
- `island/src/hime/lyrics/LyricsView.tsx`: Expanded media player lyric presentation and interactive offset controls.
- `island/src/hime/lyrics/index.ts`: Public module exports for Hime Lyrics.
- `island/scripts/lyrics.test.ts`: Bun test suite testing LRC parsing, sync engine, timer invariants, and NCM preemption.
- `island/src/hime/tasks/types.ts`: TypeScript contracts for TaskItem, TaskPriority, TaskCategory, and TaskTab.
- `island/src/hime/tasks/useTasks.ts`: Hook for task state, urgency calculations, CRUD operations, and Tauri commands.
- `island/src/hime/tasks/TaskCard.tsx`: Task item card with category pill, due date badge, complete toggle, and delete action.
- `island/src/hime/tasks/QuickAddBar.tsx`: Input bar with debounced natural language parsing and instant task creation.
- `island/src/hime/tasks/HimeTasksPanel.tsx`: Full Tasks panel with header progress ring, segmented tabs, list stream, and close/chat affordances.
- `island/src/hime/tasks/index.ts`: Public module exports for Hime Tasks.
- `island/scripts/tasks.test.ts`: Bun test suite testing task types, urgency calculations, date formatting, and component export.
