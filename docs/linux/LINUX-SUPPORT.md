# HIME LINUX SUPPORT SPECIFICATION & ARCHITECTURE
## Cross-Platform Linux Beta (Closing G10)

**Otoritas:** System Architect & Cross-Platform Lead  
**Pelaksana:** Principal Software Engineer & AI Executor  
**Status:** **Linux Beta (v1.0 RC)**  
**Tanggal:** 2026-11-14  

---

## 1. VISION: CROSS-PLATFORM BY DESIGN

While Windows is Hime's primary development and runtime platform, Hime was architected from inception to be **cross-platform by design**. All core intelligence logic, storage schemas, policy engines, and communication protocols are isolated in platform-agnostic Rust crates. Platform-specific behaviors are strictly encapsulated within the Platform Abstraction Layer (`hime-pal`).

This specification documents the Linux architecture, implementation details, packaging strategies, and supported desktop environments.

---

## 2. LINUX PAL IMPLEMENTATION DETAILS

### 2.1 Filesystem Paths (XDG Base Directory Specification)
On Linux, Hime strictly adheres to the Freedesktop XDG standard:
- **Configuration:** `$XDG_CONFIG_HOME/hime` (default: `~/.config/hime`)
- **Data & Databases:** `$XDG_DATA_HOME/hime` (default: `~/.local/share/hime`)
- **Cache & Profiles:** `$XDG_CACHE_HOME/hime` (default: `~/.cache/hime`)
- **Runtime Sockets & Locks:** `$XDG_RUNTIME_DIR/hime` (default: `/run/user/<UID>/hime` or `/tmp/hime`)

### 2.2 IPC Transport (Unix Domain Sockets)
Windows Named Pipes (`\\.\pipe\hime-ipc`) map directly to Unix Domain Sockets (`hime-core.sock`):
- Socket paths are created inside `$XDG_RUNTIME_DIR/hime/`.
- Sockets are bound with strict `0o600` permissions (read/write by owner only; group and other access denied).
- Preallocation checks and binary framing remain identical across platforms.

### 2.3 Secret Vault (Software-Encrypted Vault & Secret Service)
Because Windows DPAPI (`CryptProtectData`) is not available on Linux:
- Hime provides `SoftwareEncryptedVault` using SHA-256 CTR keystream encryption with user/machine seed isolation.
- Decrypted secrets are wrapped in `ProtectedSecret` with automatic memory zeroization on drop.
- Linux keyring integration hooks support Freedesktop Secret Service (`libsecret`).

### 2.4 Process Supervision & Isolation
On Linux, kernel Job Objects are replaced by Unix process groups:
- Managed child processes are spawned into new process groups (`setpgid`).
- Coordinated termination delivers `SIGTERM` followed by `SIGKILL` to the entire process group upon daemon shutdown, ensuring zero orphaned processes.

---

## 3. DESKTOP ENVIRONMENT & COMPOSITOR BEHAVIOR

| Feature | X11 (X.Org) | Wayland (GNOME / KDE) | Notes & Graceful Degradation |
|:---|:---:|:---:|:---|
| **Main Control Panel** | Full Support | Full Support | Standard application window via Tauri 2.0 WebKitGTK. |
| **Omnibox (Ctrl+K)** | Full Support | Full Support | Global shortcut or app-focused trigger. |
| **Dynamic Island Floating HUD** | Full Support | Degraded (Panel / Compact) | X11 allows arbitrary window coordinate placement (`x, y`) and `always-on-top`. Wayland compositors restrict arbitrary coordinate placement; on Wayland, Hime defaults to a compact top-anchored window. |
| **Always-On-Top** | Supported | Compositor Dependent | Works out-of-the-box on KDE Wayland; GNOME Wayland requires user focus or extension for persistent top-most overlay. |

---

## 4. PACKAGING & DISTRIBUTION

Hime for Linux is packaged through standard Linux formats:
1. **AppImage:** Single executable bundle containing Tauri shell, WebKitGTK bindings, and embedded core daemon.
2. **Debian Package (`.deb`):** Targets Ubuntu 22.04 LTS, Ubuntu 24.04 LTS, and Debian 12. Installs desktop entry, icons, and binaries to `/usr/bin/hime` and `/usr/lib/hime/`.
3. **RPM Package (`.rpm`):** Targets Fedora 39/40 and openSUSE.
4. **Tarball (`.tar.gz`):** Portable standalone distribution for power users and custom distributions.

---

## 5. FEATURE PARITY MATRIX

| Hime Capability | Windows 11 Status | Linux Status |
|:---|:---:|:---:|
| **Orchestration Core Daemon** | Full Support | Full Support |
| **Hermes Agent & 9Router** | Full Support | Full Support |
| **SQLite WAL Storage Engine** | Full Support | Full Support |
| **Tool Gateway & PDP/PEP** | Full Support | Full Support |
| **SSRF-Safe Web Browser** | Full Support | Full Support |
| **Developer Workspace (Git/Runner)** | Full Support | Full Support |
| **Specialist Agents & Data/AI** | Full Support | Full Support |
| **Mobile Companion mTLS** | Full Support | Full Support |
| **Dynamic Island HUD** | Full Support | Beta (X11 Full / Wayland Compact) |

---

## 6. CONCLUSION

Hime achieves genuine cross-platform readiness. Linux support is classified as **Beta** for v1.0 RC with documented compositor differences and zero broken architectural abstractions.
