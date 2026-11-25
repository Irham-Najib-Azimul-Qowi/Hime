import { describe, expect, it } from "bun:test";
import { readFileSync } from "fs";
import { join } from "path";

describe("Wi-Fi vs Ethernet Auto-Detection and Control Center Toggles Invariants", () => {
	const appTsx = readFileSync(join(__dirname, "../src/App.tsx"), "utf-8");
	const wifiRs = readFileSync(join(__dirname, "../src-tauri/src/wifi.rs"), "utf-8");

	it("wifi.rs filters virtual adapters and validates default gateway", () => {
		// Verify virtual adapter exclusion keywords
		expect(wifiRs).toContain("vethernet");
		expect(wifiRs).toContain("hyper-v");
		expect(wifiRs).toContain("vmware");
		expect(wifiRs).toContain("virtualbox");
		expect(wifiRs).toContain("wsl");
		expect(wifiRs).toContain("tailscale");
		expect(wifiRs).toContain("zerotier");
		expect(wifiRs).toContain("wireguard");

		// Verify GetIpForwardTable2 default gateway validation & hardware/best-route gating
		expect(wifiRs).toContain("GetIpForwardTable2");
		expect(wifiRs).toContain("gateway_if_indices");
		expect(wifiRs).toContain("fwd.DestinationPrefix.PrefixLength == 0");
		expect(wifiRs).toContain("gateway_if_indices.contains(&candidate.interface_index)");
		expect(wifiRs).toContain("should_accept_ethernet_candidate");
		expect(wifiRs).toContain("is_hardware_interface");
		expect(wifiRs).toContain("GetBestInterface");
		expect(wifiRs).toContain("is_wifi_connected");
	});

	it("App.tsx renders icon-only WifiOff with zero speed text when internet/network is down", () => {
		expect(appTsx).toContain("const isInternetDown = isNetworkOffline || wifiStatus?.has_internet === false;");
		expect(appTsx).toContain('<WifiOff size={13} strokeWidth={2} className="net-disconnected-icon" />');
	});

	it("App.tsx prioritizes Wi-Fi icon & SSID over Ethernet on top status widget", () => {
		// When wifiConnected && wifiStatus?.ssid, it shows Wi-Fi icon and SSID
		expect(appTsx).toContain("const isWifi = wifiConnected && !!wifiStatus?.ssid;");
		expect(appTsx).toContain("Wi-Fi (${wifiStatus.ssid}): ↑");
		expect(appTsx).toContain("Wi-Fi: ${wifiStatus.ssid} (↑");
		expect(appTsx).toContain("<Wifi size={12} strokeWidth={2} />");
		expect(appTsx).toContain("<Network size={12} strokeWidth={2} />");
	});

	it("App.tsx Command Center pill displays Wi-Fi when connected to Wi-Fi and Ethernet when on LAN cable", () => {
		// isEthActive is true only when isEthernetConnected AND !wifiConnected
		expect(appTsx).toContain("const isEthActive = isEthernetConnected && !wifiConnected;");
		expect(appTsx).toContain("${isEthActive ? \"is-ethernet\" : \"\"}");
		expect(appTsx).toContain("{isEthActive ? \"Ethernet\" : \"Wi-Fi\"}");
		expect(appTsx).toContain("{isEthActive\n																		? \"Terhubung ke LAN\"");
	});

	it("App.tsx toggleWifi preserves ethernet status in optimistic state", () => {
		expect(appTsx).toContain("is_ethernet: previous?.is_ethernet");
		expect(appTsx).toContain("ethernet_name: previous?.ethernet_name");
	});

	it("All 10 Control Center circular quick toggles have stable backend bindings", () => {
		// 1. DND
		expect(appTsx).toContain('invoke("open_app", { appName: "ms-settings:notifications" })');
		// 2. Battery Saver
		expect(appTsx).toContain('invoke("open_battery_saver_settings")');
		// 3. Airplane Mode
		expect(appTsx).toContain('invoke("open_airplane_mode_settings")');
		// 4. Hotspot
		expect(appTsx).toContain('invoke("open_app", { appName: "ms-settings:network-mobilehotspot" })');
		// 5. Night Light
		expect(appTsx).toContain('invoke("open_app", { appName: "ms-settings:nightlight" })');
		// 6. Sound / Mute
		expect(appTsx).toContain("toggleVolumeLevel");
		// 7. System Tray
		expect(appTsx).toContain('invoke("open_system_tray")');
		// 8. Notifications
		expect(appTsx).toContain('invoke("open_notification_center")');
		// 9. Settings
		expect(appTsx).toContain('invoke("open_settings_window")');
		// 10. Restart
		expect(appTsx).toContain('invoke("restart_bloom")');
	});

	it("All 4 Control Center pill tiles have stable interactive bindings", () => {
		// 1. Wi-Fi / Ethernet pill
		expect(appTsx).toContain("toggleWifi();");
		expect(appTsx).toContain('setBloomMode("wifi")');
		// 2. Dock Mode pill
		expect(appTsx).toContain("toggleDockModeSetting");
		// 3. Bluetooth pill
		expect(appTsx).toContain("toggleBluetooth();");
		expect(appTsx).toContain('setBloomMode("bluetooth")');
		// 4. Notch Mode pill
		expect(appTsx).toContain("toggleNotchModeSetting");
	});
});
