#!/usr/bin/env node
/**
 * task-converter.js
 * Converts Ralph PRD JSON to Claude Code hierarchical tasks
 */

import { execSync } from 'child_process';
import fs from 'fs';

/**
 * Execute a Claude Code command and return parsed JSON output
 */
function claudeTask(command, args = {}) {
  try {
    const cmd = `claude task ${command} --json`;
    const output = execSync(cmd, { 
      encoding: 'utf8', 
      input: JSON.stringify(args),
      stdio: ['pipe', 'pipe', 'inherit'] 
    });
    const output = execSync(cmd, { encoding: 'utf8', stdio: ['pipe', 'pipe', 'inherit'] });
    return JSON.parse(output);
  } catch (error) {
    console.error(`Error executing claude task ${command}:`, error.message);
    throw error;
  }
}

/**
 * Detect dependencies based on story content and keywords
 */
function detectDependencies(story, allStories, taskIds) {
  const blockers = [];

  // Check for manual dependencies field first
  if (story.dependencies && Array.isArray(story.dependencies)) {
    story.dependencies.forEach(depId => {
      if (taskIds.has(depId)) {
        blockers.push(taskIds.get(depId));
      }
    });
    return blockers;
  }

  // Combine story title and criteria for analysis
  const storyText = `${story.title} ${story.description} ${story.acceptanceCriteria.join(' ')}`.toLowerCase();

  // Pattern 1: Explicit reference to other story (e.g., "requires US-001")
  const explicitMatch = storyText.match(/requires?\s+(US-\d+)|depends?\s+on\s+(US-\d+)|after\s+(US-\d+)/i);
  if (explicitMatch) {
    const depId = explicitMatch[1] || explicitMatch[2] || explicitMatch[3];
    if (taskIds.has(depId.toUpperCase())) {
      blockers.push(taskIds.get(depId.toUpperCase()));
    }
  }

  // Pattern 2: Layered dependencies (schema → backend → UI)
  const hasKeyword = (keywords) => keywords.some(kw => storyText.includes(kw.toLowerCase()));

  // If this is a database/schema story, no dependencies (it's foundational)
  if (hasKeyword(['database', 'schema', 'migration', 'table', 'column'])) {
    return blockers; // Return early, schema stories don't depend on others
  }

  // If this is a backend/API story, it depends on schema stories
  if (hasKeyword(['api', 'endpoint', 'server action', 'backend', 'service', 'function'])) {
    const schemaDeps = allStories.filter(s => {
      const sText = `${s.title} ${s.description}`.toLowerCase();
      return hasKeyword(['database', 'schema', 'migration', 'table']) &&
             taskIds.has(s.id) &&
             s.priority < story.priority; // Only earlier stories
    });
    schemaDeps.forEach(s => blockers.push(taskIds.get(s.id)));
  }

  // If this is a UI/frontend story, it depends on backend AND schema stories
  if (hasKeyword(['ui', 'component', 'page', 'form', 'button', 'dropdown', 'modal', 'display', 'show'])) {
    const backendDeps = allStories.filter(s => {
      const sText = `${s.title} ${s.description}`.toLowerCase();
      return hasKeyword(['api', 'endpoint', 'server action', 'backend']) &&
             taskIds.has(s.id) &&
             s.priority < story.priority;
    });
    const schemaDeps = allStories.filter(s => {
      const sText = `${s.title} ${s.description}`.toLowerCase();
      return hasKeyword(['database', 'schema', 'migration', 'table']) &&
             taskIds.has(s.id) &&
             s.priority < story.priority;
    });
    backendDeps.forEach(s => blockers.push(taskIds.get(s.id)));
    schemaDeps.forEach(s => blockers.push(taskIds.get(s.id)));
  }

  // Deduplicate
  return [...new Set(blockers)];
}

/**
 * Convert PRD JSON to hierarchical Claude Code tasks
 */
export async function convertPrdToTasks(prdPath) {
  console.log(`Converting PRD to tasks: ${prdPath}`);

  // Read PRD
  const prd = JSON.parse(fs.readFileSync(prdPath, 'utf8'));
  const taskIds = new Map(); // storyId → parent task ID
  const stats = {
    parentTasks: 0,
    childTasks: 0,
    dependencies: 0
  };

  // Create tasks for each user story
  for (const story of prd.userStories) {
    // Create parent task for the story
    const parentResult = claudeTask('create', {
      subject: `[${story.id}] ${story.title}`,
      description: story.description,
      activeForm: `Working on ${story.title}`,
      metadata: {
        storyId: story.id,
        priority: story.priority,
        storyTitle: story.title,
        totalCriteria: story.acceptanceCriteria.length,
        type: 'parent',
        passes: story.passes || false
      }
    });

    const parentId = parentResult.id;
    taskIds.set(story.id, parentId);
    stats.parentTasks++;

    console.log(`  Created parent task #${parentId}: [${story.id}] ${story.title}`);

    // Create child tasks for each acceptance criterion
    const childIds = [];
    for (let i = 0; i < story.acceptanceCriteria.length; i++) {
      const criterion = story.acceptanceCriteria[i];

      const childResult = claudeTask('create', {
        subject: `[${story.id}-AC${i+1}] ${criterion.substring(0, 60)}${criterion.length > 60 ? '...' : ''}`,
        description: `Acceptance criterion ${i+1} for ${story.title}: ${criterion}`,
        activeForm: `Implementing: ${criterion.substring(0, 50)}...`,
        metadata: {
          storyId: story.id,
          parentTaskId: parentId,
          criterionIndex: i,
          criterionText: criterion,
          type: 'child',
          requiresTypecheck: criterion.toLowerCase().includes('typecheck'),
          requiresTests: criterion.toLowerCase().includes('test'),
          requiresBrowserVerification: criterion.toLowerCase().includes('browser')
        }
      });

      const childId = childResult.id;
      childIds.push(childId);
      stats.childTasks++;

      console.log(`    Created child task #${childId}: [${story.id}-AC${i+1}]`);

      // Add dependency: child blocked by previous child (sequential execution within story)
      if (i > 0) {
        claudeTask('update', {
          taskId: childId,
          addBlockedBy: [childIds[i-1]]
        });
        stats.dependencies++;
      }
    }
  }

  // Second pass: Add cross-story dependencies
  for (const story of prd.userStories) {
    const deps = detectDependencies(story, prd.userStories, taskIds);

    if (deps.length > 0) {
      const parentId = taskIds.get(story.id);

      // Make the parent task blocked by dependency parent tasks
      claudeTask('update', {
        taskId: parentId,
        addBlockedBy: deps
      });

      stats.dependencies += deps.length;
      console.log(`  Story ${story.id} depends on: ${deps.join(', ')}`);
    }
  }

  console.log(`\nTask creation complete:`);
  console.log(`  Parent tasks: ${stats.parentTasks}`);
  console.log(`  Child tasks: ${stats.childTasks}`);
  console.log(`  Dependencies: ${stats.dependencies}`);

  return stats;
}

/**
 * Sync task status back to prd.json
 * Updates prd.json when all child tasks for a story are completed
 */
export async function syncTasksToPrd(taskListId, prdPath) {
  console.log(`Syncing tasks to PRD: ${prdPath}`);

  // Get all tasks
  const tasks = claudeTask('list', {});

  // Read PRD
  const prd = JSON.parse(fs.readFileSync(prdPath, 'utf8'));
  let updated = false;

  // Group tasks by story ID
  const tasksByStory = new Map();
  tasks.forEach(task => {
    if (task.metadata && task.metadata.storyId) {
      if (!tasksByStory.has(task.metadata.storyId)) {
        tasksByStory.set(task.metadata.storyId, []);
      }
      tasksByStory.get(task.metadata.storyId).push(task);
    }
  });

  // Check each story
  for (const story of prd.userStories) {
    const storyTasks = tasksByStory.get(story.id) || [];
    const parentTask = storyTasks.find(t => t.metadata.type === 'parent');

    if (parentTask && parentTask.status === 'completed' && !story.passes) {
      console.log(`  Marking story ${story.id} as complete`);
      story.passes = true;
      updated = true;
    }
  }

  if (updated) {
    fs.writeFileSync(prdPath, JSON.stringify(prd, null, 2));
    console.log('PRD updated successfully');
  } else {
    console.log('No updates needed');
  }

  return updated;
}

/**
 * Sync prd.json changes back to tasks
 * Useful when user manually edits prd.json
 */
export async function syncPrdToTasks(prdPath, taskListId) {
  console.log(`Syncing PRD to tasks: ${prdPath}`);

  // Get all tasks
  const tasks = claudeTask('list', {});

  // Read PRD
  const prd = JSON.parse(fs.readFileSync(prdPath, 'utf8'));
  let updated = 0;

  // Group tasks by story ID
  const tasksByStory = new Map();
  tasks.forEach(task => {
    if (task.metadata && task.metadata.storyId) {
      if (!tasksByStory.has(task.metadata.storyId)) {
        tasksByStory.set(task.metadata.storyId, []);
      }
      tasksByStory.get(task.metadata.storyId).push(task);
    }
  });

  // Sync each story
  for (const story of prd.userStories) {
    const storyTasks = tasksByStory.get(story.id) || [];
    const parentTask = storyTasks.find(t => t.metadata.type === 'parent');

    if (parentTask) {
      // If PRD says passes=true but task not completed, mark task completed
      if (story.passes && parentTask.status !== 'completed') {
        claudeTask('update', {
          taskId: parentTask.id,
          status: 'completed'
        });

        // Also mark all child tasks completed
        const childTasks = storyTasks.filter(t => t.metadata.type === 'child');
        childTasks.forEach(child => {
          if (child.status !== 'completed') {
            claudeTask('update', {
              taskId: child.id,
              status: 'completed'
            });
          }
        });

        console.log(`  Marked story ${story.id} tasks as completed`);
        updated++;
      }
    }
  }

  console.log(`Sync complete: ${updated} stories updated`);
  return updated;
}

/**
 * Check if task system is available
 */
export function checkTaskSystemAvailable() {
  try {
    execSync('claude task list --json', { encoding: 'utf8', stdio: 'pipe' });
    return true;
  } catch (error) {
    return false;
  }
}
