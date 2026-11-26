import React, { useState } from "react";
import { Search, Trash2, X, MoreHorizontal, Pause, Play, CheckSquare } from "lucide-react";
import { HimeEyes, EyeExpression } from "../ui/HimeEyes";

export interface ChatHeaderProps {
  isOnline?: boolean;
  expression?: EyeExpression;
  isPaused?: boolean;
  onTogglePause?: () => void;
  onOpenSearch?: () => void;
  onOpenTasks?: () => void;
  onClearChat?: () => void;
  onClose?: () => void;
}

/**
 * Chat Header Component
 * Limits:
 * - Title <= 16 chars ("Hime")
 * - Buttons <= 10 chars ("Cari", "Bersih", "Jeda", "Tutup")
 */
export const ChatHeader: React.FC<ChatHeaderProps> = ({
  isOnline = true,
  expression = "netral",
  isPaused = false,
  onTogglePause,
  onOpenSearch,
  onOpenTasks,
  onClearChat,
  onClose
}) => {
  const [showMenu, setShowMenu] = useState(false);

  return (
    <div
      style={{
        display: "flex",
        alignItems: "center",
        justifyContent: "space-between",
        padding: "4px 4px 8px 4px",
        borderBottom: "1px solid rgba(255, 255, 255, 0.06)",
        marginBottom: "8px",
        position: "relative",
        minHeight: "30px"
      }}
    >
      {/* Left: Title & Status */}
      <div style={{ display: "flex", alignItems: "center", gap: "6px" }}>
        <div
          style={{
            width: "6px",
            height: "6px",
            borderRadius: "50%",
            backgroundColor: isOnline ? "#34E3A6" : "#6F7785",
            boxShadow: isOnline ? "0 0 8px rgba(52, 227, 166, 0.5)" : "none"
          }}
          title={isOnline ? "Online" : "Offline"}
        />
        <span
          style={{
            fontSize: "11.5px",
            fontWeight: 600,
            color: "#8E96A4",
            letterSpacing: "0.3px",
            fontFamily: "Consolas, monospace"
          }}
        >
          HIME
        </span>
      </div>

      {/* Center: Hime's Bot Eyes */}
      <div
        style={{
          position: "absolute",
          left: "50%",
          transform: "translateX(-50%)",
          display: "flex",
          alignItems: "center",
          justifyContent: "center"
        }}
      >
        <HimeEyes size={22} expression={expression} />
      </div>

      {/* Right: Actions */}
      <div style={{ display: "flex", alignItems: "center", gap: "4px" }}>
        {onTogglePause && (
          <button
            type="button"
            onClick={onTogglePause}
            title={isPaused ? "Lanjutkan" : "Jeda"}
            style={{
              background: isPaused ? "rgba(255, 194, 71, 0.16)" : "none",
              border: isPaused ? "1px solid rgba(255, 194, 71, 0.3)" : "none",
              borderRadius: "6px",
              width: "24px",
              height: "24px",
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              cursor: "pointer",
              color: isPaused ? "#FFC247" : "#A7AEBB"
            }}
          >
            {isPaused ? <Play size={13} /> : <Pause size={13} />}
          </button>
        )}

        {onOpenSearch && (
          <button
            type="button"
            onClick={onOpenSearch}
            title="Cari"
            style={{
              background: "none",
              border: "none",
              borderRadius: "6px",
              width: "24px",
              height: "24px",
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              cursor: "pointer",
              color: "#A7AEBB"
            }}
          >
            <Search size={13} />
          </button>
        )}

        {/* Quick Menu */}
        <div style={{ position: "relative" }}>
          <button
            type="button"
            onClick={() => setShowMenu((prev) => !prev)}
            title="Menu"
            style={{
              background: showMenu ? "rgba(255, 255, 255, 0.08)" : "none",
              border: "none",
              borderRadius: "6px",
              width: "24px",
              height: "24px",
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              cursor: "pointer",
              color: "#A7AEBB"
            }}
          >
            <MoreHorizontal size={13} />
          </button>

          {showMenu && (
            <div
              style={{
                position: "absolute",
                top: "100%",
                right: 0,
                marginTop: "4px",
                background: "rgba(20, 22, 28, 0.98)",
                border: "1px solid rgba(255, 255, 255, 0.12)",
                borderRadius: "10px",
                padding: "4px",
                minWidth: "120px",
                zIndex: 60,
                boxShadow: "0 8px 24px rgba(0, 0, 0, 0.5)"
              }}
            >
              {onClearChat && (
                <button
                  type="button"
                  onClick={() => {
                    setShowMenu(false);
                    onClearChat();
                  }}
                  style={{
                    display: "flex",
                    alignItems: "center",
                    gap: "6px",
                    width: "100%",
                    padding: "6px 8px",
                    background: "none",
                    border: "none",
                    borderRadius: "6px",
                    color: "#FF4D5E",
                    fontSize: "11px",
                    cursor: "pointer",
                    textAlign: "left"
                  }}
                >
                  <Trash2 size={12} />
                  <span>Bersih</span>
                </button>
              )}
            </div>
          )}
        </div>

        {onOpenTasks && (
          <button
            type="button"
            onClick={onOpenTasks}
            title="Tugas"
            style={{
              background: "none",
              border: "none",
              borderRadius: "6px",
              width: "24px",
              height: "24px",
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              cursor: "pointer",
              color: "#A7AEBB"
            }}
          >
            <CheckSquare size={13} />
          </button>
        )}

        {onClose && (
          <button
            type="button"
            onClick={onClose}
            title="Tutup"
            style={{
              background: "none",
              border: "none",
              borderRadius: "6px",
              width: "24px",
              height: "24px",
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              cursor: "pointer",
              color: "#A7AEBB"
            }}
          >
            <X size={13} />
          </button>
        )}
      </div>
    </div>
  );
};
