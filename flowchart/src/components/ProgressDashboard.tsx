import { useRalphStore } from '../store/useRalphStore';
import './ProgressDashboard.css';

export function ProgressDashboard() {
  const prdStatus = useRalphStore((state) => state.prdStatus);
  const ralphStatus = useRalphStore((state) => state.ralphStatus);
  const prd = useRalphStore((state) => state.prd);

  if (!prdStatus || !prdStatus.exists) {
    return (
      <div className="progress-dashboard">
        <div className="dashboard-empty">
          <p>No PRD loaded</p>
        </div>
      </div>
    );
  }

  const progressPercentage =
    prdStatus.total_stories > 0
      ? (prdStatus.completed_stories / prdStatus.total_stories) * 100
      : 0;

  return (
    <div className="progress-dashboard">
      <div className="dashboard-header">
        <h3>Progress Overview</h3>
      </div>

      <div className="progress-stats">
        <div className="stat-card">
          <div className="stat-value">{prdStatus.total_stories}</div>
          <div className="stat-label">Total Stories</div>
        </div>
        <div className="stat-card complete">
          <div className="stat-value">{prdStatus.completed_stories}</div>
          <div className="stat-label">Completed</div>
        </div>
        <div className="stat-card pending">
          <div className="stat-value">{prdStatus.incomplete_stories}</div>
          <div className="stat-label">Pending</div>
        </div>
      </div>

      <div className="progress-bar-container">
        <div className="progress-label">
          <span>Overall Progress</span>
          <span>{Math.round(progressPercentage)}%</span>
        </div>
        <div className="progress-bar">
          <div
            className="progress-bar-fill"
            style={{ width: `${progressPercentage}%` }}
          ></div>
        </div>
      </div>

      {ralphStatus && ralphStatus.running && (
        <div className="iteration-status">
          <div className="iteration-label">Current Iteration</div>
          <div className="iteration-value">
            {ralphStatus.current_iteration} / {ralphStatus.max_iterations}
          </div>
        </div>
      )}

      {prd && (
        <div className="story-list">
          <h4>Stories</h4>
          {prd.userStories
            .sort((a, b) => a.priority - b.priority)
            .map((story) => (
              <div key={story.id} className={`story-item ${story.passes ? 'complete' : ''}`}>
                <div className="story-checkbox">
                  {story.passes ? '✓' : '○'}
                </div>
                <div className="story-info">
                  <div className="story-title">{story.title}</div>
                  <div className="story-id">{story.id}</div>
                </div>
              </div>
            ))}
        </div>
      )}
    </div>
  );
}
