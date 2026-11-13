import React from 'react';

export interface SwitchProps {
  checked: boolean;
  onChange: (checked: boolean) => void;
  label?: string;
  disabled?: boolean;
}

export const Switch: React.FC<SwitchProps> = ({
  checked,
  onChange,
  label,
  disabled = false,
}) => {
  return (
    <label
      style={{
        display: 'inline-flex',
        alignItems: 'center',
        gap: '10px',
        cursor: disabled ? 'not-allowed' : 'pointer',
        opacity: disabled ? 0.5 : 1,
      }}
    >
      <div
        onClick={() => !disabled && onChange(!checked)}
        className="btn-pressable"
        style={{
          width: '36px',
          height: '20px',
          borderRadius: 'var(--radius-pill)',
          backgroundColor: checked ? 'var(--accent-primary)' : 'var(--bg-surface-3)',
          border: '1px solid var(--border-medium)',
          position: 'relative',
          transition: 'background-color var(--duration-micro) ease-out',
        }}
      >
        <div
          style={{
            position: 'absolute',
            top: '2px',
            left: checked ? '18px' : '2px',
            width: '14px',
            height: '14px',
            borderRadius: '50%',
            backgroundColor: '#ffffff',
            boxShadow: 'var(--shadow-sm)',
            transition: 'left var(--duration-micro) var(--ease-spring)',
          }}
        />
      </div>
      {label && (
        <span style={{ fontSize: 'var(--text-sm)', color: 'var(--text-secondary)' }}>
          {label}
        </span>
      )}
    </label>
  );
};

