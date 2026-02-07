import { useState, useEffect } from 'react';
import type { PRDResponse, UserStory, PRDGenerateRequest, PRDUpdateRequest } from '../types';
import './PRDEditorModal.css';

interface PRDEditorModalProps {
  isOpen: boolean;
  projectId: string;
  projectName: string;
  projectDescription: string;
  onClose: () => void;
  onSave: () => void;
}

export function PRDEditorModal({
  isOpen,
  projectId,
  projectName,
  projectDescription,
  onClose,
  onSave
}: PRDEditorModalProps) {
  const [mode, setMode] = useState<'prompt' | 'edit'>('prompt');
  const [loading, setLoading] = useState(false);
  const [prompt, setPrompt] = useState('');
  const [prd, setPrd] = useState<PRDResponse | null>(null);
  const [editedPrd, setEditedPrd] = useState<PRDResponse | null>(null);

  useEffect(() => {
    if (isOpen) {
      loadExistingPrd();
    }
  }, [isOpen, projectId]);

  const loadExistingPrd = async () => {
    try {
      const response = await fetch(`http://localhost:8000/api/projects/${projectId}/prd`);
      if (response.ok) {
        const data = await response.json();
        setPrd(data);
        setEditedPrd(data);
        setMode('edit');
      }
    } catch (error) {
      // PRD doesn't exist yet, that's okay
      setPrd(null);
      setEditedPrd(null);
    }
  };

  const handleGeneratePrd = async () => {
    if (!prompt.trim()) {
      alert('Please enter a description');
      return;
    }

    setLoading(true);
    try {
      const requestData: PRDGenerateRequest = { user_prompt: prompt };
      const response = await fetch(`http://localhost:8000/api/projects/${projectId}/prd/generate`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(requestData)
      });

      if (response.ok) {
        const data = await response.json();
        setPrd(data);
        setEditedPrd(data);
        setMode('edit');
        setPrompt('');
      } else {
        alert('Failed to generate PRD');
      }
    } catch (error) {
      console.error('Error generating PRD:', error);
      alert('Error generating PRD');
    } finally {
      setLoading(false);
    }
  };

  const handleUpdateWithAi = async () => {
    if (!prompt.trim() || !prd) {
      return;
    }

    setLoading(true);
    try {
      const requestData: PRDUpdateRequest = { update_prompt: prompt };
      const response = await fetch(`http://localhost:8000/api/projects/${projectId}/prd/update`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(requestData)
      });

      if (response.ok) {
        const data = await response.json();
        setPrd(data);
        setEditedPrd(data);
        setPrompt('');
      } else {
        alert('Failed to update PRD');
      }
    } catch (error) {
      console.error('Error updating PRD:', error);
      alert('Error updating PRD');
    } finally {
      setLoading(false);
    }
  };

  const handleSaveManual = async () => {
    if (!editedPrd) return;

    setLoading(true);
    try {
      const response = await fetch(`http://localhost:8000/api/projects/${projectId}/prd`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(editedPrd)
      });

      if (response.ok) {
        onSave();
        onClose();
      } else {
        alert('Failed to save PRD');
      }
    } catch (error) {
      console.error('Error saving PRD:', error);
      alert('Error saving PRD');
    } finally {
      setLoading(false);
    }
  };

  const updateUserStory = (index: number, field: keyof UserStory, value: any) => {
    if (!editedPrd) return;

    const updatedStories = [...editedPrd.userStories];
    updatedStories[index] = { ...updatedStories[index], [field]: value };

    setEditedPrd({ ...editedPrd, userStories: updatedStories });
  };

  const addUserStory = () => {
    if (!editedPrd) return;

    const newStory: UserStory = {
      id: `US-${String(editedPrd.userStories.length + 1).padStart(3, '0')}`,
      title: '',
      description: '',
      acceptanceCriteria: [''],
      priority: editedPrd.userStories.length + 1,
      passes: false
    };

    setEditedPrd({
      ...editedPrd,
      userStories: [...editedPrd.userStories, newStory]
    });
  };

  const removeUserStory = (index: number) => {
    if (!editedPrd) return;

    const updatedStories = editedPrd.userStories.filter((_, i) => i !== index);
    setEditedPrd({ ...editedPrd, userStories: updatedStories });
  };

  const updateAcceptanceCriteria = (storyIndex: number, criteriaIndex: number, value: string) => {
    if (!editedPrd) return;

    const updatedStories = [...editedPrd.userStories];
    const updatedCriteria = [...updatedStories[storyIndex].acceptanceCriteria];
    updatedCriteria[criteriaIndex] = value;
    updatedStories[storyIndex] = { ...updatedStories[storyIndex], acceptanceCriteria: updatedCriteria };

    setEditedPrd({ ...editedPrd, userStories: updatedStories });
  };

  const addAcceptanceCriteria = (storyIndex: number) => {
    if (!editedPrd) return;

    const updatedStories = [...editedPrd.userStories];
    updatedStories[storyIndex] = {
      ...updatedStories[storyIndex],
      acceptanceCriteria: [...updatedStories[storyIndex].acceptanceCriteria, '']
    };

    setEditedPrd({ ...editedPrd, userStories: updatedStories });
  };

  if (!isOpen) return null;

  return (
    <div className="prd-modal-overlay" onClick={onClose}>
      <div className="prd-modal-container" onClick={(e) => e.stopPropagation()}>
        <div className="prd-modal-header">
          <div>
            <h2>Product Requirements Document</h2>
            <p className="project-name">{projectName}</p>
          </div>
          <button className="close-button" onClick={onClose}>×</button>
        </div>

        <div className="prd-modal-body">
          {/* AI Prompt Section */}
          <div className="prd-section ai-section">
            <h3>✨ AI Assistant</h3>
            <div className="prompt-container">
              <textarea
                className="ai-prompt"
                placeholder={prd ? "Describe what you want to change..." : "Describe what you want to build..."}
                value={prompt}
                onChange={(e) => setPrompt(e.target.value)}
                rows={4}
              />
              <div className="prompt-actions">
                {!prd ? (
                  <button
                    className="generate-button"
                    onClick={handleGeneratePrd}
                    disabled={loading || !prompt.trim()}
                  >
                    {loading ? '⏳ Generating...' : '🚀 Generate PRD'}
                  </button>
                ) : (
                  <button
                    className="update-button"
                    onClick={handleUpdateWithAi}
                    disabled={loading || !prompt.trim()}
                  >
                    {loading ? '⏳ Updating...' : '🔄 Update with AI'}
                  </button>
                )}
              </div>
            </div>
          </div>

          {/* Manual Editor Section */}
          {editedPrd && (
            <div className="prd-section editor-section">
              <div className="section-header">
                <h3>📝 PRD Editor</h3>
                <button className="add-story-button" onClick={addUserStory}>
                  + Add User Story
                </button>
              </div>

              <div className="prd-fields">
                <div className="field-group">
                  <label>Project Name</label>
                  <input
                    type="text"
                    value={editedPrd.projectName}
                    onChange={(e) => setEditedPrd({ ...editedPrd, projectName: e.target.value })}
                  />
                </div>

                <div className="field-group">
                  <label>Branch Name</label>
                  <input
                    type="text"
                    value={editedPrd.branchName}
                    onChange={(e) => setEditedPrd({ ...editedPrd, branchName: e.target.value })}
                  />
                </div>

                <div className="field-group">
                  <label>Description</label>
                  <textarea
                    value={editedPrd.description}
                    onChange={(e) => setEditedPrd({ ...editedPrd, description: e.target.value })}
                    rows={2}
                  />
                </div>
              </div>

              <div className="user-stories">
                <h4>User Stories</h4>
                {editedPrd.userStories.map((story, storyIndex) => (
                  <div key={story.id} className="user-story-card">
                    <div className="story-header">
                      <input
                        type="text"
                        className="story-id"
                        value={story.id}
                        onChange={(e) => updateUserStory(storyIndex, 'id', e.target.value)}
                        placeholder="US-001"
                      />
                      <input
                        type="number"
                        className="story-priority"
                        value={story.priority}
                        onChange={(e) => updateUserStory(storyIndex, 'priority', parseInt(e.target.value))}
                        min="1"
                      />
                      <button
                        className="remove-story"
                        onClick={() => removeUserStory(storyIndex)}
                        title="Remove story"
                      >
                        🗑️
                      </button>
                    </div>

                    <input
                      type="text"
                      className="story-title"
                      value={story.title}
                      onChange={(e) => updateUserStory(storyIndex, 'title', e.target.value)}
                      placeholder="Story title..."
                    />

                    <textarea
                      className="story-description"
                      value={story.description}
                      onChange={(e) => updateUserStory(storyIndex, 'description', e.target.value)}
                      placeholder="Story description..."
                      rows={2}
                    />

                    <div className="acceptance-criteria">
                      <label>Acceptance Criteria</label>
                      {story.acceptanceCriteria.map((criteria, criteriaIndex) => (
                        <input
                          key={criteriaIndex}
                          type="text"
                          value={criteria}
                          onChange={(e) => updateAcceptanceCriteria(storyIndex, criteriaIndex, e.target.value)}
                          placeholder={`Criterion ${criteriaIndex + 1}`}
                        />
                      ))}
                      <button
                        className="add-criteria"
                        onClick={() => addAcceptanceCriteria(storyIndex)}
                      >
                        + Add Criterion
                      </button>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>

        <div className="prd-modal-footer">
          <button className="cancel-button" onClick={onClose}>
            Cancel
          </button>
          {editedPrd && (
            <button
              className="save-button"
              onClick={handleSaveManual}
              disabled={loading}
            >
              {loading ? 'Saving...' : '💾 Save PRD'}
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
