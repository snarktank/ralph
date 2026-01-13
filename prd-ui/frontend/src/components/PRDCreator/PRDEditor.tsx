import { useState, useEffect } from 'react';
import ReactMarkdown from 'react-markdown';
import { prdApi } from '../../services/api';
import type { Question } from '../../types/prd';
import './PRDEditor.css';

interface PRDEditorProps {
  featureDescription: string;
  answers: Record<string, string>;
  questions: Question[];
  onSubmit: (content: string) => void;
  onBack: () => void;
}

export default function PRDEditor({
  featureDescription,
  answers,
  questions,
  onSubmit,
  onBack
}: PRDEditorProps) {
  const [content, setContent] = useState('');
  const [generating, setGenerating] = useState(true);

  useEffect(() => {
    generatePRDContent();
  }, []);

  const generatePRDContent = async () => {
    setGenerating(true);
    try {
      // For now, we'll generate a basic PRD structure
      // In a full implementation, this would call the backend API
      const generated = `# PRD: ${featureDescription.split(' ').slice(0, 5).join(' ')}

## Introduction

${featureDescription}

## Goals

- Implement the feature as described
- Ensure type safety and code quality
- Maintain existing functionality

## User Stories

### US-001: Initial Implementation
**Description:** As a developer, I want to implement this feature so that users can benefit from it.

**Acceptance Criteria:**
- [ ] Implementation follows project conventions
- [ ] Typecheck passes
- [ ] Tests pass (if applicable)

## Functional Requirements

- FR-1: Implement core functionality
- FR-2: Ensure proper error handling
- FR-3: Maintain backward compatibility

## Non-Goals

- No breaking changes to existing APIs
- No changes to unrelated features

## Technical Considerations

- Follow existing code patterns
- Use established libraries
- Ensure type safety

## Success Metrics

- Feature works as described
- No regressions
- Code passes quality checks

## Open Questions

- Are there any edge cases to consider?
- Are there any performance requirements?
`;
      setContent(generated);
    } catch (err) {
      console.error('Failed to generate PRD:', err);
      setContent('# PRD\n\nError generating PRD. Please try again.');
    } finally {
      setGenerating(false);
    }
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    onSubmit(content);
  };

  if (generating) {
    return (
      <div className="prd-editor">
        <div className="card">
          <div>Generating PRD...</div>
        </div>
      </div>
    );
  }

  return (
    <div className="prd-editor">
      <div className="card">
        <h2>Review and Edit PRD</h2>
        <div className="editor-container">
          <div className="editor-panel">
            <label>Markdown Editor</label>
            <textarea
              value={content}
              onChange={(e) => setContent(e.target.value)}
              className="markdown-editor"
              rows={20}
            />
          </div>
          <div className="preview-panel">
            <label>Preview</label>
            <div className="markdown-preview">
              <ReactMarkdown>{content}</ReactMarkdown>
            </div>
          </div>
        </div>
        <div className="form-actions">
          <button type="button" onClick={onBack}>Back</button>
          <button type="button" onClick={generatePRDContent}>Regenerate</button>
          <button type="submit" onClick={handleSubmit}>Next: Save</button>
        </div>
      </div>
    </div>
  );
}
