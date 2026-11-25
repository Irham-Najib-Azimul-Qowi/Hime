import { describe, expect, it } from "bun:test";
import { readFileSync } from "fs";
import { join } from "path";
import { HIME_EXPRESSIONS, resolveEyeExpression, EXPRESSION_ALIASES } from "../src/hime/ui/HimeEyes";

describe("Mascot Eyes 22 Expressions Catalog and Periodic Notch Invariants", () => {
	const appTsx = readFileSync(join(__dirname, "../src/App.tsx"), "utf-8");
	const eyesJson = JSON.parse(readFileSync(join(__dirname, "../src/hime/assets/eyes.json"), "utf-8"));

	it("eyes.json contains exactly 22 expressions conforming to Hime catalog", () => {
		const expressionKeys = Object.keys(eyesJson.expressions);
		expect(expressionKeys.length).toBe(22);
		expect(HIME_EXPRESSIONS.length).toBe(22);

		for (const key of HIME_EXPRESSIONS) {
			expect(expressionKeys).toContain(key);
		}
	});

	it("resolveEyeExpression resolves all 22 expressions and canonical aliases", () => {
		// All 22 canonical expressions
		for (const key of HIME_EXPRESSIONS) {
			expect(resolveEyeExpression(key)).toBe(key);
		}

		// Aliases
		expect(resolveEyeExpression("panik")).toBe("kaget");
		expect(resolveEyeExpression("lelah")).toBe("ngantuk");
		expect(resolveEyeExpression("waspada")).toBe("curiga");
		expect(resolveEyeExpression("ceria")).toBe("semangat");
		expect(resolveEyeExpression("senyum")).toBe("senang");
		expect(resolveEyeExpression("kedip")).toBe("kedip-sebelah");
		expect(resolveEyeExpression("lirik_kiri")).toBe("penasaran");
		expect(resolveEyeExpression("lirik_kanan")).toBe("curiga");
	});

	it("App.tsx implements periodic and condition-based eyes replacing the clock in resting notch", () => {
		// Periodic timer ~18s
		expect(appTsx).toContain("glanceIndexRef.current = (glanceIndexRef.current + 1) % idleGlanceCatalog.length;");
		expect(appTsx).toContain("18000");

		// Default clock preservation: showCenterEyes is strictly periodicEyesActive
		expect(appTsx).toContain("const showCenterEyes = periodicEyesActive;");

		// Condition mappings
		expect(appTsx).toContain('if (!wifiConnected && !isEthernetConnected)');
		expect(appTsx).toContain('return "offline";');

		expect(appTsx).toContain('if (batteryLevel !== null && batteryLevel < lowBatteryThreshold && !isCharging)');
		expect(appTsx).toContain('return "panik";');

		expect(appTsx).toContain('if (isCharging)');
		expect(appTsx).toContain('return "senang";');

		expect(appTsx).toContain('if (hasOverdueTasks)');
		expect(appTsx).toContain('return "waspada";');

		expect(appTsx).toContain('if (hasPendingTasks)');
		expect(appTsx).toContain('return "fokus";');

		expect(appTsx).toContain('if (isPlaying)');
		expect(appTsx).toContain('return "dengar";');

		// Idle glance rotation
		expect(appTsx).toContain('"senyum"');
		expect(appTsx).toContain('"kedip"');
		expect(appTsx).toContain('"penasaran"');
		expect(appTsx).toContain('"lirik_kiri"');
		expect(appTsx).toContain('"lirik_kanan"');
	});

	it("App.tsx renders HimeEyes with size 20 in resting notch center header", () => {
		expect(appTsx).toContain("size={20}");
		expect(appTsx).toContain("expression={notchEyeExpression}");
		expect(appTsx).toContain('onClick={() => openNotch("hime")}');
	});

	it("HimeEyes and tokens.css implement realistic wink animation for kedip-sebelah", () => {
		const himeEyesTsx = readFileSync(join(__dirname, "../src/hime/ui/HimeEyes.tsx"), "utf-8");
		const tokensCss = readFileSync(join(__dirname, "../src/hime/styles/tokens.css"), "utf-8");
		const appCss = readFileSync(join(__dirname, "../src/App.css"), "utf-8");

		// HimeEyes dynamically wraps winking right eye with realistic opening and closing animation wrapper
		expect(himeEyesTsx).toContain('const isWink = resolvedExprKey === "kedip-sebelah" && !isOffline;');
		expect(himeEyesTsx).toContain("isWinkingRightEye");
		expect(himeEyesTsx).toContain("hime-eye-wink-wrap");
		expect(himeEyesTsx).toContain("hime-wink-right");

		// tokens.css defines natural rhythm keyframes (scaleY closing down to slit and springing open with bounce overshoot)
		expect(tokensCss).toContain("@keyframes hime-wink-right");
		expect(tokensCss).toContain(".hime-eye-wink-wrap");
		expect(tokensCss).toContain("scaleY(0.08)");
		expect(tokensCss).toContain("scaleY(1.18)");
		expect(tokensCss).toContain("scaleY(1)");
	});

	it("HimeEyes and tokens.css implement cartoon sleep floating Zzz for tidur", () => {
		const himeEyesTsx = readFileSync(join(__dirname, "../src/hime/ui/HimeEyes.tsx"), "utf-8");
		const tokensCss = readFileSync(join(__dirname, "../src/hime/styles/tokens.css"), "utf-8");

		// HimeEyes conditionally renders animated floating Zzz letters while sleeping
		expect(himeEyesTsx).toContain('const isSleeping = resolvedExprKey === "tidur" && !isOffline;');
		expect(himeEyesTsx).toContain("hime-sleep-zzz-container");
		expect(himeEyesTsx).toContain("hime-zzz-letter");
		expect(himeEyesTsx).toContain("hime-zzz-1");
		expect(himeEyesTsx).toContain("hime-zzz-2");
		expect(himeEyesTsx).toContain("hime-zzz-3");

		// Drift trajectory, soft cyan glow and fading opacity
		expect(tokensCss).toContain("@keyframes hime-float-zzz");
		expect(tokensCss).toContain(".hime-sleep-zzz-container");
		expect(tokensCss).toContain(".hime-zzz-letter");
		expect(tokensCss).toContain("text-shadow");
		expect(tokensCss).toContain("rgba(56, 214, 245");
	});

	it("App.tsx and App.css implement cybernetic digital glitch transition for Clock <-> Eyes", () => {
		const appCss = readFileSync(join(__dirname, "../src/App.css"), "utf-8");

		// Digital glitch keyframes with chromatic offset flicker and scanlines
		expect(appCss).toContain("@keyframes hime-glitch-in");
		expect(appCss).toContain("@keyframes hime-glitch-out");
		expect(appCss).toContain(".hime-glitch-in");
		expect(appCss).toContain(".hime-glitch-out");
		expect(appCss).toContain("hime-glitch-scanlines");
		expect(appCss).toContain("repeating-linear-gradient");
		expect(appCss).toContain("drop-shadow(-2px 0 1px rgba(56, 214, 245, 0.8))");

		// App.tsx applies glitch transitions between mascot eyes and clock in resting notch
		expect(appTsx).toContain('key="hime-eyes"');
		expect(appTsx).toContain('className="time hime-eyes-center"');
		expect(appTsx).toContain('key="clock"');
		expect(appTsx).toContain('className="time hime-glitch-in"');
	});
});
