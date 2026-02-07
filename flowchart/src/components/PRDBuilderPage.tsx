import { useState, useEffect, useRef } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import type { PRDResponse, UserStory } from '../types';
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
  const [prd, setPrd] = useState<PRDResponse | null>(null);
  const messagesEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    loadProject();
    // Add welcome message
    setMessages([{
      role: 'assistant',
      content: "Hi! I'm here to help you create a Product Requirements Document for your project. Tell me about what you want to build, and I'll help structure it into user stories with clear acceptance criteria.\n\nYou can describe features, functionality, or requirements, and I'll organize them into a PRD. Feel free to be as detailed or high-level as you like!",
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
          setPrd(existingPrd);
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
      // If we don't have a PRD yet, generate one
      if (!prd) {
        const response = await fetch(`http://localhost:8000/api/projects/${projectId}/prd/generate`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ user_prompt: inputMessage })
        });

        if (response.ok) {
          const newPrd = await response.json();
          setPrd(newPrd);

          const assistantMessage: ChatMessage = {
            role: 'assistant',
            content: `Great! I've created a PRD with ${newPrd.userStories.length} user stories based on your description. You can see the document on the right.\n\nFeel free to:\n- Edit the document directly\n- Ask me to add, modify, or remove features\n- Refine the acceptance criteria\n\nWhen you're ready, click "Create Ralph Config" to set up autonomous development!`,
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
          setPrd(updatedPrd);

          const assistantMessage: ChatMessage = {
            role: 'assistant',
            content: "I've updated the PRD based on your request. Check out the changes on the right!",
            timestamp: new Date().toISOString()
          };
          setMessages(prev => [...prev, assistantMessage]);
        }
      }
    } catch (error) {
      console.error('Error:', error);
      const errorMessage: ChatMessage = {
        role: 'assistant',
        content: "Sorry, I encountered an error. Please try again.",
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

  const handleUpdatePrdField = (field: string, value: any) => {
    if (!prd) return;
    setPrd({ ...prd, [field]: value });
  };

  const handleUpdateUserStory = (index: number, field: keyof UserStory, value: any) => {
    if (!prd) return;
    const updatedStories = [...prd.userStories];
    updatedStories[index] = { ...updatedStories[index], [field]: value };
    setPrd({ ...prd, userStories: updatedStories });
  };

  const handleSavePrd = async () => {
    if (!prd) return;

    try {
      await fetch(`http://localhost:8000/api/projects/${projectId}/prd`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(prd)
      });
    } catch (error) {
      console.error('Error saving PRD:', error);
    }
  };

  const handleCreateRalphConfig = async () => {
    if (!prd) return;

    // Save PRD first
    await handleSavePrd();

    try {
      const response = await fetch(`http://localhost:8000/api/projects/${projectId}/ralph-config`, {
        method: 'POST'
      });

      if (response.ok) {
        // Navigate back to dashboard
        navigate('/');
      }
    } catch (error) {
      console.error('Error creating Ralph config:', error);
    }
  };

  return (
    <div className="prd-builder-page">
      {/* Header */}
      <div className="prd-builder-header">
        <div className="header-content">
          <button className="back-button" onClick={() => navigate('/')}>
            ← Back to Projects
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
              placeholder={prd ? "Ask me to modify the PRD..." : "Describe what you want to build..."}
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

        {/* Right Panel - PRD Editor */}
        <div className="prd-panel">
          <div className="prd-header">
            <div className="prd-title">
              <span className="prd-icon">📄</span>
              <span>Product Requirements Document</span>
            </div>
            {prd && (
              <button className="save-prd-button" onClick={handleSavePrd}>
                💾 Save
              </button>
            )}
          </div>

          <div className="prd-content">
            {!prd ? (
              <div className="prd-empty">
                <div className="empty-icon">📝</div>
                <h3>No PRD Yet</h3>
                <p>Start chatting with the AI assistant to generate your PRD</p>
              </div>
            ) : (
              <div className="prd-document">
                <div className="prd-field">
                  <label>Project Name</label>
                  <input
                    type="text"
                    value={prd.projectName}
                    onChange={(e) => handleUpdatePrdField('projectName', e.target.value)}
                    onBlur={handleSavePrd}
                  />
                </div>

                <div className="prd-field">
                  <label>Branch Name</label>
                  <input
                    type="text"
                    value={prd.branchName}
                    onChange={(e) => handleUpdatePrdField('branchName', e.target.value)}
                    onBlur={handleSavePrd}
                  />
                </div>

                <div className="prd-field">
                  <label>Description</label>
                  <textarea
                    value={prd.description}
                    onChange={(e) => handleUpdatePrdField('description', e.target.value)}
                    onBlur={handleSavePrd}
                    rows={3}
                  />
                </div>

                <div className="user-stories-section">
                  <h3>User Stories ({prd.userStories.length})</h3>

                  {prd.userStories.map((story, index) => (
                    <div key={story.id} className="story-card">
                      <div className="story-header-row">
                        <input
                          type="text"
                          className="story-id-input"
                          value={story.id}
                          onChange={(e) => handleUpdateUserStory(index, 'id', e.target.value)}
                          onBlur={handleSavePrd}
                        />
                        <span className="story-priority">Priority: {story.priority}</span>
                      </div>

                      <input
                        type="text"
                        className="story-title-input"
                        value={story.title}
                        onChange={(e) => handleUpdateUserStory(index, 'title', e.target.value)}
                        onBlur={handleSavePrd}
                        placeholder="Story title"
                      />

                      <textarea
                        className="story-desc-input"
                        value={story.description}
                        onChange={(e) => handleUpdateUserStory(index, 'description', e.target.value)}
                        onBlur={handleSavePrd}
                        placeholder="Story description"
                        rows={2}
                      />

                      <div className="acceptance-criteria-list">
                        <label>Acceptance Criteria:</label>
                        {story.acceptanceCriteria.map((criteria, cIndex) => (
                          <div key={cIndex} className="criteria-item">
                            <span className="criteria-bullet">•</span>
                            <span>{criteria}</span>
                          </div>
                        ))}
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            )}
          </div>
        </div>
      </div>

      {/* Footer Action Bar */}
      {prd && (
        <div className="prd-builder-footer">
          <div className="footer-content">
            <div className="footer-info">
              <span className="check-icon">✓</span>
              <span>PRD ready with {prd.userStories.length} user stories</span>
            </div>
            <button
              className="create-ralph-button"
              onClick={handleCreateRalphConfig}
            >
              🚀 Create Ralph Config & Continue
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
