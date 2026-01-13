import { describe, it, expect, vi, beforeEach } from 'vitest';

// Hoist mock instance before mock factory
const mockAxiosInstance = vi.hoisted(() => ({
  post: vi.fn(),
  get: vi.fn(),
  put: vi.fn(),
}));

vi.mock('axios', () => {
  return {
    default: {
      create: vi.fn(() => mockAxiosInstance),
    },
  };
});

import { projectApi, prdApi, convertApi } from './api';

describe('API Services', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('projectApi', () => {
    it('validates project path', async () => {
      const mockResponse = { data: { valid: true, path: '/test/path' } };
      mockAxiosInstance.post.mockResolvedValue(mockResponse);

      const result = await projectApi.validate('/test/path');

      expect(mockAxiosInstance.post).toHaveBeenCalledWith('/project/validate', {
        projectPath: '/test/path',
      });
      expect(result.data).toEqual(mockResponse.data);
    });

    it('lists PRDs', async () => {
      const mockResponse = { data: { prds: [{ filename: 'prd-test.md', name: 'test' }] } };
      mockAxiosInstance.get.mockResolvedValue(mockResponse);

      const result = await projectApi.listPRDs('/test/path');

      expect(mockAxiosInstance.get).toHaveBeenCalledWith('/project/prds', {
        params: { projectPath: '/test/path' },
      });
      expect(result.data).toEqual(mockResponse.data);
    });
  });

  describe('prdApi', () => {
    it('generates questions', async () => {
      const mockResponse = { data: { questions: [] } };
      mockAxiosInstance.post.mockResolvedValue(mockResponse);

      await prdApi.generateQuestions('test feature');

      expect(mockAxiosInstance.post).toHaveBeenCalledWith('/prd/generate-questions', {
        featureDescription: 'test feature',
      });
    });

    it('creates PRD', async () => {
      const mockResponse = { data: { success: true } };
      mockAxiosInstance.post.mockResolvedValue(mockResponse);

      const data = {
        projectPath: '/test',
        featureName: 'test-feature',
        prdContent: '# Test PRD',
      };

      await prdApi.create(data);

      expect(mockAxiosInstance.post).toHaveBeenCalledWith('/prd/create', data);
    });
  });

  describe('convertApi', () => {
    it('converts PRD to JSON', async () => {
      const mockResponse = { data: { json: { project: 'Test' } } };
      mockAxiosInstance.post.mockResolvedValue(mockResponse);

      const data = { prdContent: '# Test PRD' };
      await convertApi.convert(data);

      expect(mockAxiosInstance.post).toHaveBeenCalledWith('/convert', data);
    });

    it('saves JSON', async () => {
      const mockResponse = { data: { success: true } };
      mockAxiosInstance.post.mockResolvedValue(mockResponse);

      const data = {
        projectPath: '/test',
        jsonData: { project: 'Test' },
      };

      await convertApi.save(data);

      expect(mockAxiosInstance.post).toHaveBeenCalledWith('/convert/save', data);
    });
  });
});
