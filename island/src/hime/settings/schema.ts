/**
 * Canonical Settings Schema Registry (island/src/hime/settings/schema.ts)
 * Single source of truth for 17 canonical settings sections in STAGE U1.
 * Strict text limits enforced:
 *   - Titles / Judul <= 16 chars
 *   - Descriptions / Deskripsi <= 36 chars
 *   - Buttons / Tombol <= 10 chars
 */

export interface SettingDefinition<T = any> {
	key: string;
	type: "boolean" | "string" | "number" | "enum" | "range";
	default: T;
	min?: number;
	max?: number;
	step?: number;
	options?: { label: string; value: T }[];
	section: string;
	label: string; // Strictly <= 16 chars
	desc: string; // Strictly <= 36 chars
	icon?: string;
	secret?: boolean;
}

export interface SettingsSection {
	id: string;
	title: string; // Strictly <= 16 chars
	desc: string; // Strictly <= 36 chars
	icon: string;
}

export const SETTINGS_SECTIONS: SettingsSection[] = [
	{ id: "general", title: "Umum", desc: "Preferensi dasar sistem", icon: "Settings" },
	{ id: "appearance", title: "Tampilan", desc: "Tema visual dan animasi", icon: "Palette" },
	{ id: "notch", title: "Notch", desc: "Perilaku notch dan maskot", icon: "PanelTop" },
	{ id: "hime-ai", title: "Hime & AI", desc: "Gateway 9Router dan model", icon: "Bot" },
	{ id: "chat", title: "Chat", desc: "Pengaturan obrolan dan suara", icon: "MessageSquare" },
	{ id: "modules", title: "Modul", desc: "Kelola modul ekstensi", icon: "Blocks" },
	{ id: "tasks", title: "Tugas & Rutin", desc: "Pengingat dan jam tenang", icon: "CheckSquare" },
	{ id: "focus", title: "Fokus & Aturan", desc: "Penegakan sesi fokus", icon: "ShieldAlert" },
	{ id: "shelf", title: "Simpanan", desc: "Riwayat klip dan berkas", icon: "Archive" },
	{ id: "translate", title: "Terjemah", desc: "Bahasa dan mode terjemah", icon: "Languages" },
	{ id: "music", title: "Musik & Lirik", desc: "Panel musik dan lirik", icon: "Music" },
	{ id: "keybinds", title: "Pintasan", desc: "Pintasan keyboard global", icon: "Keyboard" },
	{ id: "devices", title: "Perangkat", desc: "Sinkronisasi Android", icon: "Smartphone" },
	{ id: "interact", title: "Hime Bertanya", desc: "Frekuensi sapa proaktif", icon: "HelpCircle" },
	{ id: "privacy", title: "Privasi & Data", desc: "Penyimpanan data lokal", icon: "Lock" },
	{ id: "resources", title: "Sumber Daya", desc: "RAM, CPU dan mode hemat", icon: "Cpu" },
	{ id: "about", title: "Tentang", desc: "Versi dan lisensi open source", icon: "Info" }
];

export const SETTING_DEFINITIONS: Record<string, SettingDefinition> = {
	// 1. Umum
	"autostart": {
		key: "autostart",
		type: "boolean",
		default: true,
		section: "general",
		label: "Mulai Otomatis",
		desc: "Buka Hime saat Windows mulai"
	},
	"hime-lang": {
		key: "hime-lang",
		type: "enum",
		default: "id",
		options: [
			{ label: "Bahasa Indonesia", value: "id" },
			{ label: "English", value: "en" }
		],
		section: "general",
		label: "Bahasa",
		desc: "Bahasa antarmuka Hime"
	},
	"bloom-time-format-24h": {
		key: "bloom-time-format-24h",
		type: "boolean",
		default: true,
		section: "general",
		label: "Format 24 Jam",
		desc: "Gunakan format jam 24 jam"
	},

	// 2. Tampilan
	"themeMode": {
		key: "themeMode",
		type: "enum",
		default: "dark",
		options: [
			{ label: "Gelap", value: "dark" },
			{ label: "Terang", value: "light" },
			{ label: "Adaptif", value: "adaptive" }
		],
		section: "appearance",
		label: "Mode Tema",
		desc: "Pilih tema tampilan desktop"
	},
	"hime-animation-preset": {
		key: "hime-animation-preset",
		type: "enum",
		default: "snappy",
		options: [
			{ label: "Snappy (220ms)", value: "snappy" },
			{ label: "Smooth (300ms)", value: "smooth" },
			{ label: "Soft (350ms)", value: "soft" },
			{ label: "Mati", value: "off" }
		],
		section: "appearance",
		label: "Animasi",
		desc: "Tingkat kehalusan gerak"
	},

	// 3. Notch
	"bloom-notch-mode": {
		key: "bloom-notch-mode",
		type: "enum",
		default: "smart",
		options: [
			{ label: "Smart", value: "smart" },
			{ label: "Fixed", value: "fixed" },
			{ label: "Peek", value: "peek" }
		],
		section: "notch",
		label: "Mode Notch",
		desc: "Perilaku tampil di layar"
	},
	"hime-eyes-ambient": {
		key: "hime-eyes-ambient",
		type: "enum",
		default: "Sedang",
		options: [
			{ label: "Mati", value: "Mati" },
			{ label: "Jarang", value: "Jarang" },
			{ label: "Sedang", value: "Sedang" },
			{ label: "Sering", value: "Sering" }
		],
		section: "notch",
		label: "Mata di Notch",
		desc: "Frekuensi mata di jam"
	},
	"hime-eyes-follow": {
		key: "hime-eyes-follow",
		type: "boolean",
		default: true,
		section: "notch",
		label: "Ikuti Kursor",
		desc: "Arah pandang ikuti kursor"
	},

	// 4. Hime & AI
	"hime-9router-url": {
		key: "hime-9router-url",
		type: "string",
		default: "http://127.0.0.1:20128/v1",
		section: "hime-ai",
		label: "URL 9Router",
		desc: "Endpoint HTTP gateway lokal"
	},
	"hime-9router-model": {
		key: "hime-9router-model",
		type: "string",
		default: "anthropic/claude-3-5-sonnet",
		section: "hime-ai",
		label: "Model AI",
		desc: "Model utama respon chat"
	},
	"hime-runs-parallel": {
		key: "hime-runs-parallel",
		type: "number",
		default: 1,
		min: 1,
		max: 3,
		section: "hime-ai",
		label: "Run Paralel",
		desc: "Batas proses AI simultan"
	},
	"hime-ai-memory": {
		key: "hime-ai-memory",
		type: "boolean",
		default: true,
		section: "hime-ai",
		label: "Ingatan AI",
		desc: "Simpan konteks antar-sesi"
	},

	// 5. Chat
	"hime-voice-engine": {
		key: "hime-voice-engine",
		type: "enum",
		default: "auto",
		options: [
			{ label: "Otomatis", value: "auto" },
			{ label: "Cloud", value: "cloud" },
			{ label: "Lokal", value: "local" }
		],
		section: "chat",
		label: "Mesin Suara",
		desc: "Mode pengenalan suara"
	},
	"hime-voice-autosend": {
		key: "hime-voice-autosend",
		type: "boolean",
		default: false,
		section: "chat",
		label: "Kirim Suara",
		desc: "Kirim tanpa konfirmasi"
	},

	// 6. Modul
	"hime-mod-eyes": {
		key: "hime-mod-eyes",
		type: "boolean",
		default: true,
		section: "modules",
		label: "Mata Hime",
		desc: "Maskot ekspresi di notch"
	},
	"hime-mod-keybinds": {
		key: "hime-mod-keybinds",
		type: "boolean",
		default: true,
		section: "modules",
		label: "Pintasan",
		desc: "Pintasan global keyboard"
	},

	// 7. Tugas & Rutin
	"hime-quiet-hours": {
		key: "hime-quiet-hours",
		type: "string",
		default: "23:00-06:00",
		section: "tasks",
		label: "Jam Tenang",
		desc: "Jeda notifikasi malam"
	},

	// 8. Fokus
	"hime-enforce-delay": {
		key: "hime-enforce-delay",
		type: "number",
		default: 10,
		min: 5,
		max: 30,
		section: "focus",
		label: "Jeda Longgar",
		desc: "Menit sebelum longgar"
	},

	// 9. Simpanan (Shelf)
	"hime-clip-capture": {
		key: "hime-clip-capture",
		type: "boolean",
		default: false,
		section: "shelf",
		label: "Klip Otomatis",
		desc: "Rekam salinan clipboard"
	},
	"hime-clip-ttl-h": {
		key: "hime-clip-ttl-h",
		type: "number",
		default: 24,
		min: 1,
		max: 168,
		section: "shelf",
		label: "Batas Klip",
		desc: "Masa simpan klip (jam)"
	},

	// 10. Terjemah
	"hime-translate-target": {
		key: "hime-translate-target",
		type: "string",
		default: "id",
		section: "translate",
		label: "Target Bahasa",
		desc: "Bahasa tujuan terjemah"
	},

	// 11. Musik
	"hime-lyrics": {
		key: "hime-lyrics",
		type: "boolean",
		default: true,
		section: "music",
		label: "Lirik Lagu",
		desc: "Tampilkan lirik di notch"
	},
	"hime-lyrics-online": {
		key: "hime-lyrics-online",
		type: "boolean",
		default: true,
		section: "music",
		label: "Ambil Online",
		desc: "Cari lirik via LRCLIB"
	},
	"hime-lyrics-pill": {
		key: "hime-lyrics-pill",
		type: "boolean",
		default: true,
		section: "music",
		label: "Tampil di Pill",
		desc: "Baris lirik di pill tertutup"
	},
	"hime-lyrics-bubble": {
		key: "hime-lyrics-bubble",
		type: "boolean",
		default: false,
		section: "music",
		label: "Bubble Lirik",
		desc: "Bubble melayang di bawah notch"
	},
	"hime-lyrics-karaoke": {
		key: "hime-lyrics-karaoke",
		type: "boolean",
		default: false,
		section: "music",
		label: "Karaoke Kata",
		desc: "Sorot kata per kata"
	},
	"hime-lyrics-romanized": {
		key: "hime-lyrics-romanized",
		type: "boolean",
		default: true,
		section: "music",
		label: "Lirik Romaji",
		desc: "Aksara Latin untuk lagu asing"
	},
	"hime-lyrics-text-size": {
		key: "hime-lyrics-text-size",
		type: "enum",
		default: "md",
		section: "music",
		label: "Ukuran Teks",
		desc: "Ukuran huruf baris lirik",
		options: [
			{ label: "Kecil", value: "sm" },
			{ label: "Sedang", value: "md" },
			{ label: "Besar", value: "lg" }
		]
	},
	"hime-lyrics-offset-default": {
		key: "hime-lyrics-offset-default",
		type: "number",
		default: 0,
		section: "music",
		label: "Offset Default",
		desc: "Geser waktu default (ms)",
		min: -5000,
		max: 5000,
		step: 100
	},
	"hime-lyrics-clear-cache": {
		key: "hime-lyrics-clear-cache",
		type: "boolean",
		default: false,
		section: "music",
		label: "Cache Lirik",
		desc: "Bersihkan cache lirik lokal"
	},

	// 12. Pintasan
	"hime-key-reset": {
		key: "hime-key-reset",
		type: "boolean",
		default: false,
		section: "keybinds",
		label: "Atur Pintasan",
		desc: "Reset pintasan ke awal"
	},

	// 13. Perangkat
	"hime-dev-pair": {
		key: "hime-dev-pair",
		type: "boolean",
		default: false,
		section: "devices",
		label: "Pasang HP",
		desc: "Sambungkan ponsel Android"
	},

	// 14. Hime Bertanya
	"hime-chatty": {
		key: "hime-chatty",
		type: "enum",
		default: "seimbang",
		options: [
			{ label: "Pendiam", value: "pendiam" },
			{ label: "Seimbang", value: "seimbang" },
			{ label: "Cerewet", value: "cerewet" }
		],
		section: "interact",
		label: "Tingkat Sapa",
		desc: "Anggaran perhatian proaktif"
	},
	"hime-quiet-hours": {
		key: "hime-quiet-hours",
		type: "boolean",
		default: true,
		section: "interact",
		label: "Jam Tenang",
		desc: "Hanya P0 pada 23:00–06:00"
	},

	// 15. Privasi
	"hime-priv-clear": {
		key: "hime-priv-clear",
		type: "boolean",
		default: false,
		section: "privacy",
		label: "Hapus Data",
		desc: "Hapus riwayat lokal"
	},

	// 16. Sumber Daya
	"hime-save-mode": {
		key: "hime-save-mode",
		type: "enum",
		default: "auto",
		options: [
			{ label: "Otomatis", value: "auto" },
			{ label: "Aktif", value: "on" },
			{ label: "Mati", value: "off" }
		],
		section: "resources",
		label: "Mode Hemat",
		desc: "Optimasi baterai & RAM"
	},
	"hime-ai-idle-min": {
		key: "hime-ai-idle-min",
		type: "number",
		default: 10,
		min: 5,
		max: 60,
		section: "resources",
		label: "AI Idle-Stop",
		desc: "Hentikan AI saat diam"
	},

	// 17. Tentang
	"hime-about-update": {
		key: "hime-about-update",
		type: "boolean",
		default: true,
		section: "about",
		label: "Cek Update",
		desc: "Periksa versi terbaru"
	}
};
