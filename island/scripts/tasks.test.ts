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

	test("QuickAddBar is exported and is a function", async () => {
		const { QuickAddBar } = await import("../src/hime/tasks/QuickAddBar");
		expect(QuickAddBar).toBeDefined();
		expect(typeof QuickAddBar).toBe("function");
	});

	test("TaskCard is exported and is a function", async () => {
		const { TaskCard } = await import("../src/hime/tasks/TaskCard");
		expect(TaskCard).toBeDefined();
		expect(typeof TaskCard).toBe("function");
	});
});

describe("Quick Settings & Task UI Layout Invariants", () => {
	test("App.css configures 2 balanced rows of 5 icons for quick toggles", async () => {
		const fs = await import("fs");
		const path = await import("path");
		const css = fs.readFileSync(path.resolve(__dirname, "../src/App.css"), "utf8");

		expect(css).toContain(".cc-circular-actions-row");
		expect(css).toMatch(/grid-template-columns:\s*repeat\(5,\s*1fr\)/);
		expect(css).toMatch(/gap:\s*10px\s*14px/);
	});

	test("Quick settings buttons use rounded pill geometry and size 36px", async () => {
		const fs = await import("fs");
		const path = await import("path");
		const css = fs.readFileSync(path.resolve(__dirname, "../src/App.css"), "utf8");

		expect(css).toMatch(/\.cc-circular-btn\s*\{[^}]*width:\s*36px/);
		expect(css).toMatch(/\.cc-circular-btn\s*\{[^}]*border-radius:\s*9999px/);
	});

	test("QuickAddBar contains dark menu #121214 styling for category options", async () => {
		const fs = await import("fs");
		const path = await import("path");
		const quickAddBarSrc = fs.readFileSync(path.resolve(__dirname, "../src/hime/tasks/QuickAddBar.tsx"), "utf8");

		expect(quickAddBarSrc).toContain("#121214");
		expect(quickAddBarSrc).toContain("Pilih Kategori");
		expect(quickAddBarSrc).toContain("Tanpa Kategori");
		expect(quickAddBarSrc).toContain("+ Tambah tugas…");
	});

	test("App.tsx accommodates 2-row command-center height", async () => {
		const fs = await import("fs");
		const path = await import("path");
		const appSrc = fs.readFileSync(path.resolve(__dirname, "../src/App.tsx"), "utf8");

		expect(appSrc).toContain('if (bloomMode === "command-center") return isOpen ? 276 : 36;');
	});

	test("App.tsx and App.css hide visual chevron arrows beside clock while keeping wheel navigation", async () => {
		const fs = await import("fs");
		const path = await import("path");
		const appSrc = fs.readFileSync(path.resolve(__dirname, "../src/App.tsx"), "utf8");
		const css = fs.readFileSync(path.resolve(__dirname, "../src/App.css"), "utf8");

		expect(appSrc).toContain("const showPanelArrows = false;");
		expect(appSrc).toContain("handleWheel");
		expect(appSrc).toContain("cycleBloomMode");
		expect(css).toMatch(/\.panel-arrow\s*\{[^}]*display:\s*none\s*!important/);
	});

	test("CATEGORIES defines vibrant luminous accent colors (purple, cyan, mint, coral, amber)", async () => {
		const { CATEGORIES } = await import("../src/hime/tasks/types");
		const colors = CATEGORIES.map((c) => c.color.toUpperCase());
		expect(colors).toContain("#8B5CF6"); // Luminous Purple
		expect(colors).toContain("#06B6D4"); // Luminous Cyan
		expect(colors).toContain("#10B981"); // Luminous Mint
		expect(colors).toContain("#F43F5E"); // Luminous Coral
		expect(colors).toContain("#F59E0B"); // Luminous Amber
	});

	test("TaskCard and HimeTasksPanel include colorful luminous badge and segmented tab styles", async () => {
		const fs = await import("fs");
		const path = await import("path");
		const cardSrc = fs.readFileSync(path.resolve(__dirname, "../src/hime/tasks/TaskCard.tsx"), "utf8");
		const panelSrc = fs.readFileSync(path.resolve(__dirname, "../src/hime/tasks/HimeTasksPanel.tsx"), "utf8");

		expect(cardSrc).toContain("category.color");
		expect(cardSrc).toContain("boxShadow");
		expect(cardSrc).toContain("borderRadius: 9999");
		expect(panelSrc).toContain("linear-gradient");
		expect(panelSrc).toContain("hari ini");
	});
});
