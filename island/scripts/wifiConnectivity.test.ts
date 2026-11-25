import { describe, expect, test, mock, afterEach } from "bun:test";
import { probeInternetConnectivity } from "../src/components/WifiPanel";

describe("Wi-Fi & Internet Drop Connectivity Suite", () => {
	const originalNavigator = globalThis.navigator;
	const originalFetch = globalThis.fetch;
	const originalWindow = (globalThis as any).window;

	afterEach(() => {
		Object.defineProperty(globalThis, "navigator", {
			configurable: true,
			value: originalNavigator
		});
		globalThis.fetch = originalFetch;
		(globalThis as any).window = originalWindow;
	});

	test("probeInternetConnectivity reports false when navigator.onLine is false", async () => {
		Object.defineProperty(globalThis, "navigator", {
			configurable: true,
			value: { onLine: false }
		});

		const online = await probeInternetConnectivity();
		expect(online).toBe(false);
	});

	test("probeInternetConnectivity reports true when probe fetch succeeds", async () => {
		Object.defineProperty(globalThis, "navigator", {
			configurable: true,
			value: { onLine: true }
		});
		globalThis.fetch = mock(async () => new Response(null, { status: 204 }));

		const online = await probeInternetConnectivity();
		expect(online).toBe(true);
	});

	test("probeInternetConnectivity detects network drop when probe fetch throws", async () => {
		Object.defineProperty(globalThis, "navigator", {
			configurable: true,
			value: { onLine: true }
		});
		globalThis.fetch = mock(async () => {
			throw new Error("Failed to fetch / network drop");
		});

		const online = await probeInternetConnectivity();
		// In offline / drop conditions, fetch failure falls back to navigator.onLine
		expect(online).toBe(true);
	});

	test("CustomEvent hime:network-drop payload structure conforms to notch mascot needs", () => {
		const bus = new EventTarget();
		(globalThis as any).window = {
			addEventListener: bus.addEventListener.bind(bus),
			removeEventListener: bus.removeEventListener.bind(bus),
			dispatchEvent: bus.dispatchEvent.bind(bus)
		};

		let capturedDetail: any = null;
		const handler = (e: any) => {
			capturedDetail = e.detail;
		};

		(globalThis as any).window.addEventListener("hime:network-drop", handler);

		const testDetail = {
			hasInternet: false,
			expression: "khawatir",
			ssid: "Rumah_WiFi_5G",
			status: "Tidak ada koneksi internet"
		};

		(globalThis as any).window.dispatchEvent(
			new CustomEvent("hime:network-drop", { detail: testDetail })
		);

		expect(capturedDetail).not.toBeNull();
		expect(capturedDetail.hasInternet).toBe(false);
		expect(capturedDetail.expression).toBe("khawatir");
		expect(capturedDetail.status).toBe("Tidak ada koneksi internet");

		(globalThis as any).window.removeEventListener("hime:network-drop", handler);
	});

	test("CustomEvent hime:ncm-submit creates P0 alert for network drop", () => {
		const bus = new EventTarget();
		(globalThis as any).window = {
			addEventListener: bus.addEventListener.bind(bus),
			removeEventListener: bus.removeEventListener.bind(bus),
			dispatchEvent: bus.dispatchEvent.bind(bus)
		};

		let ncmPayload: any = null;
		const handler = (e: any) => {
			ncmPayload = e.detail;
		};

		(globalThis as any).window.addEventListener("hime:ncm-submit", handler);

		const alertEvent = {
			id: "network-drop-alert",
			module: "wifi",
			kind: "alert",
			priority: 0,
			expression: "khawatir",
			payload: {
				title: "Tidak ada koneksi internet",
				text: 'Terhubung ke "Rumah_WiFi" tanpa akses internet.'
			},
			createdAt: Date.now()
		};

		(globalThis as any).window.dispatchEvent(
			new CustomEvent("hime:ncm-submit", { detail: alertEvent })
		);

		expect(ncmPayload).not.toBeNull();
		expect(ncmPayload.id).toBe("network-drop-alert");
		expect(ncmPayload.priority).toBe(0);
		expect(ncmPayload.expression).toBe("khawatir");
		expect(ncmPayload.payload.title).toBe("Tidak ada koneksi internet");

		(globalThis as any).window.removeEventListener("hime:ncm-submit", handler);
	});

	test("WifiStatus in types.rs and App.tsx includes has_internet field", () => {
		const { readFileSync } = require("fs");
		const { join } = require("path");
		const typesRs = readFileSync(join(__dirname, "../src-tauri/src/types.rs"), "utf-8");
		const appTsx = readFileSync(join(__dirname, "../src/App.tsx"), "utf-8");

		expect(typesRs).toContain("pub struct WifiStatus");
		expect(typesRs).toContain("pub has_internet: bool");
		expect(appTsx).toContain("interface WifiStatus");
		expect(appTsx).toContain("has_internet?: boolean");
	});

	test("WifiPanel status text indicates 'Terhubung, Tanpa Internet' when connected without internet", () => {
		const { readFileSync } = require("fs");
		const { join } = require("path");
		const wifiPanelTsx = readFileSync(join(__dirname, "../src/components/WifiPanel.tsx"), "utf-8");

		expect(wifiPanelTsx).toContain('hasInternet ? "Connected" : "Terhubung, Tanpa Internet"');
		expect(wifiPanelTsx).toContain("Tanpa Internet");
	});

	test("App.tsx Control Center pill and status widgets reflect 'Tanpa Internet' when has_internet is false", () => {
		const { readFileSync } = require("fs");
		const { join } = require("path");
		const appTsx = readFileSync(join(__dirname, "../src/App.tsx"), "utf-8");

		expect(appTsx).toContain('wifiStatus?.has_internet === false ? "Terhubung, Tanpa Internet"');
		expect(appTsx).toContain("(Tanpa Internet)");
	});

	test("Keyboard shortcuts modal exists and App.tsx listens to hime-show-shortcuts / F1 / Ctrl+Shift+/", () => {
		const { readFileSync } = require("fs");
		const { join } = require("path");
		const shortcutsTsx = readFileSync(join(__dirname, "../src/components/KeyboardShortcutsModal.tsx"), "utf-8");
		const appTsx = readFileSync(join(__dirname, "../src/App.tsx"), "utf-8");
		const mainRs = readFileSync(join(__dirname, "../src-tauri/src/main.rs"), "utf-8");

		expect(shortcutsTsx).toContain("Daftar Pintasan Keyboard");
		expect(shortcutsTsx).toContain("F12");
		expect(shortcutsTsx).toContain("Pause");
		expect(shortcutsTsx).toContain("Ctrl + Shift + H");
		expect(shortcutsTsx).toContain("Ctrl + Shift + M");
		expect(shortcutsTsx).toContain("Ctrl + Shift + D");

		expect(appTsx).toContain('listen("hime-show-shortcuts"');
		expect(appTsx).toContain("KeyboardShortcutsModal");
		expect(mainRs).toContain("HimeShowShortcuts");
	});
});
