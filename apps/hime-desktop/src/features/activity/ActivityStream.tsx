import React, { useState } from 'react';
import { Card } from '../../components/ui/Card';
import { Badge } from '../../components/ui/Badge';
import { Button } from '../../components/ui/Button';
import { AuditEvent, EventCategory } from '../../types/events';

export interface ActivityStreamProps {
  events: AuditEvent[];
  isLoading: boolean;
  onRefresh: () => void;
}

export const ActivityStream: React.FC<ActivityStreamProps> = ({
  events,
  isLoading,
  onRefresh,
}) => {
  const [filter, setFilter] = useState<EventCategory>('all');
  const [expandedId, setExpandedId] = useState<number | null>(null);

  const filteredEvents = events.filter((ev) => {
    if (filter === 'all') return true;
    if (filter === 'tool') return ev.event_type.startsWith('tool.');
    if (filter === 'task') return ev.event_type.startsWith('task.');
    if (filter === 'core') return ev.event_type.startsWith('core.') || ev.event_type.startsWith('system.');
    if (filter === 'policy') return ev.event_type.includes('policy');
    return true;
  });

  const getEventBadge = (eventType: string) => {
    if (eventType.includes('policy') || eventType.includes('security')) {
      return <Badge variant="warning">Policy</Badge>;
    }
    if (eventType.startsWith('tool.')) {
      return <Badge variant="cyan">Tool</Badge>;
    }
    if (eventType.startsWith('task.')) {
      return <Badge variant="violet">Task</Badge>;
    }
    return <Badge variant="default">System</Badge>;
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 'var(--space-4)' }}>
      {/* Top Header & Actions */}
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <div>
          <h1 style={{ fontSize: 'var(--text-xl)', fontWeight: 600 }}>
            Agent Activity Stream
          </h1>
          <p style={{ fontSize: 'var(--text-sm)', color: 'var(--text-secondary)', marginTop: '2px' }}>
            Audited system events, tool invocations, and policy evaluations persisted in SQLite WAL.
          </p>
        </div>
        <Button
          variant="secondary"
          size="sm"
          isLoading={isLoading}
          onClick={onRefresh}
        >
          Refresh Events
        </Button>
      </div>

      {/* Filter Tabs */}
      <div style={{ display: 'flex', gap: '8px', borderBottom: '1px solid var(--border-subtle)', paddingBottom: '8px' }}>
        {(['all', 'tool', 'task', 'policy', 'core'] as EventCategory[]).map((cat) => (
          <button
            key={cat}
            onClick={() => setFilter(cat)}
            className="btn-pressable"
            style={{
              padding: '4px 12px',
              borderRadius: 'var(--radius-pill)',
              backgroundColor: filter === cat ? 'var(--accent-primary-subtle)' : 'transparent',
              color: filter === cat ? 'var(--accent-primary)' : 'var(--text-secondary)',
              border: filter === cat ? '1px solid rgba(139, 92, 246, 0.3)' : '1px solid transparent',
              fontSize: 'var(--text-xs)',
              fontWeight: 500,
              cursor: 'pointer',
              textTransform: 'capitalize',
            }}
          >
            {cat}
          </button>
        ))}
      </div>

      {/* Events Feed */}
      <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
        {filteredEvents.length === 0 ? (
          <Card variant="surface-2" style={{ textAlign: 'center', padding: '36px 16px' }}>
            <div style={{ color: 'var(--text-muted)', fontSize: 'var(--text-sm)' }}>
              No audit events found for category "{filter}".
            </div>
            <p style={{ color: 'var(--text-dim)', fontSize: 'var(--text-xs)', marginTop: '4px' }}>
              Execute tools or create tasks to generate live audit records.
            </p>
          </Card>
        ) : (
          filteredEvents.map((ev) => {
            const isExpanded = expandedId === ev.id;
            const dateStr = new Date(ev.timestamp * 1000).toLocaleString();

            return (
              <Card
                key={ev.id}
                variant="surface-2"
                style={{
                  cursor: 'pointer',
                  borderLeft: ev.event_type.includes('policy')
                    ? '3px solid var(--accent-warning)'
                    : '3px solid var(--border-medium)',
                }}
                onClick={() => setExpandedId(isExpanded ? null : ev.id)}
              >
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                  <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                    {getEventBadge(ev.event_type)}
                    <span style={{ fontWeight: 600, fontSize: 'var(--text-sm)' }}>
                      {ev.event_type}
                    </span>
                    <span style={{ fontSize: 'var(--text-xs)', color: 'var(--text-muted)' }}>
                      source: {ev.source}
                    </span>
                  </div>
                  <div style={{ fontSize: 'var(--text-xs)', color: 'var(--text-muted)' }}>
                    {dateStr}
                  </div>
                </div>

                {/* Short preview of payload */}
                <div
                  style={{
                    marginTop: '8px',
                    fontSize: 'var(--text-xs)',
                    color: 'var(--text-secondary)',
                    fontFamily: 'var(--font-mono)',
                    overflow: 'hidden',
                    textOverflow: 'ellipsis',
                    whiteSpace: isExpanded ? 'pre-wrap' : 'nowrap',
                    maxHeight: isExpanded ? 'none' : '20px',
                  }}
                >
                  {ev.payload}
                </div>

                {isExpanded && (
                  <div
                    className="animate-fade-in"
                    style={{
                      marginTop: '10px',
                      padding: '8px 12px',
                      backgroundColor: 'var(--bg-surface-3)',
                      borderRadius: 'var(--radius-sm)',
                      fontSize: 'var(--text-2xs)',
                      fontFamily: 'var(--font-mono)',
                      color: 'var(--text-primary)',
                      border: '1px solid var(--border-subtle)',
                    }}
                  >
                    <div>Event ID: #{ev.id}</div>
                    <div>Timestamp (Epoch): {ev.timestamp}</div>
                    <div>Source Node: {ev.source}</div>
                    <div style={{ marginTop: '4px' }}>Payload Raw:</div>
                    <pre style={{ marginTop: '4px', whiteSpace: 'pre-wrap', wordBreak: 'break-all' }}>
                      {ev.payload}
                    </pre>
                  </div>
                )}
              </Card>
            );
          })
        )}
      </div>
    </div>
  );
};

