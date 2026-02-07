import { useState, useEffect } from 'react';
import { useRalphStore } from '../store/useRalphStore';
import { api } from '../services/api';
import type { UserStory } from '../types';
import './PRDEditor.css';

export function PRDEditor() {
  const prd = useRalphStore((state) => state.prd);
  const setPRD = useRalphStore((state) => state.setPRD);
  const [editing, setEditing] = useState(false);
  const [jsonView, setJsonView] = useState(false);
  const [formData, setFormData] = useState({
    projectName: '',
    branchName: '',
    description: '',
  });

  useEffect(() => {
    if (prd) {
      setFormData({
        projectName: prd.projectName,
        branchName: prd.branchName,
        description: prd.description,
      });
    }
  }, [prd]);

  const handleCreate = async () => {
    try {
      const newPRD = await api.createPRD(formData);
      setPRD(newPRD);
      setEditing(false);
    } catch (error) {
      console.error('Failed to create PRD:', error);
      alert('Failed to create PRD');
    }
  };

  const handleUpdate = async () => {
    try {
      const updated = await api.updatePRD(formData);
      setPRD(updated);
      setEditing(false);
    } catch (error) {
      console.error('Failed to update PRD:', error);
      alert('Failed to update PRD');
    }
  };

  const handleAddStory = async () => {
    const title = prompt('Story title:');
    if (!title) return;

    const description = prompt('Story description:');
    if (!description) return;

    const criteria = prompt('Acceptance criteria (comma-separated):');
    const acceptanceCriteria = criteria ? criteria.split(',').map(c => c.trim()) : [];

    const priority = parseInt(prompt('Priority (number):') || '1');

    const story: UserStory = {
      id: `US-${Date.now()}`,
      title,
      description,
      acceptanceCriteria,
      priority,
      passes: false,
    };

    try {
      const updated = await api.addUserStory(story);
      setPRD(updated);
    } catch (error) {
      console.error('Failed to add story:', error);
      alert('Failed to add story');
    }
  };

  if (!prd && !editing) {
    return (
      <div className="prd-editor empty">
        <div className="empty-state">
          <h3>No PRD Found</h3>
          <p>Create a new PRD to get started</p>
          <button onClick={() => setEditing(true)} className="btn-primary">
            Create PRD
          </button>
        </div>
      </div>
    );
  }

  if (editing) {
    return (
      <div className="prd-editor">
        <div className="editor-header">
          <h2>{prd ? 'Edit PRD' : 'Create PRD'}</h2>
          <div className="header-actions">
            <button onClick={() => setEditing(false)} className="btn-secondary">
              Cancel
            </button>
            <button onClick={prd ? handleUpdate : handleCreate} className="btn-primary">
              {prd ? 'Update' : 'Create'}
            </button>
          </div>
        </div>
        <div className="editor-form">
          <div className="form-group">
            <label>Project Name</label>
            <input
              type="text"
              value={formData.projectName}
              onChange={(e) => setFormData({ ...formData, projectName: e.target.value })}
              placeholder="My Awesome Project"
            />
          </div>
          <div className="form-group">
            <label>Branch Name</label>
            <input
              type="text"
              value={formData.branchName}
              onChange={(e) => setFormData({ ...formData, branchName: e.target.value })}
              placeholder="ralph/feature-name"
            />
          </div>
          <div className="form-group">
            <label>Description</label>
            <textarea
              value={formData.description}
              onChange={(e) => setFormData({ ...formData, description: e.target.value })}
              placeholder="Description of what you want to build..."
              rows={4}
            />
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="prd-editor">
      <div className="editor-header">
        <div>
          <h2>{prd?.projectName}</h2>
          <p className="branch-name">{prd?.branchName}</p>
        </div>
        <div className="header-actions">
          <button onClick={() => setJsonView(!jsonView)} className="btn-secondary">
            {jsonView ? 'Form View' : 'JSON View'}
          </button>
          <button onClick={() => setEditing(true)} className="btn-secondary">
            Edit
          </button>
          <button onClick={handleAddStory} className="btn-primary">
            Add Story
          </button>
        </div>
      </div>

      {jsonView ? (
        <div className="json-view">
          <pre>{JSON.stringify(prd, null, 2)}</pre>
        </div>
      ) : (
        <div className="prd-content">
          <div className="prd-description">
            <p>{prd?.description}</p>
          </div>
          <div className="user-stories">
            <h3>User Stories ({prd?.userStories.length ?? 0})</h3>
            {prd?.userStories.map((story) => (
              <div key={story.id} className={`story-card ${story.passes ? 'complete' : ''}`}>
                <div className="story-header">
                  <div>
                    <span className="story-id">{story.id}</span>
                    <span className={`story-status ${story.passes ? 'complete' : 'pending'}`}>
                      {story.passes ? '✓ Complete' : '○ Pending'}
                    </span>
                  </div>
                  <span className="story-priority">Priority {story.priority}</span>
                </div>
                <h4>{story.title}</h4>
                <p>{story.description}</p>
                {story.acceptanceCriteria.length > 0 && (
                  <div className="acceptance-criteria">
                    <strong>Acceptance Criteria:</strong>
                    <ul>
                      {story.acceptanceCriteria.map((criteria, idx) => (
                        <li key={idx}>{criteria}</li>
                      ))}
                    </ul>
                  </div>
                )}
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
