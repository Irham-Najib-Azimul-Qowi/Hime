import React, { useState, useRef, useEffect } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { HimeCoreStatus } from "../types";
import { EyeExpression } from "../ui/HimeEyes";
import {
  useChatSession,
  useFileAttachment,
  useVoiceInput,
  ChatHeader,
  ChatMessageItem,
  RunCard,
  ChatPromptBar,
  SKILL_CATALOG
} from "../chat";

export interface SkillItem {
  id: string;
  slash: string;
  name: string;
  desc: string;
  color: string;
  icon: any;
}

// Backwards compatibility export
export const SKILL_OPTIONS: SkillItem[] = SKILL_CATALOG.map((s) => ({
  id: s.id,
  slash: s.slash,
  name: s.name,
  desc: s.desc,
  color: `var(--accent-${s.accent})`,
  icon: s.iconName
}));

export interface HimeAssistantPanelProps {
  status: HimeCoreStatus;
  onClose?: () => void;
  onOpenTasks?: () => void;
  setNotchPinned?: (pinned: boolean) => void;
  submitPresentation?: (p: any) => void;
  retractPresentation?: (id: string) => void;
}

/**
 * Refactored Modular Hime Assistant Panel
 * - Non-blocking input: submits <= 1s without disabling input
 * - Multi-run concurrency: max 2 active runs, excess queued as Antre #n
 * - Targeted cancellation by runId
 * - Universal file attachments (Dnd, native picker, paste)
 * - Push-to-Talk voice input (Web Audio API RMS <= 30Hz)
 * - NCM progress synchronization
 */
export const HimeAssistantPanel: React.FC<HimeAssistantPanelProps> = ({
  status,
  onClose,
  onOpenTasks,
  setNotchPinned,
  submitPresentation,
  retractPresentation
}) => {
  const isOnline = status.is_online;
  const [showSearch, setShowSearch] = useState(false);
  const [searchQuery, setSearchQuery] = useState("");

  const chatScrollRef = useRef<HTMLDivElement>(null);

  // File attachment pipeline
  const {
    attachments,
    isDraggingOver,
    removeAttachment,
    clearAttachments,
    openFilePicker,
    onDragOver,
    onDragLeave,
    onDrop,
    onPaste
  } = useFileAttachment();

  // Multi-Run Chat Session engine
  const {
    messages,
    runs,
    isEnforcementPaused,
    sendMessage,
    cancelRun,
    clearMessages,
    setSearchFilter
  } = useChatSession({
    submitPresentation,
    retractPresentation
  });

  // Voice Push-to-Talk engine
  const {
    isRecording: isVoiceRecording,
    audioLevel: voiceAudioLevel,
    toggleRecording: toggleVoice
  } = useVoiceInput({
    onTranscript: (transcript) => {
      if (transcript.trim()) {
        sendMessage(transcript.trim(), [], []);
      }
    },
    submitPresentation,
    retractPresentation
  });

  const claimFocus = () => {
    setNotchPinned?.(true);
    if (typeof window !== "undefined") {
      void getCurrentWebviewWindow()
        .setFocus()
        .catch(() => {});
    }
  };

  useEffect(() => {
    claimFocus();
    const t = setTimeout(claimFocus, 80);
    return () => clearTimeout(t);
  }, []);

  const scrollToBottom = (smooth = true) => {
    if (chatScrollRef.current) {
      chatScrollRef.current.scrollTo({
        top: chatScrollRef.current.scrollHeight,
        behavior: smooth ? "smooth" : "auto"
      });
    }
  };

  useEffect(() => {
    scrollToBottom(false);
  }, [messages.length, runs]);

  // Derived mascot expression
  const mascotExpression: EyeExpression = isVoiceRecording
    ? "dengar"
    : runs.some((r) => r.status === "running")
    ? "berpikir"
    : runs.some((r) => r.status === "completed" && Date.now() - (r.completedAt || 0) < 2500)
    ? "senang"
    : !isOnline
    ? "offline"
    : "netral";

  const handleSearchChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const val = e.target.value;
    setSearchQuery(val);
    setSearchFilter(val);
  };

  const handleSendPrompt = (text: string, skills: string[], attached: any[]) => {
    sendMessage(text, skills, attached);
    clearAttachments();
  };

  // Filter in-flight / active runs to render inline
  const activeRuns = runs.filter(
    (r) =>
      r.status === "running" ||
      r.status === "queued" ||
      r.status === "error" ||
      r.status === "cancelled"
  );

  return (
    <motion.div
      className="hime-assistant-panel"
      initial={{ opacity: 0, scale: 0.98 }}
      animate={{ opacity: 1, scale: 1 }}
      exit={{ opacity: 0, scale: 0.98 }}
      transition={{ type: "spring", stiffness: 450, damping: 30 }}
      onDragOver={onDragOver}
      onDragLeave={onDragLeave}
      onDrop={onDrop}
      onPaste={onPaste}
      onClick={(e) => {
        e.stopPropagation();
        claimFocus();
      }}
      style={{
        display: "flex",
        flexDirection: "column",
        height: "100%",
        width: "100%",
        padding: "8px 12px 10px 12px",
        color: "#F4F6FA",
        fontFamily: "inherit",
        boxSizing: "border-box",
        overflow: "hidden",
        position: "relative",
        background: isDraggingOver ? "rgba(52, 227, 166, 0.08)" : "transparent",
        border: isDraggingOver ? "2px dashed #34E3A6" : "none",
        borderRadius: "16px",
        transition: "background 0.2s ease, border 0.2s ease"
      }}
    >
      {/* Chat Header */}
      <ChatHeader
        isOnline={isOnline}
        expression={mascotExpression}
        isPaused={isEnforcementPaused}
        onOpenSearch={() => setShowSearch((prev) => !prev)}
        onOpenTasks={onOpenTasks}
        onClearChat={clearMessages}
        onClose={onClose}
      />

      {/* Quick Search Input */}
      {showSearch && (
        <motion.div
          initial={{ opacity: 0, height: 0 }}
          animate={{ opacity: 1, height: "auto" }}
          exit={{ opacity: 0, height: 0 }}
          style={{ marginBottom: "6px" }}
        >
          <input
            type="text"
            value={searchQuery}
            onChange={handleSearchChange}
            placeholder="Cari pesan..."
            style={{
              width: "100%",
              padding: "4px 8px",
              borderRadius: "8px",
              background: "rgba(255, 255, 255, 0.06)",
              border: "1px solid rgba(255, 255, 255, 0.1)",
              color: "#F4F6FA",
              fontSize: "11px",
              outline: "none",
              boxSizing: "border-box"
            }}
          />
        </motion.div>
      )}

      {/* Messages & Process Cards Scroll Stream */}
      <div
        ref={chatScrollRef}
        className="hime-scrollbar"
        style={{
          flex: 1,
          minHeight: 0,
          display: "flex",
          flexDirection: "column",
          overflowY: "auto",
          overflowX: "hidden",
          paddingRight: "4px",
          marginBottom: "8px",
          scrollbarWidth: "thin",
          scrollbarColor: "rgba(255, 255, 255, 0.2) transparent"
        }}
      >
        {/* Welcome Empty State */}
        {messages.length === 0 && activeRuns.length === 0 && (
          <div
            style={{
              margin: "auto",
              textAlign: "center",
              padding: "24px 12px",
              color: "#A7AEBB"
            }}
          >
            <div style={{ fontSize: "13px", fontWeight: 600, color: "#F4F6FA", marginBottom: "4px" }}>
              Hime Island
            </div>
            <div style={{ fontSize: "11px", color: "#6F7785", marginBottom: "16px" }}>
              Asisten otonom siap membantu
            </div>

            <div
              style={{
                display: "flex",
                flexWrap: "wrap",
                gap: "6px",
                justifyContent: "center",
                maxWidth: "280px",
                margin: "0 auto"
              }}
            >
              {SKILL_CATALOG.slice(0, 4).map((s) => (
                <button
                  key={s.id}
                  type="button"
                  onClick={() => handleSendPrompt(`Tolong jelaskan kemampuan ${s.slash}`, [s.id], [])}
                  style={{
                    background: "rgba(255, 255, 255, 0.04)",
                    border: "1px solid rgba(255, 255, 255, 0.08)",
                    borderRadius: "8px",
                    padding: "4px 8px",
                    fontSize: "10.5px",
                    color: "#F4F6FA",
                    cursor: "pointer"
                  }}
                >
                  {s.slash}
                </button>
              ))}
            </div>
          </div>
        )}

        {/* Render Chat Messages */}
        {messages.map((msg) => (
          <ChatMessageItem key={msg.id} message={msg} />
        ))}

        {/* Render Active and Queued Process Runs */}
        <AnimatePresence>
          {activeRuns.map((run) => (
            <RunCard
              key={run.id}
              run={run}
              onCancel={cancelRun}
              onRetry={(failedRun) => sendMessage(failedRun.prompt, failedRun.skills, failedRun.attachments)}
            />
          ))}
        </AnimatePresence>
      </div>

      {/* Non-Blocking Chat Prompt Bar */}
      <ChatPromptBar
        onSendMessage={handleSendPrompt}
        attachments={attachments}
        onRemoveAttachment={removeAttachment}
        onOpenFilePicker={openFilePicker}
        isVoiceRecording={isVoiceRecording}
        voiceAudioLevel={voiceAudioLevel}
        onToggleVoice={toggleVoice}
      />
    </motion.div>
  );
};
