import React from 'react';

export interface BadgeProps {
  variant?: 'default' | 'success' | 'warning' | 'danger' | 'cyan' | 'violet';
  size?: 'xs' | 'sm';
  dot?: boolean;
  children: React.ReactNode;
  className?: string;
}

export const Badge: React.FC<BadgeProps> = ({
  variant = 'default',
  size = 'sm',
  dot = false,
  children,
  className = '',
}) => {
  const getBadgeColors = () => {
    switch (variant) {
      case 'success':
        return {
          bg: 'var(--accent-success-subtle)',
          text: 'var(--accent-success)',
          border: 'rgba(16, 185, 129, 0.25)',
          dotColor: 'var(--accent-success)',
        };
      case 'warning':
        return {
          bg: 'var(--accent-warning-subtle)',
          text: 'var(--accent-warning)',
          border: 'rgba(245, 158, 11, 0.25)',
          dotColor: 'var(--accent-warning)',
        };
      case 'danger':
        return {
          bg: 'var(--accent-danger-subtle)',
          text: 'var(--accent-danger)',
          border: 'rgba(239, 68, 68, 0.25)',
          dotColor: 'var(--accent-danger)',
        };
      case 'cyan':
        return {
          bg: 'var(--accent-cyan-subtle)',
          text: 'var(--accent-cyan)',
          border: 'rgba(6, 182, 212, 0.25)',
          dotColor: 'var(--accent-cyan)',
        };
      case 'violet':
        return {
          bg: 'var(--accent-primary-subtle)',
          text: 'var(--accent-primary)',
          border: 'rgba(139, 92, 246, 0.25)',
          dotColor: 'var(--accent-primary)',
        };
      case 'default':
      default:
        return {
          bg: 'var(--bg-surface-3)',
          text: 'var(--text-secondary)',
          border: 'var(--border-subtle)',
          dotColor: 'var(--text-muted)',
        };
    }
  };

  const colors = getBadgeColors();

  return (
    <span
      className={className}
      style={{
        display: 'inline-flex',
        alignItems: 'center',
        gap: '4px',
        padding: size === 'xs' ? '2px 6px' : '3px 8px',
        fontSize: size === 'xs' ? 'var(--text-2xs)' : 'var(--text-xs)',
        fontWeight: 500,
        borderRadius: 'var(--radius-pill)',
        backgroundColor: colors.bg,
        color: colors.text,
        border: `1px solid ${colors.border}`,
        whiteSpace: 'nowrap',
        letterSpacing: '0.02em',
      }}
    >
      {dot && (
        <span
          style={{
            width: '5px',
            height: '5px',
            borderRadius: '50%',
            backgroundColor: colors.dotColor,
          }}
        />
      )}
      {children}
    </span>
  );
};

