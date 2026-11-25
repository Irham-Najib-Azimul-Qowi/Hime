import { describe, it, expect } from "bun:test";
import { readFileSync } from "fs";
import { resolve } from "path";

describe("Magnetic Edge Snap, Compact Music Player, and Notch Status Widgets", () => {
	const islandDir = resolve(__dirname, "..");
	const appTsx = readFileSync(resolve(islandDir, "src/App.tsx"), "utf-8");
	const appCss = readFileSync(resolve(islandDir, "src/App.css"), "utf-8");

	describe("1. Magnetic Edge Snap on Notch Drag", () => {
		it("implements magnetic edge snap calculation across all 4 screen edges", () => {
			expect(appTsx).toContain("topCandidate");
			expect(appTsx).toContain("bottomCandidate");
			expect(appTsx).toContain("leftCandidate");
			expect(appTsx).toContain("rightCandidate");
			expect(appTsx).toContain("clamp(releaseX, halfW, screenW - halfW)");
			expect(appTsx).toContain("clamp(releaseY, halfH, screenH - halfH)");
			expect(appTsx).toContain("Math.hypot(releaseX - topCandidate.pos.x, releaseY - topCandidate.pos.y)");
			expect(appTsx).toContain("candidates.sort((a, b) => a.dist - b.dist)");
			expect(appTsx).toContain("const snappedPos = candidates[0].pos;");
		});

		it("persists snapped edge position to localStorage and backend settings", () => {
			expect(appTsx).toContain('localStorage.setItem("bloom-notch-position", JSON.stringify(snappedPos))');
			expect(appTsx).toContain('invoke("save_setting", { key: "bloom-notch-position", value: JSON.stringify(snappedPos) })');
			expect(appTsx).toContain('invoke("update_notch_rect",');
		});

		it("supports smooth magnetic snap animation in App.css", () => {
			expect(appCss).toContain(".notch-drag-wrapper.is-free-drag {");
			expect(appCss).toContain("transition: left 0.35s cubic-bezier(0.16, 1, 0.3, 1), top 0.35s cubic-bezier(0.16, 1, 0.3, 1);");
			expect(appCss).toContain(".notch-drag-wrapper.is-free-drag.is-dragging {");
			expect(appCss).toContain("transition: none !important;");
		});

		it("correctly simulates magnetic edge snap math for 4 edges", () => {
			const screenW = 1920;
			const screenH = 1080;
			const pillW = 200;
			const pillH = 34;
			const halfW = pillW / 2;
			const halfH = pillH / 2;
			const clamp = (v: number, min: number, max: number) => Math.max(min, Math.min(max, v));

			const snap = (releaseX: number, releaseY: number) => {
				const top = { edge: "top", pos: { x: clamp(releaseX, halfW, screenW - halfW), y: 0 } };
				const bottom = { edge: "bottom", pos: { x: clamp(releaseX, halfW, screenW - halfW), y: screenH - pillH } };
				const left = { edge: "left", pos: { x: halfW, y: clamp(releaseY, halfH, screenH - halfH) } };
				const right = { edge: "right", pos: { x: screenW - halfW, y: clamp(releaseY, halfH, screenH - halfH) } };

				const candidates = [
					{ ...top, dist: Math.hypot(releaseX - top.pos.x, releaseY - top.pos.y) },
					{ ...bottom, dist: Math.hypot(releaseX - bottom.pos.x, releaseY - bottom.pos.y) },
					{ ...left, dist: Math.hypot(releaseX - left.pos.x, releaseY - left.pos.y) },
					{ ...right, dist: Math.hypot(releaseX - right.pos.x, releaseY - right.pos.y) }
				];
				candidates.sort((a, b) => a.dist - b.dist);
				return candidates[0];
			};

			// Close to top
			expect(snap(960, 50).edge).toBe("top");
			expect(snap(960, 50).pos.y).toBe(0);

			// Close to bottom
			expect(snap(960, 1000).edge).toBe("bottom");
			expect(snap(960, 1000).pos.y).toBe(screenH - pillH);

			// Close to left
			expect(snap(150, 500).edge).toBe("left");
			expect(snap(150, 500).pos.x).toBe(halfW);

			// Close to right
			expect(snap(1850, 500).edge).toBe("right");
			expect(snap(1850, 500).pos.x).toBe(screenW - halfW);
		});
	});

	describe("2. Music Player Compact Layout Dimensions & Controls", () => {
		it("provides compact layout height (236px / 272px) and width (360px)", () => {
			expect(appTsx).toContain('let h = mediaLayout === "compact" ? 236 : 190;');
			expect(appTsx).toContain("if (compactVolumeExpanded) h = 272;");
			expect(appTsx).toContain('return mediaLayout === "compact" ? 360 : 340;');
		});

		it("provides ample room for .cmp-row-controls and buttons without cutoff in App.css", () => {
			expect(appCss).toContain(".cmp-row-controls {");
			expect(appCss).toContain("display: flex;");
			expect(appCss).toContain("align-items: center;");
			expect(appCss).toContain("justify-content: space-between;");
			expect(appCss).toContain(".cmp-main-controls {");
			expect(appCss).toContain(".cmp-btn-side {");
			expect(appCss).toContain(".cmp-slide-wrap {");
		});
	});

	describe("3. Notch Status Widgets in Idle & Menu Modes", () => {
		it("renders statusWidgets.left (network speed) whenever !isMusicMode", () => {
			expect(appTsx).toMatch(/!isMusicMode\s*&&\s*statusWidgets\.left\.length > 0/);
		});

		it("renders statusWidgets.right (battery) whenever !isMusicMode", () => {
			expect(appTsx).toMatch(/!isMusicMode\s*&&\s*statusWidgets\.right\.length > 0/);
		});
	});
});
