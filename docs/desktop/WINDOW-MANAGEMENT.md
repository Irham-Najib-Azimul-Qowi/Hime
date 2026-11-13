# Hime Desktop Experience — Window Management Specification

- **Document ID:** DOCS-DESKTOP-WINDOW-MANAGEMENT
- **Phase:** Phase 07 — Hime Desktop Experience
- **Authority:** System Architect & Technical Lead
- **Status:** APPROVED IMPLEMENTATION SPECIFICATION

---

## 1. Native Windows Integration & Tauri Window API

Hime Desktop operates on Windows 11 with specific window management requirements to accommodate both compact companion workflows and rich desktop management.

### Window Geometry Presets

| Mode | Width | Height | Min Dimensions | Purpose |
|:---|:---:|:---:|:---:|:---|
| **Expanded Dashboard** | 980 px | 700 px | 440 × 88 px | Full system control panel, activity stream, chat, and task board |
| **Compact Floating HUD** | 460 px | 88 px | 440 × 88 px | Minimal floating companion centered at the top of the screen |

---

## 2. Window Control Commands

Tauri 2 provides native commands implemented in `apps/hime-desktop/src-tauri/src/main.rs`:

```rust
#[command]
async fn window_set_always_on_top(window: tauri::Window, always_on_top: bool) -> Result<(), String> {
    window.set_always_on_top(always_on_top).map_err(|e| e.to_string())
}

#[command]
async fn window_set_size_mode(window: tauri::Window, mode: String) -> Result<(), String> {
    match mode.as_str() {
        "compact" => window.set_size(Size::Logical(LogicalSize { width: 460.0, height: 88.0 })),
        "expanded" => window.set_size(Size::Logical(LogicalSize { width: 980.0, height: 700.0 })),
        _ => Err(format!("Unknown mode: {}", mode)),
    }
}

#[command]
async fn window_minimize(window: tauri::Window) -> Result<(), String> { window.minimize().map_err(|e| e.to_string()) }

#[command]
async fn window_maximize(window: tauri::Window) -> Result<(), String> {
    if window.is_maximized().map_err(|e| e.to_string())? { window.unmaximize().map_err(|e| e.to_string()) }
    else { window.maximize().map_err(|e| e.to_string()) }
}

#[command]
async fn window_close(window: tauri::Window) -> Result<(), String> { window.close().map_err(|e| e.to_string()) }
```

---

## 3. Dragging & Hit-Testing

- **Titlebar Header (`.app-drag-region`):** Uses `-webkit-app-region: drag` so users can click and move the window anywhere across multi-monitor setups.
- **Interactive Controls (`.app-no-drag`):** Buttons, Dynamic Island, and inputs within the titlebar use `-webkit-app-region: no-drag` ensuring clicks, keyboard inputs, and hover states register accurately without triggering window moves.
- **Focus Management:** Modals (Omnibox, Task Creation) automatically capture focus upon opening and release focus upon `Escape` or backdrop click.

