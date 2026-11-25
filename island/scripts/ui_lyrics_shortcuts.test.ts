import { describe, it, expect } from "bun:test";
import { readFileSync } from "fs";
import { join } from "path";

describe("worker-ui-lyrics: KeyboardShortcutsModal, LyricsView, and handleWheel invariants", () => {
	const shortcutsPath = join(__dirname, "../src/components/KeyboardShortcutsModal.tsx");
	const lyricsPath = join(__dirname, "../src/hime/lyrics/LyricsView.tsx");
	const useLyricsPath = join(__dirname, "../src/hime/lyrics/useLyrics.ts");
	const cmpPath = join(__dirname, "../src/CompactMediaPlayer.tsx");
	const appPath = join(__dirname, "../src/App.tsx");
	const dockPath = join(__dirname, "../src/Dock.tsx");
	const overlayPath = join(__dirname, "../src/Overlay.tsx");

	const shortcutsSrc = readFileSync(shortcutsPath, "utf-8");
	const lyricsSrc = readFileSync(lyricsPath, "utf-8");
	const useLyricsSrc = readFileSync(useLyricsPath, "utf-8");
	const cmpSrc = readFileSync(cmpPath, "utf-8");
	const appSrc = readFileSync(appPath, "utf-8");
	const dockSrc = readFileSync(dockPath, "utf-8");
	const overlaySrc = readFileSync(overlayPath, "utf-8");

	describe("1. KeyboardShortcutsModal UI & Aesthetic", () => {
		it("conforms to Hime dark glassmorphism and cyan accent #00D2FF", () => {
			expect(shortcutsSrc).toContain("#00D2FF");
			expect(shortcutsSrc).toContain("backdropFilter");
			expect(shortcutsSrc).toContain("blur(");
		});

		it("contains short concise labels instead of verbose sentences", () => {
			expect(shortcutsSrc).toContain('"Notch"');
			expect(shortcutsSrc).toContain('"Asisten"');
			expect(shortcutsSrc).toContain('"Musik & Lirik"');
			expect(shortcutsSrc).toContain('"Dock"');
			expect(shortcutsSrc).toContain('"Daftar Pintasan"');
			expect(shortcutsSrc).toContain('"Tutup Modal"');
			expect(shortcutsSrc).toContain('"Chat Panjang"');
			expect(shortcutsSrc).toContain('"Keluar"');
		});

		it("uses structured 2-column or responsive grid layout", () => {
			expect(shortcutsSrc).toContain("gridTemplateColumns");
		});

		it("renders glowing <kbd> pills with box-shadow and cyan highlights", () => {
			expect(shortcutsSrc).toContain("<kbd");
			expect(shortcutsSrc).toContain("boxShadow");
			expect(shortcutsSrc).toContain("rgba(0, 210, 255");
		});

		it("preserves test required strings for backwards compatibility", () => {
			expect(shortcutsSrc).toContain("Daftar Pintasan Keyboard");
			expect(shortcutsSrc).toContain("F12");
			expect(shortcutsSrc).toContain("Pause");
			expect(shortcutsSrc).toContain("Ctrl + Shift + H");
			expect(shortcutsSrc).toContain("Ctrl + Shift + M");
			expect(shortcutsSrc).toContain("Ctrl + Shift + D");
		});
	});

	describe("2. LyricsView High Contrast & Fine Offset Controls", () => {
		it("styles active line with crisp pure white #FFFFFF, cyan glow, bold 16px and highlight", () => {
			expect(lyricsSrc).toContain("#FFFFFF");
			expect(lyricsSrc).toContain("#00D2FF");
			expect(lyricsSrc).toContain("#38D6F5");
			expect(lyricsSrc).toContain('16px"');
			expect(lyricsSrc).toContain("isActive ? 700");
			expect(lyricsSrc).toContain("0 0 12px #00D2FF");
			expect(lyricsSrc).toContain("rgba(0, 210, 255, 0.12)");
		});

		it("styles inactive lines with high readability rgba(255, 255, 255, 0.72) and shadow 0 1px 3px rgba(0, 0, 0, 0.85)", () => {
			expect(lyricsSrc).toContain("rgba(255, 255, 255, 0.72)");
			expect(lyricsSrc).toContain("0 1px 3px rgba(0, 0, 0, 0.85)");
		});

		it("provides fine-grained offset controls: [-0.5s], [-0.2s], [-50ms], [+50ms], [+0.2s], [+0.5s] and [Reset]", () => {
			expect(lyricsSrc).toContain("-0.5s");
			expect(lyricsSrc).toContain("-0.2s");
			expect(lyricsSrc).toContain("-50ms");
			expect(lyricsSrc).toContain("+50ms");
			expect(lyricsSrc).toContain("+0.2s");
			expect(lyricsSrc).toContain("+0.5s");
			expect(lyricsSrc).toContain("Reset");
			expect(lyricsSrc).toContain("onAdjustOffset(-500)");
			expect(lyricsSrc).toContain("onAdjustOffset(-200)");
			expect(lyricsSrc).toContain("onAdjustOffset(-50)");
			expect(lyricsSrc).toContain("onAdjustOffset(50)");
			expect(lyricsSrc).toContain("onAdjustOffset(200)");
			expect(lyricsSrc).toContain("onAdjustOffset(500)");
			expect(lyricsSrc).toContain("onResetOffset");
		});

		it("preserves scrollbar suppression styles required by global invariants", () => {
			expect(lyricsSrc).toContain('scrollbarWidth: "none"');
			expect(lyricsSrc).toContain('msOverflowStyle: "none"');
			expect(lyricsSrc).toContain('overflowX: "hidden"');
		});

		it("ensures relative positioning on container for accurate offsetTop autoscroll centering", () => {
			expect(lyricsSrc).toContain('position: "relative"');
		});
	});

	describe("3. App.tsx handleWheel Navigation Protection", () => {
		it("prevents card scrolling from switching notch panels when isOpen === true", () => {
			expect(appSrc).toContain("if (isOpen)");
			expect(appSrc).toContain('.closest(".time-center")');
			expect(appSrc).toContain('.closest(".main-row-inner")');
		});
	});

	describe("4. Music Lyrics Timing Precision & Audio Latency Compensation", () => {
		it("ticks displayPosition smoothly at 50ms in CompactMediaPlayer", () => {
			expect(cmpSrc).toContain("setInterval(tick, 50)");
		});

		it("passes Date.now() as positionUpdatedAt to prevent double-extrapolation", () => {
			expect(cmpSrc).toContain("positionUpdatedAt: Date.now()");
		});

		it("applies DEFAULT_LATENCY_OFFSET_MS = 0 for accurate 1:1 lyric alignment without premature line jumping", () => {
			expect(useLyricsSrc).toContain("DEFAULT_LATENCY_OFFSET_MS = 0");
			expect(useLyricsSrc).toContain("targetPos + DEFAULT_LATENCY_OFFSET_MS");
		});
	});

	describe("5. Hover Persist, Auto-Hide Delay & Click-Outside Dismissal", () => {
		it("App.tsx notch uses 2.5s unhover timer and click-outside dismissal", () => {
			expect(appSrc).toContain("2500");
			expect(appSrc).toContain('window.addEventListener("pointerdown", handleGlobalPointerDown');
		});

		it("Dock.tsx uses 2.5s unhover grace timer and click-outside dismissal", () => {
			expect(dockSrc).toContain("2500");
			expect(dockSrc).toContain('window.addEventListener("pointerdown", handleDockOutsideClick');
			expect(dockSrc).toContain('listen("dock-dismiss"');
			expect(dockSrc).toContain("setIsUserDismissed(true)");
		});

		it("Overlay.tsx volume/brightness sliders support hover lock, 2.5s unhover timer, and click-outside dismissal", () => {
			expect(overlaySrc).toContain("2500");
			expect(overlaySrc).toContain("onMouseEnter");
			expect(overlaySrc).toContain("onMouseLeave");
			expect(overlaySrc).toContain('window.addEventListener("pointerdown", handleOverlayOutsideClick');
			expect(overlaySrc).toContain('listen("overlay-dismiss"');
		});
	});

	describe("6. Music Side Icons, Music Panel Access & Keyboard Shortcuts Button", () => {
		it("renders collapsed music visualizer and album art via showSideMusicIcons without isMusicMode gating", () => {
			expect(appSrc).toContain("{showSideMusicIcons && settingsVisualizerEnabled ?");
			expect(appSrc).toContain("{showSideMusicIcons &&");
			expect(appSrc).toContain("showMusicBubble = musicActivity && (timerActive || isVerticalCollapsed)");
			expect(appSrc).toContain('const isMusicPanelOpen = isOpen && bloomMode === "music";');
			expect(appSrc).toContain("Tidak ada media yang sedang diputar");
		});

		it("provides direct UI buttons and Slash keycode fallback for KeyboardShortcutsModal", () => {
			expect(appSrc).toContain('e.code === "Slash"');
			expect(appSrc).toContain('aria-label="Daftar Pintasan Keyboard"');
			expect(appSrc).toContain('setBloomMode("shortcuts")');
		});

		it("wakes hidden/unvisible notch for hotkeys including hime-toggle-assistant and prevents stale lyrics on track change", () => {
			expect(appSrc).toContain("const wakeNotchForHotkey = () => {");
			expect(appSrc).toContain("wakeNotchForHotkey();");
			expect(useLyricsSrc).toContain("const isNewTrack = trackKey !== lastTrackRef.current;");
			expect(cmpSrc).toContain("Math.abs(drift) > 850");
		});

		it("enforces expanded music safe padding (>= 20px horizontal), unconditional music panel sizing, and NotchErrorBoundary", () => {
			const appCssSrc = readFileSync(join(__dirname, "../src/App.css"), "utf-8");
			const expandedMatch = appCssSrc.match(/\.expanded-music-container\s*\{[^}]*padding:\s*(\d+)px\s+(\d+)px\s+(\d+)px\s+(\d+)px/);
			expect(expandedMatch).toBeTruthy();
			expect(Number(expandedMatch![1])).toBeGreaterThanOrEqual(10);
			expect(Number(expandedMatch![2])).toBeGreaterThanOrEqual(24);
			expect(Number(expandedMatch![3])).toBeGreaterThanOrEqual(12);
			expect(Number(expandedMatch![4])).toBeGreaterThanOrEqual(24);

			expect(appSrc).toContain("if (isMusicPanelOpen)");
			expect(appSrc).toContain("return mediaLayout === \"compact\" ? 396 : 360;");
			expect(appSrc).toContain("class NotchErrorBoundary extends Component");
			expect(appSrc).toContain("<NotchErrorBoundary");
		});
	});
});
