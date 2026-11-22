import React from "react";

export type PillVariant = "solid" | "tint" | "outline";
export type PillAccent =
  | "mint"
  | "cyan"
  | "sky"
  | "violet"
  | "pink"
  | "coral"
  | "amber"
  | "lime"
  | "red"
  | "text-3";

export interface PillProps {
  label: string;
  variant?: PillVariant;
  accent?: PillAccent;
  icon?: React.ReactNode;
  onRemove?: () => void;
  onClick?: () => void;
  className?: string;
}

export const Pill: React.FC<PillProps> = ({
  label,
  variant = "tint",
  accent = "mint",
  icon,
  onRemove,
  onClick,
  className = ""
}) => {
  const getStyle = (): React.CSSProperties => {
    const colorVar = accent === "text-3" ? "var(--text-3, #6F7785)" : `var(--accent-${accent}, #34E3A6)`;
    const tintVar = accent === "text-3" ? "rgba(111, 119, 133, 0.18)" : `var(--accent-${accent}-tint, rgba(52, 227, 166, 0.18))`;

    if (variant === "solid") {
      const isMutedAccent = accent === "text-3";
      return {
        backgroundColor: colorVar,
        color: isMutedAccent ? "#F4F6FA" : "#0A0B0E",
        border: "none",
        fontWeight: 600
      };
    }
    if (variant === "outline") {
      return {
        backgroundColor: "transparent",
        color: colorVar,
        border: `1px solid ${colorVar}`,
        fontWeight: 500
      };
    }
    // tint variant (default)
    return {
      backgroundColor: tintVar,
      color: colorVar,
      border: "none",
      fontWeight: 500
    };
  };

  return (
    <div
      className={`hime-pill ${className}`}
      onClick={onClick}
      style={{
        display: "inline-flex",
        alignItems: "center",
        height: "24px",
        padding: "0 8px",
        borderRadius: "9999px",
        fontSize: "12px",
        lineHeight: "1",
        gap: "4px",
        cursor: onClick ? "pointer" : "default",
        userSelect: "none",
        transition: "all 0.15s ease",
        ...getStyle()
      }}
    >
      {icon && <span style={{ display: "inline-flex", fontSize: "14px" }}>{icon}</span>}
      <span style={{ overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
        {label}
      </span>
      {onRemove && (
        <span
          onClick={(e) => {
            e.stopPropagation();
            onRemove();
          }}
          style={{
            cursor: "pointer",
            marginLeft: "2px",
            opacity: 0.7,
            fontSize: "10px"
          }}
        >
          ✕
        </span>
      )}
    </div>
  );
};
