import React from 'react';

export interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: 'primary' | 'secondary' | 'subtle' | 'danger' | 'ghost';
  size?: 'sm' | 'md' | 'lg';
  isLoading?: boolean;
  icon?: React.ReactNode;
}

export const Button: React.FC<ButtonProps> = ({
  variant = 'secondary',
  size = 'md',
  isLoading = false,
  icon,
  children,
  className = '',
  disabled,
  ...props
}) => {
  const getVariantStyles = (): React.CSSProperties => {
    switch (variant) {
      case 'primary':
        return {
          backgroundColor: 'var(--accent-primary)',
          color: '#ffffff',
          border: '1px solid rgba(255, 255, 255, 0.15)',
        };
      case 'danger':
        return {
          backgroundColor: 'var(--accent-danger)',
          color: '#ffffff',
          border: '1px solid rgba(255, 255, 255, 0.15)',
        };
      case 'subtle':
        return {
          backgroundColor: 'var(--accent-primary-subtle)',
          color: 'var(--accent-primary)',
          border: '1px solid rgba(139, 92, 246, 0.2)',
        };
      case 'ghost':
        return {
          backgroundColor: 'transparent',
          color: 'var(--text-secondary)',
          border: '1px solid transparent',
        };
      case 'secondary':
      default:
        return {
          backgroundColor: 'var(--bg-surface-2)',
          color: 'var(--text-primary)',
          border: '1px solid var(--border-medium)',
        };
    }
  };

  const getSizeStyles = (): React.CSSProperties => {
    switch (size) {
      case 'sm':
        return {
          padding: '4px 10px',
          fontSize: 'var(--text-xs)',
          borderRadius: 'var(--radius-sm)',
        };
      case 'lg':
        return {
          padding: '10px 18px',
          fontSize: 'var(--text-md)',
          borderRadius: 'var(--radius-md)',
        };
      case 'md':
      default:
        return {
          padding: '6px 14px',
          fontSize: 'var(--text-sm)',
          borderRadius: 'var(--radius-sm)',
        };
    }
  };

  return (
    <button
      className={`btn-pressable ${className}`}
      disabled={disabled || isLoading}
      style={{
        display: 'inline-flex',
        alignItems: 'center',
        justifyContent: 'center',
        gap: '6px',
        fontWeight: 500,
        cursor: disabled || isLoading ? 'not-allowed' : 'pointer',
        opacity: disabled ? 0.5 : 1,
        fontFamily: 'inherit',
        ...getVariantStyles(),
        ...getSizeStyles(),
      }}
      {...props}
    >
      {isLoading ? (
        <span
          className="animate-spin"
          style={{
            display: 'inline-block',
            width: '12px',
            height: '12px',
            border: '2px solid currentColor',
            borderRightColor: 'transparent',
            borderRadius: '50%',
          }}
        />
      ) : (
        icon
      )}
      {children}
    </button>
  );
};

