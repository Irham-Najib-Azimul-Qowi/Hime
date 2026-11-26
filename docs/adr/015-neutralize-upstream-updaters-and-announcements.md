# ADR-015: Neutralization of Upstream Updaters and In-App Announcements

**Status:** Accepted  
**Date:** 2026-11-15  
**Authors:** Principal Engineer + Technical Lead  
**Deciders:** Project Owner, Hime Engineering Team  

---

## 1. Context
Bloom incorporates an automated background update mechanism (`tauri-plugin-updater` with periodic checks every 24 hours and at startup) configured to fetch releases from `https://github.com/SehajveerSingh2005/bloom/releases/latest/download/latest.json` and auto-install them. Additionally, Bloom polls `https://bloom.sehaz.space/announcements.json` for in-app broadcast announcements.

In a custom downstream fork such as Hime:
1. If the updater remains active, Bloom would detect upstream releases, download upstream Bloom binaries, and overwrite the Hime fork, destroying all integrated Hime features.
2. Polling external announcement endpoints introduces unwanted external network calls and UI popups.

---

## 2. Decision
1. **Disable Auto-Updater:**
   - In `src-tauri/tauri.conf.json`: set `plugins.updater.active = false` and clear endpoints.
   - In `src-tauri/src/updater.rs`: replace `check` with a static no-op returning `available: false`, and replace `run_startup_check` with an empty no-op.
2. **Disable In-App Announcements:**
   - In `src/hooks/useAnnouncement.ts`: neutralize `fetchAnnouncement` to resolve `null` without network calls.
   - In `src-tauri/tauri.conf.json`: remove `https://bloom.sehaz.space` from CSP `connect-src`.
3. **Register Touchpoints:**
   - Documented in `docs/hime/TOUCHPOINTS.md` under TP-001, TP-002, and TP-003.

---

## 3. Alternatives Considered
- **Alternative A: Point Updater to Hime Release Server immediately:**
  - *Deferred:* Local development and initial stages (H0–H2, Phases 08R–14R) do not operate a public release channel. Can be revisited prior to Phase 15R production distribution.
- **Alternative B: Completely remove updater module from Rust codebase:**
  - *Rejected:* Removing the module breaks IPC signatures (`check_for_updates`, `install_update`) expected by `Settings.tsx` and requires unnecessary touchpoints. Neutralizing the functions is cleaner and safer.

---

## 4. Consequences
- **Positive:** Guarantees Hime fork stability; prevents unwanted external network traffic; eliminates risk of upstream binary overwrite.
- **Negative:** None.

