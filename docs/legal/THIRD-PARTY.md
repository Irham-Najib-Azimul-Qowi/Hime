# Third-Party Licenses and Dependencies

This document catalogues the direct dependencies used by Bloom v3.10.2 / Hime Shell.

## 1. Host Project
- **Name**: Bloom
- **Repository**: `https://github.com/little-systems/bloom.git`
- **License**: GNU General Public License v3.0 (`GPL-3.0-only`)
- **Author**: Sehajveer Singh and contributors

## 2. Frontend Dependencies (`package.json`)

| Package | Version | License | Purpose |
|---|---|---|---|
| `react` | `^19.3.0` | MIT | UI Library |
| `react-dom` | `^19.3.0` | MIT | React DOM renderer |
| `framer-motion` | `^12.43.0` | MIT | Physics spring animations & morphing |
| `lucide-react` | `^1.47.0` | ISC | System and action iconography |
| `@tauri-apps/api` | `~2.11.1` | Apache-2.0 / MIT | Tauri v2 frontend IPC bindings |
| `@tauri-apps/plugin-autostart` | `^2.5.1` | Apache-2.0 / MIT | Startup registry persistence |
| `@tauri-apps/plugin-clipboard-manager` | `~2.3.2` | Apache-2.0 / MIT | Clipboard operations |
| `@tauri-apps/plugin-dialog` | `^2.7.3` | Apache-2.0 / MIT | Native system dialogs |
| `@tauri-apps/plugin-opener` | `^2.5.5` | Apache-2.0 / MIT | Open files/URLs in default handlers |
| `@crabnebula/tauri-plugin-drag` | `^2.1.0` | MIT / Apache-2.0 | Native dragging interop |
| `@dnd-kit/core` | `^6.3.1` | MIT | Drag and drop utilities |
| `@dnd-kit/sortable` | `^10.0.0` | MIT | Reorderable lists |
| `@dnd-kit/utilities` | `^3.2.2` | MIT | DND coordinates/transforms |
| `tailwindcss` | `^4.3.3` | MIT | CSS utility styling framework |
| `typescript` | `~5.9.3` | Apache-2.0 | Static typing |
| `vite` | `^7.3.6` | MIT | Bundler and dev server |
| `oxfmt` | `0.70.0` | MIT / Apache-2.0 | Code formatting |

## 3. Rust Backend Dependencies (`src-tauri/Cargo.toml`)

| Crate | Version | License | Purpose |
|---|---|---|---|
| `tauri` | `2.11` | Apache-2.0 / MIT | Application framework shell |
| `windows` | `0.61` | MIT / Apache-2.0 | Win32 / COM / WinRT low-level APIs |
| `windows-core` | `0.61` | MIT / Apache-2.0 | Windows type system bindings |
| `windows-collections` | `0.2` | MIT / Apache-2.0 | Windows WinRT collections |
| `serde` / `serde_json` | `1.0` | MIT / Apache-2.0 | Serialization / Deserialization |
| `window-vibrancy` | `0.7.1` | MIT / Apache-2.0 | Acrylic / Mica blur effects |
| `tokio` | `1.0` | MIT | Asynchronous runtime |
| `webview2-com` | `0.38` | MIT / Apache-2.0 | Edge WebView2 COM interop |
| `image` | `0.25` | MIT / Apache-2.0 | Image thumbnail decoding |
| `zip` | `9.0.5` | MIT | Archive decompression (shelf preview) |
| `wmi` | `0.14` | MIT / Apache-2.0 | Windows Management Instrumentation |
| `rustfft` | `6.0` | MIT / Apache-2.0 | Fast Fourier Transform for audio visualizer |
| `base64` | `0.22` | MIT / Apache-2.0 | Base64 byte conversions |
| `tauri-plugin-updater` | `2.0` | MIT / Apache-2.0 | Auto-updater (to be neutralized for Hime) |

