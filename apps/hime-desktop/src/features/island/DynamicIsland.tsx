import React, { useState } from 'react';
import { SystemStatus } from '../../types/ipc';
import { Badge } from '../../components/ui/Badge';

export interface DynamicIslandProps {
  status: SystemStatus | null;
  isCoreOnline: boolean;
  activeActivity: string | null;
  translationResult?: { original: string; translated: string } | null;
  onClearTranslation?: () => void;
  onOpenOmnibox: () => void;
  onOpenAssistant?: () => void;
  onOpenWorkstation?: () => void;
  onMinimizeWindow?: () => void;
  onCloseWindow?: () => void;
  unreadCount: number;
  isFloatingMode?: boolean;
}

export const DynamicIsland: React.FC<DynamicIslandProps> = ({
  status: _status,
  isCoreOnline,
  activeActivity,
  translationResult,
  onClearTranslation,
  onOpenOmnibox,
  onOpenAssistant,
  onOpenWorkstation,
  onMinimizeWindow,
  onCloseWindow,
  unreadCount,
  isFloatingMode = false,
}) => {
  const [isExpanded, setIsExpanded] = useState(false);
  const [quickPrompt, setQuickPrompt] = useState('');

  const isActivityState = Boolean(activeActivity);
  const hasTranslation = Boolean(translationResult);

  const handleQuickSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!quickPrompt.trim()) return;
    if (onOpenAssistant) {
      onOpenAssistant();
    }
  };

  return (
    <div
      style={{
        display: 'flex',
        justifyContent: 'center',
        alignItems: 'center',
        width: '100%',
        padding: isFloatingMode ? '10px 16px' : '0',
        position: 'relative',
        zIndex: 50,
      }}
    >
      <div
        className="app-drag-region"
        style={{
          width: hasTranslation ? '520px' : isExpanded ? '520px' : '460px',
          minHeight: hasTranslation ? '110px' : isExpanded ? '150px' : '44px',
          backgroundColor: 'rgba(9, 11, 17, 0.96)',
          borderRadius: isExpanded || hasTranslation ? '22px' : '9999px',
          border: '1px solid rgba(255, 255, 255, 0.12)',
          boxShadow:
            '0 16px 36px -6px rgba(0, 0, 0, 0.8), 0 0 0 1px rgba(255, 255, 255, 0.06), 0 0 20px rgba(139, 92, 246, 0.2)',
          display: 'flex',
          flexDirection: 'column',
          justifyContent: isExpanded || hasTranslation ? 'space-between' : 'center',
          padding: isExpanded || hasTranslation ? '14px 18px' : '0 14px',
          transition:
            'width 220ms cubic-bezier(0.16, 1, 0.3, 1), min-height 220ms cubic-bezier(0.16, 1, 0.3, 1), border-radius 220ms ease-out, background-color 150ms ease-out',
          backdropFilter: 'blur(20px)',
          overflow: 'hidden',
          cursor: 'default',
        }}
      >
        {/* State A: Translation Banner */}
        {hasTranslation && translationResult ? (
          <div
            className="app-no-drag animate-fade-in"
            style={{
              display: 'flex',
              flexDirection: 'column',
              height: '100%',
              justifyContent: 'space-between',
              gap: '6px',
            }}
          >
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                <Badge variant="cyan" size="xs">TERJEMAHAN ID</Badge>
                <span
                  style={{
                    fontSize: '11px',
                    color: '#94a3b8',
                    textOverflow: 'ellipsis',
                    overflow: 'hidden',
                    whiteSpace: 'nowrap',
                    maxWidth: '300px',
                  }}
                >
                  "{translationResult.original}"
                </span>
              </div>
              <div style={{ display: 'flex', gap: '6px' }}>
                <button
                  onClick={() => navigator.clipboard.writeText(translationResult.translated)}
                  className="btn-pressable"
                  style={{
                    backgroundColor: 'rgba(255, 255, 255, 0.08)',
                    border: '1px solid rgba(255, 255, 255, 0.1)',
                    borderRadius: '4px',
                    padding: '2px 8px',
                    fontSize: '10px',
                    color: '#f8fafc',
                    cursor: 'pointer',
                  }}
                >
                  Salin
                </button>
                <button
                  onClick={onClearTranslation}
                  className="btn-pressable"
                  style={{
                    backgroundColor: 'rgba(255, 255, 255, 0.08)',
                    border: '1px solid rgba(255, 255, 255, 0.1)',
                    borderRadius: '4px',
                    padding: '2px 8px',
                    fontSize: '10px',
                    color: '#94a3b8',
                    cursor: 'pointer',
                  }}
                >
                  ✕
                </button>
              </div>
            </div>
            <div style={{ fontSize: '13px', fontWeight: 600, color: '#f8fafc' }}>
              {translationResult.translated}
            </div>
          </div>
        ) : (
          <>
            {/* Top Bar: Always Visible Compact Pill */}
            <div
              style={{
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'space-between',
                width: '100%',
                height: '44px',
              }}
            >
              {/* Left: Mascot + Glowing Status LED */}
              <div
                className="app-no-drag"
                onClick={() => setIsExpanded(!isExpanded)}
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: '8px',
                  cursor: 'pointer',
                }}
                title="Klik untuk ekspansi Dynamic Island"
              >
                <span
                  style={{
                    width: '8px',
                    height: '8px',
                    borderRadius: '50%',
                    backgroundColor: isCoreOnline ? '#10b981' : '#ef4444',
                    display: 'inline-block',
                    boxShadow: isCoreOnline ? '0 0 10px #10b981' : 'none',
                    animation: isCoreOnline ? 'pulse 2s infinite' : 'none',
                  }}
                />
                <span
                  style={{
                    fontWeight: 700,
                    fontSize: '12px',
                    letterSpacing: '0.04em',
                    color: '#f8fafc',
                  }}
                >
                  ✨ Hime
                </span>
                <span
                  style={{
                    fontSize: '10px',
                    color: isCoreOnline ? '#10b981' : '#ef4444',
                    fontWeight: 500,
                  }}
                >
                  {isCoreOnline ? 'Online' : 'Offline'}
                </span>
              </div>

              {/* Center: Activity / Quick Prompt Trigger */}
              <div
                className="app-no-drag"
                onClick={onOpenOmnibox}
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: '6px',
                  backgroundColor: 'rgba(255, 255, 255, 0.05)',
                  border: '1px solid rgba(255, 255, 255, 0.08)',
                  borderRadius: '9999px',
                  padding: '4px 12px',
                  cursor: 'pointer',
                  maxWidth: '220px',
                  overflow: 'hidden',
                  textOverflow: 'ellipsis',
                  whiteSpace: 'nowrap',
                }}
                title="Buka Omnibox Command Palette (Ctrl+K)"
              >
                {isActivityState ? (
                  <>
                    <span
                      style={{
                        display: 'inline-block',
                        width: '8px',
                        height: '8px',
                        border: '2px solid #06b6d4',
                        borderRightColor: 'transparent',
                        borderRadius: '50%',
                        animation: 'spin 1s linear infinite',
                      }}
                    />
                    <span style={{ fontSize: '11px', color: '#06b6d4', fontWeight: 500 }}>
                      {activeActivity}
                    </span>
                  </>
                ) : (
                  <>
                    <span style={{ fontSize: '11px', color: '#64748b' }}>
                      Tanya Hime...
                    </span>
                    <span
                      style={{
                        fontSize: '9px',
                        backgroundColor: 'rgba(255, 255, 255, 0.1)',
                        padding: '1px 5px',
                        borderRadius: '4px',
                        color: '#94a3b8',
                        fontWeight: 600,
                      }}
                    >
                      Ctrl+K
                    </span>
                  </>
                )}
              </div>

              {/* Right: Quick Controls & Workstation Expansion */}
              <div
                className="app-no-drag"
                style={{ display: 'flex', alignItems: 'center', gap: '6px' }}
              >
                {unreadCount > 0 && (
                  <Badge variant="warning" size="xs">
                    {unreadCount}
                  </Badge>
                )}

                {/* Toggle Island Expanded Drawer */}
                <button
                  onClick={() => setIsExpanded(!isExpanded)}
                  className="btn-pressable"
                  style={{
                    backgroundColor: isExpanded ? 'rgba(139, 92, 246, 0.25)' : 'rgba(255, 255, 255, 0.06)',
                    border: '1px solid rgba(255, 255, 255, 0.1)',
                    borderRadius: '9999px',
                    padding: '4px 8px',
                    fontSize: '11px',
                    color: isExpanded ? '#a78bfa' : '#94a3b8',
                    cursor: 'pointer',
                  }}
                  title={isExpanded ? 'Ciutkan Island' : 'Buka Panel Cepat Island'}
                >
                  {isExpanded ? '▲' : '💬'}
                </button>

                {/* Expand to Full Workstation */}
                {onOpenWorkstation && (
                  <button
                    onClick={onOpenWorkstation}
                    className="btn-pressable"
                    style={{
                      backgroundColor: 'rgba(255, 255, 255, 0.06)',
                      border: '1px solid rgba(255, 255, 255, 0.1)',
                      borderRadius: '9999px',
                      padding: '4px 8px',
                      fontSize: '11px',
                      color: '#94a3b8',
                      cursor: 'pointer',
                    }}
                    title="Buka Workstation Lengkap (Papan Tugas, Storage, Dokumen, Dev)"
                  >
                    ⛶
                  </button>
                )}

                {/* Floating mode window controls */}
                {isFloatingMode && (
                  <>
                    {onMinimizeWindow && (
                      <button
                        onClick={onMinimizeWindow}
                        style={{
                          background: 'transparent',
                          border: 'none',
                          color: '#64748b',
                          cursor: 'pointer',
                          padding: '2px 6px',
                          fontSize: '11px',
                        }}
                        title="Minimize"
                      >
                        —
                      </button>
                    )}
                    {onCloseWindow && (
                      <button
                        onClick={onCloseWindow}
                        style={{
                          background: 'transparent',
                          border: 'none',
                          color: '#64748b',
                          cursor: 'pointer',
                          padding: '2px 6px',
                          fontSize: '11px',
                        }}
                        title="Tutup"
                      >
                        ✕
                      </button>
                    )}
                  </>
                )}
              </div>
            </div>

            {/* Bottom State: Island Expansion Drawer */}
            {isExpanded && (
              <div
                className="app-no-drag animate-fade-in"
                style={{
                  display: 'flex',
                  flexDirection: 'column',
                  gap: '10px',
                  paddingTop: '10px',
                  borderTop: '1px solid rgba(255, 255, 255, 0.08)',
                }}
              >
                {/* Fast prompt form */}
                <form
                  onSubmit={handleQuickSubmit}
                  style={{ display: 'flex', gap: '8px', alignItems: 'center' }}
                >
                  <input
                    type="text"
                    value={quickPrompt}
                    onChange={(e) => setQuickPrompt(e.target.value)}
                    placeholder="Ketik pertanyaan atau instruksi tugas..."
                    autoFocus
                    style={{
                      flex: 1,
                      backgroundColor: 'rgba(255, 255, 255, 0.05)',
                      border: '1px solid rgba(255, 255, 255, 0.12)',
                      borderRadius: '8px',
                      padding: '8px 12px',
                      color: '#f8fafc',
                      fontSize: '12px',
                      outline: 'none',
                    }}
                  />
                  <button
                    type="submit"
                    className="btn-pressable"
                    style={{
                      backgroundColor: '#8b5cf6',
                      border: 'none',
                      borderRadius: '8px',
                      padding: '8px 14px',
                      color: '#ffffff',
                      fontSize: '12px',
                      fontWeight: 600,
                      cursor: 'pointer',
                    }}
                  >
                    Kirim
                  </button>
                </form>

                {/* Quick Action Suggestion Chips */}
                <div style={{ display: 'flex', gap: '6px', flexWrap: 'wrap' }}>
                  <button
                    onClick={() => {
                      if (onOpenAssistant) onOpenAssistant();
                    }}
                    className="btn-pressable"
                    style={{
                      backgroundColor: 'rgba(255, 255, 255, 0.04)',
                      border: '1px solid rgba(255, 255, 255, 0.08)',
                      borderRadius: '6px',
                      padding: '4px 10px',
                      fontSize: '11px',
                      color: '#94a3b8',
                      cursor: 'pointer',
                    }}
                  >
                    💬 Buka Percakapan Penuh
                  </button>

                  {onOpenWorkstation && (
                    <button
                      onClick={onOpenWorkstation}
                      className="btn-pressable"
                      style={{
                        backgroundColor: 'rgba(255, 255, 255, 0.04)',
                        border: '1px solid rgba(255, 255, 255, 0.08)',
                        borderRadius: '6px',
                        padding: '4px 10px',
                        fontSize: '11px',
                        color: '#94a3b8',
                        cursor: 'pointer',
                      }}
                    >
                      ⛶ Buka Workstation Lengkap
                    </button>
                  )}

                  <button
                    onClick={onOpenOmnibox}
                    className="btn-pressable"
                    style={{
                      backgroundColor: 'rgba(255, 255, 255, 0.04)',
                      border: '1px solid rgba(255, 255, 255, 0.08)',
                      borderRadius: '6px',
                      padding: '4px 10px',
                      fontSize: '11px',
                      color: '#94a3b8',
                      cursor: 'pointer',
                    }}
                  >
                    ⚡ Cari Perintah (Ctrl+K)
                  </button>
                </div>
              </div>
            )}
          </>
        )}
      </div>
    </div>
  );
};
