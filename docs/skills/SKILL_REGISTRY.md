# Hime Skill Registry & Vetting Matrix
**Project:** Hime Assistant  
**Document:** Curated AI Agent Skill Library  
**Author:** AI Engineering Infrastructure Architect & Repository Specialist  
**Last Updated:** 2026-11-06 (Post TASK-00-02 Verification)  

---

## 1. Registry Overview & Evaluation Criteria

In accordance with [`docs/governance/SKILL_POLICY.md`](file:///x:/folder_ai/Hime/docs/governance/SKILL_POLICY.md), all external AI agent skills are evaluated against strict criteria before installation into `.agent/skills/`.

Status classifications:
- **`APPROVED`**: Pure markdown instructions or audited safe utilities with zero external risk. Fully authorized for production agent use.
- **`APPROVED_WITH_RESTRICTIONS`**: High value, but carries scripts, hooks, or environment assumptions that must be sandboxed or adapted.
- **`PENDING_REVIEW`**: Candidate skill discovered; full code audit not yet completed.
- **`REJECTED`**: Fails security, licensing, or architectural alignment requirements.

---

## 2. Skill Inventory & Verification Status

| Skill Name | Canonical Path | Upstream Source | License | Format | Status | Vetting Notes |
|:---|:---|:---|:---:|:---:|:---:|:---|
| **`animate`** | `.agent/skills/animate/` | `emilkowalski/skills` | MIT | `SKILL.md` + `RECIPES.md` | **`APPROVED`** | Pure Markdown, physics curves, durations <300ms, offline recipe cache. |
| **`emil-design-eng`** | `.agent/skills/emil-design-eng/` | `emilkowalski/skills` | MIT | `SKILL.md` | **`APPROVED`** | Pure Markdown, craft sensibility, Before/After table review format. |
| **`ponytail`** | `.agent/skills/ponytail/` | `DietrichGebert/ponytail` | MIT | `SKILL.md` | **`APPROVED_WITH_RESTRICTIONS`** | Minimalist 7-rung solution ladder. Node.js hooks excluded; pure ruleset active. |
| **`rea`** | `.agent/skills/rea/` | `morluto/rea` | MIT | `SKILL.md` + 7 refs | **`APPROVED`** | Canonical reverse engineering skill. Duplicate junction `reverse-engineer-anything` unlinked. |
| **`systematic-debugging`** | `.agent/skills/systematic-debugging/` | `obra/superpowers` | MIT | `SKILL.md` | **`APPROVED`** | Pure Markdown 4-phase root cause analysis process. |
| **`verification-before-completion`** | `.agent/skills/verification-before-completion/` | `obra/superpowers` | MIT | `SKILL.md` | **`APPROVED`** | Evidence gating before completion claims. Pure procedural checklist. |
| **`web-design-guidelines`** | `.agent/skills/web-design-guidelines/` | `vercel-labs/agent-skills` | MIT | `SKILL.md` + ref cache | **`APPROVED`** | Web accessibility & UX checklist, offline guidelines cache stored. |

---

## 3. Methodology Skills Interaction & Priority Hierarchy

Specialized investigation was conducted on the interaction between **`ponytail`**, **`systematic-debugging`**, and **`verification-before-completion`**:

### 3.1 Mutual Compatibility Analysis
- **Zero Instruction Conflicts:** The three methodology skills govern distinct, complementary phases of the task lifecycle:
  1. `systematic-debugging` governs **Investigation** (preventing guessing and symptom patching).
  2. `ponytail` governs **Design & Implementation** (enforcing the 7-rung solution ladder and preventing bloat).
  3. `verification-before-completion` governs **Completion Gating** (mandating fresh command evidence before claiming success).
- **Mutual Reinforcement:**
  - `ponytail` explicitly states: *"Never cut validation at trust boundaries, error handling that prevents data loss, security... Lazy code without its check is unfinished: new non-trivial logic leaves one small test or assert-based self-check."*
  - This directly aligns with `verification-before-completion`'s Iron Law: *"NO COMPLETION CLAIMS WITHOUT FRESH VERIFICATION EVIDENCE."*
  - Both directly implement [`AGENTS.md`](file:///x:/folder_ai/Hime/AGENTS.md) Section B (Evidence-First Engineering), Section C (Minimal Implementation), and Section H (Completion Verification).

### 3.2 Invocation & Priority Guide for Agents

```text
Problem / Request Received
           │
           ▼
[ Is it a bug, failure, or unexpected behavior? ]
     ├─► YES: Activate 'systematic-debugging'
     │        Phase 1: Root Cause Investigation (NO code edits yet)
     │        Phase 2: Pattern Analysis across all callers
     │
     └─► NO / When Ready to Code:
              │
              ▼
         Activate 'ponytail' (Default Level: full)
         Apply 7-Rung Solution Ladder:
         YAGNI → Existing Code → Stdlib → Native Platform → Deps → One-Liner → Minimal Diff
              │
              ▼
         [ Is UI / Animation Involved? ]
              ├─► YES: Comply with 'emil-design-eng' & 'animate' (<300ms, exact curves)
              └─► NO: Continue
              │
              ▼
         Activate 'verification-before-completion'
         Run full command → Read exit code → Confirm output → Present claim with evidence
              │
              ▼
         Task Completed (Submit for Review)
```

---

## 4. Canonical Location Resolution for REA Skill

- **Finding:** A directory junction existed at `.agent/skills/reverse-engineer-anything` pointing to `.agent/skills/rea`.
- **Resolution:** In TASK-00-02, the junction was unlinked safely (`cmd /c rmdir`). The canonical skill location is permanently set to:
  `X:\folder_ai\Hime\.agent\skills\rea/`
- **Verification:** Antigravity correctly indexes the skill from `rea/SKILL.md` using the YAML frontmatter name `reverse-engineer-anything`.
