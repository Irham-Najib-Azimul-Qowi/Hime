# Source Materials: Nusa Dynamic Island

This directory contains the original binary artifacts and extracted assets used for the reverse engineering of Nusa Dynamic Island.

## Artifact Inventory

- **Installer:** `NusaSetup.exe` (SHA-256 verified in `RE/Nusa/01-initial-inspection/01-initial-reconnaissance.md`)
- **Distribution Archive:** `NusaSetup.zip`
- **Extracted Application Directory:** `extracted/` (Unpacked via `innounp 2.70.1` Inno Setup unpacker)

## Reproduction Instructions

To acquire and unpack the source materials independently:
1. Download `NusaSetup.exe` from the official repository release page: `https://rama2555.github.io/nusa/`
2. Extract the Inno Setup container:
   ```cmd
   innounp.exe -x -dextracted NusaSetup.exe
   ```

*Note: Binary installers (`.exe`, `.zip`) and raw extracted binary files are excluded from git version control via `.gitignore`. Analysis reports, extraction indexes (`extracted-structure.txt`), and architecture documents are tracked in `RE/Nusa/`.*
