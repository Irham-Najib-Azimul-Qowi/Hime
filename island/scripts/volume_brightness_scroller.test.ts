import { describe, expect, test } from "bun:test";
import { readFileSync } from "fs";
import { resolve } from "path";

describe("Volume & Brightness Bars & Scroller Interactions", () => {
	test("Volume wheel scroller step and clamp calculation", () => {
		const calculateVolumeWheel = (currentVol: number, deltaY: number) => {
			const delta = deltaY < 0 ? 0.04 : -0.04;
			return Math.max(0, Math.min(1, Math.round((currentVol + delta) * 100) / 100));
		};

		// Normal scroll UP (+4%)
		expect(calculateVolumeWheel(0.5, -100)).toBe(0.54);
		// Normal scroll DOWN (-4%)
		expect(calculateVolumeWheel(0.5, 100)).toBe(0.46);

		// Clamp at top (100%)
		expect(calculateVolumeWheel(0.98, -100)).toBe(1.0);
		expect(calculateVolumeWheel(1.0, -100)).toBe(1.0);

		// Clamp at bottom (0%)
		expect(calculateVolumeWheel(0.02, 100)).toBe(0.0);
		expect(calculateVolumeWheel(0.0, 100)).toBe(0.0);
	});

	test("Brightness wheel scroller step and clamp calculation", () => {
		const calculateBrightnessWheel = (currentBright: number, deltaY: number) => {
			const delta = deltaY < 0 ? 4 : -4;
			return Math.max(0, Math.min(100, Math.round(currentBright + delta)));
		};

		// Normal scroll UP (+4)
		expect(calculateBrightnessWheel(50, -100)).toBe(54);
		// Normal scroll DOWN (-4)
		expect(calculateBrightnessWheel(50, 100)).toBe(46);

		// Clamp at top (100)
		expect(calculateBrightnessWheel(98, -100)).toBe(100);
		expect(calculateBrightnessWheel(100, -100)).toBe(100);

		// Clamp at bottom (0)
		expect(calculateBrightnessWheel(2, 100)).toBe(0);
		expect(calculateBrightnessWheel(0, 100)).toBe(0);
	});

	test("Overlay.css contains vibrant gradients for volume and brightness", () => {
		const overlayCss = readFileSync(resolve(__dirname, "../src/Overlay.css"), "utf-8");

		// Volume: vibrant electric cyan to dynamic blue-violet
		expect(overlayCss).toContain("#38D6F5");
		expect(overlayCss).toContain("#4361EE");
		expect(overlayCss).toContain(".volume-notch-fill");
		expect(overlayCss).toContain(".volume-mixer-fill");

		// Brightness: warm amber to sunburst to gold-crimson
		expect(overlayCss).toContain("#FFB703");
		expect(overlayCss).toContain("#FB8500");
		expect(overlayCss).toContain("#E50914");
		expect(overlayCss).toContain(".brightness-notch-fill");
	});

	test("App.css contains vibrant gradients for notch sliders", () => {
		const appCss = readFileSync(resolve(__dirname, "../src/App.css"), "utf-8");

		// Music player volume slider
		expect(appCss).toContain(".slider-progress-fill");
		expect(appCss).toContain("#38D6F5");
		expect(appCss).toContain("#4361EE");

		// Control Center volume and brightness
		expect(appCss).toContain(".cc-volume-fill");
		expect(appCss).toContain(".cc-brightness-fill");
		expect(appCss).toContain("#FFB703");
		expect(appCss).toContain("#FB8500");
	});

	test("Overlay.tsx contains onWheel handlers on bars and tiles", () => {
		const overlayTsx = readFileSync(resolve(__dirname, "../src/Overlay.tsx"), "utf-8");

		expect(overlayTsx).toContain("onWheel={handleWheel}");
		expect(overlayTsx).toContain("volume-mixer-bar");
		expect(overlayTsx).toContain("volume-notch-bar");
		expect(overlayTsx).toContain("brightness-notch-bar");
	});

	test("App.tsx contains onWheel handlers on volume and brightness notch sliders", () => {
		const appTsx = readFileSync(resolve(__dirname, "../src/App.tsx"), "utf-8");

		expect(appTsx).toContain("handleVolumeWheel");
		expect(appTsx).toContain("handleBrightnessWheel");
		expect(appTsx).toContain("volume-slider-container");
		expect(appTsx).toContain("cc-volume-slider-row");
		expect(appTsx).toContain("cc-brightness-slider-row");
	});
});
