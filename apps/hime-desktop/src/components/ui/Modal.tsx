import React, { useEffect } from 'react';

export interface ModalProps {
  isOpen: boolean;
  onClose: () => void;
  title?: React.ReactNode;
  children: React.ReactNode;
  maxWidth?: string;
}

export const Modal: React.FC<ModalProps> = ({
  isOpen,
  onClose,
  title,
  children,
  maxWidth = '540px',
}) => {
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && isOpen) {
        onClose();
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [isOpen, onClose]);

  if (!isOpen) return null;

  return (
    <div
      style={{
        position: 'fixed',
        inset: 0,
        zIndex: 999,
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        backgroundColor: 'rgba(0, 0, 0, 0.65)',
        backdropFilter: 'blur(8px)',
      }}
      onClick={(e) => {
        if (e.target === e.currentTarget) onClose();
      }}
    >
      <div
        className="animate-modal-enter"
        style={{
          width: '90%',
          maxWidth,
          backgroundColor: 'var(--bg-surface-1)',
          borderRadius: 'var(--radius-lg)',
          border: '1px solid var(--border-medium)',
          boxShadow: 'var(--shadow-modal)',
          overflow: 'hidden',
          display: 'flex',
          flexDirection: 'column',
          maxHeight: '85vh',
        }}
      >
        {title && (
          <div
            style={{
              padding: 'var(--space-4)',
              borderBottom: '1px solid var(--border-subtle)',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'space-between',
            }}
          >
            <div style={{ fontWeight: 600, fontSize: 'var(--text-md)' }}>{title}</div>
            <button
              onClick={onClose}
              className="btn-pressable"
              style={{
                background: 'transparent',
                border: 'none',
                color: 'var(--text-muted)',
                cursor: 'pointer',
                fontSize: 'var(--text-md)',
                padding: '4px 8px',
                borderRadius: 'var(--radius-xs)',
              }}
              title="Close (Esc)"
            >
              ✕
            </button>
          </div>
        )}
        <div style={{ padding: 'var(--space-4)', overflowY: 'auto' }}>{children}</div>
      </div>
    </div>
  );
};

