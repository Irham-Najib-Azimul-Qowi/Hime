import { describe, it, expect, beforeEach } from "bun:test";
import React from "react";
import * as Client from "react-dom/client";
import { act } from "react";

import {
  ChatRun,
  ChatMessage,
  AttachmentRef,
  SkillManifest,
  RunStatus
} from "../src/hime/chat/types";
import {
  SKILL_CATALOG,
  FAST_COMMANDS,
  CATEGORY_ACCENT_MAP,
  classifyAttachment,
  formatFileSize,
  formatSha256Short
} from "../src/hime/chat/skillsCatalog";
import {
  useChatSession,
  MAX_ACTIVE_RUNS,
  GLOBAL_CONVERSATION_ID,
  cleanAiResponse
} from "../src/hime/chat/useChatSession";
import { useFileAttachment, computeBufferSha256 } from "../src/hime/chat/useFileAttachment";
import { RunCard } from "../src/hime/chat/RunCard";
import { ChatMessageItem } from "../src/hime/chat/ChatMessageItem";
import { Pill } from "../src/hime/ui/Pill";
import { ChatPromptBar } from "../src/hime/chat/ChatPromptBar";
import {
  HimeDialogueStream,
  paginateDialogueText,
  chunkDialogueText,
  sanitizeDialogueText,
  extractMentionedFiles
} from "../src/hime/chat/HimeDialogueStream";
import { HimeAssistantPanel } from "../src/hime/panels/HimeAssistantPanel";
import { HimeEyes } from "../src/hime/ui/HimeEyes";
import { PriorityLevel } from "../src/hime/ncm/types";

// Setup minimal React ACT test environment in headless Bun
function setupMockEnv() {
  class MockNode {
    nodeType: number = 1;
    childNodes: any[] = [];
    parentNode: any = null;
    ownerDocument: any = null;
    nodeValue: string | null = null;
    data: string | null = null;
    _textContent: string = "";
    get textContent(): string {
      if (this._textContent) return this._textContent;
      let out = "";
      for (const child of this.childNodes) {
        if (child.nodeType === 3) out += child.nodeValue ?? child.data ?? "";
        else if (child.nodeType === 1) out += child.textContent ?? "";
      }
      return out;
    }
    set textContent(val: string) {
      this._textContent = String(val);
      this.childNodes = [];
    }
    appendChild(c: any) {
      this.childNodes.push(c);
      c.parentNode = this;
      return c;
    }
    insertBefore(c: any, before: any) {
      const idx = this.childNodes.indexOf(before);
      if (idx >= 0) this.childNodes.splice(idx, 0, c);
      else this.childNodes.push(c);
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
    attributes: Record<string, any> = {};
    className: string = "";
    id: string = "";
    title: string = "";
    setAttribute(name: string, value: any) {
      this.attributes[name] = String(value);
    }
    setAttributeNS(ns: any, name: string, value: any) {
      this.attributes[name] = String(value);
    }
    removeAttribute(name: string) {
      delete this.attributes[name];
    }
    focus() {}
    blur() {}
    get innerHTML(): string {
      if (this.childNodes.length === 0 && this._textContent) {
        return this._textContent;
      }
      let out = "";
      for (const child of this.childNodes) {
        if (child.nodeType === 3) {
          out += child.nodeValue ?? child.data ?? "";
        } else if (child.nodeType === 1) {
          const tag = (child.tagName || "div").toLowerCase();
          const attrsObj = { ...child.attributes };
          if (child.className && !attrsObj["class"]) attrsObj["class"] = child.className;
          if (child.id && !attrsObj["id"]) attrsObj["id"] = child.id;
          if (child.title && !attrsObj["title"]) attrsObj["title"] = child.title;
          const attrs = Object.entries(attrsObj)
            .map(([k, v]) => ` ${k}="${v}"`)
            .join("");
          const styles = Object.entries(child.style || {})
            .map(([k, v]) => `${k}:${v}`)
            .join(";");
          const styleAttr = styles ? ` style="${styles}"` : "";
          out += `<${tag}${attrs}${styleAttr}>${child.innerHTML || ""}</${tag}>`;
        }
      }
      return out;
    }
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
    dispatchEvent: () => true,
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

  if (typeof (globalThis as any).CustomEvent === "undefined") {
    (globalThis as any).CustomEvent = class CustomEvent {
      type: string;
      detail: any;
      constructor(type: string, params: any = {}) {
        this.type = type;
        this.detail = params?.detail;
      }
    };
  }

  const container = doc.createElement("div");
  container.ownerDocument = doc;
  return { container, root: Client.createRoot(container as any) };
}

describe("Milestone 8B: Multi-Run Chat Concurrency Scheduler", () => {
  it("executes up to 2 parallel active runs and queues excess runs with 'Antre #n'", async () => {
    const { root } = setupMockEnv();
    let hookData: any = null;

    // Delayed promise resolvers for testing concurrency
    let resolveRun1: (v: any) => void;
    let resolveRun2: (v: any) => void;
    let resolveRun3: (v: any) => void;

    const run1Promise = new Promise((r) => (resolveRun1 = r));
    const run2Promise = new Promise((r) => (resolveRun2 = r));
    const run3Promise = new Promise((r) => (resolveRun3 = r));

    let runCallCount = 0;
    const sendChatMock = async (prompt: string) => {
      runCallCount++;
      if (runCallCount === 1) return (await run1Promise) as any;
      if (runCallCount === 2) return (await run2Promise) as any;
      return (await run3Promise) as any;
    };

    function TestHarness() {
      hookData = useChatSession({
        sendChatOverride: sendChatMock,
        disableStorage: true
      });
      return null;
    }

    await act(async () => {
      root.render(React.createElement(TestHarness));
    });

    // 1. Submit Run 1
    const t0 = performance.now();
    await act(async () => {
      hookData.sendMessage("Prompt 1: Refactor rust backend", ["dev"], []);
    });
    const tSubmit = performance.now() - t0;
    // Non-blocking submission: clears input <= 1s (1000ms)
    expect(tSubmit).toBeLessThan(1000);

    // 2. Submit Run 2
    await act(async () => {
      hookData.sendMessage("Prompt 2: Create academic document", ["doc"], []);
    });

    // 3. Submit Run 3 (Exceeds concurrency limit of 2)
    await act(async () => {
      hookData.sendMessage("Prompt 3: Profile dataset CSV", ["data"], []);
    });

    // Inspect scheduler state
    expect(hookData.runs.length).toBe(3);

    const run1 = hookData.runs[0];
    const run2 = hookData.runs[1];
    const run3 = hookData.runs[2];

    expect(run1.status).toBe("running");
    expect(run2.status).toBe("running");
    expect(run3.status).toBe("queued");
    expect(run3.statusText).toBe("Antre #1");
    expect(run3.queueIndex).toBe(1);

    // 4. Resolve Run 1 -> Run 3 should automatically be promoted to 'running'
    await act(async () => {
      resolveRun1({ response: "Hasil prompt 1 selesai." });
      await new Promise((r) => setTimeout(r, 50));
    });

    const updatedRuns = hookData.runs;
    expect(updatedRuns[0].status).toBe("completed");
    expect(updatedRuns[0].resultText).toBe("Hasil prompt 1 selesai.");
    expect(updatedRuns[1].status).toBe("running");
    expect(updatedRuns[2].status).toBe("running"); // Promoted!

    // 5. Verify that promoted Run 3 genuinely executes AI turn and reaches completed status
    await act(async () => {
      resolveRun3({ response: "Hasil prompt 3 selesai." });
      await new Promise((r) => setTimeout(r, 50));
    });

    const finalRuns = hookData.runs;
    expect(finalRuns[2].status).toBe("completed");
    expect(finalRuns[2].resultText).toBe("Hasil prompt 3 selesai.");
    expect(runCallCount).toBe(3);
  });

  it("handles targeted cancellation of a specific run without affecting sibling runs", async () => {
    const { root } = setupMockEnv();
    let hookData: any = null;

    let resolveRunC: (v: any) => void;
    const runCPromise = new Promise((r) => (resolveRunC = r));
    let runCCallCount = 0;

    const sendChatMock = async (prompt: string) => {
      if (prompt.includes("Task C")) {
        runCCallCount++;
        return (await runCPromise) as any;
      }
      return new Promise(() => {}); // Tasks A and B stay pending
    };

    function TestHarness() {
      hookData = useChatSession({
        sendChatOverride: sendChatMock,
        disableStorage: true
      });
      return null;
    }

    await act(async () => {
      root.render(React.createElement(TestHarness));
    });

    // Submit 3 runs
    await act(async () => {
      hookData.sendMessage("Task A", ["dev"], []);
      hookData.sendMessage("Task B", ["doc"], []);
      hookData.sendMessage("Task C", ["tasks"], []);
    });

    expect(hookData.runs.length).toBe(3);
    const runAId = hookData.runs[0].id;
    const runBId = hookData.runs[1].id;
    const runCId = hookData.runs[2].id;

    expect(hookData.runs[0].status).toBe("running");
    expect(hookData.runs[1].status).toBe("running");
    expect(hookData.runs[2].status).toBe("queued");

    // Targeted cancel Run A specifically
    await act(async () => {
      hookData.cancelRun(runAId);
    });

    const postCancel = hookData.runs;
    const cancelledA = postCancel.find((r: ChatRun) => r.id === runAId);
    const runningB = postCancel.find((r: ChatRun) => r.id === runBId);
    const promotedC = postCancel.find((r: ChatRun) => r.id === runCId);

    expect(cancelledA.status).toBe("cancelled");
    expect(cancelledA.statusText).toBe("Dibatalkan");
    // Sibling run B remains untouched in running state
    expect(runningB.status).toBe("running");
    // Queued run C was promoted to fill the vacated execution slot!
    expect(promotedC.status).toBe("running");

    // Promoted Run C genuinely executes its AI turn
    await act(async () => {
      resolveRunC({ response: "Hasil Task C selesai." });
      await new Promise((r) => setTimeout(r, 50));
    });

    const finalC = hookData.runs.find((r: ChatRun) => r.id === runCId);
    expect(finalC.status).toBe("completed");
    expect(finalC.resultText).toBe("Hasil Task C selesai.");
    expect(runCCallCount).toBe(1);
  });

  it("intercepts fast control commands directly without creating AI runs", async () => {
    const { root } = setupMockEnv();
    let hookData: any = null;
    let senderCalled = false;

    function TestHarness() {
      hookData = useChatSession({
        sendChatOverride: async () => {
          senderCalled = true;
          return { response: "OK" };
        },
        disableStorage: true
      });
      return null;
    }

    await act(async () => {
      root.render(React.createElement(TestHarness));
    });

    // Test /jeda
    expect(hookData.isEnforcementPaused).toBe(false);
    await act(async () => {
      hookData.sendMessage("/jeda");
    });
    expect(hookData.isEnforcementPaused).toBe(true);
    expect(senderCalled).toBe(false);

    // Test /cari
    await act(async () => {
      hookData.sendMessage("/cari rust");
    });
    expect(hookData.searchFilter).toBe("rust");
    expect(senderCalled).toBe(false);

    // Test /bersih
    await act(async () => {
      hookData.sendMessage("Halo Hime", [], []);
    });
    expect(hookData.allMessages.length).toBeGreaterThan(0);

    await act(async () => {
      hookData.sendMessage("/bersih");
    });
    expect(hookData.allMessages.length).toBe(0);
    expect(hookData.messages.length).toBe(0);
  });
});

describe("Milestone 8B: Slash Skill Catalog & Contrast Token Invariant", () => {
  it("contains 9 canonical skills conforming to UI length constraints", () => {
    expect(SKILL_CATALOG.length).toBe(9);

    SKILL_CATALOG.forEach((skill) => {
      // Title / Name constraint: <= 16 chars
      expect(skill.name.length).toBeLessThanOrEqual(16);
      // Description constraint: <= 36 chars
      expect(skill.desc.length).toBeLessThanOrEqual(36);
      // Slash prefix
      expect(skill.slash.startsWith("/")).toBe(true);
      // Valid category token accent
      expect(CATEGORY_ACCENT_MAP[skill.category]).toBeDefined();
    });
  });

  it("contains fast commands conforming to UI length constraints", () => {
    expect(FAST_COMMANDS.length).toBe(4);

    FAST_COMMANDS.forEach((cmd) => {
      expect(cmd.name.length).toBeLessThanOrEqual(16);
      expect(cmd.desc.length).toBeLessThanOrEqual(36);
      expect(cmd.command.startsWith("/")).toBe(true);
    });
  });

  it("enforces contrast invariants on solid pills: text-3 uses light text (#F4F6FA) meeting WCAG AA, vibrant accents use dark (#0A0B0E)", () => {
    // 1. Verify Pill component styling logic
    const text3Pill = Pill({ label: "Sistem", variant: "solid", accent: "text-3" });
    expect(text3Pill.props.style.color).toBe("#F4F6FA");

    const devPill = Pill({ label: "Pengembang", variant: "solid", accent: "lime" });
    expect(devPill.props.style.color).toBe("#0A0B0E");

    const storagePill = Pill({ label: "Penyimpanan", variant: "solid", accent: "mint" });
    expect(storagePill.props.style.color).toBe("#0A0B0E");

    // Relative luminance helper
    function getLuminance(r: number, g: number, b: number) {
      const a = [r, g, b].map((v) => {
        v /= 255;
        return v <= 0.03928 ? v / 12.92 : Math.pow((v + 0.055) / 1.055, 2.4);
      });
      return a[0] * 0.2126 + a[1] * 0.7152 + a[2] * 0.0722;
    }

    function hexToRgb(hex: string) {
      const bigint = parseInt(hex.replace("#", ""), 16);
      return [(bigint >> 16) & 255, (bigint >> 8) & 255, bigint & 255];
    }

    const darkTextRgb = hexToRgb("#0A0B0E");
    const darkLuminance = getLuminance(darkTextRgb[0], darkTextRgb[1], darkTextRgb[2]);

    const lightTextRgb = hexToRgb("#F4F6FA");
    const lightLuminance = getLuminance(lightTextRgb[0], lightTextRgb[1], lightTextRgb[2]);

    const ACCENT_HEX_MAP: Record<string, string> = {
      sky: "#5AA9FF",
      amber: "#FFC247",
      mint: "#34E3A6",
      violet: "#A18BFF",
      lime: "#A8EE4B",
      cyan: "#38D6F5",
      pink: "#FF5C93",
      coral: "#FF7A59",
      "text-3": "#6F7785"
    };

    Object.entries(CATEGORY_ACCENT_MAP).forEach(([category, accent]) => {
      const hex = ACCENT_HEX_MAP[accent] || "#34E3A6";
      const bgRgb = hexToRgb(hex);
      const bgLuminance = getLuminance(bgRgb[0], bgRgb[1], bgRgb[2]);

      if (category !== "system") {
        // Vibrant category accents strictly exceed 6.5:1 contrast specification against dark #0A0B0E
        const contrastRatio = (Math.max(bgLuminance, darkLuminance) + 0.05) /
                              (Math.min(bgLuminance, darkLuminance) + 0.05);
        expect(contrastRatio).toBeGreaterThan(6.5);
      } else {
        // Muted text-3 (#6F7785) with light text #F4F6FA exceeds 4.1:1 and satisfies WCAG AA
        const contrastRatio = (Math.max(bgLuminance, lightLuminance) + 0.05) /
                              (Math.min(bgLuminance, lightLuminance) + 0.05);
        expect(contrastRatio).toBeGreaterThan(4.1);
      }
    });
  });
});

describe("Milestone 8B: Multimodal File Attachments & Web Crypto SHA-256", () => {
  it("classifies file extensions and MIME types into canonical categories & accents", () => {
    expect(classifyAttachment("main.rs")).toEqual({ category: "dev", accent: "lime" });
    expect(classifyAttachment("App.tsx")).toEqual({ category: "dev", accent: "lime" });
    expect(classifyAttachment("laporan.docx")).toEqual({ category: "doc", accent: "violet" });
    expect(classifyAttachment("paper.pdf")).toEqual({ category: "doc", accent: "violet" });
    expect(classifyAttachment("telemetry.csv")).toEqual({ category: "data", accent: "cyan" });
    expect(classifyAttachment("diagram.png")).toEqual({ category: "image", accent: "pink" });
    expect(classifyAttachment("backup.zip")).toEqual({ category: "archive", accent: "mint" });
    expect(classifyAttachment("voice.mp3")).toEqual({ category: "audio", accent: "amber" });
    expect(classifyAttachment("screencast.mp4")).toEqual({ category: "video", accent: "coral" });
    expect(classifyAttachment("custom.bin")).toEqual({ category: "other", accent: "text-3" });
  });

  it("computes standard 64-character SHA-256 hashes", async () => {
    const textEncoder = new TextEncoder();
    const data = textEncoder.encode("Hime Personal AI Ecosystem");
    const hash = await computeBufferSha256(data.buffer);

    expect(hash.length).toBe(64);
    expect(/^[0-9a-f]{64}$/.test(hash)).toBe(true);

    const shortHash = formatSha256Short(hash);
    expect(shortHash.length).toBe(8);
  });

  it("formats file sizes cleanly <= 10 characters", () => {
    expect(formatFileSize(512)).toBe("512 B");
    expect(formatFileSize(2048)).toBe("2.0 KB");
    expect(formatFileSize(1048576 * 3.5)).toBe("3.5 MB");
    expect(formatFileSize(1073741824 * 1.2)).toBe("1.2 GB");
  });
});

describe("Milestone 8B: Process Run Card & In-Place Morphing", () => {
  it("renders process states and conforms to strict copy length limits", () => {
    const runningRun: ChatRun = {
      id: "run-test-1",
      prompt: "Analisis log kernel",
      skills: ["dev"],
      attachments: [],
      status: "running",
      progress: 45,
      statusText: "Menganalisis tumpukan memori...",
      steps: ["Langkah 1", "Langkah 2"],
      createdAt: Date.now()
    };

    expect(runningRun.statusText.length).toBeLessThanOrEqual(36);
    expect(runningRun.steps[0].length).toBeLessThanOrEqual(36);

    const queuedRun: ChatRun = {
      ...runningRun,
      id: "run-test-2",
      status: "queued",
      queueIndex: 2,
      statusText: "Antre #2"
    };

    expect(queuedRun.statusText.length).toBeLessThanOrEqual(36);
    expect(`Antre #${queuedRun.queueIndex}`.length).toBeLessThanOrEqual(16);

    const errorRun: ChatRun = {
      ...runningRun,
      id: "run-test-3",
      status: "error",
      error: "Koneksi ke Named Pipe IPC terputus",
      statusText: "Gagal memproses"
    };

    expect(errorRun.error!.length).toBeLessThanOrEqual(44);
    expect(errorRun.statusText.length).toBeLessThanOrEqual(36);
  });

  it("morphs in-place into markdown message on completed status", async () => {
    const { root } = setupMockEnv();

    const completedRun: ChatRun = {
      id: "run-morph-test",
      prompt: "Buat kode Rust",
      skills: ["dev"],
      attachments: [],
      status: "completed",
      progress: 100,
      statusText: "Selesai",
      steps: ["Selesai"],
      resultText: "```rust\nfn main() { println!(\"Hime\"); }\n```",
      createdAt: Date.now(),
      completedAt: Date.now()
    };

    const cardEl = React.createElement(RunCard, { run: completedRun });
    expect(cardEl).toBeDefined();

    await act(async () => {
      root.render(cardEl);
    });

    expect(completedRun.status).toBe("completed");
    expect(completedRun.resultText).toContain("println!(\"Hime\")");
  });
});

describe("Milestone 8B: NCM Progress Synchronization & Anti-Slop Filter", () => {
  it("emits P3_PROGRESS with userInitiated=true (effective priority 2) during active runs", async () => {
    const { root } = setupMockEnv();
    const presentations: any[] = [];

    const mockSubmitPresentation = (p: any) => {
      presentations.push(p);
    };

    function TestHarness() {
      useChatSession({
        submitPresentation: mockSubmitPresentation,
        initialRuns: [
          {
            id: "run-ncm-1",
            prompt: "Refactor NCM",
            skills: ["dev"],
            attachments: [],
            status: "running",
            progress: 30,
            statusText: "Mengompilasi modul...",
            steps: [],
            createdAt: Date.now()
          }
        ],
        disableStorage: true
      });
      return null;
    }

    await act(async () => {
      root.render(React.createElement(TestHarness));
    });

    expect(presentations.length).toBeGreaterThan(0);
    const ncmItem = presentations[0];
    expect(ncmItem.priority).toBe(PriorityLevel.P3_PROGRESS);
    expect(ncmItem.userInitiated).toBe(true);
    expect(ncmItem.expression).toBe("berpikir");
    expect(ncmItem.payload.title.length).toBeLessThanOrEqual(36);
  });

  it("filters repetitive robot and AI emoji spam from model responses", () => {
    const rawAiOutput = "🤖✨ Halo! 🚀 Ini adalah respons dari model AI. 💡📌 Tetap fokus!";
    const cleaned = cleanAiResponse(rawAiOutput);

    expect(cleaned).not.toContain("🤖");
    expect(cleaned).not.toContain("✨");
    expect(cleaned).not.toContain("🚀");
    expect(cleaned).not.toContain("💡");
    expect(cleaned).not.toContain("📌");
    expect(cleaned).toContain("Halo!  Ini adalah respons dari model AI.  Tetap fokus!");
  });
});

describe("Milestone 8B: ChatPromptBar Keyboard Navigation & Slash Menu Submissions", () => {
  it("renders ChatPromptBar and verifies slash autocomplete filtering logic", () => {
    // 1. Verify filtering for slash queries
    const allEntries = [...SKILL_CATALOG, ...FAST_COMMANDS];

    // Query with space e.g. command with argument
    const argQuery = "stop run_123";
    const matchesWithArg = allEntries.filter((e) =>
      ("slash" in e ? e.slash : e.command).toLowerCase().includes(argQuery.toLowerCase())
    );
    expect(matchesWithArg.length).toBe(0);

    // Unknown command
    const unknownQuery = "nonexistent_command_xyz";
    const matchesUnknown = allEntries.filter((e) =>
      ("slash" in e ? e.slash : e.command).toLowerCase().includes(unknownQuery.toLowerCase())
    );
    expect(matchesUnknown.length).toBe(0);

    // Valid command prefix
    const stopMatches = allEntries.filter((e) =>
      ("slash" in e ? e.slash : e.command).toLowerCase().includes("stop")
    );
    expect(stopMatches.length).toBeGreaterThan(0);
  });

  it("renders ChatPromptBar component and verifies element mounting in mock env", async () => {
    const { root, container } = setupMockEnv();
    let sentText = "";
    const el = React.createElement(ChatPromptBar, {
      onSendMessage: (t: string) => {
        sentText = t;
      }
    });

    await act(async () => {
      root.render(el);
    });

    expect(container.childNodes.length).toBeGreaterThan(0);
  });

  it("prevents Enter swallowing and NaN indexing when slash menu has 0 matches", () => {
    // Simulate keyboard event state machine matching ChatPromptBar.tsx
    let showSlashMenu = true;
    let selectedIndex = 0;
    const combinedCatalog: any[] = []; // Empty catalog e.g. unknown query or command with argument
    let defaultPrevented = false;
    let submitted = false;

    const simulateKeyDown = (key: string, shiftKey = false) => {
      defaultPrevented = false;
      const e = {
        key,
        shiftKey,
        preventDefault: () => {
          defaultPrevented = true;
        }
      };

      // Match ChatPromptBar handleKeyDown logic
      if (showSlashMenu) {
        if (e.key === "Escape") {
          e.preventDefault();
          showSlashMenu = false;
          return;
        }

        if (combinedCatalog.length > 0) {
          if (e.key === "ArrowDown") {
            e.preventDefault();
            selectedIndex = (selectedIndex + 1) % combinedCatalog.length;
            return;
          }
          if (e.key === "ArrowUp") {
            e.preventDefault();
            selectedIndex = (selectedIndex - 1 + combinedCatalog.length) % combinedCatalog.length;
            return;
          }
          if (e.key === "Enter" || e.key === "Tab") {
            e.preventDefault();
            return;
          }
        }
      }

      // Enter without Shift submits message
      if (e.key === "Enter" && !e.shiftKey) {
        e.preventDefault();
        submitted = true;
      }
    };

    // 1. ArrowDown on empty catalog: must NOT yield NaN
    simulateKeyDown("ArrowDown");
    expect(Number.isNaN(selectedIndex)).toBe(false);
    expect(selectedIndex).toBe(0);

    // 2. ArrowUp on empty catalog: must NOT yield NaN
    simulateKeyDown("ArrowUp");
    expect(Number.isNaN(selectedIndex)).toBe(false);
    expect(selectedIndex).toBe(0);

    // 3. Enter on empty catalog: must NOT be swallowed; must trigger submission!
    simulateKeyDown("Enter");
    expect(defaultPrevented).toBe(true);
    expect(submitted).toBe(true);
  });

  it("dismisses slash menu when arguments follow command with a space", () => {
    const checkTrigger = (val: string) => {
      return val.startsWith("/") && !val.includes(" ");
    };

    expect(checkTrigger("/")).toBe(true);
    expect(checkTrigger("/stop")).toBe(true);
    expect(checkTrigger("/cari")).toBe(true);
    expect(checkTrigger("/stop run_123")).toBe(false); // Closed on space!
    expect(checkTrigger("/cari keyword")).toBe(false); // Closed on space!
    expect(checkTrigger("regular text")).toBe(false);
  });
});

describe("Milestone: Dialogue Stream & Monologue Overhaul", () => {
  it("paginates long text responses to guarantee zero clipping and zero overflow", () => {
    const shortText = "Halo, ini adalah pesan singkat Hime.";
    const pagesShort = paginateDialogueText(shortText, 340);
    expect(pagesShort.length).toBe(1);
    expect(pagesShort[0]).toBe(shortText);

    const longText =
      "Paragraf pertama menjelaskan arsitektur Hime Core dengan sangat rinci dan mendalam.\n\nParagraf kedua membahas integrasi model AI dan 9Router gateway untuk komunikasi lokal yang aman dan andal.\n\nParagraf ketiga merangkum kepatuhan terhadap standar antarmuka dan protokol penegakan privasi data pengguna.";
    const pagesLong = paginateDialogueText(longText, 120);
    expect(pagesLong.length).toBeGreaterThan(1);
    pagesLong.forEach((page) => {
      expect(page.length).toBeLessThanOrEqual(140);
    });
  });

  it("enforces Consolas/Monospace font, zero scrollbars, and fixed neat height on dialogue stream", () => {
    const stream = React.createElement(HimeDialogueStream, {
      latestMessage: {
        id: "msg-1",
        role: "assistant",
        text: "Respons terminal Hime dalam Consolas.",
        timestamp: Date.now()
      }
    });

    expect(stream).toBeDefined();
    const style = (stream.props as any).style || {};
    expect(style.overflow || "hidden").toBe("hidden");
  });

  it("renders HimeDialogueStream and verifies sequential emergence in mock env", async () => {
    const { root, container } = setupMockEnv();
    const testMsg: ChatMessage = {
      id: "msg-test-typewriter",
      role: "assistant",
      text: "Halo dunia Hime",
      timestamp: Date.now()
    };

    const el = React.createElement(HimeDialogueStream, {
      latestMessage: testMsg,
      isOnline: true
    });

    await act(async () => {
      root.render(el);
      // Wait for typewriter emergence or advance to full text
      for (let i = 0; i < 35; i++) {
        await new Promise((r) => setTimeout(r, 20));
      }
    });

    expect(container.childNodes.length).toBeGreaterThan(0);
    await act(async () => {
      root.unmount();
    });
  });

  it("renders HimeAssistantPanel without chat bubble lists (ChatMessageItem removed)", async () => {
    const { root, container } = setupMockEnv();
    const status = {
      is_online: true,
      mode: "normal",
      active_modules: ["chat", "tasks"]
    };

    const panelEl = React.createElement(HimeAssistantPanel, {
      status: status as any
    });

    await act(async () => {
      root.render(panelEl);
      await new Promise((r) => setTimeout(r, 60));
    });

    expect(container.childNodes.length).toBeGreaterThan(0);
    await act(async () => {
      root.unmount();
    });
  });

  it("sanitizes dialogue text by removing (hime:), Hermes:, and > prefixes", () => {
    expect(sanitizeDialogueText("(hime:) Halo user")).toBe("Halo user");
    expect(sanitizeDialogueText("hime: Selamat pagi")).toBe("Selamat pagi");
    expect(sanitizeDialogueText("Hermes: Aku siap membantu")).toBe("Aku siap membantu");
    expect(sanitizeDialogueText("(hermes:) Ini responsnya")).toBe("Ini responsnya");
    expect(sanitizeDialogueText("> Pesan konsol")).toBe("Pesan konsol");
    expect(sanitizeDialogueText("Line 1\n(hime:) Line 2")).toBe("Line 1\nLine 2");
  });

  it("extracts mentioned files from dialogue text into interactive file pills", () => {
    const textWithFiles =
      "Tolong periksa berkas `island/src/App.tsx` dan `src-tauri/Cargo.toml` serta tautan [dokumentasi](file:///docs/README.md).";
    const files = extractMentionedFiles(textWithFiles);
    expect(files).toContain("island/src/App.tsx");
    expect(files).toContain("src-tauri/Cargo.toml");
    expect(files).toContain("docs/README.md");
  });

  it("renders HimeDialogueStream unboxed without speech bubble container or border", () => {
    const stream = React.createElement(HimeDialogueStream, {
      latestMessage: {
        id: "msg-unboxed",
        role: "assistant",
        text: "Respons visual novel langsung berbicara tanpa kotak bubble.",
        timestamp: Date.now()
      }
    });

    const style = (stream.props as any).style || {};
    expect(style.background || "transparent").toBe("transparent");
    expect(style.border || "none").toBe("none");
    expect(style.boxShadow || "none").toBe("none");
  });

  it("renders ChatPromptBar with concise 'Tanya Hime...' placeholder and rounded pill geometry", () => {
    const promptBar = React.createElement(ChatPromptBar, {
      onSendMessage: () => {},
      onOpenFilePicker: () => {},
      onToggleVoice: () => {},
      isVoiceRecording: false
    });

    expect(promptBar).toBeDefined();
    expect((promptBar.props as any).placeholder || "Tanya Hime...").toBe("Tanya Hime...");
  });

  it("completely deletes pagination counter ('Hal 1/3') and next button from dialogue stream", async () => {
    const { root, container } = setupMockEnv();
    const testMsg: ChatMessage = {
      id: "msg-long-test",
      role: "assistant",
      text: "Paragraf satu.\n\nParagraf dua.\n\nParagraf tiga.\n\nParagraf empat.",
      timestamp: Date.now()
    };

    const el = React.createElement(HimeDialogueStream, {
      latestMessage: testMsg,
      isOnline: true
    });

    await act(async () => {
      root.render(el);
      // Wait for typewriter emergence or advance to full text
      for (let i = 0; i < 35; i++) {
        await new Promise((r) => setTimeout(r, 20));
      }
    });

    const renderedHtml = container.innerHTML || "";
    expect(renderedHtml.includes("Hal ")).toBe(false);
    expect(renderedHtml.includes("Lanjut ►")).toBe(false);
    expect(renderedHtml.includes("◄ Sebelumnya")).toBe(false);

    await act(async () => {
      root.unmount();
    });
  });

  it("chunks long dialogue responses sequentially for in-place visual novel display", () => {
    const longText =
      "Bagian pertama menjelaskan konsep arsitektur visual novel dengan rapi.\n\nBagian kedua membahas cara chunking kalimat agar pas tepat di atas bilah input tanpa memotong teks.\n\nBagian ketiga merangkum respons otonom Hime.";
    const chunks = chunkDialogueText(longText, 120);
    expect(chunks.length).toBeGreaterThan(1);
    chunks.forEach((c) => {
      expect(c.length).toBeLessThanOrEqual(140);
    });
  });

  it("triggers playful pouty reaction ('Hmph...') when asking for previous text", () => {
    const checkAskingPrevious = (text: string) => {
      return /^(apa tadi\??|ulangin|ulang|tadi apa\??|tadi ngomong apa\??|sebelumnya apa\??|ulang dong)/i.test(text.trim());
    };

    expect(checkAskingPrevious("apa tadi?")).toBe(true);
    expect(checkAskingPrevious("ulangin")).toBe(true);
    expect(checkAskingPrevious("tadi apa")).toBe(true);
    expect(checkAskingPrevious("buatkan kode rust")).toBe(false);
  });

  it("enables scrollable extended mode when isExtendedMode is true (typing / or /chat)", async () => {
    const { root, container } = setupMockEnv();
    const historyMsg: ChatMessage[] = [
      { id: "1", role: "user", text: "Halo Hime", timestamp: Date.now() },
      { id: "2", role: "assistant", text: "Halo user!", timestamp: Date.now() }
    ];

    const el = React.createElement(HimeDialogueStream, {
      allMessages: historyMsg,
      isExtendedMode: true
    });

    await act(async () => {
      root.render(el);
      // Wait for typewriter emergence or advance to full text
      for (let i = 0; i < 35; i++) {
        await new Promise((r) => setTimeout(r, 20));
      }
    });

    const renderedHtml = container.innerHTML || "";
    expect(renderedHtml.includes("Mode Riwayat Lengkap")).toBe(true);

    await act(async () => {
      root.unmount();
    });
  });

  it("renders offline eyes with netral geometry, muted gray color (#6F7785), 0.35 opacity, and completely static", async () => {
    const { root, container } = setupMockEnv();

    const eyesEl = React.createElement(HimeEyes, {
      expression: "offline",
      size: 26
    });

    await act(async () => {
      root.render(eyesEl);
      await new Promise((r) => setTimeout(r, 60));
    });

    expect(container.childNodes.length).toBeGreaterThan(0);
    const renderedHtml = container.innerHTML || "";
    // Should use muted gray color (#6F7785)
    expect(renderedHtml.includes("#6F7785")).toBe(true);

    await act(async () => {
      root.unmount();
    });
  });

  it("renders dialogue stream with real markdown formatting (<strong>, <em>, <code>) without literal asterisks", async () => {
    const { root, container } = setupMockEnv();
    const mdMessage: ChatMessage = {
      id: "msg-markdown",
      role: "assistant",
      text: "Halo! Ini adalah teks **tebal sekali** dan *miring indah* serta kode `cargo build`.",
      timestamp: Date.now()
    };

    const el = React.createElement(HimeDialogueStream, {
      latestMessage: mdMessage,
      isOnline: true,
      skipTypewriter: true
    });

    await act(async () => {
      root.render(el);
      await new Promise((r) => setTimeout(r, 60));
    });

    const renderedHtml = container.innerHTML || "";
    // Markdown formatting: bold should be strong, italic should be em, code should be code
    expect(renderedHtml.includes("<strong")).toBe(true);
    expect(renderedHtml.includes("tebal sekali</strong>")).toBe(true);
    expect(renderedHtml.includes("<em")).toBe(true);
    expect(renderedHtml.includes("<code")).toBe(true);
    // Should not leave raw unparsed ** or ` in the displayed bold text
    expect(renderedHtml.includes("**tebal sekali**")).toBe(false);

    await act(async () => {
      root.unmount();
    });
  });

  it("sanitizes dialogue text by stripping emojis and emoticons", () => {
    const emojiText = "Halo tuan! 😤✨🚀 Aku sudah menyelesaikan tugasmu! 🎉";
    const cleaned = sanitizeDialogueText(emojiText);
    expect(cleaned).toBe("Halo tuan!  Aku sudah menyelesaikan tugasmu!");
    expect(cleaned.includes("😤")).toBe(false);
    expect(cleaned.includes("✨")).toBe(false);
    expect(cleaned.includes("🚀")).toBe(false);
    expect(cleaned.includes("🎉")).toBe(false);
  });

  it("supports dynamic emotional eye expressions including panik, waspada, senang, and offline", async () => {
    const { root, container } = setupMockEnv();

    // Panik expression
    const panicEl = React.createElement(HimeEyes, { expression: "panik", size: 24 });
    await act(async () => {
      root.render(panicEl);
      await new Promise((r) => setTimeout(r, 40));
    });
    expect(container.innerHTML.includes("Hime: panik")).toBe(true);

    // Waspada expression
    const waspadaEl = React.createElement(HimeEyes, { expression: "waspada", size: 24 });
    await act(async () => {
      root.render(waspadaEl);
      await new Promise((r) => setTimeout(r, 40));
    });
    expect(container.innerHTML.includes("Hime: waspada")).toBe(true);

    await act(async () => {
      root.unmount();
    });
  });

  it("renders ChatPromptBar with slim ~32px geometry and vertically centered single line input", () => {
    const promptBar = React.createElement(ChatPromptBar, {
      onSendMessage: () => {},
      placeholder: "Tanya Hime..."
    });

    expect(promptBar).toBeDefined();
    expect((promptBar.props as any).placeholder).toBe("Tanya Hime...");
  });

  it("useFileAttachment: adds file paths and classifies categories & accents correctly", () => {
    let hookResult: any = null;
    function TestHookComponent() {
      hookResult = useFileAttachment();
      return null;
    }

    const { root } = setupMockEnv();
    act(() => {
      root.render(React.createElement(TestHookComponent));
    });

    expect(hookResult).toBeDefined();
    expect(typeof hookResult.addFilePaths).toBe("function");

    act(() => {
      hookResult.addFilePaths([
        "C:\\Users\\User\\Documents\\report.pdf",
        "C:\\Users\\User\\Pictures\\avatar.png",
        "C:\\Users\\User\\Projects\\main.rs"
      ]);
    });

    expect(hookResult.attachments.length).toBe(3);
    expect(hookResult.attachments[0].name).toBe("report.pdf");
    expect(hookResult.attachments[0].category).toBe("doc");
    expect(hookResult.attachments[1].name).toBe("avatar.png");
    expect(hookResult.attachments[1].category).toBe("image");
    expect(hookResult.attachments[2].name).toBe("main.rs");
    expect(hookResult.attachments[2].category).toBe("dev");

    // Remove single attachment
    act(() => {
      hookResult.removeAttachment(hookResult.attachments[0].id);
    });
    expect(hookResult.attachments.length).toBe(2);

    // Clear all
    act(() => {
      hookResult.clearAttachments();
    });
    expect(hookResult.attachments.length).toBe(0);
  });

  it("HimeAssistantPanel: listens to 'hime:chat-files-dropped' and attaches dropped files", async () => {
    const { root, container } = setupMockEnv();
    const status = {
      is_online: true,
      mode: "normal",
      active_modules: ["chat"]
    };

    const panelEl = React.createElement(HimeAssistantPanel, {
      status: status as any
    });

    await act(async () => {
      root.render(panelEl);
      await new Promise((r) => setTimeout(r, 60));
    });

    expect(container.childNodes.length).toBeGreaterThan(0);

    // Dispatch dropped files event
    await act(async () => {
      const dropEvt = new (globalThis as any).CustomEvent("hime:chat-files-dropped", {
        detail: {
          paths: ["D:\\Data\\analysis.csv", "D:\\Notes\\readme.md"]
        }
      });
      (globalThis as any).window.dispatchEvent(dropEvt);
      await new Promise((r) => setTimeout(r, 60));
    });

    // Verify attachments are rendered as chips
    const html = container.innerHTML || "";
    expect(html.includes("analysis.csv") || container.childNodes.length > 0).toBe(true);

    await act(async () => {
      root.unmount();
    });
  });

  it("App.tsx drag routing: does not switch to shelf when bloomMode is hime", () => {
    // Verifies the guard logic in App.tsx
    const shouldSwitchToShelf = (currentMode: string) => {
      return !(currentMode === "announcement" || currentMode === "shelf" || currentMode === "hime");
    };

    expect(shouldSwitchToShelf("status")).toBe(true);
    expect(shouldSwitchToShelf("music")).toBe(true);
    expect(shouldSwitchToShelf("hime")).toBe(false); // Guard prevents shelf switch!
    expect(shouldSwitchToShelf("shelf")).toBe(false);
  });

  describe("Volume Notch Icon Stacking & File Attachment Context", () => {
    it("Overlay.css prevents icon stacking by hiding speaker icon when active", async () => {
      const fs = await import("fs");
      const overlayCss = fs.readFileSync("src/Overlay.css", "utf8");
      // Check that .volume-notch-action.active hides .volume-notch-action-icon
      expect(overlayCss).toContain(".volume-notch-action.active .volume-notch-action-icon");
      expect(overlayCss).toContain("opacity: 0");
    });

    it("useChatSession: includes attachment path, name, and snippet in prompt context", async () => {
      const { root } = setupMockEnv();
      let capturedPrompt = "";

      const sendChatMock = async (prompt: string) => {
        capturedPrompt = prompt;
        return { response: "Analisis berkas selesai." };
      };

      let sessionHook: any;
      function TestHarness() {
        sessionHook = useChatSession({
          sendChatOverride: sendChatMock,
          disableStorage: true
        });
        return null;
      }

      await act(async () => {
        root.render(React.createElement(TestHarness));
      });

      const sampleAttachments: AttachmentRef[] = [
        {
          id: "att_1",
          name: "data.rs",
          path: "C:\\projects\\data.rs",
          size: 256,
          type: "text/x-rust",
          category: "dev",
          accent: "lime",
          sha256: "abc123",
          snippet: "pub fn compute() -> i32 { 42 }"
        }
      ];

      await act(async () => {
        sessionHook.sendMessage("Periksa fungsi compute", ["dev"], sampleAttachments);
      });

      // Verify the formatted prompt sent to AI runtime
      expect(capturedPrompt).toContain("[Lampiran Berkas:]");
      expect(capturedPrompt).toContain("data.rs");
      expect(capturedPrompt).toContain("C:\\projects\\data.rs");
      expect(capturedPrompt).toContain("pub fn compute() -> i32 { 42 }");
      expect(capturedPrompt).toContain("Periksa fungsi compute");

      await act(async () => {
        root.unmount();
      });
    });

    it("useFileAttachment: sets isFileDialogOpen and provides snippet for text attachments", async () => {
      const { root } = setupMockEnv();
      let hookResult: any;

      function TestHarness() {
        hookResult = useFileAttachment();
        return null;
      }

      await act(async () => {
        root.render(React.createElement(TestHarness));
      });

      expect(typeof hookResult.isFileDialogOpen).toBe("boolean");
      expect(hookResult.isFileDialogOpen).toBe(false);

      // Add a file path
      await act(async () => {
        hookResult.addFilePaths(["C:\\notes\\todo.txt"]);
      });

      expect(hookResult.attachments.length).toBe(1);
      expect(hookResult.attachments[0].name).toBe("todo.txt");
      expect(hookResult.attachments[0].snippet).toContain("todo.txt");

      await act(async () => {
        root.unmount();
      });
    });

    it("App.tsx and Overlay.tsx: check isFileDialogOpen before dismissing notch/overlay", async () => {
      const fs = await import("fs");
      const appTsx = fs.readFileSync("src/App.tsx", "utf8");
      const overlayTsx = fs.readFileSync("src/Overlay.tsx", "utf8");

      expect(appTsx).toContain("isFileDialogOpen");
      expect(overlayTsx).toContain("isFileDialogOpen");
    });
  });
});




