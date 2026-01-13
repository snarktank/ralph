import { describe, it, expect } from '@jest/globals';
import { convertPRDToJSON, validateJSON } from './jsonConverter.js';

describe('jsonConverter', () => {
  describe('convertPRDToJSON', () => {
    it('converts PRD markdown to JSON format', () => {
      const markdown = `# PRD: Test Feature

## Introduction

Test feature description

## User Stories

### US-001: Add test field
**Description:** As a developer, I want to add a test field.

**Acceptance Criteria:**
- [ ] Add test field to database
- [ ] Typecheck passes
`;

      const result = convertPRDToJSON(markdown, 'TestProject');

      expect(result.project).toBe('TestProject');
      expect(result.branchName).toMatch(/^ralph\//);
      expect(result.userStories).toHaveLength(1);
      expect(result.userStories[0].id).toBe('US-001');
      expect(result.userStories[0].passes).toBe(false);
    });

    it('adds Typecheck passes to acceptance criteria if missing', () => {
      const markdown = `# PRD: Test

## User Stories

### US-001: Test
**Description:** Test description

**Acceptance Criteria:**
- [ ] Some criterion
`;

      const result = convertPRDToJSON(markdown, 'Test');

      const criteria = result.userStories[0].acceptanceCriteria;
      expect(criteria).toContain('Typecheck passes');
    });

    it('orders stories by dependencies', () => {
      const markdown = `# PRD: Test

## User Stories

### US-001: UI Component
**Description:** Add UI component

### US-002: Database Schema
**Description:** Add database schema
`;

      const result = convertPRDToJSON(markdown, 'Test');

      // Database stories should come before UI stories
      const dbStory = result.userStories.find(s => s.title.includes('Database'));
      const uiStory = result.userStories.find(s => s.title.includes('UI'));
      
      if (dbStory && uiStory) {
        expect(dbStory.priority).toBeLessThan(uiStory.priority);
      }
    });
  });

  describe('validateJSON', () => {
    it('validates correct JSON structure', () => {
      const json = {
        project: 'Test',
        branchName: 'ralph/test',
        description: 'Test',
        userStories: [
          {
            id: 'US-001',
            title: 'Test',
            description: 'Test',
            acceptanceCriteria: ['Test'],
            priority: 1,
            passes: false,
            notes: '',
          },
        ],
      };

      const result = validateJSON(json);
      expect(result.valid).toBe(true);
      expect(result.errors).toHaveLength(0);
    });

    it('returns errors for invalid JSON', () => {
      const json = {
        project: 'Test',
        // Missing branchName
        userStories: [],
      };

      const result = validateJSON(json);
      expect(result.valid).toBe(false);
      expect(result.errors.length).toBeGreaterThan(0);
    });

    it('validates user stories structure', () => {
      const json = {
        project: 'Test',
        branchName: 'ralph/test',
        description: 'Test',
        userStories: [
          {
            // Missing id
            title: 'Test',
          },
        ],
      };

      const result = validateJSON(json);
      expect(result.valid).toBe(false);
      expect(result.errors.some(e => e.includes('ID'))).toBe(true);
    });
  });
});
