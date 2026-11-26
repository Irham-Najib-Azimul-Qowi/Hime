# Hime Documentation Index

Navigation index for all specifications, architectural decision records, legal compliance documents, and agent briefing packs in the Hime ecosystem.

---

## 1. Top-Level Project Tracking
- [`PROJECT.md`](../PROJECT.md): Scope, interface contracts, and milestone tracking.
- [`PROJECT_STATUS.md`](../PROJECT_STATUS.md): Milestone progress and active phase status.
- [`BACKLOG.md`](../BACKLOG.md): Monorepo task backlog and planned features.
- [`RISK_REGISTER.md`](../RISK_REGISTER.md): Ecosystem risk register and mitigations.
- [`CHANGELOG.md`](../CHANGELOG.md): Product version changelog.
- [`AGENTS.md`](../AGENTS.md): Permanent agent engineering instructions and standards.

## 2. Architecture & Decision Records (`docs/adr/`)
- [`004-intelligence-core-and-tool-gateway.md`](adr/004-intelligence-core-and-tool-gateway.md)
- [`005-storage-intelligence-and-safe-file-operations.md`](adr/005-storage-intelligence-and-safe-file-operations.md)
- [`006-mobile-pairing-mtls-and-device-architecture.md`](adr/006-mobile-pairing-mtls-and-device-architecture.md)
- [`007-document-engine-and-accountability-architecture.md`](adr/007-document-engine-and-accountability-architecture.md)
- [`008-browser-service-and-developer-workspace-architecture.md`](adr/008-browser-service-and-developer-workspace-architecture.md)
- [`009-specialist-agents-and-proactive-assistance-architecture.md`](adr/009-specialist-agents-and-proactive-assistance-architecture.md)
- [`010-v1-release-hardening-and-cross-platform-architecture.md`](adr/010-v1-release-hardening-and-cross-platform-architecture.md)
- [`013-shell-hime-as-bloom-fork-core-independent.md`](adr/013-shell-hime-as-bloom-fork-core-independent.md): Desktop shell as additive Bloom fork.
- [`014-additive-integration-and-touchpoint-registry.md`](adr/014-additive-integration-and-touchpoint-registry.md): Additive touchpoint discipline.
- [`015-neutralize-upstream-updaters-and-announcements.md`](adr/015-neutralize-upstream-updaters-and-announcements.md): Neutralize upstream updaters and announcements.
- [`016-dual-rust-workspace-architecture.md`](adr/016-dual-rust-workspace-architecture.md): Dual Rust workspace strategy (`core` vs `island/src-tauri`).

## 3. Legal & Open Source Compliance (`docs/legal/`)
- [`GPL-NOTES.md`](legal/GPL-NOTES.md): GPL-3.0 compliance and process boundary isolation.
- [`THIRD-PARTY.md`](legal/THIRD-PARTY.md): Third-party software inventory and license registry.
- [`MODIFICATIONS.md`](legal/MODIFICATIONS.md): GPL-3.0 §5 modification log detailing changes against Bloom v3.10.2.
- [`docs/hime/UPSTREAM.md`](hime/UPSTREAM.md): Upstream Bloom repository and commit tracking.

## 4. Agent Briefing Packs (`docs/agent-pack/`)
- `update-v2/`: Current authoritative architecture, monorepo layout (02b), branding (03), design system (04), modularity (14), and lifecycle specifications.
- `bloom/`: Bloom shell analysis, touchpoint mappings, and integration notes.
- `legacy/`: Historical architecture documentation.
