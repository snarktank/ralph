/**
 * JSON Converter Service
 * Implements logic from skills/ralph/SKILL.md
 * Converts PRD markdown to prd.json format
 */

import { parsePRD, extractFeatureName } from '../utils/markdownParser.js';

/**
 * Convert PRD markdown to JSON format
 */
export function convertPRDToJSON(markdown, projectName = 'Project') {
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
