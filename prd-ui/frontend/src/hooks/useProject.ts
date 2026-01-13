import { useState, useCallback } from 'react';
import { projectApi } from '../services/api';

export function useProject() {
  const [projectPath, setProjectPath] = useState<string>('');
  const [isValidating, setIsValidating] = useState(false);
  const [isValid, setIsValid] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const validateProject = useCallback(async (path: string) => {
    if (!path) {
      setIsValid(false);
      setError(null);
      return;
    }

    setIsValidating(true);
    setError(null);

    try {
      const response = await projectApi.validate(path);
      setIsValid(response.data.valid);
      if (response.data.valid) {
        setProjectPath(response.data.path);
      } else {
        setError(response.data.error || 'Invalid project path');
      }
    } catch (err: any) {
      setIsValid(false);
      setError(err.response?.data?.error || 'Failed to validate project path');
    } finally {
      setIsValidating(false);
    }
  }, []);

  const clearProject = useCallback(() => {
    setProjectPath('');
    setIsValid(false);
    setError(null);
  }, []);

  return {
    projectPath,
    isValidating,
    isValid,
    error,
    validateProject,
    clearProject,
    setProjectPath
  };
}
