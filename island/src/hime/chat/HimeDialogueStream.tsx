import React, { useState, useEffect, useMemo, useRef } from "react";
import { motion } from "framer-motion";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import { FileText, Sparkles } from "lucide-react";
import { ChatRun, ChatMessage } from "./types";
import { formatFileSize } from "./skillsCatalog";

export interface HimeDialogueStreamProps {
  activeRun?: ChatRun | null;
  latestMessage?: ChatMessage | null;
  allMessages?: ChatMessage[];
  isOnline?: boolean;
  isExtendedMode?: boolean;
  skipTypewriter?: boolean;
  onSelectSkill?: (prompt: string, skills: string[]) => void;
  onCancelRun?: (runId: string) => void;
  onPoutyReaction?: () => void;
  poutyMessage?: string | null;
  className?: string;
  style?: React.CSSProperties;
}

/**
 * Sanitizes dialogue text to guarantee Otome Game / Visual Novel subtitle style:
 * - Strips any "(hime:)", "hime:", "Hermes:", "(hermes:)", "assistant:" prefixes.
 * - Strips leading terminal prompt markers "> ".
 * - Never prefixes with "(hime:)" or "Hermes:".
 * - Strips emojis/emoticons: mascot eyes handle emotion dynamically; text stays clean.
 */
export function sanitizeDialogueText(raw: string): string {
  if (!raw) return "";
  let clean = raw.trim();
  clean = clean.replace(/^(\((?:hime|hermes):?\):?|(?:hime|hermes|assistant):)\s*/gi, "");
  clean = clean.replace(/^>\s*/, "");
  clean = clean.replace(/\n(\((?:hime|hermes):?\):?|(?:hime|hermes|assistant):)\s*/gi, "\n");
  // Remove system prompt / format instructions leaks
  clean = clean.replace(/\[Format:.*?\]/gi, "");
  clean = clean.replace(/Permintaan(?:mu| Anda):\s*>\s*\*.*?\*\s*/gi, "");
  // Remove emojis: mascot eyes express emotions; text remains crisp and emoji-free
  clean = clean.replace(/[\u{1F300}-\u{1F9FF}\u{2600}-\u{26FF}\u{2700}-\u{27BF}\u{1F1E6}-\u{1F1FF}\u{1F600}-\u{1F64F}\u{1F680}-\u{1F6FF}]/gu, "");
  return clean.trim();
}

/**
 * Standard Markdown components for Otome visual novel dialogue:
 * - **bold** renders as luminous high-contrast bold
 * - *italic* renders as cyan italic
 * - `code` renders as sleek monospace pill
 * - Zero residual literal asterisks or raw markdown tokens
 */
export const DIALOGUE_MARKDOWN_COMPONENTS = {
  p({ children }: any) {
    return <span style={{ display: "inline" }}>{children}</span>;
  },
  strong({ children }: any) {
    return (
      <strong
        style={{
          color: "#FFFFFF",
          fontWeight: 700,
          textShadow: "0 0 8px rgba(56, 214, 245, 0.25)"
        }}
      >
        {children}
      </strong>
    );
  },
  em({ children }: any) {
    return (
      <em
        style={{
          fontStyle: "italic",
          color: "#38D6F5"
        }}
      >
        {children}
      </em>
    );
  },
  code({ children }: any) {
    return (
      <code
        style={{
          background: "rgba(255, 255, 255, 0.08)",
          padding: "1px 5px",
          borderRadius: "4px",
          fontSize: "11px",
          fontFamily: "'Consolas', 'Cascadia Code', monospace",
          color: "#34E3A6",
          border: "1px solid rgba(52, 227, 166, 0.2)"
        }}
      >
        {children}
      </code>
    );
  },
  ul({ children }: any) {
    return <ul style={{ paddingLeft: "16px", margin: "2px 0 6px 0" }}>{children}</ul>;
  },
  ol({ children }: any) {
    return <ol style={{ paddingLeft: "16px", margin: "2px 0 6px 0" }}>{children}</ol>;
  },
  li({ children }: any) {
    return <li style={{ margin: "2px 0" }}>{children}</li>;
  },
  h1({ children }: any) {
    return <div style={{ fontSize: "13.5px", fontWeight: 700, color: "#38D6F5", margin: "3px 0" }}>{children}</div>;
  },
  h2({ children }: any) {
    return <div style={{ fontSize: "13px", fontWeight: 700, color: "#38D6F5", margin: "3px 0" }}>{children}</div>;
  },
  h3({ children }: any) {
    return <div style={{ fontSize: "12.5px", fontWeight: 600, color: "#34E3A6", margin: "2px 0" }}>{children}</div>;
  }
};

/**
 * Chunks long dialogue text into discrete visual novel scenes/chunks to guarantee:
 * - Response text stops right above prompt bar with zero clipping
 * - Zero vertical scrollbars in Otome dialogue mode
 * - Natural sentence & paragraph chunk boundaries
 */
export function chunkDialogueText(text: string, maxChars = 240): string[] {
  if (!text) return [""];
  const trimmed = sanitizeDialogueText(text);
  if (trimmed.length <= maxChars) return [trimmed];

  const chunks: string[] = [];
  let remaining = trimmed;

  while (remaining.length > 0) {
    if (remaining.length <= maxChars) {
      chunks.push(remaining.trim());
      break;
    }

    let sliceIdx = remaining.lastIndexOf("\n\n", maxChars);
    if (sliceIdx <= maxChars * 0.35) {
      sliceIdx = remaining.lastIndexOf("\n", maxChars);
    }
    if (sliceIdx <= maxChars * 0.35) {
      sliceIdx = remaining.lastIndexOf(". ", maxChars);
      if (sliceIdx !== -1) sliceIdx += 1;
    }
    if (sliceIdx <= maxChars * 0.35) {
      sliceIdx = remaining.lastIndexOf("? ", maxChars);
      if (sliceIdx !== -1) sliceIdx += 1;
    }
    if (sliceIdx <= maxChars * 0.35) {
      sliceIdx = remaining.lastIndexOf(" ", maxChars);
    }
    if (sliceIdx <= 0) {
      sliceIdx = maxChars;
    }

    chunks.push(remaining.slice(0, sliceIdx).trim());
    remaining = remaining.slice(sliceIdx).trim();
  }

  return chunks.length > 0 ? chunks : [trimmed];
}

// Backwards compatibility alias for existing test suites
export const paginateDialogueText = chunkDialogueText;

/**
 * Extracts mentioned file paths and backticked filenames from dialogue text:
 * Supports markdown links [label](file://path) or backticked `path/file.ext`
 */
export function extractMentionedFiles(text: string): string[] {
  if (!text) return [];
  const files: string[] = [];

  const mdLinkRegex = /\[([^\]]+)\]\((?:file:\/\/\/?)?([^)]+)\)/g;
  let m: RegExpExecArray | null;
  while ((m = mdLinkRegex.exec(text)) !== null) {
    let candidate = m[2];
    if (candidate) {
      candidate = candidate.replace(/^\/+/, "");
      if (candidate.includes(".") || candidate.includes("/") || candidate.includes("\\")) {
        if (!files.includes(candidate)) files.push(candidate);
      }
    }
  }

  const codeRegex = /`([^`\n\r\t ]+\.[a-zA-Z0-9]{1,8})`/g;
  while ((m = codeRegex.exec(text)) !== null) {
    const candidate = m[1];
    if (candidate && !files.includes(candidate)) {
      files.push(candidate);
    }
  }

  return files;
}

/**
 * Invokes native desktop file opener (shelf_open via Tauri) or browser fallback
 */
export async function openFileFromPill(filePath: string) {
  if (!filePath) return;
  try {
    if (typeof window !== "undefined" && (window as any).__TAURI_INTERNALS__) {
      const { invoke } = await import("@tauri-apps/api/core");
      await invoke("shelf_open", { path: filePath });
    } else {
      window.open(filePath, "_blank");
    }
  } catch (err) {
    console.warn("openFileFromPill failed:", filePath, err);
  }
}

/**
 * Hime Dialogue Stream supporting Dual Chat Modes:
 * 1. Default Mode (Otome dialogue mode):
 *    - Unboxed subtitle text stopping right above prompt bar (no vertical scrollbar)
 *    - Long responses chunked and displayed in-place sequentially
 *    - Pouty / playful reaction ("Hmph, masa kamu lupa...") when asking for previous text
 * 2. Extended Mode (typing / or /chat):
 *    - Scrollable full conversation history mode
 */
export const HimeDialogueStream: React.FC<HimeDialogueStreamProps> = ({
  activeRun,
  latestMessage,
  allMessages = [],
  isOnline: _isOnline = true,
  isExtendedMode = false,
  skipTypewriter = false,
  onSelectSkill,
  onCancelRun,
  onPoutyReaction,
  poutyMessage,
  className = "",
  style
}) => {
  const [displayedChars, setDisplayedChars] = useState<number>(0);
  const [chunkIndex, setChunkIndex] = useState<number>(0);
  const lastMsgIdRef = useRef<string | null>(null);
  const scrollRef = useRef<HTMLDivElement | null>(null);

  // Split response into natural in-place visual novel chunks
  const chunks = useMemo(() => {
    if (!latestMessage?.text) return [""];
    return chunkDialogueText(latestMessage.text, 240);
  }, [latestMessage?.text]);

  const currentChunk = Math.min(chunkIndex, Math.max(0, chunks.length - 1));
  const currentChunkText = poutyMessage || chunks[currentChunk] || "";

  // Reset typewriter emergence when message ID changes
  useEffect(() => {
    if (!latestMessage) {
      setDisplayedChars(0);
      setChunkIndex(0);
      lastMsgIdRef.current = null;
      return;
    }

    if (latestMessage.id !== lastMsgIdRef.current) {
      lastMsgIdRef.current = latestMessage.id;
      setChunkIndex(0);
      setDisplayedChars(skipTypewriter ? currentChunkText.length : 0);
    }
  }, [latestMessage?.id, skipTypewriter, currentChunkText.length]);

  // Sequential character-by-character emergence (typewriter cadence)
  useEffect(() => {
    if (skipTypewriter || !currentChunkText) {
      setDisplayedChars(skipTypewriter && currentChunkText ? currentChunkText.length : 0);
      return;
    }

    if (displayedChars >= currentChunkText.length) {
      return;
    }

    const timer = setTimeout(() => {
      setDisplayedChars((prev) => Math.min(prev + 2, currentChunkText.length));
    }, 16);

    return () => clearTimeout(timer);
  }, [displayedChars, currentChunkText, skipTypewriter]);

  // Auto-scroll to bottom in extended mode
  useEffect(() => {
    if (isExtendedMode && scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [isExtendedMode, allMessages.length, displayedChars]);

  const isStreaming = !skipTypewriter && displayedChars < currentChunkText.length;
  const visibleText = skipTypewriter ? currentChunkText : currentChunkText.slice(0, displayedChars);

  // Extract files from attachments + text mentions
  const filePills = useMemo(() => {
    const list: Array<{ name: string; path?: string; size?: number }> = [];

    if (latestMessage?.attachments && latestMessage.attachments.length > 0) {
      latestMessage.attachments.forEach((att) => {
        list.push({
          name: att.name,
          path: att.path || att.name,
          size: att.size
        });
      });
    }

    if (latestMessage?.text) {
      const extracted = extractMentionedFiles(latestMessage.text);
      extracted.forEach((f) => {
        if (!list.some((item) => item.path === f || item.name === f)) {
          const name = f.split(/[/\\]/).pop() || f;
          list.push({ name, path: f });
        }
      });
    }

    return list;
  }, [latestMessage?.attachments, latestMessage?.text]);

  // Handle advancing to next chunk in Otome dialogue mode
  const handleAdvanceChunk = () => {
    if (isStreaming) {
      setDisplayedChars(currentChunkText.length);
      return;
    }

    if (chunkIndex < chunks.length - 1) {
      setChunkIndex((prev) => prev + 1);
      setDisplayedChars(0);
    }
  };

  // Handle going back to previous chunk (triggers playful pouty reaction)
  const handlePreviousChunk = (e: React.MouseEvent) => {
    e.stopPropagation();
    if (chunkIndex > 0) {
      onPoutyReaction?.();
      setChunkIndex((prev) => prev - 1);
      setDisplayedChars(0);
    }
  };

  // ==========================================
  // EXTENDED MODE: Full Scrollable History
  // ==========================================
  if (isExtendedMode) {
    return (
      <div
        ref={scrollRef}
        className={`hime-dialogue-stream hime-dialogue-extended ${className}`}
        style={{
          flex: 1,
          minHeight: "180px",
          maxHeight: "220px",
          height: "210px",
          background: "transparent",
          border: "none",
          padding: "4px 8px",
          display: "flex",
          flexDirection: "column",
          gap: "8px",
          boxSizing: "border-box",
          overflowY: "auto", // SCROLLABLE in Extended Mode!
          overflowX: "hidden",
          scrollbarWidth: "none",
          msOverflowStyle: "none",
          position: "relative",
          fontFamily: "'Consolas', 'Cascadia Code', 'JetBrains Mono', monospace",
          color: "#F4F6FA",
          userSelect: "text",
          ...style
        }}
      >
        <div
          style={{
            display: "inline-flex",
            alignItems: "center",
            gap: "5px",
            alignSelf: "flex-start",
            fontSize: "10px",
            color: "#38D6F5",
            background: "rgba(56, 214, 245, 0.12)",
            border: "1px solid rgba(56, 214, 245, 0.28)",
            borderRadius: "9999px",
            padding: "2px 8px",
            marginBottom: "4px"
          }}
        >
          <Sparkles size={10} color="#38D6F5" />
          <span>Mode Riwayat Lengkap (/chat)</span>
        </div>

        {allMessages.map((msg, idx) => (
          <div
            key={msg.id || idx}
            style={{
              display: "flex",
              flexDirection: "column",
              gap: "2px",
              padding: "4px 6px",
              borderRadius: "8px",
              background: msg.role === "user" ? "rgba(56, 214, 245, 0.06)" : "transparent",
              border: msg.role === "user" ? "1px solid rgba(56, 214, 245, 0.12)" : "none"
            }}
          >
            <div
              style={{
                fontSize: "10.5px",
                fontWeight: 600,
                color: msg.role === "user" ? "#38D6F5" : "#34E3A6"
              }}
            >
              {msg.role === "user" ? "&gt; Kamu" : "Hime"}
            </div>
            <div
              style={{
                fontSize: "12px",
                lineHeight: 1.5,
                color: "#F4F6FA",
                whiteSpace: "pre-wrap",
                wordBreak: "break-word"
              }}
            >
              <ReactMarkdown
                remarkPlugins={[remarkGfm]}
                components={DIALOGUE_MARKDOWN_COMPONENTS}
              >
                {sanitizeDialogueText(msg.text)}
              </ReactMarkdown>
            </div>
          </div>
        ))}
      </div>
    );
  }

  // ==========================================
  // DEFAULT MODE: Otome Game Dialogue Subtitle
  // ==========================================
  return (
    <div
      className={`hime-dialogue-stream ${className}`}
      onClick={handleAdvanceChunk}
      style={{
        flex: 1,
        minHeight: "180px",
        maxHeight: "220px",
        height: "210px",
        background: "transparent", // UNBOXED: Zero speech bubble container!
        backdropFilter: "none",
        border: "none",            // UNBOXED: Zero speech bubble border!
        borderRadius: "0",
        padding: "4px 8px",
        display: "flex",
        flexDirection: "column",
        justifyContent: "space-between",
        boxSizing: "border-box",
        overflow: "hidden", // STRICT: Not scrollable in Default Otome Mode!
        position: "relative",
        fontFamily: "'Consolas', 'Cascadia Code', 'JetBrains Mono', monospace",
        color: "#F4F6FA",
        boxShadow: "none",
        cursor: isStreaming || chunkIndex < chunks.length - 1 ? "pointer" : "default",
        userSelect: "text",
        ...style
      }}
    >
      {/* 1. ACTIVE PROCESS STATE: In-Flight Visual Novel Dialogue */}
      {activeRun && (activeRun.status === "running" || activeRun.status === "queued") ? (
        <div
          style={{
            display: "flex",
            flexDirection: "column",
            height: "100%",
            justifyContent: "space-between"
          }}
        >
          <div>
            <div
              style={{
                fontSize: "12.5px",
                lineHeight: 1.65,
                color: "#F4F6FA",
                textShadow: "0 1px 3px rgba(0, 0, 0, 0.8)",
                marginBottom: "6px"
              }}
            >
              <span style={{ color: "#34E3A6", fontWeight: 600 }}>Sedang memproses instruksimu</span>
              {activeRun.prompt ? (
                <span style={{ color: "#A7AEBB" }}>: "{activeRun.prompt}"</span>
              ) : null}
              <motion.span
                animate={{ opacity: [1, 0, 1] }}
                transition={{ repeat: Infinity, duration: 0.6, ease: "linear" }}
                style={{
                  display: "inline-block",
                  width: "7px",
                  height: "14px",
                  backgroundColor: "#34E3A6",
                  marginLeft: "4px",
                  verticalAlign: "-1px"
                }}
              />
            </div>

            <div
              style={{
                fontSize: "10.5px",
                color: "#38D6F5",
                display: "inline-flex",
                alignItems: "center",
                gap: "5px",
                background: "rgba(56, 214, 245, 0.1)",
                border: "1px solid rgba(56, 214, 245, 0.25)",
                borderRadius: "9999px",
                padding: "2px 8px"
              }}
            >
              <Sparkles size={10} color="#38D6F5" />
              <span>{activeRun.statusText || (activeRun.status === "queued" ? "Antre" : "Berpikir...")}</span>
            </div>
          </div>

          {/* Minimal Otome / Game Progress Bar & Cancel Pill */}
          <div
            style={{
              display: "flex",
              alignItems: "center",
              gap: "8px",
              paddingTop: "4px"
            }}
          >
            <div
              style={{
                flex: 1,
                height: "3px",
                background: "rgba(255, 255, 255, 0.08)",
                borderRadius: "9999px",
                overflow: "hidden"
              }}
            >
              <div
                style={{
                  width: `${activeRun.progress || 18}%`,
                  height: "100%",
                  background: "linear-gradient(90deg, #34E3A6, #38D6F5)",
                  borderRadius: "9999px",
                  transition: "width 0.3s ease"
                }}
              />
            </div>
            <span
              style={{
                color: "#34E3A6",
                fontSize: "10px",
                fontWeight: 600,
                minWidth: "28px",
                textAlign: "right"
              }}
            >
              {activeRun.progress || 0}%
            </span>
            {onCancelRun && (
              <button
                type="button"
                onClick={(e) => {
                  e.stopPropagation();
                  onCancelRun(activeRun.id);
                }}
                style={{
                  background: "rgba(255, 77, 94, 0.14)",
                  border: "1px solid rgba(255, 77, 94, 0.3)",
                  color: "#FF4D5E",
                  borderRadius: "9999px",
                  padding: "2px 8px",
                  fontSize: "10px",
                  cursor: "pointer",
                  fontFamily: "inherit",
                  transition: "background 0.15s ease"
                }}
              >
                Batal
              </button>
            )}
          </div>
        </div>
      ) : latestMessage || poutyMessage ? (
        /* 2. COMPLETED MONOLOGUE / DIRECT UNBOXED SUBTITLE STREAM */
        <div
          style={{
            display: "flex",
            flexDirection: "column",
            height: "100%",
            justifyContent: "space-between"
          }}
        >
          <div
            style={{
              flex: 1,
              overflow: "hidden",
              fontSize: "12.5px",
              lineHeight: 1.65,
              whiteSpace: "pre-wrap",
              wordBreak: "break-word",
              color: "#F4F6FA",
              letterSpacing: "0.2px",
              textShadow: "0 1px 3px rgba(0, 0, 0, 0.85)"
            }}
          >
            <div style={{ display: "inline" }}>
              <ReactMarkdown
                remarkPlugins={[remarkGfm]}
                components={DIALOGUE_MARKDOWN_COMPONENTS}
              >
                {visibleText}
              </ReactMarkdown>
            </div>
            {isStreaming ? (
              <motion.span
                animate={{ opacity: [1, 0, 1] }}
                transition={{ repeat: Infinity, duration: 0.6, ease: "linear" }}
                style={{
                  display: "inline-block",
                  width: "7px",
                  height: "14px",
                  backgroundColor: "#34E3A6",
                  marginLeft: "4px",
                  verticalAlign: "-1px"
                }}
              />
            ) : chunkIndex < chunks.length - 1 ? (
              /* Visual Novel In-Place Chunk Continuation Indicator */
              <motion.span
                animate={{ opacity: [0.4, 1, 0.4], y: [0, 2, 0] }}
                transition={{ repeat: Infinity, duration: 1.2, ease: "easeInOut" }}
                style={{
                  display: "inline-block",
                  marginLeft: "6px",
                  color: "#34E3A6",
                  fontSize: "11px",
                  fontWeight: 600,
                  cursor: "pointer"
                }}
                title="Klik untuk lanjut"
              >
                ▼
              </motion.span>
            ) : null}

            {/* Interactive Rounded Pill Widgets for Files */}
            {filePills.length > 0 && !isStreaming && (
              <div
                style={{
                  display: "flex",
                  flexWrap: "wrap",
                  gap: "6px",
                  marginTop: "8px"
                }}
              >
                {filePills.map((file, idx) => (
                  <motion.button
                    key={`${file.name}-${idx}`}
                    type="button"
                    whileHover={{ scale: 1.04 }}
                    whileTap={{ scale: 0.96 }}
                    onClick={(e) => {
                      e.stopPropagation();
                      openFileFromPill(file.path || file.name);
                    }}
                    title={`Klik untuk membuka berkas: ${file.path || file.name}`}
                    style={{
                      display: "inline-flex",
                      alignItems: "center",
                      gap: "5px",
                      background: "rgba(255, 255, 255, 0.08)",
                      backdropFilter: "blur(8px)",
                      border: "1px solid rgba(255, 255, 255, 0.16)",
                      borderRadius: "9999px",
                      padding: "2px 8px",
                      fontSize: "10.5px",
                      fontFamily: "'Consolas', monospace",
                      color: "#F4F6FA",
                      cursor: "pointer",
                      transition: "background 0.15s ease, border-color 0.15s ease",
                      boxShadow: "0 2px 6px rgba(0, 0, 0, 0.25)"
                    }}
                    onMouseEnter={(e) => {
                      e.currentTarget.style.background = "rgba(52, 227, 166, 0.16)";
                      e.currentTarget.style.borderColor = "rgba(52, 227, 166, 0.4)";
                    }}
                    onMouseLeave={(e) => {
                      e.currentTarget.style.background = "rgba(255, 255, 255, 0.08)";
                      e.currentTarget.style.borderColor = "rgba(255, 255, 255, 0.16)";
                    }}
                  >
                    <FileText size={11} color="#34E3A6" />
                    <span
                      style={{
                        maxWidth: "140px",
                        overflow: "hidden",
                        textOverflow: "ellipsis",
                        whiteSpace: "nowrap"
                      }}
                    >
                      {file.name}
                    </span>
                    {file.size ? (
                      <span style={{ fontSize: "9px", color: "#8E96A4" }}>
                        ({formatFileSize(file.size)})
                      </span>
                    ) : null}
                  </motion.button>
                ))}
              </div>
            )}
          </div>

          {/* Subtitle bottom controls: Previous chunk affordance if chunkIndex > 0 */}
          {chunkIndex > 0 && !isStreaming && (
            <div
              style={{
                display: "flex",
                justifyContent: "flex-end",
                paddingTop: "2px"
              }}
            >
              <button
                type="button"
                onClick={handlePreviousChunk}
                style={{
                  background: "transparent",
                  border: "none",
                  color: "#8E96A4",
                  fontSize: "9.5px",
                  cursor: "pointer",
                  fontFamily: "inherit",
                  display: "flex",
                  alignItems: "center",
                  gap: "3px",
                  transition: "color 0.15s ease"
                }}
                onMouseEnter={(e) => (e.currentTarget.style.color = "#FF4D5E")}
                onMouseLeave={(e) => (e.currentTarget.style.color = "#8E96A4")}
                title="Minta Hime ulangi bagian sebelumnya"
              >
                <span>◄ Ulangi tadi?</span>
              </button>
            </div>
          )}
        </div>
      ) : (
        /* 3. IDLE READY MONOLOGUE: Visual Novel Direct Greeting & Quick Action Pills */
        <div
          style={{
            display: "flex",
            flexDirection: "column",
            height: "100%",
            justifyContent: "space-between"
          }}
        >
          <div>
            <div
              style={{
                color: "#F4F6FA",
                fontSize: "12.5px",
                lineHeight: 1.65,
                textShadow: "0 1px 3px rgba(0, 0, 0, 0.85)"
              }}
            >
              Halo! Aku Hime, siap membantumu. Tanyakan apa saja atau pilih instruksi cepat di bawah ini.
              <motion.span
                animate={{ opacity: [1, 0, 1] }}
                transition={{ repeat: Infinity, duration: 0.9, ease: "linear" }}
                style={{
                  display: "inline-block",
                  width: "7px",
                  height: "14px",
                  backgroundColor: "#34E3A6",
                  marginLeft: "4px",
                  verticalAlign: "-1px"
                }}
              />
            </div>
          </div>

          {/* Quick Action Pills with Rounded Pill Geometry */}
          <div>
            <div
              style={{
                fontSize: "10px",
                color: "#8E96A4",
                marginBottom: "6px",
                letterSpacing: "0.2px"
              }}
            >
              Pilih instruksi cepat:
            </div>
            <div
              style={{
                display: "flex",
                flexWrap: "wrap",
                gap: "6px"
              }}
            >
              {[
                { slash: "/dev", label: "Kode", desc: "Tolong bantu tinjau kode proyek ini" },
                { slash: "/doc", label: "Dokumen", desc: "Buat ringkasan dokumen akademik" },
                { slash: "/data", label: "Data", desc: "Analisis performa sistem Hime" },
                { slash: "/tasks", label: "Tugas", desc: "Tampilkan ringkasan tugas hari ini" }
              ].map((item) => (
                <button
                  key={item.slash}
                  type="button"
                  onClick={(e) => {
                    e.stopPropagation();
                    onSelectSkill?.(item.desc, [item.slash.slice(1)]);
                  }}
                  style={{
                    background: "rgba(255, 255, 255, 0.06)",
                    border: "1px solid rgba(255, 255, 255, 0.12)",
                    borderRadius: "9999px",
                    padding: "3px 10px",
                    fontSize: "10.5px",
                    color: "#F4F6FA",
                    cursor: "pointer",
                    display: "flex",
                    alignItems: "center",
                    gap: "5px",
                    fontFamily: "inherit",
                    transition: "background 0.15s ease, border-color 0.15s ease"
                  }}
                  onMouseEnter={(e) => {
                    e.currentTarget.style.background = "rgba(52, 227, 166, 0.16)";
                    e.currentTarget.style.borderColor = "rgba(52, 227, 166, 0.35)";
                  }}
                  onMouseLeave={(e) => {
                    e.currentTarget.style.background = "rgba(255, 255, 255, 0.06)";
                    e.currentTarget.style.borderColor = "rgba(255, 255, 255, 0.12)";
                  }}
                >
                  <span style={{ color: "#34E3A6", fontWeight: 600 }}>{item.slash}</span>
                  <span style={{ color: "#A7AEBB", fontSize: "10px" }}>{item.label}</span>
                </button>
              ))}
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
