import express from 'express';
import fileService from '../services/fileService.js';
import { convertPRDToJSON, validateJSON } from '../services/jsonConverter.js';
import { validateProjectPath } from '../utils/validation.js';

const router = express.Router();

/**
 * Convert PRD markdown to JSON
 * POST /api/convert
 */
router.post('/', async (req, res, next) => {
  try {
    const { projectPath, prdPath, prdContent, projectName } = req.body;

    let markdown = prdContent;

    // If prdPath provided, read from file
    if (!markdown && prdPath && projectPath) {
      validateProjectPath(projectPath);
      const featureName = prdPath.replace(/^prd-/, '').replace(/\.md$/, '');
      markdown = await fileService.readPRD(projectPath, featureName);
    }

    if (!markdown) {
      return res.status(400).json({ error: 'Either prdContent or prdPath with projectPath is required' });
    }

    // Convert to JSON
    const json = convertPRDToJSON(markdown, projectName || 'Project');

    // Validate JSON
    const validation = validateJSON(json);
    if (!validation.valid) {
      return res.status(400).json({
        error: 'Invalid JSON structure',
        details: validation.errors
      });
    }

    res.json({
      json,
      validation
    });
  } catch (error) {
    next(error);
  }
});

/**
 * Save prd.json to project
 * POST /api/convert/save
 */
router.post('/save', async (req, res, next) => {
  try {
    const { projectPath, jsonData, projectName } = req.body;

    if (!projectPath) {
      return res.status(400).json({ error: 'Project path is required' });
    }

    if (!jsonData) {
      return res.status(400).json({ error: 'JSON data is required' });
    }

    validateProjectPath(projectPath);

    // Validate JSON structure
    const validation = validateJSON(jsonData);
    if (!validation.valid) {
      return res.status(400).json({
        error: 'Invalid JSON structure',
        details: validation.errors
      });
    }

    // Use provided projectName or from jsonData
    const finalJson = {
      ...jsonData,
      project: projectName || jsonData.project || 'Project'
    };

    const filePath = await fileService.savePRDJSON(projectPath, finalJson);
    
    res.json({
      success: true,
      filePath
    });
  } catch (error) {
    next(error);
  }
});

export default router;
