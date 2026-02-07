import { useEffect, useRef, useState } from 'react';
import type { Conversation, Message } from '../types';
import './ConversationHistory.css';

interface ConversationHistoryProps {
  type: 'orchestrator' | 'subagent';
  conversation?: Conversation;
  onRefresh?: () => void;
}

export function ConversationHistory({ type, conversation, onRefresh }: ConversationHistoryProps) {
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const [expandedMessages, setExpandedMessages] = useState<Set<string>>(new Set());

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [conversation?.messages]);

  const toggleMessageExpanded = (messageId: string) => {
    setExpandedMessages(prev => {
      const next = new Set(prev);
      if (next.has(messageId)) {
        next.delete(messageId);
      } else {
        next.add(messageId);
      }
      return next;
    });
  };

  const renderMessage = (msg: Message) => {
    const isExpanded = expandedMessages.has(msg.id);
    const hasToolCalls = msg.tool_calls && msg.tool_calls.length > 0;
    const hasToolResults = msg.tool_results && msg.tool_results.length > 0;

    return (
      <div key={msg.id} className={`message message-${msg.role}`}>
        <div className="message-header">
          <span className="message-role">{msg.role}</span>
          <span className="message-timestamp">
            {new Date(msg.timestamp).toLocaleString()}
          </span>
        </div>
        <div className="message-content">{msg.content}</div>

        {(hasToolCalls || hasToolResults) && (
          <div className="message-tools">
            <button
              className="toggle-tools"
              onClick={() => toggleMessageExpanded(msg.id)}
            >
              {isExpanded ? '▼' : '▶'} {hasToolCalls ? `${msg.tool_calls?.length} tool calls` : ''}
              {hasToolCalls && hasToolResults ? ', ' : ''}
              {hasToolResults ? `${msg.tool_results?.length} results` : ''}
            </button>

            {isExpanded && (
              <div className="tools-detail">
                {hasToolCalls && (
                  <div className="tool-calls">
                    <strong>Tool Calls:</strong>
                    <pre>{JSON.stringify(msg.tool_calls, null, 2)}</pre>
                  </div>
                )}
                {hasToolResults && (
                  <div className="tool-results">
                    <strong>Tool Results:</strong>
                    <pre>{JSON.stringify(msg.tool_results, null, 2)}</pre>
                  </div>
                )}
              </div>
            )}
          </div>
        )}
      </div>
    );
  };

  return (
    <div className="conversation-history">
      <div className="conversation-header">
        <div>
          <h2>
            {type === 'orchestrator' ? 'Orchestrator Conversation' : 'Subagent Conversation'}
          </h2>
          {conversation && (
            <p className="conversation-meta">
              {conversation.messages.length} messages
              {conversation.iteration && ` • Iteration ${conversation.iteration}`}
              {conversation.story_id && ` • Story: ${conversation.story_id}`}
            </p>
          )}
        </div>
        {onRefresh && (
          <button className="refresh-button" onClick={onRefresh}>
            ↻ Refresh
          </button>
        )}
      </div>

      <div className="conversation-messages">
        {!conversation || conversation.messages.length === 0 ? (
          <div className="empty-state">
            <p>No messages yet. {type === 'orchestrator' ? 'Start Ralph to begin.' : 'Waiting for subagent...'}</p>
          </div>
        ) : (
          <>
            {conversation.messages.map(renderMessage)}
            <div ref={messagesEndRef} />
          </>
        )}
      </div>
    </div>
  );
}
