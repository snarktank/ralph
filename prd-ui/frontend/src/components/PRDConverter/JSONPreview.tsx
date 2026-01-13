import { useState, useEffect } from 'react';
import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter';
import { vscDarkPlus } from 'react-syntax-highlighter/dist/esm/styles/prism';
import { convertApi } from '../../services/api';
import type { PRDJSON } from '../../types/prd';
import './JSONPreview.css';

interface JSONPreviewProps {
  projectPath: string;
  prdContent: string;
  selectedPRD: string;
  onJSONGenerated: (json: PRDJSON) => void;
  onBack: () => void;
  onProjectNameChange: (name: string) => void;
}

export default function JSONPreview({
  projectPath,
  prdContent,
  selectedPRD,
  onJSONGenerated,
  onBack,
  onProjectNameChange
}: JSONPreviewProps) {
  const [jsonData, setJsonData] = useState<PRDJSON | null>(null);
  const [projectName, setProjectName] = useState('');
  const [converting, setConverting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [validation, setValidation] = useState<any>(null);

  useEffect(() => {
    convertPRD();
  }, [prdContent, projectPath]);

  const convertPRD = async () => {
    setConverting(true);
    setError(null);
    try {
      const response = await convertApi.convert({
        projectPath,
        prdContent,
        projectName: projectName || undefined
      });
      setJsonData(response.data.json);
      setValidation(response.data.validation);
      onProjectNameChange(response.data.json.project);
    } catch (err: any) {
      setError(err.response?.data?.error || 'Failed to convert PRD');
    } finally {
      setConverting(false);
    }
  };

  const handleProjectNameChange = (name: string) => {
    setProjectName(name);
    onProjectNameChange(name);
  };

  const handleNext = () => {
    if (jsonData) {
      onJSONGenerated(jsonData);
    }
  };

  if (converting) {
    return (
      <div className="json-preview">
        <div className="card">
          <div>Converting PRD to JSON...</div>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="json-preview">
        <div className="card">
          <div className="error-message">{error}</div>
          <button onClick={convertPRD}>Retry</button>
        </div>
      </div>
    );
  }

  return (
    <div className="json-preview">
      <div className="card">
        <h2>Preview JSON</h2>

        <div className="form-group">
          <label htmlFor="project-name">Project Name</label>
          <input
            id="project-name"
            type="text"
            value={projectName || jsonData?.project || ''}
            onChange={(e) => handleProjectNameChange(e.target.value)}
            placeholder="Project name"
          />
        </div>

        {validation && !validation.valid && (
          <div className="validation-warnings">
            <h3>Validation Warnings</h3>
            <ul>
              {validation.errors?.map((err: string, i: number) => (
                <li key={i}>{err}</li>
              ))}
            </ul>
          </div>
        )}

        {jsonData && (
          <div className="json-container">
            <SyntaxHighlighter
              language="json"
              style={vscDarkPlus}
              customStyle={{
                borderRadius: '4px',
                padding: '1rem',
                fontSize: '0.9rem'
              }}
            >
              {JSON.stringify(jsonData, null, 2)}
            </SyntaxHighlighter>
          </div>
        )}

        <div className="form-actions">
          <button type="button" onClick={onBack}>Back</button>
          <button type="button" onClick={convertPRD}>Regenerate</button>
          <button
            type="button"
            onClick={handleNext}
            disabled={!jsonData || (validation && !validation.valid)}
          >
            Next: Save
          </button>
        </div>
      </div>
    </div>
  );
}
