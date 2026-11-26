import { describe, it, expect } from "bun:test";
import React, { act, useState } from "react";
import * as Client from "react-dom/client";
import {
	PriorityLevel,
	Presentation,
	SlotState,
	arbitrateSlotState,
	calculateEffectivePriority,
	useNCM,
	pickAmbientExpression
} from "../src/hime/ncm";
import { ApprovalItem } from "../src/hime/types";

// Minimal DOM Mock for React 19 Client Act Environment in headless Bun
function setupMockEnv() {
	class MockNode {
		nodeType: number = 1;
		childNodes: any[] = [];
		parentNode: any = null;
		ownerDocument: any = null;
		appendChild(c: any) {
			this.childNodes.push(c);
			c.parentNode = this;
			return c;
		}
		removeChild(c: any) {
			const idx = this.childNodes.indexOf(c);
			if (idx >= 0) this.childNodes.splice(idx, 1);
			c.parentNode = null;
			return c;
		}
		addEventListener() {}
		removeEventListener() {}
	}
	class MockElement extends MockNode {
		tagName: string = "DIV";
		style: any = {};
	}
	class MockHTMLIFrameElement extends MockElement {}

	const doc: any = {
		nodeType: 9,
		documentElement: new MockElement(),
		body: new MockElement(),
		createElement: (tag: string) => {
			const el = new MockElement();
			el.tagName = tag.toUpperCase();
			el.ownerDocument = doc;
			return el;
		},
		createElementNS: (ns: string, tag: string) => doc.createElement(tag),
		createTextNode: (text: string) => ({ nodeType: 3, nodeValue: text, ownerDocument: doc }),
		addEventListener: () => {},
		removeEventListener: () => {},
		activeElement: null
	};
	const win: any = {
		document: doc,
		addEventListener: () => {},
		removeEventListener: () => {},
		setTimeout,
		clearTimeout,
		setInterval,
		clearInterval,
		HTMLIFrameElement: MockHTMLIFrameElement,
		HTMLElement: MockElement,
		Element: MockElement,
		Node: MockNode
	};
	doc.defaultView = win;
	(globalThis as any).document = doc;
	(globalThis as any).window = win;
	(globalThis as any).HTMLIFrameElement = MockHTMLIFrameElement;
	(globalThis as any).HTMLElement = MockElement;
	(globalThis as any).Element = MockElement;
	(globalThis as any).Node = MockNode;
	(globalThis as any).IS_REACT_ACT_ENVIRONMENT = true;

	const container = doc.createElement("div");
	container.ownerDocument = doc;
	return { container, root: Client.createRoot(container as any) };
}

describe("Notch Compose Manager (NCM) Pure Arbiter", () => {
	it("enforces P0 Critical Alert preemption and bubble capping via arbitrateSlotState", () => {
		const presentations: Presentation[] = [
			{
				id: "ambient-eyes",
				module: "mascot",
				kind: "ambient",
				priority: PriorityLevel.P5_AMBIENT,
				expression: "netral",
				createdAt: 1000
			},
			{
				id: "music-bubble",
				module: "music",
				kind: "bubble",
				priority: PriorityLevel.P5_AMBIENT,
				createdAt: 1100
			},
			{
				id: "agent-progress",
				module: "agent",
				kind: "bubble",
				priority: PriorityLevel.P3_PROGRESS,
				createdAt: 1200
			},
			{
				id: "security-approval",
				module: "approvals",
				kind: "alert",
				priority: PriorityLevel.P0_CRITICAL,
				expression: "berpikir",
				payload: { description: "Izin Operasi" },
				createdAt: 1300
			}
		];

		const state = arbitrateSlotState(presentations, { isOnline: true });

		// P0 Critical alert must be activeAlert
		expect(state.activeAlert).not.toBeNull();
		expect(state.activeAlert?.id).toBe("security-approval");
		expect(state.activeAlert?.priority).toBe(PriorityLevel.P0_CRITICAL);
		expect(state.fallbackToClock).toBe(false);
		expect(state.centerSlot).toBe("Izin Operasi");
		expect(state.mascotExpression).toBe("berpikir");

		// Bubbles limited to max 2 and sorted by priority (P3 then P5)
		expect(state.bubbles.length).toBe(2);
		expect(state.bubbles[0].id).toBe("agent-progress");
		expect(state.bubbles[1].id).toBe("music-bubble");
	});

	it("escalates priority by 1 class for userInitiated presentations except P0", () => {
		const normalP2: Presentation = {
			id: "download-query",
			module: "storage",
			kind: "bubble",
			priority: PriorityLevel.P2_ACTION,
			userInitiated: false,
			createdAt: 1000
		};

		const userP2: Presentation = {
			id: "clipboard-copy",
			module: "clipboard",
			kind: "bubble",
			priority: PriorityLevel.P2_ACTION,
			userInitiated: true,
			createdAt: 2000
		};

		const userP0: Presentation = {
			id: "critical-p0",
			module: "security",
			kind: "alert",
			priority: PriorityLevel.P0_CRITICAL,
			userInitiated: true,
			createdAt: 3000
		};

		expect(calculateEffectivePriority(normalP2)).toBe(2);
		expect(calculateEffectivePriority(userP2)).toBe(1); // P2 -> P1
		expect(calculateEffectivePriority(userP0)).toBe(0); // P0 remains P0
	});

	it("preserves fallbackToClock when no active alert is present", () => {
		const emptyPresentations: Presentation[] = [];
		const state = arbitrateSlotState(emptyPresentations, { isOnline: true });

		expect(state.fallbackToClock).toBe(true);
		expect(state.activeAlert).toBeNull();
		expect(state.mascotExpression).toBe("netral");
	});

	it("determines mascot expression correctly across offline, thinking, and senang", () => {
		// 1. Offline overrides everything
		const offlineState = arbitrateSlotState([], { isOnline: false, approvalJustResolved: true });
		expect(offlineState.mascotExpression).toBe("offline");

		// 2. Active approvals force 'berpikir'
		const thinkingState = arbitrateSlotState([], {
			isOnline: true,
			approvals: [
				{
					request_id: "req-1",
					source: "hermes",
					risk_level: "high",
					action_type: "file_delete",
					description: "Hapus file",
					created_at: Date.now(),
					expires_at: Date.now() + 60000
				}
			]
		});
		expect(thinkingState.mascotExpression).toBe("berpikir");

		// 3. Approval just resolved triggers 'senang'
		const senangState = arbitrateSlotState([], {
			isOnline: true,
			approvals: [],
			approvalJustResolved: true
		});
		expect(senangState.mascotExpression).toBe("senang");
		expect(senangState.fallbackToClock).toBe(true);
	});
});

describe("NCM useNCM Approval Synchronization Lifecycle", () => {
	it("cleans up activeAlert, triggers 'senang', and restores fallbackToClock upon approval resolution", async () => {
		const { root } = setupMockEnv();
		let ncmResult: ReturnType<typeof useNCM> | null = null;
		let updateProps: (p: { isOnline: boolean; approvals: ApprovalItem[]; approvalJustResolved: boolean }) => void = () => {};

		const initialApproval: ApprovalItem = {
			request_id: "req-99",
			source: "hermes",
			risk_level: "high",
			action_type: "file_delete",
			description: "Hapus cache sandbox",
			created_at: Date.now(),
			expires_at: Date.now() + 60000
		};

		function TestHarness() {
			const [props, setProps] = useState({
				isOnline: true,
				approvals: [initialApproval],
				approvalJustResolved: false
			});
			updateProps = setProps;
			ncmResult = useNCM(props);
			return null;
		}

		await act(async () => {
			root.render(React.createElement(TestHarness));
		});

		// 1. Initial pending approval state
		expect(ncmResult).not.toBeNull();
		expect(ncmResult!.slotState.activeAlert?.id).toBe("approval-req-99");
		expect(ncmResult!.slotState.fallbackToClock).toBe(false);
		expect(ncmResult!.slotState.mascotExpression).toBe("berpikir");

		// 2. Resolve approval: approvals emptied, approvalJustResolved = true
		await act(async () => {
			updateProps({
				isOnline: true,
				approvals: [],
				approvalJustResolved: true
			});
		});

		// Invariants after approval resolution:
		// Stale presentation is pruned, alert cleared, fallbackToClock is true, and expression is 'senang'
		expect(ncmResult!.slotState.activeAlert).toBeNull();
		expect(ncmResult!.slotState.fallbackToClock).toBe(true);
		expect(ncmResult!.slotState.mascotExpression).toBe("senang");
	});

	it("handles multi-approval resolution correctly by promoting next approval", async () => {
		const { root } = setupMockEnv();
		let ncmResult: ReturnType<typeof useNCM> | null = null;
		let updateProps: (p: { isOnline: boolean; approvals: ApprovalItem[]; approvalJustResolved: boolean }) => void = () => {};

		const appA: ApprovalItem = {
			request_id: "req-A",
			source: "hermes",
			risk_level: "high",
			action_type: "file_delete",
			description: "Hapus A",
			created_at: Date.now(),
			expires_at: Date.now() + 60000
		};
		const appB: ApprovalItem = {
			request_id: "req-B",
			source: "hermes",
			risk_level: "medium",
			action_type: "exec",
			description: "Jalankan B",
			created_at: Date.now() + 100,
			expires_at: Date.now() + 60000
		};

		function MultiHarness() {
			const [props, setProps] = useState({
				isOnline: true,
				approvals: [appA, appB],
				approvalJustResolved: false
			});
			updateProps = setProps;
			ncmResult = useNCM(props);
			return null;
		}

		await act(async () => {
			root.render(React.createElement(MultiHarness));
		});

		// First approval is active
		expect(ncmResult!.slotState.activeAlert?.id).toBe("approval-req-A");

		// Resolve appA -> appB remains
		await act(async () => {
			updateProps({
				isOnline: true,
				approvals: [appB],
				approvalJustResolved: false
			});
		});

		// appA must be pruned, appB promoted to activeAlert
		expect(ncmResult!.slotState.activeAlert?.id).toBe("approval-req-B");
		expect(ncmResult!.slotState.fallbackToClock).toBe(false);
	});

	it("renders cleanly with default arguments without triggering Maximum update depth exceeded", async () => {
		const { root } = setupMockEnv();
		let resultZeroArgs: ReturnType<typeof useNCM> | null = null;
		let renderCountZeroArgs = 0;
		let resultEmptyOptions: ReturnType<typeof useNCM> | null = null;
		let renderCountEmptyOptions = 0;

		function ZeroArgsHarness() {
			renderCountZeroArgs++;
			resultZeroArgs = useNCM();
			return null;
		}

		function EmptyOptionsHarness() {
			renderCountEmptyOptions++;
			resultEmptyOptions = useNCM({});
			return null;
		}

		// Calling useNCM() with zero arguments should render cleanly without infinite loop
		await act(async () => {
			root.render(React.createElement(ZeroArgsHarness));
		});

		expect(resultZeroArgs).not.toBeNull();
		expect(resultZeroArgs!.presentations).toEqual([]);
		expect(resultZeroArgs!.slotState.fallbackToClock).toBe(true);
		expect(resultZeroArgs!.slotState.mascotExpression).toBe("netral");
		expect(renderCountZeroArgs).toBe(1);

		// Calling useNCM({}) with empty options should render cleanly without infinite loop
		await act(async () => {
			root.render(React.createElement(EmptyOptionsHarness));
		});

		expect(resultEmptyOptions).not.toBeNull();
		expect(resultEmptyOptions!.presentations).toEqual([]);
		expect(resultEmptyOptions!.slotState.fallbackToClock).toBe(true);
		expect(resultEmptyOptions!.slotState.mascotExpression).toBe("netral");
		expect(renderCountEmptyOptions).toBe(1);
	});
});

describe("NCM Dynamic Slot Arbitration & Constraints", () => {
	it("arbitrates left slot: eyes (meaningful) > status-icon > clock", () => {
		// 1. Resting: clock
		const resting = arbitrateSlotState([], { isOnline: true, hasStatusIcon: false });
		expect(resting.leftSlot).toBe("clock");
		expect(resting.mascotExpression).toBe("netral");

		// 2. Status icon active when expression is netral
		const statusIconState = arbitrateSlotState([], { isOnline: true, hasStatusIcon: true });
		expect(statusIconState.leftSlot).toBe("status-icon");

		// 3. Meaningful expression overrides status icon: eyes
		const meaningfulExprState = arbitrateSlotState(
			[
				{
					id: "focus-session",
					module: "routine",
					kind: "bubble",
					priority: PriorityLevel.P1_TIME_SENSITIVE,
					expression: "fokus",
					createdAt: 1000
				}
			],
			{ isOnline: true, hasStatusIcon: true }
		);
		expect(meaningfulExprState.leftSlot).toBe("eyes");
		expect(meaningfulExprState.mascotExpression).toBe("fokus");

		// 4. Offline overrides everything to eyes
		const offlineState = arbitrateSlotState([], { isOnline: false, hasStatusIcon: true });
		expect(offlineState.leftSlot).toBe("eyes");
		expect(offlineState.mascotExpression).toBe("offline");

		// 5. Ambient peek requests eyes
		const ambientState = arbitrateSlotState([
			{
				id: "ambient-peek",
				module: "idle-director",
				kind: "ambient",
				priority: PriorityLevel.P5_AMBIENT,
				expression: "penasaran",
				createdAt: 1000
			}
		]);
		expect(ambientState.leftSlot).toBe("eyes");
		expect(ambientState.mascotExpression).toBe("penasaran");
	});

	it("clamps center slot text strictly to <= 36 characters", () => {
		const longText = "Ini adalah deskripsi yang sangat panjang sekali melebihi tiga puluh enam karakter maksimum";
		const state = arbitrateSlotState([
			{
				id: "long-alert",
				module: "test",
				kind: "alert",
				priority: PriorityLevel.P0_CRITICAL,
				payload: { description: longText },
				createdAt: 1000
			}
		]);

		expect(state.centerSlot).not.toBeNull();
		expect(state.centerSlot!.length).toBeLessThanOrEqual(36);
		expect(state.centerSlot).toBe(longText.slice(0, 36));
	});

	it("populates right slot with primary indicator and task counts", () => {
		const progressState = arbitrateSlotState([
			{
				id: "agent-task",
				module: "hermes",
				kind: "bubble",
				priority: PriorityLevel.P3_PROGRESS,
				payload: { progress: 75, count: 3 },
				createdAt: 1000
			}
		]);

		expect(progressState.rightSlot).not.toBeNull();
		expect(progressState.rightSlot?.progress).toBe(75);
		expect(progressState.rightSlot?.count).toBe(3);
	});
});

describe("NCM Conflict Scenarios & Decision Table", () => {
	it("resolves Lyrics (P5) + Translation (P4) preemption and resumption", () => {
		const lyrics: Presentation = {
			id: "music-lyrics",
			module: "music",
			kind: "ambient",
			priority: PriorityLevel.P5_AMBIENT,
			payload: { title: "Lirik Lagu Aktif" },
			createdAt: 1000
		};

		const translation: Presentation = {
			id: "translation-hint",
			module: "translate",
			kind: "bubble",
			priority: PriorityLevel.P4_CONTEXTUAL,
			payload: { title: "Terjemahan Teks" },
			createdAt: 1200
		};

		// 1. Only lyrics active: center slot shows lyrics
		const onlyLyrics = arbitrateSlotState([lyrics]);
		expect(onlyLyrics.centerSlot).toBe("Lirik Lagu Aktif");

		// 2. Translation arrives: higher priority (P4 vs P5) preempts lyrics in center slot
		const preempted = arbitrateSlotState([lyrics, translation]);
		expect(preempted.centerSlot).toBe("Terjemahan Teks");

		// 3. Translation dismisses / expires: lyrics resumes
		const resumed = arbitrateSlotState([lyrics]);
		expect(resumed.centerSlot).toBe("Lirik Lagu Aktif");
	});

	it("coalesces presentations with matching coalesceKey into a single item with updated count", async () => {
		const { root } = setupMockEnv();
		let ncmResult: ReturnType<typeof useNCM> | null = null;

		function CoalesceHarness() {
			ncmResult = useNCM();
			return null;
		}

		await act(async () => {
			root.render(React.createElement(CoalesceHarness));
		});

		expect(ncmResult).not.toBeNull();

		// Submit first duplicate download notification
		await act(async () => {
			ncmResult!.submitPresentation({
				id: "dl-1",
				module: "storage",
				kind: "bubble",
				priority: PriorityLevel.P2_ACTION,
				coalesceKey: "dup-download",
				payload: { count: 1, description: "1 file ganda" }
			});
		});

		expect(ncmResult!.presentations.length).toBe(1);
		expect(ncmResult!.presentations[0].payload.count).toBe(1);

		// Submit second duplicate download notification with identical coalesceKey
		await act(async () => {
			ncmResult!.submitPresentation({
				id: "dl-2",
				module: "storage",
				kind: "bubble",
				priority: PriorityLevel.P2_ACTION,
				coalesceKey: "dup-download",
				payload: { count: 1, description: "2 file ganda" }
			});
		});

		// Coalesced into a single presentation with aggregated count = 2
		expect(ncmResult!.presentations.length).toBe(1);
		expect(ncmResult!.presentations[0].payload.count).toBe(2);
		expect(ncmResult!.presentations[0].payload.description).toBe("2 file ganda");
	});

	it("enforces mode suppression: fullscreen permits only P0, focus/dnd permits only P0 and P1", () => {
		const items: Presentation[] = [
			{
				id: "p0-item",
				module: "security",
				kind: "alert",
				priority: PriorityLevel.P0_CRITICAL,
				createdAt: 1000
			},
			{
				id: "p1-item",
				module: "routine",
				kind: "bubble",
				priority: PriorityLevel.P1_TIME_SENSITIVE,
				createdAt: 1100
			},
			{
				id: "p2-item",
				module: "storage",
				kind: "bubble",
				priority: PriorityLevel.P2_ACTION,
				createdAt: 1200
			},
			{
				id: "p4-item",
				module: "translate",
				kind: "bubble",
				priority: PriorityLevel.P4_CONTEXTUAL,
				createdAt: 1300
			}
		];

		// Fullscreen mode: strictly P0 only
		const fsState = arbitrateSlotState(items, { mode: "fullscreen" });
		expect(fsState.activeAlert?.id).toBe("p0-item");
		expect(fsState.bubbles.length).toBe(0);

		// Focus mode: P0 and P1 pass; P2/P4 suppressed
		const focusState = arbitrateSlotState(items, { mode: "focus" });
		expect(focusState.activeAlert?.id).toBe("p0-item");
		expect(focusState.bubbles.length).toBe(1);
		expect(focusState.bubbles[0].id).toBe("p1-item");

		// Normal mode: all admitted
		const normalState = arbitrateSlotState(items, { mode: "normal" });
		expect(normalState.activeAlert?.id).toBe("p0-item");
		expect(normalState.bubbles.length).toBe(2);
	});

	it("resumes preempted presentation and preserves lower tiers while P0 alert resolves", async () => {
		const { root } = setupMockEnv();
		let ncmResult: ReturnType<typeof useNCM> | null = null;
		let updateProps: (p: { isOnline: boolean; approvals: ApprovalItem[]; approvalJustResolved: boolean }) => void = () => {};

		const pendingApproval: ApprovalItem = {
			request_id: "req-crit",
			source: "hermes",
			risk_level: "high",
			action_type: "file_delete",
			description: "Hapus File",
			created_at: Date.now(),
			expires_at: Date.now() + 60000
		};

		function PreemptHarness() {
			const [props, setProps] = useState({
				isOnline: true,
				approvals: [] as ApprovalItem[],
				approvalJustResolved: false
			});
			updateProps = setProps;
			ncmResult = useNCM(props);
			return null;
		}

		await act(async () => {
			root.render(React.createElement(PreemptHarness));
		});

		// 1. Submit lower-tier P4 presentation with 5000ms TTL
		await act(async () => {
			ncmResult!.submitPresentation({
				id: "trans-hint",
				module: "translate",
				kind: "bubble",
				priority: PriorityLevel.P4_CONTEXTUAL,
				payload: { title: "Terjemahan Teks" },
				ttl: 5000
			});
		});

		expect(ncmResult!.presentations.length).toBe(1);
		expect(ncmResult!.slotState.centerSlot).toBe("Terjemahan Teks");

		// 2. Critical P0 approval arrives -> preempts P4 in activeAlert
		await act(async () => {
			updateProps({
				isOnline: true,
				approvals: [pendingApproval],
				approvalJustResolved: false
			});
		});

		expect(ncmResult!.slotState.activeAlert?.id).toBe("approval-req-crit");

		// 3. Resolve approval -> P4 resumes in presentation list and center slot
		await act(async () => {
			updateProps({
				isOnline: true,
				approvals: [],
				approvalJustResolved: true
			});
		});

		expect(ncmResult!.slotState.activeAlert).toBeNull();
		const p4Item = ncmResult!.presentations.find((p) => p.id === "trans-hint");
		expect(p4Item).toBeDefined();
		expect(ncmResult!.slotState.centerSlot).toBe("Terjemahan Teks");
	});

	it("is fully idempotent when receiving fresh unmemoized approvals array references", async () => {
		const { root } = setupMockEnv();
		let renderCount = 0;
		let ncmResult: ReturnType<typeof useNCM> | null = null;
		let triggerParentRender: () => void = () => {};

		const approvalItem: ApprovalItem = {
			request_id: "req-unmemoized-1",
			action_type: "file_read",
			path: "/test/file.txt",
			description: "Baca File",
			status: "pending",
			created_at: 1000,
			expires_at: Date.now() + 60000
		};

		function UnmemoizedHarness() {
			const [, setTick] = useState(0);
			triggerParentRender = () => setTick((t) => t + 1);
			renderCount++;

			// Inline fresh array reference on every render
			ncmResult = useNCM({
				isOnline: true,
				approvals: [{ ...approvalItem }]
			});
			return null;
		}

		await act(async () => {
			root.render(React.createElement(UnmemoizedHarness));
		});

		// Initial mount + state sync should stabilize in exactly 2 renders (mount + presentation state sync)
		expect(renderCount).toBe(2);
		expect(ncmResult!.slotState.activeAlert?.id).toBe("approval-req-unmemoized-1");

		// Trigger 5 consecutive external parent re-renders with fresh approval array references
		for (let i = 0; i < 5; i++) {
			await act(async () => {
				triggerParentRender();
			});
		}

		// Renders should increase exactly 1:1 with parent renders (no cascading infinite loops)
		expect(renderCount).toBe(7); // 2 initial + 5 manual
		expect(ncmResult!.slotState.activeAlert?.id).toBe("approval-req-unmemoized-1");
		expect(ncmResult!.presentations.length).toBe(1);
	});
});

describe("Idle Director & Ambient Selection Engine", () => {
	it("selects appropriate ambient expressions based on time and context", () => {
		// Late night (02:00) selects drowsy expressions
		const lateNightExpr = pickAmbientExpression({ hour: 2 });
		expect(["ngantuk", "menguap", "tidur"]).toContain(lateNightExpr);

		// Task completion context selects positive expressions
		const taskDoneExpr = pickAmbientExpression({ taskJustFinished: true });
		expect(["senang", "bangga"]).toContain(taskDoneExpr);

		// Low battery context selects khawatir
		const batteryExpr = pickAmbientExpression({ isBatteryLow: true });
		expect(batteryExpr).toBe("khawatir");

		// Consecutive repetition avoidance
		const nextExpr = pickAmbientExpression({ hour: 14, lastExpression: "netral" });
		expect(["penasaran", "kedip-sebelah"]).toContain(nextExpr);
	});

	it("suppresses ambient peeks when in fullscreen, focus, or Mati frequency", async () => {
		const { root } = setupMockEnv();
		let ncmResult: ReturnType<typeof useNCM> | null = null;

		function IdleHarness() {
			ncmResult = useNCM({ isFullscreen: true, freq: "Mati" });
			return null;
		}

		await act(async () => {
			root.render(React.createElement(IdleHarness));
		});

		expect(ncmResult).not.toBeNull();

		// Attempt ambient peek during fullscreen / Mati
		await act(async () => {
			ncmResult!.triggerAmbientPeek("penasaran");
		});

		// Must remain 0 presentations
		expect(ncmResult!.presentations.length).toBe(0);
	});
});

describe("Mascot 22 Expressions Catalog Validation", () => {
	it("arbitrates and assigns all 22 expressions without errors", () => {
		const all22Expressions = [
			"netral",
			"senang",
			"semangat",
			"bangga",
			"fokus",
			"berpikir",
			"dengar",
			"penasaran",
			"kaget",
			"khawatir",
			"bosan",
			"ngantuk",
			"menguap",
			"tidur",
			"kesal",
			"marah",
			"sedih",
			"curiga",
			"nakal",
			"kedip-sebelah",
			"pusing",
			"offline"
		] as const;

		for (const expr of all22Expressions) {
			const state = arbitrateSlotState([
				{
					id: `expr-${expr}`,
					module: "mascot",
					kind: "ambient",
					priority: PriorityLevel.P5_AMBIENT,
					expression: expr,
					createdAt: 1000
				}
			]);
			expect(state.mascotExpression).toBe(expr);
			expect(state.leftSlot).toBe("eyes");
		}
	});

	it("preserves real system notification expression & text even when Core is offline", () => {
		const state = arbitrateSlotState(
			[
				{
					id: "sys-notif-1",
					module: "system-notification",
					kind: "alert",
					priority: PriorityLevel.P1_TIME_SENSITIVE,
					expression: "senang",
					payload: { title: "WhatsApp: Halo!", description: "WhatsApp: Halo!" },
					createdAt: 1000
				}
			],
			{ isOnline: false }
		);
		expect(state.mascotExpression).toBe("senang");
		expect(state.centerSlot).toBe("WhatsApp: Halo!");
		expect(state.leftSlot).toBe("eyes");
	});
});


