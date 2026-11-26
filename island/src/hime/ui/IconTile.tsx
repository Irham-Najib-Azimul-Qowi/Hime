import React from "react";
import { PillAccent } from "./Pill";

export interface IconTileProps {
  icon: React.ReactNode;
  accent?: PillAccent;
  size?: number; // 28 to 32px
  className?: string;
}

export const IconTile: React.FC<IconTileProps> = ({
  icon,
  accent = "mint",
  size = 30,
  className = ""
}) => {
  return (
    <div
      className={`hime-icon-tile ${className}`}
      style={{
        width: `${size}px`,
        height: `${size}px`,
        minWidth: `${size}px`,
        borderRadius: "var(--radius-icon-tile, 10px)",
        backgroundColor: `var(--accent-${accent}-tint, rgba(52, 227, 166, 0.16))`,
        color: `var(--accent-${accent}, #34E3A6)`,
        display: "inline-flex",
        alignItems: "center",
        justifyContent: "center",
        flexShrink: 0
      }}
    >
      {icon}
    </div>
  );
};
