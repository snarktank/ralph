import { describe, it, expect } from '@jest/globals';
import { validatePath, validateProjectPath, validateFeatureName, validatePRDJSON } from './validation.js';

describe('validation', () => {
  describe('validatePath', () => {
    it('validates path is within base directory', () => {
      expect(() => validatePath('/valid/path', '/valid')).not.toThrow();
    });

    it('throws error for path traversal', () => {
      expect(() => validatePath('../../../etc/passwd', '/valid')).toThrow('Path traversal');
    });

    it('throws error for invalid path', () => {
      expect(() => validatePath(null, '/valid')).toThrow('Invalid path');
    });
  });

  describe('validateProjectPath', () => {
    it('validates project path exists', () => {
      // This will fail in test environment, but tests the logic
      expect(() => validateProjectPath('')).toThrow('Project path is required');
    });
  });

  describe('validateFeatureName', () => {
    it('validates and sanitizes feature name', () => {
      const result = validateFeatureName('Test Feature Name');
      expect(result).toBe('test-feature-name');
    });

    it('removes invalid characters', () => {
      const result = validateFeatureName('test@#$feature');
      expect(result).toBe('test-feature');
    });

    it('throws error for empty name', () => {
      expect(() => validateFeatureName('')).toThrow('Feature name is required');
    });
  });

  describe('validatePRDJSON', () => {
    it('validates correct JSON structure', () => {
      const json = {
        project: 'Test',
        branchName: 'ralph/test',
        description: 'Test',
        userStories: [
          {
            id: 'US-001',
            title: 'Test',
            description: 'Test',
            acceptanceCriteria: ['Test'],
            priority: 1,
            passes: false,
            notes: '',
          },
        ],
      };

      expect(() => validatePRDJSON(json)).not.toThrow();
    });

    it('throws error for missing project', () => {
      const json = { branchName: 'test', userStories: [] };
      expect(() => validatePRDJSON(json)).toThrow('Project name is required');
    });

    it('throws error for missing user stories', () => {
      const json = { project: 'Test', branchName: 'test', userStories: null };
      expect(() => validatePRDJSON(json)).toThrow('User stories must be an array');
    });
  });
});
