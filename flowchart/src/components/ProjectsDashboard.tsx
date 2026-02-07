import { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import type { Project, ProjectCreate } from '../types';
import { ProjectCreationModal } from './ProjectCreationModal';
import './ProjectsDashboard.css';

export function ProjectsDashboard() {
  const navigate = useNavigate();
  const [projects, setProjects] = useState<Project[]>([]);
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [loading, setLoading] = useState(false);

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
        await fetchProjects();
        setIsModalOpen(false);
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

              <div className="project-actions">
                <button
                  className="action-button ralph"
                  onClick={() => navigate(`/project/${project.id}/ralph`)}
                  title="Open Ralph AI Dashboard"
                >
                  🤖 Ralph
                </button>

                {project.status === 'running' ? (
                  <>
                    <a
                      href={project.url || `http://localhost:${project.port}`}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="action-button primary"
                    >
                      🚀 Open
                    </a>
                    <button
                      className="action-button secondary"
                      onClick={() => handleStopProject(project.id)}
                    >
                      Stop
                    </button>
                  </>
                ) : project.status === 'stopped' || project.status === 'created' || project.status === 'ready' ? (
                  <button
                    className="action-button primary"
                    onClick={() => handleStartProject(project.id)}
                  >
                    ▶ Start
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
    </div>
  );
}
