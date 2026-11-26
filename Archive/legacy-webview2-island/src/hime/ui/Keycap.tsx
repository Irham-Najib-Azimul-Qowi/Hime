import React from "react";

export interface KeycapProps {
  combo: string; // e.g. "Ctrl+Alt+Space"
  className?: string;
}

export const Keycap: React.FC<KeycapProps> = ({ combo, className = "" }) => {
  const keys = combo.split("+");

  return (
    <div
      className={`hime-keycap-group ${className}`}
      style={{ display: "inline-flex", alignItems: "center", gap: "4px" }}
    >
      {keys.map((k, idx) => (
        <span
          key={idx}
          style={{
            display: "inline-flex",
            alignItems: "center",
            justifyContent: "center",
            minWidth: "22px",
            height: "22px",
            padding: "0 5px",
            borderRadius: "6px",
            backgroundColor: "var(--surface-3, #1B1E26)",
            border: "1px solid var(--line, #262A34)",
            borderBottom: "2px solid var(--line, #262A34)",
            fontSize: "11px",
            fontWeight: 600,
            color: "var(--text-1, #F4F6FA)",
            fontFamily: "inherit",
            userSelect: "none"
          }}
        >
          {k.trim()}
        </span>
      ))}
    </div>
  );
};
