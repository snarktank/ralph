/**
 * Markdown Parser for PRD files
 * Parses PRD markdown structure to extract structured data
 */

/**
 * Parse PRD markdown and extract metadata
 */
export function parsePRD(markdown) {
  const lines = markdown.split('\n');
  const result = {
    title: '',
    introduction: '',
    goals: [],
    userStories: [],
    functionalRequirements: [],
    nonGoals: [],
    technicalConsiderations: [],
    successMetrics: [],
    openQuestions: []
  };

  let currentSection = null;
  let currentStory = null;
  let buffer = [];

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    const trimmed = line.trim();

    // Parse title
    if (trimmed.startsWith('# PRD:') || trimmed.startsWith('# ')) {
      result.title = trimmed.replace(/^#+\s*PRD:\s*/, '').replace(/^#+\s*/, '');
      continue;
    }

    // Parse sections
    if (trimmed.startsWith('## ')) {
      // Save previous section
      if (currentSection && buffer.length > 0) {
        result[currentSection] = buffer.join('\n').trim();
        buffer = [];
      }

      const sectionName = trimmed.replace(/^##\s+/, '').toLowerCase();
      currentSection = mapSectionName(sectionName);
      continue;
    }

    // Parse user stories
    if (trimmed.startsWith('### US-')) {
      // Save previous story
      if (currentStory) {
        result.userStories.push(currentStory);
      }

      const match = trimmed.match(/^###\s*(US-\d+):\s*(.+)$/);
      if (match) {
        currentStory = {
          id: match[1],
          title: match[2],
          description: '',
          acceptanceCriteria: []
        };
        buffer = [];
      }
      continue;
    }

    // Parse story description
    if (currentStory && trimmed.startsWith('**Description:**')) {
      currentStory.description = trimmed.replace(/^\*\*Description:\*\*\s*/, '');
      continue;
    }

    // Parse acceptance criteria
    if (currentStory && trimmed.startsWith('**Acceptance Criteria:**')) {
      continue; // Next lines will be criteria
    }

    if (currentStory && trimmed.startsWith('- [ ]')) {
      const criterion = trimmed.replace(/^-\s*\[\s*\]\s*/, '').trim();
      if (criterion) {
        currentStory.acceptanceCriteria.push(criterion);
      }
      continue;
    }

    // Parse lists
    if (trimmed.startsWith('- ') && !trimmed.startsWith('- [ ]')) {
      const item = trimmed.replace(/^-\s*/, '').trim();
      if (item && currentSection) {
        if (Array.isArray(result[currentSection])) {
          result[currentSection].push(item);
        } else {
          buffer.push(item);
        }
      }
      continue;
    }

    // Parse functional requirements (FR-X format)
    if (trimmed.match(/^FR-\d+:/)) {
      result.functionalRequirements.push(trimmed);
      continue;
    }

    // Buffer other content
    if (trimmed && currentSection) {
      buffer.push(trimmed);
    }
  }

  // Save last story
  if (currentStory) {
    result.userStories.push(currentStory);
  }

  // Save last section
  if (currentSection && buffer.length > 0) {
    if (Array.isArray(result[currentSection])) {
      result[currentSection] = [...result[currentSection], ...buffer];
    } else {
      result[currentSection] = buffer.join('\n').trim();
    }
  }

  return result;
}

function mapSectionName(sectionName) {
  const mapping = {
    'introduction': 'introduction',
    'overview': 'introduction',
    'goals': 'goals',
    'user stories': 'userStories',
    'functional requirements': 'functionalRequirements',
    'non-goals': 'nonGoals',
    'non-goals (out of scope)': 'nonGoals',
    'out of scope': 'nonGoals',
    'technical considerations': 'technicalConsiderations',
    'success metrics': 'successMetrics',
    'open questions': 'openQuestions'
  };

  return mapping[sectionName.toLowerCase()] || null;
}

/**
 * Extract feature name from PRD title
 */
export function extractFeatureName(title) {
  return title
    .replace(/^PRD:\s*/i, '')
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-|-$/g, '');
}
