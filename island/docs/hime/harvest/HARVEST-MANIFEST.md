# Harvest & Migration Manifest

> Document: `HARVEST-MANIFEST.md`  
> Author: Principal Engineer / Tech Lead  
> Date: 2026-11-16  
> Source Root: `X:\folder_ai\Hime`  
> Destination Root: `X:\folder_ai\Hime-Bloom`

---

## 1. Classification & Routing Matrix

| Source Item (`X:\folder_ai\Hime`) | Class | Destination Target (`X:\folder_ai\Hime-Bloom`) | Notes |
|:---|:---:|:---|:---|
| `crates/hime-core/` | **KEEP-MOVE** | `core/crates/hime-core/` | All source modules + 11 integration test files (85 tests) |
| `crates/hime-ipc/` | **KEEP-MOVE** | `core/crates/hime-ipc/` | Framing (12-byte), JSON-RPC 2.0, Named Pipe, memory duplex (15 tests) |
| `crates/hime-pal/` | **KEEP-MOVE** | `core/crates/hime-pal/` | OS abstraction layer, Windows Job Object, Named Pipe endpoints (5 tests) |
| `crates/hime-security/` | **KEEP-MOVE** | `core/crates/hime-security/` | DPAPI vault, CSPRNG tokens, path traversal defense, redaction (9 tests) |
| `crates/hime-storage/` | **KEEP-MOVE** | `core/crates/hime-storage/` | SQLite WAL, audit hash chains, storage indexing, rollback (15 tests) |
| `Cargo.toml` | **ADAPT** | `core/Cargo.toml` | Remove `apps/hime-desktop/src-tauri` from members list |
| `Cargo.lock` | **ADAPT** | `core/Cargo.lock` | Updated for standalone core Cargo workspace |
| `apps/mobile/` | **KEEP-MOVE** | `mobile/` | Android Jetpack Compose app, mTLS pairing, remote approval |
| `docs/architecture/` | **KEEP-MOVE** | `docs/architecture/` | Core system architecture and data flow specs |
| `docs/adr/` | **KEEP-MOVE** | `docs/adr/` | Preserve legacy ADR 001 - 012 alongside new Bloom ADRs (013+) |
| `docs/security/` | **KEEP-MOVE** | `docs/security/` | Security baseline, threat model, IB-01/IB-02 documentation |
| `docs/environment/` | **KEEP-MOVE** | `docs/environment/` | Hardware baseline and OS specs |
| `project-management/product-requirements/` | **KEEP-MOVE** | `docs/product/requirements/` | 33 FR, 18 NFR, PRD specifications |
| `project-management/phases/` | **KEEP-MOVE** | `docs/history/phases/` | Historical phase completion reports |
| `RE/Nusa/` | **KEEP-MOVE** | `docs/reference/nusa/` | Nusa reverse engineering architectural findings (clean documentation only) |
| `RE/Hermes/` | **KEEP-MOVE** | `docs/reference/hermes/` | Nous Hermes Agent integration specs |
| `RE/9ROUTER/` | **KEEP-MOVE** | `docs/reference/9router/` | 9Router AI Gateway integration specs |
| `hime-bloom-pack/` | **KEEP-MOVE** | `docs/agent-pack/` | Full Hime×Bloom directive and roadmap pack |
| `.agent/skills/` | **KEEP-MOVE** | `.agent/skills/` | Agent skills library |
| `apps/hime-desktop/src/` | **REFERENCE-THEN-DELETE** | `docs/hime/OLD-UI-LESSONS.md` & `docs/hime/types-draft.md` | Extract behavior and TS DTO types, then delete code |
| `apps/hime-desktop/` | **DELETE** | *N/A (Purge without backup)* | Old failed custom Tauri UI, node_modules, dist |
| `target/` | **DELETE** | *N/A (Purge without backup)* | 22.4 GB legacy build target cache |
| `scripts/` | **ADAPT** | `scripts/` | Adapt `build-core.ps1`, `dev-hime.ps1`, `check-env.ps1` |

---

## 2. Integrity Verification Protocol

1. Target directories are created in `X:\folder_ai\Hime-Bloom`.
2. Files are copied cleanly from source to target.
3. SHA-256 checksums are calculated and verified between source and destination files.
4. Core workspace is built and verified via `cargo check --locked` and `cargo test --locked` in `core/`.
5. Secret scanner verifies no plaintext tokens or credentials were transferred.
