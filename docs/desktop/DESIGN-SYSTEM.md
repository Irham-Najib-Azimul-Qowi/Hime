# Hime Desktop Experience — Design System Specification

- **Document ID:** DOCS-DESKTOP-DESIGN-SYSTEM
- **Phase:** Phase 07 — Hime Desktop Experience
- **Authority:** System Architect & Technical Lead
- **Philosophy:** Emil Kowalski Design Engineering (Craft, Purposeful Motion, Visual Hierarchy)
- **Status:** APPROVED IMPLEMENTATION REFERENCE

---

## 1. Design Direction & Visual Identity

Hime is designed as a personal AI desktop companion that avoids SaaS dashboard templates, excessive glow, generic cards, and arbitrary gradients. The aesthetic is **obsidian minimal, precise, modern, and purposeful**, drawing inspiration from dynamic island physical affordances, desktop HUDs, and focused productivity interfaces.

### Core Visual Principles
1. **Unseen details compound:** Every micro-interaction (button press scales, smooth spring easings, hover transitions) is tuned so the interface feels right without calling attention to itself.
2. **Restraint over decoration:** No gradients on every card; borders are subtle (6%–12% opacity), shadows are measured elevations, and neon glows are restricted strictly to active live status indicators.
3. **Purposeful motion:** Actions repeated hundreds of times a day (Omnibox dismissal, view switching) have immediate or near-zero duration, while state morphs (Dynamic Island expansion) use a spring curve (`cubic-bezier(0.16, 1, 0.3, 1)`).
4. **Honest state representation:** Features scheduled for future milestones (Hermes Agent reasoning loop in Phase 08) are cleanly and transparently labeled (`[Phase 08 Planned]`) rather than fabricated with fake mock AI responses.

---

## 2. Design Tokens

### A. Color Palette

#### Dark Mode (Default)
| Token | Value | Semantic Purpose |
|:---|:---|:---|
| `--bg-app` | `#090b10` | Base desktop viewport canvas |
| `--bg-surface-1` | `#10131c` | Sidebar navigation, primary window panels |
| `--bg-surface-2` | `#161b26` | Content cards, input fields, control tiles |
| `--bg-surface-3` | `#1f2536` | Active tab pill, chip backgrounds, nested containers |
| `--border-subtle` | `rgba(255, 255, 255, 0.06)` | Divider lines, card borders |
| `--border-medium` | `rgba(255, 255, 255, 0.12)` | Focused input borders, window separation |
| `--border-focus` | `rgba(139, 92, 246, 0.6)` | Keyboard accessibility focus rings |
| `--text-primary` | `#f8fafc` | Primary titles, active text |
| `--text-secondary` | `#94a3b8` | Subtitles, labels, card descriptions |
| `--text-muted` | `#64748b` | Timestamps, metadata, hints |
| `--accent-primary` | `#8b5cf6` | Hime Violet identity, primary buttons |
| `--accent-cyan` | `#06b6d4` | Live IPC telemetry, active tasks, tool execution |
| `--accent-success` | `#10b981` | Core Online, passed checks, operational indicators |
| `--accent-warning` | `#f59e0b` | Policy approval requests, open blockers (IB-01/IB-02) |
| `--accent-danger` | `#ef4444` | Core Offline, policy denial, error alerts |

#### Light Mode
| Token | Value | Semantic Purpose |
|:---|:---|:---|
| `--bg-app` | `#f8fafc` | Light background canvas |
| `--bg-surface-1` | `#ffffff` | Clean card background |
| `--bg-surface-2` | `#f1f5f9` | Elevated light container |
| `--text-primary` | `#0f172a` | Deep slate contrast typography |
| `--accent-primary` | `#7c3aed` | High-contrast violet accent |

### B. Typography Scale

Hime utilizes native system sans typography (`-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto`) for clean legibility across DPI scales, and monospace (`"JetBrains Mono", Consolas`) for telemetry and code.

| Token | Size | Line Height | Usage |
|:---|:---|:---|:---|
| `--text-2xs` | 10px | 1.4 | Badges, status pills, micro metadata |
| `--text-xs` | 12px | 1.5 | Timestamps, category chips, keyboard keys |
| `--text-sm` | 13px | 1.5 | Navigation links, body copy, card descriptions |
| `--text-base` | 14px | 1.5 | Inputs, default copy, chat messages |
| `--text-md` | 16px | 1.4 | Card headers, modal titles, Omnibox input |
| `--text-xl` | 20px | 1.3 | View page titles |

### C. Spacing Scale
- `--space-1`: 4px
- `--space-2`: 8px
- `--space-3`: 12px
- `--space-4`: 16px
- `--space-5`: 20px
- `--space-6`: 24px
- `--space-8`: 32px

### D. Radii System
- `--radius-xs`: 4px (keyboard shortcut badges)
- `--radius-sm`: 6px (buttons, inputs)
- `--radius-md`: 10px (cards, chat bubbles)
- `--radius-lg`: 14px (modals, dialogs)
- `--radius-xl`: 20px (expanded Dynamic Island)
- `--radius-pill`: 9999px (compact Dynamic Island pill, status badges)

### E. Motion Tokens
- Spring ease: `cubic-bezier(0.16, 1, 0.3, 1)`
- Smooth exit: `cubic-bezier(0.25, 1, 0.5, 1)`
- Micro duration: `120ms`
- Normal duration: `200ms`
- Layout expand duration: `280ms`
- Pressable button transform: `:active { transform: scale(0.97); }`
- Accessible reduced-motion overrides: `prefers-reduced-motion: reduce` sets durations to `0ms`.

