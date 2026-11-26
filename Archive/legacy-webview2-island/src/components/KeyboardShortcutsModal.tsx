import React, { useEffect } from "react";
import { Keyboard, X } from "lucide-react";

export interface KeyboardShortcutsModalProps {
	isOpen: boolean;
	onClose: () => void;
}

interface ShortcutItem {
	keys: string[];
	label: string;
}

interface ShortcutCategory {
	title: string;
	items: ShortcutItem[];
}

const SHORTCUT_CATEGORIES_LEFT: ShortcutCategory[] = [
	{
		title: "Aktivasi & Tampilan",
		items: [
			{ keys: ["F12", "Pause"], label: "Notch" },
			{ keys: ["Ctrl + Shift + H"], label: "Asisten" },
			{ keys: ["Ctrl + Shift + M"], label: "Musik & Lirik" },
			{ keys: ["Ctrl + Shift + D"], label: "Dock" },
			{ keys: ["F1", "Ctrl + Shift + /"], label: "Daftar Pintasan" },
			{ keys: ["Esc"], label: "Tutup Modal" }
		]
	}
];

const SHORTCUT_CATEGORIES_RIGHT: ShortcutCategory[] = [
	{
		title: "Asisten & Navigasi",
		items: [
			{ keys: ["/", "/chat"], label: "Chat Panjang" },
			{ keys: ["Enter"], label: "Kirim Pesan" },
			{ keys: ["Shift + Enter"], label: "Baris Baru" },
			{ keys: ["Shift + Scroll"], label: "Ganti Tab" }
		]
	},
	{
		title: "Sistem & Kontrol",
		items: [
			{ keys: ["Ctrl + Shift + Q"], label: "Keluar" },
			{ keys: ["Ctrl + Alt + Shift + P"], label: "Jeda Darurat" }
		]
	}
];

export const KeyboardShortcutsModal: React.FC<KeyboardShortcutsModalProps> = ({ isOpen, onClose }) => {
	useEffect(() => {
		if (!isOpen) return;
		const handleKeyDown = (e: KeyboardEvent) => {
			if (e.key === "Escape") {
				e.preventDefault();
				onClose();
			}
		};
		window.addEventListener("keydown", handleKeyDown);
		return () => window.removeEventListener("keydown", handleKeyDown);
	}, [isOpen, onClose]);

	const renderCategory = (cat: ShortcutCategory) => (
		<div key={cat.title} className="shortcuts-category-card" style={{ display: "flex", flexDirection: "column", gap: 5, minWidth: 0 }}>
			<div
				style={{
					fontSize: 9.5,
					fontWeight: 700,
					color: "#00D2FF",
					textTransform: "uppercase",
					letterSpacing: "0.08em",
					paddingLeft: 2
				}}
			>
				{cat.title}
			</div>
			<div
				style={{
					display: "flex",
					flexDirection: "column",
					gap: 4,
					backgroundColor: "rgba(255, 255, 255, 0.025)",
					border: "1px solid rgba(255, 255, 255, 0.06)",
					borderRadius: 10,
					padding: "6px 8px",
					minWidth: 0
				}}
			>
				{cat.items.map((item, idx) => (
					<div
						key={idx}
						style={{
							display: "flex",
							alignItems: "center",
							justifyContent: "space-between",
							gap: 6,
							padding: "3.5px 6px",
							borderRadius: 6,
							backgroundColor: "rgba(255, 255, 255, 0.02)",
							minWidth: 0
						}}
					>
						<span style={{ fontSize: 11, fontWeight: 500, color: "#E2E8F0", whiteSpace: "nowrap" }}>
							{item.label}
						</span>
						<div
							style={{
								display: "flex",
								alignItems: "center",
								gap: 3,
								flexWrap: "nowrap",
								justifyContent: "flex-end",
								flexShrink: 0
							}}
						>
							{item.keys.map((k, kIdx) => (
								<React.Fragment key={kIdx}>
									{kIdx > 0 && (
										<span style={{ fontSize: 8.5, color: "rgba(255, 255, 255, 0.28)", padding: "0 1px" }}>/</span>
									)}
									<kbd
										style={{
											display: "inline-flex",
											alignItems: "center",
											justifyContent: "center",
											padding: "1.5px 5px",
											fontSize: 9,
											fontFamily: "Consolas, 'SF Mono', monospace",
											fontWeight: 600,
											color: "#E2F8FF",
											backgroundColor: "rgba(0, 210, 255, 0.09)",
											border: "1px solid rgba(0, 210, 255, 0.32)",
											borderRadius: 4,
											boxShadow:
												"0 0 6px rgba(0, 210, 255, 0.18), inset 0 1px 0 rgba(255, 255, 255, 0.12)",
											textShadow: "0 0 5px rgba(0, 210, 255, 0.4)",
											whiteSpace: "nowrap",
											lineHeight: 1.2
										}}
									>
										{k}
									</kbd>
								</React.Fragment>
							))}
						</div>
					</div>
				))}
			</div>
		</div>
	);

	if (!isOpen) return null;

	return (
		<div
			className="shortcuts-content"
			style={{
				display: "flex",
				flexDirection: "column",
				justifyContent: "space-between",
				height: "100%",
				width: "100%",
				padding: "6px 16px 8px 16px",
				boxSizing: "border-box",
				color: "#F4F6FA",
				overflow: "hidden",
				backgroundColor: "transparent",
				backdropFilter: "blur(20px)",
				WebkitBackdropFilter: "blur(20px)"
			}}
			onClick={(e) => e.stopPropagation()}
		>
			{/* Notch Header */}
			<div
				className="shortcuts-header"
				style={{
					display: "flex",
					alignItems: "center",
					justifyContent: "space-between",
					minHeight: "34px",
					paddingBottom: 6,
					borderBottom: "1px solid rgba(255, 255, 255, 0.06)",
					flexShrink: 0
				}}
			>
				<div style={{ display: "flex", alignItems: "center", gap: 8 }}>
					<div
						style={{
							width: 24,
							height: 24,
							borderRadius: 7,
							backgroundColor: "rgba(0, 210, 255, 0.12)",
							border: "1px solid rgba(0, 210, 255, 0.35)",
							display: "flex",
							alignItems: "center",
							justifyContent: "center",
							color: "#00D2FF",
							boxShadow: "0 0 10px rgba(0, 210, 255, 0.2)"
						}}
					>
						<Keyboard size={13} />
					</div>
					<div style={{ display: "flex", alignItems: "baseline", gap: 6 }}>
						<h3
							style={{
								margin: 0,
								fontSize: 13,
								fontWeight: 600,
								color: "#FFFFFF",
								letterSpacing: "-0.01em"
							}}
						>
							Daftar Pintasan Keyboard
						</h3>
						<span
							style={{
								fontSize: 9.5,
								color: "#00D2FF",
								backgroundColor: "rgba(0, 210, 255, 0.1)",
								padding: "1px 5px",
								borderRadius: 4,
								border: "1px solid rgba(0, 210, 255, 0.25)"
							}}
						>
							F1
						</span>
					</div>
				</div>
				<button
					onClick={onClose}
					aria-label="Tutup"
					style={{
						background: "rgba(255, 255, 255, 0.04)",
						border: "1px solid rgba(255, 255, 255, 0.08)",
						color: "#94A3B8",
						cursor: "pointer",
						padding: "4px 5px",
						display: "flex",
						alignItems: "center",
						justifyContent: "center",
						borderRadius: 6,
						transition: "all 0.15s ease"
					}}
					onMouseEnter={(e) => {
						e.currentTarget.style.color = "#FFFFFF";
						e.currentTarget.style.backgroundColor = "rgba(255, 255, 255, 0.12)";
					}}
					onMouseLeave={(e) => {
						e.currentTarget.style.color = "#94A3B8";
						e.currentTarget.style.backgroundColor = "rgba(255, 255, 255, 0.04)";
					}}
				>
					<X size={13} />
				</button>
			</div>

			{/* 2-Column Grid Layout */}
			<div
				className="shortcuts-list"
				style={{
					display: "grid",
					gridTemplateColumns: "minmax(0, 1fr) minmax(0, 1fr)",
					gap: 14,
					flex: 1,
					minHeight: 0,
					overflowY: "auto",
					scrollbarWidth: "none",
					padding: "4px 0 6px 0"
				}}
			>
				{/* Left Column: Aktivasi & Tampilan */}
				<div style={{ display: "flex", flexDirection: "column", gap: 8, minWidth: 0 }}>
					{SHORTCUT_CATEGORIES_LEFT.map(renderCategory)}
				</div>

				{/* Right Column: Asisten & Sistem */}
				<div style={{ display: "flex", flexDirection: "column", gap: 8, minWidth: 0 }}>
					{SHORTCUT_CATEGORIES_RIGHT.map(renderCategory)}
				</div>
			</div>

			{/* Footer hint */}
			<div
				style={{
					fontSize: 9.5,
					color: "rgba(255, 255, 255, 0.35)",
					textAlign: "center",
					paddingTop: 4,
					borderTop: "1px solid rgba(255, 255, 255, 0.05)",
					display: "flex",
					alignItems: "center",
					justifyContent: "center",
					gap: 4,
					flexShrink: 0
				}}
			>
				Tekan{" "}
				<kbd
					style={{
						display: "inline-block",
						padding: "1px 4px",
						fontSize: 8.5,
						fontFamily: "Consolas, 'SF Mono', monospace",
						fontWeight: 600,
						borderRadius: 3,
						backgroundColor: "rgba(0, 210, 255, 0.08)",
						border: "1px solid rgba(0, 210, 255, 0.25)",
						color: "#00D2FF"
					}}
				>
					Esc
				</kbd>{" "}
				atau{" "}
				<kbd
					style={{
						display: "inline-block",
						padding: "1px 4px",
						fontSize: 8.5,
						fontFamily: "Consolas, 'SF Mono', monospace",
						fontWeight: 600,
						borderRadius: 3,
						backgroundColor: "rgba(0, 210, 255, 0.08)",
						border: "1px solid rgba(0, 210, 255, 0.25)",
						color: "#00D2FF"
					}}
				>
					F1
				</kbd>{" "}
				atau klik di luar untuk menutup
			</div>
		</div>
	);
};
