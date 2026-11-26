import { describe, it, expect } from "bun:test";
import { readFileSync, existsSync } from "fs";
import { resolve } from "path";

describe("Settings Toggles & Draggable Repositioning (Notch & Overlays)", () => {
	const islandDir = resolve(__dirname, "..");
	const appTsx = readFileSync(resolve(islandDir, "src/App.tsx"), "utf-8");
	const appCss = readFileSync(resolve(islandDir, "src/App.css"), "utf-8");
	const overlayTsx = readFileSync(resolve(islandDir, "src/Overlay.tsx"), "utf-8");
	const overlayCss = readFileSync(resolve(islandDir, "src/Overlay.css"), "utf-8");
	const useSettingsTs = readFileSync(resolve(islandDir, "src/settings/useSettings.ts"), "utf-8");
	const notchTabTsx = readFileSync(resolve(islandDir, "src/settings/NotchTab.tsx"), "utf-8");
	const overlaysTabTsx = readFileSync(resolve(islandDir, "src/settings/OverlaysTab.tsx"), "utf-8");
	const settingsTsx = readFileSync(resolve(islandDir, "src/Settings.tsx"), "utf-8");
	const servicesRs = readFileSync(resolve(islandDir, "src-tauri/src/services.rs"), "utf-8");

	describe("1. Settings Hook & Tabs Integration", () => {
		it("declares freeDragEnabled and assistantEnabled in useSettings with appropriate defaults", () => {
			expect(useSettingsTs).toContain("bloom-free-drag-enabled");
			expect(useSettingsTs).toContain("bloom-assistant-enabled");
			expect(useSettingsTs).toContain("const [freeDragEnabled, setFreeDragEnabled] = useState(");
			expect(useSettingsTs).toContain("const [assistantEnabled, setAssistantEnabled] = useState(");
			// Default for freeDragEnabled is false, assistantEnabled is true
			expect(useSettingsTs).toContain("localStorage.getItem(\"bloom-free-drag-enabled\") === \"true\"");
			expect(useSettingsTs).toContain("localStorage.getItem(\"bloom-assistant-enabled\") !== \"false\"");
			// Exports setters/togglers
			expect(useSettingsTs).toContain("toggleFreeDrag");
			expect(useSettingsTs).toContain("toggleAssistant");
		});

		it("renders Free Position / Draggable Notch toggle in NotchTab", () => {
			expect(notchTabTsx).toContain("Free Position / Draggable Notch");
			expect(notchTabTsx).toContain("freeDragEnabled");
			expect(notchTabTsx).toContain("toggleFreeDrag");
		});

		it("renders Hime AI Assistant toggle in NotchTab", () => {
			expect(notchTabTsx).toContain("Hime AI Assistant");
			expect(notchTabTsx).toContain("assistantEnabled");
			expect(notchTabTsx).toContain("toggleAssistant");
		});

		it("renders Replace Windows Notifications & Calendar toggle in NotchTab and wires it in useSettings/Settings", () => {
			expect(useSettingsTs).toContain("bloom-suppress-native-notifications");
			expect(useSettingsTs).toContain("suppressNativeNotifications");
			expect(useSettingsTs).toContain("toggleSuppressNativeNotifications");
			expect(notchTabTsx).toContain("Replace Windows Notifications & Calendar");
			expect(settingsTsx).toContain("suppressNativeNotifications={settings.suppressNativeNotifications}");
			expect(settingsTsx).toContain("toggleSuppressNativeNotifications={settings.toggleSuppressNativeNotifications}");
		});

		it("renders Free Position / Draggable Overlays toggle in OverlaysTab", () => {
			expect(overlaysTabTsx).toContain("Free Position / Draggable Overlays");
			expect(overlaysTabTsx).toContain("freeDragEnabled");
			expect(overlaysTabTsx).toContain("toggleFreeDrag");
		});

		it("passes free drag and assistant props in Settings.tsx", () => {
			expect(settingsTsx).toContain("freeDragEnabled={settings.freeDragEnabled}");
			expect(settingsTsx).toContain("toggleFreeDrag={settings.toggleFreeDrag}");
			expect(settingsTsx).toContain("assistantEnabled={settings.assistantEnabled}");
			expect(settingsTsx).toContain("toggleAssistant={settings.toggleAssistant}");
		});
	});

	describe("2. Draggable Notch in App.tsx & App.css", () => {
		it("manages freeDragEnabled and notchPosition state with localStorage/settings sync", () => {
			expect(appTsx).toContain("const [freeDragEnabled, setFreeDragEnabled] = useState(");
			expect(appTsx).toContain("const [notchPosition, setNotchPosition] = useState<{ x: number; y: number } | null>(");
			expect(appTsx).toContain("bloom-notch-position");
			expect(appTsx).toContain("bloom-free-drag-enabled");
		});

		it("implements pointer drag handling for notch repositioning", () => {
			expect(appTsx).toContain("handleNotchPointerDown");
			expect(appTsx).toContain("setIsDragging");
			expect(appTsx).toContain("bloom-notch-position");
		});

		it("wraps .bloom in .notch-drag-wrapper with is-free-drag and is-floating classes", () => {
			expect(appTsx).toContain("notch-drag-wrapper");
			expect(appTsx).toContain("is-free-drag");
			expect(appTsx).toContain("is-floating");
			expect(appCss).toContain(".notch-drag-wrapper");
			expect(appCss).toContain(".notch-drag-wrapper.is-free-drag");
			expect(appCss).toContain(".bloom.is-floating");
		});

		it("keeps all notch features functional (pill target & window region calculations account for custom position)", () => {
			expect(appTsx).toContain("pillTargetRef");
			expect(appTsx).toContain("notchPosition");
			expect(appTsx).toContain("updateNotchRegion");
		});
	});

	describe("3. Hime AI Assistant Panel & Mascot Dialogue Suppression", () => {
		it("manages settingsAssistantEnabled in App.tsx", () => {
			expect(appTsx).toContain("settingsAssistantEnabled");
			expect(appTsx).toContain("bloom-assistant-enabled");
		});

		it("filters out hime and tasks panels when assistant is disabled", () => {
			// panelEnabled helper checks settingsAssistantEnabled for hime and tasks
			expect(appTsx).toContain("if (panel === \"hime\") return settingsAssistantEnabled;");
			expect(appTsx).toContain("if (panel === \"tasks\") return settingsAssistantEnabled;");
		});

		it("suppresses AI approvals and periodic assistant glances when assistant is disabled", () => {
			expect(appTsx).toContain("approvals: settingsAssistantEnabled ? himeApprovals : emptyApprovals");
			expect(appTsx).toContain("himeApprovals.length > 0 && !isOpen && settingsAssistantEnabled");
			expect(appTsx).toContain("!isOpen && !isHovered && !isCenterHovered && settingsAssistantEnabled");
		});
	});

	describe("4. Draggable Overlays in Overlay.tsx & Overlay.css", () => {
		it("manages freeDragEnabled, volumePosition, and brightnessPosition in OverlayApp", () => {
			expect(overlayTsx).toContain("const [freeDragEnabled, setFreeDragEnabled] = useState(");
			expect(overlayTsx).toContain("const [volumePosition, setVolumePosition] = useState<");
			expect(overlayTsx).toContain("const [brightnessPosition, setBrightnessPosition] = useState<");
			expect(overlayTsx).toContain("bloom-volume-overlay-position");
			expect(overlayTsx).toContain("bloom-brightness-overlay-position");
		});

		it("provides overlay drag handles and pointer move handlers in VolumeNotch and BrightnessNotch", () => {
			expect(overlayTsx).toContain("overlay-drag-handle");
			expect(overlayTsx).toContain("handleCardPointerDown");
			expect(overlayTsx).toContain("onPositionChange");
			expect(overlayTsx).toContain("onDragEnd");
		});

		it("renders edge-docked and dragging styling in Overlay.css for draggable overlays", () => {
			expect(overlayCss).toContain(".volume-notch-wrapper.is-edge-left");
			expect(overlayCss).toContain(".brightness-notch-wrapper.is-edge-right");
			expect(overlayCss).toContain(".volume-notch-wrapper.is-edge-top");
			expect(overlayCss).toContain(".volume-notch-wrapper.is-dragging");
			expect(overlayCss).toContain(".brightness-notch-wrapper.is-dragging");
			expect(overlayCss).toContain(".overlay-drag-handle");
		});

		it("includes active position rectangles in sendOverlayRegion for click-through support", () => {
			expect(overlayTsx).toContain("activeVolumePosition");
			expect(overlayTsx).toContain("activeBrightnessPosition");
			expect(overlayTsx).toContain("customRects");
		});
	});

	describe("5. Rust Low-Level Mouse Hook & Window Sizing in services.rs", () => {
		it("includes r.y translation in low-level mouse hook notch hit-testing", () => {
			expect(servicesRs).toContain("win_pos.y + (r.y as f64 * scale) as i32");
		});

		it("expands main appbar reservation height to screen height when free drag is enabled", () => {
			expect(servicesRs).toContain("bloom-free-drag-enabled");
			expect(servicesRs).toContain("m_size.height as i32");
		});
	});
});
