# HIME USER MANUAL & CAPABILITIES GUIDE
## Release Candidate v1.0 Complete Reference

---

## 1. INTRODUCING HIME

**Hime** is an agentic personal operating environment designed for Windows (and cross-platform on Linux). It blends an elegant, unobtrusive floating Dynamic Island interface with an autonomous AI orchestration engine, secure local storage, cross-device integration, and specialized developer workspaces.

Unlike passive chatbots, Hime acts as your **personal autonomous collaborator** capable of planning, executing complex multi-step workflows, managing your files safely, and keeping you accountable across your desktop and mobile devices.

---

## 2. THE DYNAMIC ISLAND INTERFACE

The Dynamic Island sits at the top of your screen as a non-intrusive status pill that expands dynamically when active:

- **Idle State:** Displays current system status and readiness.
- **Thinking / Active State:** Pulses gently when an agent is planning or reasoning, showing current step and progress.
- **Permission Approval State:** Lights up with an attention indicator when an action requires human approval (e.g. file modification or git commit).
- **Notification State:** Briefly expands to show reminders, completed tasks, or device pairing alerts.
- **Error / Offline State:** Informs you clearly if background services are offline, with a one-click reconnect trigger.

Clicking the Island expands it into the full Conversation & Workstation panel.

---

## 3. OMNIBOX COMMAND PALETTE (`Ctrl+K`)

Press `Ctrl+K` from anywhere in Hime to open the **Omnibox**.

The Omnibox is your keyboard-first command center:
- Type natural language queries to instruct the AI agent.
- Type action commands like `/dev`, `/storage`, `/academic`, `/specialist` to jump directly to specific workspaces.
- Filter through files, tasks, notes, and browser bookmarks in milliseconds.
- Press `Up` / `Down` to navigate, `Enter` to execute, and `Esc` to close.

---

## 4. WORKSTATION VIEWS & FEATURES

### 4.1 Conversation & AI Reasoning (`Alt+1`)
- Chat with advanced reasoning models through Hermes Agent and the 9Router gateway.
- Real-time token streaming with automatic Chain-of-Thought redaction (`<think>` blocks are hidden from view to keep the UI clean).
- Instant turn cancellation via the "Stop" button.

### 4.2 Agent Activity & System Supervision (`Alt+2`)
- Inspect active background jobs, subprocess states (Hermes, 9Router), CPU and memory consumption.
- Real-time event stream showing tool calls, security policy decisions, and system logs.

### 4.3 Task Board & Accountability Kanban (`Alt+3`)
- Visual Kanban board for personal projects: Backlog, In Progress, Review, and Completed.
- Daily accountability reminders synchronized across desktop and your paired Android smartphone.

### 4.4 Storage Intelligence & Safe Operations (`Alt+5`)
- **Fast Local Indexer:** Scans and indexes local directories without sending metadata to the cloud.
- **Duplicate & Large File Finder:** Detects redundant files and reclaimable storage space.
- **Safe Operations & Hime Recycle:** Deleting or reorganizing files never permanently destroys data; files are moved to `.hime/recycle/` with 1-Click reversible restore.

### 4.5 Academic & Document Engine (`Alt+6`)
- **Indonesian Academic Paper Template:** Generates standards-compliant academic reports (Times New Roman 12pt, 1.5 line spacing, 4-3-3-3 cm margins).
- **PDF Toolkit:** Merge, split, and compress multi-page PDF files locally.
- **Artcraft Discovery:** Automatically indexes and searches local Artcraft UI/UX components.

### 4.6 Developer Workspace (`Alt+7`)
- **Allowlisted Command Runner:** Run development tools (`git`, `cargo`, `pnpm`, `python`, `tsc`, `ruff`) directly within Hime with sanitized outputs.
- **Safe Git Operations:** Review modified files, unified diffs, and perform pre-commit secret scans before committing.
- **SE Templates:** One-click generation of Software Engineering documents (IEEE 830 / ISO 29148 SRS, IEEE 1016 SAD, Test Plans, and Code Review checklists).

### 4.7 Browser & Cloud Integrations (`Alt+8`)
- **SSRF-Safe Web Research:** Browse and extract text from documentation and articles with strict loopback and metadata protection.
- **Anti-Prompt Injection Defense:** Web content is safely encapsulated so malicious page DOM cannot hijack agent reasoning.
- **Quarantined Downloads:** Downloaded assets are placed in an isolated quarantine folder with SHA-256 integrity checks.
- **GitHub & Google Drive:** Search repositories, open issues, and upload files with explicit human approval gates.

### 4.8 Specialist Agents & Data/AI Studio (`Alt+9`)
- **Domain Specialists:** Delegate complex tasks to specialized agents (Software Engineer, Data & AI, Research, Document, Productivity, Storage).
- **Data & AI Studio:** Profile CSV datasets with zero external dependencies (inferred types, missing values, summary statistics) and design leakage-free ML experiments with Stratified 5-Fold Cross Validation.
- **Safe Proactive Assistance:** Opt-in, non-intrusive proactive suggestions with cooldown rate limiting and quick mute controls.

---

## 5. PERMISSION & SAFETY MODEL

Hime operates on a strict **Fail-Closed, Least-Privilege Policy**:
- **L0 (Read-Only / Ambient):** Inspecting files, querying models, and reading status execute automatically.
- **L1 (Safe Create):** Creating temporary drafts or staged files is audit-logged.
- **L2 (Reversible Modify):** Moving files to recycle or updating tasks executes with clear notifications.
- **L3 (External Communication):** Uploading to Drive, pushing Git commits, or sending external network packets requires **explicit human approval** (via Desktop dialog or paired Android phone).
- **L4 (Irreversible Destructive):** Permanent file deletion or executing non-allowlisted commands is **strictly blocked**.

---

## 6. COMPLETE KEYBOARD SHORTCUTS REFERENCE

| Key Combination | Action |
|:---|:---|
| `Ctrl+K` | Open / Close Omnibox Command Palette |
| `Escape` | Dismiss modal dialog, overlay, or clear search |
| `Alt+1` | Jump to Conversation / Dynamic Island |
| `Alt+2` | Jump to Agent Activity & Status |
| `Alt+3` | Jump to Task Board / Kanban |
| `Alt+4` | Jump to Notification Center |
| `Alt+5` | Jump to Storage Intelligence |
| `Alt+6` | Jump to Documents & Academic Engine |
| `Alt+7` | Jump to Developer Workspace |
| `Alt+8` | Jump to Browser & Cloud Integrations |
| `Alt+9` | Jump to Specialist Agents & Data/AI Studio |
| `Enter` | Submit prompt / Execute selected item |

