/**
 * PRD Generator Service
 * Implements logic from skills/prd/SKILL.md
 */

import { spawn } from 'child_process';

/**
 * Generate clarifying questions based on feature description
 */
export function generateQuestions(featureDescription) {
  // Basic questions that can be customized based on feature description
  const questions = [
    {
      id: 'goal',
      text: 'What is the primary goal of this feature?',
      options: [
        'Improve user experience',
        'Increase user engagement',
        'Reduce support burden',
        'Add new functionality',
        'Other (please specify)'
      ]
    },
    {
      id: 'targetUser',
      text: 'Who is the target user?',
      options: [
        'New users only',
        'Existing users only',
        'All users',
        'Admin users only',
        'Other (please specify)'
      ]
    },
    {
      id: 'scope',
      text: 'What is the scope?',
      options: [
        'Minimal viable version',
        'Full-featured implementation',
        'Just the backend/API',
        'Just the UI',
        'Other (please specify)'
      ]
    },
    {
      id: 'priority',
      text: 'What is the priority level?',
      options: [
        'High - Critical for business',
        'Medium - Important but not urgent',
        'Low - Nice to have',
        'Other (please specify)'
      ]
    }
  ];

  return questions;
}

/**
 * Generate PRD markdown from feature description and answers
 * Follows structure from skills/prd/SKILL.md
 * Uses Cursor CLI agent if available, otherwise uses template generation
 * 
 * @param {Function} progressCallback - Optional callback for progress updates (status, message)
 */
export async function generatePRD(featureDescription, answers, projectName = 'Project', progressCallback = null) {
  const updateProgress = (status, message) => {
    if (progressCallback) {
      progressCallback({ status, message, timestamp: new Date().toISOString() });
    }
    console.log(`[PRD Generation] ${status}: ${message}`);
  };

  updateProgress('checking', 'Checking if Cursor CLI agent is available...');
  
  // Try to use agent first if available
  const agentAvailable = await isAgentAvailable();
  
  if (agentAvailable) {
    updateProgress('generating', 'Using Cursor CLI agent to generate PRD...');
    try {
      const result = await generatePRDWithAgent(featureDescription, answers, projectName, updateProgress);
      updateProgress('complete', 'PRD generated successfully');
      return result;
    } catch (error) {
      // Fallback to template generation
      updateProgress('fallback', `Agent generation failed, using template: ${error.message}`);
      console.warn('Falling back to template generation:', error.message);
      return generatePRDTemplate(featureDescription, answers, projectName);
    }
  } else {
    // Agent not available, use template generation
    updateProgress('template', 'Cursor CLI not available, using template generation...');
    return generatePRDTemplate(featureDescription, answers, projectName);
  }
}

/**
 * Generate PRD using template-based approach (fallback)
 * Follows structure from skills/prd/SKILL.md
 */
function generatePRDTemplate(featureDescription, answers, projectName = 'Project') {
  const featureName = extractFeatureName(featureDescription);
  const title = `PRD: ${featureName}`;

  let prd = `# ${title}\n\n`;

  // 1. Introduction/Overview
  prd += `## Introduction\n\n`;
  prd += generateIntroduction(featureDescription, answers);
  prd += `\n`;

  // 2. Goals
  prd += `## Goals\n\n`;
  prd += generateGoals(featureDescription, answers);
  prd += `\n`;

  // 3. User Stories
  prd += `## User Stories\n\n`;
  prd += generateUserStories(featureDescription, answers);
  prd += `\n`;

  // 4. Functional Requirements
  prd += `## Functional Requirements\n\n`;
  prd += generateFunctionalRequirements(featureDescription, answers);
  prd += `\n`;

  // 5. Non-Goals (Out of Scope)
  prd += `## Non-Goals (Out of Scope)\n\n`;
  prd += generateNonGoals(featureDescription, answers);
  prd += `\n`;

  // 6. Design Considerations
  prd += `## Design Considerations\n\n`;
  prd += generateDesignConsiderations(featureDescription, answers);
  prd += `\n`;

  // 7. Technical Considerations
  prd += `## Technical Considerations\n\n`;
  prd += generateTechnicalConsiderations(featureDescription, answers);
  prd += `\n`;

  // 8. Success Metrics
  prd += `## Success Metrics\n\n`;
  prd += generateSuccessMetrics(featureDescription, answers);
  prd += `\n`;

  // 9. Open Questions
  prd += `## Open Questions\n\n`;
  prd += generateOpenQuestions(featureDescription, answers);
  prd += `\n`;

  return prd;
}

/**
 * Extract feature name from description
 * Converts to kebab-case for filenames
 */
function extractFeatureName(description) {
  // Get first line or first sentence
  const firstLine = description.split('\n')[0].split('.')[0];
  // Take first 5-7 words
  const words = firstLine.split(' ').slice(0, 7).filter(w => w.length > 0);
  return words.join(' ');
}

/**
 * Generate Introduction section
 */
function generateIntroduction(featureDescription, answers) {
  let intro = featureDescription.trim();
  
  // Add problem statement if goal is provided
  if (answers.goal) {
    intro += `\n\nThis feature addresses the goal of ${answers.goal.toLowerCase()}.`;
  }
  
  // Add target user context
  if (answers.targetUser) {
    intro += ` The primary target users are ${answers.targetUser.toLowerCase()}.`;
  }
  
  return intro;
}

/**
 * Generate Goals section
 * Uses answers to create specific, measurable goals
 */
function generateGoals(featureDescription, answers) {
  const goals = [];
  
  // Primary goal from answers
  if (answers.goal) {
    const goalText = answers.goal.toLowerCase();
    if (goalText.includes('user experience')) {
      goals.push('Improve user experience and usability');
    } else if (goalText.includes('engagement')) {
      goals.push('Increase user engagement and interaction');
    } else if (goalText.includes('support')) {
      goals.push('Reduce support burden and user confusion');
    } else if (goalText.includes('functionality')) {
      goals.push('Add new functionality to meet user needs');
    } else {
      goals.push(`Achieve the goal: ${answers.goal}`);
    }
  }
  
  // Scope-based goals
  if (answers.scope) {
    const scope = answers.scope.toLowerCase();
    if (scope.includes('minimal') || scope.includes('mvp')) {
      goals.push('Deliver a minimal viable version that provides core value');
    } else if (scope.includes('full')) {
      goals.push('Deliver a complete, full-featured implementation');
    } else if (scope.includes('backend') || scope.includes('api')) {
      goals.push('Implement backend/API functionality');
    } else if (scope.includes('ui')) {
      goals.push('Implement user interface components and interactions');
    }
  }
  
  // Feature-specific goals
  const descLower = featureDescription.toLowerCase();
  if (descLower.includes('filter') || descLower.includes('search')) {
    goals.push('Enable users to efficiently find and filter content');
  }
  if (descLower.includes('create') || descLower.includes('add')) {
    goals.push('Allow users to create or add new items');
  }
  if (descLower.includes('edit') || descLower.includes('update')) {
    goals.push('Enable users to modify existing content');
  }
  if (descLower.includes('delete') || descLower.includes('remove')) {
    goals.push('Allow users to remove unwanted items');
  }
  if (descLower.includes('priority') || descLower.includes('sort')) {
    goals.push('Help users organize and prioritize their work');
  }
  
  // Default goals if none generated
  if (goals.length === 0) {
    goals.push('Implement the feature as described');
    goals.push('Ensure type safety and code quality');
    goals.push('Maintain existing functionality');
  }
  
  return goals.map(goal => `- ${goal}`).join('\n');
}

/**
 * Analyze feature description to determine what components are needed
 */
function analyzeFeature(description) {
  const descLower = description.toLowerCase();
  
  return {
    needsDatabase: descLower.includes('database') || 
                   descLower.includes('schema') || 
                   descLower.includes('table') || 
                   descLower.includes('model') ||
                   descLower.includes('store') ||
                   descLower.includes('save') ||
                   descLower.includes('persist'),
    needsBackend: descLower.includes('api') || 
                  descLower.includes('endpoint') || 
                  descLower.includes('server') ||
                  descLower.includes('backend') ||
                  descLower.includes('route') ||
                  !descLower.includes('ui only') && !descLower.includes('frontend only'),
    needsUI: descLower.includes('ui') || 
             descLower.includes('interface') || 
             descLower.includes('component') || 
             descLower.includes('page') ||
             descLower.includes('button') ||
             descLower.includes('form') ||
             descLower.includes('display') ||
             descLower.includes('show') ||
             descLower.includes('view') ||
             !descLower.includes('backend only') && !descLower.includes('api only'),
    isFullStack: !descLower.includes('backend only') && 
                 !descLower.includes('api only') && 
                 !descLower.includes('ui only') &&
                 !descLower.includes('frontend only')
  };
}

/**
 * Generate User Stories section
 * Breaks down feature into multiple small, implementable stories
 */
function generateUserStories(featureDescription, answers) {
  const analysis = analyzeFeature(featureDescription);
  const scope = answers.scope?.toLowerCase() || '';
  const targetUser = answers.targetUser || 'user';
  
  const stories = [];
  let storyNumber = 1;
  
  // Determine what to include based on scope
  const includeDatabase = analysis.needsDatabase && 
                          !scope.includes('ui only') && 
                          !scope.includes('frontend only');
  const includeBackend = analysis.needsBackend && 
                         !scope.includes('ui only') && 
                         !scope.includes('frontend only');
  const includeUI = analysis.needsUI && 
                    !scope.includes('backend only') && 
                    !scope.includes('api only');
  
  // Story 1: Database/Schema changes (if needed)
  if (includeDatabase) {
    stories.push({
      id: `US-${String(storyNumber).padStart(3, '0')}`,
      title: 'Add database schema changes',
      description: `As a developer, I need to store data for ${extractFeatureName(featureDescription).toLowerCase()} so it persists across sessions.`,
      criteria: [
        'Add necessary database columns/tables',
        'Generate and run migration successfully',
        'Typecheck passes'
      ],
      hasUI: false
    });
    storyNumber++;
  }
  
  // Story 2: Backend API/Logic (if needed)
  if (includeBackend) {
    const descLower = featureDescription.toLowerCase();
    let backendTitle = 'Implement backend functionality';
    let backendDesc = `As a developer, I need backend logic for ${extractFeatureName(featureDescription).toLowerCase()} so the feature works correctly.`;
    
    if (descLower.includes('api') || descLower.includes('endpoint')) {
      backendTitle = 'Add API endpoints';
      backendDesc = `As a developer, I need API endpoints for ${extractFeatureName(featureDescription).toLowerCase()} so frontend can interact with the feature.`;
    } else if (descLower.includes('create') || descLower.includes('add')) {
      backendTitle = 'Implement create functionality';
      backendDesc = `As a ${targetUser.toLowerCase()}, I want to create new items so I can add content to the system.`;
    } else if (descLower.includes('edit') || descLower.includes('update')) {
      backendTitle = 'Implement update functionality';
      backendDesc = `As a ${targetUser.toLowerCase()}, I want to update existing items so I can modify content.`;
    } else if (descLower.includes('delete') || descLower.includes('remove')) {
      backendTitle = 'Implement delete functionality';
      backendDesc = `As a ${targetUser.toLowerCase()}, I want to delete items so I can remove unwanted content.`;
    }
    
    stories.push({
      id: `US-${String(storyNumber).padStart(3, '0')}`,
      title: backendTitle,
      description: backendDesc,
      criteria: [
        'Backend logic follows project conventions',
        'Proper error handling and validation',
        'Typecheck passes'
      ],
      hasUI: false
    });
    storyNumber++;
  }
  
  // Story 3+: UI Components (if needed)
  if (includeUI) {
    const descLower = featureDescription.toLowerCase();
    
    // Display/List story
    if (descLower.includes('list') || descLower.includes('display') || descLower.includes('show') || descLower.includes('view')) {
      stories.push({
        id: `US-${String(storyNumber).padStart(3, '0')}`,
        title: 'Display feature content',
        description: `As a ${targetUser.toLowerCase()}, I want to see ${extractFeatureName(featureDescription).toLowerCase()} so I can view the information.`,
        criteria: [
          'Content displays correctly',
          'Layout is responsive and accessible',
          'Typecheck passes',
          'Verify in browser using dev-browser skill'
        ],
        hasUI: true
      });
      storyNumber++;
    }
    
    // Form/Input story
    if (descLower.includes('form') || descLower.includes('input') || descLower.includes('create') || descLower.includes('add') || descLower.includes('edit')) {
      stories.push({
        id: `US-${String(storyNumber).padStart(3, '0')}`,
        title: 'Add input form',
        description: `As a ${targetUser.toLowerCase()}, I want to input data for ${extractFeatureName(featureDescription).toLowerCase()} so I can create or modify content.`,
        criteria: [
          'Form fields are properly labeled',
          'Validation provides clear error messages',
          'Form submission works correctly',
          'Typecheck passes',
          'Verify in browser using dev-browser skill'
        ],
        hasUI: true
      });
      storyNumber++;
    }
    
    // Filter/Search story
    if (descLower.includes('filter') || descLower.includes('search') || descLower.includes('sort')) {
      stories.push({
        id: `US-${String(storyNumber).padStart(3, '0')}`,
        title: 'Add filtering and search',
        description: `As a ${targetUser.toLowerCase()}, I want to filter or search ${extractFeatureName(featureDescription).toLowerCase()} so I can find specific content.`,
        criteria: [
          'Filter/search controls are visible and accessible',
          'Results update in real-time or on submit',
          'Empty states are handled gracefully',
          'Typecheck passes',
          'Verify in browser using dev-browser skill'
        ],
        hasUI: true
      });
      storyNumber++;
    }
    
    // Action button story
    if (descLower.includes('button') || descLower.includes('action') || descLower.includes('click')) {
      stories.push({
        id: `US-${String(storyNumber).padStart(3, '0')}`,
        title: 'Add action buttons',
        description: `As a ${targetUser.toLowerCase()}, I want to perform actions on ${extractFeatureName(featureDescription).toLowerCase()} so I can interact with the feature.`,
        criteria: [
          'Buttons are clearly labeled',
          'Actions execute correctly',
          'Loading states are shown during operations',
          'Typecheck passes',
          'Verify in browser using dev-browser skill'
        ],
        hasUI: true
      });
      storyNumber++;
    }
    
    // If no specific UI patterns detected, add generic UI story
    if (storyNumber === (includeDatabase ? 2 : 1) + (includeBackend ? 1 : 0) + 1) {
      stories.push({
        id: `US-${String(storyNumber).padStart(3, '0')}`,
        title: 'Implement user interface',
        description: `As a ${targetUser.toLowerCase()}, I want to interact with ${extractFeatureName(featureDescription).toLowerCase()} through a user interface so I can use the feature.`,
        criteria: [
          'UI components are properly structured',
          'User interactions work as expected',
          'Typecheck passes',
          'Verify in browser using dev-browser skill'
        ],
        hasUI: true
      });
      storyNumber++;
    }
  }
  
  // If no stories generated, create at least one
  if (stories.length === 0) {
    stories.push({
      id: 'US-001',
      title: 'Implement feature',
      description: `As a ${targetUser.toLowerCase()}, I want ${extractFeatureName(featureDescription).toLowerCase()} so I can benefit from this functionality.`,
      criteria: [
        'Implementation follows project conventions',
        'Typecheck passes'
      ],
      hasUI: includeUI
    });
  }
  
  // Format stories as markdown
  let prdStories = '';
  stories.forEach((story) => {
    prdStories += `### ${story.id}: ${story.title}\n`;
    prdStories += `**Description:** ${story.description}\n\n`;
    prdStories += `**Acceptance Criteria:**\n`;
    story.criteria.forEach(criterion => {
      prdStories += `- [ ] ${criterion}\n`;
    });
    if (story.hasUI) {
      prdStories += `- [ ] Verify in browser using dev-browser skill\n`;
    }
    prdStories += `\n`;
  });
  
  return prdStories;
}

/**
 * Generate Functional Requirements section
 * Numbered FR-1, FR-2, etc.
 */
function generateFunctionalRequirements(featureDescription, answers) {
  const requirements = [];
  const descLower = featureDescription.toLowerCase();
  const analysis = analyzeFeature(featureDescription);
  let frNumber = 1;
  
  // Database requirements
  if (analysis.needsDatabase) {
    requirements.push(`FR-${frNumber}: Add database schema changes to support ${extractFeatureName(featureDescription).toLowerCase()}`);
    frNumber++;
  }
  
  // Backend requirements
  if (analysis.needsBackend) {
    if (descLower.includes('create') || descLower.includes('add')) {
      requirements.push(`FR-${frNumber}: Allow users to create new ${extractFeatureName(featureDescription).toLowerCase()} items`);
      frNumber++;
    }
    if (descLower.includes('read') || descLower.includes('view') || descLower.includes('get')) {
      requirements.push(`FR-${frNumber}: Allow users to retrieve ${extractFeatureName(featureDescription).toLowerCase()} data`);
      frNumber++;
    }
    if (descLower.includes('update') || descLower.includes('edit') || descLower.includes('modify')) {
      requirements.push(`FR-${frNumber}: Allow users to update existing ${extractFeatureName(featureDescription).toLowerCase()} items`);
      frNumber++;
    }
    if (descLower.includes('delete') || descLower.includes('remove')) {
      requirements.push(`FR-${frNumber}: Allow users to delete ${extractFeatureName(featureDescription).toLowerCase()} items`);
      frNumber++;
    }
    if (descLower.includes('api') || descLower.includes('endpoint')) {
      requirements.push(`FR-${frNumber}: Provide API endpoints for ${extractFeatureName(featureDescription).toLowerCase()} operations`);
      frNumber++;
    }
  }
  
  // UI requirements
  if (analysis.needsUI) {
    if (descLower.includes('display') || descLower.includes('show') || descLower.includes('list')) {
      requirements.push(`FR-${frNumber}: Display ${extractFeatureName(featureDescription).toLowerCase()} in the user interface`);
      frNumber++;
    }
    if (descLower.includes('form') || descLower.includes('input')) {
      requirements.push(`FR-${frNumber}: Provide input forms for ${extractFeatureName(featureDescription).toLowerCase()}`);
      frNumber++;
    }
    if (descLower.includes('filter') || descLower.includes('search')) {
      requirements.push(`FR-${frNumber}: Enable filtering and searching of ${extractFeatureName(featureDescription).toLowerCase()}`);
      frNumber++;
    }
    if (descLower.includes('button') || descLower.includes('action')) {
      requirements.push(`FR-${frNumber}: Provide action buttons for ${extractFeatureName(featureDescription).toLowerCase()} operations`);
      frNumber++;
    }
  }
  
  // Validation and error handling
  requirements.push(`FR-${frNumber}: Ensure proper validation and error handling for all operations`);
  frNumber++;
  
  // Default if none generated
  if (requirements.length === 0) {
    requirements.push('FR-1: Implement core functionality as described');
    requirements.push('FR-2: Ensure proper error handling and validation');
  }
  
  return requirements.join('\n');
}

/**
 * Generate Non-Goals section
 */
function generateNonGoals(featureDescription, answers) {
  const nonGoals = [];
  const scope = answers.scope?.toLowerCase() || '';
  const descLower = featureDescription.toLowerCase();
  
  // Scope-based non-goals
  if (scope.includes('backend') || scope.includes('api')) {
    nonGoals.push('No UI changes or frontend modifications');
  } else if (scope.includes('ui') || scope.includes('frontend')) {
    nonGoals.push('No backend changes or API modifications');
  }
  
  if (scope.includes('minimal') || scope.includes('mvp')) {
    nonGoals.push('No advanced features beyond the minimal viable version');
  }
  
  // Feature-specific non-goals
  if (!descLower.includes('notification') && !descLower.includes('alert') && !descLower.includes('email')) {
    nonGoals.push('No notifications or alerts');
  }
  
  if (!descLower.includes('export') && !descLower.includes('download')) {
    nonGoals.push('No export or download functionality');
  }
  
  if (!descLower.includes('import') && !descLower.includes('upload')) {
    nonGoals.push('No import or upload functionality');
  }
  
  // Standard non-goals
  nonGoals.push('No breaking changes to existing APIs');
  nonGoals.push('No changes to unrelated features');
  
  return nonGoals.map(goal => `- ${goal}`).join('\n');
}

/**
 * Generate Design Considerations section
 */
function generateDesignConsiderations(featureDescription, answers) {
  const considerations = [];
  const analysis = analyzeFeature(featureDescription);
  const descLower = featureDescription.toLowerCase();
  
  if (analysis.needsUI) {
    considerations.push('Follow existing UI/UX patterns and design system');
    
    if (descLower.includes('form') || descLower.includes('input')) {
      considerations.push('Ensure forms are accessible with proper labels and error messages');
    }
    
    if (descLower.includes('list') || descLower.includes('table')) {
      considerations.push('Maintain consistent spacing and layout with existing lists/tables');
    }
    
    if (descLower.includes('button') || descLower.includes('action')) {
      considerations.push('Use consistent button styles and placement patterns');
    }
    
    considerations.push('Ensure responsive design works on mobile and desktop');
    considerations.push('Reuse existing components where possible');
  } else {
    considerations.push('No UI changes required for this feature');
  }
  
  if (considerations.length === 0) {
    considerations.push('Follow existing design patterns and conventions');
  }
  
  return considerations.map(consideration => `- ${consideration}`).join('\n');
}

/**
 * Generate Technical Considerations section
 */
function generateTechnicalConsiderations(featureDescription, answers) {
  const considerations = [];
  const analysis = analyzeFeature(featureDescription);
  const descLower = featureDescription.toLowerCase();
  
  // Database considerations
  if (analysis.needsDatabase) {
    considerations.push('Follow existing database schema patterns and naming conventions');
    considerations.push('Ensure migrations are reversible');
  }
  
  // Backend considerations
  if (analysis.needsBackend) {
    considerations.push('Follow existing API patterns and error handling conventions');
    considerations.push('Ensure proper type safety throughout');
    
    if (descLower.includes('api') || descLower.includes('endpoint')) {
      considerations.push('Maintain API versioning and backward compatibility');
    }
  }
  
  // UI considerations
  if (analysis.needsUI) {
    considerations.push('Follow existing component patterns and state management');
    considerations.push('Ensure proper TypeScript types for all components');
  }
  
  // Performance considerations
  if (descLower.includes('list') || descLower.includes('table') || descLower.includes('display')) {
    considerations.push('Consider pagination or virtualization for large datasets');
  }
  
  if (descLower.includes('search') || descLower.includes('filter')) {
    considerations.push('Implement efficient search/filter algorithms');
  }
  
  // Standard considerations
  considerations.push('Follow existing code patterns and conventions');
  considerations.push('Ensure type safety throughout');
  
  if (considerations.length === 0) {
    considerations.push('Follow existing technical patterns');
    considerations.push('Ensure code quality and type safety');
  }
  
  return considerations.map(consideration => `- ${consideration}`).join('\n');
}

/**
 * Generate Success Metrics section
 */
function generateSuccessMetrics(featureDescription, answers) {
  const metrics = [];
  const descLower = featureDescription.toLowerCase();
  const priority = answers.priority?.toLowerCase() || '';
  
  // Priority-based metrics
  if (priority.includes('high') || priority.includes('critical')) {
    metrics.push('Feature is stable and reliable for critical business use');
  } else if (priority.includes('medium')) {
    metrics.push('Feature works as expected and improves user workflow');
  } else {
    metrics.push('Feature provides value and enhances user experience');
  }
  
  // Feature-specific metrics
  if (descLower.includes('performance') || descLower.includes('speed')) {
    metrics.push('Performance meets or exceeds current benchmarks');
  }
  
  if (descLower.includes('user experience') || descLower.includes('ux')) {
    metrics.push('User feedback indicates improved experience');
  }
  
  if (descLower.includes('error') || descLower.includes('bug')) {
    metrics.push('Error rates are reduced');
  }
  
  // Standard metrics
  metrics.push('Feature works as described in the requirements');
  metrics.push('No regressions in existing functionality');
  metrics.push('Code passes all quality checks (typecheck, tests)');
  
  return metrics.map(metric => `- ${metric}`).join('\n');
}

/**
 * Generate Open Questions section
 */
function generateOpenQuestions(featureDescription, answers) {
  const questions = [];
  const descLower = featureDescription.toLowerCase();
  const analysis = analyzeFeature(featureDescription);
  
  // Database questions
  if (analysis.needsDatabase) {
    questions.push('Are there any data migration requirements for existing data?');
  }
  
  // UI questions
  if (analysis.needsUI) {
    questions.push('Are there any specific accessibility requirements?');
    questions.push('What are the mobile/tablet design considerations?');
  }
  
  // Performance questions
  if (descLower.includes('list') || descLower.includes('table') || descLower.includes('display')) {
    questions.push('What is the expected data volume and performance requirements?');
  }
  
  // Integration questions
  if (analysis.isFullStack) {
    questions.push('Are there any integration points with external services?');
  }
  
  // Standard questions
  questions.push('Are there any edge cases or error scenarios to consider?');
  questions.push('Are there any specific performance or scalability requirements?');
  
  return questions.map(q => `- ${q}`).join('\n');
}

/**
 * Build prompt for Cursor CLI agent to generate PRD
 */
function buildPRDPrompt(featureDescription, answers) {
  let prompt = `Create a PRD for: ${featureDescription}\n\n`;
  
  prompt += `The user has answered the following clarifying questions:\n`;
  if (answers.goal) {
    prompt += `- Goal: ${answers.goal}\n`;
  }
  if (answers.targetUser) {
    prompt += `- Target User: ${answers.targetUser}\n`;
  }
  if (answers.scope) {
    prompt += `- Scope: ${answers.scope}\n`;
  }
  if (answers.priority) {
    prompt += `- Priority: ${answers.priority}\n`;
  }
  
  prompt += `\n`;
  prompt += `Please generate a complete PRD following the structure in skills/prd/SKILL.md. Include all sections: Introduction, Goals, User Stories (multiple small stories broken down properly), Functional Requirements (numbered FR-1, FR-2, etc.), Non-Goals (Out of Scope), Design Considerations, Technical Considerations, Success Metrics, and Open Questions.\n\n`;
  prompt += `Output only the PRD markdown content. Do not save to file. Start with "# PRD: [feature name]" and include all sections.`;
  
  return prompt;
}

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
 * Extract PRD markdown from agent output
 * Agent may include other text, so we extract the PRD section
 */
function extractPRDFromOutput(output) {
  // Look for PRD starting with "# PRD:" or "# PRD:"
  const prdMatch = output.match(/#\s*PRD:[\s\S]*/);
  if (prdMatch) {
    return prdMatch[0].trim();
  }
  
  // If no match, try to find markdown starting with #
  const markdownMatch = output.match(/^#\s+.*[\s\S]*/m);
  if (markdownMatch) {
    return markdownMatch[0].trim();
  }
  
  // If still no match, return the whole output (may be just the PRD)
  return output.trim();
}

/**
 * Execute Cursor CLI agent command using spawn for proper argument handling
 * Avoids all shell escaping issues by passing arguments directly to the process
 * @param {string} prompt - The prompt to send to the agent
 * @param {string} outputFormat - The output format (text or json)
 * @param {number} timeout - Timeout in milliseconds
 * @returns {Promise<{stdout: string, stderr: string}>}
 */
export async function execAgentCommand(prompt, outputFormat = 'text', timeout = 120000) {
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
 * Generate PRD using Cursor CLI agent
 */
async function generatePRDWithAgent(featureDescription, answers, projectName, updateProgress = null) {
  const log = (status, message) => {
    if (updateProgress) updateProgress(status, message);
  };

  log('building', 'Building prompt for Cursor CLI agent...');
  const prompt = buildPRDPrompt(featureDescription, answers);
  
  try {
    log('executing', 'Executing Cursor CLI agent command...');
    // --print flag is required to enable shell execution (bash access)
    // --force flag forces allow commands unless explicitly denied
    // Use spawn to avoid shell escaping issues with long prompts
    log('waiting', 'Waiting for agent response (this may take 30-120 seconds)...');
    
    const { stdout, stderr } = await execAgentCommand(prompt, 'text', 120000);
    
    log('parsing', 'Parsing agent output...');
    // Extract PRD from output
    const prdContent = extractPRDFromOutput(stdout);
    
    if (!prdContent || prdContent.length < 100) {
      throw new Error('Agent output too short or invalid');
    }
    
    log('complete', 'PRD extracted from agent output');
    return prdContent;
  } catch (error) {
    // Log error but don't throw - we'll fallback to template
    log('error', `Agent generation failed: ${error.message}`);
    console.error('Agent generation failed:', error.message);
    if (error.stderr) {
      console.error('Agent stderr:', error.stderr);
    }
    throw error;
  }
}

/**
 * Generate a more detailed PRD by analyzing the feature description
 * Uses Cursor CLI agent if available, falls back to template generation
 */
export async function generateDetailedPRD(featureDescription, answers, projectName) {
  // Check if agent is available
  const agentAvailable = await isAgentAvailable();
  
  if (agentAvailable) {
    try {
      return await generatePRDWithAgent(featureDescription, answers, projectName);
    } catch (error) {
      // Fallback to template generation
      console.warn('Falling back to template generation:', error.message);
      return generatePRD(featureDescription, answers, projectName);
    }
  } else {
    // Agent not available, use template generation
    return generatePRD(featureDescription, answers, projectName);
  }
}
