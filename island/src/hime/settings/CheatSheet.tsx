import React, { useState } from "react";
import { Search, X, Command } from "lucide-react";

interface ShortcutEntry {
	keyCombo: string;
	title: string; // Strictly <= 16 chars
	desc: string; // Strictly <= 36 chars
	type: "global" | "local";
}

const SHORTCUTS: ShortcutEntry[] = [
	// Global shortcuts
	{ keyCombo: "Ctrl + Alt + Space", title: "Chat Hime", desc: "Buka chat dan fokus input", type: "global" },
	{ keyCombo: "Ctrl + Alt + M", title: "Input Suara", desc: "Tahan rekam atau ketuk bicara", type: "global" },
	{ keyCombo: "Ctrl + Alt + T", title: "Terjemahkan", desc: "Terjemahkan teks yang dipilih", type: "global" },
	{ keyCombo: "Ctrl + Alt + N", title: "Tugas Baru", desc: "Tambah tugas cepat di notch", type: "global" },
	{ keyCombo: "Ctrl + Alt + S", title: "Buka Simpanan", desc: "Buka panel simpanan klip", type: "global" },
	{ keyCombo: "Ctrl + Alt + V", title: "Tempel Klip", desc: "Pilih dan tempel riwayat klip", type: "global" },
	{ keyCombo: "Ctrl + Alt + F", title: "Sesi Fokus", desc: "Mulai atau jeda sesi fokus", type: "global" },
	{ keyCombo: "Ctrl + Alt + L", title: "Tampil Lirik", desc: "Tampilkan lirik lagu di notch", type: "global" },
	{ keyCombo: "Ctrl + Alt + H", title: "Buka Island", desc: "Buka atau sembunyikan notch", type: "global" },
	{ keyCombo: "Ctrl + Alt + Shift + P", title: "Jeda Darurat", desc: "Jeda penegakan aturan darurat", type: "global" },
	{ keyCombo: "Ctrl + Alt + /", title: "Pintasan", desc: "Daftar pintasan keyboard", type: "global" },
	{ keyCombo: "Ctrl + Alt + Q", title: "Keluar Hime", desc: "Matikan seluruh sistem Hime", type: "global" },

	// Local shortcuts
	{ keyCombo: "Esc", title: "Tutup Panel", desc: "Mundur satu tingkat atau tutup", type: "local" },
	{ keyCombo: "← / →", title: "Pindah Panel", desc: "Geser antar panel notch", type: "local" },
	{ keyCombo: "1 — 6", title: "Pilih Panel", desc: "Lompat langsung ke panel 1..6", type: "local" },
	{ keyCombo: "/", title: "Menu Skill", desc: "Buka pilihan kemampuan chat", type: "local" },
	{ keyCombo: "Enter", title: "Kirim Chat", desc: "Kirim pesan teks atau aksi", type: "local" },
	{ keyCombo: "Shift + Enter", title: "Baris Baru", desc: "Buat baris baru di input", type: "local" },
	{ keyCombo: "Ctrl + Enter", title: "Kirim Cepat", desc: "Kirim tanpa lepas fokus", type: "local" },
	{ keyCombo: "Ctrl + F", title: "Cari Item", desc: "Buka pencarian riwayat", type: "local" },
	{ keyCombo: "↑ / ↓", title: "Pilih Menu", desc: "Navigasi pilihan dalam daftar", type: "local" },
	{ keyCombo: "Space", title: "Cek Tugas", desc: "Tandai tugas selesai", type: "local" },
	{ keyCombo: "Ctrl + Z", title: "Urungkan", desc: "Batal aksi sebelumnya", type: "local" },
	{ keyCombo: "[ / ]", title: "Geser Waktu", desc: "Offset lirik ±0.5 detik", type: "local" },
	{ keyCombo: "Ctrl + [ / ]", title: "Geser Halus", desc: "Offset lirik ±0.1 detik", type: "local" }
];

interface CheatSheetProps {
	onClose?: () => void;
}

export const CheatSheet: React.FC<CheatSheetProps> = ({ onClose }) => {
	const [query, setQuery] = useState("");
	const [filter, setFilter] = useState<"all" | "global" | "local">("all");

	const filtered = SHORTCUTS.filter((s) => {
		if (filter !== "all" && s.type !== filter) return false;
		if (!query.trim()) return true;
		const q = query.toLowerCase();
		return (
			s.title.toLowerCase().includes(q) ||
			s.desc.toLowerCase().includes(q) ||
			s.keyCombo.toLowerCase().includes(q)
		);
	});

	return (
		<div
			style={{
				display: "flex",
				flexDirection: "column",
				gap: "12px",
				padding: "16px",
				background: "var(--surface-2, #14161C)",
				border: "1px solid var(--line, #262A34)",
				borderRadius: "14px",
				color: "var(--text-1, #F4F6FA)",
				width: "100%",
				maxWidth: "460px",
				boxSizing: "border-box"
			}}
		>
			{/* Header */}
			<div
				style={{
					display: "flex",
					alignItems: "center",
					justifyContent: "space-between"
				}}
			>
				<div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
					<Command size={16} color="var(--accent-mint, #34E3A6)" />
					<span style={{ fontSize: "14px", fontWeight: 600 }}>Daftar Pintasan</span>
				</div>
				{onClose && (
					<button
						type="button"
						onClick={onClose}
						style={{
							background: "none",
							border: "none",
							color: "var(--text-3, #6F7785)",
							cursor: "pointer",
							padding: "4px"
						}}
					>
						<X size={16} />
					</button>
				)}
			</div>

			{/* Search & Filter Controls */}
			<div style={{ display: "flex", gap: "8px" }}>
				<div
					style={{
						display: "flex",
						alignItems: "center",
						gap: "6px",
						flex: 1,
						background: "rgba(255, 255, 255, 0.06)",
						border: "1px solid rgba(255, 255, 255, 0.1)",
						borderRadius: "8px",
						padding: "4px 8px"
					}}
				>
					<Search size={13} color="var(--text-3, #6F7785)" />
					<input
						type="text"
						value={query}
						onChange={(e) => setQuery(e.target.value)}
						placeholder="Cari pintasan..."
						style={{
							background: "transparent",
							border: "none",
							color: "#fff",
							fontSize: "11px",
							outline: "none",
							width: "100%"
						}}
					/>
				</div>

				<div style={{ display: "flex", gap: "4px" }}>
					{(["all", "global", "local"] as const).map((t) => (
						<button
							key={t}
							type="button"
							onClick={() => setFilter(t)}
							style={{
								background: filter === t ? "rgba(52, 227, 166, 0.2)" : "rgba(255, 255, 255, 0.05)",
								border: filter === t ? "1px solid rgba(52, 227, 166, 0.4)" : "1px solid transparent",
								borderRadius: "6px",
								padding: "4px 8px",
								fontSize: "11px",
								color: filter === t ? "#34E3A6" : "var(--text-2, #A7AEBB)",
								cursor: "pointer",
								textTransform: "capitalize"
							}}
						>
							{t === "all" ? "Semua" : t === "global" ? "Global" : "Notch"}
						</button>
					))}
				</div>
			</div>

			{/* Shortcut Items List */}
			<div
				style={{
					display: "flex",
					flexDirection: "column",
					gap: "6px",
					maxHeight: "320px",
					overflowY: "auto"
				}}
			>
				{filtered.map((s, idx) => (
					<div
						key={idx}
						style={{
							display: "flex",
							alignItems: "center",
							justifyContent: "space-between",
							padding: "6px 8px",
							background: "rgba(255, 255, 255, 0.02)",
							borderRadius: "6px"
						}}
					>
						<div style={{ display: "flex", flexDirection: "column", gap: "2px" }}>
							<span style={{ fontSize: "12px", fontWeight: 500 }}>{s.title}</span>
							<span style={{ fontSize: "10px", color: "var(--text-3, #6F7785)" }}>
								{s.desc}
							</span>
						</div>
						<kbd
							style={{
								padding: "3px 6px",
								background: "rgba(255, 255, 255, 0.08)",
								border: "1px solid rgba(255, 255, 255, 0.15)",
								borderRadius: "4px",
								fontSize: "10px",
								fontFamily: "monospace",
								color: "var(--accent-cyan, #38D6F5)"
							}}
						>
							{s.keyCombo}
						</kbd>
					</div>
				))}
				{filtered.length === 0 && (
					<div
						style={{
							textAlign: "center",
							fontSize: "11px",
							color: "var(--text-3, #6F7785)",
							padding: "16px 0"
						}}
					>
						Tidak ada pintasan ditemukan
					</div>
				)}
			</div>
		</div>
	);
};
