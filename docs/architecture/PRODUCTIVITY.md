# Hime Architecture — Productivity & Accountability Engine

> **Domain:** Productivity, Task Management, Collision-Free Scheduling & Accountability Governance  
> **Phase:** 12 — Productivity, Accountability & Document Suite  
> **Status:** APPROVED & IMPLEMENTED  

---

## 1. Overview & Core Philosophy

The Hime Productivity Engine provides an integrated, non-intrusive personal task and study management system designed around deterministic scheduling and psychological safety.

### The Cardinal Principle: "Hime menagih, bukan mengunci"
Unlike traditional parental controls or coercive blocker software, Hime **never takes over the host OS**, **never terminates applications violently without approval**, and **never locks the user out of their computer**. Hime operates as an empathetic, disciplined assistant that:
- Prominently surfaces commitments and deadlines upon startup.
- Breaks overwhelming tasks into structured, actionable subtasks.
- Enforces focus periods with non-punitive gentle nudges.
- Always provides a snooze option accompanied by a brief self-reflection prompt.

---

## 2. Component Topology

```text
┌────────────────────────────────────────────────────────┐
│                      Hime Desktop                      │
│     (KanbanBoard, StudyTimetable, AccountabilityCards)  │
└───────────────────────────┬────────────────────────────┘
                            │ Named Pipe JSON-RPC (20129)
┌───────────────────────────▼────────────────────────────┐
│                  Hime Core Control Plane               │
│                                                        │
│  ┌─────────────────┐ ┌────────────────┐ ┌───────────┐  │
│  │  KanbanManager  │ │ StudyScheduler │ │ Account-  │  │
│  │ (NLP Parsing &  │ │(Collision-Free │ │ ability   │  │
│  │  Subtasks Gen)  │ │ Break Padding) │ │  Manager  │  │
│  └────────┬────────┘ └────────┬───────┘ └─────┬─────┘  │
│           │                   │               │        │
│  ┌────────▼───────────────────▼───────────────▼─────┐  │
│  │             hime-storage (SQLite Database)        │  │
│  │ kanban_boards, kanban_columns, productivity_tasks│  │
│  │ study_schedules, accountability_settings         │  │
│  │ screen_time_sessions                             │  │
│  └──────────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────┘
```

---

## 3. Module Specifications

### A. Kanban Engine (`hime-core::productivity::kanban`)
- **Default Board & Columns**: Automatically provisions `Hime Master Workspace` with 4 default workflow states:
  1. `To Do` (Initial backlog)
  2. `In Progress` (Active focus)
  3. `Review` (Verification & polish)
  4. `Done` (Archived & audited)
- **Natural Language Parsing**:
  - Deterministically parses colloquial inputs in both Indonesian and English:
    - `"Selesaikan bab 4 metodologi besok siang prioritas tinggi"`
    - Extracted: Title, priority (`high`), deadline (timestamp), and tags.
- **Subtask Decomposition**:
  - Automatically suggests 3 structured sub-actions for any complex task (e.g., "Outline & gather sources", "Draft core sections", "Proofread and citations").

### B. Study Scheduler (`hime-core::productivity::scheduler`)
- **Deterministic Collision Avoidance**:
  - When scheduling blocks (e.g. 45-minute Pomodoro sessions), evaluates all existing active schedules.
  - Automatically inserts **10 minutes of buffer/break padding** between blocks.
  - If a collision occurs with an existing block, calculates the earliest non-overlapping time slot.
- **Rescheduling Audit Trail**:
  - Tracks previous schedule IDs in `rescheduled_from` to analyze study habits without penalty.

### C. Accountability Governance (`hime-core::productivity::accountability`)
Three distinct operating modes:

| Mode | Target Context | Startup Behavior | In-Session Distraction Nudge | Snooze Rules |
|:---|:---|:---|:---|:---|
| **Santai** | Holidays, creative exploration, weekend rest | Silent; no startup popups | Disabled | Infinite snooze without prompt |
| **Biasa** *(Default)* | Everyday professional and academic work | Concise startup greeting & urgent tasks | Gentle notification if non-work window > 15m | 15m / 30m / 1h snooze options |
| **Strict** | Thesis sprint, exam week, tight release | Prominent task breakdown & 45m focus timer | Nudge with urgent deadline reminders | Snooze requires 1-sentence reflection |

---

## 4. Privacy & Security Compliance

1. **Zero Secret Leakage**: No personal study notes or window titles leave localhost.
2. **Explicit Opt-in for Screen Time Monitoring**: Window title recording is **disabled by default (`opt_in = 0`)** to ensure compliance with SEC-09 and least-privilege principles.
3. **Audit Trail**: Every task creation, status advancement, and mode switch is immutably recorded to SQLite with SHA-256 hash chaining.
