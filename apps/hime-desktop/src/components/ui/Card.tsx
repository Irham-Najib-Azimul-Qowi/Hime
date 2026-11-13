import React from 'react';

export interface CardProps extends React.HTMLAttributes<HTMLDivElement> {
  variant?: 'surface-1' | 'surface-2' | 'surface-3';
  interactive?: boolean;
}

export const Card: React.FC<CardProps> = ({
  variant = 'surface-1',
  interactive = false,
  children,
  className = '',
  style,
  ...props
}) => {
  const getBg = () => {
    switch (variant) {
      case 'surface-2':
        return 'var(--bg-surface-2)';
      case 'surface-3':
        return 'var(--bg-surface-3)';
      case 'surface-1':
      default:
        return 'var(--bg-surface-1)';
    }
  };

  return (
    <div
      className={`card ${interactive ? 'btn-pressable' : ''} ${className}`}
      style={{
        backgroundColor: getBg(),
        borderRadius: 'var(--radius-md)',
        border: '1px solid var(--border-subtle)',
        padding: 'var(--space-4)',
        boxShadow: 'var(--shadow-sm)',
        transition: 'border-color var(--duration-normal) ease-out, background-color var(--duration-normal) ease-out',
        ...style,
      }}
      {...props}
    >
      {children}
    </div>
  );
};

