# ADR 010: v1.0 Release Hardening, Cross-Platform PAL, and Packaging Architecture

## Status
Accepted

## Date
2026-11-14

## Context
Phase 15 represents the culmination of the Hime development lifecycle to reach v1.0 Release Candidate status.

Key architectural challenges for release readiness include:
1. **Windows Installer & Release Packaging**: Hime requires a zero-friction, per-user installer that does not demand administrative privilege elevation for everyday use, bundles the native shell and frontend, and supports reproducible builds.
2. **Cross-Platform Compatibility ("Cross-Platform by Design")**: While Windows is the primary platform, Hime's core architecture must run cleanly on Linux without Windows-specific coupling in shared business logic.
3. **Secret Storage across Operating Systems**: Windows utilizes DPAPI (`CryptProtectData`), but Linux requires an equivalent secure store without leaving plaintext keys on disk.
4. **Child Process Supervision**: Windows leverages kernel Job Objects (`KILL_ON_JOB_CLOSE`), while Linux systems require signal-based process group management (`setpgid`, `SIGTERM`/`SIGKILL`).
5. **IPC Socket Transport**: Windows Named Pipes (`\\.\pipe\hime-*`) must seamlessly map to Unix Domain Sockets (`$XDG_RUNTIME_DIR/hime/*.sock` or `/tmp/*.sock`) with strict `0600` permissions.
6. **Performance & Memory Boundaries**: Under consumer laptop constraints (quad-core Intel i5, 8–16 GB RAM), the entire Hime runtime (Core + Desktop) must remain strictly within a 150 MB combined RAM envelope.

## Decisions

1. **Platform Abstraction Layer (`hime-pal`) as Sole Operating System Boundary**:
   - All filesystem path resolution (`PlatformPaths::app_data_dir()`, `cache_dir()`, `data_dir()`, `runtime_dir()`) respects Windows `%APPDATA%`/`%LOCALAPPDATA%` on Windows, and XDG Base Directory Specifications (`$XDG_CONFIG_HOME`, `$XDG_CACHE_HOME`, `$XDG_DATA_HOME`, `$XDG_RUNTIME_DIR`) on Linux.
   - `PlatformPaths::default_ipc_endpoint()` abstracts Windows Named Pipes vs Unix Domain Sockets.

2. **Dual-Tier Secret Vault Strategy**:
   - `WindowsDpapiVault` remains the primary hardware-tied credential vault on Windows.
   - `SoftwareEncryptedVault` (with SHA-256 CTR keystream and automatic zeroization on drop) is provided as the native cross-platform fallback for Linux, storing secrets with user-restricted filesystem permissions.
   - `create_default_vault()` dynamically instantiates the correct vault at runtime.

3. **Cross-Platform Process Supervisor**:
   - `JobSupervisor` maintains Win32 Job Object encapsulation on Windows with `CREATE_SUSPENDED` and fail-closed termination.
   - On Linux/Unix, `JobSupervisor` tracks child process PIDs and issues coordinated `SIGTERM`/`SIGKILL` signals across process lifecycles.

4. **Tauri 2.0 Packaging & Bundle Configuration**:
   - Updated `tauri.conf.json` with version `1.0.0`, high-resolution icons, and multi-target bundle definitions (`nsis`, `msi`, `appimage`, `deb`).
   - Per-user installation mode eliminates the need for administrator rights on consumer Windows machines.

5. **Release Build Optimization**:
   - Rust crates compile under release profile with Link-Time Optimization (`lto = true` or `thin`) and stripped debug symbols for minimal binary footprint.

## Consequences

- The Hime desktop environment is fully packaged for production distribution.
- Linux support is established as a first-class citizen at the PAL layer with verified abstraction boundaries.
- Memory and CPU consumption are predictable and rigorously bounded.
