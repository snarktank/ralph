import { useState } from 'react';
import { prdApi } from '../../services/api';
import './SaveStep.css';

interface SaveStepProps {
  projectPath: string;
  featureName: string;
  prdContent: string;
  projectName: string;
  onBack: () => void;
}

export default function SaveStep({
  projectPath,
  featureName,
  prdContent,
  projectName,
  onBack
}: SaveStepProps) {
  const [saving, setSaving] = useState(false);
  const [success, setSuccess] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Check if project path is missing
  if (!projectPath) {
    return (
      <div className="save-step">
        <div className="card">
          <div className="error-message">
            <h2>Project Path Missing</h2>
            <p>Project path is required to save the PRD. Please go back and select a project.</p>
            <button onClick={onBack}>Go Back to Project Selection</button>
          </div>
        </div>
      </div>
    );
  }

  const handleSave = async (e: React.FormEvent) => {
    e.preventDefault();
    setSaving(true);
    setError(null);

    try {
      await prdApi.create({
        projectPath,
        featureName,
        prdContent,
        projectName
      });
      setSuccess(true);
    } catch (err: any) {
      setError(err.response?.data?.error || 'Failed to save PRD');
    } finally {
      setSaving(false);
    }
  };

  if (success) {
    return (
      <div className="save-step">
        <div className="card success">
          <h2>✓ PRD Saved Successfully!</h2>
          <p>Your PRD has been saved to:</p>
          <code>{projectPath}/tasks/prd-{featureName}.md</code>
          <button onClick={() => window.location.href = '/convert'}>
            Convert to JSON
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="save-step">
      <div className="card">
        <h2>Save PRD</h2>
        <div className="save-info">
          <p><strong>Project:</strong> {projectPath}</p>
          <p><strong>Filename:</strong> prd-{featureName}.md</p>
          <p><strong>Location:</strong> {projectPath}/tasks/prd-{featureName}.md</p>
        </div>

        {error && <div className="error-message">{error}</div>}

        <form onSubmit={handleSave}>
          <div className="form-actions">
            <button type="button" onClick={onBack} disabled={saving}>
              Back
            </button>
            <button type="submit" disabled={saving}>
              {saving ? 'Saving...' : 'Save PRD'}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
