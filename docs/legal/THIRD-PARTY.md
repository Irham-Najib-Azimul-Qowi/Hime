# THIRD-PARTY NOTICES & LEGAL ATTRIBUTION
## Clean-Room Reimplementation & Third-Party Licenses (Closing G11)

**Otoritas:** Legal & Compliance Lead  
**Pelaksana:** Principal Software Engineer & AI Executor  
**Klasifikasi:** Legal & Licensing Documentation  
**Tanggal:** 2026-11-14  

---

## 1. NUSA CLEAN-ROOM REIMPLEMENTATION AUDIT (CLOSING G11)

### Status: **VERIFIED CLEAN-ROOM — 100% INDEPENDENT IMPLEMENTATION**

1. **Zero Source Code Reuse:**
   - The proprietary source code of Nusa Dynamic Island is completely unavailable and was never accessed or used in the development of Hime.
   - Hime Desktop is a **clean-room reimplementation** developed entirely from scratch using Rust, Tauri 2.0, TypeScript, and modern web standards.
2. **Zero Copyrighted Assets:**
   - All visual assets, UI layouts, icons, fonts, CSS styles, and design tokens in Hime are original creations or sourced from permissible open-source libraries (e.g. Lucide / SVG icons).
   - No binaries, decompiled assets, images, audio files, or reverse-engineered resources from Nusa are contained within this repository or shipped application bundles.

---

## 2. EXTERNAL DEPENDENCIES & RUNTIMES INTEGRATION

Hime integrates external open-source projects as managed background services under documented boundaries without modifying upstream copyright notices:

| Component / Project | Original Author | License | Integration Topology |
|:---|:---|:---:|:---|
| **Hermes Agent** | NousResearch | MIT License | External Subprocess via Named Pipe / HTTP API |
| **9Router** | decolua | Apache License 2.0 | Managed Loopback Gateway via `127.0.0.1:20128` |

Both dependencies remain unmodified. Hime interacts strictly through standard network and IPC boundaries (Dual-Control Plane).

---

## 3. OPEN SOURCE LIBRARIES & LICENSES

Hime incorporates the following open-source libraries in compliance with their respective licenses:

### 3.1 Rust Workspace Crates (`Cargo.lock`)
- **Tauri Framework (`tauri`, `tauri-build`):** Copyright (c) 2019 - present Tauri Programme within The Commons Conservancy. Licensed under MIT or Apache 2.0.
- **Tokio (`tokio`, `tokio-macros`):** Copyright (c) 2024 Tokio Contributors. Licensed under MIT.
- **Rusqlite (`rusqlite`):** Copyright (c) 2014-2024 The Rusqlite Developers. Licensed under MIT. SQLite itself is Public Domain.
- **Serde (`serde`, `serde_json`):** Copyright (c) 2014-2024 David Tolnay and Serde Contributors. Licensed under MIT or Apache 2.0.
- **Tracing (`tracing`, `tracing-subscriber`):** Copyright (c) 2024 Tokio Contributors. Licensed under MIT.
- **Windows-rs (`windows`):** Copyright (c) Microsoft Corporation. Licensed under MIT or Apache 2.0.
- **Sha2 (`sha2`):** Copyright (c) RustCrypto Contributors. Licensed under MIT or Apache 2.0.
- **Zeroize (`zeroize`):** Copyright (c) RustCrypto Contributors. Licensed under Apache 2.0.
- **Thiserror (`thiserror`):** Copyright (c) 2024 David Tolnay. Licensed under MIT or Apache 2.0.

### 3.2 Desktop Frontend (`package.json`)
- **React & React DOM:** Copyright (c) Meta Platforms, Inc. and affiliates. Licensed under MIT.
- **Vite:** Copyright (c) 2019-present Evan You & Vite Contributors. Licensed under MIT.
- **TypeScript:** Copyright (c) Microsoft Corporation. Licensed under Apache 2.0.

---

## 4. DISTRIBUTION AND COMPLIANCE NOTICE

The Hime project is released under the **MIT License OR Apache License 2.0** (as specified in root `Cargo.toml`).

Users, distributors, and contributors are permitted to use, modify, and distribute Hime under the conditions set forth in the project license, provided that original copyright notices and this third-party attribution document remain intact.
