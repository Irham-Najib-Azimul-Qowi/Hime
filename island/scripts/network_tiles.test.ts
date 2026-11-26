import { describe, it, expect } from "bun:test";
import { readFileSync } from "fs";
import { resolve } from "path";

describe("Diverse Colors for Active Quick Settings Tiles Suite", () => {
	const appCss = readFileSync(resolve(__dirname, "../src/App.css"), "utf-8");
	const qpCss = readFileSync(resolve(__dirname, "../src/components/QuickPanel.css"), "utf-8");

	it("contains Electric Cyan colors for Wi-Fi (#00B4D8 / #38D6F5)", () => {
		expect(appCss).toContain("#00B4D8");
		expect(appCss).toContain("#38D6F5");
		expect(appCss).toContain(".tile-wifi.active");
		expect(qpCss).toContain(".qp-switch.on.qp-switch--wifi");
		expect(qpCss).toContain("#00B4D8");
		expect(qpCss).toContain("#38D6F5");
	});

	it("contains Royal Indigo/Blue colors for Bluetooth (#4361EE / #3A86FF)", () => {
		expect(appCss).toContain("#4361EE");
		expect(appCss).toContain("#3A86FF");
		expect(appCss).toContain(".tile-bluetooth.active");
		expect(qpCss).toContain(".qp-switch.on.qp-switch--bluetooth");
		expect(qpCss).toContain("#4361EE");
		expect(qpCss).toContain("#3A86FF");
	});

	it("contains Vibrant Amber/Orange colors for Airplane Mode (#F77F00 / #FF9E00)", () => {
		expect(appCss).toContain("#F77F00");
		expect(appCss).toContain("#FF9E00");
		expect(appCss).toContain(".tile-airplane.active");
		expect(appCss).toContain(".cc-circular-btn--airplane.active");
	});

	it("contains Radiant Violet/Magenta colors for Focus / DND (#7209B7 / #B5179E)", () => {
		expect(appCss).toContain("#7209B7");
		expect(appCss).toContain("#B5179E");
		expect(appCss).toContain(".tile-focus.active");
		expect(appCss).toContain(".cc-circular-btn--dnd.active");
	});

	it("contains Emerald / Mint colors for Hotspot (#06D6A0 / #2EC4B6)", () => {
		expect(appCss).toContain("#06D6A0");
		expect(appCss).toContain("#2EC4B6");
		expect(appCss).toContain(".tile-hotspot.active");
		expect(appCss).toContain(".cc-circular-btn--hotspot.active");
	});

	it("contains Lime / Green colors for Battery / Power Saver (#52B788)", () => {
		expect(appCss).toContain("#52B788");
		expect(appCss).toContain(".tile-battery.active");
		expect(appCss).toContain(".cc-circular-btn--battery.active");
	});

	it("contains Golden Sun colors for Night Light (#FFB703)", () => {
		expect(appCss).toContain("#FFB703");
		expect(appCss).toContain(".tile-nightlight.active");
		expect(appCss).toContain(".cc-circular-btn--nightlight.active");
	});

	it("contains Crimson / Coral colors for Sound / Mute (#E63946)", () => {
		expect(appCss).toContain("#E63946");
		expect(appCss).toContain(".tile-sound.active");
		expect(appCss).toContain(".cc-circular-btn--sound.active");
	});
});

describe("Ethernet / LAN Cable Detection Suite", () => {
	const appTsx = readFileSync(resolve(__dirname, "../src/App.tsx"), "utf-8");
	const wifiPanelTsx = readFileSync(resolve(__dirname, "../src/components/WifiPanel.tsx"), "utf-8");
	const wifiRs = readFileSync(resolve(__dirname, "../src-tauri/src/wifi.rs"), "utf-8");
	const commandsRs = readFileSync(resolve(__dirname, "../src-tauri/src/commands.rs"), "utf-8");

	it("detects Ethernet / LAN in Rust backend", () => {
		expect(wifiRs).toContain("pub fn is_ethernet_connected() -> (bool, Option<String>)");
		expect(wifiRs).toContain("IF_TYPE_ETHERNET_CSMACD");
		expect(commandsRs).toContain("is_ethernet_connected");
		expect(commandsRs).toContain("get_ethernet_status");
	});

	it("replaces Wi-Fi icon with Ethernet/LAN icon and updates text in notch / CC", () => {
		expect(appTsx).toContain("isEthernetConnected");
		expect(appTsx).toContain("Terhubung ke LAN");
		expect(appTsx).toContain("<Network");
	});

	it("renders Ethernet / LAN card inside WifiPanel when connected via LAN", () => {
		expect(wifiPanelTsx).toContain("qp-ethernet-card");
		expect(wifiPanelTsx).toContain("Terhubung ke LAN");
		expect(wifiPanelTsx).toContain("Network");
	});
});
