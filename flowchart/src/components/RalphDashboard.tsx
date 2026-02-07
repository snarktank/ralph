import { useEffect } from 'react';
import { useRalphStore } from '../store/useRalphStore';
import { useWebSocket } from '../hooks/useWebSocket';
import { api } from '../services/api';
import { OrchestratorConversationViewer } from './OrchestratorConversationViewer';
import { SubagentConversationViewer } from './SubagentConversationViewer';
import { PRDEditor } from './PRDEditor';
import { ProgressDashboard } from './ProgressDashboard';
import { ControlPanel } from './ControlPanel';
import './RalphDashboard.css';

export function RalphDashboard() {
  const { setPRD, setPRDStatus, setRalphStatus } = useRalphStore();

  // Initialize WebSocket connection
  useWebSocket();

  // Load initial data
  useEffect(() => {
    loadData();

    // Poll for status updates
    const interval = setInterval(loadData, 5000);

    // Listen for refresh events from WebSocket
    const handleRefresh = () => {
      loadData();
    };
    window.addEventListener('refresh-prd-data', handleRefresh);

    return () => {
      clearInterval(interval);
      window.removeEventListener('refresh-prd-data', handleRefresh);
    };
  }, []);

  const loadData = async () => {
    try {
      const [prd, prdStatus, ralphStatus] = await Promise.all([
        api.getPRD(),
        api.getPRDStatus(),
        api.getRalphStatus(),
      ]);

      if (prd) setPRD(prd);
      setPRDStatus(prdStatus);
      setRalphStatus(ralphStatus);
    } catch (error) {
      console.error('Failed to load data:', error);
    }
  };

  return (
    <div className="ralph-dashboard">
      <div className="dashboard-header">
        <h1>Ralph - Autonomous AI Agent Dashboard</h1>
        <p>Build features autonomously with AI-powered PRD execution</p>
      </div>

      <div className="dashboard-layout">
        {/* Left Column: PRD Editor & Progress */}
        <div className="left-column">
          <div className="prd-section">
            <PRDEditor />
          </div>
          <div className="progress-section">
            <ProgressDashboard />
          </div>
        </div>

        {/* Right Column: Chat & Controls */}
        <div className="right-column">
          <div className="control-section">
            <ControlPanel />
          </div>
          <div className="orchestrator-section">
            <OrchestratorConversationViewer />
          </div>
          <div className="subagent-section">
            <SubagentConversationViewer />
          </div>
        </div>
      </div>
    </div>
  );
}
