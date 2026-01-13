import { describe, it, expect } from '@jest/globals';
import { generateQuestions, generatePRD } from './prdGenerator.js';

describe('prdGenerator', () => {
  describe('generateQuestions', () => {
    it('generates clarifying questions', () => {
      const questions = generateQuestions('test feature');

      expect(questions).toBeInstanceOf(Array);
      expect(questions.length).toBeGreaterThan(0);
      expect(questions[0]).toHaveProperty('id');
      expect(questions[0]).toHaveProperty('text');
      expect(questions[0]).toHaveProperty('options');
      expect(questions[0].options.length).toBeGreaterThan(0);
    });
  });

  describe('generatePRD', () => {
    it('generates PRD markdown from feature description', () => {
      const description = 'Add user authentication';
      const answers = {
        goal: 'Improve security',
        targetUser: 'All users',
        scope: 'Full-featured implementation',
      };

      const prd = generatePRD(description, answers, 'TestProject');

      expect(prd).toContain('# PRD:');
      expect(prd).toContain('## Introduction');
      expect(prd).toContain('## Goals');
      expect(prd).toContain('## User Stories');
      expect(prd).toContain(description);
    });

    it('includes all required sections', () => {
      const prd = generatePRD('Test feature', {}, 'Test');

      const sections = [
        'Introduction',
        'Goals',
        'User Stories',
        'Functional Requirements',
        'Non-Goals',
        'Technical Considerations',
        'Success Metrics',
        'Open Questions',
      ];

      sections.forEach((section) => {
        expect(prd).toContain(`## ${section}`);
      });
    });

    it('includes user stories with acceptance criteria', () => {
      const prd = generatePRD('Test feature', {}, 'Test');

      expect(prd).toContain('### US-');
      expect(prd).toContain('**Description:**');
      expect(prd).toContain('**Acceptance Criteria:**');
      expect(prd).toContain('- [ ]');
    });
  });
});
