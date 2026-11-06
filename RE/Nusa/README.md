# Nusa Dynamic Island — Reverse Engineering Archive

Direktori ini memuat seluruh materi sumber, artefak ekstraksi, hasil dekompilasi, dan laporan investigasi teknis mendalam terhadap **Nusa Dynamic Island** (`NusaSetup.exe` v1.2.0.0).

---

## Struktur Direktori

- **`00-source-materials/`**: Installer asli (`NusaSetup.exe`, `NusaSetup.zip`) dan direktori hasil ekstraksi biner statis (`extracted/` berisi `nusa.exe`, `install_script.iss`, dan `rdata_strings.txt`).
- **`01-initial-inspection/`**: 
  - `01-initial-reconnaissance.md`: Verifikasi biner, hash SHA-256, metadata PE, signature digital, dan identifikasi compiler Inno Setup.
  - `02-installer-analysis.md`: Analisis dekompilasi script konfigurasi Inno Setup `.iss`, Registry, hak akses `lowest`, dan folder tujuan.
- **`02-binary-analysis/`**:
  - `03-binary-analysis.md`: Analisis biner mendalam, arsitektur PE32+ Rust, seksi biner, crate `windows-rs`, dan eliminasi hipotesis Electron/WPF.
  - `extracted-structure.txt`: Peta struktur seksi biner dan tata letak penyimpanan lokal.
- **`03-application-architecture/`**:
  - `04-runtime-analysis.md`: Observasi runtime proses live, HWND `NotchWindow`, extended styles (`WS_EX_TOPMOST`, `WS_EX_NOREDIRECTIONBITMAP`), multi-monitor, dan lifecycle.
- **`04-ui-and-interaction-analysis/`**:
  - `05-ui-ux-analysis.md`: Dimensi kapsul (idle vs expanded 740×212 px), palet warna OLED, frosted glass backdrop, tipografi `Segoe UI`, dan micro-interactions.
  - `screenshots/`: Direktori artefak visual.
- **`05-dependencies-and-frameworks/`**:
  - `06-feature-analysis.md`: Pemetaan seluruh modul fitur (Media GSMTC, Lirik LRCLIB, File Shelf, Clipboard, To-Do, Pomodoro, Water Tracker, Jadwal Shalat Kemenag RI, HUD).
- **`06-modification-feasibility/`**:
  - `07-modification-feasibility.md`: Evaluasi perbandingan Option A (Source Mod), Option B (Binary Patching), dan Option C (Independent Reimplementation via Tauri 2.0).
- **`findings/`**: Ringkasan poin-poin temuan teknis utama.
- **`reports/`**:
  - `FINAL-REVERSE-ENGINEERING-REPORT.md`: Laporan utama komprehensif master yang merangkum seluruh investigasi.

