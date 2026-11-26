# ADR 009: Specialist Agents, Data/AI Engine, and Proactive Assistance Architecture

## Status
Accepted

## Date
2026-11-14

## Context
Phase 14 requires expanding Hime's autonomous intelligence to support domain-specific specialist agents, advanced Data & AI engineering capabilities (UR-13), safe proactive user assistance, and automated specialist evaluation.

Several architectural challenges must be addressed:
1. **Process Proliferation & Memory Footprint**: Running independent OS processes or containers for every domain specialist degrades system responsiveness on target consumer hardware (such as dual-core or quad-core Intel i5 laptops).
2. **Specialist Scope Creep & Privilege Escalation**: Domain specialists might inadvertently execute tools outside their intended scope or exceed token consumption budgets.
3. **Data Leakage in Machine Learning Pipelines**: Data profiling and ML experiment generation frequently suffer from target leakage, where preprocessing or scaling statistics computed on full datasets leak into validation sets.
4. **Intrusive Proactivity ("Clippy Syndrome")**: Unsolicited proactive assistance can frustrate users if suggestions interrupt active workflows or repeat frequently.
5. **Hallucinated Task Completion**: Specialist workflows could falsely claim task success without verifiable evidence.

## Decisions

1. **Context-Isolated Specialists within Single Orchestrator**:
   - Rather than spawning heavyweight OS processes, specialists (`specialist.software_engineer`, `specialist.data_ai`, `specialist.research`, `specialist.document`, `specialist.productivity`, `specialist.storage`) are registered as scoped logical entities within `hime-core`.
   - Each specialist defines a tailored system prompt, dedicated token budget (e.g. 4,000–8,000 tokens), and strict tool allowlist (`allowed_tools`).

2. **Fail-Closed Tool Authorization via PDP/PEP**:
   - The `DelegationEngine` acts as an enforcement point before tool execution. If a specialist requests a tool not in its `allowed_tools` set, the request is rejected immediately with an authorization error.
   - Task completion requires evidence validation (`verify_evidence`).

3. **Leakage-Free Data/AI Experimentation Engine**:
   - The `DataAiEngine` computes dataset metadata and statistical summaries without external heavy dependencies.
   - For ML experiments, the engine enforces nested Stratified 5-Fold Cross Validation where all preprocessing and scaling steps are strictly confined within training folds, and baseline benchmarks are constrained to CPU limits (e.g. max 4 threads for Intel i5).

4. **Opt-In, Non-Intrusive Proactive Suggestions**:
   - Proactivity is disabled by default (`is_enabled: false`) until explicitly toggled by the user.
   - High-confidence threshold (>= 0.75) and configurable cooldown periods (default: 30 minutes) prevent notification fatigue.
   - Users can quick-mute proactive suggestions for 1, 4, or 24 hours. User responses (`accepted`, `dismissed`, `muted`) are recorded to measure real usefulness.

5. **Automated Evaluation Harness**:
   - The `EvaluationHarness` runs benchmark tests covering task completion, tool allowlist adherence, evidence verification, and token efficiency, continuously computing objective capability scores stored in SQLite.

## Consequences

- Domain specialists operate with zero OS process bloat while maintaining strict tool and token boundaries.
- Data science pipelines are protected against data leakage by construction.
- Proactive assistance remains respectful of user attention and controllable at all times.
- Quality and safety of specialist behavior are continuously benchmarked and auditable.

