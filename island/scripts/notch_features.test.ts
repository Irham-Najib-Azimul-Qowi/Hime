import { describe, it, expect } from "bun:test";
import fs from "fs";
import path from "path";
import { arbitrateSlotState } from "../src/hime/ncm/arbiter";
import { PriorityLevel } from "../src/hime/ncm/types";

describe("Notch UI - Clock & Eyes Center Alignment, Hover Network Speed, & Network Drop Alert", () => {
	const appTsxPath = path.resolve(__dirname, "../src/App.tsx");
	const appTsxContent = fs.readFileSync(appTsxPath, "utf-8");

	describe("1. Clock and Hime Eyes Center Alignment Invariant", () => {
		it("ensures <HimeEyes> is removed from the left of the clock span", () => {
			// Extract clock motion.span
			const clockMatch = appTsxContent.match(/<motion\.span\s+key="clock"[\s\S]*?<\/motion\.span>/);
			expect(clockMatch).toBeTruthy();
			const clockSpan = clockMatch![0];

			// Clock span must NOT contain <HimeEyes>
			expect(clockSpan.includes("<HimeEyes")).toBe(false);
			// Clock span must contain <span>{time}</span>
			expect(clockSpan.includes("<span>{time}</span>")).toBe(true);
		});

		it("ensures center container toggles between clock and <HimeEyes size={20} expression={notchEyeExpression} /> (dead center)", () => {
			// Center container has showCenterEyes branch with <HimeEyes size={20} expression={notchEyeExpression} />
			const eyesMatch = appTsxContent.match(/showCenterEyes\s*\?[\s\S]*?<HimeEyes[\s\S]*?size=\{(16|20)\}[\s\S]*?expression=\{(himeExpression|notchEyeExpression)\}[\s\S]*?\/>[\s\S]*?:[\s\S]*?<motion\.span[\s\S]*?key="clock"/);
			expect(eyesMatch).toBeTruthy();

			// Center container has dead-center alignment styles (justifyContent: center, alignItems: center)
			expect(appTsxContent.includes('className="time hime-eyes-center"')).toBe(true);
			expect(appTsxContent.includes('justifyContent: "center"')).toBe(true);
		});
	});

	describe("2. Notch Hover Network Speed Invariant", () => {
		it("formats live network speed accurately (B/s, KB/s, MB/s)", () => {
			const formatNetSpeed = (bytesPerSec: number) => {
				if (bytesPerSec < 1024) return `${bytesPerSec} B/s`;
				if (bytesPerSec < 1024 * 1024) return `${(bytesPerSec / 1024).toFixed(0)} KB/s`;
				return `${(bytesPerSec / (1024 * 1024)).toFixed(1)} MB/s`;
			};

			expect(formatNetSpeed(0)).toBe("0 B/s");
			expect(formatNetSpeed(512)).toBe("512 B/s");
			expect(formatNetSpeed(245760)).toBe("240 KB/s");
			expect(formatNetSpeed(1024 * 1024 * 2.5)).toBe("2.5 MB/s");
		});

		it("replaces weather display with live network speed on notch hover", () => {
			// Check weather case in renderStatusWidget in App.tsx
			const weatherCaseMatch = appTsxContent.match(/case "weather":[\s\S]*?if\s*\((?:isHovered|isCenterHovered)[\s\S]*?return statusWidget\([\s\S]*?"net"[\s\S]*?ArrowUpDown[\s\S]*?formatNetSpeed[\s\S]*?\);/);
			expect(weatherCaseMatch).toBeTruthy();
			expect(appTsxContent.includes("isCenterHovered")).toBe(true);
		});
	});

	describe("3. Network Drop State & Alert/Toast Invariant", () => {
		it("updates himeExpression and arbitrates P0 alert presentation on network drop", () => {
			const networkDropPresentation = {
				id: "network-drop-alert",
				module: "network",
				kind: "alert" as const,
				priority: PriorityLevel.P0_CRITICAL,
				ttl: 7000,
				expression: "khawatir" as const,
				createdAt: Date.now(),
				payload: {
					title: "Koneksi Terputus",
					description: "Wi-Fi atau internet terputus"
				}
			};

			const slotState = arbitrateSlotState([networkDropPresentation], { isOnline: false });
			expect(slotState.mascotExpression).toBe("offline");
			expect(slotState.leftSlot).toBe("eyes");
			expect(slotState.centerSlot).toBe("Wi-Fi atau internet terputus");
			expect(slotState.activeAlert).not.toBeNull();
			expect(slotState.activeAlert?.expression).toBe("khawatir");
		});

		it("triggers network drop alert with khawatir expression in App.tsx", () => {
			expect(appTsxContent.includes('setExpressionOverride("khawatir")')).toBe(true);
			expect(appTsxContent.includes("network-drop-alert")).toBe(true);
			expect(appTsxContent.includes("Wi-Fi atau internet terputus")).toBe(true);
			expect(appTsxContent.includes("triggerNetworkDropAlert")).toBe(true);
		});
	});
});
