import React from "react";

export type ButtonVariant = "primary" | "secondary" | "ghost" | "danger";

export interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: ButtonVariant;
  size?: "sm" | "md";
  icon?: React.ReactNode;
}

export const Button: React.FC<ButtonProps> = ({
  children,
  variant = "primary",
  size = "md",
  icon,
  style,
  disabled,
  className = "",
  ...props
}) => {
  const getVariantStyles = (): React.CSSProperties => {
    switch (variant) {
      case "primary":
        return {
          backgroundColor: "var(--accent-mint, #34E3A6)",
          color: "#0A0B0E",
          border: "none",
          fontWeight: 600
        };
      case "secondary":
        return {
          backgroundColor: "var(--accent-mint-tint, rgba(52, 227, 166, 0.18))",
          color: "var(--accent-mint, #34E3A6)",
          border: "none",
          fontWeight: 500
        };
      case "danger":
        return {
          backgroundColor: "var(--accent-red-tint, rgba(255, 77, 94, 0.18))",
          color: "var(--accent-red, #FF4D5E)",
          border: "none",
          fontWeight: 500
        };
      case "ghost":
      default:
        return {
          backgroundColor: "transparent",
          color: "var(--text-1, #F4F6FA)",
          border: "1px solid var(--line, #262A34)",
          fontWeight: 500
        };
    }
  };

  const height = size === "sm" ? "28px" : "34px";

  return (
    <button
      className={`hime-button ${className}`}
      disabled={disabled}
      style={{
        display: "inline-flex",
        alignItems: "center",
        justifyContent: "center",
        gap: "6px",
        height,
        padding: size === "sm" ? "0 10px" : "0 14px",
        borderRadius: "var(--radius-input, 12px)",
        fontSize: size === "sm" ? "12px" : "13px",
        cursor: disabled ? "not-allowed" : "pointer",
        opacity: disabled ? 0.5 : 1,
        transition: "all 0.15s ease",
        outline: "none",
        ...getVariantStyles(),
        ...style
      }}
      {...props}
    >
      {icon && <span style={{ display: "inline-flex" }}>{icon}</span>}
      {children}
    </button>
  );
};
