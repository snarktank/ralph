/**
 * PRD Generator Service
 * Implements logic from skills/prd/SKILL.md
 */

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
 */
export function generatePRD(featureDescription, answers, projectName = 'Project') {
  const featureName = extractFeatureName(featureDescription);
  const title = `PRD: ${featureName}`;

  let prd = `# ${title}\n\n`;

  // Introduction/Overview
  prd += `## Introduction\n\n`;
  prd += `${featureDescription}\n\n`;
  if (answers.goal) {
    prd += `**Primary Goal:** ${answers.goal}\n\n`;
  }

  // Goals
  prd += `## Goals\n\n`;
  prd += generateGoals(featureDescription, answers);
  prd += `\n`;

  // User Stories
  prd += `## User Stories\n\n`;
  prd += generateUserStories(featureDescription, answers);
  prd += `\n`;

  // Functional Requirements
  prd += `## Functional Requirements\n\n`;
  prd += generateFunctionalRequirements(featureDescription, answers);
  prd += `\n`;

  // Non-Goals
  prd += `## Non-Goals (Out of Scope)\n\n`;
  prd += generateNonGoals(answers);
  prd += `\n`;

  // Technical Considerations
  prd += `## Technical Considerations\n\n`;
  prd += generateTechnicalConsiderations(answers);
  prd += `\n`;

  // Success Metrics
  prd += `## Success Metrics\n\n`;
  prd += generateSuccessMetrics(answers);
  prd += `\n`;

  // Open Questions
  prd += `## Open Questions\n\n`;
  prd += generateOpenQuestions(answers);
  prd += `\n`;

  return prd;
}

function extractFeatureName(description) {
  // Try to extract a feature name from the description
  const firstLine = description.split('\n')[0];
  const words = firstLine.split(' ').slice(0, 5);
  return words.join(' ');
}

function generateGoals(description, answers) {
  const goals = [
    `Implement ${description.split(' ').slice(0, 10).join(' ')}`,
    'Ensure type safety and code quality',
    'Maintain existing functionality'
  ];

  if (answers.scope) {
    goals.push(`Deliver ${answers.scope.toLowerCase()} version`);
  }

  return goals.map(goal => `- ${goal}`).join('\n');
}

function generateUserStories(description, answers) {
  // Generate basic user stories based on feature description
  // This is a simplified version - in a real implementation, this would be more sophisticated
  const stories = [
    {
      id: 'US-001',
      title: 'Initial implementation',
      description: `As a developer, I want to implement ${description.split(' ').slice(0, 5).join(' ')} so that users can benefit from this feature.`,
      criteria: [
        'Implementation follows project conventions',
        'Typecheck passes',
        'Tests pass (if applicable)'
      ],
      hasUI: answers.scope?.includes('UI') || !answers.scope?.includes('backend')
    }
  ];

  let prdStories = '';
  stories.forEach((story, index) => {
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

function generateFunctionalRequirements(description, answers) {
  const requirements = [
    `FR-1: Implement core functionality for ${description.split(' ').slice(0, 5).join(' ')}`,
    'FR-2: Ensure proper error handling and validation',
    'FR-3: Maintain backward compatibility'
  ];

  return requirements.join('\n');
}

function generateNonGoals(answers) {
  const nonGoals = [
    'No breaking changes to existing APIs',
    'No changes to unrelated features'
  ];

  if (answers.scope?.includes('backend')) {
    nonGoals.push('No UI changes');
  } else if (answers.scope?.includes('UI')) {
    nonGoals.push('No backend changes');
  }

  return nonGoals.map(goal => `- ${goal}`).join('\n');
}

function generateTechnicalConsiderations(answers) {
  const considerations = [
    'Follow existing code patterns',
    'Use established libraries and frameworks',
    'Ensure type safety throughout'
  ];

  return considerations.map(consideration => `- ${consideration}`).join('\n');
}

function generateSuccessMetrics(answers) {
  const metrics = [
    'Feature works as described',
    'No regressions in existing functionality',
    'Code passes all quality checks'
  ];

  return metrics.map(metric => `- ${metric}`).join('\n');
}

function generateOpenQuestions(answers) {
  const questions = [
    'Are there any edge cases to consider?',
    'Are there any performance requirements?'
  ];

  return questions.map(q => `- ${q}`).join('\n');
}

/**
 * Generate a more detailed PRD by analyzing the feature description
 * This would ideally use AI/LLM in the future, but for now provides a template
 */
export function generateDetailedPRD(featureDescription, answers, projectName) {
  // For now, use the basic generator
  // In the future, this could call Cursor CLI or another AI service
  return generatePRD(featureDescription, answers, projectName);
}
