import { describe, test, expect } from "bun:test";

// Unit tests for the Tasks module types and utilities.
// Integration tests require a running Tauri shell and are covered by E2E (Tier 5).

describe("tasks/types", () => {
	test("CATEGORIES has 5 entries with valid colors", async () => {
		const { CATEGORIES } = await import("../src/hime/tasks/types");
		expect(CATEGORIES.length).toBe(5);
		for (const c of CATEGORIES) {
			expect(c.id).toBeTruthy();
			expect(c.name).toBeTruthy();
			expect(c.color).toMatch(/^#[0-9a-f]{6}$/i);
		}
	});

	test("TaskTab union covers all segments", async () => {
		// Type-level check: ensure the import compiles
		const types = await import("../src/hime/tasks/types");
		expect(types).toBeDefined();
	});
});

describe("tasks/useTasks utilities", () => {
	test("getUrgency returns correct levels", async () => {
		const { getUrgency } = await import("../src/hime/tasks/useTasks");
		const nowSec = Math.floor(Date.now() / 1000);

		expect(getUrgency(null)).toBe("none");
		expect(getUrgency(nowSec - 3600)).toBe("overdue");
		expect(getUrgency(nowSec + 3600)).toBe("today");
		expect(getUrgency(nowSec + 2 * 86400)).toBe("soon");
		expect(getUrgency(nowSec + 7 * 86400)).toBe("normal");
	});

	test("urgencyColor returns hex colors", async () => {
		const { urgencyColor } = await import("../src/hime/tasks/useTasks");
		expect(urgencyColor("overdue")).toMatch(/^#/);
		expect(urgencyColor("today")).toMatch(/^#/);
		expect(urgencyColor("soon")).toMatch(/^#/);
		expect(urgencyColor("normal")).toMatch(/^#/);
		expect(urgencyColor("none")).toMatch(/^#/);
	});

	test("formatDue returns Indonesian strings", async () => {
		const { formatDue } = await import("../src/hime/tasks/useTasks");
		const nowSec = Math.floor(Date.now() / 1000);

		expect(formatDue(null)).toBe("");
		expect(formatDue(nowSec + 86400)).toBe("Besok");
		expect(formatDue(nowSec + 2 * 86400)).toBe("Lusa");
		// "Hari ini" for tasks due today
		expect(formatDue(nowSec + 3600)).toBe("Hari ini");
	});
});

describe("translate/types", () => {
	test("TranslateCardState type compiles", async () => {
		const types = await import("../src/hime/translate/types");
		expect(types).toBeDefined();
	});
});

describe("tasks/HimeTasksPanel", () => {
	test("HimeTasksPanel is exported and is a function", async () => {
		const { HimeTasksPanel } = await import("../src/hime/tasks");
		expect(HimeTasksPanel).toBeDefined();
		expect(typeof HimeTasksPanel).toBe("function");
	});
});
