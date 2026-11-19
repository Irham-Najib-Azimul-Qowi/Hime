# Source Materials: Hermes Agent

This directory contains the upstream source repository of Hermes Agent used for architectural analysis and integration research.

## Upstream Repository Information

- **Upstream Source:** `https://github.com/NousResearch/hermes-agent.git`
- **Tracked Research Commit:** `c5f347eb` (Branch: `main`)
- **Maintained By:** Nous Research

## Reproduction Instructions

To clone the exact upstream commit:
```bash
git clone https://github.com/NousResearch/hermes-agent.git hermes-agent
cd hermes-agent
git checkout c5f347eb
```

*Note: The `hermes-agent/` directory is maintained as an independent git repository and is excluded from the root Hime repository via `.gitignore` to prevent repository bloating and subproject commit conflicts. All analysis reports, lifecycle maps, and integration documents are tracked in `RE/Hermes/reports/`.*
