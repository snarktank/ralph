import { useState, useEffect, useRef } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import './PRDBuilderPage.css';

interface ChatMessage {
  role: 'user' | 'assistant';
  content: string;
  timestamp: string;
}

export function PRDBuilderPage() {
  const { projectId } = useParams<{ projectId: string }>();
  const navigate = useNavigate();

  const [projectName, setProjectName] = useState('');
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [inputMessage, setInputMessage] = useState('');
  const [loading, setLoading] = useState(false);
  const [prdText, setPrdText] = useState('');
  const [ralphJson, setRalphJson] = useState('');
  const [showRalphJson, setShowRalphJson] = useState(false);
  const messagesEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    loadProject();
    // Add welcome message
    setMessages([{
      role: 'assistant',
      content: "Hi! I'm here to help you create a Product Requirements Document. Tell me about what you want to build, and I'll help you draft a PRD.\n\nYou can also edit the PRD directly in the editor on the right. When you're ready, click 'Create Ralph Config' to generate the configuration for autonomous development.",
      timestamp: new Date().toISOString()
    }]);
  }, [projectId]);

  useEffect(() => {
    scrollToBottom();
  }, [messages]);

  const loadProject = async () => {
    try {
      const response = await fetch(`http://localhost:8000/api/projects/${projectId}`);
      if (response.ok) {
        const project = await response.json();
        setProjectName(project.name);

        // Try to load existing PRD
        const prdResponse = await fetch(`http://localhost:8000/api/projects/${projectId}/prd`);
        if (prdResponse.ok) {
          const existingPrd = await prdResponse.json();
          setPrdText(JSON.stringify(existingPrd, null, 2));
        } else {
          // Set initial empty PRD template
          const template = {
            projectName: project.name,
            branchName: `feature/${project.name.toLowerCase().replace(/\s+/g, '-')}`,
            description: project.description,
            userStories: []
          };
          setPrdText(JSON.stringify(template, null, 2));
        }
      }
    } catch (error) {
      console.error('Error loading project:', error);
    }
  };

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  };

  const handleSendMessage = async () => {
    if (!inputMessage.trim() || loading) return;

    const userMessage: ChatMessage = {
      role: 'user',
      content: inputMessage,
      timestamp: new Date().toISOString()
    };

    setMessages(prev => [...prev, userMessage]);
    setInputMessage('');
    setLoading(true);

    try {
      // Check if PRD is empty or just template
      const currentPrd = JSON.parse(prdText);
      const isEmpty = !currentPrd.userStories || currentPrd.userStories.length === 0;

      if (isEmpty) {
        // Generate initial PRD
        const response = await fetch(`http://localhost:8000/api/projects/${projectId}/prd/generate`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ user_prompt: inputMessage })
        });

        if (response.ok) {
          const newPrd = await response.json();
          setPrdText(JSON.stringify(newPrd, null, 2));

          const assistantMessage: ChatMessage = {
            role: 'assistant',
            content: `Great! I've created a PRD with ${newPrd.userStories.length} user stories. You can see it in the editor on the right.\n\nFeel free to:\n- Edit the PRD directly in the editor\n- Ask me to add or modify features\n- Click "Create Ralph Config" when you're ready`,
            timestamp: new Date().toISOString()
          };
          setMessages(prev => [...prev, assistantMessage]);
        }
      } else {
        // Update existing PRD
        const response = await fetch(`http://localhost:8000/api/projects/${projectId}/prd/update`, {
          method: 'PUT',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ update_prompt: inputMessage })
        });

        if (response.ok) {
          const updatedPrd = await response.json();
          setPrdText(JSON.stringify(updatedPrd, null, 2));

          const assistantMessage: ChatMessage = {
            role: 'assistant',
            content: "I've updated the PRD based on your request. Check out the changes in the editor!",
            timestamp: new Date().toISOString()
          };
          setMessages(prev => [...prev, assistantMessage]);
        }
      }
    } catch (error) {
      console.error('Error:', error);
      const errorMessage: ChatMessage = {
        role: 'assistant',
        content: "Sorry, I encountered an error. Please try again or edit the PRD directly in the editor.",
        timestamp: new Date().toISOString()
      };
      setMessages(prev => [...prev, errorMessage]);
    } finally {
      setLoading(false);
    }
  };

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSendMessage();
    }
  };

  const handleSavePrd = async () => {
    try {
      const prdData = JSON.parse(prdText);
      await fetch(`http://localhost:8000/api/projects/${projectId}/prd`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(prdData)
      });
    } catch (error) {
      console.error('Error saving PRD:', error);
      alert('Error saving PRD. Please check the JSON format.');
    }
  };

  const handleCreateRalphConfig = async () => {
    try {
      // First save the current PRD
      await handleSavePrd();

      // Ralph.json is just a copy of the PRD
      setRalphJson(prdText);
      setShowRalphJson(true);

      // Also save it to the file system
      const prdData = JSON.parse(prdText);
      await fetch(`http://localhost:8000/api/projects/${projectId}/ralph-config`, {
        method: 'POST'
      });

      const assistantMessage: ChatMessage = {
        role: 'assistant',
        content: "✅ Ralph configuration created! I've generated ralph.json from your PRD. You can see it below. When you're ready, you can go back to the dashboard to start Ralph's autonomous development loop.",
        timestamp: new Date().toISOString()
      };
      setMessages(prev => [...prev, assistantMessage]);
    } catch (error) {
      console.error('Error creating Ralph config:', error);
      alert('Error creating Ralph config. Please check the PRD format.');
    }
  };

  return (
    <div className="prd-builder-page">
      {/* Header */}
      <div className="prd-builder-header">
        <div className="header-content">
          <button className="back-button" onClick={() => navigate('/')}>
            ← Back to Dashboard
          </button>
          <h1>{projectName}</h1>
          <div className="header-subtitle">PRD Builder</div>
        </div>
      </div>

      {/* Main Split View */}
      <div className="prd-builder-content">
        {/* Left Panel - Chat */}
        <div className="chat-panel">
          <div className="chat-header">
            <div className="chat-title">
              <span className="chat-icon">💬</span>
              <span>AI Assistant</span>
            </div>
            <div className="chat-status">
              {loading && <span className="typing-indicator">●●●</span>}
            </div>
          </div>

          <div className="chat-messages">
            {messages.map((message, index) => (
              <div key={index} className={`message ${message.role}`}>
                <div className="message-avatar">
                  {message.role === 'assistant' ? '🤖' : '👤'}
                </div>
                <div className="message-content">
                  <div className="message-text">{message.content}</div>
                  <div className="message-time">
                    {new Date(message.timestamp).toLocaleTimeString()}
                  </div>
                </div>
              </div>
            ))}
            <div ref={messagesEndRef} />
          </div>

          <div className="chat-input-container">
            <textarea
              className="chat-input"
              placeholder="Describe what you want to add to the PRD..."
              value={inputMessage}
              onChange={(e) => setInputMessage(e.target.value)}
              onKeyPress={handleKeyPress}
              rows={3}
            />
            <button
              className="send-button"
              onClick={handleSendMessage}
              disabled={loading || !inputMessage.trim()}
            >
              {loading ? '⏳' : '➤'}
            </button>
          </div>
        </div>

        {/* Right Panel - Text Editor */}
        <div className="prd-panel">
          <div className="prd-header">
            <div className="prd-title">
              <span className="prd-icon">📄</span>
              <span>PRD Document (JSON)</span>
            </div>
            <button className="save-prd-button" onClick={handleSavePrd}>
              💾 Save
            </button>
          </div>

          <div className="prd-editor-container">
            <textarea
              className="prd-text-editor"
              value={prdText}
              onChange={(e) => setPrdText(e.target.value)}
              onBlur={handleSavePrd}
              spellCheck={false}
              placeholder="Your PRD will appear here..."
            />
          </div>

          {/* Ralph JSON Display */}
          {showRalphJson && (
            <div className="ralph-json-container">
              <div className="ralph-json-header">
                <span className="ralph-icon">🤖</span>
                <span>Ralph Configuration (ralph.json)</span>
              </div>
              <textarea
                className="ralph-json-editor"
                value={ralphJson}
                readOnly
                spellCheck={false}
              />
            </div>
          )}
        </div>
      </div>

      {/* Footer Action Bar */}
      <div className="prd-builder-footer">
        <div className="footer-content">
          <div className="footer-info">
            {!showRalphJson ? (
              <>
                <span className="info-icon">📝</span>
                <span>Edit your PRD above, then create Ralph config when ready</span>
              </>
            ) : (
              <>
                <span className="check-icon">✓</span>
                <span>Ralph config created! You can now start autonomous development</span>
              </>
            )}
          </div>
          {!showRalphJson ? (
            <button
              className="create-ralph-button"
              onClick={handleCreateRalphConfig}
            >
              🚀 Create Ralph Config
            </button>
          ) : (
            <button
              className="dashboard-button"
              onClick={() => navigate('/')}
            >
              ← Back to Dashboard
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
