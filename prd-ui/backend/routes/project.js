import express from 'express';
import fileService from '../services/fileService.js';
import { validateProjectPath } from '../utils/validation.js';

const router = express.Router();

/**
 * Validate project path
 * POST /api/project/validate
 */
router.post('/validate', async (req, res, next) => {
  try {
    const { projectPath } = req.body;

    if (!projectPath) {
      return res.status(400).json({ error: 'Project path is required' });
    }

    const validatedPath = validateProjectPath(projectPath);
    
    res.json({
      valid: true,
      path: validatedPath
    });
  } catch (error) {
    res.status(400).json({
      valid: false,
      error: error.message
    });
  }
});

/**
 * List all PRD files in project
 * GET /api/project/prds?projectPath=...
 */
router.get('/prds', async (req, res, next) => {
  try {
    const { projectPath } = req.query;

    if (!projectPath) {
      return res.status(400).json({ error: 'Project path is required' });
    }

    const prds = await fileService.listPRDs(projectPath);
    
    res.json({
      prds: prds.map(file => ({
        filename: file,
        name: file.replace(/^prd-/, '').replace(/\.md$/, '')
      }))
    });
  } catch (error) {
    res.status(400).json({ error: error.message });
  }
});

export default router;
