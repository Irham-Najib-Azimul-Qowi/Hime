import React from "react";

export interface ToggleProps {
  checked: boolean;
  onChange: (checked: boolean) => void;
  disabled?: boolean;
  className?: string;
}

export const Toggle: React.FC<ToggleProps> = ({
  checked,
  onChange,
  disabled = false,
  className = ""
}) => {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={checked}
      disabled={disabled}
      className={`hime-toggle ${className}`}
      onClick={() => !disabled && onChange(!checked)}
      style={{
        position: "relative",
        width: "36px",
        height: "20px",
        borderRadius: "9999px",
        backgroundColor: checked ? "var(--accent-mint, #34E3A6)" : "var(--surface-3, #1B1E26)",
        border: "1px solid var(--line, #262A34)",
        cursor: disabled ? "not-allowed" : "pointer",
        padding: "2px",
        display: "inline-flex",
        alignItems: "center",
        transition: "background-color 0.2s cubic-bezier(0.16, 1, 0.3, 1)",
        outline: "none"
      }}
    >
      <span
        style={{
          width: "14px",
          height: "14px",
          borderRadius: "50%",
          backgroundColor: checked ? "#0A0B0E" : "var(--text-2, #A7AEBB)",
          transform: checked ? "translateX(16px)" : "translateX(0px)",
          transition: "transform 0.22s cubic-bezier(0.16, 1, 0.3, 1), background-color 0.2s ease"
        }}
      />
    </button>
  );
};
