import React, { useState, useRef, useEffect } from 'react';
import { Card } from '../../components/ui/Card';
import { Badge } from '../../components/ui/Badge';
import { Button } from '../../components/ui/Button';
import {
  createConversation,
  listConversations,
  getConversationMessages,
  sendConversationMessage,
  cancelConversation,
} from '../../services/ipcService';
import { Conversation, ChatMessage as ApiChatMessage } from '../../types/ipc';

export interface UIConversationMessage {
  id: string;
  sender: 'user' | 'assistant' | 'system';
  content: string;
  timestamp: string;
  status?: 'delivered' | 'streaming' | 'completed' | 'error';
  badge?: string;
}

export interface ConversationViewProps {
  isCoreOnline: boolean;
  onSendSystemCommand: (cmd: string) => Promise<string>;
}

export const ConversationView: React.FC<ConversationViewProps> = ({
  isCoreOnline,
}) => {
  const [conversations, setConversations] = useState<Conversation[]>([]);
  const [activeConvId, setActiveConvId] = useState<string>('');
  const [messages, setMessages] = useState<UIConversationMessage[]>([
    {
      id: 'msg-init-1',
      sender: 'system',
      content:
        'Hime Assistant AI Runtime initialized. Hermes Agent is active and orchestrated via Hime Core. Model requests are routed through Hime-Owned Loopback Provider Adapter with DPAPI vault protection (IB-01).',
      timestamp: new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }),
      badge: 'Hermes AI Ready',
    },
  ]);
  const [input, setInput] = useState('');
  const [isGenerating, setIsGenerating] = useState(false);
  const [agentStatus, setAgentStatus] = useState<string>('idle');
  const messagesEndRef = useRef<HTMLDivElement>(null);

  // Initialize conversations list
  useEffect(() => {
    const initConversations = async () => {
      try {
        const list = await listConversations();
        if (list.length > 0) {
          setConversations(list);
          setActiveConvId(list[0].id);
          loadMessages(list[0].id);
        } else {
          // Create default conversation thread
          const newConv = await createConversation('Diskusi Utama Hime', 'hermes-3-llama-3.1-8b');
          setConversations([newConv]);
          setActiveConvId(newConv.id);
        }
      } catch {
        // Fallback for preview mode
      }
    };
    initConversations();
  }, [isCoreOnline]);

  const loadMessages = async (convId: string) => {
    try {
      const msgs = await getConversationMessages(convId);
      if (msgs.length > 0) {
        setMessages(
          msgs.map((m: ApiChatMessage) => ({
            id: m.id,
            sender: m.role === 'user' ? 'user' : 'assistant',
            content: m.content,
            timestamp: new Date(m.created_at * 1000).toLocaleTimeString([], {
              hour: '2-digit',
              minute: '2-digit',
            }),
            status: 'completed',
          }))
        );
      }
    } catch {
      // Keep existing
    }
  };

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  };

  useEffect(() => {
    scrollToBottom();
  }, [messages]);

  const handleSendPrompt = async (text: string) => {
    const trimmed = text.trim();
    if (!trimmed || isGenerating) return;

    let targetConvId = activeConvId;
    if (!targetConvId) {
      try {
        const conv = await createConversation('Sesi Percakapan Baru', 'hermes-3-llama-3.1-8b');
        targetConvId = conv.id;
        setActiveConvId(conv.id);
        setConversations((prev) => [conv, ...prev]);
      } catch {
        targetConvId = 'conv-fallback';
      }
    }

    const timeStr = new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
    const userMsgId = `user-${Date.now()}`;
    const userMsg: UIConversationMessage = {
      id: userMsgId,
      sender: 'user',
      content: trimmed,
      timestamp: timeStr,
      status: 'delivered',
    };

    setMessages((prev) => [...prev, userMsg]);
    setInput('');
    setIsGenerating(true);
    setAgentStatus('reasoning');

    // Add placeholder assistant message
    const assistantMsgId = `asst-${Date.now()}`;
    const initialAsstMsg: UIConversationMessage = {
      id: assistantMsgId,
      sender: 'assistant',
      content: 'Thinking...',
      timestamp: timeStr,
      status: 'streaming',
      badge: 'Hermes 3',
    };
    setMessages((prev) => [...prev, initialAsstMsg]);

    try {
      setAgentStatus('streaming');
      const result = await sendConversationMessage(targetConvId, trimmed);

      setMessages((prev) =>
        prev.map((m) =>
          m.id === assistantMsgId
            ? {
                ...m,
                content: result.response,
                status: 'completed',
              }
            : m
        )
      );
      setAgentStatus('idle');
    } catch (err: unknown) {
      setMessages((prev) =>
        prev.map((m) =>
          m.id === assistantMsgId
            ? {
                ...m,
                content: `Error: ${err instanceof Error ? err.message : String(err)}`,
                status: 'error',
                badge: 'AI Error',
              }
            : m
        )
      );
      setAgentStatus('idle');
    } finally {
      setIsGenerating(false);
    }
  };

  const handleCancel = async () => {
    try {
      await cancelConversation();
      setAgentStatus('idle');
      setIsGenerating(false);
      setMessages((prev) => [
        ...prev,
        {
          id: `cancel-${Date.now()}`,
          sender: 'system',
          content: 'Generation was cancelled by user.',
          timestamp: new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }),
          badge: 'Cancelled',
        },
      ]);
    } catch {
      // Ignore
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSendPrompt(input);
    }
  };

  const handleNewConversation = async () => {
    try {
      const conv = await createConversation(
        `Percakapan ${conversations.length + 1}`,
        'hermes-3-llama-3.1-8b'
      );
      setConversations((prev) => [conv, ...prev]);
      setActiveConvId(conv.id);
      setMessages([
        {
          id: `msg-${Date.now()}`,
          sender: 'system',
          content: `New conversation started: "${conv.title}". Hermes Agent is ready.`,
          timestamp: new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }),
          badge: 'Thread Created',
        },
      ]);
    } catch (e) {
      console.error(e);
    }
  };

  return (
    <div
      style={{
        display: 'flex',
        flexDirection: 'column',
        height: '100%',
        maxHeight: 'calc(100vh - 120px)',
        gap: 'var(--space-3)',
      }}
    >
      {/* AI Runtime Active Banner */}
      <Card
        variant="surface-2"
        style={{
          borderLeft: '4px solid var(--accent-emerald)',
          padding: '10px 16px',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
        }}
      >
        <div style={{ display: 'flex', alignItems: 'center', gap: '10px' }}>
          <span style={{ fontSize: 'var(--text-md)' }}>🤖</span>
          <div>
            <div style={{ fontWeight: 600, fontSize: 'var(--text-xs)', color: 'var(--text-primary)' }}>
              Hermes Agent AI Runtime: ACTIVE (Phase 08 End-to-End Orchestration)
            </div>
            <div style={{ fontSize: 'var(--text-2xs)', color: 'var(--text-muted)' }}>
              Model: Nous Hermes 3 (Llama 3.1 8B) • Gateway: Hime Loopback Adapter (DPAPI Protected) • SEC-09 CoT Redacted
            </div>
          </div>
        </div>
        <div style={{ display: 'flex', gap: '6px', alignItems: 'center' }}>
          <Badge variant={agentStatus === 'idle' ? 'success' : 'violet'} dot>
            {agentStatus === 'idle'
              ? 'Agent Ready'
              : agentStatus === 'reasoning'
              ? 'Reasoning...'
              : 'Streaming Response...'}
          </Badge>
          <Button variant="ghost" size="sm" onClick={handleNewConversation}>
            + New Thread
          </Button>
        </div>
      </Card>

      {/* Message List */}
      <div
        style={{
          flex: 1,
          overflowY: 'auto',
          display: 'flex',
          flexDirection: 'column',
          gap: 'var(--space-3)',
          padding: 'var(--space-3)',
          backgroundColor: 'var(--bg-surface-1)',
          borderRadius: 'var(--radius-md)',
          border: '1px solid var(--border-subtle)',
        }}
      >
        {messages.map((msg) => {
          const isUser = msg.sender === 'user';
          const isSystem = msg.sender === 'system';
          return (
            <div
              key={msg.id}
              className="animate-slide-down"
              style={{
                display: 'flex',
                flexDirection: 'column',
                alignItems: isUser ? 'flex-end' : 'flex-start',
                width: '100%',
              }}
            >
              <div
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: '6px',
                  marginBottom: '4px',
                  fontSize: 'var(--text-2xs)',
                  color: 'var(--text-muted)',
                }}
              >
                <span>{isUser ? 'You' : isSystem ? 'Hime System' : 'Hermes Agent'}</span>
                <span>•</span>
                <span>{msg.timestamp}</span>
                {msg.badge && (
                  <Badge variant={isUser ? 'default' : isSystem ? 'cyan' : 'success'} size="xs">
                    {msg.badge}
                  </Badge>
                )}
              </div>

              <div
                style={{
                  maxWidth: '80%',
                  padding: '10px 14px',
                  borderRadius: 'var(--radius-md)',
                  backgroundColor: isUser
                    ? 'var(--accent-primary)'
                    : msg.status === 'error'
                    ? 'var(--accent-danger-subtle)'
                    : isSystem
                    ? 'var(--bg-surface-3)'
                    : 'var(--bg-surface-2)',
                  color: isUser
                    ? '#ffffff'
                    : msg.status === 'error'
                    ? 'var(--accent-danger)'
                    : 'var(--text-primary)',
                  border: isUser
                    ? '1px solid rgba(255, 255, 255, 0.1)'
                    : '1px solid var(--border-subtle)',
                  fontSize: 'var(--text-sm)',
                  lineHeight: '1.5',
                  wordBreak: 'break-word',
                  whiteSpace: 'pre-wrap',
                }}
              >
                {msg.content}
              </div>
            </div>
          );
        })}
        <div ref={messagesEndRef} />
      </div>

      {/* Suggested Quick Prompt Chips */}
      <div style={{ display: 'flex', gap: '8px', overflowX: 'auto', paddingBottom: '4px' }}>
        {[
          'Halo Hermes, bagaimana arsitektur keamanan Hime Core?',
          'Jelaskan mitigasi IB-01 Credential Security melalui DPAPI Vault',
          'Analisis status IB-02 Localhost Bypass pada 9Router',
          'Lakukan evaluasi kebijakan baca file docs/desktop/DESIGN-SYSTEM.md',
        ].map((chip) => (
          <button
            key={chip}
            onClick={() => handleSendPrompt(chip)}
            disabled={isGenerating}
            className="btn-pressable"
            style={{
              padding: '4px 10px',
              backgroundColor: 'var(--bg-surface-2)',
              border: '1px solid var(--border-subtle)',
              borderRadius: 'var(--radius-pill)',
              color: 'var(--text-secondary)',
              fontSize: 'var(--text-xs)',
              cursor: isGenerating ? 'not-allowed' : 'pointer',
              whiteSpace: 'nowrap',
              opacity: isGenerating ? 0.6 : 1,
            }}
          >
            {chip}
          </button>
        ))}
      </div>

      {/* Message Composer */}
      <div
        style={{
          display: 'flex',
          gap: '8px',
          alignItems: 'flex-end',
          backgroundColor: 'var(--bg-surface-2)',
          padding: '8px 12px',
          borderRadius: 'var(--radius-md)',
          border: '1px solid var(--border-medium)',
        }}
      >
        <textarea
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder="Kirim instruksi ke Hermes Agent (Enter untuk kirim, Shift+Enter untuk baris baru)..."
          rows={2}
          disabled={isGenerating}
          style={{
            flex: 1,
            backgroundColor: 'transparent',
            border: 'none',
            outline: 'none',
            color: 'var(--text-primary)',
            fontSize: 'var(--text-sm)',
            fontFamily: 'inherit',
            resize: 'none',
          }}
        />
        {isGenerating ? (
          <Button variant="danger" size="sm" onClick={handleCancel}>
            Stop
          </Button>
        ) : (
          <Button
            variant="primary"
            size="sm"
            disabled={!input.trim()}
            onClick={() => handleSendPrompt(input)}
          >
            Send
          </Button>
        )}
      </div>
    </div>
  );
};
