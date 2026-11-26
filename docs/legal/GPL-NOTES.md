# GPL-3.0 Compliance Notes & Architectural Boundary

## 1. Upstream License
- **Project**: Bloom (`https://github.com/little-systems/bloom.git`)
- **License**: GNU General Public License v3.0 (`GPL-3.0-only`)
- **Copyright**: Sehajveer Singh and Bloom contributors.

## 2. Shell Fork Classification
Hime Shell is built on top of Bloom (a direct fork of Bloom tagged at `v3.10.2`).
Under the terms of GPL-3.0:
- All modifications within the Shell codebase (`src/**`, `src-tauri/**`) are licensed under GPL-3.0.
- All Hime-specific additions within the shell (`src/hime/**`, `src-tauri/src/hime/**`) are additive and compatible with GPL-3.0.
- Source code of Hime Shell must remain accessible under GPL-3.0 to recipients of binaries.

## 3. Boundary with Hime Core (Separate Process)
As established in ADR D1, ADR D2, and ADR-013:
- **Hime Core** is an independent, decoupled backend process (`hime-core.exe`).
- **Communication Transport**: Standard Win32 Local Named Pipe (`\\.\pipe\hime-ipc`) using JSON-RPC messages over serialized streams.
- **Process Boundary**: The Shell and Core do not link at compile-time, do not share address space, and do not include GPL headers in the Core codebase.
- **Independence Principle**: Per FSF GPL FAQ guidance regarding inter-process communication:
  > "If the modules are included in the same executable file, they are definitely combined in one program. If modules are designed to run in separate processes that communicate over an IPC mechanism (such as pipes, sockets, or command-line arguments), they are almost always separate programs."
- Therefore, Hime Core, Hermes Agent, and 9Router retain their respective licenses and independent development boundaries.

## 4. Third-Party Upstream Dependencies
- Hermes Agent: NousResearch / Open-source (Apache-2.0 / MIT)
- 9Router: decolua / Open-source (MIT)
- Bloom: GPL-3.0

## 5. Distribution and Delivery Rules
1. Any distribution of Hime Shell binaries must accompany or provide access to the corresponding complete machine-readable source code of the shell.
2. No proprietary license restrictions or anti-reverse-engineering clauses may be imposed upon the Shell.
3. Upstream push is explicitly disabled (`DISABLE-NO-PUSH`) to prevent accidental upstream pollutions.

