import React from "react";
import { IconTile } from "./IconTile";
import { PillAccent } from "./Pill";

export interface RowProps {
  title: string;
  desc?: string;
  icon?: React.ReactNode;
  accent?: PillAccent;
  trailing?: React.ReactNode;
  onClick?: () => void;
  className?: string;
}

export const Row: React.FC<RowProps> = ({
  title,
  desc,
  icon,
  accent = "mint",
  trailing,
  onClick,
  className = ""
}) => {
  return (
    <div
      className={`hime-row ${className}`}
      onClick={onClick}
      style={{
        display: "flex",
        alignItems: "center",
        justifyContent: "space-between",
        minHeight: "44px",
        padding: "8px 12px",
        gap: "12px",
        borderRadius: "var(--radius-card, 14px)",
        backgroundColor: "var(--surface-2, #14161C)",
        cursor: onClick ? "pointer" : "default",
        transition: "background-color 0.15s ease"
      }}
    >
      <div style={{ display: "flex", alignItems: "center", gap: "10px", minWidth: 0 }}>
        {icon && <IconTile icon={icon} accent={accent} size={28} />}
        <div style={{ display: "flex", flexDirection: "column", minWidth: 0 }}>
          <span
            style={{
              fontSize: "var(--font-title-size, 14px)",
              fontWeight: 600,
              color: "var(--text-1, #F4F6FA)",
              overflow: "hidden",
              textOverflow: "ellipsis",
              whiteSpace: "nowrap"
            }}
          >
            {title}
          </span>
          {desc && (
            <span
              style={{
                fontSize: "var(--font-desc-size, 12px)",
                color: "var(--text-2, #A7AEBB)",
                overflow: "hidden",
                textOverflow: "ellipsis",
                whiteSpace: "nowrap"
              }}
            >
              {desc}
            </span>
          )}
        </div>
      </div>

      {trailing && <div style={{ flexShrink: 0, marginLeft: "8px" }}>{trailing}</div>}
    </div>
  );
};
