# Third-Party Notices and Attributions

This project, **Hime Island**, incorporates and integrates third-party open-source software.
This document outlines the attributions, source repositories, and licenses under which these works are used.

---

## 1. Primary Upstream Derivatives

### Bloom Desktop Dynamic Island
- **Origin / Upstream Project**: Bloom
- **Author**: little-systems and contributors
- **Repository**: [https://github.com/little-systems/bloom](https://github.com/little-systems/bloom)
- **Pinned Tag**: `v3.10.2`
- **Pinned Commit**: `0ab9f2f08b75cfeffd3040330e312c2244d7726a`
- **License**: GNU General Public License v3.0 (`GPL-3.0`)
- **Attribution Notice**:
  > Hime Island is a derivative work based upon Bloom.
  > Modifications, Hime Core integration, dynamic eyes mascot, and design system are contributed by Hime Island contributors under the terms of the GNU General Public License v3.0.
  > See [LICENSE](file:///x:/folder_ai/Hime/LICENSE) and [NOTICE](file:///x:/folder_ai/Hime/NOTICE) for the complete license terms and copyright statements.

---

## 2. Integrated External Dependencies (Autonomous Agent Ecosystem)

### NousResearch Hermes Agent
- **Origin / Upstream Project**: Hermes Agent
- **Author**: NousResearch
- **Repository**: [https://github.com/NousResearch/Hermes-Agent](https://github.com/NousResearch/Hermes-Agent)
- **License**: Apache License 2.0 / MIT
- **Role**: Autonomous AI agent runtime executing in an isolated Python subprocess communicating via localhost named pipe (`\\.\pipe\hime-ipc`) with ticket-based session authentication.

### 9Router Local AI Gateway
- **Origin / Upstream Project**: 9Router
- **Author**: decolua and contributors
- **Repository**: [https://github.com/decolua/9router](https://github.com/decolua/9router)
- **License**: MIT License
- **Role**: Local AI model router and multi-provider gateway managed as a local background service bound strictly to `127.0.0.1:20128`.

---

## 3. Core Rust Libraries (Cargo Workspace)

The Rust backend (`core/crates/`) utilizes the following open-source libraries:

| Library | Version | License | Authors / Maintainers |
|:---|:---|:---|:---|
| `tokio` | 1.38.0 | MIT | Tokio Contributors |
| `tauri` | 2.0.0 | MIT OR Apache-2.0 | Tauri Programme within The Commons Conservancy |
| `rusqlite` | 0.31.0 | MIT | The rusqlite developers |
| `serde` / `serde_json` | 1.0.x | MIT OR Apache-2.0 | David Tolnay and serde contributors |
| `windows` | 0.58.0 | MIT OR Apache-2.0 | Microsoft Corporation |
| `tracing` / `tracing-subscriber` | 0.1.x / 0.3.x | MIT | Tokio Contributors |
| `thiserror` / `anyhow` | 1.0.x | MIT OR Apache-2.0 | David Tolnay |
| `sha2` | 0.10.8 | MIT OR Apache-2.0 | RustCrypto Project Developers |
| `zeroize` | 1.8.1 | Apache-2.0 OR MIT | The RustCrypto Project Developers |
| `regex` | 1.10.x | MIT OR Apache-2.0 | The Rust Project Developers |
| `reqwest` | 0.12.x | MIT OR Apache-2.0 | Sean McArthur |
| `async-trait` | 0.1.80 | MIT OR Apache-2.0 | David Tolnay |

---

## 4. Frontend & Desktop Shell Libraries (Bun / TypeScript)

The island frontend (`island/`) utilizes the following open-source packages:

| Library | Version | License | Authors / Maintainers |
|:---|:---|:---|:---|
| `react` / `react-dom` | 18.x | MIT | Meta Platforms, Inc. |
| `tailwindcss` | 3.x | MIT | Tailwind Labs, Inc. |
| `lucide-react` | 0.x | ISC | Lucide Project Contributors |
| `framer-motion` | 11.x | MIT | Framer B.V. |
| `clsx` / `tailwind-merge` | 2.x | MIT | Luke Edwards / Dany Castillo |
| `@tauri-apps/api` | 2.x | MIT OR Apache-2.0 | Tauri Programme |

---

## 5. License Copies

- For the full text of the GNU General Public License v3.0, refer to [LICENSE](file:///x:/folder_ai/Hime/LICENSE).
- For copyright notices and derivatives documentation, refer to [NOTICE](file:///x:/folder_ai/Hime/NOTICE).
- For system compatibility requirements and locked upstream hashes, refer to [COMPATIBILITY.json](file:///x:/folder_ai/Hime/docs/hime/COMPATIBILITY.json).
