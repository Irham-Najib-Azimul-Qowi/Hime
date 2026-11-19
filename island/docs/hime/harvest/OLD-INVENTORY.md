# Inventory of Legacy Hime Repository (`X:\folder_ai\Hime`)

> Date: 2026-11-16  
> Host: Windows 11 Pro x64  
> Stage: STAGE H1 (Harvest & Restructure)

---

## 1. Git Repository State

- **Root Location**: `X:\folder_ai\Hime`
- **Active Branch**: `phase/15-hardening-release-linux`
- **Local Branches**:
  - `main`
  - `phase/08-ai-runtime`
  - `phase/09-intelligence-core`
  - `phase/10-storage-intelligence`
  - `phase/11-mobile-device-layer`
  - `phase/12-productivity-documents`
  - `phase/13-browser-dev-workspace`
  - `phase/14-specialist-agents-skills`
  - `phase/15-hardening-release-linux` (checked out)
- **Latest Commit**: `4a410b9` — `feat(island): implement true nusa-style floating dynamic island with top-docking, win-d immunity, and zero desktop window chrome`
- **Preceding Commit**: `4fc34cc` — `chore(scripts): add one-click run-hime.bat launcher`
- **Working Tree**: Clean tracking; untracked zip/pack: `hime-bloom-pack.zip`, `hime-bloom-pack/`

---

## 2. Directory Hierarchy & Storage Footprint

| Directory / Target | Files Count | Size (Bytes) | Size (MB) | Planned Action |
|:---|:---:|:---:|:---:|:---|
| `target/` | 48,565 | 23,556,625,424 | ~22,465 MB | **DELETE** (Build cache / compiler artifacts) |
| `apps/hime-desktop/` | 2,693 | 69,255,209 | ~66.05 MB | **REFERENCE-THEN-DELETE** (Failed UI) |
| `apps/mobile/` | 23 | 58,535 | ~0.06 MB | **KEEP-MOVE** -> `mobile/` |
| `crates/` | 97 | 784,283 | ~0.75 MB | **KEEP-MOVE** -> `core/crates/` |
| `docs/` | 89 | 462,479 | ~0.44 MB | **KEEP-MOVE / ADAPT** -> `docs/` |
| `project-management/` | 133 | 1,913,503 | ~1.82 MB | **KEEP-MOVE / ADAPT** -> `docs/history/` & `docs/` |
| `RE/` | 24,392 | 265,687,087 | ~253.38 MB | **KEEP-MOVE** (Nusa docs -> `docs/reference/nusa/`, audits -> `docs/reference/`) |
| `scripts/` | 7 | 41,987 | ~0.04 MB | **ADAPT** -> `scripts/` |
| `proto/` | 1 | 2,915 | < 0.01 MB | **KEEP-MOVE** -> `docs/architecture/proto/` |
| `.agent/` | 16 | 238,158 | ~0.23 MB | **KEEP-MOVE** (Skills) |
| Root configs & docs | 10 | ~260,000 | ~0.25 MB | **ADAPT** |

---

## 3. Rust Backend Crates Verification (Pre-Harvest Baseline)

Ran `cargo test --workspace --exclude hime-desktop` in `X:\folder_ai\Hime`:
- **Result**: `129 passed, 0 failed, 0 ignored` in total.

Breakdown per crate:
1. **`hime-core`**:
   - `unittests (src/lib.rs)`: **22 passed** (AI resources, task engine, event engine, memory, translation, browser policy, secret scanner, supervisor, tool gateway).
   - `ai_integration`: **5 passed**
   - `browser_integration`: **3 passed**
   - `developer_integration`: **4 passed**
   - `device_integration`: **7 passed**
   - `document_integration`: **5 passed**
   - `foundation_integration`: **7 passed**
   - `integrations_integration`: **3 passed**
   - `intelligence_integration`: **8 passed**
   - `productivity_integration`: **5 passed**
   - `specialist_integration`: **6 passed**
   - `storage_integration`: **10 passed**
   - *Subtotal `hime-core`*: **85 passed**
2. **`hime-ipc`**:
   - `unittests (src/lib.rs)`: **15 passed** (Framing, 12-byte header, JSON-RPC 2.0, token+nonce, memory duplex, unix socket/named pipe)
3. **`hime-pal`**:
   - `unittests (src/lib.rs)`: **5 passed** (Paths, Job Object supervisor, selection reader)
4. **`hime-security`**:
   - `unittests (src/lib.rs)`: **9 passed** (DPAPI vault, CSPRNG token, path traversal defense, redaction)
5. **`hime-storage`**:
   - `unittests (src/lib.rs)`: **15 passed** (SQLite WAL, audit hash chains, credentials, memory items, tasks, backups, journals)

---

## 4. Mobile Component Inventory

- Target: `apps/mobile/`
- Structure: Android Kotlin Gradle project (Jetpack Compose, mTLS client, remote approval flow, chunked file transfer).
- Planned Destination: `X:\folder_ai\Hime-Bloom\mobile/`.

---

## 5. UI Inventory (Failed Desktop Shell)

- Location: `apps/hime-desktop/`
- Reason for rejection: Rendered as standard OS window frame with red/black cards and AI slop widgets, rather than a genuine Dynamic Island floating pill.
- Action:
  - Extract architectural lessons (approval token flow, IPC DTO, Activity stream event IDs) -> `docs/hime/OLD-UI-LESSONS.md`.
  - Extract TypeScript interfaces -> `docs/hime/types-draft.md`.
  - Completely purge `apps/hime-desktop` and dependencies without backup per Owner decision.
