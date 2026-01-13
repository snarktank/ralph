import { useState } from 'react';
import { convertApi } from '../../services/api';
import type { PRDJSON } from '../../types/prd';
import './SaveJSONStep.css';

interface SaveJSONStepProps {
  projectPath: string;
  jsonData: PRDJSON;
  projectName: string;
  onBack: () => void;
}

export default function SaveJSONStep({
  projectPath,
  jsonData,
  projectName,
  onBack
}: SaveJSONStepProps) {
  const [saving, setSaving] = useState(false);
  const [success, setSuccess] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleSave = async (e: React.FormEvent) => {
    e.preventDefault();
    setSaving(true);
    setError(null);

    try {
      await convertApi.save({
        projectPath,
        jsonData: {
          ...jsonData,
          project: projectName || jsonData.project
        },
        projectName
      });
      setSuccess(true);
    } catch (err: any) {
      setError(err.response?.data?.error || 'Failed to save prd.json');
    } finally {
      setSaving(false);
    }
  };

  if (success) {
    return (
      <div className="save-json-step">
        <div className="card success">
          <h2>✓ prd.json Saved Successfully!</h2>
          <p>Your prd.json has been saved to:</p>
          <code>{projectPath}/prd.json</code>
          <p className="next-steps">
            You can now run <code>./scripts/ralph/ralph.sh</code> to start Ralph!
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="save-json-step">
      <div className="card">
        <h2>Save prd.json</h2>
        <div className="save-info">
          <p><strong>Project:</strong> {projectPath}</p>
          <p><strong>Filename:</strong> prd.json</p>
          <p><strong>Location:</strong> {projectPath}/prd.json</p>
          <p><strong>Stories:</strong> {jsonData.userStories.length}</p>
        </div>

        {error && <div className="error-message">{error}</div>}

        <form onSubmit={handleSave}>
          <div className="form-actions">
            <button type="button" onClick={onBack} disabled={saving}>
              Back
            </button>
            <button type="submit" disabled={saving}>
              {saving ? 'Saving...' : 'Save prd.json'}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
