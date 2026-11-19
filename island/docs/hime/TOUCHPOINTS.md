# Bloom Touchpoint Registry

This registry tracks **every single line** modified or added within existing Bloom files.
Rule: Kode Hime bersifat aditif dan terisolasi (`src/hime/**` dan `src-tauri/src/hime/**`).
Setiap sentuhan pada berkas asli Bloom wajib didokumentasikan di sini dengan alasan dan prosedur revert.

---

## Registry of Touchpoints

### TP-001: Neutralize Auto-Updater & Update Endpoints

- **File**: `src-tauri/tauri.conf.json`
- **Lines**: 99-101, 125-131
- **Component**: Security CSP & Tauri Plugins
- **Reason**: [G14 / D9 / Task H0.5] Neutralize upstream updater so Bloom does not auto-download upstream releases and overwrite the Hime fork. Removed `https://bloom.sehaz.space` from CSP `connect-src`.
- **Change**: Set `plugins.updater.active = false`, emptied `endpoints` and `pubkey`.
- **Revert Procedure**: Restore `"active": true`, restore original pubkey and GitHub endpoint.

### TP-002: Neutralize Rust Updater Logic

- **File**: `src-tauri/src/updater.rs`
- **Lines**: 136-141, 239-242
- **Component**: Rust Backend (`updater` module)
- **Reason**: [G14 / D9 / Task H0.5] Prevent background update checks (`run_startup_check`) and manual check (`check`) from triggering network requests or polling GitHub.
- **Change**: `check` returns `Ok(UpdateCheckResult::default())` immediately; `run_startup_check` is an empty no-op.
- **Revert Procedure**: Restore original check and rollout age verification logic.

### TP-003: Neutralize In-App Announcements

- **File**: `src/hooks/useAnnouncement.ts`
- **Lines**: 38-46
- **Component**: Frontend Hook (`useAnnouncement`)
- **Reason**: [G14 / D9 / Task H0.5] Prevent shell from calling external endpoint `https://bloom.sehaz.space/announcements.json`.
- **Change**: `fetchAnnouncement` resolves to `null` without network calls; `readCached` returns `null`.
- **Revert Procedure**: Restore original fetch controller and localStorage reader.

### TP-004: Safe Minimal Rebrand (Product Name, Identifier & Windows Mutex)

- **File**: `src-tauri/tauri.conf.json`, `src-tauri/src/main.rs`, `package.json`
- **Lines**: `tauri.conf.json:3,5,16`, `main.rs:64,68,410,412,419`, `package.json:2`
- **Component**: App Identity & Windows Process Mutex
- **Reason**: [Task H2.1] Prevent conflict with native Bloom installations (`com.sehaz.bloom` and `BloomSingleInstance` mutex/event). Set product name to "hime", identifier to "ai.hime.desktop", mutex to "HimeSingleInstance", event to "HimeOpenSettings", tray tooltip/menu to "Hime".
- **Change**: Rebranded app identifier, single-instance mutex, and tray labels to Hime.
- **Revert Procedure**: Revert strings back to "bloom" / "com.sehaz.bloom".

### TP-005: Rust Hime Bridge Subsystem Registration

- **File**: `src-tauri/src/main.rs`
- **Lines**: 118-125, 461-468, 482
- **Component**: Rust Application Entrypoint (`main.rs`)
- **Reason**: [Task H2.2] Register `pub mod hime;`, expose Hime IPC commands (`hime_core_status`, `hime_ping`, `hime_approval_list`, `hime_approval_resolve`, `hime_dev_trigger_approval`) in Tauri `generate_handler!`, and initialize background core supervisor via `hime::init(app.handle())`.
- **Change**: Added module declaration, command list, and initialization hook.
- **Revert Procedure**: Remove module import, commands, and `hime::init` call.

### TP-006: Settings Hime Tab Registration

- **File**: `src/settings/types.ts`, `src/Settings.tsx`
- **Lines**: `types.ts:1-2`, `Settings.tsx:7,21,22,33,43,52,342-344`
- **Component**: Bloom Settings Window
- **Reason**: [Task H2.7] Expose Hime diagnostics, Core process status, ping tool, approval simulator, and master toggle inside native Bloom Settings tabs.
- **Change**: Added `"hime"` to `SettingsTab` union, added Lucide `Bot` icon, copy text, `useHimeEvents` hook call, and conditional rendering of `<HimeSettingsTab />`.
- **Revert Procedure**: Remove `"hime"` from `SettingsTab`, delete tab definition and render block.

### TP-007: Island Slice & Live Activity Bubble Integration

- **File**: `src/App.tsx`
- **Lines**: ~75, ~91, ~455, ~1420, ~1960, ~2207, ~3120, ~3150, ~3815, ~4610, ~4725
- **Component**: Bloom Dynamic Island Shell (`App.tsx`)
- **Reason**: [Task H2.4 & H2.6] Seamlessly morph the Dynamic Island into Hime Assistant panel and Approval detail card without desktop window chrome; render `<ApprovalBubble>` beside collapsed pill when approvals are pending; add subtle status dot in collapsed pill.
- **Change**: Added `"hime"` to `NotchPanel`, `"hime" | "hime-approval"` to `BloomMode`, dynamic dimensions (380×300 and 380×160), panel switch handling, status dot indicator, and live activity bubble.
- **Revert Procedure**: Revert `NotchPanel`, `BloomMode`, dimensions, and render blocks.
