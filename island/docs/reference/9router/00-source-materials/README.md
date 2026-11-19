# Source Materials: 9Router Multi-Provider Gateway

This directory contains the upstream source repository of 9Router used for routing layer analysis and failover research.

## Upstream Repository Information

- **Upstream Source:** `https://github.com/decolua/9router.git`
- **Tracked Research Version:** `v0.5.112` (Commit: `1fe586bb`, Branch: `master`)
- **Maintained By:** decolua

## Reproduction Instructions

To clone the exact upstream commit:
```bash
git clone https://github.com/decolua/9router.git 9router
cd 9router
git checkout 1fe586bb
```

*Note: The `9router/` directory is maintained as an independent git repository and is excluded from the root Hime repository via `.gitignore` to prevent repository bloating and subproject commit conflicts. All 14 technical reports and architecture syntheses are tracked in `RE/9ROUTER/`.*
