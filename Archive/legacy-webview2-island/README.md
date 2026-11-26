# Legacy WebView2 + Tauri + React Island Shell (Archived)

This directory contains the legacy WebView2 / Tauri 2.0 / React desktop shell
(`hime-island.exe` + `hime_hotkey.exe` + Bun/Vite frontend) archived per
**ADR-017: Modular Two-Subsystem Native Rust Architecture (`Hime Island` + `Hime AI`) — Phase 4**.

## Active Pure-Rust Native Replacement

- **Subsystem 1 (`hime-island-native.exe`)**: `island/native/` (Pure-Rust Win32 + Direct2D + DirectWrite + DirectComposition, with built-in global hotkeys and zero WebView2).
- **Subsystem 2 (`hime-core.exe`)**: `core/` (Rust workspace: `hime-core`, `hime-ipc`, `hime-modsdk`, `hime-pal`, `hime-security`, `hime-storage`).

These archived files are kept strictly for historical/UI parity reference and are not built or required by the active desktop runtime or verification gates.
