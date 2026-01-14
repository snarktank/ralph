/**
 * JSON Converter Service
 * Implements logic from skills/ralph/SKILL.md
 * Converts PRD markdown to prd.json format
 */

import { parsePRD, extractFeatureName } from '../utils/markdownParser.js';
import { spawn } from 'child_process';

/**
 * Check if Cursor CLI agent command is available
 */
async function isAgentAvailable() {
  return new Promise((resolve) => {
    const child = spawn('agent', ['--version'], { 
      shell: false,
      stdio: 'ignore'
    });
    
    const timeoutId = setTimeout(() => {
      child.kill();
      resolve(false);
    }, 5000);
    
    child.on('close', (code) => {
      clearTimeout(timeoutId);
      resolve(code === 0);
    });
    
    child.on('error', () => {
      clearTimeout(timeoutId);
      resolve(false);
    });
  });
}

/**
 * Build prompt for Cursor CLI agent to convert PRD to JSON
 */
function buildJSONConversionPrompt(markdown, projectName) {
  let prompt = `Convert this PRD to prd.json format for the Ralph autonomous agent system.\n\n`;
  prompt += `Project Name: ${projectName}\n\n`;
  prompt += `PRD Markdown:\n${markdown}\n\n`;
  prompt += `Please convert this PRD to the prd.json format following the structure in skills/ralph/SKILL.md. `;
  prompt += `The output JSON MUST include these top-level fields: "project" (use "${projectName}"), "branchName" (format: "ralph/[feature-name-kebab-case]" derived from PRD title), "description", and "userStories". `;
  prompt += `Each user story object in the userStories array MUST have ALL of these fields: "id" (string, format: "US-001"), "title" (string), "description" (string), "acceptanceCriteria" (array of strings), "priority" (number, 1-based ordering), "passes" (boolean, set to false), and "notes" (string, can be empty). `;
  prompt += `IMPORTANT: Every story's acceptanceCriteria array MUST include "Typecheck passes" as one of the criteria. If it's not in the PRD, add it automatically. `;
  prompt += `Stories should be ordered by dependencies (schema -> backend -> UI). `;
  prompt += `Output only the JSON content, do not save to file.`;
  
  return prompt;
}

/**
 * Extract JSON from agent output
 * Handles both wrapped (--output-format json) and unwrapped responses
 * @param {string} output - The raw output from the agent command
 * @returns {object} The extracted and parsed JSON object
 */
export function extractJSONFromOutput(output) {
  if (!output || typeof output !== 'string') {
    throw new Error('Invalid output: empty or not a string');
  }
  
  // Step 1: Try to parse output as JSON (might be the metadata wrapper)
  try {
    const parsed = JSON.parse(output);
    
    // If it's the agent metadata wrapper with a 'result' field
    if (parsed.result && typeof parsed.result === 'string') {
      return extractJSONFromString(parsed.result);
    }
    
    // If it's already the PRD JSON (has project, branchName, userStories)
    if (parsed.project || parsed.branchName || parsed.userStories) {
      return parsed;
    }
    
    // Otherwise, it might be wrapped differently
    return parsed;
  } catch (e) {
    // Not valid JSON, try extracting from text
    return extractJSONFromString(output);
  }
}

/**
 * Extract JSON from a string that might contain markdown code fences or mixed content
 */
function extractJSONFromString(text) {
  // Try to extract from markdown code fence first (```json ... ```)
  const markdownMatch = text.match(/```json\s*\n([\s\S]*?)\n```/);
  if (markdownMatch) {
    try {
      return JSON.parse(markdownMatch[1]);
    } catch (e) {
      // Continue to other methods
    }
  }
  
  // Try to extract from any code fence (``` ... ```)
  const codeMatch = text.match(/```\s*\n([\s\S]*?)\n```/);
  if (codeMatch) {
    try {
      return JSON.parse(codeMatch[1]);
    } catch (e) {
      // Continue to other methods
    }
  }
  
  // Try to find JSON object in the text
  const jsonMatch = text.match(/\{[\s\S]*"userStories"[\s\S]*\}/);
  if (jsonMatch) {
    try {
      return JSON.parse(jsonMatch[0]);
    } catch (e) {
      // Continue to other methods
    }
  }
  
  // Try to find any JSON object
  const anyJsonMatch = text.match(/\{[\s\S]*\}/);
  if (anyJsonMatch) {
    try {
      return JSON.parse(anyJsonMatch[0]);
    } catch (e) {
      // Continue to other methods
    }
  }
  
  // Last resort: try parsing the whole text
  try {
    return JSON.parse(text.trim());
  } catch (e) {
    throw new Error('Could not extract valid JSON from agent output');
  }
}

/**
 * Execute Cursor CLI agent command using spawn for proper argument handling
 * Avoids all shell escaping issues by passing arguments directly to the process
 * @param {string} prompt - The prompt to send to the agent
 * @param {string} outputFormat - The output format (text or json)
 * @param {number} timeout - Timeout in milliseconds
 * @returns {Promise<{stdout: string, stderr: string}>}
 */
export async function execAgentCommand(prompt, outputFormat = 'json', timeout = 120000) {
  return new Promise((resolve, reject) => {
    const args = ['--print', '--force', '--output-format', outputFormat, prompt];
    
    const child = spawn('agent', args, { 
      shell: false,
      stdio: ['ignore', 'pipe', 'pipe']
    });
    
    let stdout = '';
    let stderr = '';
    let timeoutId;
    
    // Set up timeout
    if (timeout > 0) {
      timeoutId = setTimeout(() => {
        child.kill('SIGTERM');
        reject(new Error('Command timeout'));
      }, timeout);
    }
    
    child.stdout.on('data', (data) => {
      stdout += data.toString();
    });
    
    child.stderr.on('data', (data) => {
      stderr += data.toString();
    });
    
    child.on('close', (code) => {
      if (timeoutId) clearTimeout(timeoutId);
      
      if (code === 0) {
        resolve({ stdout, stderr });
      } else {
        reject(new Error(`Command failed with exit code ${code}${stderr ? ': ' + stderr : ''}`));
      }
    });
    
    child.on('error', (error) => {
      if (timeoutId) clearTimeout(timeoutId);
      reject(error);
    });
  });
}

/**
 * Convert PRD to JSON using Cursor CLI agent
 */
async function convertPRDToJSONWithAgent(markdown, projectName, updateProgress = null) {
  const log = (status, message) => {
    if (updateProgress) updateProgress(status, message);
  };

  log('building', 'Building prompt for Cursor CLI agent...');
  const prompt = buildJSONConversionPrompt(markdown, projectName);
  
  try {
    log('executing', 'Executing Cursor CLI agent command...');
    // --print flag is required to enable shell execution (bash access)
    // --force flag forces allow commands unless explicitly denied
    // Use spawn to avoid shell escaping issues with long prompts
    log('waiting', 'Waiting for agent response (this may take 30-120 seconds)...');
    
    const { stdout, stderr } = await execAgentCommand(prompt, 'json', 120000);
    
    log('parsing', 'Parsing agent output...');
    const json = extractJSONFromOutput(stdout);
    
    // Validate the JSON structure
    const validation = validateJSON(json);
    if (!validation.valid) {
      throw new Error(`Invalid JSON structure: ${validation.errors.join(', ')}`);
    }
    
    log('complete', 'JSON extracted and validated');
    return json;
  } catch (error) {
    log('error', `Agent conversion failed: ${error.message}`);
    console.error('Agent conversion failed:', error.message);
    if (error.stderr) {
      console.error('Agent stderr:', error.stderr);
    }
    throw error;
  }
}

/**
 * Convert PRD markdown to JSON format
 * Uses Cursor CLI agent if available, otherwise uses template-based conversion
 * 
 * @param {Function} progressCallback - Optional callback for progress updates
 */
export async function convertPRDToJSON(markdown, projectName = 'Project', progressCallback = null) {
  const updateProgress = (status, message) => {
    if (progressCallback) {
      progressCallback({ status, message, timestamp: new Date().toISOString() });
    }
    console.log(`[JSON Conversion] ${status}: ${message}`);
  };

  updateProgress('checking', 'Checking if Cursor CLI agent is available...');
  
  // Try to use agent first if available
  const agentAvailable = await isAgentAvailable();
  
  if (agentAvailable) {
    updateProgress('generating', 'Using Cursor CLI agent to convert PRD to JSON...');
    try {
      const result = await convertPRDToJSONWithAgent(markdown, projectName, updateProgress);
      updateProgress('complete', 'JSON conversion completed successfully');
      return result;
    } catch (error) {
      // Fallback to template conversion
      updateProgress('fallback', `Agent conversion failed, using template: ${error.message}`);
      console.warn('Falling back to template conversion:', error.message);
      return convertPRDToJSONTemplate(markdown, projectName);
    }
  } else {
    // Agent not available, use template conversion
    updateProgress('template', 'Cursor CLI not available, using template conversion...');
    return convertPRDToJSONTemplate(markdown, projectName);
  }
}

/**
 * Convert PRD markdown to JSON format using template-based approach (fallback)
 */
function convertPRDToJSONTemplate(markdown, projectName = 'Project') {
  const parsed = parsePRD(markdown);
  
  // Extract feature name from title
  const featureName = extractFeatureName(parsed.title || 'feature');
  const branchName = `ralph/${featureName}`;

  // Convert user stories
  const userStories = parsed.userStories.map((story, index) => {
    // Ensure "Typecheck passes" is in acceptance criteria
    let criteria = [...story.acceptanceCriteria];
    const hasTypecheck = criteria.some(c => 
      c.toLowerCase().includes('typecheck') || 
      c.toLowerCase().includes('type check')
    );
    
    if (!hasTypecheck) {
      criteria.push('Typecheck passes');
    }

    return {
      id: story.id || `US-${String(index + 1).padStart(3, '0')}`,
      title: story.title || 'Untitled story',
      description: story.description || story.title || '',
      acceptanceCriteria: criteria,
      priority: determinePriority(story, index, parsed.userStories),
      passes: false,
      notes: ''
    };
  });

  // Validate story sizes
  validateStorySizes(userStories);

  // Order stories by dependencies
  const orderedStories = orderStoriesByDependencies(userStories);

  return {
    project: projectName,
    branchName: branchName,
    description: parsed.introduction || parsed.title || `Feature: ${featureName}`,
    userStories: orderedStories
  };
}

/**
 * Determine priority based on story content and position
 * Stories about schema/database come first, then backend, then UI
 */
function determinePriority(story, index, allStories) {
  const title = (story.title || '').toLowerCase();
  const description = (story.description || '').toLowerCase();

  // Database/schema changes get highest priority
  if (
    title.includes('database') ||
    title.includes('schema') ||
    title.includes('table') ||
    title.includes('migration') ||
    description.includes('database') ||
    description.includes('schema')
  ) {
    return 1;
  }

  // Backend/API changes get medium-high priority
  if (
    title.includes('api') ||
    title.includes('backend') ||
    title.includes('server') ||
    title.includes('service') ||
    description.includes('api') ||
    description.includes('backend')
  ) {
    return Math.min(2, allStories.length);
  }

  // UI changes get lower priority
  if (
    title.includes('ui') ||
    title.includes('component') ||
    title.includes('page') ||
    title.includes('display') ||
    title.includes('show') ||
    description.includes('ui') ||
    description.includes('component')
  ) {
    return Math.min(index + 2, allStories.length);
  }

  // Default: use index + 1
  return index + 1;
}

/**
 * Validate that stories are small enough to complete in one iteration
 */
function validateStorySizes(stories) {
  const warnings = [];

  stories.forEach((story, index) => {
    const title = story.title.toLowerCase();
    const description = story.description.toLowerCase();
    const criteriaCount = story.acceptanceCriteria.length;

    // Check for overly broad titles
    const broadTerms = [
      'entire',
      'complete',
      'full',
      'all',
      'everything',
      'whole system',
      'refactor'
    ];

    const isTooBroad = broadTerms.some(term => 
      title.includes(term) || description.includes(term)
    );

    if (isTooBroad) {
      warnings.push({
        storyId: story.id,
        issue: 'Story title/description suggests it may be too large',
        suggestion: 'Consider splitting into smaller stories'
      });
    }

    // Check for too many acceptance criteria (suggests complexity)
    if (criteriaCount > 8) {
      warnings.push({
        storyId: story.id,
        issue: `Story has ${criteriaCount} acceptance criteria, which may be too many`,
        suggestion: 'Consider splitting into smaller, focused stories'
      });
    }
  });

  return warnings;
}

/**
 * Order stories by dependencies
 * Schema -> Backend -> UI -> Dashboard
 */
function orderStoriesByDependencies(stories) {
  // Categorize stories
  const schemaStories = [];
  const backendStories = [];
  const uiStories = [];
  const otherStories = [];

  stories.forEach(story => {
    const title = story.title.toLowerCase();
    const description = story.description.toLowerCase();

    if (
      title.includes('database') ||
      title.includes('schema') ||
      title.includes('table') ||
      title.includes('migration') ||
      description.includes('database') ||
      description.includes('schema')
    ) {
      schemaStories.push(story);
    } else if (
      title.includes('api') ||
      title.includes('backend') ||
      title.includes('server') ||
      title.includes('service') ||
      description.includes('api') ||
      description.includes('backend')
    ) {
      backendStories.push(story);
    } else if (
      title.includes('ui') ||
      title.includes('component') ||
      title.includes('page') ||
      title.includes('display') ||
      title.includes('show') ||
      title.includes('filter') ||
      title.includes('dropdown') ||
      description.includes('ui') ||
      description.includes('component')
    ) {
      uiStories.push(story);
    } else {
      otherStories.push(story);
    }
  });

  // Reorder and reassign priorities
  const ordered = [...schemaStories, ...backendStories, ...uiStories, ...otherStories];
  
  ordered.forEach((story, index) => {
    story.priority = index + 1;
  });

  return ordered;
}

/**
 * Validate the converted JSON structure
 */
export function validateJSON(json) {
  const errors = [];

  if (!json.project) {
    errors.push('Missing project name');
  }

  if (!json.branchName) {
    errors.push('Missing branch name');
  }

  if (!Array.isArray(json.userStories) || json.userStories.length === 0) {
    errors.push('No user stories found');
  }

  json.userStories?.forEach((story, index) => {
    if (!story.id) {
      errors.push(`Story ${index + 1} missing ID`);
    }
    if (!story.title) {
      errors.push(`Story ${index + 1} missing title`);
    }
    if (!Array.isArray(story.acceptanceCriteria) || story.acceptanceCriteria.length === 0) {
      errors.push(`Story ${index + 1} missing acceptance criteria`);
    }
    if (typeof story.priority !== 'number') {
      errors.push(`Story ${index + 1} missing priority`);
    }
  });

  return {
    valid: errors.length === 0,
    errors
  };
}
