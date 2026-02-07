import { useEffect, useRef } from 'react';
import { useRalphStore } from '../store/useRalphStore';
import './SubagentPanel.css';

export function SubagentPanel() {
  const messages = useRalphStore((state) => state.subagentMessages);
  const ralphStatus = useRalphStore((state) => state.ralphStatus);
  const messagesEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages]);

  return (
    <div className="subagent-panel">
      <div className="panel-header">
        <div>
          <h2>Current Iteration</h2>
          {ralphStatus?.running && (
            <p>
              Iteration {ralphStatus.current_iteration} of {ralphStatus.max_iterations}
            </p>
          )}
        </div>
        {ralphStatus?.running && (
          <div className="status-indicator running">
            <span className="pulse"></span>
            Running
          </div>
        )}
      </div>
      <div className="panel-messages">
        {messages.length === 0 ? (
          <div className="empty-state">
            <p>Waiting for subagent to start...</p>
          </div>
        ) : (
          messages.map((msg, idx) => (
            <div key={idx} className={`subagent-message message-${msg.role}`}>
              <div className="message-header">
                <span className="message-role">{msg.role}</span>
                {msg.iteration && (
                  <span className="message-iteration">Iteration {msg.iteration}</span>
                )}
              </div>
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
