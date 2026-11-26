# Hime Settings Registry (`SETTINGS.md`)

This document registers all configuration keys specific to the Hime subsystem within the Bloom shell (`X:\folder_ai\Hime-Bloom`).

---

## 1. Subsystem Settings Keys

| Key                         | Type      | Default             | Scope                   | Description                                                                                     |
| :-------------------------- | :-------- | :------------------ | :---------------------- | :---------------------------------------------------------------------------------------------- |
| `hime-enabled`              | `boolean` | `true`              | LocalStorage / Settings | Master toggle for Hime personal AI assistant and Dynamic Island integration.                    |
| `hime-ipc-endpoint`         | `string`  | `\\.\pipe\hime-ipc` | Rust Backend            | Win32 Named Pipe endpoint for communicating with standalone Hime Core PEP.                      |
| `hime-auto-start-core`      | `boolean` | `true`              | Rust Backend / Settings | When enabled, Hime Shell supervisor automatically checks and starts `hime-core.exe` if offline. |
| `hime-approval-sound`       | `boolean` | `true`              | LocalStorage / Settings | Plays synthesized chime via Web Audio API when a pending security PEP approval bubble arrives.  |
| `hime-notch-remember-state` | `boolean` | `true`              | LocalStorage            | Remembers whether the Hime Assistant panel was the active Island view across restarts.          |

---

## 2. Security Invariants for Settings

1. **Zero Secret Storage**: No API keys, tokens, session nonces, or passwords are EVER written to settings keys or `localStorage`. Secrets are exclusively sealed in the Hime Core DPAPI vault.
2. **Offline Fallback**: If `hime-enabled` is set to `false`, the Shell completely disconnects from IPC and suppresses the Hime indicator dot, bubble, and Island panel.
3. **Atomic Persistence**: Setting changes flow through Bloom's atomic settings cache write pipeline (`utils::save_setting`), preventing partial corruption on sudden shutdown.
