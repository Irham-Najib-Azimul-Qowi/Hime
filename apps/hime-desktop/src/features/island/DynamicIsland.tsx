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
  onTriggerPing: () => void;
  onTriggerPolicyTest: () => void;
  unreadCount: number;
}

export const DynamicIsland: React.FC<DynamicIslandProps> = ({
  status,
  isCoreOnline,
  activeActivity,
  translationResult,
  onClearTranslation,
  onOpenOmnibox,
  onTriggerPing,
  onTriggerPolicyTest,
  unreadCount,
}) => {
  const [isExpanded, setIsExpanded] = useState(false);

  // If there is active tool/agent activity or translation, show state
  const isActivityState = Boolean(activeActivity);
  const hasTranslation = Boolean(translationResult);

  return (
    <div
      style={{
        display: 'flex',
        justifyContent: 'center',
        padding: '8px 16px',
        position: 'relative',
        zIndex: 50,
      }}
    >
      <div
        className="app-no-drag"
        onClick={() => {
          if (!isActivityState && !hasTranslation) {
            setIsExpanded(!isExpanded);
          }
        }}
        style={{
          minWidth: hasTranslation ? '520px' : isExpanded ? '480px' : isActivityState ? '380px' : '310px',
          maxWidth: '640px',
          height: hasTranslation ? '120px' : isExpanded ? '110px' : '40px',
          backgroundColor: 'var(--island-bg)',
          borderRadius: (isExpanded || hasTranslation) ? 'var(--radius-xl)' : 'var(--radius-pill)',
          border: '1px solid var(--island-border)',
          boxShadow: 'var(--island-shadow)',
          display: 'flex',
          flexDirection: 'column',
          justifyContent: (isExpanded || hasTranslation) ? 'space-between' : 'center',
          padding: (isExpanded || hasTranslation) ? '12px 18px' : '0 14px',
          cursor: hasTranslation ? 'default' : 'pointer',
          transition:
            'min-width var(--duration-expand) var(--ease-spring), height var(--duration-expand) var(--ease-spring), border-radius var(--duration-expand) var(--ease-spring), background-color var(--duration-normal) ease-out',
          backdropFilter: 'blur(12px)',
          overflow: 'hidden',
        }}
      >
        {/* Translation Banner Mode */}
        {hasTranslation && translationResult ? (
          <div className="animate-fade-in" style={{ display: 'flex', flexDirection: 'column', height: '100%', justifyContent: 'space-between' }}>
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                <Badge variant="cyan" size="xs">TERJEMAHAN ID</Badge>
                <span style={{ fontSize: 'var(--text-xs)', color: 'var(--text-secondary)', textOverflow: 'ellipsis', overflow: 'hidden', whiteSpace: 'nowrap', maxWidth: '300px' }}>
                  "{translationResult.original}"
                </span>
              </div>
              <div style={{ display: 'flex', gap: '6px' }}>
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    navigator.clipboard.writeText(translationResult.translated);
                  }}
                  className="btn-pressable"
                  style={{
                    backgroundColor: 'var(--bg-surface-2)',
                    border: '1px solid var(--border-medium)',
                    borderRadius: 'var(--radius-xs)',
                    padding: '2px 8px',
                    fontSize: 'var(--text-2xs)',
                    cursor: 'pointer',
                  }}
                >
                  Salin
                </button>
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    onClearTranslation?.();
                  }}
                  className="btn-pressable"
                  style={{
                    backgroundColor: 'var(--bg-surface-2)',
                    border: '1px solid var(--border-medium)',
                    borderRadius: 'var(--radius-xs)',
                    padding: '2px 8px',
                    fontSize: 'var(--text-2xs)',
                    cursor: 'pointer',
                  }}
                >
                  ✕
                </button>
              </div>
            </div>
            <div style={{ fontSize: 'var(--text-sm)', fontWeight: 600, color: 'var(--text-primary)', marginTop: '4px' }}>
              {translationResult.translated}
            </div>
            <div style={{ fontSize: 'var(--text-2xs)', color: 'var(--text-muted)' }}>
              Island Intelligence • Privacy Opt-in Verified
            </div>
          </div>
        ) : (
          <>
            {/* Compact / Idle Bar */}
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
            width: '100%',
            height: '40px',
          }}
        >
          {/* Left: Status Dot & Identifier */}
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
            <span
              className={isCoreOnline ? 'animate-pulse-subtle' : ''}
              style={{
                width: '8px',
                height: '8px',
                borderRadius: '50%',
                backgroundColor: isCoreOnline
                  ? 'var(--accent-success)'
                  : 'var(--accent-danger)',
                display: 'inline-block',
                boxShadow: isCoreOnline
                  ? '0 0 8px var(--accent-success)'
                  : 'none',
              }}
            />
            <span
              style={{
                fontWeight: 600,
                fontSize: 'var(--text-xs)',
                letterSpacing: '0.04em',
                color: 'var(--text-primary)',
              }}
            >
              HIME
            </span>
            <Badge
              variant={isCoreOnline ? 'success' : 'danger'}
              size="xs"
            >
              {isCoreOnline ? 'ONLINE' : 'OFFLINE'}
            </Badge>
          </div>

          {/* Center: Activity or Search affordance */}
          <div
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: '6px',
              fontSize: 'var(--text-xs)',
              color: isActivityState ? 'var(--accent-cyan)' : 'var(--text-muted)',
              overflow: 'hidden',
              textOverflow: 'ellipsis',
              whiteSpace: 'nowrap',
              maxWidth: '180px',
            }}
          >
            {isActivityState ? (
              <>
                <span
                  className="animate-spin"
                  style={{
                    display: 'inline-block',
                    width: '10px',
                    height: '10px',
                    border: '2px solid var(--accent-cyan)',
                    borderRightColor: 'transparent',
                    borderRadius: '50%',
                  }}
                />
                <span>{activeActivity}</span>
              </>
            ) : (
              <span>Ctrl+K to command</span>
            )}
          </div>

          {/* Right: Quick actions and badge */}
          <div style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
            {unreadCount > 0 && (
              <Badge variant="warning" size="xs">
                {unreadCount}
              </Badge>
            )}
            <button
              onClick={(e) => {
                e.stopPropagation();
                onOpenOmnibox();
              }}
              className="btn-pressable"
              style={{
                backgroundColor: 'var(--bg-surface-3)',
                border: '1px solid var(--border-subtle)',
                borderRadius: 'var(--radius-pill)',
                padding: '3px 8px',
                fontSize: 'var(--text-2xs)',
                color: 'var(--text-secondary)',
                cursor: 'pointer',
                display: 'flex',
                alignItems: 'center',
                gap: '4px',
              }}
              title="Open Omnibox (Ctrl+K)"
            >
              ⌘K
            </button>
          </div>
        </div>

        {/* Expanded Controls Drawer */}
        {isExpanded && (
          <div
            className="animate-fade-in"
            style={{
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'space-between',
              paddingTop: '8px',
              borderTop: '1px solid var(--border-subtle)',
            }}
          >
            <div style={{ display: 'flex', gap: '8px' }}>
              <button
                onClick={(e) => {
                  e.stopPropagation();
                  onTriggerPing();
                }}
                className="btn-pressable"
                style={{
                  backgroundColor: 'var(--bg-surface-2)',
                  border: '1px solid var(--border-medium)',
                  borderRadius: 'var(--radius-sm)',
                  padding: '4px 10px',
                  fontSize: 'var(--text-xs)',
                  color: 'var(--text-primary)',
                  cursor: 'pointer',
                }}
              >
                Ping Core
              </button>
              <button
                onClick={(e) => {
                  e.stopPropagation();
                  onTriggerPolicyTest();
                }}
                className="btn-pressable"
                style={{
                  backgroundColor: 'var(--bg-surface-2)',
                  border: '1px solid var(--border-medium)',
                  borderRadius: 'var(--radius-sm)',
                  padding: '4px 10px',
                  fontSize: 'var(--text-xs)',
                  color: 'var(--text-primary)',
                  cursor: 'pointer',
                }}
              >
                Test Policy
              </button>
            </div>

            <div style={{ fontSize: 'var(--text-2xs)', color: 'var(--text-muted)' }}>
              Named Pipe: {status?.named_pipe || '\\\\.\\pipe\\hime-ipc'}
            </div>
          </div>
        )}
          </>
        )}
      </div>
    </div>
  );
};

