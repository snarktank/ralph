import { useState } from 'react';
import type { ProjectCreate } from '../types';
import './ProjectCreationModal.css';

interface ProjectCreationModalProps {
  isOpen: boolean;
  onClose: () => void;
  onCreate: (project: ProjectCreate) => Promise<void>;
}

export function ProjectCreationModal({ isOpen, onClose, onCreate }: ProjectCreationModalProps) {
  const [step, setStep] = useState<'input' | 'generating' | 'success'>('input');
  const [userRequest, setUserRequest] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [generatingStatus, setGeneratingStatus] = useState('');

  if (!isOpen) return null;

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    if (!userRequest.trim()) return;

    setIsLoading(true);
    setStep('generating');
    setGeneratingStatus('Analyzing your request...');

    try {
      // Simulate status updates like Lovable
      setTimeout(() => setGeneratingStatus('Generating project structure...'), 1000);
      setTimeout(() => setGeneratingStatus('Setting up dependencies...'), 2000);
      setTimeout(() => setGeneratingStatus('Creating beautiful UI...'), 3000);

      await onCreate({
        name: extractProjectName(userRequest),
        description: userRequest,
        user_request: userRequest
      });

      setGeneratingStatus('Project created successfully!');
      setStep('success');

      // Auto-close after success
      setTimeout(() => {
        handleClose();
      }, 2000);
    } catch (error) {
      console.error('Failed to create project:', error);
      setGeneratingStatus('Failed to create project. Please try again.');
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
    setStep('input');
    setUserRequest('');
    setIsLoading(false);
    setGeneratingStatus('');
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

        {step === 'input' && (
          <>
            <div className="modal-header">
              <h2>Create a new project</h2>
              <p>Describe what you want to build, and we'll create it for you</p>
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
                Create Project
              </button>
            </form>
          </>
        )}

        {step === 'generating' && (
          <div className="generating-state">
            <div className="loader-container">
              <div className="loader"></div>
            </div>
            <h3>Creating your project...</h3>
            <p className="status-text">{generatingStatus}</p>
            <div className="progress-indicators">
              <div className="progress-dot active"></div>
              <div className="progress-dot active"></div>
              <div className="progress-dot"></div>
            </div>
          </div>
        )}

        {step === 'success' && (
          <div className="success-state">
            <div className="success-icon">✓</div>
            <h3>Project created successfully!</h3>
            <p>Your project is ready to start</p>
          </div>
        )}
      </div>
    </div>
  );
}
