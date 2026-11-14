# ADR 007: Document Engine and Non-Punitive Accountability Architecture

## Status
ACCEPTED

## Context
Phase 12 requires two major feature domains for Hime:
1. **Productivity and Accountability (UR-10, UR-11, UR-12, UR-13, UR-15)**: Task management, natural language parsing, collision-free study timetables, and an accountability system that encourages discipline without acting like coercive malware.
2. **Document Processing and Artcraft Interoperability (UR-06, UR-14, B3, B4)**: Document synthesis compliant with Indonesian academic paper standards (Times New Roman 12 pt, 1.5 line spacing, 4-3-3-3 margin configuration), PDF manipulation, and safe interoperability with Artcraft (WordCraft / FigCraft).

Key architectural challenges:
- Coercive software that locks users out of their OS induces user resentment, creates critical safety risks in emergencies, and is frequently uninstalled or bypassed.
- Blind GUI automation of proprietary office suites (synthesizing clicks and typing) is notoriously fragile across varying display resolutions, scaling factors, and active window states.
- High-intensity PDF compilation and image-heavy report generation can exhaust memory or lock the core server thread if not sandboxed and throttled.

## Decision
1. **Adopt "Hime menagih, bukan mengunci" Philosophy**:
   - Hime will never lock the operating system, terminate running programs without approval, or block emergency functions.
   - Three discrete modes are established: *Santai*, *Biasa* (default), and *Strict*.
   - In *Strict* mode, urgent obligations and a 45-minute focus timer are immediately presented upon startup, but the user retains the prerogative to snooze the session by providing a brief reflection reason.
   - Screen time tracking of active window titles is strictly **opt-in** (`opt_in = 0` default) to prevent secret or sensitive data leakage.
2. **Open Interchange Format Model for Artcraft (WordCraft / FigCraft)**:
   - Rather than simulating mouse clicks on GUI windows, Hime discovers installed Artcraft suites via standard filesystem paths and registry entries, communicating strictly through open document standards (`.docx`, `.pptx`, `.pdf`, `.svg`).
   - Standard academic papers are generated directly into semantic HTML5 or standard OpenXML (`.docx`) with embedded style metadata conforming to Indonesian academic guidelines (Times New Roman 12 pt, 1.5 spacing, 4-3-3-3 margins, automated TOC).
3. **Sandboxed Worker Pool with Heavy Job Mutex**:
   - Heavy document jobs (merging, compression, synthesis) execute in `.hime/documents_sandbox` under an asynchronous mutex lock, preventing concurrency exhaustion.
   - Every completed document undergoes structural verification, computing an automated fidelity score.

## Consequences
### Positive
- Zero risk of host system lockouts or destructive app terminations.
- Guaranteed visual and typographic compliance for Indonesian university theses and reports.
- Reliable, resolution-independent integration with local Artcraft apps.
- Full compliance with Hime's fail-closed security and least-privilege standards.

### Negative / Trade-offs
- Users who desire hard OS locks (like Cold Turkey or Freedom) will find Hime non-coercive by design. This is an intentional product design decision.
- WordCraft/FigCraft interaction is asynchronous via file interchange rather than live bidirectional socket streaming until Artcraft publishes a formalized RPC plugin API.

