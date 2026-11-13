import React, { useState } from 'react';
import { Card } from '../../components/ui/Card';
import { Badge } from '../../components/ui/Badge';
import { Button } from '../../components/ui/Button';
import { Modal } from '../../components/ui/Modal';
import { Input } from '../../components/ui/Input';
import { HimeTask, TaskPriority } from '../../types/tasks';

export interface TasksViewProps {
  tasks: HimeTask[];
  isLoading: boolean;
  onCreateTask: (name: string, description: string, priority: TaskPriority) => Promise<void>;
  onCancelTask: (id: string) => Promise<void>;
  onRefresh: () => void;
}

export const TasksView: React.FC<TasksViewProps> = ({
  tasks,
  isLoading,
  onCreateTask,
  onCancelTask,
  onRefresh,
}) => {
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [name, setName] = useState('');
  const [description, setDescription] = useState('');
  const [priority, setPriority] = useState<TaskPriority>('normal');
  const [isSubmitting, setIsSubmitting] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!name.trim()) return;

    setIsSubmitting(true);
    try {
      await onCreateTask(name.trim(), description.trim(), priority);
      setName('');
      setDescription('');
      setPriority('normal');
      setIsModalOpen(false);
    } finally {
      setIsSubmitting(false);
    }
  };

  const getPriorityBadge = (p: TaskPriority) => {
    switch (p) {
      case 'urgent':
        return <Badge variant="danger">Urgent</Badge>;
      case 'high':
        return <Badge variant="warning">High</Badge>;
      case 'low':
        return <Badge variant="default">Low</Badge>;
      case 'normal':
      default:
        return <Badge variant="cyan">Normal</Badge>;
    }
  };

  const getStatusBadge = (status: string) => {
    switch (status) {
      case 'running':
        return <Badge variant="cyan" dot>Running</Badge>;
      case 'completed':
        return <Badge variant="success">Completed</Badge>;
      case 'cancelled':
        return <Badge variant="default">Cancelled</Badge>;
      case 'failed':
        return <Badge variant="danger">Failed</Badge>;
      case 'queued':
      default:
        return <Badge variant="violet" dot>Queued</Badge>;
    }
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 'var(--space-4)' }}>
      {/* Header and New Task Button */}
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <div>
          <h1 style={{ fontSize: 'var(--text-xl)', fontWeight: 600 }}>
            Task Management
          </h1>
          <p style={{ fontSize: 'var(--text-sm)', color: 'var(--text-secondary)', marginTop: '2px' }}>
            Workflows and processes orchestrated by Hime Core Process Supervisor.
          </p>
        </div>
        <div style={{ display: 'flex', gap: '8px' }}>
          <Button variant="secondary" size="sm" onClick={onRefresh} isLoading={isLoading}>
            Refresh
          </Button>
          <Button variant="primary" size="sm" onClick={() => setIsModalOpen(true)}>
            + Create Task
          </Button>
        </div>
      </div>

      {/* Task List */}
      <div style={{ display: 'flex', flexDirection: 'column', gap: '10px' }}>
        {tasks.length === 0 ? (
          <Card variant="surface-2" style={{ textAlign: 'center', padding: '36px 16px' }}>
            <div style={{ color: 'var(--text-muted)', fontSize: 'var(--text-sm)' }}>
              No tasks currently registered.
            </div>
            <p style={{ color: 'var(--text-dim)', fontSize: 'var(--text-xs)', marginTop: '4px' }}>
              Click "+ Create Task" to register a new supervised task in Hime Core.
            </p>
          </Card>
        ) : (
          tasks.map((task) => {
            const dateStr = new Date(task.created_at * 1000).toLocaleString();
            return (
              <Card
                key={task.id}
                variant="surface-2"
                style={{
                  display: 'flex',
                  justifyContent: 'space-between',
                  alignItems: 'center',
                }}
              >
                <div style={{ display: 'flex', flexDirection: 'column', gap: '4px' }}>
                  <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                    <span style={{ fontWeight: 600, fontSize: 'var(--text-md)' }}>
                      {task.name}
                    </span>
                    {getPriorityBadge(task.priority)}
                    {getStatusBadge(task.status)}
                  </div>
                  {task.description && (
                    <div style={{ fontSize: 'var(--text-xs)', color: 'var(--text-secondary)' }}>
                      {task.description}
                    </div>
                  )}
                  <div style={{ fontSize: 'var(--text-2xs)', color: 'var(--text-muted)', marginTop: '2px' }}>
                    ID: {task.id} • Created: {dateStr}
                  </div>
                </div>

                {task.status !== 'cancelled' && task.status !== 'completed' && (
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => onCancelTask(task.id)}
                    style={{ color: 'var(--accent-danger)' }}
                  >
                    Cancel
                  </Button>
                )}
              </Card>
            );
          })
        )}
      </div>

      {/* Create Task Modal */}
      <Modal
        isOpen={isModalOpen}
        onClose={() => setIsModalOpen(false)}
        title="Create New Supervised Task"
      >
        <form onSubmit={handleSubmit} style={{ display: 'flex', flexDirection: 'column', gap: '14px' }}>
          <div>
            <label style={{ display: 'block', fontSize: 'var(--text-xs)', color: 'var(--text-secondary)', marginBottom: '4px' }}>
              Task Name
            </label>
            <Input
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="e.g. Inspect Project Directories"
              required
            />
          </div>

          <div>
            <label style={{ display: 'block', fontSize: 'var(--text-xs)', color: 'var(--text-secondary)', marginBottom: '4px' }}>
              Description
            </label>
            <Input
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              placeholder="Detailed description of task intent"
            />
          </div>

          <div>
            <label style={{ display: 'block', fontSize: 'var(--text-xs)', color: 'var(--text-secondary)', marginBottom: '4px' }}>
              Priority Level
            </label>
            <select
              value={priority}
              onChange={(e) => setPriority(e.target.value as TaskPriority)}
              style={{
                width: '100%',
                padding: '8px 12px',
                backgroundColor: 'var(--bg-surface-2)',
                border: '1px solid var(--border-medium)',
                borderRadius: 'var(--radius-sm)',
                color: 'var(--text-primary)',
                fontFamily: 'inherit',
                fontSize: 'var(--text-sm)',
                outline: 'none',
              }}
            >
              <option value="low">Low</option>
              <option value="normal">Normal</option>
              <option value="high">High</option>
              <option value="urgent">Urgent</option>
            </select>
          </div>

          <div style={{ display: 'flex', justifyContent: 'flex-end', gap: '8px', marginTop: '10px' }}>
            <Button
              type="button"
              variant="secondary"
              size="sm"
              onClick={() => setIsModalOpen(false)}
            >
              Cancel
            </Button>
            <Button
              type="submit"
              variant="primary"
              size="sm"
              isLoading={isSubmitting}
            >
              Submit Task
            </Button>
          </div>
        </form>
      </Modal>
    </div>
  );
};

