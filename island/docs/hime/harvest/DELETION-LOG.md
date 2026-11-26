# Deletion Log — Legacy Hime Repository Cleanup

> Execution Date: 2026-11-16  
> Authority: Owner directive (05-HARVEST-AND-CLEANUP-MANIFEST.md §5.1)  
> Scope: Deletion of failed desktop UI, obsolete configurations, and compiler build artifacts without backup.

---

## 1. Pre-Deletion Verification Checklist (§5.1)

- [x] **`core/` builds and all tests pass**: Verified via `cargo check --locked` (0.22s) and `cargo test --locked` (131/131 tests passed).
- [x] **Architecture and governance documentation preserved**: 12 ADRs, 33 FRs, 18 NFRs, security baseline, hardware baseline, RE Nusa/Hermes/9Router analysis verified in `X:\folder_ai\Hime-Bloom\docs\`.
- [x] **`OLD-UI-LESSONS.md` completed**: Extracted approval lifecycle (TTL 60s, one-time ticket, anti-replay, fail-closed), activity event model (dedup ID), and pure TypeScript DTOs in `types-draft.md`.
- [x] **Manifest and hash verification recorded**: `HARVEST-MANIFEST.md` and `HASH-VERIFICATION.csv` (97/97 crate files with identical SHA-256).
- [x] **Secret scan clean**: 0 real credentials detected in harvested core files (`scripts/secret-scan.ps1` exit code 0).
- [x] **Phase 08–15 work accounted for**: All backend logic preserved in `core/crates/`; mobile app preserved in `mobile/`.

---

## 2. Purge Targets & Execution

| Path | Category | File Count | Size (MB) | Reason / Justification |
|:---|:---:|:---:|:---:|:---|
| `X:\folder_ai\Hime\target\` | **DELETE** | 48,565 | ~22,465 MB | Obsolete Rust target build cache and compiler intermediate artifacts |
| `X:\folder_ai\Hime\apps\hime-desktop\` | **REFERENCE-THEN-DELETE** | 2,693 | ~66.05 MB | Failed desktop UI shell (standard OS window frame, AI slop widgets, pnpm/node_modules) |
| `X:\folder_ai\Hime\config\` | **DELETE** | 1 | < 0.01 MB | Old local dev configs per Owner directive |
| `X:\folder_ai\Hime\hime-bloom-pack.zip` | **DELETE** | 1 | 0.12 MB | Ephemeral archive unpacked to `docs/agent-pack/` |

---

## 3. Post-Purge Verification

Post-purge disk verification confirms ~22.5 GB freed. No files outside `X:\folder_ai\Hime` were touched.
`X:\folder_ai\Hime-Bloom` serves as the sole, authoritative source of truth for the project.
