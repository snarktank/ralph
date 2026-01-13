import fs from 'fs-extra';
import path from 'path';
import { validatePath, validateProjectPath, validateFeatureName } from '../utils/validation.js';

/**
 * Safe file service with path validation
 */
class FileService {
  /**
   * Read file with path validation
   */
  async readFile(filePath, baseDir = process.cwd()) {
    const validatedPath = validatePath(filePath, baseDir);
    
    if (!await fs.pathExists(validatedPath)) {
      throw new Error('File does not exist');
    }

    return await fs.readFile(validatedPath, 'utf-8');
  }

  /**
   * Write file with path validation
   */
  async writeFile(filePath, content, baseDir = process.cwd()) {
    const validatedPath = validatePath(filePath, baseDir);
    const dir = path.dirname(validatedPath);
    
    // Create directory if it doesn't exist
    await fs.ensureDir(dir);
    
    return await fs.writeFile(validatedPath, content, 'utf-8');
  }

  /**
   * Check if file exists
   */
  async fileExists(filePath, baseDir = process.cwd()) {
    try {
      const validatedPath = validatePath(filePath, baseDir);
      return await fs.pathExists(validatedPath);
    } catch {
      return false;
    }
  }

  /**
   * List files in directory
   */
  async listFiles(dirPath, baseDir = process.cwd()) {
    const validatedPath = validatePath(dirPath, baseDir);
    
    if (!await fs.pathExists(validatedPath)) {
      return [];
    }

    const files = await fs.readdir(validatedPath);
    return files.filter(file => {
      // Only return .md files for PRDs
      return file.endsWith('.md') && file.startsWith('prd-');
    });
  }

  /**
   * Create tasks directory in project if it doesn't exist
   */
  async ensureTasksDir(projectPath) {
    const validatedProjectPath = validateProjectPath(projectPath);
    const tasksDir = path.join(validatedProjectPath, 'tasks');
    await fs.ensureDir(tasksDir);
    return tasksDir;
  }

  /**
   * Save PRD file to project
   */
  async savePRD(projectPath, featureName, content) {
    const validatedProjectPath = validateProjectPath(projectPath);
    const sanitizedFeatureName = validateFeatureName(featureName);
    const tasksDir = await this.ensureTasksDir(validatedProjectPath);
    const filePath = path.join(tasksDir, `prd-${sanitizedFeatureName}.md`);
    
    await this.writeFile(filePath, content, validatedProjectPath);
    return filePath;
  }

  /**
   * Read PRD file from project
   */
  async readPRD(projectPath, featureName) {
    const validatedProjectPath = validateProjectPath(projectPath);
    const sanitizedFeatureName = validateFeatureName(featureName);
    const filePath = path.join(validatedProjectPath, 'tasks', `prd-${sanitizedFeatureName}.md`);
    
    return await this.readFile(filePath, validatedProjectPath);
  }

  /**
   * Save prd.json to project root
   */
  async savePRDJSON(projectPath, jsonData) {
    const validatedProjectPath = validateProjectPath(projectPath);
    const filePath = path.join(validatedProjectPath, 'prd.json');
    
    const content = JSON.stringify(jsonData, null, 2);
    await this.writeFile(filePath, content, validatedProjectPath);
    return filePath;
  }

  /**
   * List all PRD files in project
   */
  async listPRDs(projectPath) {
    const validatedProjectPath = validateProjectPath(projectPath);
    const tasksDir = path.join(validatedProjectPath, 'tasks');
    
    if (!await fs.pathExists(tasksDir)) {
      return [];
    }

    return await this.listFiles(tasksDir, validatedProjectPath);
  }
}

export default new FileService();
