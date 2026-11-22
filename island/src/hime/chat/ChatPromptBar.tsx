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
  isVoiceRecording?: boolean;
  voiceAudioLevel?: number;
  onToggleVoice?: () => void;
  placeholder?: string;
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
  isVoiceRecording = false,
  voiceAudioLevel = 0,
  onToggleVoice,
  placeholder = "Tanya Hime... (ketik / untuk skill)"
}) => {
  const [text, setText] = useState("");
  const [selectedSkill, setSelectedSkill] = useState<SkillManifest | null>(null);
  const [showSlashMenu, setShowSlashMenu] = useState(false);
  const [slashQuery, setSlashQuery] = useState("");
  const [selectedIndex, setSelectedIndex] = useState(0);

  const textareaRef = useRef<HTMLTextAreaElement | null>(null);

  // Auto-expand textarea height (1 to 6 lines, max ~120px)
  useEffect(() => {
    if (textareaRef.current) {
      textareaRef.current.style.height = "auto";
      const scrollH = textareaRef.current.scrollHeight;
      textareaRef.current.style.height = `${Math.min(120, Math.max(34, scrollH))}px`;
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
    textareaRef.current?.focus();
  };

  const selectCommand = (commandSlash: string) => {
    setShowSlashMenu(false);
    setText(commandSlash);
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

    // Clear input state immediately <= 1s
    setText("");
    setSelectedSkill(null);
    setShowSlashMenu(false);

    if (textareaRef.current) {
      textareaRef.current.style.height = "34px";
      textareaRef.current.focus();
    }

    onSendMessage(trimmed, skills, attached);
  };

  return (
    <div
      style={{
        position: "relative",
        width: "100%",
        display: "flex",
        flexDirection: "column",
        gap: "6px",
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
              background: "rgba(13, 14, 18, 0.98)",
              backdropFilter: "blur(16px)",
              border: "1px solid rgba(255, 255, 255, 0.12)",
              borderRadius: "14px",
              padding: "6px",
              zIndex: 50,
              boxShadow: "0 12px 32px rgba(0, 0, 0, 0.6)"
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
                    borderRadius: "8px",
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

      {/* Attachment chips row if attachments are staged */}
      {attachments.length > 0 && (
        <div
          style={{
            display: "flex",
            flexWrap: "wrap",
            gap: "6px",
            padding: "4px 8px",
            background: "rgba(20, 22, 28, 0.7)",
            borderRadius: "10px",
            border: "1px solid rgba(255, 255, 255, 0.06)"
          }}
        >
          {attachments.map((att) => (
            <div
              key={att.id}
              style={{
                display: "inline-flex",
                alignItems: "center",
                gap: "4px",
                padding: "2px 8px",
                borderRadius: "6px",
                background: "rgba(255, 255, 255, 0.06)",
                border: "1px solid rgba(255, 255, 255, 0.1)",
                fontSize: "10.5px",
                color: "#F4F6FA"
              }}
            >
              <Paperclip size={10} color={`var(--accent-${att.accent})`} />
              <span
                style={{
                  maxWidth: "100px",
                  overflow: "hidden",
                  textOverflow: "ellipsis",
                  whiteSpace: "nowrap"
                }}
              >
                {att.name}
              </span>
              <span style={{ fontSize: "9px", color: "#6F7785" }}>
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
                    alignItems: "center"
                  }}
                >
                  <X size={10} />
                </button>
              )}
            </div>
          ))}
        </div>
      )}

      {/* Input container: Solid category pill + Auto-expanding textarea + Action buttons */}
      <div
        style={{
          display: "flex",
          alignItems: "flex-end",
          gap: "6px",
          background: "rgba(20, 22, 28, 0.95)",
          border: isVoiceRecording
            ? "1px solid #34E3A6"
            : "1px solid rgba(255, 255, 255, 0.12)",
          borderRadius: "14px",
          padding: "6px 8px",
          boxShadow: isVoiceRecording
            ? "0 0 16px rgba(52, 227, 166, 0.25)"
            : "none",
          transition: "border 0.2s ease, box-shadow 0.2s ease"
        }}
      >
        {/* Active Solid Category Skill Pill */}
        {selectedSkill && (
          <div style={{ marginBottom: "4px", flexShrink: 0 }}>
            <SkillPill
              id={selectedSkill.id}
              name={selectedSkill.id}
              accent={selectedSkill.accent}
              onRemove={() => setSelectedSkill(null)}
            />
          </div>
        )}

        {/* Auto-expanding prompt textarea — NEVER disabled */}
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
            lineHeight: 1.5,
            color: "#F4F6FA",
            padding: "5px 4px",
            fontFamily: "inherit",
            boxSizing: "border-box"
          }}
        />

        {/* Action icons: File Attachment Button */}
        {onOpenFilePicker && (
          <button
            type="button"
            onClick={onOpenFilePicker}
            title="Lampirkan berkas"
            style={{
              background: "none",
              border: "none",
              borderRadius: "8px",
              width: "28px",
              height: "28px",
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              cursor: "pointer",
              color: "#A7AEBB",
              transition: "transform 140ms ease-out, color 140ms ease-out",
              marginBottom: "2px",
              flexShrink: 0
            }}
            onMouseDown={(e) => (e.currentTarget.style.transform = "scale(0.92)")}
            onMouseUp={(e) => (e.currentTarget.style.transform = "scale(1)")}
          >
            <Paperclip size={15} />
          </button>
        )}

        {/* Voice Push-to-Talk Mic Button */}
        {onToggleVoice && (
          <button
            type="button"
            onClick={onToggleVoice}
            title={isVoiceRecording ? "Hentikan" : "Bicara"}
            style={{
              background: isVoiceRecording
                ? "rgba(52, 227, 166, 0.22)"
                : "none",
              border: isVoiceRecording
                ? "1px solid #34E3A6"
                : "none",
              borderRadius: "8px",
              width: "28px",
              height: "28px",
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              cursor: "pointer",
              color: isVoiceRecording ? "#34E3A6" : "#A7AEBB",
              transition: "transform 140ms ease-out",
              transform: isVoiceRecording
                ? `scale(${1 + voiceAudioLevel * 0.2})`
                : "scale(1)",
              marginBottom: "2px",
              flexShrink: 0
            }}
          >
            {isVoiceRecording ? <Mic size={15} /> : <MicOff size={15} />}
          </button>
        )}

        {/* Send Button */}
        <button
          type="button"
          onClick={handleSubmit}
          disabled={!text.trim() && attachments.length === 0}
          title="Kirim"
          style={{
            background:
              text.trim() || attachments.length > 0
                ? "var(--accent-mint, #34E3A6)"
                : "rgba(255, 255, 255, 0.06)",
            border: "none",
            borderRadius: "8px",
            width: "28px",
            height: "28px",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            cursor: text.trim() || attachments.length > 0 ? "pointer" : "default",
            color: text.trim() || attachments.length > 0 ? "#0A0B0E" : "#6F7785",
            transition: "transform 140ms ease-out, background 140ms ease-out",
            marginBottom: "2px",
            flexShrink: 0
          }}
          onMouseDown={(e) => {
            if (text.trim() || attachments.length > 0) {
              e.currentTarget.style.transform = "scale(0.92)";
            }
          }}
          onMouseUp={(e) => (e.currentTarget.style.transform = "scale(1)")}
        >
          <ArrowUp size={15} />
        </button>
      </div>
    </div>
  );
};
