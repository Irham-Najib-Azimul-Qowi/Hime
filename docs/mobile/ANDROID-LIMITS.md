# Hime Android Technical Boundaries & Platform Reality

> **Status:** Authoritative Architectural Policy  
> **Phase:** Phase 11 — Mobile and Device Layer  
> **Classification:** Security & Compliance Standard

---

## 1. Core Principle: Zero-Root, Non-Intrusive Engineering

Hime Android Companion is engineered as a secure, privacy-first companion to Hime Desktop. It adheres strictly to official Android SDK APIs and Google Play platform policies.

- **NO Root Access:** Hime never prompts for, expects, or leverages root privileges.
- **NO Accessibility Abuse:** Hime does not use Android Accessibility Services to scrape user screens, intercept keystrokes, or automate third-party applications.
- **NO Remote Shell:** Hime provides no remote terminal or arbitrary command execution capability on Android devices.
- **Honest UI:** Any feature that cannot be reliably performed within official Android constraints is presented transparently to the user, not faked with fragile workarounds.

---

## 2. Storage & File System Access

### A. Storage Access Framework (SAF)
- Under Android 10+ (API 29+ Scoped Storage), applications cannot arbitrarily traverse `/sdcard` or other applications' private directories.
- Hime utilizes **SAF (`ACTION_OPEN_DOCUMENT_TREE`)** allowing the user to explicitly select and authorize specific directories (e.g. `Documents`, `Downloads`, or a custom workspace).
- Read, write, and index operations are strictly sandboxed to these user-authorized document trees.

### B. MediaStore API
- For camera photos, videos, and audio, Hime utilizes `MediaStore` queries with standard Android permissions (`READ_MEDIA_IMAGES`, `READ_MEDIA_VIDEO`).
- File metadata and SHA-256 hashes are indexed locally in SQLite/Room; **actual file bytes are never transferred to Desktop or AI providers without explicit user sync approval.**

### C. `MANAGE_EXTERNAL_STORAGE` Opt-In
- "All files access" (`MANAGE_EXTERNAL_STORAGE`) is strictly an optional, advanced opt-in setting with explicit justification in the Settings UI. It is never enabled by default.

---

## 3. Background Execution & OEM Battery Optimizations

### A. Foreground Service Requirement
- Android aggressively terminates background processes via **Doze Mode**, App Standby buckets, and aggressive OEM background task killers (e.g., Xiaomi MIUI/HyperOS, Samsung OneUI, OnePlus OxygenOS).
- Hime maintains a persistent connection using `FOREGROUND_SERVICE_DATA_SYNC` with an ongoing notification ("Hime Companion Active").
- Users are guided through OEM-specific "Ignore Battery Optimization" settings (`REQUEST_IGNORE_BATTERY_OPTIMIZATIONS`) if continuous background syncing is desired.

### B. WorkManager for Periodic Maintenance
- Non-urgent sync tasks, metadata refresh, and storage health audits run via Android `WorkManager` subject to charging and Wi-Fi network constraints.

---

## 4. Device Posture & Privacy (UR-12)

### A. Read-Only Posture Auditing
Hime queries device security posture exclusively through official platform APIs:
- **Screen Lock:** `KeyguardManager.isDeviceSecure()`
- **Hardware Encryption:** `DevicePolicyManager.getStorageEncryptionStatus()`
- **Security Patch:** `Build.VERSION.SECURITY_PATCH`
- **Sideloading Setting:** `Settings.Secure.getInt(..., INSTALL_NON_MARKET_APPS)`

### B. Advisory-Only Governance
- Hime is an **advisor and observer**, not an MDM (Mobile Device Management) agent.
- Hime **never modifies system security settings** or forces policy changes on the user's personal device.
- Posture reports are aggregated locally; sensitive posture data is never transmitted to cloud LLM providers without explicit user consent.

---

## 5. Screen Time & App Usage Nudges (UR-11)

- Screen time tracking utilizes `UsageStatsManager.queryUsageStats()`.
- Access requires user authorization via `Settings.ACTION_USAGE_ACCESS_SETTINGS`.
- Usage telemetry remains 100% on-device; only aggregate distraction counts are shared with Hime Core for schedule nudging according to the chosen strictness mode (*Relaxed*, *Balanced*, or *Strict*).

