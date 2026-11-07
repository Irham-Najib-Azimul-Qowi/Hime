# Architecture Decision Log
**Project:** Hime Assistant  
**Last Updated:** 2026-11-06 (Post TASK-00-02 Review Correction)

---

## Agreed Architectural Decisions (Baseline)

| Decision ID | Area | Decision Summary | Rationale & Evidence | Date | Status |
|:---:|:---|:---|:---|:---:|:---:|
| **DEC-01** | Desktop Framework | **Reimplement Nusa using Tauri 2.0 (Rust + Web Frontend)** | Nusa source code is closed/unavailable. Tauri 2.0 provides native Win32 window handles, DirectComposition GPU support, small bundle size (<15MB), and memory safety. | 2026-11-05 | **APPROVED** |
| **DEC-02** | AI Agent Core | **Integrate Nous Research Hermes Agent (Do Not Rebuild)** | Hermes Agent provides a proven 10-phase reasoning loop, dynamic tool registries, FTS5 memory, and Named Pipe IPC. Rebuilding from scratch is unnecessary duplication. | 2026-11-05 | **APPROVED** |
| **DEC-03** | Model Gateway | **Integrate decolua 9Router as Managed Background Service (Do Not Rebuild)** | 9Router solves 140+ provider adapters, format normalizations, multi-account exponential backoff, and model combos. Rebuilding is high risk and wasteful. | 2026-11-05 | **APPROVED** |
| **DEC-04** | Gateway Topology | **Dual-Control Plane Architecture (Topologi Opsi B)** | Hermes connects directly to `http://127.0.0.1:20128/v1` for low-latency inference. Hime Native Core connects to `/api/*` for discovery, quotas, and failover notifications. | 2026-11-05 | **APPROVED** |
| **DEC-05** | Agent Engineering | **Evidence-First & Minimal Implementation via AGENTS.md** | Mandate observable evidence for all claims. Adopt the 7-rung solution ladder (YAGNI, stdlib, existing patterns, minimal diffs). | 2026-11-06 | **APPROVED** |
| **DEC-06** | Skill Standard | **Standardize on Agent Skills (`SKILL.md`) Format** | Use canonical `SKILL.md` structure in `.agent/skills/`. Vetted skills: `animate`, `emil-design-eng`, `ponytail`, `rea`, `systematic-debugging`, `verification-before-completion`, `web-design-guidelines`. | 2026-11-06 | **APPROVED** |
| **DEC-07** | Network Binding | **Strict Localhost Loopback Isolation (`127.0.0.1`)** | Prevent network exposure of local daemons (9Router, Hermes, Tauri IPC) on untrusted LANs. | 2026-11-06 | **APPROVED** |
| **DEC-08** | Skill Packaging | **Canonical Skill Location & Removal of Duplicate REA Junction** | The duplicate junction `.agent/skills/reverse-engineer-anything` was unlinked. Canonical skill folder is `.agent/skills/rea/` (which declares `name: reverse-engineer-anything` in frontmatter). Junction `.agents` is ignored in Git to prevent recursive indexing. | 2026-11-06 | **APPROVED** |
| **DEC-09** | Repository Isolation | **Exclusion of Nested Upstream Repositories from Root Git** | External research repositories (`RE/Hermes/00-source-materials/hermes-agent` [~345MB] and `RE/9ROUTER/00-source-materials/9router` [~34MB]) and binary installers are excluded in root `.gitignore`. This protects the main repository from bloat and avoids broken submodules, while documentation in `00-source-materials/README.md` preserves exact upstream reproducibility. | 2026-11-06 | **APPROVED** |
| **DEC-10** | Toolchain Phasing | **Toolchain Phasing Strategy & Non-Blocking Rust Status** | Toolchain dependencies are phased: Git, Node.js, Python, uv are Required Now (Phase 00/01). Rust & Cargo are Required for Phase 06–07 (Engineering Foundation & Hime Desktop). Rust is explicitly classified as NOT a blocker for Phase 01 (Discovery & Environment Audit). | 2026-11-06 | **APPROVED** |
| **DEC-11** | Gateway Configuration & Security | **Process Environment Injection & Multi-Layer Credential Isolation** | Pass 9Router initial bootstrap settings (such as `INITIAL_PASSWORD`, `DATA_DIR`, `HOSTNAME=127.0.0.1`) strictly via process environment variables during child process spawn by Hime Process Supervisor (Rust `std::process::Command::env`), avoiding direct tampering with 9Router's internal SQLite database. Mitigate 9Router plaintext SQLite storage via multi-layered defense (NTFS ACLs, directory encryption, loopback isolation, in-memory credential design in Phase 05). | 2026-11-06 | **APPROVED** |
