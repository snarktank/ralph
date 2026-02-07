import { useState, useEffect } from 'react';
import type { Conversation } from '../types';
import { ConversationHistory } from './ConversationHistory';
import './SubagentConversationViewer.css';

interface SubagentConversationViewerProps {
  onRefresh?: () => void;
}

export function SubagentConversationViewer({ onRefresh }: SubagentConversationViewerProps) {
  const [conversations, setConversations] = useState<Conversation[]>([]);
  const [selectedIteration, setSelectedIteration] = useState<number | null>(null);
  const [loading, setLoading] = useState(false);

  const fetchConversations = async () => {
    setLoading(true);
    try {
      const response = await fetch('http://localhost:8000/api/conversations/subagents');
      if (response.ok) {
        const data = await response.json();
        setConversations(data);

        // Auto-select the latest iteration
        if (data.length > 0 && !selectedIteration) {
          const latest = data[data.length - 1];
          setSelectedIteration(latest.iteration || 0);
        }
      }
    } catch (error) {
      console.error('Failed to fetch subagent conversations:', error);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchConversations();

    // Poll for updates every 2 seconds
    const interval = setInterval(fetchConversations, 2000);
    return () => clearInterval(interval);
  }, []);

  const handleRefresh = () => {
    fetchConversations();
    onRefresh?.();
  };

  const selectedConversation = conversations.find(
    c => c.iteration === selectedIteration
  );

  return (
    <div className="subagent-viewer">
      <div className="iteration-tabs">
        <div className="tabs-header">
          <h3>Subagent Iterations</h3>
          <span className="tabs-count">{conversations.length} iterations</span>
        </div>
        <div className="tabs-list">
          {conversations.length === 0 ? (
            <div className="no-iterations">
              No iterations yet
            </div>
          ) : (
            conversations.map((conv) => (
              <button
                key={conv.id}
                className={`iteration-tab ${selectedIteration === conv.iteration ? 'active' : ''}`}
                onClick={() => setSelectedIteration(conv.iteration || 0)}
              >
                <div className="tab-title">Iteration {conv.iteration}</div>
                <div className="tab-meta">
                  {conv.story_id && <span className="story-id">{conv.story_id}</span>}
                  <span className="message-count">{conv.messages.length} msgs</span>
                </div>
              </button>
            ))
          )}
        </div>
      </div>

      <div className="conversation-container">
        {loading && conversations.length === 0 ? (
          <div className="loading-state">Loading conversations...</div>
        ) : selectedConversation ? (
          <ConversationHistory
            type="subagent"
            conversation={selectedConversation}
            onRefresh={handleRefresh}
          />
        ) : (
          <div className="no-selection">
            Select an iteration to view its conversation
          </div>
        )}
      </div>
    </div>
  );
}
