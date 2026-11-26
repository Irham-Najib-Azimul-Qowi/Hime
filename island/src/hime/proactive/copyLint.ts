import { ProactiveItem } from "./types";

export const COPY_LIMITS = {
	MAX_TITLE: 16,
	MAX_DESC: 36,
	MAX_BUTTON: 10,
	MAX_BUBBLE: 40
} as const;

export function assertTitle(title: string): string {
	if (title.length > COPY_LIMITS.MAX_TITLE) {
		throw new Error(
			`Proactive title exceeds limit: "${title}" (${title.length}/${COPY_LIMITS.MAX_TITLE})`
		);
	}
	return title;
}

export function assertDesc(desc: string): string {
	if (desc.length > COPY_LIMITS.MAX_DESC) {
		throw new Error(
			`Proactive desc exceeds limit: "${desc}" (${desc.length}/${COPY_LIMITS.MAX_DESC})`
		);
	}
	return desc;
}

export function assertButton(label: string): string {
	if (label.length > COPY_LIMITS.MAX_BUTTON) {
		throw new Error(
			`Proactive button label exceeds limit: "${label}" (${label.length}/${COPY_LIMITS.MAX_BUTTON})`
		);
	}
	return label;
}

export function assertBubble(text: string): string {
	if (text.length > COPY_LIMITS.MAX_BUBBLE) {
		throw new Error(
			`Proactive bubble text exceeds limit: "${text}" (${text.length}/${COPY_LIMITS.MAX_BUBBLE})`
		);
	}
	return text;
}

export function validateProactiveItem(item: ProactiveItem): boolean {
	assertTitle(item.title);
	assertDesc(item.description);
	assertBubble(item.bubbleText);
	if (item.actions) {
		if (item.actions.length > 3) {
			throw new Error(`Proactive actions cannot exceed 3: found ${item.actions.length}`);
		}
		for (const act of item.actions) {
			assertButton(act.label);
		}
	}
	return true;
}

/**
 * Pre-validated canonical catalog situations from 11-PROACTIVE-INTERACTION.md §3.
 */
export const CATALOG_TEMPLATES = {
	fileDuplicate: {
		situationId: "file_duplicate",
		level: "tanya" as const,
		title: "Unduhan Ganda", // 13 chars
		description: "File sudah ada. Ganti?", // 22 chars
		bubbleText: "File sudah ada. Ganti?", // 22 chars
		expression: "penasaran" as const,
		actions: [
			{ id: "replace", label: "Ganti", isPrimary: true }, // 5 chars
			{ id: "keep_both", label: "Simpan Dua" }, // 10 chars
			{ id: "delete_new", label: "Hapus", isDestructive: true } // 5 chars
		],
		hasRememberOption: true
	},
	storageLow: {
		situationId: "storage_low",
		level: "tanya" as const,
		title: "Storage Penuh", // 13 chars
		description: "Drive C: tersisa 8%", // 19 chars
		bubbleText: "C: tersisa 8%", // 13 chars
		expression: "khawatir" as const,
		actions: [
			{ id: "view", label: "Lihat", isPrimary: true }, // 5 chars
			{ id: "later", label: "Nanti" } // 5 chars
		],
		hasRememberOption: false
	},
	morningBriefing: {
		situationId: "morning_briefing",
		level: "penting" as const,
		title: "Ringkasan Pagi", // 14 chars
		description: "3 tugas hari ini. Belajar 19:00", // 31 chars
		bubbleText: "3 tugas hari ini. Belajar 19:00", // 31 chars
		expression: "senang" as const,
		actions: [
			{ id: "view", label: "Lihat", isPrimary: true }, // 5 chars
			{ id: "close", label: "Tutup" } // 5 chars
		],
		hasRememberOption: false
	},
	eveningRecap: {
		situationId: "evening_recap",
		level: "colek" as const,
		title: "Rekap Malam", // 11 chars
		description: "2 selesai, 1 geser besok", // 24 chars
		bubbleText: "2 selesai, 1 geser besok", // 24 chars
		expression: "tenang" as const,
		actions: [
			{ id: "view", label: "Lihat", isPrimary: true }, // 5 chars
			{ id: "close", label: "Tutup" } // 5 chars
		],
		autoDismissSecs: 8,
		hasRememberOption: false
	},
	routineStart: {
		situationId: "routine_start",
		level: "penting" as const,
		title: "Rutinitas Mulai", // 15 chars
		description: "Waktunya belajar 30 mnt", // 23 chars
		bubbleText: "Waktunya belajar 30 mnt", // 23 chars
		expression: "fokus" as const,
		actions: [
			{ id: "start", label: "Mulai", isPrimary: true }, // 5 chars
			{ id: "snooze", label: "Tunda 10m" }, // 9 chars
			{ id: "skip", label: "Lewati" } // 6 chars
		],
		hasRememberOption: false
	},
	routineMissed: {
		situationId: "routine_missed",
		level: "tanya" as const,
		title: "Belajar Lewat", // 13 chars
		description: "Belajar terlewat. Jadwal ulang?", // 31 chars
		bubbleText: "Belajar terlewat. Jadwal ulang?", // 31 chars
		expression: "sedih" as const,
		actions: [
			{ id: "reschedule_tonight", label: "Jam 21:00", isPrimary: true }, // 9 chars
			{ id: "reschedule_tomorrow", label: "Besok" }, // 5 chars
			{ id: "skip", label: "Lewati" } // 6 chars
		],
		hasRememberOption: false
	},
	distractionLimit: {
		situationId: "distraction_limit",
		level: "penting" as const,
		title: "Batas Pengalih", // 14 chars
		description: "Sudah 45 mnt di luar belajar", // 28 chars
		bubbleText: "Sudah 45 mnt di luar belajar", // 28 chars
		expression: "kesal" as const,
		actions: [
			{ id: "resume", label: "Belajar", isPrimary: true }, // 7 chars
			{ id: "snooze", label: "Tunda 5m" }, // 8 chars
			{ id: "pause", label: "Jeda" } // 4 chars
		],
		hasRememberOption: false
	},
	taskCompleted: {
		situationId: "task_completed",
		level: "colek" as const,
		title: "Tugas Selesai", // 13 chars
		description: "Selesai. Mantap.", // 16 chars
		bubbleText: "Selesai. Mantap.", // 16 chars
		expression: "senang" as const,
		actions: [],
		autoDismissSecs: 4,
		hasRememberOption: false
	}
};
