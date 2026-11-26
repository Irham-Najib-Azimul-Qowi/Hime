import { describe, it, expect } from "bun:test";
import { readFileSync } from "fs";
import { resolve } from "path";

describe("Edge Orientation & Snapping Suite (Main Notch & Volume/Brightness)", () => {
	const islandDir = resolve(__dirname, "..");
	const appTsx = readFileSync(resolve(islandDir, "src/App.tsx"), "utf-8");
	const appCss = readFileSync(resolve(islandDir, "src/App.css"), "utf-8");
	const overlayTsx = readFileSync(resolve(islandDir, "src/Overlay.tsx"), "utf-8");
	const overlayCss = readFileSync(resolve(islandDir, "src/Overlay.css"), "utf-8");
	const edgeSurfaceCss = readFileSync(resolve(islandDir, "src/components/EdgeSurface.css"), "utf-8");

	describe("1. Main Notch Edge Snapping & Vertical Layout", () => {
		it("disallows bottom edge docking and snaps to top, left, or right edges", () => {
			expect(appTsx).toContain("topCandidate");
			expect(appTsx).toContain("bottomCandidate");
			expect(appTsx).toContain("leftCandidate");
			expect(appTsx).toContain("rightCandidate");
			expect(appTsx).toContain('.filter((c) => c.edge !== "bottom")');
		});

		it("allows custom placement along vertical edge with releaseY clamping", () => {
			expect(appTsx).toContain("clamp(releaseY, halfH, screenH - halfH)");
			expect(appTsx).toContain("isVertical");
			expect(appTsx).toContain("currentEdge");
		});

		it("rearranges network speed, clock, and battery widgets into vertical orientation", () => {
			expect(appTsx).toContain("vertical-notch-slot vertical-slot-top");
			expect(appTsx).toContain("vertical-notch-speed");
			expect(appTsx).toContain("vertical-notch-slot vertical-slot-center");
			expect(appTsx).toContain("vertical-notch-clock");
			expect(appTsx).toContain("vertical-notch-slot vertical-slot-bottom");
			expect(appTsx).toContain("vertical-notch-battery");
		});

		it("renders EdgeSurface contour for left, right, and top edges", () => {
			expect(appTsx).toContain("<EdgeSurface");
			expect(appTsx).toContain("edge={currentEdge}");
			expect(appCss).toContain(".bloom.is-vertical.is-edge-left");
			expect(appCss).toContain(".bloom.is-vertical.is-edge-right");
			expect(appCss).toContain("border-radius: 0 18px 18px 0");
			expect(appCss).toContain("border-radius: 18px 0 0 18px");
		});

		it("provides clean styling for main notch vertical mode in App.css", () => {
			expect(appCss).toContain(".main-row-inner.is-vertical");
			expect(appCss).toContain("flex-direction: column !important;");
			expect(appCss).toContain("width: 42px !important;");
			expect(appCss).toContain("height: 196px !important;");
		});

		it("stacks vertical clock, network speed, center alerts, and flush edge auto-hide bar", () => {
			expect(appTsx).toContain("clockParts");
			expect(appTsx).toContain("vertical-clock-stack");
			expect(appTsx).toContain("vertical-clock-part");
			expect(appTsx).toContain("formatNetSpeedParts");
			expect(appTsx).toContain("vertical-speed-value");
			expect(appTsx).toContain("vertical-speed-unit");
			expect(appTsx).toContain("vertical-center-alert");
			expect(appTsx).toContain("vertical-alert-text");
			expect(appTsx).toContain("hasVerticalCenterAlert");
			expect(appCss).toContain("height: 256px !important;");
			expect(appCss).toContain(".notch-drag-wrapper.is-vertical.is-hidden");
			expect(appCss).toContain("width: 5px !important;");
			expect(appTsx).toContain('left: currentEdge === "left" ? 0 : undefined');
			expect(appTsx).toContain('right: currentEdge === "right" ? 0 : undefined');
		});

		it("routes system notification, calendar toggle, and notification center events to main window only", () => {
			expect(appTsx).toContain('listen("hime-toggle-calendar"');
			expect(appTsx).toContain('listen("hime-open-notifications"');
			expect(appTsx).toContain('"hime-system-notification"');
			expect(appTsx).toContain('if (windowLabel !== "main") return;');
		});
	});

	describe("2. Volume & Brightness Notch 3-Edge Snapping & Orientation", () => {
		it("resolves active edge (top, left, right) and horizontal orientation on top edge", () => {
			expect(overlayTsx).toContain("function resolveOverlayEdge(");
			expect(overlayTsx).toContain('if (position.edge === "top" || position.edge === "left" || position.edge === "right")');
			expect(overlayTsx).toContain('const activeEdge = resolveOverlayEdge(position, "left", screenW);');
			expect(overlayTsx).toContain('const activeEdge = resolveOverlayEdge(position, "right", screenW);');
			expect(overlayTsx).toContain('const isHorizontal = activeEdge === "top";');
		});

		it("switches to horizontal layout on top edge with inline buttons and horizontal slider track", () => {
			expect(overlayTsx).toContain("volume-notch-wrapper");
			expect(overlayTsx).toContain("is-horizontal");
			expect(overlayTsx).toContain("brightness-notch-wrapper");
			expect(overlayTsx).toContain("animate={{ width: `${brightness}%`, height: \"100%\" }}");
			expect(overlayTsx).toContain("animate={{ width: isMuted ? \"0%\" : `${percentage}%`, height: \"100%\" }}");
		});

		it("computes horizontal drag coordinates relative to bar width", () => {
			expect(overlayTsx).toContain("relativeX = clientX - rect.left");
			expect(overlayTsx).toContain("relativeX / rect.width");
		});

		it("snaps Volume and Brightness Notches to top, left, or right edge (never bottom) with scale division", () => {
			expect(overlayTsx).toContain("function snapReleaseToEdge(");
			expect(overlayTsx).toContain("const dx = rawDx / safeScale;");
			expect(overlayTsx).toContain("const dy = rawDy / safeScale;");
			expect(overlayTsx).toContain('edge: "top"');
			expect(overlayTsx).toContain('return { x: 0, y: snappedY, edge: "left" };');
			expect(overlayTsx).toContain('edge: "right"');
			expect(overlayTsx).not.toContain('edge: "bottom"');
		});

		it("syncs Volume and Brightness overlay bounds to Rust via useOverlayRectSync", () => {
			expect(overlayTsx).toContain("function useOverlayRectSync(");
			expect(overlayTsx).toContain('useOverlayRectSync("update_volume_overlay_rect"');
			expect(overlayTsx).toContain('useOverlayRectSync("update_brightness_overlay_rect"');
		});

		it("renders EdgeSurface wrapper with activeEdge and flares={!isDragging} on both notches", () => {
			expect(overlayTsx).toContain("<EdgeSurface edge={activeEdge} flares={!isDragging} resolutionScale={safeScale} />");
			expect(overlayTsx).toContain('isDragging ? "is-dragging" : `is-edge-${activeEdge}`');
		});

		it("provides clean edge-docked, horizontal, mixer, and non-intrusive drag handle styles in Overlay.css", () => {
			expect(overlayCss).toContain(".volume-notch-wrapper.is-edge-left");
			expect(overlayCss).toContain(".volume-notch-wrapper.is-edge-right");
			expect(overlayCss).toContain(".volume-notch-wrapper.is-edge-top");
			expect(overlayCss).toContain(".volume-notch-wrapper.is-edge-right .volume-mixer-panel");
			expect(overlayCss).toContain(".volume-notch.is-horizontal .volume-mixer-panel");
			expect(overlayCss).toContain(".volume-notch-wrapper.is-horizontal");
			expect(overlayCss).toContain(".brightness-notch-wrapper.is-horizontal");
			expect(overlayCss).toContain("flex-direction: row;");
			expect(overlayCss).toContain("border-radius: 0 0 18px 18px;");
			expect(overlayCss).toContain("width: 196px;");
			expect(overlayCss).toContain("height: 42px;");
			expect(overlayCss).toContain("position: absolute;");
		});

		it("includes EdgeSurface clip-path path fallbacks for cross-platform rendering", () => {
			expect(edgeSurfaceCss).toContain('clip-path: path("M 0 0 C 0 10 3 13 13 13 L 13 14 L 0 14 Z");');
			expect(edgeSurfaceCss).toContain('clip-path: path("M 0 0 C 10 0 13 3 13 13 L 14 13 L 14 0 Z");');
		});
	});
});
