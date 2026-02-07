import { useState, useEffect, useRef } from 'react';
import './RalphProgressViewer.css';

interface RalphMessage {
  role: string;
  content: string;
  timestamp: string;
  type?: string;
}

interface RalphProgressViewerProps {
  projectId: string;
  isExpanded: boolean;
  onClose: () => void;
}

export function RalphProgressViewer({ projectId, isExpanded, onClose }: RalphProgressViewerProps) {
  const [messages, setMessages] = useState<RalphMessage[]>([]);
  const [isRunning, setIsRunning] = useState(false);
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const wsRef = useRef<WebSocket | null>(null);

  useEffect(() => {
    if (isExpanded) {
      connectWebSocket();
      fetchStatus();
    }

    return () => {
      if (wsRef.current) {
        wsRef.current.close();
      }
    };
  }, [isExpanded, projectId]);

  useEffect(() => {
    scrollToBottom();
  }, [messages]);

  const connectWebSocket = () => {
    if (wsRef.current) {
      wsRef.current.close();
    }

    const ws = new WebSocket(`ws://localhost:8000/ws/project/${projectId}`);

    ws.onopen = () => {
      console.log('WebSocket connected for project:', projectId);
    };

    ws.onmessage = (event) => {
      const message = JSON.parse(event.data);
      setMessages((prev) => [...prev, message]);
    };

    ws.onerror = (error) => {
      console.error('WebSocket error:', error);
    };

    ws.onclose = () => {
      console.log('WebSocket closed');
    };

    wsRef.current = ws;
  };

  const fetchStatus = async () => {
    try {
      const response = await fetch(`http://localhost:8000/api/projects/${projectId}/ralph/status`);
      if (response.ok) {
        const data = await response.json();
        setIsRunning(data.ralph_running);
        if (data.conversation && data.conversation.length > 0) {
          setMessages(data.conversation);
        }
      }
    } catch (error) {
      console.error('Error fetching Ralph status:', error);
    }
  };

  const handleStart = async () => {
    try {
      const response = await fetch(`http://localhost:8000/api/projects/${projectId}/ralph/start`, {
        method: 'POST'
      });

      if (response.ok) {
        setIsRunning(true);
        setMessages([]);
      } else {
        alert('Failed to start Ralph');
      }
    } catch (error) {
      console.error('Error starting Ralph:', error);
      alert('Error starting Ralph');
    }
  };

  const handleStop = async () => {
    try {
      const response = await fetch(`http://localhost:8000/api/projects/${projectId}/ralph/stop`, {
        method: 'POST'
      });

      if (response.ok) {
        setIsRunning(false);
      } else {
        alert('Failed to stop Ralph');
      }
    } catch (error) {
      console.error('Error stopping Ralph:', error);
      alert('Error stopping Ralph');
    }
  };

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  };

  if (!isExpanded) return null;

  return (
    <div className="ralph-progress-viewer">
      <div className="ralph-header">
        <div className="ralph-title">
          <span className="ralph-icon">🤖</span>
          <h4>Ralph AI Progress</h4>
          <span className={`status-indicator ${isRunning ? 'running' : 'stopped'}`}>
            {isRunning ? '● Running' : '○ Stopped'}
          </span>
        </div>
        <div className="ralph-controls">
          {!isRunning ? (
            <button className="start-ralph-button" onClick={handleStart}>
              ▶ Start
            </button>
          ) : (
            <button className="stop-ralph-button" onClick={handleStop}>
              ⏸ Stop
            </button>
          )}
          <button className="close-viewer-button" onClick={onClose}>
            ✕
          </button>
        </div>
      </div>

      <div className="ralph-messages">
        {messages.length === 0 ? (
          <div className="empty-messages">
            <p>No messages yet. Start Ralph to begin.</p>
          </div>
        ) : (
          messages.map((message, index) => (
            <div key={index} className={`ralph-message ${message.role} ${message.type || ''}`}>
              <div className="message-meta">
                <span className="message-role">{message.role}</span>
                <span className="message-time">{new Date(message.timestamp).toLocaleTimeString()}</span>
              </div>
              <div className="message-content">{message.content}</div>
            </div>
          ))
        )}
        <div ref={messagesEndRef} />
      </div>
    </div>
  );
}
