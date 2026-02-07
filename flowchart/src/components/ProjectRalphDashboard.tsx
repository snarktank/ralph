import { useState, useEffect, useRef } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import './ProjectRalphDashboard.css';

interface RalphMessage {
  type: string;
  stream?: string;
  content: string;
  parsed?: {
    type: string;
    detail: string;
  } | null;
  timestamp: string;
}

interface Project {
  id: string;
  name: string;
  description: string;
  path: string;
  port: number;
  stack: string;
  status: string;
  created_at: string;
  url?: string;
  prd_path?: string;
}

export function ProjectRalphDashboard() {
  const { projectId } = useParams<{ projectId: string }>();
  const navigate = useNavigate();
  const [project, setProject] = useState<Project | null>(null);
  const [messages, setMessages] = useState<RalphMessage[]>([]);
  const [isRalphRunning, setIsRalphRunning] = useState(false);
  const [loading, setLoading] = useState(true);
  const wsRef = useRef<WebSocket | null>(null);
  const messagesEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!projectId) return;

    // Fetch project details
    fetchProject();

    // Connect to project-specific WebSocket
    connectWebSocket();

    // Check Ralph status
    checkRalphStatus();

    return () => {
      // Cleanup WebSocket on unmount
      if (wsRef.current) {
        wsRef.current.close();
      }
    };
  }, [projectId]);

  useEffect(() => {
    // Auto-scroll to bottom when new messages arrive
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages]);

  const fetchProject = async () => {
    try {
      const response = await fetch(`http://localhost:8000/api/projects/${projectId}`);
      if (response.ok) {
        const data = await response.json();
        setProject(data);
      }
    } catch (error) {
      console.error('Failed to fetch project:', error);
    } finally {
      setLoading(false);
    }
  };

  const connectWebSocket = () => {
    const ws = new WebSocket(`ws://localhost:8000/ws/project/${projectId}`);

    ws.onopen = () => {
      console.log(`Connected to project ${projectId} WebSocket`);
      // Send ping to keep alive
      setInterval(() => {
        if (ws.readyState === WebSocket.OPEN) {
          ws.send(JSON.stringify({ command: 'ping' }));
        }
      }, 30000);
    };

    ws.onmessage = (event) => {
      const data = JSON.parse(event.data);
      console.log('Received WebSocket message:', data);

      if (data.type === 'project_ralph_message') {
        // Add message to conversation
        setMessages((prev) => [...prev, data.data]);
      }
    };

    ws.onerror = (error) => {
      console.error('WebSocket error:', error);
    };

    ws.onclose = () => {
      console.log('WebSocket closed');
      // Attempt to reconnect after 3 seconds
      setTimeout(connectWebSocket, 3000);
    };

    wsRef.current = ws;
  };

  const checkRalphStatus = async () => {
    try {
      const response = await fetch(`http://localhost:8000/api/projects/${projectId}/ralph/status`);
      if (response.ok) {
        const data = await response.json();
        setIsRalphRunning(data.ralph_running);

        // Load existing conversation if available
        if (data.conversation && data.conversation.orchestrator) {
          setMessages(data.conversation.orchestrator);
        }
      }
    } catch (error) {
      console.error('Failed to check Ralph status:', error);
    }
  };

  const startRalph = async () => {
    try {
      const response = await fetch(`http://localhost:8000/api/projects/${projectId}/ralph/start`, {
        method: 'POST'
      });

      if (response.ok) {
        setIsRalphRunning(true);
        setMessages([{
          type: 'system',
          content: 'Starting Ralph autonomous agent...',
          timestamp: new Date().toISOString()
        }]);
      }
    } catch (error) {
      console.error('Failed to start Ralph:', error);
    }
  };

  const stopRalph = async () => {
    try {
      const response = await fetch(`http://localhost:8000/api/projects/${projectId}/ralph/stop`, {
        method: 'POST'
      });

      if (response.ok) {
        setIsRalphRunning(false);
      }
    } catch (error) {
      console.error('Failed to stop Ralph:', error);
    }
  };

  const getMessageTypeIcon = (type: string) => {
    switch (type) {
      case 'system': return '⚙️';
      case 'error': return '❌';
      case 'message': return '💬';
      default: return '📝';
    }
  };

  const getMessageTypeClass = (type: string) => {
    switch (type) {
      case 'error': return 'message-error';
      case 'system': return 'message-system';
      default: return 'message-normal';
    }
  };

  if (loading) {
    return (
      <div className="project-ralph-dashboard loading">
        <div className="spinner-large"></div>
        <p>Loading project...</p>
      </div>
    );
  }

  if (!project) {
    return (
      <div className="project-ralph-dashboard error">
        <h2>Project not found</h2>
        <button onClick={() => navigate('/')}>Back to Projects</button>
      </div>
    );
  }

  return (
    <div className="project-ralph-dashboard">
      {/* Header */}
      <div className="ralph-header">
        <div className="header-left">
          <button className="back-button" onClick={() => navigate('/')}>
            ← Back to Projects
          </button>
          <div className="project-info">
            <h1>{project.name}</h1>
            <p>{project.description}</p>
          </div>
        </div>
        <div className="header-right">
          <div className={`ralph-status ${isRalphRunning ? 'running' : 'stopped'}`}>
            <span className="status-dot"></span>
            {isRalphRunning ? 'Ralph Running' : 'Ralph Stopped'}
          </div>
          {isRalphRunning ? (
            <button className="control-button stop" onClick={stopRalph}>
              Stop Ralph
            </button>
          ) : (
            <button className="control-button start" onClick={startRalph}>
              Start Ralph
            </button>
          )}
        </div>
      </div>

      {/* Messages Area */}
      <div className="ralph-messages">
        {messages.length === 0 ? (
          <div className="empty-messages">
            <div className="empty-icon">🤖</div>
            <h3>No Ralph activity yet</h3>
            <p>Start Ralph to begin autonomous development</p>
          </div>
        ) : (
          <>
            {messages.map((msg, index) => (
              <div key={index} className={`ralph-message ${getMessageTypeClass(msg.type)}`}>
                <div className="message-header">
                  <span className="message-icon">{getMessageTypeIcon(msg.type)}</span>
                  <span className="message-type">{msg.type}</span>
                  {msg.stream && <span className="message-stream">({msg.stream})</span>}
                  <span className="message-time">
                    {new Date(msg.timestamp).toLocaleTimeString()}
                  </span>
                </div>
                <div className="message-content">
                  {msg.content}
                </div>
                {msg.parsed && (
                  <div className="message-parsed">
                    <strong>{msg.parsed.type}:</strong> {msg.parsed.detail}
                  </div>
                )}
              </div>
            ))}
            <div ref={messagesEndRef} />
          </>
        )}
      </div>

      {/* Footer Info */}
      <div className="ralph-footer">
        <div className="footer-item">
          <span className="label">Project ID:</span>
          <span className="value">{project.id}</span>
        </div>
        <div className="footer-item">
          <span className="label">Stack:</span>
          <span className="value">{project.stack}</span>
        </div>
        <div className="footer-item">
          <span className="label">Port:</span>
          <span className="value">{project.port}</span>
        </div>
        {project.url && (
          <div className="footer-item">
            <a href={project.url} target="_blank" rel="noopener noreferrer" className="project-link">
              Open Project →
            </a>
          </div>
        )}
      </div>
    </div>
  );
}
