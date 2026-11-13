# Third-Party Dependencies & Licensing Audit

> **Document:** Open Source Licenses & Compliance Matrix  
> **Phase:** 12 — Productivity, Accountability & Document Suite  
> **Audited By:** Principal Software Engineer  
> **Status:** COMPLIANT  

---

## 1. Overview

Hime integrates several robust, open-source libraries and external tools to facilitate document synthesis, PDF manipulation, and text parsing. This document catalogs all third-party components, their licensing, compatibility, and compliance obligations.

---

## 2. Dependency Matrix

| Component | Language / Ecosystem | Version | License | Usage in Hime | Commercial Friendly |
|:---|:---|:---|:---|:---|:---|
| **serde / serde_json** | Rust | 1.0.x | MIT / Apache-2.0 | Serialization & JSON-RPC framing | Yes |
| **rusqlite** | Rust | 0.31.x | MIT | Local SQLite embedded storage engine | Yes |
| **chrono** | Rust | 0.4.x | MIT / Apache-2.0 | Timestamps, scheduling & time zones | Yes |
| **uuid** | Rust | 1.8.x | MIT / Apache-2.0 | Universally unique identifiers | Yes |
| **tokio** | Rust | 1.38.x | MIT | Async runtime & IO primitives | Yes |
| **tracing** | Rust | 0.1.x | MIT | Structured logging & diagnostics | Yes |
| **windows-sys** | Rust | 0.52.x | MIT / Apache-2.0 | Windows native APIs (Named Pipes, DPAPI) | Yes |
| **React** | TypeScript | 18.x | MIT | Desktop frontend UI library | Yes |
| **Vite** | TypeScript | 5.x | MIT | Frontend build tool and bundler | Yes |
| **Tailwind CSS** | CSS / JS | 3.x | MIT | Utility-first CSS styling framework | Yes |
| **Tauri** | Rust / C++ | 2.0.x | MIT / Apache-2.0 | Lightweight desktop shell | Yes |

---

## 3. Optional External CLI Utilities (User-Installed)

For enhanced document conversions, Hime can optionally invoke the following system-installed binaries when detected in `%PATH%`. Note that Hime does **not bundle proprietary or copyleft binaries** directly in the distribution; it interacts strictly via standard process invocation.

| Tool | Upstream License | Distribution Mechanism | Compliance Verification |
|:---|:---|:---|:---|
| **Pandoc** | GPL-3.0-or-later | User installed via winget/brew | Executed as external subprocess; no static/dynamic linking. Fully compliant. |
| **LibreOffice (headless)** | MPL-2.0 | User installed | Interfaced via CLI command line argument. Fully compliant. |
| **qpdf** | Apache-2.0 | User installed / bundled permissive | Permissive Apache-2.0 license. Fully compliant. |
| **Tesseract OCR** | Apache-2.0 | User installed / optional | Permissive Apache-2.0 license. Fully compliant. |

---

## 4. Compliance Verification Summary

- Zero proprietary copyleft code is statically linked into `hime-core`, `hime-storage`, `hime-ipc`, or `hime-desktop`.
- All directly linked dependencies utilize permissive licenses (MIT or Apache-2.0).
- External tools with copyleft licenses (such as GPL-3.0 for Pandoc) are executed exclusively as isolated external subprocesses with standard IO streams, adhering strictly to FSF compliance guidelines.
