import { useState, useEffect } from 'react';
import { useProject } from '../hooks/useProject';
import { projectApi } from '../services/api';
import type { PRDFile } from '../types/prd';
import './ProjectSelector.css';

interface ProjectSelectorProps {
  onProjectSelected?: (path: string) => void;
}

export default function ProjectSelector({ onProjectSelected }: ProjectSelectorProps) {
  const { projectPath, isValidating, isValid, error, validateProject, setProjectPath } = useProject();
  const [inputPath, setInputPath] = useState('');
  const [prds, setPrds] = useState<PRDFile[]>([]);
  const [loadingPRDs, setLoadingPRDs] = useState(false);
  const [showHelp, setShowHelp] = useState(false);

  useEffect(() => {
    if (isValid && projectPath) {
      loadPRDs(projectPath);
    }
  }, [isValid, projectPath]);

  const handleContinue = () => {
    if (isValid && projectPath) {
      onProjectSelected?.(projectPath);
    }
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    await validateProject(inputPath);
  };

  const loadPRDs = async (path: string) => {
    setLoadingPRDs(true);
    try {
      const response = await projectApi.listPRDs(path);
      setPrds(response.data.prds || []);
    } catch (err) {
      console.error('Failed to load PRDs:', err);
      setPrds([]);
    } finally {
      setLoadingPRDs(false);
    }
  };

  return (
    <div className="project-selector">
      <form onSubmit={handleSubmit} className="project-form">
        <label htmlFor="project-path">Project Path</label>
        <div className="input-group">
          <input
            id="project-path"
            type="text"
            value={inputPath}
            onChange={(e) => setInputPath(e.target.value)}
            placeholder="/path/to/your/project or C:\path\to\your\project"
            className={error ? 'error' : ''}
            disabled={isValidating}
          />
          <button 
            type="button" 
            onClick={() => setShowHelp(!showHelp)}
            className="help-button"
            title="Show path examples and help"
          >
            ?
          </button>
          <button type="submit" disabled={isValidating || !inputPath.trim()}>
            {isValidating ? 'Validating...' : 'Validate'}
          </button>
        </div>
        
        {showHelp && (
          <div className="help-box">
            <h4>How to find your project path:</h4>
            <ul>
              <li><strong>macOS/Linux:</strong> Open terminal, navigate to your project, run <code>pwd</code></li>
              <li><strong>Windows:</strong> Open PowerShell, navigate to your project, run <code>pwd</code></li>
              <li><strong>VS Code:</strong> Right-click project folder → "Copy Path"</li>
              <li><strong>Finder (macOS):</strong> Right-click folder → Option+Click "Copy as Pathname"</li>
            </ul>
            <p><strong>Example paths:</strong></p>
            <ul>
              <li>macOS: <code>/Users/username/projects/my-app</code></li>
              <li>Windows: <code>C:\Users\username\projects\my-app</code></li>
              <li>Linux: <code>/home/username/projects/my-app</code></li>
            </ul>
          </div>
        )}
        
        <small className="path-hint">
          Enter the full absolute path to your project directory
        </small>
        {error && <div className="error-message">{error}</div>}
        {isValid && (
          <div className="success-message">
            ✓ Project path validated: {projectPath}
          </div>
        )}
      </form>

      {isValid && (
        <>
          <div className="continue-section">
            <button 
              type="button"
              onClick={handleContinue}
              className="continue-button"
            >
              Continue PRD →
            </button>
          </div>
          <div className="prd-list">
            <h3>Existing PRDs</h3>
            {loadingPRDs ? (
              <div>Loading PRDs...</div>
            ) : prds.length > 0 ? (
              <ul>
                {prds.map((prd) => (
                  <li key={prd.filename}>{prd.name}</li>
                ))}
              </ul>
            ) : (
              <div className="no-prds">No PRD files found in tasks/ directory</div>
            )}
          </div>
        </>
      )}
    </div>
  );
}
