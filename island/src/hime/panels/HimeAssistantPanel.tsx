import React, { useEffect, useState, useRef } from "react";
import { motion } from "framer-motion";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { HimeCoreStatus } from "../types";
import { HimeEyes, EyeExpression } from "../ui/HimeEyes";
import {
  useChatSession,
  useFileAttachment,
  useVoiceInput,
  ChatPromptBar,
  HimeDialogueStream,
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
 * Hime Dialogue & Assistant Monologue Panel
 * - Header Cleanup:
 *   * Top status bar (clock/net speed/battery row) hidden in Assistant view
 *   * Removed • HIME:CORE row, task icon, repeat/refresh icon, and close X icon
 *   * Keeps ONLY the two animated robot eyes in the center
 * - Horizontal Header Scrolling:
 *   * Switching panels triggered by horizontal scroll (deltaX / wheel left-right) on top header/eyes
 * - Offline Eyes:
 *   * When offline (!isOnline), displays netral eye geometry with #6F7785 muted gray, 0.35 opacity, completely static
 * - Dual Chat Modes:
 *   * Default Mode: Otome visual novel dialogue stopping right above prompt bar (zero vertical scrollbar)
 *   * In-place sequential chunking for long text; playful pouty reaction ("Hmph...") on asking for previous text
 *   * Extended Mode: typing / or /chat enables scrollable dialogue history
 * - Slim Single-Line Input Bar:
 *   * Compact ~36px height by default with "Tanya Hime..." placeholder; expands up to 2 lines (~64px) on wrap
 */
export const HimeAssistantPanel: React.FC<HimeAssistantPanelProps> = ({
  status,
  onClose,
  onOpenTasks: _onOpenTasks,
  setNotchPinned,
  submitPresentation,
  retractPresentation
}) => {
  const isOnline = status.is_online;
  const [inputText, setInputText] = useState("");
  const [isPouty, setIsPouty] = useState(false);
  const [poutyMessage, setPoutyMessage] = useState<string | null>(null);
  const [emotionOverride, setEmotionOverride] = useState<EyeExpression | null>(null);
  const poutyTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const emotionTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const triggerEmotion = (expr: EyeExpression, duration = 3000) => {
    setEmotionOverride(expr);
    if (emotionTimerRef.current) clearTimeout(emotionTimerRef.current);
    emotionTimerRef.current = setTimeout(() => {
      setEmotionOverride(null);
    }, duration);
  };

  // File attachment pipeline
  const {
    attachments,
    isDraggingOver,
    isFileDialogOpen,
    addFilePaths,
    removeAttachment,
    clearAttachments,
    openFilePicker,
    onDragOver,
    onDragLeave,
    onDrop,
    onPaste
  } = useFileAttachment();

  // Pin notch while file picker dialog is open to prevent auto-hide
  useEffect(() => {
    if (isFileDialogOpen) {
      setNotchPinned?.(true);
    }
  }, [isFileDialogOpen, setNotchPinned]);

  // Listen to files dropped onto notch while in Assistant view
  useEffect(() => {
    const handleDroppedFiles = (e: Event) => {
      const customEvt = e as CustomEvent<{ paths?: string[]; text?: string }>;
      const paths = customEvt.detail?.paths;
      if (paths && paths.length > 0) {
        addFilePaths(paths);
      }
    };

    window.addEventListener("hime:chat-files-dropped", handleDroppedFiles);
    return () => {
      window.removeEventListener("hime:chat-files-dropped", handleDroppedFiles);
    };
  }, [addFilePaths]);

  // Multi-Run Chat Session engine
  const {
    messages,
    runs,
    sendMessage,
    cancelRun
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
    if (typeof window !== "undefined" && (window as any).__TAURI_INTERNALS__) {
      try {
        void getCurrentWebviewWindow()
          .setFocus()
          .catch(() => {});
      } catch {
        // Headless or non-Tauri browser context
      }
    }
  };

  useEffect(() => {
    claimFocus();
    const t = setTimeout(claimFocus, 80);
    return () => clearTimeout(t);
  }, []);

  // Find latest assistant monologue / response
  const assistantMessages = messages.filter((m) => m.role === "assistant");
  const latestMessage =
    assistantMessages.length > 0
      ? assistantMessages[assistantMessages.length - 1]
      : null;

  // Trigger playful pouty reaction ("Hmph, masa kamu lupa apa yang baru saja aku bilang!")
  // Strictly zero emoticons/emojis in text: mascot eyes handle the emotional expression
  const triggerPouty = (customPrev?: string) => {
    setIsPouty(true);
    const prevText = customPrev || latestMessage?.text || "Aku baru saja menyapamu!";
    setPoutyMessage(`Hmph, masa kamu lupa apa yang baru saja aku bilang!\n\nNih, aku ingatkan:\n"${prevText}"`);

    if (poutyTimerRef.current) clearTimeout(poutyTimerRef.current);
    poutyTimerRef.current = setTimeout(() => {
      setIsPouty(false);
      setPoutyMessage(null);
    }, 4500);
  };

  // Derived mascot expression dynamically reflecting emotional states:
  // - offline (!isOnline): "offline" (static muted gray)
  // - pouty: "kesal" or "marah"
  // - hearing voice: "dengar"
  // - active processing / queued: "berpikir"
  // - error / trouble: "panik" or "waspada"
  // - success / compliment: "senang" or "senyum"
  // - default: "netral"
  const mascotExpression: EyeExpression = !isOnline
    ? "offline"
    : isPouty
    ? "kesal"
    : isVoiceRecording
    ? "dengar"
    : runs.some((r) => r.status === "running" || r.status === "queued")
    ? "berpikir"
    : runs.some((r) => r.status === "error")
    ? "panik"
    : emotionOverride
    ? emotionOverride
    : runs.some((r) => r.status === "completed" && Date.now() - (r.completedAt || 0) < 3000)
    ? "senang"
    : "netral";

  const handleSendPrompt = (text: string, skills: string[], attached: any[]) => {
    const trimmed = text.trim();
    const isAskingPrevious = /^(apa tadi\??|ulangin|ulang|tadi apa\??|tadi ngomong apa\??|sebelumnya apa\??|ulang dong)/i.test(trimmed);
    if (isAskingPrevious && (latestMessage?.text || messages.length > 0)) {
      triggerPouty();
      clearAttachments();
      return;
    }

    // Compliment and alert sentiment detection
    if (/(\b(terima kasih|makasih|pintar|hebat|keren|kamu lucu|cantik|terbaik|good job)\b)/i.test(trimmed)) {
      triggerEmotion("senang", 3500);
    } else if (/(\b(bahaya|tolong|rusak|error|bug|masalah|gagal|awas)\b)/i.test(trimmed)) {
      triggerEmotion("waspada", 3500);
    }

    if (poutyTimerRef.current) clearTimeout(poutyTimerRef.current);
    setIsPouty(false);
    setPoutyMessage(null);
    sendMessage(trimmed, skills, attached);
    clearAttachments();
  };

  // Find currently active in-flight run
  const activeRun =
    runs.find((r) => r.status === "running") ||
    runs.find((r) => r.status === "queued") ||
    runs.find((r) => r.status === "error" && Date.now() - (r.createdAt || 0) < 10000) ||
    null;

  // Extended Mode: typing / or /chat enables scrollable full dialogue history
  const isExtendedMode = inputText.startsWith("/") || inputText.startsWith("/chat");

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
        justifyContent: "space-between",
        height: "100%",
        width: "100%",
        padding: "4px 12px 10px 12px",
        color: "#F4F6FA",
        fontFamily: "'Consolas', 'Cascadia Code', 'JetBrains Mono', monospace",
        boxSizing: "border-box",
        overflow: "hidden", // STRICT: Zero scrollbar on root
        position: "relative",
        background: isDraggingOver ? "rgba(52, 227, 166, 0.08)" : "transparent",
        border: isDraggingOver ? "2px dashed #34E3A6" : "none",
        borderRadius: "16px",
        transition: "background 0.2s ease, border 0.2s ease"
      }}
    >
      {/* 1. Clean Header: ONLY the two animated robot eyes dead-center */}
      <div
        className="hime-assistant-header"
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          padding: "16px 0 6px 0",
          position: "relative",
          minHeight: "42px",
          flexShrink: 0
        }}
      >
        <div
          className="hime-eyes-container"
          style={{
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            cursor: "pointer"
          }}
          title={`Hime: ${mascotExpression}`}
          onClick={onClose}
        >
          <HimeEyes
            size={26}
            expression={mascotExpression}
            audioLevel={voiceAudioLevel}
          />
        </div>
      </div>

      {/* 2. Dual Mode Dialogue Stream Display (Default Otome Mode & Extended Scrollable Mode) */}
      <HimeDialogueStream
        activeRun={activeRun}
        latestMessage={latestMessage}
        allMessages={messages}
        isOnline={isOnline}
        isExtendedMode={isExtendedMode}
        onSelectSkill={(prompt, skills) => handleSendPrompt(prompt, skills, [])}
        onCancelRun={cancelRun}
        onPoutyReaction={() => triggerPouty()}
        poutyMessage={poutyMessage}
        style={{ marginBottom: "6px" }}
      />

      {/* 3. Slim Single-Line ChatGPT-style Prompt Input Bar */}
      <ChatPromptBar
        onSendMessage={handleSendPrompt}
        onTextChange={setInputText}
        attachments={attachments}
        onRemoveAttachment={removeAttachment}
        onOpenFilePicker={() => {
          setNotchPinned?.(true);
          openFilePicker();
        }}
        isFileDialogOpen={isFileDialogOpen}
        isVoiceRecording={isVoiceRecording}
        voiceAudioLevel={voiceAudioLevel}
        onToggleVoice={toggleVoice}
      />
    </motion.div>
  );
};

