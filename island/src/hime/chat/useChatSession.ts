import { useState, useRef, useCallback, useEffect } from "react";
import { ChatMessage, ChatRun, AttachmentRef, RunStatus } from "./types";
import { sendChatMessage, createAcademicReport } from "../api";
import { PriorityLevel } from "../ncm/types";

export interface UseChatSessionOptions {
  submitPresentation?: (p: any) => void;
  retractPresentation?: (id: string) => void;
  sendChatOverride?: (
    prompt: string,
    model?: string
  ) => Promise<{ response: string; conversation_id?: string }>;
  maxConcurrentRuns?: number;
  initialMessages?: ChatMessage[];
  initialRuns?: ChatRun[];
  disableStorage?: boolean;
}

export const MAX_ACTIVE_RUNS = 2;
export const GLOBAL_CONVERSATION_ID = "conv-global";

/**
 * Strips repetitive AI templates and emojis
 */
export function cleanAiResponse(text: string): string {
  if (!text) return "";
  return text
    .replace(/^(\s*)[🤖✨💡🚀📌🎯🔥🎉⚡]\s*/gm, "$1- ")
    .replace(/[🤖✨💡🚀📌🎯🔥🎉⚡]/g, "")
    .trim();
}

/**
 * Multi-Run Concurrency Scheduler Hook
 * - Concurrency limit: max 2 parallel active runs (status: "running")
 * - Additional runs enter FIFO queue (status: "queued", "Antre #n")
 * - When active runs finish/fail/cancel, FIFO dequeues & promotes next
 * - Non-blocking: submits clear input immediately <= 1s
 * - Single persistent thread: "conv-global"
 */
export function useChatSession(options: UseChatSessionOptions = {}) {
  const {
    submitPresentation,
    retractPresentation,
    sendChatOverride,
    maxConcurrentRuns = MAX_ACTIVE_RUNS,
    initialMessages,
    initialRuns,
    disableStorage = false
  } = options;

  const [messages, setMessages] = useState<ChatMessage[]>(() => {
    if (initialMessages) return initialMessages;
    if (!disableStorage && typeof localStorage !== "undefined") {
      try {
        const saved = localStorage.getItem("hime-chat-history");
        return saved ? JSON.parse(saved) : [];
      } catch {
        return [];
      }
    }
    return [];
  });

  const [runs, setRunsState] = useState<ChatRun[]>(() => initialRuns || []);
  const [searchFilter, setSearchFilter] = useState<string>("");
  const [isEnforcementPaused, setIsEnforcementPaused] = useState<boolean>(false);

  const runsRef = useRef<ChatRun[]>(initialRuns || []);

  const setRuns = useCallback((updater: (prev: ChatRun[]) => ChatRun[]) => {
    setRunsState((prev) => {
      const next = updater(prev);
      runsRef.current = next;
      return next;
    });
  }, []);

  // Persist messages to local mirror
  useEffect(() => {
    if (!disableStorage && typeof localStorage !== "undefined") {
      try {
        localStorage.setItem("hime-chat-history", JSON.stringify(messages.slice(-30)));
      } catch {
        // Storage quota / security policy
      }
    }
  }, [messages, disableStorage]);

  // Synchronize active run presentation with NCM
  useEffect(() => {
    const activeRuns = runs.filter((r) => r.status === "running");
    if (activeRuns.length > 0) {
      const topRun = activeRuns[0];
      const presentation = {
        id: `chat-run-${topRun.id}`,
        module: "chat",
        kind: "panel-hint",
        priority: PriorityLevel.P3_PROGRESS,
        userInitiated: true, // Elevates effective priority to 2
        expression: "berpikir",
        payload: {
          title: topRun.statusText.slice(0, 36),
          progress: topRun.progress
        }
      };

      if (submitPresentation) {
        submitPresentation(presentation);
      } else if (typeof window !== "undefined" && typeof window.dispatchEvent === "function") {
        window.dispatchEvent(new CustomEvent("hime:ncm-submit", { detail: presentation }));
      }
    } else {
      // Retract any lingering chat run presentations
      runs.forEach((r) => {
        if (retractPresentation) {
          retractPresentation(`chat-run-${r.id}`);
        } else if (typeof window !== "undefined" && typeof window.dispatchEvent === "function") {
          window.dispatchEvent(
            new CustomEvent("hime:ncm-retract", { detail: { id: `chat-run-${r.id}` } })
          );
        }
      });
    }
  }, [runs, submitPresentation, retractPresentation]);

  // Execute a single run
  const executeRun = useCallback(
    async (runId: string) => {
      const run = runsRef.current.find((r) => r.id === runId);
      if (!run || run.status !== "running") return;

      try {
        // Progress step: Step 1
        setRuns((prev) =>
          prev.map((r) =>
            r.id === runId && r.status === "running"
              ? {
                  ...r,
                  progress: 35,
                  statusText: "Memproses konteks...",
                  steps: [...r.steps, "Analisis prompt selesai"]
                }
              : r
          )
        );

        let botReply = "";

        // Academic report fast-path
        const lowerPrompt = run.prompt.toLowerCase();
        if (
          lowerPrompt.includes("laporan akademik") ||
          lowerPrompt.includes("academic report")
        ) {
          const reportTitle = "Laporan_Akademik_Hime";
          const sampleBody = `# LAPORAN AKADEMIK SISTEM HIME\n## BAB I PENDAHULUAN\nHime adalah ekosistem asisten personal dengan runtime otonom.\n## BAB II PEMBAHASAN\nSistem menggunakan standar TNR 12 margin 4-3-3-3.\n## BAB III KESIMPULAN\nFormat karya ilmiah terverifikasi.`;
          const res = await createAcademicReport(reportTitle, sampleBody);
          botReply = `Dokumen laporan akademik berhasil dibuat.\n\n- Template: Akademik-ID (Margin 4-3-3-3)\n- Berkas: \`${res.output_path}\`\n- Ukuran: ${res.file_size_bytes} bytes`;
        } else {
          // Normal chat turn
          const activeModel =
            (typeof localStorage !== "undefined" &&
              localStorage.getItem("hime-9router-model")) ||
            "hime-combo";

          let attachmentContext = "";
          if (run.attachments && run.attachments.length > 0) {
            const lines: string[] = ["[Lampiran Berkas:]"];
            for (const att of run.attachments) {
              const fileInfo = att.path
                ? `- Berkas: ${att.name} (${att.path})`
                : `- Berkas: ${att.name}`;
              lines.push(fileInfo);
              const textContent = att.content || att.snippet;
              if (textContent && textContent.trim()) {
                const snippetText = textContent.slice(0, 1500).trim();
                lines.push("```\n" + snippetText + "\n```");
              }
            }
            attachmentContext = "\n\n" + lines.join("\n");
          }

          const promptBody = run.prompt
            ? `${run.prompt}${attachmentContext}`
            : `Tolong periksa dan analisis berkas terlampir:${attachmentContext}`;

          const formattedPrompt =
            run.skills.length > 0
              ? `[Mode Spesialis: ${run.skills.join(", ")}]\n${promptBody}\n\n[Format: Jawablah dengan lugas, profesional, dan rapi dalam Markdown standar. Tanpa emoji berlebihan.]`
              : `${promptBody}\n\n[Format: Jawablah dengan lugas, profesional, dan rapi dalam Markdown standar. Tanpa emoji berlebihan.]`;

          // Progress step: Step 2
          setRuns((prev) =>
            prev.map((r) =>
              r.id === runId && r.status === "running"
                ? {
                    ...r,
                    progress: 70,
                    statusText: "Menyiapkan jawaban...",
                    steps: [...r.steps, "Menghubungi AI runtime"]
                  }
                : r
            )
          );

          const sender = sendChatOverride || sendChatMessage;
          const res = await sender(formattedPrompt, activeModel);
          botReply = cleanAiResponse(res.response || "Respons diterima dari Hime Core.");
        }

        // Check if run was cancelled during async resolution
        const freshRun = runsRef.current.find((r) => r.id === runId);
        if (freshRun && freshRun.status === "cancelled") {
          return;
        }

        // Complete run
        setRuns((prev) =>
          prev.map((r) =>
            r.id === runId
              ? {
                  ...r,
                  status: "completed",
                  progress: 100,
                  statusText: "Selesai",
                  steps: [...r.steps, "Jawaban selesai dibuat"],
                  resultText: botReply,
                  completedAt: Date.now()
                }
              : r
          )
        );

        // Append assistant message to thread
        const assistantMsg: ChatMessage = {
          id: `ast_${Date.now()}_${Math.random().toString(36).substring(2, 6)}`,
          role: "assistant",
          text: botReply,
          timestamp: Date.now(),
          runId: runId
        };

        setMessages((prev) => [...prev, assistantMsg]);

        // Transient NCM celebration
        const celebration = {
          id: `celebrate-${runId}`,
          module: "chat",
          kind: "ambient",
          priority: PriorityLevel.P3_PROGRESS,
          expression: "senang",
          ttl: 2500
        };
        if (submitPresentation) {
          submitPresentation(celebration);
        } else if (typeof window !== "undefined" && typeof window.dispatchEvent === "function") {
          window.dispatchEvent(
            new CustomEvent("hime:ncm-submit", { detail: celebration })
          );
        }
      } catch (err: any) {
        // Check if run was already cancelled
        const freshRun = runsRef.current.find((r) => r.id === runId);
        if (freshRun && freshRun.status === "cancelled") return;

        const rawErr = err?.message || String(err || "Koneksi ke Hime Core gagal");
        const formattedErr = rawErr.slice(0, 44);

        setRuns((prev) =>
          prev.map((r) =>
            r.id === runId
              ? {
                  ...r,
                  status: "error",
                  error: formattedErr,
                  statusText: "Gagal memproses",
                  steps: [...r.steps, `Error: ${formattedErr}`]
                }
              : r
          )
        );
      }
    },
    [sendChatOverride, submitPresentation]
  );

  // FIFO Scheduler Pump: Promotes queued runs when active runs < maxConcurrentRuns
  useEffect(() => {
    const activeCount = runs.filter((r) => r.status === "running").length;
    if (activeCount < maxConcurrentRuns) {
      const nextQueued = runs.find((r) => r.status === "queued");
      if (nextQueued) {
        // Promote next queued run to running and re-index remaining queue
        let remainingQueuedIdx = 1;
        const updatedRuns = runsRef.current.map((r) => {
          if (r.id === nextQueued.id) {
            return {
              ...r,
              status: "running" as RunStatus,
              progress: 15,
              statusText: "Menganalisis...",
              steps: [...r.steps, "Mulai diproses"]
            };
          }
          if (r.status === "queued") {
            const updated = {
              ...r,
              queueIndex: remainingQueuedIdx,
              statusText: `Antre #${remainingQueuedIdx}`
            };
            remainingQueuedIdx++;
            return updated;
          }
          return r;
        });

        runsRef.current = updatedRuns;
        setRunsState(updatedRuns);

        // Execute promoted run
        executeRun(nextQueued.id);
      }
    }
  }, [runs, maxConcurrentRuns, executeRun]);

  // Targeted Run Cancellation: cancels specific run without touching siblings
  const cancelRun = useCallback((targetRunId?: string) => {
    setRuns((prev) => {
      let idToCancel = targetRunId;
      if (!idToCancel) {
        // Default to latest running or queued run
        const candidate = [...prev]
          .reverse()
          .find((r) => r.status === "running" || r.status === "queued");
        idToCancel = candidate?.id;
      }

      if (!idToCancel) return prev;

      let nextQueueIdx = 1;
      return prev.map((r) => {
        if (r.id === idToCancel) {
          return {
            ...r,
            status: "cancelled",
            statusText: "Dibatalkan",
            steps: [...r.steps, "Proses dibatalkan oleh pengguna"]
          };
        }
        if (r.status === "queued") {
          const updated = {
            ...r,
            queueIndex: nextQueueIdx,
            statusText: `Antre #${nextQueueIdx}`
          };
          nextQueueIdx++;
          return updated;
        }
        return r;
      });
    });
  }, []);

  // Clear all visible messages in panel view
  const clearMessages = useCallback(() => {
    setMessages([]);
    if (!disableStorage && typeof localStorage !== "undefined") {
      try {
        localStorage.removeItem("hime-chat-history");
      } catch {
        // Ignored
      }
    }
  }, [disableStorage]);

  // Non-blocking submission: clears input <= 1s without disabling input
  const sendMessage = useCallback(
    (promptText: string, skills: string[] = [], attachments: AttachmentRef[] = []) => {
      const trimmed = promptText.trim();
      if (!trimmed && attachments.length === 0) return;

      // Fast Control Commands Interception (07-CHAT-SPEC §3)
      if (trimmed.startsWith("/stop")) {
        const parts = trimmed.split(/\s+/);
        const specificId = parts[1];
        cancelRun(specificId);
        return;
      }

      if (trimmed.startsWith("/bersih")) {
        clearMessages();
        return;
      }

      if (trimmed.startsWith("/cari")) {
        const query = trimmed.replace(/^\/cari\s*/, "").trim();
        setSearchFilter(query);
        return;
      }

      if (trimmed.startsWith("/jeda")) {
        setIsEnforcementPaused((prev) => !prev);
        return;
      }

      const runId = `run_${Date.now()}_${Math.random().toString(36).substring(2, 7)}`;

      // 1. Optimistic User Message
      const userMsg: ChatMessage = {
        id: `usr_${Date.now()}_${Math.random().toString(36).substring(2, 6)}`,
        role: "user",
        text: trimmed || `[${attachments.length} berkas terlampir]`,
        timestamp: Date.now(),
        runId,
        skills,
        attachments
      };

      setMessages((prev) => [...prev, userMsg]);

      // 2. Concurrency Check: running vs queued
      const runningCount = runsRef.current.filter((r) => r.status === "running").length;
      const willRun = runningCount < maxConcurrentRuns;
      const queuedCount = runsRef.current.filter((r) => r.status === "queued").length;
      const queueNumber = queuedCount + 1;

      const newRun: ChatRun = {
        id: runId,
        prompt: trimmed,
        skills,
        attachments,
        status: willRun ? "running" : "queued",
        queueIndex: willRun ? undefined : queueNumber,
        progress: willRun ? 15 : 0,
        statusText: willRun ? "Menganalisis..." : `Antre #${queueNumber}`,
        steps: ["Permintaan diterima"],
        createdAt: Date.now()
      };

      runsRef.current = [...runsRef.current, newRun];
      setRuns((prev) => [...prev, newRun]);

      if (willRun) {
        executeRun(runId);
      }
    },
    [cancelRun, clearMessages, executeRun, maxConcurrentRuns]
  );

  const filteredMessages = messages.filter((m) => {
    if (!searchFilter) return true;
    return m.text.toLowerCase().includes(searchFilter.toLowerCase());
  });

  return {
    messages: filteredMessages,
    allMessages: messages,
    runs,
    searchFilter,
    isEnforcementPaused,
    sendMessage,
    cancelRun,
    clearMessages,
    setSearchFilter
  };
}
