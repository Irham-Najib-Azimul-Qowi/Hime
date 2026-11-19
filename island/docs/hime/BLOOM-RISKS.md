# Bloom Risk Assessment & Outbound Network Audit

## 1. Outbound Network & Telemetry Audit (Task H0.5 / G14 / D9)

A comprehensive codebase audit was conducted across all frontend (`src/**`) and backend (`src-tauri/**`) files.

### 1.1 Outbound Network Endpoints Found in Upstream Bloom

| Location | Endpoint / Domain | Protocol | Purpose in Bloom | Risk to Hime | Mitigation in Hime | Status |
|---|---|---|---|---|---|---|
| `tauri.conf.json`, `updater.rs` | `https://github.com/SehajveerSingh2005/bloom/releases/...` | HTTPS | Auto-updater check every 24h & at startup | **CRITICAL**: Would overwrite Hime Shell with upstream Bloom binary | Neutralized in `tauri.conf.json` and `updater.rs` (TP-001, TP-002) | **RESOLVED** |
| `useAnnouncement.ts` | `https://bloom.sehaz.space/announcements.json` | HTTPS | In-app announcement cards | **MEDIUM**: Fetches third-party JSON on startup | Neutralized in `useAnnouncement.ts` (TP-003); removed from CSP | **RESOLVED** |
| `useWeather.ts` | `https://api.open-meteo.com` | HTTPS | Weather data for Notch | **LOW**: Anonymous public weather API | Allowed in Bloom CSP; does not leak user data | **ACCEPTED (Bloom)** |
| `useWeather.ts` | `https://ipapi.co/json/`, `http://ip-api.com/json/...` | HTTP/S | Fallback IP geolocation for weather | **LOW**: Used only if weather is enabled | Allowed in Bloom CSP | **ACCEPTED (Bloom)** |

### 1.2 Telemetry & Analytics Audit
- **Telemetry search**: `telemetry`, `analytics`, `sentry`, `posthog`, `google-analytics`, `mixpanel` were scanned across all files.
- **Finding**: **0 instances found**. Upstream Bloom has **zero** telemetry or user analytics built-in.
- **Conclusion**: Aside from updater, announcements, and weather, Bloom does not transmit data across the network.

---

## 2. Desktop Environment & Shell Risks

### 2.1 Taskbar Replacement (AppBar & Shell_TrayWnd)
- **Risk**: Bloom calls `ShowWindow(tray_hwnd, SW_HIDE)` and registers an AppBar via `SHAppBarMessage(ABM_NEW)`. If the shell crashes or hangs without executing its teardown handler, the Windows taskbar remains hidden, leaving the user with an empty desktop.
- **Mitigation**:
  - `scripts/restore-taskbar.ps1` created and verified (unhides `Shell_TrayWnd` and restarts Explorer if necessary).
  - `scripts/run-safe.ps1` watchdog runner wraps dev/test executions with a timeout and guaranteed `finally` restoration.
  - Manual emergency procedures documented in `docs/hime/RECOVERY.md`.

### 2.2 IPC & Process Isolation (Hime Core Decoupling)
- **Risk**: If Hime AI orchestration were executed directly inside the Shell process, a model hang or memory surge would crash the user's desktop shell (including notch and dock).
- **Mitigation**:
  - Hime Core remains an independent external process (`hime-core.exe`).
  - Shell connects via Win32 Named Pipe (`\\.\pipe\hime-ipc`) solely as a UI client.
  - If Core stops or is killed, Shell remains 100% operational as a Bloom desktop shell with clear "Core Offline" state indicator.

### 2.3 Licensing & Legal Risk (GPL-3.0 Copyleft)
- **Risk**: Bloom is licensed under GPL-3.0-only. Monolithic linkage could impact external backend components.
- **Mitigation**:
  - Shell is an explicit fork governed by GPL-3.0.
  - Core is a separate process over IPC (Named Pipe JSON-RPC), legally and architecturally separated per FSF GPL guidance (documented in `docs/legal/GPL-NOTES.md`).

