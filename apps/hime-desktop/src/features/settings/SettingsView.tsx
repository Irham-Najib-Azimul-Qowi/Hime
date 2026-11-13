import React from 'react';
import { Card } from '../../components/ui/Card';
import { Switch } from '../../components/ui/Switch';
import { Badge } from '../../components/ui/Badge';
import { Button } from '../../components/ui/Button';
import { ThemeMode, IslandPresentationMode } from '../../types/theme';

export interface SettingsViewProps {
  theme: ThemeMode;
  onToggleTheme: () => void;
  islandMode: IslandPresentationMode;
  onToggleIslandMode: (mode: IslandPresentationMode) => void;
  alwaysOnTop: boolean;
  onToggleAlwaysOnTop: (enabled: boolean) => void;
  onToggleWindowSize: (mode: 'compact' | 'expanded') => void;
  onOpenOmnibox: () => void;
}

export const SettingsView: React.FC<SettingsViewProps> = ({
  theme,
  onToggleTheme,
  islandMode,
  onToggleIslandMode,
  alwaysOnTop,
  onToggleAlwaysOnTop,
  onToggleWindowSize,
  onOpenOmnibox,
}) => {
  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 'var(--space-5)' }}>
      {/* Header */}
      <div>
        <h1 style={{ fontSize: 'var(--text-xl)', fontWeight: 600 }}>
          Desktop Settings & Preferences
        </h1>
        <p style={{ fontSize: 'var(--text-sm)', color: 'var(--text-secondary)', marginTop: '2px' }}>
          Configure appearance, windowing behavior, and inspect runtime capabilities.
        </p>
      </div>

      {/* Appearance & Theme Section */}
      <Card variant="surface-1">
        <h3 style={{ fontSize: 'var(--text-md)', fontWeight: 600, marginBottom: '4px' }}>
          Appearance & Themes
        </h3>
        <p style={{ fontSize: 'var(--text-xs)', color: 'var(--text-secondary)', marginBottom: '16px' }}>
          Select your visual environment theme and presentation style.
        </p>

        <div style={{ display: 'flex', flexDirection: 'column', gap: '14px' }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
            <div>
              <div style={{ fontWeight: 500, fontSize: 'var(--text-sm)' }}>Dark Mode Theme</div>
              <div style={{ fontSize: 'var(--text-xs)', color: 'var(--text-muted)' }}>
                Obsidian slate dark palette with high-contrast semantic accents
              </div>
            </div>
            <Switch
              checked={theme === 'dark'}
              onChange={() => onToggleTheme()}
              label={theme === 'dark' ? 'Dark' : 'Light'}
            />
          </div>

          <div
            style={{
              display: 'flex',
              justifyContent: 'space-between',
              alignItems: 'center',
              paddingTop: '12px',
              borderTop: '1px solid var(--border-subtle)',
            }}
          >
            <div>
              <div style={{ fontWeight: 500, fontSize: 'var(--text-sm)' }}>Dynamic Island Mode</div>
              <div style={{ fontSize: 'var(--text-xs)', color: 'var(--text-muted)' }}>
                Integrated desktop header bar vs floating companion pill
              </div>
            </div>
            <div style={{ display: 'flex', gap: '8px' }}>
              <Button
                variant={islandMode === 'integrated' ? 'primary' : 'secondary'}
                size="sm"
                onClick={() => onToggleIslandMode('integrated')}
              >
                Integrated
              </Button>
              <Button
                variant={islandMode === 'floating' ? 'primary' : 'secondary'}
                size="sm"
                onClick={() => onToggleIslandMode('floating')}
              >
                Floating Pill
              </Button>
            </div>
          </div>
        </div>
      </Card>

      {/* Window Behavior Section */}
      <Card variant="surface-1">
        <h3 style={{ fontSize: 'var(--text-md)', fontWeight: 600, marginBottom: '4px' }}>
          Windows Desktop Behavior
        </h3>
        <p style={{ fontSize: 'var(--text-xs)', color: 'var(--text-secondary)', marginBottom: '16px' }}>
          Control Tauri 2 native window properties and size modes.
        </p>

        <div style={{ display: 'flex', flexDirection: 'column', gap: '14px' }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
            <div>
              <div style={{ fontWeight: 500, fontSize: 'var(--text-sm)' }}>Always On Top</div>
              <div style={{ fontSize: 'var(--text-xs)', color: 'var(--text-muted)' }}>
                Keep Hime Desktop window above other active desktop applications
              </div>
            </div>
            <Switch
              checked={alwaysOnTop}
              onChange={(val) => onToggleAlwaysOnTop(val)}
            />
          </div>

          <div
            style={{
              display: 'flex',
              justifyContent: 'space-between',
              alignItems: 'center',
              paddingTop: '12px',
              borderTop: '1px solid var(--border-subtle)',
            }}
          >
            <div>
              <div style={{ fontWeight: 500, fontSize: 'var(--text-sm)' }}>Window Geometry Preset</div>
              <div style={{ fontSize: 'var(--text-xs)', color: 'var(--text-muted)' }}>
                Quick switch window dimensions (Floating HUD 460x88 vs Full Dashboard 980x700)
              </div>
            </div>
            <div style={{ display: 'flex', gap: '8px' }}>
              <Button
                variant="secondary"
                size="sm"
                onClick={() => onToggleWindowSize('compact')}
              >
                Compact HUD
              </Button>
              <Button
                variant="primary"
                size="sm"
                onClick={() => onToggleWindowSize('expanded')}
              >
                Full Dashboard
              </Button>
            </div>
          </div>
        </div>
      </Card>

      {/* Keyboard Shortcuts Reference */}
      <Card variant="surface-1">
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '12px' }}>
          <div>
            <h3 style={{ fontSize: 'var(--text-md)', fontWeight: 600 }}>Keyboard Navigation</h3>
            <p style={{ fontSize: 'var(--text-xs)', color: 'var(--text-secondary)' }}>
              Universal hotkeys for rapid desktop interaction.
            </p>
          </div>
          <Button variant="subtle" size="sm" onClick={onOpenOmnibox}>
            Open Omnibox (Ctrl+K)
          </Button>
        </div>

        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))', gap: '8px' }}>
          {[
            { key: 'Ctrl + K', desc: 'Universal Omnibox' },
            { key: 'Ctrl + P', desc: 'Ping Core IPC' },
            { key: 'Ctrl + T', desc: 'Toggle Dark / Light' },
            { key: 'Ctrl + M', desc: 'Toggle Window Size' },
            { key: 'Alt + 1..6', desc: 'Direct View Navigation' },
            { key: 'Escape', desc: 'Dismiss Modal / Island' },
          ].map((item) => (
            <div
              key={item.key}
              style={{
                display: 'flex',
                justifyContent: 'space-between',
                padding: '8px 12px',
                backgroundColor: 'var(--bg-surface-2)',
                borderRadius: 'var(--radius-sm)',
                fontSize: 'var(--text-xs)',
              }}
            >
              <span style={{ color: 'var(--text-secondary)' }}>{item.desc}</span>
              <kbd
                style={{
                  fontFamily: 'var(--font-mono)',
                  backgroundColor: 'var(--bg-surface-3)',
                  padding: '2px 6px',
                  borderRadius: 'var(--radius-xs)',
                  border: '1px solid var(--border-subtle)',
                  color: 'var(--text-primary)',
                  fontWeight: 600,
                }}
              >
                {item.key}
              </kbd>
            </div>
          ))}
        </div>
      </Card>

      {/* System & Toolchain Info */}
      <Card variant="surface-1">
        <h3 style={{ fontSize: 'var(--text-md)', fontWeight: 600, marginBottom: '8px' }}>
          Host Environment & Architecture Status
        </h3>
        <div style={{ display: 'flex', flexDirection: 'column', gap: '8px', fontSize: 'var(--text-xs)' }}>
          <div style={{ display: 'flex', justifyContent: 'space-between' }}>
            <span style={{ color: 'var(--text-secondary)' }}>Host Platform</span>
            <span>Windows 11 Pro (x86_64) • MSVC v143 • Windows 11 SDK</span>
          </div>
          <div style={{ display: 'flex', justifyContent: 'space-between' }}>
            <span style={{ color: 'var(--text-secondary)' }}>Native Runtimes</span>
            <span>rustc 1.99.1 • Node.js v24.18.0 • pnpm v12.14.0 • Python 3.13</span>
          </div>
          <div style={{ display: 'flex', justifyContent: 'space-between' }}>
            <span style={{ color: 'var(--text-secondary)' }}>Process Supervisor</span>
            <Badge variant="success">Win32 Job Object (Operational Orphan Prevention)</Badge>
          </div>
          <div style={{ display: 'flex', justifyContent: 'space-between' }}>
            <span style={{ color: 'var(--text-secondary)' }}>DPAPI Secret Vault</span>
            <Badge variant="success">Active (Localhost Protected)</Badge>
          </div>
          <div style={{ display: 'flex', justifyContent: 'space-between' }}>
            <span style={{ color: 'var(--text-secondary)' }}>IB-01 & IB-02 Governance</span>
            <Badge variant="warning">OPEN IMPLEMENTATION BLOCKERS</Badge>
          </div>
        </div>
      </Card>
    </div>
  );
};

