import { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import type { Project, ProjectCreate } from '../types';
import { ProjectCreationModal } from './ProjectCreationModal';
import { PRDEditorModal } from './PRDEditorModal';
import { RalphProgressViewer } from './RalphProgressViewer';
import './ProjectsDashboard.css';

export function ProjectsDashboard() {
  const navigate = useNavigate();
  const [projects, setProjects] = useState<Project[]>([]);
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [loading, setLoading] = useState(false);
  const [prdModalProjectId, setPrdModalProjectId] = useState<string | null>(null);
  const [expandedProgressId, setExpandedProgressId] = useState<string | null>(null);

  useEffect(() => {
    fetchProjects();
    // Poll for updates
    const interval = setInterval(fetchProjects, 3000);
    return () => clearInterval(interval);
  }, []);

  const fetchProjects = async () => {
    try {
      const response = await fetch('http://localhost:8000/api/projects/');
      if (response.ok) {
        const data = await response.json();
        setProjects(data.projects || []);
      }
    } catch (error) {
      console.error('Failed to fetch projects:', error);
    }
  };

  const handleCreateProject = async (projectData: ProjectCreate) => {
    try {
      const response = await fetch('http://localhost:8000/api/projects/', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(projectData)
      });

      if (response.ok) {
        const project = await response.json();
        setIsModalOpen(false);

        // Navigate to PRD Builder for the new project
        navigate(`/project/${project.id}/prd-builder`);
      }
    } catch (error) {
      console.error('Failed to create project:', error);
      throw error;
    }
  };

  const handleStartProject = async (projectId: string) => {
    try {
      const response = await fetch(`http://localhost:8000/api/projects/${projectId}/start`, {
        method: 'POST'
      });

      if (response.ok) {
        await fetchProjects();
      }
    } catch (error) {
      console.error('Failed to start project:', error);
    }
  };

  const handleStopProject = async (projectId: string) => {
    try {
      const response = await fetch(`http://localhost:8000/api/projects/${projectId}/stop`, {
        method: 'POST'
      });

      if (response.ok) {
        await fetchProjects();
      }
    } catch (error) {
      console.error('Failed to stop project:', error);
    }
  };

  const handleCreateRalphConfig = async (projectId: string) => {
    try {
      const response = await fetch(`http://localhost:8000/api/projects/${projectId}/ralph-config`, {
        method: 'POST'
      });

      if (response.ok) {
        await fetchProjects();
        alert('Ralph configuration created successfully!');
      } else {
        alert('Failed to create Ralph configuration. Make sure PRD exists first.');
      }
    } catch (error) {
      console.error('Failed to create Ralph config:', error);
      alert('Error creating Ralph configuration');
    }
  };

  const handleOpenPrdEditor = (projectId: string) => {
    setPrdModalProjectId(projectId);
  };

  const handleClosePrdEditor = () => {
    setPrdModalProjectId(null);
    fetchProjects();
  };

  const handleToggleProgress = (projectId: string) => {
    setExpandedProgressId(expandedProgressId === projectId ? null : projectId);
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'running': return '#10b981';
      case 'installing': return '#f59e0b';
      case 'stopped': return '#6b7280';
      case 'error': return '#ef4444';
      default: return '#3b82f6';
    }
  };

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'running': return '▶';
      case 'installing': return '⏳';
      case 'stopped': return '⏸';
      case 'error': return '⚠';
      default: return '📦';
    }
  };

  return (
    <div className="projects-dashboard">
      <div className="dashboard-hero">
        <div className="hero-content">
          <h1>Your Projects</h1>
          <p>Create and manage beautiful web applications with AI</p>
        </div>
        <button className="new-project-button" onClick={() => setIsModalOpen(true)}>
          <span className="plus-icon">+</span>
          New Project
        </button>
      </div>

      <div className="projects-grid">
        {projects.length === 0 ? (
          <div className="empty-state">
            <div className="empty-icon">📦</div>
            <h3>No projects yet</h3>
            <p>Create your first project to get started</p>
            <button className="create-first-button" onClick={() => setIsModalOpen(true)}>
              Create Project
            </button>
          </div>
        ) : (
          projects.map((project) => (
            <div key={project.id} className="project-card">
              <div className="project-header">
                <div className="project-info">
                  <h3>{project.name}</h3>
                  <p className="project-description">{project.description}</p>
                </div>
                <div
                  className="status-badge"
                  style={{ background: getStatusColor(project.status) }}
                >
                  <span className="status-icon">{getStatusIcon(project.status)}</span>
                  {project.status}
                </div>
              </div>

              <div className="project-meta">
                <div className="meta-item">
                  <span className="meta-label">Stack:</span>
                  <span className="meta-value">{project.stack}</span>
                </div>
                <div className="meta-item">
                  <span className="meta-label">Port:</span>
                  <span className="meta-value">{project.port}</span>
                </div>
                <div className="meta-item">
                  <span className="meta-label">URL:</span>
                  {project.status === 'running' ? (
                    <a
                      href={project.url || `http://localhost:${project.port}`}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="meta-link active"
                    >
                      {project.url || `http://localhost:${project.port}`}
                    </a>
                  ) : (
                    <span className="meta-link inactive" title="Start the project to open this URL">
                      {project.url || `http://localhost:${project.port}`}
                      <span className="status-hint"> (Start to open)</span>
                    </span>
                  )}
                </div>
                <div className="meta-item">
                  <span className="meta-label">Created:</span>
                  <span className="meta-value">
                    {new Date(project.created_at).toLocaleDateString()}
                  </span>
                </div>
              </div>

              {/* Ralph Workflow Section */}
              <div className="ralph-workflow">
                <h4 className="workflow-title">🤖 Ralph AI Workflow</h4>

                <div className="workflow-steps">
                  {/* Step 1: Create PRD */}
                  <div className={`workflow-step ${project.has_prd ? 'completed' : 'pending'}`}>
                    <div className="step-header">
                      <span className="step-number">1</span>
                      <span className="step-label">Create PRD</span>
                      {project.has_prd && <span className="step-check">✓</span>}
                    </div>
                    <button
                      className="step-button"
                      onClick={() => handleOpenPrdEditor(project.id)}
                    >
                      {project.has_prd ? '📝 Edit PRD' : '✨ Create PRD'}
                    </button>
                  </div>

                  {/* Step 2: Create Ralph Config */}
                  <div className={`workflow-step ${project.has_ralph_config ? 'completed' : project.has_prd ? 'ready' : 'disabled'}`}>
                    <div className="step-header">
                      <span className="step-number">2</span>
                      <span className="step-label">Setup Ralph</span>
                      {project.has_ralph_config && <span className="step-check">✓</span>}
                    </div>
                    <button
                      className="step-button"
                      onClick={() => handleCreateRalphConfig(project.id)}
                      disabled={!project.has_prd}
                      title={!project.has_prd ? 'Create PRD first' : ''}
                    >
                      {project.has_ralph_config ? '✅ Configured' : '⚙️ Setup'}
                    </button>
                  </div>

                  {/* Step 3: Start Ralph Loop */}
                  <div className={`workflow-step ${project.ralph_status === 'running' ? 'active' : project.has_ralph_config ? 'ready' : 'disabled'}`}>
                    <div className="step-header">
                      <span className="step-number">3</span>
                      <span className="step-label">Run Ralph</span>
                      {project.ralph_status === 'running' && <span className="step-active">●</span>}
                    </div>
                    <button
                      className="step-button"
                      onClick={() => handleToggleProgress(project.id)}
                      disabled={!project.has_ralph_config}
                      title={!project.has_ralph_config ? 'Complete setup first' : ''}
                    >
                      {expandedProgressId === project.id ? '👁️ Hide Progress' : '👀 View Progress'}
                    </button>
                  </div>
                </div>
              </div>

              {/* Inline Ralph Progress Viewer */}
              <RalphProgressViewer
                projectId={project.id}
                isExpanded={expandedProgressId === project.id}
                onClose={() => setExpandedProgressId(null)}
              />

              {/* Project Actions */}
              <div className="project-actions">
                {project.status === 'running' ? (
                  <>
                    <a
                      href={project.url || `http://localhost:${project.port}`}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="action-button primary"
                    >
                      🚀 Open App
                    </a>
                    <button
                      className="action-button secondary"
                      onClick={() => handleStopProject(project.id)}
                    >
                      Stop App
                    </button>
                  </>
                ) : project.status === 'stopped' || project.status === 'created' || project.status === 'ready' ? (
                  <button
                    className="action-button primary"
                    onClick={() => handleStartProject(project.id)}
                  >
                    ▶ Start App
                  </button>
                ) : null}

                {project.status === 'installing' && (
                  <div className="installing-indicator">
                    <div className="spinner"></div>
                    Installing...
                  </div>
                )}
              </div>
            </div>
          ))
        )}
      </div>

      <ProjectCreationModal
        isOpen={isModalOpen}
        onClose={() => setIsModalOpen(false)}
        onCreate={handleCreateProject}
      />

      {prdModalProjectId && (
        <PRDEditorModal
          isOpen={true}
          projectId={prdModalProjectId}
          projectName={projects.find(p => p.id === prdModalProjectId)?.name || ''}
          projectDescription={projects.find(p => p.id === prdModalProjectId)?.description || ''}
          onClose={handleClosePrdEditor}
          onSave={handleClosePrdEditor}
        />
      )}
    </div>
  );
}
