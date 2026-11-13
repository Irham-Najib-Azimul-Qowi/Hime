import React, { useState } from 'react';
import { Card } from '../../components/ui/Card';
import { Badge } from '../../components/ui/Badge';
import { Button } from '../../components/ui/Button';
import { NotificationItem, NotificationCategory } from '../../types/notifications';

export interface NotificationCenterProps {
  notifications: NotificationItem[];
  onDismiss: (id: string) => void;
  onMarkAllRead: () => void;
  onApproveAction?: (id: string) => void;
  onDenyAction?: (id: string) => void;
}

export const NotificationCenter: React.FC<NotificationCenterProps> = ({
  notifications,
  onDismiss,
  onMarkAllRead,
  onApproveAction,
  onDenyAction,
}) => {
  const [filterCategory, setFilterCategory] = useState<string>('all');

  const filtered = notifications.filter((n) => {
    if (filterCategory === 'all') return true;
    return n.category === filterCategory;
  });

  const getCategoryBadge = (cat: NotificationCategory) => {
    switch (cat) {
      case 'permission':
        return <Badge variant="warning">Permission Request</Badge>;
      case 'error':
        return <Badge variant="danger">System Alert</Badge>;
      case 'task':
        return <Badge variant="violet">Task</Badge>;
      case 'agent':
        return <Badge variant="cyan">Agent</Badge>;
      case 'system':
      default:
        return <Badge variant="default">System</Badge>;
    }
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 'var(--space-4)' }}>
      {/* Header and Bulk Actions */}
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <div>
          <h1 style={{ fontSize: 'var(--text-xl)', fontWeight: 600 }}>
            Notification Center
          </h1>
          <p style={{ fontSize: 'var(--text-sm)', color: 'var(--text-secondary)', marginTop: '2px' }}>
            Security policy approval requests, agent milestones, and operational alerts.
          </p>
        </div>
        {notifications.length > 0 && (
          <Button variant="secondary" size="sm" onClick={onMarkAllRead}>
            Mark All as Read
          </Button>
        )}
      </div>

      {/* Category Filter Pills */}
      <div style={{ display: 'flex', gap: '8px', borderBottom: '1px solid var(--border-subtle)', paddingBottom: '8px' }}>
        {['all', 'permission', 'task', 'system', 'error'].map((cat) => (
          <button
            key={cat}
            onClick={() => setFilterCategory(cat)}
            className="btn-pressable"
            style={{
              padding: '4px 12px',
              borderRadius: 'var(--radius-pill)',
              backgroundColor: filterCategory === cat ? 'var(--accent-primary-subtle)' : 'transparent',
              color: filterCategory === cat ? 'var(--accent-primary)' : 'var(--text-secondary)',
              border: filterCategory === cat ? '1px solid rgba(139, 92, 246, 0.3)' : '1px solid transparent',
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

      {/* Notifications Feed */}
      <div style={{ display: 'flex', flexDirection: 'column', gap: '10px' }}>
        {filtered.length === 0 ? (
          <Card variant="surface-2" style={{ textAlign: 'center', padding: '36px 16px' }}>
            <div style={{ color: 'var(--text-muted)', fontSize: 'var(--text-sm)' }}>
              No notifications matching "{filterCategory}".
            </div>
            <p style={{ color: 'var(--text-dim)', fontSize: 'var(--text-xs)', marginTop: '4px' }}>
              All pending policy approvals and system notices have been cleared.
            </p>
          </Card>
        ) : (
          filtered.map((item) => {
            const dateStr = new Date(item.timestamp).toLocaleTimeString([], {
              hour: '2-digit',
              minute: '2-digit',
            });

            return (
              <Card
                key={item.id}
                variant="surface-2"
                style={{
                  borderLeft:
                    item.category === 'permission'
                      ? '4px solid var(--accent-warning)'
                      : item.category === 'error'
                      ? '4px solid var(--accent-danger)'
                      : '4px solid var(--border-medium)',
                  display: 'flex',
                  flexDirection: 'column',
                  gap: '8px',
                }}
              >
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                  <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                    {getCategoryBadge(item.category)}
                    <span style={{ fontWeight: 600, fontSize: 'var(--text-sm)' }}>
                      {item.title}
                    </span>
                    {!item.read && <Badge variant="cyan" size="xs">New</Badge>}
                  </div>
                  <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                    <span style={{ fontSize: 'var(--text-xs)', color: 'var(--text-muted)' }}>
                      {dateStr}
                    </span>
                    <button
                      onClick={() => onDismiss(item.id)}
                      className="btn-pressable"
                      style={{
                        background: 'transparent',
                        border: 'none',
                        color: 'var(--text-muted)',
                        cursor: 'pointer',
                        padding: '2px 6px',
                        fontSize: 'var(--text-xs)',
                      }}
                      title="Dismiss"
                    >
                      ✕
                    </button>
                  </div>
                </div>

                <div style={{ fontSize: 'var(--text-xs)', color: 'var(--text-secondary)', lineHeight: 1.5 }}>
                  {item.message}
                </div>

                {/* If it is an interactive permission approval request */}
                {item.actionRequired && item.actionType === 'approval' && (
                  <div
                    style={{
                      marginTop: '8px',
                      display: 'flex',
                      gap: '8px',
                      paddingTop: '8px',
                      borderTop: '1px solid var(--border-subtle)',
                    }}
                  >
                    <Button
                      variant="primary"
                      size="sm"
                      onClick={() => {
                        const reqId = (item.contextData?.requestId as string) || item.id;
                        onApproveAction?.(reqId);
                        onDismiss(item.id);
                      }}
                    >
                      Approve Execution
                    </Button>
                    <Button
                      variant="danger"
                      size="sm"
                      onClick={() => {
                        const reqId = (item.contextData?.requestId as string) || item.id;
                        onDenyAction?.(reqId);
                        onDismiss(item.id);
                      }}
                    >
                      Deny Request
                    </Button>
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

