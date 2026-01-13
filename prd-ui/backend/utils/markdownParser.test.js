import { describe, it, expect } from '@jest/globals';
import { parsePRD, extractFeatureName } from './markdownParser.js';

describe('markdownParser', () => {
  describe('parsePRD', () => {
    it('parses PRD title', () => {
      const markdown = '# PRD: Test Feature\n\nContent';
      const result = parsePRD(markdown);
      expect(result.title).toBe('Test Feature');
    });

    it('parses user stories', () => {
      const markdown = `# PRD: Test

## User Stories

### US-001: Test Story
**Description:** Test description

**Acceptance Criteria:**
- [ ] Criterion 1
- [ ] Criterion 2
`;

      const result = parsePRD(markdown);
      expect(result.userStories).toHaveLength(1);
      expect(result.userStories[0].id).toBe('US-001');
      expect(result.userStories[0].title).toBe('Test Story');
      expect(result.userStories[0].acceptanceCriteria).toHaveLength(2);
    });

    it('parses goals section', () => {
      const markdown = `# PRD: Test

## Goals

- Goal 1
- Goal 2
`;

      const result = parsePRD(markdown);
      expect(result.goals).toContain('Goal 1');
      expect(result.goals).toContain('Goal 2');
    });
  });

  describe('extractFeatureName', () => {
    it('extracts feature name from title', () => {
      const result = extractFeatureName('PRD: Test Feature Name');
      expect(result).toBe('test-feature-name');
    });

    it('handles special characters', () => {
      const result = extractFeatureName('PRD: Test & Feature!');
      expect(result).toBe('test-feature');
    });
  });
});
