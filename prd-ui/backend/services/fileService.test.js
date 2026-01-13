import { describe, it, expect } from '@jest/globals';

describe('FileService', () => {
  describe('FileService structure', () => {
    it('has readFile method', async () => {
      const fileService = (await import('./fileService.js')).default;
      expect(fileService).toHaveProperty('readFile');
      expect(typeof fileService.readFile).toBe('function');
    });

    it('has writeFile method', async () => {
      const fileService = (await import('./fileService.js')).default;
      expect(fileService).toHaveProperty('writeFile');
      expect(typeof fileService.writeFile).toBe('function');
    });

    it('has savePRD method', async () => {
      const fileService = (await import('./fileService.js')).default;
      expect(fileService).toHaveProperty('savePRD');
      expect(typeof fileService.savePRD).toBe('function');
    });

    it('has listPRDs method', async () => {
      const fileService = (await import('./fileService.js')).default;
      expect(fileService).toHaveProperty('listPRDs');
      expect(typeof fileService.listPRDs).toBe('function');
    });
  });

  describe('Path validation', () => {
    it('validates paths correctly', async () => {
      const fileService = (await import('./fileService.js')).default;
      
      // Test that invalid paths throw errors
      await expect(
        fileService.readFile('../../../etc/passwd', process.cwd())
      ).rejects.toThrow();
    });
  });
});
