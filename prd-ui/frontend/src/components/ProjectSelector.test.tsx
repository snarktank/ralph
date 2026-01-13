import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import ProjectSelector from './ProjectSelector';
import * as api from '../services/api';

// Mock the API
vi.mock('../services/api', () => ({
  projectApi: {
    validate: vi.fn(),
    listPRDs: vi.fn(),
  },
}));

// Mock the useProject hook
vi.mock('../hooks/useProject', () => ({
  useProject: () => ({
    projectPath: '',
    isValidating: false,
    isValid: false,
    error: null,
    validateProject: vi.fn(),
    setProjectPath: vi.fn(),
  }),
}));

describe('ProjectSelector', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders project path input', () => {
    render(<ProjectSelector />);
    expect(screen.getByLabelText(/project path/i)).toBeInTheDocument();
  });

  it('shows validate button', () => {
    render(<ProjectSelector />);
    expect(screen.getByRole('button', { name: /validate/i })).toBeInTheDocument();
  });

  it('shows browse button', () => {
    render(<ProjectSelector />);
    expect(screen.getByRole('button', { name: /browse/i })).toBeInTheDocument();
  });

  it('allows entering project path', async () => {
    const user = userEvent.setup();
    render(<ProjectSelector />);
    
    const input = screen.getByLabelText(/project path/i);
    await user.type(input, '/path/to/project');
    
    expect(input).toHaveValue('/path/to/project');
  });

  it('disables validate button when input is empty', () => {
    render(<ProjectSelector />);
    const validateButton = screen.getByRole('button', { name: /validate/i });
    expect(validateButton).toBeDisabled();
  });
});
