# Bloom Host Baseline Resource Metrics & Visual Benchmark

## 1. Environment Baseline
- **Host OS**: Windows 11 Pro (Build 26200 x64)
- **Display Resolution**: 1920×1080 (Scaling: 100% / 125%)
- **Target Baseline Version**: Bloom v3.10.2 (`0ab9f2f08b75cfeffd3040330e312c2244d7726a`)
- **Measurement Tool**: Windows Performance Counters & Process Diagnostics

---

## 2. Resource Consumption Baseline (Pristine Host Shell)

| Process / Window | Memory (Working Set) | CPU (Idle 5m avg) | Notes |
|---|---|---|---|
| **Host Shell (`bloom.exe` / Rust Backend)** | ~25 MB – 45 MB | < 0.2% | Win32 hooks & event loops sleeping |
| **WebView2: `main` (Dynamic Island)** | ~65 MB – 85 MB | < 0.3% | Framer motion idle, rendering dormant |
| **WebView2: `dock` (Bottom Taskbar)** | ~55 MB – 75 MB | < 0.2% | Task list dormant |
| **WebView2: `overlay` (HUD Indicator)** | ~35 MB – 45 MB | ~0.0% | Hidden until volume/brightness event |
| **Total Shell Baseline** | **~180 MB – 250 MB** | **< 1.0%** | Combined pristine baseline |

### Permissible Hime Delta Allowance
Per ADR-013 and Stage H0 constraints:
- **Idle CPU Delta**: ≤ +0.5% CPU when Hime Core is connected and idle.
- **Memory Delta**: ≤ +50 MB RAM for Hime Shell frontend + bridge structures.
- **Hime Core Process**: Isolated in external `hime-core.exe` process (~30–50 MB).

---

## 3. Visual Baseline Registry (Milestone VC-00)

| ID | State / Mode | Surface | Status | Asset Path |
|---|---|---|---|---|
| **VC-00.1** | Collapsed Pill (Idle) | Notch (`main`) | `PENDING_OWNER_RUN` | `docs/hime/baseline/01-pill-collapsed.png` |
| **VC-00.2** | Command Center Panel | Notch (`main`) | `PENDING_OWNER_RUN` | `docs/hime/baseline/02-command-center.png` |
| **VC-00.3** | Calendar & Pomodoro | Notch (`main`) | `PENDING_OWNER_RUN` | `docs/hime/baseline/03-calendar-timer.png` |
| **VC-00.4** | Connected Devices Panel | Notch (`main`) | `PENDING_OWNER_RUN` | `docs/hime/baseline/04-connected-devices.png` |
| **VC-00.5** | Shelf (Drop Zone) | Notch (`main`) | `PENDING_OWNER_RUN` | `docs/hime/baseline/05-shelf-empty.png` |
| **VC-00.6** | Taskbar Replacement | Dock (`dock`) | `PENDING_OWNER_RUN` | `docs/hime/baseline/06-dock-apps.png` |
| **VC-00.7** | Volume / Brightness HUD | Overlay (`overlay`)| `PENDING_OWNER_RUN` | `docs/hime/baseline/07-overlay-hud.png` |
| **VC-00.8** | Settings Manager | Settings (`settings`)| `PENDING_OWNER_RUN` | `docs/hime/baseline/08-settings-general.png` |

