import { useEffect } from 'react';
import { useRalphStore } from '../store/useRalphStore';
import { WS_URL } from '../services/api';
import type { WebSocketMessage } from '../types';

export function useWebSocket() {
  const {
    setWebSocket,
    setWSConnected,
    addOrchestratorMessage,
    addSubagentMessage,
    setPRDStatus,
    setRalphStatus,
    setPRD,
  } = useRalphStore();

  useEffect(() => {
    let ws: WebSocket | null = null;
    let reconnectTimeout: ReturnType<typeof setTimeout>;

    const connect = () => {
      ws = new WebSocket(WS_URL);

      ws.onopen = () => {
        console.log('WebSocket connected');
        setWSConnected(true);
        setWebSocket(ws);
      };

      ws.onclose = () => {
        console.log('WebSocket disconnected');
        setWSConnected(false);
        setWebSocket(null);

        // Attempt to reconnect after 3 seconds
        reconnectTimeout = setTimeout(() => {
          console.log('Attempting to reconnect...');
          connect();
        }, 3000);
      };

      ws.onerror = (error) => {
        console.error('WebSocket error:', error);
      };

      ws.onmessage = (event) => {
        try {
          const message: WebSocketMessage = JSON.parse(event.data);
          handleMessage(message);
        } catch (error) {
          console.error('Failed to parse WebSocket message:', error);
        }
      };
    };

    const handleMessage = (message: WebSocketMessage) => {
      console.log('WebSocket message:', message);

      switch (message.type) {
        case 'orchestrator_message':
          addOrchestratorMessage({
            role: message.data.role,
            content: message.data.content,
            timestamp: message.timestamp,
          });
          break;

        case 'subagent_message':
          addSubagentMessage({
            role: message.data.role,
            content: message.data.content,
            timestamp: message.timestamp,
            iteration: message.iteration,
          });
          break;

        case 'iteration_start':
          addSubagentMessage({
            role: 'system',
            content: `Starting iteration ${message.data.iteration}: ${message.data.story_id} - ${message.data.story_title}`,
            timestamp: message.timestamp,
            iteration: message.data.iteration,
          });
          break;

        case 'iteration_complete':
          addSubagentMessage({
            role: 'system',
            content: `Iteration ${message.data.iteration} complete for story ${message.data.story_id} (${message.data.status})`,
            timestamp: message.timestamp,
            iteration: message.data.iteration,
          });
          break;

        case 'story_update':
          addSubagentMessage({
            role: 'system',
            content: `Story ${message.data.story_id} marked as ${message.data.passes ? 'complete' : 'incomplete'}`,
            timestamp: message.timestamp,
          });
          // Refresh PRD and status
          refreshPRDData();
          break;

        case 'progress_update':
          addSubagentMessage({
            role: 'system',
            content: `Progress updated: ${message.data.entry}`,
            timestamp: message.timestamp,
          });
          break;

        case 'git_commit':
          addSubagentMessage({
            role: 'system',
            content: `Git commit: ${message.data.commit_hash.substring(0, 7)} - ${message.data.message}`,
            timestamp: message.timestamp,
          });
          break;

        case 'tool_call':
          addSubagentMessage({
            role: 'system',
            content: `Tool call: ${message.data.tool_name}`,
            timestamp: message.timestamp,
            iteration: message.iteration,
          });
          break;

        case 'tool_result':
          addSubagentMessage({
            role: 'system',
            content: `Tool result (${message.data.tool_name}): ${message.data.success ? 'Success' : 'Failed'}`,
            timestamp: message.timestamp,
            iteration: message.iteration,
          });
          break;

        case 'error':
          addOrchestratorMessage({
            role: 'assistant',
            content: `Error: ${message.data.message}`,
            timestamp: message.timestamp,
          });
          break;

        case 'complete':
          addOrchestratorMessage({
            role: 'assistant',
            content: message.data.message,
            timestamp: message.timestamp,
          });
          // Refresh data
          refreshPRDData();
          break;

        default:
          console.log('Unknown message type:', message.type);
      }
    };

    const refreshPRDData = async () => {
      // This will be called from the component that uses this hook
      // We'll trigger a custom event that the RalphDashboard can listen to
      window.dispatchEvent(new CustomEvent('refresh-prd-data'));
    };

    connect();

    return () => {
      if (reconnectTimeout) {
        clearTimeout(reconnectTimeout);
      }
      if (ws) {
        ws.close();
      }
    };
  }, [
    setWebSocket,
    setWSConnected,
    addOrchestratorMessage,
    addSubagentMessage,
    setPRDStatus,
    setRalphStatus,
    setPRD,
  ]);
}
