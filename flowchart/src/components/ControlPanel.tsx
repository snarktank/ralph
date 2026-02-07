import { useState } from 'react';
import { useRalphStore } from '../store/useRalphStore';
import { api } from '../services/api';
import './ControlPanel.css';

export function ControlPanel() {
  const ralphStatus = useRalphStore((state) => state.ralphStatus);
  const [maxIterations, setMaxIterations] = useState(10);
  const [useCli, setUseCli] = useState(false);
  const wsConnected = useRalphStore((state) => state.wsConnected);

  const handleStart = async () => {
    try {
      await api.startRalph(maxIterations, useCli);
      // Status will be updated via WebSocket
    } catch (error) {
      console.error('Failed to start Ralph:', error);
      alert('Failed to start Ralph. Check console for details.');
    }
  };

  const handleStop = async () => {
    try {
      await api.stopRalph();
      // Status will be updated via WebSocket
    } catch (error) {
      console.error('Failed to stop Ralph:', error);
      alert('Failed to stop Ralph');
    }
  };

  const isRunning = ralphStatus?.running || false;

  return (
    <div className="control-panel">
      <div className="connection-status">
        <div className={`status-dot ${wsConnected ? 'connected' : 'disconnected'}`}></div>
        <span>{wsConnected ? 'Connected' : 'Disconnected'}</span>
      </div>

      <div className="control-section">
        <h3>Ralph Control</h3>

        <div className="control-group">
          <label>Max Iterations</label>
          <input
            type="number"
            value={maxIterations}
            onChange={(e) => setMaxIterations(parseInt(e.target.value))}
            min={1}
            max={100}
            disabled={isRunning}
          />
        </div>

        <div className="control-group checkbox-group">
          <label>
            <input
              type="checkbox"
              checked={useCli}
              onChange={(e) => setUseCli(e.target.checked)}
              disabled={isRunning}
            />
            Use CLI (instead of API)
          </label>
        </div>

        <div className="control-actions">
          {!isRunning ? (
            <button onClick={handleStart} className="btn-start" disabled={!wsConnected}>
              Start Ralph
            </button>
          ) : (
            <button onClick={handleStop} className="btn-stop">
              Stop Ralph
            </button>
          )}
        </div>

        {!wsConnected && (
          <div className="warning-message">
            WebSocket disconnected. Reconnecting...
          </div>
        )}
      </div>
    </div>
  );
}
