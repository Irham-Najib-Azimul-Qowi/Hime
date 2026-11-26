import { describe, expect, test } from "bun:test";
import { readFileSync } from "fs";
import { join } from "path";
import {
	isSameApp,
	isBrowserHost,
	isBrowserPwa,
	appIdentity,
	itemKey,
	formatWindowTitle,
	type AppInfo
} from "../src/dockApps";

describe("Volume & Brightness Edge Overlays (no collision / clean replacement)", () => {
	const overlayCss = readFileSync(join(__dirname, "../src/Overlay.css"), "utf-8");
	const overlayTsx = readFileSync(join(__dirname, "../src/Overlay.tsx"), "utf-8");

	test("Volume notch hides icon and mixer icon when showing percentage", () => {
		expect(overlayCss).toContain(".volume-notch-action.showing-percent .volume-notch-action-icon");
		expect(overlayCss).toContain("opacity: 0 !important;");
		expect(overlayCss).toContain(".volume-notch-action.showing-percent .volume-notch-action-percent");
		expect(overlayCss).toContain("opacity: 1 !important;");
	});

	test("Brightness notch has showPercentage state and replaces icon cleanly", () => {
		expect(overlayTsx).toContain("const [showPercentage, setShowPercentage] = useState(false);");
		expect(overlayTsx).toContain("brightness-notch-action");
		expect(overlayTsx).toContain("brightness-notch-action-icon");
		expect(overlayTsx).toContain("brightness-notch-action-percent");
		expect(overlayCss).toContain(".brightness-notch-action.showing-percent .brightness-notch-action-icon");
		expect(overlayCss).toContain(".brightness-notch-action.showing-percent .brightness-notch-action-percent");
	});
});

describe("Dock Pinned App Window Merging & Active Indicator", () => {
	const dockCss = readFileSync(join(__dirname, "../src/Dock.css"), "utf-8");
	const dockTsx = readFileSync(join(__dirname, "../src/Dock.tsx"), "utf-8");

	test("Active indicator dot is removed in CSS and JSX per user request", () => {
		expect(dockCss).toContain(".active-indicator,\n.window-dots {\n\tdisplay: none !important;");
		expect(dockTsx).toContain("function WindowDots(");
		expect(dockTsx).toContain("return null;");
	});

	test("Pinned Chrome merges cleanly with running Chrome window", () => {
		const pinnedChrome: AppInfo = {
			name: "Google Chrome",
			path: "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe",
			icon: null,
			is_running: false,
			is_pinned: true,
			executable: "chrome.exe"
		};

		const runningChromeWindow: AppInfo = {
			name: "Google Chrome",
			path: "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe",
			icon: null,
			is_running: true,
			hwnd: 65538,
			executable: "chrome.exe",
			window_title: "GitHub - Hime - Google Chrome",
			all_hwnds: [[65538, "GitHub - Hime - Google Chrome"]]
		};

		expect(isSameApp(pinnedChrome, runningChromeWindow)).toBe(true);
	});

	test("Pinned Chrome shortcut (.lnk) merges with running chrome.exe", () => {
		const pinnedShortcut: AppInfo = {
			name: "Google Chrome",
			path: "C:\\ProgramData\\Microsoft\\Windows\\Start Menu\\Programs\\Google Chrome.lnk",
			icon: null,
			is_running: false,
			is_pinned: true,
			executable: "chrome.exe"
		};

		const runningChrome: AppInfo = {
			name: "Google Chrome",
			path: "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe",
			icon: null,
			is_running: true,
			hwnd: 70001,
			executable: "chrome.exe",
			window_title: "New Tab - Google Chrome"
		};

		expect(isSameApp(pinnedShortcut, runningChrome)).toBe(true);
	});

	test("Pinned Chrome does NOT merge with an independent PWA (e.g. YouTube PWA)", () => {
		const pinnedChrome: AppInfo = {
			name: "Google Chrome",
			path: "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe",
			icon: null,
			is_running: false,
			is_pinned: true,
			executable: "chrome.exe"
		};

		const runningPwa: AppInfo = {
			name: "YouTube",
			path: "chrome._crx_agimnkijcaieampoomlpimhhpkflimdn",
			icon: null,
			is_running: true,
			hwnd: 88888,
			executable: "chrome.exe",
			window_title: "YouTube"
		};

		expect(isBrowserPwa(runningPwa)).toBe(true);
		expect(isBrowserPwa(pinnedChrome)).toBe(false);
		expect(isSameApp(pinnedChrome, runningPwa)).toBe(false);
	});

	test("Pinned PWA merges with its own running PWA window", () => {
		const pinnedPwa: AppInfo = {
			name: "YouTube",
			path: "chrome._crx_agimnkijcaieampoomlpimhhpkflimdn",
			icon: null,
			is_running: false,
			is_pinned: true,
			executable: "chrome.exe"
		};

		const runningPwa: AppInfo = {
			name: "YouTube",
			path: "chrome._crx_agimnkijcaieampoomlpimhhpkflimdn",
			icon: null,
			is_running: true,
			hwnd: 99999,
			executable: "chrome.exe",
			window_title: "Lo-Fi Beats - YouTube"
		};

		expect(isSameApp(pinnedPwa, runningPwa)).toBe(true);
	});

	test("Multi-window individual icons: each window gets its own separate icon in the dock", () => {
		const pinnedChrome: AppInfo = {
			name: "Google Chrome",
			path: "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe",
			icon: null,
			is_running: false,
			is_pinned: true,
			executable: "chrome.exe"
		};

		const win1: AppInfo = {
			name: "Google Chrome",
			path: "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe",
			icon: null,
			is_running: true,
			hwnd: 101,
			executable: "chrome.exe",
			window_title: "Page 1 - Google Chrome",
			all_hwnds: [[101, "Page 1 - Google Chrome"]]
		};

		const win2: AppInfo = {
			name: "Google Chrome",
			path: "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe",
			icon: null,
			is_running: true,
			hwnd: 102,
			executable: "chrome.exe",
			window_title: "Page 2 - Google Chrome",
			all_hwnds: [[102, "Page 2 - Google Chrome"]]
		};

		const activeApps = [win1, win2];
		const matchedRunningKeys = new Set<string>();

		const findFirstRunningApp = (p: AppInfo): AppInfo | undefined => {
			const found = activeApps.find(
				(app) => !matchedRunningKeys.has(itemKey(app)) && isSameApp(p, app)
			);
			if (found) {
				matchedRunningKeys.add(itemKey(found));
				return found;
			}
			return undefined;
		};

		// Pinned Chrome matches Window 1
		const runningFirst = findFirstRunningApp(pinnedChrome);
		expect(runningFirst?.hwnd).toBe(101);
		expect(matchedRunningKeys.has(itemKey(win1))).toBe(true);

		// Window 2 is NOT claimed by the pinned icon, so it remains in unpinned items!
		expect(matchedRunningKeys.has(itemKey(win2))).toBe(false);

		const unpinned = activeApps.filter((a) => !matchedRunningKeys.has(itemKey(a)));
		// Window 2 has its OWN separate icon in the dock!
		expect(unpinned.length).toBe(1);
		expect(unpinned[0].hwnd).toBe(102);
	});
});

describe("Dock Hover Window Preview Title & Snappy Speed", () => {
	const dockTsx = readFileSync(join(__dirname, "../src/Dock.tsx"), "utf-8");
	const dockCss = readFileSync(join(__dirname, "../src/Dock.css"), "utf-8");

	test("Dock preview displays actual window title from prev.title or app.window_title", () => {
		expect(dockTsx).toContain("prev.title || app.window_title || app.title || app.name");
		expect(dockTsx).toContain("get_window_title");
	});

	test("Dock hover preview timer is snappy (~60-100ms, set to 80ms)", () => {
		expect(dockTsx).toContain("}, 80);");
	});

	test("formatWindowTitle truncates titles exceeding 30 characters and appends ellipsis", () => {
		expect(formatWindowTitle("Short Title")).toBe("Short Title");
		expect(formatWindowTitle("Exact 30 chars - 1234567890123")).toBe("Exact 30 chars - 1234567890123");
		expect(formatWindowTitle("This is a very long window title that exceeds 30 characters")).toBe("This is a very long window ...");
		expect(formatWindowTitle("   GitHub - Hime - Google Chrome - Issues and PRs   ")).toBe("GitHub - Hime - Google Chro...");
		expect(formatWindowTitle(null)).toBe("");
		expect(formatWindowTitle(undefined)).toBe("");
	});

	test("Dock.css enforces ellipsis and max-width on .preview-label", () => {
		expect(dockCss).toContain(".preview-label {");
		expect(dockCss).toContain("max-width: 180px;");
		expect(dockCss).toContain("text-overflow: ellipsis;");
		expect(dockCss).toContain("overflow: hidden;");
		expect(dockCss).toContain("white-space: nowrap;");
	});
});
