import { useState, useEffect } from 'react';
import { projectApi, prdApi } from '../../services/api';
import type { PRDFile } from '../../types/prd';
import './PRDSelector.css';

interface PRDSelectorProps {
  projectPath: string;
  onPRDSelected: (filename: string, content: string) => void;
  onBack: () => void;
}

export default function PRDSelector({ projectPath, onPRDSelected, onBack }: PRDSelectorProps) {
  const [prds, setPrds] = useState<PRDFile[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [customContent, setCustomContent] = useState('');
  const [useCustom, setUseCustom] = useState(false);

  useEffect(() => {
    loadPRDs();
  }, [projectPath]);

  const loadPRDs = async () => {
    setLoading(true);
    setError(null);
    try {
      const response = await projectApi.listPRDs(projectPath);
      setPrds(response.data.prds || []);
    } catch (err: any) {
      setError(err.response?.data?.error || 'Failed to load PRDs');
    } finally {
      setLoading(false);
    }
  };

  const handlePRDSelect = async (filename: string) => {
    if (!projectPath) {
      setError('Project path is required. Please go back and select a project.');
      return;
    }
    
    setLoading(true);
    setError(null);
    try {
      const featureName = filename.replace(/^prd-/, '').replace(/\.md$/, '');
      const response = await prdApi.read(projectPath, featureName);
      onPRDSelected(filename, response.data.content);
    } catch (err: any) {
      setError(err.response?.data?.error || 'Failed to read PRD');
    } finally {
      setLoading(false);
    }
  };

  const handleCustomSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (customContent.trim()) {
      onPRDSelected('custom.md', customContent);
    }
  };

  if (loading) {
    return (
      <div className="prd-selector">
        <div className="card">
          <div>Loading PRDs...</div>
        </div>
      </div>
    );
  }

  return (
    <div className="prd-selector">
      <div className="card">
        <h2>Select PRD to Convert</h2>

        {error && <div className="error-message">{error}</div>}

        {!useCustom ? (
          <>
            {prds.length > 0 ? (
              <div className="prd-list">
                {prds.map((prd) => (
                  <button
                    key={prd.filename}
                    className="prd-item"
                    onClick={() => handlePRDSelect(prd.filename)}
                  >
                    {prd.name}
                  </button>
                ))}
              </div>
            ) : (
              <div className="no-prds">
                No PRD files found. You can paste PRD content below.
              </div>
            )}

            <div className="divider">
              <span>OR</span>
            </div>

            <button
              className="toggle-custom"
              onClick={() => setUseCustom(true)}
            >
              Paste PRD Content
            </button>
          </>
        ) : (
          <form onSubmit={handleCustomSubmit} className="custom-form">
            <label htmlFor="custom-content">Paste PRD Markdown</label>
            <textarea
              id="custom-content"
              value={customContent}
              onChange={(e) => setCustomContent(e.target.value)}
              rows={15}
              placeholder="Paste your PRD markdown here..."
              required
            />
            <div className="form-actions">
              <button type="button" onClick={() => setUseCustom(false)}>
                Back to List
              </button>
              <button type="submit" disabled={!customContent.trim()}>
                Use This PRD
              </button>
            </div>
          </form>
        )}

        <div className="form-actions">
          <button type="button" onClick={onBack}>Back</button>
        </div>
      </div>
    </div>
  );
}
