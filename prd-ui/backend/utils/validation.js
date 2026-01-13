import path from 'path';
import { existsSync } from 'fs';

/**
 * Validate and sanitize file paths to prevent directory traversal
 */
export function validatePath(filePath, baseDir) {
  if (!filePath || typeof filePath !== 'string') {
    throw new Error('Invalid path');
  }

  // Resolve the path
  const resolvedPath = path.resolve(filePath);
  const resolvedBase = path.resolve(baseDir);

  // Check if path is within base directory
  if (!resolvedPath.startsWith(resolvedBase)) {
    throw new Error('Path traversal detected');
  }

  return resolvedPath;
}

/**
 * Validate project path exists and is accessible
 */
export function validateProjectPath(projectPath) {
  if (!projectPath || typeof projectPath !== 'string') {
    throw new Error('Project path is required');
  }

  const resolvedPath = path.resolve(projectPath);
  
  if (!existsSync(resolvedPath)) {
    throw new Error('Project path does not exist');
  }

  return resolvedPath;
}

/**
 * Validate feature name for file naming
 */
export function validateFeatureName(featureName) {
  if (!featureName || typeof featureName !== 'string') {
    throw new Error('Feature name is required');
  }

  // Remove invalid characters for file names
  const sanitized = featureName
    .toLowerCase()
    .replace(/[^a-z0-9-]/g, '-')
    .replace(/-+/g, '-')
    .replace(/^-|-$/g, '');

  if (!sanitized) {
    throw new Error('Feature name must contain valid characters');
  }

  return sanitized;
}

/**
 * Validate JSON structure
 */
export function validatePRDJSON(json) {
  if (!json || typeof json !== 'object') {
    throw new Error('Invalid JSON structure');
  }

  if (!json.project || typeof json.project !== 'string') {
    throw new Error('Project name is required');
  }

  if (!json.branchName || typeof json.branchName !== 'string') {
    throw new Error('Branch name is required');
  }

  if (!Array.isArray(json.userStories)) {
    throw new Error('User stories must be an array');
  }

  // Validate each user story
  json.userStories.forEach((story, index) => {
    if (!story.id) {
      throw new Error(`User story ${index + 1} missing ID`);
    }
    if (!story.title) {
      throw new Error(`User story ${index + 1} missing title`);
    }
    if (!Array.isArray(story.acceptanceCriteria)) {
      throw new Error(`User story ${index + 1} missing acceptance criteria`);
    }
    if (typeof story.priority !== 'number') {
      throw new Error(`User story ${index + 1} missing priority`);
    }
  });

  return true;
}
