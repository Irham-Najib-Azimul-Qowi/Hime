# Hime Architecture — Document Suite & Artcraft Interoperability

> **Domain:** Document Synthesis, Academic Templating, PDF Processing & Artcraft Integration  
> **Phase:** 12 — Productivity, Accountability & Document Suite  
> **Status:** APPROVED & IMPLEMENTED  

---

## 1. Overview & Objectives

The Hime Document Suite delivers a robust, offline-capable document generation engine capable of producing standards-compliant academic papers (Indonesian university standards) and modern business reports. Additionally, it integrates safely with local installations of Artcraft (WordCraft / FigCraft) via open interchange formats.

### Core Objectives:
1. **Academic ID Standard Compliance (UR-14 / B3)**: Built-in specifications conforming to Indonesian academic paper standards (Times New Roman 12 pt, 1.5 line spacing, 4-3-3-3 margin configuration, automated table of contents).
2. **Visual Fidelity Verification**: Every generated document is evaluated against structural criteria (page counts, typography, margins, cover page, table of contents) to produce an automated fidelity score.
3. **Safe Artcraft Interoperability (UR-06 / B4)**: Avoid fragile, dangerous blind GUI automation. Detect installed Artcraft suites via registry/filesystem and communicate exclusively through open standard formats (`.docx`, `.pptx`, `.pdf`, `.svg`).
4. **Sandboxed Worker Execution**: All heavy document rendering, merging, and compression jobs execute in an isolated sandbox (`.hime/documents_sandbox`) under mutex locking to prevent CPU thrashing.

---

## 2. Component Architecture

```text
┌────────────────────────────────────────────────────────┐
│                      Hime Desktop                      │
│      (DocumentGenerator, PDFMerge, ArtcraftStatus)     │
└───────────────────────────┬────────────────────────────┘
                            │ Named Pipe JSON-RPC (20129)
┌───────────────────────────▼────────────────────────────┐
│                  Hime Core Control Plane               │
│                                                        │
│  ┌─────────────────┐ ┌────────────────┐ ┌───────────┐  │
│  │TemplateRegistry │ │ DocumentOps    │ │  Artcraft │  │
│  │ (academic-id,   │ │ (HTML/DOCX gen,│ │  Adapter  │  │
│  │  modern-report) │ │  merge, verify)│ │ (Registry)│  │
│  └────────┬────────┘ └────────┬───────┘ └─────┬─────┘  │
│           │                   │               │        │
│  ┌────────▼───────────────────▼───────────────▼─────┐  │
│  │             DocumentService Worker Pool           │  │
│  │  - Sandboxed path: .hime/documents_sandbox        │  │
│  │  - Single heavy job execution mutex lock          │  │
│  │  - SQLite audit logging in `document_jobs`        │  │
│  └──────────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────┘
```

---

## 3. Academic Indonesian Paper Standard Specifications

The `academic-id` template enforces the following rigid formatting constraints:

| Attribute | Specification | Justification |
|:---|:---|:---|
| **Font Family** | Times New Roman | Indonesian university standard |
| **Font Size** | 12 pt (Body), 14 pt bold (Headings) | Academic legibility |
| **Line Spacing** | 1.5 lines | Proofreading & annotation room |
| **Left Margin** | 4.0 cm (1.57 inches) | Binding margin buffer |
| **Top Margin** | 3.0 cm (1.18 inches) | Header spacing |
| **Right Margin** | 3.0 cm (1.18 inches) | Edge gutter |
| **Bottom Margin** | 3.0 cm (1.18 inches) | Page numbering |
| **Table of Contents** | Required if document > 5 pages | Rapid navigation |
| **Abstract** | Dual language (Bahasa Indonesia + English) | Institutional requirement |

---

## 4. Artcraft Interoperability Philosophy (UR-06 / B4)

Direct GUI automation (synthesizing mouse clicks and keystrokes on arbitrary application windows) is strictly forbidden for document workflows due to:
1. Vulnerability to resolution and DPI scaling changes.
2. Unpredictable window focus loss and user disruption.
3. Lack of deterministic verification and feedback loops.

Instead, Hime employs the **Open Interchange Format Model**:
- **Discovery**: Hime checks standard installation directories (`%LOCALAPPDATA%\Programs\Artcraft`, `C:\Program Files\Artcraft`) for WordCraft and FigCraft binaries.
- **Interchange**: Hime exports documents in standard `.docx` / `.pptx` / `.pdf` format. When WordCraft is present, Hime invokes the application via command-line arguments to load the generated file into the user's workspace.
- **Visual Design**: For UI design assets, Hime generates standard SVG and JSON layouts compatible with FigCraft import.

---

## 5. Security & Isolation

- **Path Traversal Protection**: All user-provided output paths are resolved and validated by `hime-security::PolicyEngine`. If a requested path lies outside safe directories (e.g. system directories), the operation fails closed.
- **Quota & Resource Protection**: Maximum document size is bounded at 100 MB. Mutex locks prevent multiple heavy PDF compression/merge tasks from starving system memory.
- **Sandboxed Staging**: Intermediate compilation files reside strictly within `.hime/documents_sandbox` and are cleaned up upon job completion.
