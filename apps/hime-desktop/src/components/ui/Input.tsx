import React from 'react';

export interface InputProps extends React.InputHTMLAttributes<HTMLInputElement> {
  icon?: React.ReactNode;
}

export const Input = React.forwardRef<HTMLInputElement, InputProps>(
  ({ icon, className = '', style, ...props }, ref) => {
    return (
      <div
        style={{
          position: 'relative',
          display: 'flex',
          alignItems: 'center',
          width: '100%',
        }}
      >
        {icon && (
          <span
            style={{
              position: 'absolute',
              left: '12px',
              color: 'var(--text-muted)',
              display: 'flex',
              alignItems: 'center',
              pointerEvents: 'none',
            }}
          >
            {icon}
          </span>
        )}
        <input
          ref={ref}
          className={`hime-input ${className}`}
          style={{
            width: '100%',
            padding: icon ? '8px 12px 8px 36px' : '8px 12px',
            fontSize: 'var(--text-sm)',
            fontFamily: 'inherit',
            backgroundColor: 'var(--bg-surface-2)',
            color: 'var(--text-primary)',
            border: '1px solid var(--border-medium)',
            borderRadius: 'var(--radius-sm)',
            transition: 'border-color var(--duration-micro) ease-out, box-shadow var(--duration-micro) ease-out',
            outline: 'none',
            ...style,
          }}
          {...props}
        />
      </div>
    );
  }
);

Input.displayName = 'Input';

