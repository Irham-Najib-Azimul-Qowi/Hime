export interface AppInfo {
	name: string;
	path: string;
	icon: string | null;
	is_running: boolean;
	is_pinned?: boolean;
	hwnd?: number;
	executable?: string;
	all_hwnds?: [number, string][];
	tray_ids?: string[];
	is_background?: boolean;
	// A pin keeps its display name, but its icon is cached under the live name.
	icon_name?: string;
	icon_path?: string;
	window_title?: string;
	title?: string;
}

export interface TrayApp {
	name: string;
	path: string;
	tray_ids: string[];
	window_handles: number[];
}

export const HOST_PROCESSES = ["msedge.exe", "chrome.exe", "brave.exe", "applicationframehost.exe"];

export function isBrowserHost(path: string) {
	const p = path.toLowerCase();
	return HOST_PROCESSES.some((host) => p.includes(host));
}

export function isBrowserPwa(app: Pick<AppInfo, "path" | "name">): boolean {
	const p = (app.path || "").toLowerCase();
	const n = (app.name || "").toLowerCase();
	if (p.includes("_crx_") || p.includes("!app") || p.includes("extension://")) return true;
	if (isBrowserHost(p)) {
		const isChrome = p.includes("chrome");
		const isEdge = p.includes("msedge") || p.includes("edge");
		const isBrave = p.includes("brave");
		if (isChrome && !n.includes("chrome") && !n.includes("google")) return true;
		if (isEdge && !n.includes("edge") && !n.includes("microsoft")) return true;
		if (isBrave && !n.includes("brave")) return true;
	}
	return false;
}

export function appIdentity(p: string, executable?: string, name?: string) {
	if (!p) return "";
	const normalized = p.toLowerCase().replace(/\\/g, "/");
	if (!normalized.includes("/")) return normalized;
	if (name && isBrowserHost(normalized)) {
		return `${normalized}:${name.toLowerCase()}`;
	}
	if (executable) return `${normalized}:${executable.toLowerCase()}`;
	return normalized;
}

export const itemKey = (app: AppInfo) =>
	app.hwnd
		? `${appIdentity(app.path, app.executable, app.name)}#${app.hwnd}`
		: appIdentity(app.path, app.executable, app.name);

export const fileOf = (p: string) =>
	(p.split("/").pop()?.split("\\").pop()?.toLowerCase() || "").replace(/\.lnk$/, "");

export const isIdentifier = (p: string) => !p.includes("/") && !p.includes("\\");

export function isSameApp(a: AppInfo, b: AppInfo): boolean {
	if (!a.path || !b.path) return false;
	const aNorm = a.path.toLowerCase().replace(/\\/g, "/");
	const bNorm = b.path.toLowerCase().replace(/\\/g, "/");

	if (itemKey(a) === itemKey(b)) return true;
	if (aNorm === bNorm && !isBrowserHost(aNorm)) return true;
	if (matchesPackagedExecutable(a, b)) return true;

	const aIsHost = isBrowserHost(a.path);
	const bIsHost = isBrowserHost(b.path);
	if (aIsHost || bIsHost) {
		const aExe = (a.executable?.toLowerCase() || fileOf(a.path)).replace(/\.(exe|lnk)$/, "");
		const bExe = (b.executable?.toLowerCase() || fileOf(b.path)).replace(/\.(exe|lnk)$/, "");
		const sameBrowserFamily =
			(aExe.includes("chrome") && bExe.includes("chrome")) ||
			(aExe.includes("msedge") && bExe.includes("msedge")) ||
			(aExe.includes("brave") && bExe.includes("brave")) ||
			aExe === bExe;

		if (!sameBrowserFamily) return false;

		const aPwa = isBrowserPwa(a);
		const bPwa = isBrowserPwa(b);

		// Both are regular browser instances
		if (!aPwa && !bPwa) return true;

		// Both are PWAs
		if (aPwa && bPwa) {
			return a.name.toLowerCase() === b.name.toLowerCase() || aNorm === bNorm;
		}

		// One is PWA and the other is main browser -> separate
		return false;
	}

	const aId = isIdentifier(a.path);
	const bId = isIdentifier(b.path);
	if (aId && bId) return aNorm === bNorm;
	if (aId !== bId) {
		const fileSide = aId ? b : a;
		const idLower = (aId ? a : b).path.toLowerCase();
		const na = a.name.toLowerCase();
		const nb = b.name.toLowerCase();
		if (na !== nb && !na.includes(nb) && !nb.includes(na)) return false;
		const stem = (fileSide.executable?.toLowerCase() || fileOf(fileSide.path)).replace(
			/\.(exe|lnk)$/,
			""
		);
		return stem.length >= 3 && idLower.includes(stem);
	}

	if (appIdentity(a.path, a.executable, a.name) === appIdentity(b.path, b.executable, b.name)) return true;

	const normExe = (s: string) => (s.endsWith(".exe") ? s : `${s}.exe`);
	const aExe = a.executable?.toLowerCase() || fileOf(a.path);
	const bExe = b.executable?.toLowerCase() || fileOf(b.path);
	if (!aExe || !bExe) return false;
	if (normExe(aExe) === normExe(bExe)) return true;

	if (a.name && b.name && a.name.trim().toLowerCase() === b.name.trim().toLowerCase()) {
		return true;
	}

	return false;
}

export function dockIconKey(
	app: Pick<AppInfo, "path" | "name" | "hwnd" | "icon_name" | "icon_path">,
	isHost: boolean
) {
	const path = app.icon_path ?? app.path;
	return isHost
		? `${path}:${(app.icon_name ?? app.name).toLowerCase()}`
		: app.hwnd
			? `${path}-${app.hwnd}`
			: path;
}

// Match an AUMID to an executable inside its exact Windows package family.
// Requiring the same display name avoids merging separate apps shipped in one
// package; two different AUMIDs never match through this fallback.
export function matchesPackagedExecutable(
	a: Pick<AppInfo, "path" | "name">,
	b: Pick<AppInfo, "path" | "name">
): boolean {
	if (!a.name.trim() || a.name.trim().toLowerCase() !== b.name.trim().toLowerCase()) return false;
	const family = (path: string) => {
		const match = /^([^/\\!]+_[^/\\!_]+)![^/\\!]+$/.exec(path.toLowerCase());
		return match?.[1];
	};
	const executableFamily = (path: string) => {
		const match = /(?:^|\/)windowsapps\/([^/]+)\/.+\.exe$/.exec(
			path.toLowerCase().replace(/\\/g, "/")
		);
		if (!match) return undefined;
		const parts = match[1].split("_");
		if (parts.length !== 5 || !/^\d+\.\d+\.\d+\.\d+$/.test(parts[1]) || !parts[4]) return undefined;
		return `${parts[0]}_${parts[4]}`;
	};
	const aFamily = family(a.path);
	const bFamily = family(b.path);
	if (!!aFamily === !!bFamily) return false;
	return aFamily ? aFamily === executableFamily(b.path) : bFamily === executableFamily(a.path);
}

const trayPath = (app: TrayApp) => app.path.toLowerCase().replace(/\\/g, "/");

const isBrowserHostPath = (p: string) => /(msedge|chrome|brave|vivaldi|firefox)\.exe$/i.test(p);

const ownsVisibleWindow = (window: AppInfo, tray: TrayApp) =>
	(window.all_hwnds ?? (window.hwnd ? [[window.hwnd, window.name]] : [])).some(([hwnd]) =>
		tray.window_handles.includes(hwnd as number)
	) ||
	// A window whose UI runs in a helper process (Steam's steamwebhelper.exe)
	// is reported under the app's own exe; its tray icon belongs to that exe.
	// Browser hosts are excluded: one exe hosts many unrelated web apps.
	matchesPackagedExecutable(window, tray) ||
	(!isBrowserHostPath(window.path) &&
		window.path.toLowerCase().replace(/\\/g, "/") === trayPath(tray));

// An unpinned notification icon becomes a dock item only after its app has
// appeared with a window. Pinned apps are eligible immediately at startup.
// Forget observed apps when their tray icon exits, so a later tray-only launch
// does not inherit a previous session's dock slot.
export function selectDockTrayApps(
	windows: AppInfo[],
	tray: TrayApp[],
	previouslyOpened: Set<string>,
	isPinned: (app: TrayApp) => boolean
): { visible: TrayApp[]; observed: Set<string> } {
	const observed = new Set<string>();
	for (const app of tray) {
		const path = trayPath(app);
		if (previouslyOpened.has(path) || windows.some((window) => ownsVisibleWindow(window, app))) {
			observed.add(path);
		}
	}
	return {
		visible: tray.filter((app) => observed.has(trayPath(app)) || isPinned(app)),
		observed
	};
}

// Associate through actual owner windows, including AUMID/PWA entries whose
// launch path differs from their executable. Never attach a tray to a different
// app just because it has the same display name or executable basename.
export function mergeTrayApps(windows: AppInfo[], tray: TrayApp[]): AppInfo[] {
	const result = windows.map((app) => ({
		...app,
		is_background: false,
		tray_ids: undefined as string[] | undefined
	}));
	for (const app of tray) {
		const owners = result.filter((item) => ownsVisibleWindow(item, app));
		if (owners.length) {
			// A browser tray can own several unrelated PWAs. Keep the running
			// items distinct and don't give any one PWA the browser's tray menu.
			if (owners.length === 1) owners[0].tray_ids = app.tray_ids;
			continue;
		}
		result.push({
			name: app.name,
			path: app.path,
			icon: null,
			is_running: true,
			is_background: true,
			all_hwnds: [],
			tray_ids: app.tray_ids,
			executable: app.path.split(/[\\/]/).pop()
		});
	}
	return result;
}

export function formatWindowTitle(rawTitle: string | undefined | null, maxLen = 30): string {
	if (!rawTitle) return "";
	const trimmed = rawTitle.trim();
	if (trimmed.length <= maxLen) return trimmed;
	return trimmed.slice(0, maxLen - 3) + "...";
}
