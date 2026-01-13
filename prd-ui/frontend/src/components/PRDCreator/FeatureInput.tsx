import { useState } from 'react';
import './FeatureInput.css';

interface FeatureInputProps {
  onSubmit: (description: string, name: string, projectName: string) => void;
  onBack: () => void;
}

export default function FeatureInput({ onSubmit, onBack }: FeatureInputProps) {
  const [description, setDescription] = useState('');
  const [name, setName] = useState('');
  const [projectName, setProjectName] = useState('');

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (description.trim() && name.trim()) {
      onSubmit(description, name, projectName || 'Project');
    }
  };

  return (
    <div className="feature-input">
      <div className="card">
        <h2>Describe Your Feature</h2>
        <form onSubmit={handleSubmit}>
          <div className="form-group">
            <label htmlFor="feature-description">Feature Description</label>
            <textarea
              id="feature-description"
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              placeholder="Describe what you want to build..."
              rows={6}
              required
            />
          </div>

          <div className="form-group">
            <label htmlFor="feature-name">Feature Name (for filename)</label>
            <input
              id="feature-name"
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="task-priority"
              required
            />
            <small>Will be saved as prd-{name || 'feature-name'}.md</small>
          </div>

          <div className="form-group">
            <label htmlFor="project-name">Project Name (optional)</label>
            <input
              id="project-name"
              type="text"
              value={projectName}
              onChange={(e) => setProjectName(e.target.value)}
              placeholder="MyApp"
            />
          </div>

          <div className="form-actions">
            <button type="button" onClick={onBack}>Back</button>
            <button type="submit" disabled={!description.trim() || !name.trim()}>
              Next: Answer Questions
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
