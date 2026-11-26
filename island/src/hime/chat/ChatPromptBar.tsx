import React, { useState, useRef, useEffect, useMemo } from "react";
import { motion, AnimatePresence } from "framer-motion";
import {
  ArrowUp,
  Paperclip,
  Mic,
  MicOff,
  X,
  Code,
  FileText,
  BarChart3,
  Archive,
  CheckSquare,
  Clock,
  Languages,
  Smartphone,
  Cpu,
  StopCircle,
  PauseCircle,
  Search,
  Trash2
} from "lucide-react";
import { SkillManifest, AttachmentRef } from "./types";
import { SKILL_CATALOG, FAST_COMMANDS, formatFileSize } from "./skillsCatalog";
import { SkillPill } from "../ui/SkillPill";

export interface ChatPromptBarProps {
  onSendMessage: (
    text: string,
    skills: string[],
    attachments: AttachmentRef[]
  ) => void;
  attachments?: AttachmentRef[];
  onRemoveAttachment?: (id: string) => void;
  onOpenFilePicker?: () => void;
  isFileDialogOpen?: boolean;
  isVoiceRecording?: boolean;
  voiceAudioLevel?: number;
  onToggleVoice?: () => void;
  placeholder?: string;
  onTextChange?: (text: string) => void;
}

// Icon component lookup map
const ICON_MAP: Record<string, React.FC<any>> = {
  Code,
  FileText,
  BarChart3,
  Archive,
  CheckSquare,
  Clock,
  Languages,
  Smartphone,
  Cpu,
  StopCircle,
  PauseCircle,
  Search,
  Trash2
};

export const ChatPromptBar: React.FC<ChatPromptBarProps> = ({
  onSendMessage,
  attachments = [],
  onRemoveAttachment,
  onOpenFilePicker,
  isFileDialogOpen,
  isVoiceRecording = false,
  voiceAudioLevel = 0,
  onToggleVoice,
  placeholder = "Tanya Hime...",
  onTextChange
}) => {
  const [text, setText] = useState("");
  const [internalFileDialogOpen, setInternalFileDialogOpen] = useState(false);
  const isFileDialogActive = isFileDialogOpen ?? internalFileDialogOpen;
  const [selectedSkill, setSelectedSkill] = useState<SkillManifest | null>(null);
  const [showSlashMenu, setShowSlashMenu] = useState(false);
  const [slashQuery, setSlashQuery] = useState("");
  const [selectedIndex, setSelectedIndex] = useState(0);
  const [isExpanded, setIsExpanded] = useState(false);

  const textareaRef = useRef<HTMLTextAreaElement | null>(null);

  // Auto-expand textarea: slim single-line (~20px height) by default; only expand up to 2 lines (~40px) when text wraps
  useEffect(() => {
    if (textareaRef.current) {
      textareaRef.current.style.height = "20px";
      const scrollH = textareaRef.current.scrollHeight;
      if (scrollH > 24) {
        setIsExpanded(true);
        // Expand up to 2 lines (~40px max)
        textareaRef.current.style.height = `${Math.min(40, scrollH)}px`;
      } else {
        setIsExpanded(false);
        textareaRef.current.style.height = "20px";
      }
    }
  }, [text]);

  // Combined searchable catalog (skills + fast commands)
  const combinedCatalog = useMemo(() => {
    const list: Array<
      | { type: "skill"; item: SkillManifest }
      | { type: "command"; item: (typeof FAST_COMMANDS)[0] }
    > = [];

    SKILL_CATALOG.forEach((s) => list.push({ type: "skill", item: s }));
    FAST_COMMANDS.forEach((c) => list.push({ type: "command", item: c }));

    if (!slashQuery) return list;
    const q = slashQuery.toLowerCase();
    return list.filter((entry) => {
      if (entry.type === "skill") {
        return (
          entry.item.slash.toLowerCase().includes(q) ||
          entry.item.name.toLowerCase().includes(q) ||
          entry.item.desc.toLowerCase().includes(q) ||
          entry.item.aliases?.some((a) => a.toLowerCase().includes(q))
        );
      } else {
        return (
          entry.item.command.toLowerCase().includes(q) ||
          entry.item.name.toLowerCase().includes(q) ||
          entry.item.desc.toLowerCase().includes(q)
        );
      }
    });
  }, [slashQuery]);

  const selectSkill = (skill: SkillManifest) => {
    setSelectedSkill(skill);
    setShowSlashMenu(false);
    setText("");
    onTextChange?.("");
    textareaRef.current?.focus();
  };

  const selectCommand = (commandSlash: string) => {
    setShowSlashMenu(false);
    setText(commandSlash);
    onTextChange?.(commandSlash);
    textareaRef.current?.focus();
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (showSlashMenu) {
      if (e.key === "Escape") {
        e.preventDefault();
        setShowSlashMenu(false);
        return;
      }

      if (combinedCatalog.length > 0) {
        if (e.key === "ArrowDown") {
          e.preventDefault();
          setSelectedIndex((prev) => (prev + 1) % combinedCatalog.length);
          return;
        }
        if (e.key === "ArrowUp") {
          e.preventDefault();
          setSelectedIndex((prev) => (prev - 1 + combinedCatalog.length) % combinedCatalog.length);
          return;
        }
        if (e.key === "Enter" || e.key === "Tab") {
          e.preventDefault();
          const selected = combinedCatalog[selectedIndex];
          if (selected) {
            if (selected.type === "skill") {
              selectSkill(selected.item);
            } else {
              selectCommand(selected.item.command);
            }
          }
          return;
        }
      }
    }

    // Backspace on empty input removes active skill pill
    if (e.key === "Backspace" && text === "" && selectedSkill) {
      setSelectedSkill(null);
      return;
    }

    // Enter without Shift sends message
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSubmit();
    }
  };

  const handleChange = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
    const val = e.target.value;
    setText(val);
    onTextChange?.(val);

    // Detect slash trigger (dismiss if text contains space e.g. after typing command arguments)
    if (val.startsWith("/") && !val.includes(" ")) {
      setShowSlashMenu(true);
      setSlashQuery(val.slice(1));
      setSelectedIndex(0);
    } else {
      setShowSlashMenu(false);
    }
  };

  const handleSubmit = () => {
    const trimmed = text.trim();
    if (!trimmed && attachments.length === 0) return;

    const skills = selectedSkill ? [selectedSkill.id] : [];
    const attached = [...attachments];

    // Clear input state immediately
    setText("");
    onTextChange?.("");
    setSelectedSkill(null);
    setShowSlashMenu(false);

    if (textareaRef.current) {
      textareaRef.current.style.height = "24px";
      textareaRef.current.focus();
    }

    onSendMessage(trimmed, skills, attached);
  };

  const hasContent = text.trim().length > 0 || attachments.length > 0;

  return (
    <div
      style={{
        position: "relative",
        width: "100%",
        display: "flex",
        flexDirection: "column",
        gap: "4px",
        boxSizing: "border-box"
      }}
    >
      {/* Slash Autocomplete Popover (Origin-aware, keyboard-navigable) */}
      <AnimatePresence>
        {showSlashMenu && combinedCatalog.length > 0 && (
          <motion.div
            initial={{ opacity: 0, y: 8, scale: 0.96 }}
            animate={{ opacity: 1, y: 0, scale: 1 }}
            exit={{ opacity: 0, y: 6, scale: 0.96 }}
            transition={{ duration: 0.16, ease: [0.16, 1, 0.3, 1] }}
            style={{
              position: "absolute",
              bottom: "calc(100% + 8px)",
              left: 0,
              right: 0,
              maxHeight: "220px",
              overflowY: "auto",
              overflowX: "hidden",
              scrollbarWidth: "none",
              msOverflowStyle: "none",
              background: "rgba(16, 18, 24, 0.96)",
              backdropFilter: "blur(18px)",
              border: "1px solid rgba(255, 255, 255, 0.12)",
              borderRadius: "16px",
              padding: "6px",
              zIndex: 50,
              boxShadow: "0 14px 36px rgba(0, 0, 0, 0.65)"
            }}
          >
            {combinedCatalog.map((entry, idx) => {
              const isSelected = idx === selectedIndex;
              const isSkill = entry.type === "skill";
              const IconComp = isSkill
                ? ICON_MAP[entry.item.iconName] || Code
                : entry.item.command === "/stop"
                ? StopCircle
                : entry.item.command === "/jeda"
                ? PauseCircle
                : entry.item.command === "/cari"
                ? Search
                : Trash2;

              return (
                <div
                  key={isSkill ? entry.item.id : entry.item.command}
                  onClick={() => {
                    if (isSkill) selectSkill(entry.item);
                    else selectCommand(entry.item.command);
                  }}
                  onMouseEnter={() => setSelectedIndex(idx)}
                  style={{
                    display: "flex",
                    alignItems: "center",
                    gap: "8px",
                    padding: "6px 10px",
                    borderRadius: "10px",
                    background: isSelected ? "rgba(255, 255, 255, 0.08)" : "transparent",
                    cursor: "pointer",
                    transition: "background 0.1s ease"
                  }}
                >
                  <div
                    style={{
                      width: "24px",
                      height: "24px",
                      borderRadius: "6px",
                      display: "flex",
                      alignItems: "center",
                      justifyContent: "center",
                      background: isSkill
                        ? `var(--accent-${entry.item.accent}-tint, rgba(52, 227, 166, 0.18))`
                        : "rgba(255, 255, 255, 0.06)",
                      color: isSkill
                        ? `var(--accent-${entry.item.accent}, #34E3A6)`
                        : "#A7AEBB",
                      flexShrink: 0
                    }}
                  >
                    <IconComp size={13} />
                  </div>

                  <div style={{ flex: 1, minWidth: 0 }}>
                    <div style={{ display: "flex", alignItems: "center", gap: "6px" }}>
                      <span
                        style={{
                          fontSize: "11.5px",
                          fontWeight: 600,
                          color: "#F4F6FA"
                        }}
                      >
                        {isSkill ? entry.item.slash : entry.item.command}
                      </span>
                      <span
                        style={{
                          fontSize: "10.5px",
                          color: "#A7AEBB"
                        }}
                      >
                        {entry.item.name}
                      </span>
                    </div>
                    <div
                      style={{
                        fontSize: "10px",
                        color: "#6F7785",
                        overflow: "hidden",
                        textOverflow: "ellipsis",
                        whiteSpace: "nowrap"
                      }}
                    >
                      {entry.item.desc}
                    </div>
                  </div>
                </div>
              );
            })}
          </motion.div>
        )}
      </AnimatePresence>

      {/* Attachment chips row if attachments are staged (clean rounded pill geometry) */}
      {attachments.length > 0 && (
        <div
          style={{
            display: "flex",
            flexWrap: "wrap",
            gap: "6px",
            padding: "4px 8px",
            background: "rgba(22, 24, 29, 0.8)",
            backdropFilter: "blur(12px)",
            borderRadius: "14px",
            border: "1px solid rgba(255, 255, 255, 0.08)"
          }}
        >
          {attachments.map((att) => (
            <div
              key={att.id}
              style={{
                display: "inline-flex",
                alignItems: "center",
                gap: "5px",
                padding: "2px 8px",
                borderRadius: "9999px",
                background: "rgba(255, 255, 255, 0.08)",
                border: "1px solid rgba(255, 255, 255, 0.12)",
                fontSize: "10px",
                color: "#F4F6FA"
              }}
            >
              <Paperclip size={10} color={`var(--accent-${att.accent})`} />
              <span
                style={{
                  maxWidth: "110px",
                  overflow: "hidden",
                  textOverflow: "ellipsis",
                  whiteSpace: "nowrap"
                }}
              >
                {att.name}
              </span>
              <span style={{ fontSize: "8.5px", color: "#8E96A4" }}>
                ({formatFileSize(att.size)})
              </span>
              {onRemoveAttachment && (
                <button
                  type="button"
                  onClick={() => onRemoveAttachment(att.id)}
                  title="Hapus"
                  style={{
                    background: "none",
                    border: "none",
                    padding: 0,
                    cursor: "pointer",
                    color: "#A7AEBB",
                    display: "flex",
                    alignItems: "center",
                    marginLeft: "2px"
                  }}
                >
                  <X size={10} />
                </button>
              )}
            </div>
          ))}
        </div>
      )}

      {/* Slim Single-Line (~32-34px height) ChatGPT-style Rounded Pill Prompt Input Container */}
      <div
        style={{
          display: "flex",
          alignItems: "center",
          gap: "5px",
          background: "rgba(24, 26, 32, 0.92)",
          backdropFilter: "blur(16px)",
          border: isVoiceRecording
            ? "1px solid #34E3A6"
            : "1px solid rgba(255, 255, 255, 0.12)",
          borderRadius: "9999px", // Seamless pill geometry
          padding: "2px 6px 2px 10px",
          minHeight: "32px",
          height: isExpanded ? "auto" : "32px",
          maxHeight: "58px", // Max ~58px when wrapped to 2 lines
          boxShadow: isVoiceRecording
            ? "0 0 18px rgba(52, 227, 166, 0.25)"
            : "0 4px 16px rgba(0, 0, 0, 0.35)",
          transition: "border 0.2s ease, box-shadow 0.2s ease",
          boxSizing: "border-box"
        }}
      >
        {/* Attachment Pill Button (Left) */}
        {onOpenFilePicker && (
          <button
            type="button"
            onClick={(e) => {
              e.stopPropagation();
              setInternalFileDialogOpen(true);
              if (typeof window !== "undefined") {
                (window as any).isFileDialogOpen = true;
                (window as any).__isFileDialogOpen = true;
                (window as any).__himeFileDialogOpen = true;
                window.addEventListener(
                  "focus",
                  () => {
                    setTimeout(() => {
                      setInternalFileDialogOpen(false);
                      (window as any).isFileDialogOpen = false;
                      (window as any).__isFileDialogOpen = false;
                      (window as any).__himeFileDialogOpen = false;
                    }, 500);
                  },
                  { once: true }
                );
              }
              onOpenFilePicker();
            }}
            title="Lampirkan berkas"
            style={{
              background: isFileDialogActive ? "rgba(52, 227, 166, 0.12)" : "transparent",
              border: "none",
              borderRadius: "50%",
              width: "26px",
              height: "26px",
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              cursor: "pointer",
              color: isFileDialogActive ? "#34E3A6" : "#A7AEBB",
              transition: "background 0.15s ease, color 0.15s ease, transform 0.1s ease",
              flexShrink: 0
            }}
            onMouseEnter={(e) => {
              e.currentTarget.style.background = "rgba(255, 255, 255, 0.08)";
              e.currentTarget.style.color = "#F4F6FA";
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.background = "transparent";
              e.currentTarget.style.color = "#A7AEBB";
            }}
            onMouseDown={(e) => (e.currentTarget.style.transform = "scale(0.92)")}
            onMouseUp={(e) => (e.currentTarget.style.transform = "scale(1)")}
          >
            <Paperclip size={14} />
          </button>
        )}

        {/* Active Solid Category Skill Pill */}
        {selectedSkill && (
          <div style={{ flexShrink: 0 }}>
            <SkillPill
              id={selectedSkill.id}
              name={selectedSkill.id}
              accent={selectedSkill.accent}
              onRemove={() => setSelectedSkill(null)}
            />
          </div>
        )}

        {/* Compact Auto-expanding Textarea (~20px single line, vertically centered, up to 40px for 2 lines) */}
        <textarea
          ref={textareaRef}
          value={text}
          onChange={handleChange}
          onKeyDown={handleKeyDown}
          placeholder={
            selectedSkill
              ? `Tanya ${selectedSkill.name}...`
              : placeholder
          }
          rows={1}
          style={{
            flex: 1,
            background: "transparent",
            border: "none",
            outline: "none",
            resize: "none",
            fontSize: "12px",
            lineHeight: "20px",
            color: "#F4F6FA",
            padding: "0 4px",
            height: "20px",
            maxHeight: "40px",
            fontFamily: "'Consolas', 'Cascadia Code', 'JetBrains Mono', -apple-system, sans-serif",
            boxSizing: "border-box",
            overflowY: isExpanded ? "auto" : "hidden",
            display: "block"
          }}
        />

        {/* Voice Push-to-Talk Mic Pill Button (Right) */}
        {onToggleVoice && (
          <button
            type="button"
            onClick={onToggleVoice}
            title={isVoiceRecording ? "Hentikan rekaman" : "Bicara dengan Hime"}
            style={{
              background: isVoiceRecording
                ? "rgba(52, 227, 166, 0.22)"
                : "transparent",
              border: isVoiceRecording
                ? "1px solid #34E3A6"
                : "none",
              borderRadius: "50%",
              width: "26px",
              height: "26px",
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              cursor: "pointer",
              color: isVoiceRecording ? "#34E3A6" : "#A7AEBB",
              transition: "background 0.15s ease, color 0.15s ease, transform 0.12s ease",
              transform: isVoiceRecording
                ? `scale(${1 + voiceAudioLevel * 0.2})`
                : "scale(1)",
              flexShrink: 0
            }}
            onMouseEnter={(e) => {
              if (!isVoiceRecording) {
                e.currentTarget.style.background = "rgba(255, 255, 255, 0.08)";
                e.currentTarget.style.color = "#F4F6FA";
              }
            }}
            onMouseLeave={(e) => {
              if (!isVoiceRecording) {
                e.currentTarget.style.background = "transparent";
                e.currentTarget.style.color = "#A7AEBB";
              }
            }}
            onMouseDown={(e) => (e.currentTarget.style.transform = "scale(0.92)")}
            onMouseUp={(e) => (e.currentTarget.style.transform = "scale(1)")}
          >
            {isVoiceRecording ? <Mic size={13} /> : <MicOff size={13} />}
          </button>
        )}

        {/* Modern High-contrast Send Pill Button (Right) */}
        <button
          type="button"
          onClick={handleSubmit}
          disabled={!hasContent}
          title="Kirim instruksi"
          style={{
            background: hasContent
              ? "#FFFFFF"
              : "rgba(255, 255, 255, 0.08)",
            border: "none",
            borderRadius: "50%",
            width: "26px",
            height: "26px",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            cursor: hasContent ? "pointer" : "default",
            color: hasContent ? "#0A0B0E" : "rgba(255, 255, 255, 0.25)",
            boxShadow: hasContent
              ? "0 2px 8px rgba(255, 255, 255, 0.25)"
              : "none",
            transition: "transform 0.12s ease, background 0.15s ease, color 0.15s ease, box-shadow 0.15s ease",
            flexShrink: 0
          }}
          onMouseDown={(e) => {
            if (hasContent) {
              e.currentTarget.style.transform = "scale(0.92)";
            }
          }}
          onMouseUp={(e) => (e.currentTarget.style.transform = "scale(1)")}
        >
          <ArrowUp size={14} strokeWidth={2.4} />
        </button>
      </div>
    </div>
  );
};
