# HIME INSTALLATION & SETUP GUIDE
## Release Candidate v1.0 User Guide

Welcome to **Hime (Personal AI Desktop Environment)**. This guide walks you through system requirements, installation steps, and first-time configuration.

---

## 1. SYSTEM REQUIREMENTS

### Minimum Specifications
- **Operating System:** Windows 10 (version 1903+) or Windows 11 64-bit; Linux (Ubuntu 22.04+, Fedora 39+, Debian 12+).
- **Processor:** Intel Core i3 / AMD Ryzen 3 (or equivalent dual-core/quad-core 2.0 GHz+).
- **Memory (RAM):** 4 GB RAM minimum (8 GB recommended for local indexing and ML experiment design).
- **Disk Space:** 500 MB free space (NVMe or SATA SSD recommended).
- **Graphics:** DirectX 11 / DirectComposition compatible GPU (Intel UHD 620 or higher).

### Optional Mobile Companion
- **Mobile OS:** Android 10 (API 29) or higher.
- **Connectivity:** Local Wi-Fi network (same subnet as desktop PC for mTLS pairing).

---

## 2. WINDOWS INSTALLATION

Hime provides two distribution formats for Windows:

### Option A: Per-User Setup Installer (Recommended)
1. Download `Hime-Setup-1.0.0.exe`.
2. Double-click the installer. 
   > [!NOTE]
   > The installer installs strictly to `%LOCALAPPDATA%\Programs\Hime\` and **does not require administrator privileges**.
3. If Windows SmartScreen displays a warning ("Windows protected your PC"), click **More info** and select **Run anyway** (this occurs because official binary code-signing requires the Project Owner's commercial certificate).
4. Follow the setup wizard to create Desktop and Start Menu shortcuts.
5. Hime will launch automatically upon completion.

### Option B: Portable Distribution (.zip)
1. Download `Hime-Portable-1.0.0.zip`.
2. Extract the archive to any folder (e.g. `C:\Tools\Hime` or `%LOCALAPPDATA%\Hime`).
3. Run `Hime.exe` directly. All application data will be stored in `%APPDATA%\Hime` and `%LOCALAPPDATA%\Hime`.

---

## 3. LINUX INSTALLATION (BETA)

### Option A: AppImage (Universal Linux)
1. Download `Hime-1.0.0.AppImage`.
2. Make the file executable:
   ```bash
   chmod +x Hime-1.0.0.AppImage
   ```
3. Run `./Hime-1.0.0.AppImage`.

### Option B: Debian / Ubuntu Package (.deb)
1. Download `hime_1.0.0_amd64.deb`.
2. Install via apt:
   ```bash
   sudo apt install ./hime_1.0.0_amd64.deb
   ```
3. Launch Hime from your application menu or run `hime` in terminal.

---

## 4. FIRST-RUN CONFIGURATION

Upon launching Hime for the first time:
1. **Local Security & Storage Setup:**
   - Hime automatically initializes its secure local SQLite database at `%APPDATA%\Hime\hime.db` (or `~/.local/share/hime/hime.db` on Linux) with Write-Ahead Logging.
   - Hardware-backed Windows DPAPI vault is generated for API keys and pairing secrets.
2. **Dynamic Island HUD:**
   - The floating HUD appears docked at the top center of your primary monitor.
   - Press `Ctrl+K` to open the Omnibox command palette.
   - Press `Alt+1` through `Alt+9` to switch between main views.
3. **Configuring AI Model Providers:**
   - Press `Ctrl+K` and type `Settings` (or navigate to Provider Configuration).
   - Enter your API keys (OpenAI, Anthropic, Google, OpenRouter). Keys are encrypted immediately into DPAPI; zero plaintext keys are saved.
4. **Pairing Mobile Device (Optional):**
   - Press `Alt+4` (Notification Center) or open Devices tab.
   - Click **Pair Mobile Device** to generate a single-use, time-limited pairing code and QR code.
   - In the Hime Android Companion app, scan the QR code to establish mutual TLS (mTLS) pairing.

---

## 5. UNINSTALLATION & DATA CLEANUP

- To uninstall on Windows: Open **Windows Settings > Installed apps**, locate **Hime Assistant**, and click **Uninstall**.
- To remove all stored local data, delete `%APPDATA%\Hime` and `%LOCALAPPDATA%\Hime`.
