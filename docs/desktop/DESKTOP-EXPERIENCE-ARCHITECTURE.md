# Hime Desktop Experience — Architecture Specification

- **Document ID:** DOCS-DESKTOP-ARCHITECTURE
- **Phase:** Phase 07 — Hime Desktop Experience
- **Authority:** System Architect & Technical Lead
- **Status:** APPROVED IMPLEMENTATION ARCHITECTURE

---

## 1. System Topology & Layers

Hime Desktop is structured as a Tauri 2.0 dual-process application connecting directly to the Hime Core kernel daemon over native Windows IPC:

```text
┌────────────────────────────────────────────────────────────────────────┐
│                          HIME DESKTOP SHELL                            │
│                                                                        │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │                    React 18 Presentation UI                      │  │
│  │                                                                  │  │
│  │  ┌───────────────┐ ┌───────────────┐ ┌────────────────────────┐  │  │
│  │  │Dynamic Island │ │Omnibox Modal  │ │ Main Control Panel     │  │  │
│  │  │(Idle/Exp/Act) │ │(Command Disp) │ │ (6 Domain Views)       │  │  │
│  │  └───────────────┘ └───────────────┘ └────────────────────────┘  │  │
│  │                                                                  │  │
│  │  ┌────────────────────────────────────────────────────────────┐  │  │
│  │  │  Services Layer: ipcService.ts & commandRegistry.ts        │  │  │
│  │  └────────────────────────────────────────────────────────────┘  │  │
│  └─────────────────────────────────┬────────────────────────────────┘  │
│                                    │ @tauri-apps/api/core::invoke       │
│  ┌─────────────────────────────────▼────────────────────────────────┐  │
│  │                     Tauri 2 Rust Backend                         │  │
│  │  - Window Management (set_size, set_always_on_top, minimize)     │  │
│  │  - IPC Client Transport (connect_pipe_client)                    │  │
│  │  - 12-Byte Binary Framing & JSON-RPC 2.0 Deserializer            │  │
│  └─────────────────────────────────┬────────────────────────────────┘  │
└────────────────────────────────────┼───────────────────────────────────┘
                                     │ Windows Named Pipe: \\.\pipe\hime-ipc
                                     ▼
┌────────────────────────────────────────────────────────────────────────┐
│                           HIME CORE KERNEL                             │
│  - CoreServer Listener & JSON-RPC Dispatcher                           │
│  - StoragePort (SQLite WAL + audit_events)                             │
│  - PolicyPort (PEP/PDP security boundary)                              │
│  - JobSupervisor (Win32 Job Object Operational Orphan Prevention)       │
└────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Directory Layout & Module Ownership

```text
apps/hime-desktop/
├── package.json               # Frontend dependencies (React 18, Vite, TypeScript)
├── vite.config.ts             # Vite bundler configuration
├── tsconfig.json              # TypeScript strict configuration
├── index.html                 # Single page application entrypoint
├── src/
│   ├── main.tsx               # Application bootstrap
│   ├── App.tsx                # Main application shell with sidebar & layout
│   ├── index.css              # Global styles & drag regions
│   ├── styles/
│   │   ├── tokens.css         # Visual design tokens & dark/light palettes
│   │   └── animations.css     # Purposeful motion & keyframes
│   ├── types/
│   │   ├── ipc.ts             # Core telemetry & ping types
│   │   ├── events.ts          # SQLite audit event structures
│   │   ├── commands.ts        # Omnibox registry interfaces
│   │   ├── tasks.ts           # Task board and priority types
│   │   ├── notifications.ts   # Notification center contracts
│   │   └── theme.ts           # Presentation & view state definitions
│   ├── services/
│   │   ├── ipcService.ts      # Typed IPC client invoking Tauri commands
│   │   └── commandRegistry.ts # Universal Omnibox command registry
│   ├── components/ui/
│   │   ├── Button.tsx         # Accessible pressable button
│   │   ├── Badge.tsx          # Status and priority badge
│   │   ├── Card.tsx           # Surface container
│   │   ├── Input.tsx          # Text input with icon support
│   │   ├── Modal.tsx          # Accessible dialog with escape handling
│   │   └── Switch.tsx         # Accessible toggle switch
│   └── features/
│       ├── island/            # Dynamic Island (Idle, Expanded, Activity)
│       ├── omnibox/           # Universal command search dialog
│       ├── overview/          # System telemetry & architecture blockers
│       ├── conversation/      # Honest AI chat interface (Phase 08 boundary)
│       ├── activity/          # Live SQLite audit event stream
│       ├── tasks/             # Supervised tasks management board
│       ├── notifications/     # Interactive approvals & notification queue
│       └── settings/          # Themes, window geometry & toolchain info
└── src-tauri/
    ├── Cargo.toml             # Desktop Rust crate manifest
    ├── tauri.conf.json        # Window properties & security configuration
    ├── build.rs               # Tauri build hooks
    ├── src/
    │   ├── lib.rs             # Pure IPC command logic (ping, status, tasks, events)
    │   └── main.rs            # Tauri application entrypoint & window commands
    └── tests/
        └── desktop_integration.rs # Real desktop IPC integration tests
```

---

## 3. Resilience & Offline Handling

When Hime Core is offline:
1. `ipcService.ts` catches connection failures gracefully.
2. The UI renders an active offline banner warning the user that `\\.\pipe\hime-ipc` is not responding.
3. The Dynamic Island status dot switches to red with an `OFFLINE` badge.
4. No white screen, freeze, or unhandled promise rejection occurs.
5. The UI automatically retries polling every 4 seconds, restoring `ONLINE` status immediately when `hime-core.exe` starts.

