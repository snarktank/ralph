import { useEffect, useRef } from 'react';
import { useRalphStore } from '../store/useRalphStore';
import './OrchestratorChat.css';

export function OrchestratorChat() {
  const messages = useRalphStore((state) => state.orchestratorMessages);
  const messagesEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages]);

  return (
    <div className="orchestrator-chat">
      <div className="chat-header">
        <h2>Orchestrator Chat</h2>
        <p>Your conversation with Ralph</p>
      </div>
      <div className="chat-messages">
        {messages.length === 0 ? (
          <div className="empty-state">
            <p>No messages yet. Start Ralph to begin the conversation.</p>
          </div>
        ) : (
          messages.map((msg, idx) => (
            <div key={idx} className={`message message-${msg.role}`}>
              <div className="message-role">{msg.role}</div>
              <div className="message-content">{msg.content}</div>
              <div className="message-timestamp">
                {new Date(msg.timestamp).toLocaleTimeString()}
              </div>
            </div>
          ))
        )}
        <div ref={messagesEndRef} />
      </div>
    </div>
  );
}
