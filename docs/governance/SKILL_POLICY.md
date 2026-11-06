# Skill Policy & Vetting Governance
**Project:** Hime Assistant  
**Version:** 1.0  
**Effective Date:** 2026-11-06  

---

## 1. Objective

This policy establishes strict procurement, vetting, installation, and auditing standards for AI Agent Skills used in the Hime Assistant ecosystem.

Skills augment agent capabilities through structured prompts, procedural workflows, and reference materials. Because malicious or unvetted skills can introduce supply-chain attacks, prompt injection vulnerabilities, or unauthorized execution vectors, no skill may be installed without passing this vetting policy.

---

## 2. Skill Vetting Lifecycle

Every prospective skill must pass through four formal gates before becoming active:

```text
Discovery & Nomination → Technical & Security Vetting → Approval Classification → Controlled Installation & Pinning
```

### Stage 1: Discovery & Nomination
- Source must be traced to an authoritative, official repository (e.g., Anthropic, Vercel Labs, Trail of Bits, Jesse Vincent).
- Anonymous, unmaintained forks, or third-party mirrors are rejected by default.
- The repository must have an explicit open-source license (MIT, Apache-2.0, BSD). Copyleft licenses (GPL, AGPL) or CC-BY-SA require explicit legal review.

### Stage 2: Technical & Security Vetting
Every skill directory must be thoroughly inspected for:
1. **Executable Code & Scripts:** Does the skill bundle `.sh`, `.ps1`, `.py`, `.js`, or binary executables (`.exe`, `.dll`, `.so`)?
   - Pure Markdown skills (`SKILL.md` + reference `.md`) are strongly preferred.
   - Any bundled script must be audited line-by-line for external network calls, file overwrites, and shell execution.
2. **Instruction Conflicts:** Does the skill contradict root [`AGENTS.md`](file:///x:/folder_ai/Hime/AGENTS.md) or project policies?
   - Skills that encourage skipping tests, using `--force`, bypassing linters, or ignoring error handling are rejected.
3. **Permission Requirements:** What agent tools does the skill declare in frontmatter (`allowed-tools`) or require in practice?
4. **Supply Chain Risks:** Does the skill instruct the agent to run `curl | bash`, fetch unpinned remote assets, or install global npm/pip packages?

### Stage 3: Approval Classification
Skills are assigned one of four statuses in [`SKILL_REGISTRY.md`](file:///x:/folder_ai/Hime/docs/skills/SKILL_REGISTRY.md):

| Status | Definition | Action Permitted |
|:---|:---|:---|
| **APPROVED** | Verified pure markdown or fully audited safe scripts with zero external risk. | Safe for full installation into `.agent/skills/`. |
| **APPROVED_WITH_RESTRICTIONS** | Functionally valuable, but contains scripts or narrow tool assumptions that must be sandboxed or adapted. | Install with specific disabled hooks/scripts or runtime constraints. |
| **PENDING_REVIEW** | Candidate identified, but formal audit has not yet completed. | Prohibited from installation or active agent usage. |
| **REJECTED** | Fails security, license, quality, or architectural alignment criteria. | Strictly prohibited. Documented in registry with reason. |

### Stage 4: Controlled Installation & Pinning
- Skills must be installed in `.agent/skills/<skill-name>/` (with `.agents/` preserved as a symlink).
- The exact upstream commit hash or version tag must be recorded in the skill registry.
- Upstream files must not be silently modified. Any project-specific adaptation must be recorded with justification.

---

## 3. Maintenance & Periodic Re-Audit

- Installed skills must be re-audited whenever upstream updates are fetched.
- If a security vulnerability is identified in an installed skill, it must be demoted to `REJECTED` and removed from `.agent/skills/` immediately.

