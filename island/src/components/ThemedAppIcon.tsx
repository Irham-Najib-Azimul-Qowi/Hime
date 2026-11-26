import React from "react";
import {
	Folder,
	Globe,
	Mail,
	Music,
	Settings,
	Terminal,
	Image as ImageIcon,
	Calculator,
	ShoppingBag,
	Bot,
	Trash2
} from "lucide-react";
import { AppInfo } from "../dockApps";

export type AppCategory =
	| "folder"
	| "browser"
	| "mail"
	| "chat"
	| "music"
	| "notes"
	| "calendar"
	| "settings"
	| "terminal"
	| "photos"
	| "calculator"
	| "store"
	| "hime"
	| "start"
	| "trash"
	| "fallback";

export function getAppCategory(name: string, path: string): AppCategory {
	const n = (name || "").toLowerCase();
	const p = (path || "").toLowerCase();

	if (p === "start") return "start";
	if (p.includes("recycle") || p.includes("trash") || n.includes("recycle bin") || n.includes("tempat sampah")) return "trash";

	// Files / Folder
	if (
		n.includes("explorer") ||
		n.includes("file") ||
		n.includes("berkas") ||
		n.includes("folder") ||
		p.includes("explorer.exe") ||
		p.includes("totalcmd") ||
		p.includes("directory")
	) {
		return "folder";
	}

	// Browser / Web
	if (
		n.includes("chrome") ||
		n.includes("edge") ||
		n.includes("brave") ||
		n.includes("firefox") ||
		n.includes("opera") ||
		n.includes("vivaldi") ||
		n.includes("safari") ||
		n.includes("browser") ||
		n.includes("internet") ||
		p.includes("chrome.exe") ||
		p.includes("msedge.exe") ||
		p.includes("brave.exe") ||
		p.includes("firefox.exe")
	) {
		return "browser";
	}

	// Mail / Email
	if (
		n.includes("mail") ||
		n.includes("outlook") ||
		n.includes("thunderbird") ||
		n.includes("proton") ||
		n.includes("surel") ||
		p.includes("outlook.exe") ||
		p.includes("thunderbird.exe")
	) {
		return "mail";
	}

	// Chat / Messaging
	if (
		n.includes("whatsapp") ||
		n.includes("telegram") ||
		n.includes("discord") ||
		n.includes("slack") ||
		n.includes("teams") ||
		n.includes("messenger") ||
		n.includes("wechat") ||
		n.includes("signal") ||
		n.includes("chat") ||
		n.includes("pesan") ||
		p.includes("whatsapp") ||
		p.includes("telegram.exe") ||
		p.includes("discord.exe") ||
		p.includes("slack.exe")
	) {
		return "chat";
	}

	// Music / Audio
	if (
		n.includes("spotify") ||
		n.includes("music") ||
		n.includes("musik") ||
		n.includes("media player") ||
		n.includes("groove") ||
		n.includes("itunes") ||
		n.includes("aimp") ||
		n.includes("foobar") ||
		n.includes("vlc") ||
		n.includes("sound") ||
		p.includes("spotify.exe") ||
		p.includes("wmplayer.exe")
	) {
		return "music";
	}

	// Notes / Text / Editor
	if (
		n.includes("notepad") ||
		n.includes("notes") ||
		n.includes("catatan") ||
		n.includes("code") ||
		n.includes("sublime") ||
		n.includes("obsidian") ||
		n.includes("word") ||
		n.includes("notion") ||
		n.includes("text") ||
		n.includes("teks") ||
		p.includes("notepad.exe") ||
		p.includes("code.exe")
	) {
		return "notes";
	}

	// Calendar
	if (
		n.includes("calendar") ||
		n.includes("kalender") ||
		n.includes("date") ||
		p.includes("calendar")
	) {
		return "calendar";
	}

	// Settings
	if (
		n.includes("settings") ||
		n.includes("pengaturan") ||
		n.includes("control panel") ||
		n.includes("hime") ||
		p.includes("systemsettings.exe") ||
		p.includes("control.exe")
	) {
		return "settings";
	}

	// Terminal / Dev
	if (
		n.includes("terminal") ||
		n.includes("powershell") ||
		n.includes("cmd") ||
		n.includes("command prompt") ||
		n.includes("bash") ||
		n.includes("alacritty") ||
		n.includes("hyper") ||
		p.includes("powershell.exe") ||
		p.includes("cmd.exe") ||
		p.includes("windowsterminal.exe")
	) {
		return "terminal";
	}

	// Photos / Graphics
	if (
		n.includes("photo") ||
		n.includes("foto") ||
		n.includes("gallery") ||
		n.includes("galeri") ||
		n.includes("paint") ||
		n.includes("photoshop") ||
		n.includes("figma") ||
		n.includes("canva") ||
		p.includes("mspaint.exe") ||
		p.includes("photoshop.exe")
	) {
		return "photos";
	}

	// Calculator
	if (n.includes("calculator") || n.includes("kalkulator") || p.includes("calc.exe")) {
		return "calculator";
	}

	// Store
	if (n.includes("store") || n.includes("steam") || n.includes("toko") || p.includes("steam.exe")) {
		return "store";
	}

	// Hime Assistant
	if (n.includes("hime") || n.includes("hermes") || n.includes("9router")) {
		return "hime";
	}

	return "fallback";
}

interface ThemedAppIconProps {
	app: Pick<AppInfo, "name" | "path" | "icon"> & Partial<AppInfo>;
	customIcon?: string | null;
	resolvedStartIcon?: string;
}

export const ThemedAppIcon: React.FC<ThemedAppIconProps> = ({
	app,
	customIcon,
	resolvedStartIcon
}) => {
	const category = getAppCategory(app.name, app.path);
	const now = new Date();
	const monthStr = now.toLocaleString("en-US", { month: "short" }).toUpperCase();
	const dayNum = now.getDate();

	// Custom icon overrides if specifically uploaded by user
	if (customIcon && !customIcon.startsWith("theme:")) {
		return (
			<div className="themed-icon-squircle custom-squircle">
				<img src={customIcon} alt={app.name} className="themed-icon-img" draggable={false} />
			</div>
		);
	}

	switch (category) {
		case "folder":
			return (
				<div className="themed-icon-squircle squircle-folder" title={app.name}>
					<Folder size={22} color="#ffffff" strokeWidth={2.4} fill="rgba(255, 255, 255, 0.28)" />
				</div>
			);

		case "browser":
			return (
				<div className="themed-icon-squircle squircle-browser" title={app.name}>
					<Globe size={22} color="#ffffff" strokeWidth={2.3} />
				</div>
			);

		case "mail":
			return (
				<div className="themed-icon-squircle squircle-mail" title={app.name}>
					<Mail size={22} color="#ffffff" strokeWidth={2.3} />
				</div>
			);

		case "chat":
			return (
				<div className="themed-icon-squircle squircle-chat" title={app.name}>
					{/* Solid rounded chat bubble with tail */}
					<svg width="22" height="22" viewBox="0 0 24 24" fill="#ffffff" style={{ filter: "drop-shadow(0 1px 2px rgba(0,0,0,0.15))" }}>
						<path d="M12 2C6.477 2 2 6.03 2 11c0 2.68 1.306 5.08 3.394 6.708L4.2 21.6a.6.6 0 0 0 .848.748l4.42-2.148C10.36 20.65 11.16 20.8 12 20.8c5.523 0 10-4.03 10-9s-4.477-9-10-9z" />
					</svg>
					{/* Notification badge */}
					<div className="dock-badge">1</div>
				</div>
			);

		case "music":
			return (
				<div className="themed-icon-squircle squircle-music" title={app.name}>
					<Music size={22} color="#ffffff" strokeWidth={2.4} />
				</div>
			);

		case "notes":
			return (
				<div className="themed-icon-squircle squircle-notes" title={app.name}>
					{/* 4 horizontal bars matching the user's reference screenshot */}
					<div className="notes-glyph-container">
						<div className="notes-line line-1" />
						<div className="notes-line line-2" />
						<div className="notes-line line-3" />
						<div className="notes-line line-4" />
					</div>
				</div>
			);

		case "calendar":
			return (
				<div className="themed-icon-squircle squircle-calendar" title={app.name}>
					<div className="calendar-header">{monthStr}</div>
					<div className="calendar-body">{dayNum}</div>
				</div>
			);

		case "settings":
			return (
				<div className="themed-icon-squircle squircle-settings" title={app.name}>
					<Settings size={22} color="#ffffff" strokeWidth={2.3} />
				</div>
			);

		case "terminal":
			return (
				<div className="themed-icon-squircle squircle-terminal" title={app.name}>
					<Terminal size={21} color="#38bdf8" strokeWidth={2.5} />
				</div>
			);

		case "photos":
			return (
				<div className="themed-icon-squircle squircle-photos" title={app.name}>
					<ImageIcon size={22} color="#ffffff" strokeWidth={2.3} />
				</div>
			);

		case "calculator":
			return (
				<div className="themed-icon-squircle squircle-calculator" title={app.name}>
					<Calculator size={22} color="#ffffff" strokeWidth={2.3} />
				</div>
			);

		case "store":
			return (
				<div className="themed-icon-squircle squircle-store" title={app.name}>
					<ShoppingBag size={22} color="#ffffff" strokeWidth={2.3} />
				</div>
			);

		case "hime":
			return (
				<div className="themed-icon-squircle squircle-hime" title={app.name}>
					<Bot size={22} color="#ffffff" strokeWidth={2.3} />
				</div>
			);

		case "trash":
			return (
				<div className="themed-icon-squircle squircle-trash" title={app.name}>
					<Trash2 size={22} color="#ffffff" strokeWidth={2.3} />
				</div>
			);

		case "start":
			return (
				<div className="themed-icon-squircle squircle-start" title="Start">
					{resolvedStartIcon ? (
						<img src={resolvedStartIcon} alt="Start" className="themed-icon-img" draggable={false} />
					) : (
						<svg width="20" height="20" viewBox="0 0 24 24" fill="#ffffff">
							<path d="M3 3h8v8H3zM13 3h8v8h-8zM3 13h8v8H3zM13 13h8v8h-8z" />
						</svg>
					)}
				</div>
			);

		case "fallback":
		default:
			// If app has an extracted icon from Windows exe, frame it inside the squircle tile
			if (app.icon) {
				return (
					<div className="themed-icon-squircle squircle-fallback" title={app.name}>
						<img src={app.icon} alt={app.name} className="themed-icon-img-fallback" draggable={false} />
					</div>
				);
			}
			return (
				<div className="themed-icon-squircle squircle-generic" title={app.name}>
					<span className="generic-letter">{app.name ? app.name[0].toUpperCase() : "A"}</span>
				</div>
			);
	}
};
