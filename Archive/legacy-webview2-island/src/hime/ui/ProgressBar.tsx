import React from "react";
import { PillAccent } from "./Pill";

export interface ProgressBarProps {
  value: number; // 0 to 100
  accent?: PillAccent;
  showLabel?: boolean;
  className?: string;
}

export const ProgressBar: React.FC<ProgressBarProps> = ({
  value,
  accent = "lime",
  showLabel = false,
  className = ""
}) => {
  const clamped = Math.max(0, Math.min(100, value));

  return (
    <div
      className={`hime-progress-bar-wrapper ${className}`}
      style={{ display: "flex", alignItems: "center", gap: "8px", width: "100%" }}
    >
      <div
        style={{
          flex: 1,
          height: "6px",
          borderRadius: "9999px",
          backgroundColor: "var(--surface-3, #1B1E26)",
          overflow: "hidden",
          position: "relative"
        }}
      >
        <div
          style={{
            height: "100%",
            width: `${clamped}%`,
            borderRadius: "9999px",
            backgroundColor: `var(--accent-${accent}, #A8EE4B)`,
            transition: "width 0.3s cubic-bezier(0.16, 1, 0.3, 1)"
          }}
        />
      </div>
      {showLabel && (
        <span
          style={{
            fontSize: "11px",
            color: "var(--text-2, #A7AEBB)",
            fontVariantNumeric: "tabular-nums",
            minWidth: "30px",
            textAlign: "right"
          }}
        >
          {Math.round(clamped)}%
        </span>
      )}
    </div>
  );
};
