import { useState } from 'react';
import type { ProjectCreate } from '../types';
import './ProjectCreationModal.css';

interface ProjectCreationModalProps {
  isOpen: boolean;
  onClose: () => void;
  onCreate: (project: ProjectCreate) => Promise<void>;
}

export function ProjectCreationModal({ isOpen, onClose, onCreate }: ProjectCreationModalProps) {
  const [userRequest, setUserRequest] = useState('');
  const [isLoading, setIsLoading] = useState(false);

  if (!isOpen) return null;

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    if (!userRequest.trim()) return;

    setIsLoading(true);

    try {
      // Create project and navigate directly - no delays!
      await onCreate({
        name: extractProjectName(userRequest),
        description: userRequest,
        user_request: userRequest
      });

      // Modal will close and navigate happens in onCreate callback
    } catch (error) {
      console.error('Failed to create project:', error);
      alert('Failed to create project. Please try again.');
      setIsLoading(false);
    }
  };

  const extractProjectName = (request: string): string => {
    // Extract a project name from the user request
    const words = request.toLowerCase().split(' ');
    const meaningfulWords = words.filter(w =>
      !['a', 'an', 'the', 'i', 'want', 'to', 'create', 'build', 'make'].includes(w)
    );
    return meaningfulWords.slice(0, 3).join('-') || 'new-project';
  };

  const handleClose = () => {
    setUserRequest('');
    setIsLoading(false);
    onClose();
  };

  const examplePrompts = [
    "Create a dashboard for tracking my daily habits",
    "Build a chat application with real-time messaging",
    "Make a portfolio website to showcase my projects",
    "Create an e-commerce store for selling products"
  ];

  return (
    <div className="modal-overlay" onClick={handleClose}>
      <div className="modal-content" onClick={(e) => e.stopPropagation()}>
        <button className="modal-close" onClick={handleClose}>×</button>

        <div className="modal-header">
          <h2>Create a new project</h2>
          <p>Describe what you want to build, and we'll take you to the PRD builder</p>
        </div>

        <form onSubmit={handleSubmit} className="creation-form">
          <div className="input-container">
            <textarea
              value={userRequest}
              onChange={(e) => setUserRequest(e.target.value)}
              placeholder="I want to create..."
              className="project-input"
              rows={4}
              autoFocus
            />
          </div>

          <div className="example-prompts">
            <p className="example-label">Try these examples:</p>
            {examplePrompts.map((prompt, idx) => (
              <button
                key={idx}
                type="button"
                className="example-prompt"
                onClick={() => setUserRequest(prompt)}
              >
                {prompt}
              </button>
            ))}
          </div>

          <button
            type="submit"
            className="create-button"
            disabled={!userRequest.trim() || isLoading}
          >
            {isLoading ? 'Creating...' : 'Start Building →'}
          </button>
        </form>
      </div>
    </div>
  );
}
