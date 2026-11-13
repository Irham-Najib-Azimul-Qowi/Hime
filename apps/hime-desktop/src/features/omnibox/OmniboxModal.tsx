import React, { useState, useEffect, useRef } from 'react';
import { CommandItem } from '../../types/commands';
import { Badge } from '../../components/ui/Badge';

export interface OmniboxModalProps {
  isOpen: boolean;
  onClose: () => void;
  commands: CommandItem[];
}

export const OmniboxModal: React.FC<OmniboxModalProps> = ({
  isOpen,
  onClose,
  commands,
}) => {
  const [query, setQuery] = useState('');
  const [selectedIndex, setSelectedIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);

  // Filter commands by query
  const filteredCommands = commands.filter(
    (cmd) =>
      cmd.title.toLowerCase().includes(query.toLowerCase()) ||
      cmd.description.toLowerCase().includes(query.toLowerCase()) ||
      cmd.category.toLowerCase().includes(query.toLowerCase())
  );

  useEffect(() => {
    if (isOpen) {
      setQuery('');
      setSelectedIndex(0);
      setTimeout(() => inputRef.current?.focus(), 50);
    }
  }, [isOpen]);

  useEffect(() => {
    setSelectedIndex(0);
  }, [query]);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (!isOpen) return;

      if (e.key === 'ArrowDown') {
        e.preventDefault();
        setSelectedIndex((prev) =>
          prev < filteredCommands.length - 1 ? prev + 1 : 0
        );
      } else if (e.key === 'ArrowUp') {
        e.preventDefault();
        setSelectedIndex((prev) =>
          prev > 0 ? prev - 1 : filteredCommands.length - 1
        );
      } else if (e.key === 'Enter') {
        e.preventDefault();
        const selected = filteredCommands[selectedIndex];
        if (selected) {
          selected.action();
          onClose();
        }
      } else if (e.key === 'Escape') {
        e.preventDefault();
        onClose();
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [isOpen, selectedIndex, filteredCommands, onClose]);

  if (!isOpen) return null;

  return (
    <div
      style={{
        position: 'fixed',
        inset: 0,
        zIndex: 999,
        display: 'flex',
        alignItems: 'flex-start',
        justifyContent: 'center',
        paddingTop: '80px',
        backgroundColor: 'rgba(0, 0, 0, 0.65)',
        backdropFilter: 'blur(8px)',
      }}
      onClick={(e) => {
        if (e.target === e.currentTarget) onClose();
      }}
    >
      <div
        className="animate-modal-enter app-no-drag"
        style={{
          width: '100%',
          maxWidth: '560px',
          backgroundColor: 'var(--bg-surface-1)',
          borderRadius: 'var(--radius-lg)',
          border: '1px solid var(--border-medium)',
          boxShadow: 'var(--shadow-modal)',
          overflow: 'hidden',
          display: 'flex',
          flexDirection: 'column',
          maxHeight: '480px',
        }}
      >
        {/* Search Input Bar */}
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            padding: '12px 16px',
            borderBottom: '1px solid var(--border-subtle)',
            gap: '10px',
          }}
        >
          <span style={{ color: 'var(--accent-primary)', fontSize: 'var(--text-md)' }}>
            ⌘
          </span>
          <input
            ref={inputRef}
            type="text"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="Type a command or search action..."
            style={{
              width: '100%',
              backgroundColor: 'transparent',
              border: 'none',
              outline: 'none',
              color: 'var(--text-primary)',
              fontSize: 'var(--text-md)',
              fontFamily: 'inherit',
            }}
          />
          <span
            style={{
              fontSize: 'var(--text-2xs)',
              color: 'var(--text-muted)',
              border: '1px solid var(--border-subtle)',
              borderRadius: 'var(--radius-xs)',
              padding: '2px 6px',
            }}
          >
            ESC
          </span>
        </div>

        {/* Command Results List */}
        <div
          style={{
            padding: '8px',
            overflowY: 'auto',
            maxHeight: '380px',
          }}
        >
          {filteredCommands.length === 0 ? (
            <div
              style={{
                padding: '24px 16px',
                textAlign: 'center',
                color: 'var(--text-muted)',
                fontSize: 'var(--text-sm)',
              }}
            >
              No matching commands found for "{query}".
            </div>
          ) : (
            filteredCommands.map((cmd, idx) => {
              const isSelected = idx === selectedIndex;
              return (
                <div
                  key={cmd.id}
                  onClick={() => {
                    cmd.action();
                    onClose();
                  }}
                  onMouseEnter={() => setSelectedIndex(idx)}
                  className="btn-pressable"
                  style={{
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'space-between',
                    padding: '10px 14px',
                    borderRadius: 'var(--radius-sm)',
                    backgroundColor: isSelected
                      ? 'var(--bg-surface-hover)'
                      : 'transparent',
                    cursor: 'pointer',
                    marginBottom: '2px',
                    borderLeft: isSelected
                      ? '3px solid var(--accent-primary)'
                      : '3px solid transparent',
                  }}
                >
                  <div style={{ display: 'flex', flexDirection: 'column', gap: '2px' }}>
                    <div
                      style={{
                        display: 'flex',
                        alignItems: 'center',
                        gap: '8px',
                      }}
                    >
                      <span
                        style={{
                          fontWeight: 500,
                          fontSize: 'var(--text-sm)',
                          color: isSelected
                            ? 'var(--text-primary)'
                            : 'var(--text-primary)',
                        }}
                      >
                        {cmd.title}
                      </span>
                      {cmd.badge && (
                        <Badge
                          variant={cmd.isPlanned ? 'warning' : 'cyan'}
                          size="xs"
                        >
                          {cmd.badge}
                        </Badge>
                      )}
                    </div>
                    <span
                      style={{
                        fontSize: 'var(--text-xs)',
                        color: 'var(--text-muted)',
                      }}
                    >
                      {cmd.description}
                    </span>
                  </div>

                  {cmd.shortcut && (
                    <span
                      style={{
                        fontSize: 'var(--text-2xs)',
                        color: 'var(--text-muted)',
                        backgroundColor: 'var(--bg-surface-2)',
                        padding: '2px 6px',
                        borderRadius: 'var(--radius-xs)',
                        border: '1px solid var(--border-subtle)',
                        fontFamily: 'var(--font-mono)',
                      }}
                    >
                      {cmd.shortcut}
                    </span>
                  )}
                </div>
              );
            })
          )}
        </div>

        {/* Footer info bar */}
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
            padding: '8px 16px',
            borderTop: '1px solid var(--border-subtle)',
            backgroundColor: 'var(--bg-surface-2)',
            fontSize: 'var(--text-2xs)',
            color: 'var(--text-muted)',
          }}
        >
          <span>Use ↑↓ to navigate • ↵ to execute</span>
          <span>Hime Universal Omnibox</span>
        </div>
      </div>
    </div>
  );
};

