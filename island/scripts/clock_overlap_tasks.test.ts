import { describe, expect, it } from "bun:test";
import { readFileSync } from "fs";
import { join } from "path";

describe("Resting Notch Clock Preservation, Text Overlap Audit & Tasks Error Handling", () => {
	const appTsx = readFileSync(join(__dirname, "../src/App.tsx"), "utf-8");
	const appCss = readFileSync(join(__dirname, "../src/App.css"), "utf-8");
	const taskCardTsx = readFileSync(join(__dirname, "../src/hime/tasks/TaskCard.tsx"), "utf-8");
	const quickAddBarTsx = readFileSync(join(__dirname, "../src/hime/tasks/QuickAddBar.tsx"), "utf-8");
	const useTasksTs = readFileSync(join(__dirname, "../src/hime/tasks/useTasks.ts"), "utf-8");
	const tasksPanelTsx = readFileSync(join(__dirname, "../src/hime/tasks/HimeTasksPanel.tsx"), "utf-8");

	it("App.tsx collapsedMode is always 'status' so resting collapsed notch keeps centered clock", () => {
		expect(appTsx).toContain('const collapsedMode = "status";');
		expect(appTsx).not.toContain('const collapsedMode = musicActivity && !timerActive ? "music" : "status";');
	});

	it("App.tsx shows compact visualizer on side when music is playing without replacing center clock", () => {
		expect(appTsx).toContain("const showCompactMusicIcons = isMusicMode || showMusicOnStatusHover || (isPlaying && settingsMusicModeEnabled);");
		expect(appTsx).toContain("time-center");
		expect(appCss).toContain("flex-shrink: 0");
	});

	it("TaskCard.tsx and QuickAddBar.tsx apply minWidth 0 and ellipsis to prevent text overlapping", () => {
		// TaskCard title span
		expect(taskCardTsx).toContain("minWidth: 0");
		expect(taskCardTsx).toContain('overflow: "hidden"');
		expect(taskCardTsx).toContain('textOverflow: "ellipsis"');
		expect(taskCardTsx).toContain('whiteSpace: "nowrap"');

		// QuickAddBar input field
		expect(quickAddBarTsx).toContain("minWidth: 0");
	});

	it("useTasks.ts intercepts raw Named Pipe errors and displays friendly status with auto-retry", () => {
		expect(useTasksTs).toContain("named pipe");
		expect(useTasksTs).toContain("cannot find the file specified");
		expect(useTasksTs).toContain('setError("Menghubungkan ke layanan Hime...");');
		expect(useTasksTs).toContain("1500");
	});

	it("HimeTasksPanel.tsx renders graceful error message and retry button with refresh()", () => {
		expect(tasksPanelTsx).toContain("refresh");
		expect(tasksPanelTsx).toContain("Coba lagi");
		expect(tasksPanelTsx).toContain("refresh()");
		expect(tasksPanelTsx).toContain('textOverflow: "ellipsis"');
	});
});
