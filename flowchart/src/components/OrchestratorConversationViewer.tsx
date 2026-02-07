import { useState, useEffect } from 'react';
import type { Conversation } from '../types';
import { ConversationHistory } from './ConversationHistory';

export function OrchestratorConversationViewer() {
  const [conversation, setConversation] = useState<Conversation | null>(null);
  const [loading, setLoading] = useState(false);

  const fetchConversation = async () => {
    setLoading(true);
    try {
      const response = await fetch('http://localhost:8000/api/conversations/orchestrator');
      if (response.ok) {
        const data = await response.json();
        setConversation(data);
      }
    } catch (error) {
      console.error('Failed to fetch orchestrator conversation:', error);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchConversation();

    // Poll for updates every 2 seconds
    const interval = setInterval(fetchConversation, 2000);
    return () => clearInterval(interval);
  }, []);

  if (loading && !conversation) {
    return <div className="loading-state">Loading conversation...</div>;
  }

  return (
    <ConversationHistory
      type="orchestrator"
      conversation={conversation || undefined}
      onRefresh={fetchConversation}
    />
  );
}
