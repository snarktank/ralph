import { describe, it, expect, beforeAll } from '@jest/globals';
import { convertPRDToJSON, validateJSON, execAgentCommand, extractJSONFromOutput } from './jsonConverter.js';
import { spawn } from 'child_process';

describe('jsonConverter', () => {
  describe('extractJSONFromOutput', () => {
    it('extracts JSON from agent metadata wrapper', () => {
      const wrappedOutput = JSON.stringify({
        type: 'result',
        subtype: 'success',
        result: JSON.stringify({
          project: 'TestProject',
          branchName: 'ralph/test',
          description: 'Test',
          userStories: []
        })
      });

      const result = extractJSONFromOutput(wrappedOutput);
      
      expect(result.project).toBe('TestProject');
      expect(result.branchName).toBe('ralph/test');
      expect(result.userStories).toEqual([]);
    });

    it('extracts JSON from markdown code fence in result field', () => {
      const wrappedOutput = JSON.stringify({
        type: 'result',
        result: '```json\n{\n  "project": "TestProject",\n  "branchName": "ralph/test",\n  "userStories": []\n}\n```'
      });

      const result = extractJSONFromOutput(wrappedOutput);
      
      expect(result.project).toBe('TestProject');
      expect(result.branchName).toBe('ralph/test');
    });

    it('extracts JSON from mixed text with markdown code fence', () => {
      const output = 'Here is the JSON:\n```json\n{\n  "project": "Test",\n  "branchName": "ralph/test",\n  "userStories": []\n}\n```\nDone!';

      const result = extractJSONFromOutput(output);
      
      expect(result.project).toBe('Test');
    });

    it('extracts JSON from plain code fence', () => {
      const output = '```\n{\n  "project": "Test",\n  "branchName": "ralph/test",\n  "userStories": []\n}\n```';

      const result = extractJSONFromOutput(output);
      
      expect(result.project).toBe('Test');
    });

    it('handles already parsed PRD JSON directly', () => {
      const directJson = JSON.stringify({
        project: 'Test',
        branchName: 'ralph/test',
        userStories: []
      });

      const result = extractJSONFromOutput(directJson);
      
      expect(result.project).toBe('Test');
    });

    it('finds JSON object in mixed text', () => {
      const output = 'Some text before {"project": "Test", "branchName": "ralph/test", "userStories": []} some text after';

      const result = extractJSONFromOutput(output);
      
      expect(result.project).toBe('Test');
    });

    it('handles agent wrapper with markdown in result', () => {
      const realExample = {
        type: 'result',
        subtype: 'success',
        is_error: false,
        result: 'Converting to JSON:\n```json\n{\n  "project": "TestProject",\n  "branchName": "ralph/feature",\n  "description": "Add feature",\n  "userStories": [\n    {\n      "id": "US-001",\n      "title": "Test",\n      "description": "Test",\n      "acceptanceCriteria": ["Test"],\n      "priority": 1,\n      "passes": false,\n      "notes": ""\n    }\n  ]\n}\n```'
      };

      const result = extractJSONFromOutput(JSON.stringify(realExample));
      
      expect(result.project).toBe('TestProject');
      expect(result.branchName).toBe('ralph/feature');
      expect(result.userStories).toHaveLength(1);
      expect(result.userStories[0].id).toBe('US-001');
    });

    it('throws error for invalid input', () => {
      expect(() => extractJSONFromOutput('')).toThrow();
      expect(() => extractJSONFromOutput(null)).toThrow();
      expect(() => extractJSONFromOutput('not json at all')).toThrow();
    });
  });

  describe('convertPRDToJSON', () => {
    it('converts PRD markdown to JSON format', async () => {
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

      const result = await convertPRDToJSON(markdown, 'TestProject');

      expect(result.project).toBe('TestProject');
      expect(result.branchName).toMatch(/^ralph\//);
      expect(result.userStories).toHaveLength(1);
      expect(result.userStories[0].id).toBe('US-001');
      expect(result.userStories[0].passes).toBe(false);
    }, 150000); // Increase timeout to 150 seconds (agent can take 30-120s)

    it('adds Typecheck passes to acceptance criteria if missing', async () => {
      const markdown = `# PRD: Test

## User Stories

### US-001: Test
**Description:** Test description

**Acceptance Criteria:**
- [ ] Some criterion
`;

      const result = await convertPRDToJSON(markdown, 'Test');

      const criteria = result.userStories[0].acceptanceCriteria;
      expect(criteria).toContain('Typecheck passes');
    }, 150000); // Increase timeout to 150 seconds (agent can take 30-120s)

    it('orders stories by dependencies', async () => {
      const markdown = `# PRD: Test

## User Stories

### US-001: UI Component
**Description:** Add UI component

### US-002: Database Schema
**Description:** Add database schema
`;

      const result = await convertPRDToJSON(markdown, 'Test');

      // Database stories should come before UI stories
      const dbStory = result.userStories.find(s => s.title.includes('Database'));
      const uiStory = result.userStories.find(s => s.title.includes('UI'));
      
      if (dbStory && uiStory) {
        expect(dbStory.priority).toBeLessThan(uiStory.priority);
      }
    }, 150000); // Increase timeout to 150 seconds (agent can take 30-120s)
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

  describe('execAgentCommand', () => {
    let agentAvailable = false;

    beforeAll(async () => {
      // Check if agent is available
      agentAvailable = await new Promise((resolve) => {
        const child = spawn('agent', ['--version'], { shell: false, stdio: 'ignore' });
        const timeoutId = setTimeout(() => {
          child.kill();
          resolve(false);
        }, 2000);
        child.on('close', (code) => {
          clearTimeout(timeoutId);
          resolve(code === 0);
        });
        child.on('error', () => {
          clearTimeout(timeoutId);
          resolve(false);
        });
      });
    }, 10000); // Increase beforeAll timeout to 10 seconds

    it('executes agent command with simple prompt', async () => {
      if (!agentAvailable) {
        console.log('Skipping test: agent command not available');
        return;
      }

      const prompt = 'Convert this to JSON: {"test": "value"}';
      const result = await execAgentCommand(prompt, 'json', 120000);
      
      expect(result).toHaveProperty('stdout');
      expect(result).toHaveProperty('stderr');
      expect(typeof result.stdout).toBe('string');
      expect(result.stdout.length).toBeGreaterThan(0);
    }, 150000);

    it('handles prompts with special characters', async () => {
      if (!agentAvailable) {
        console.log('Skipping test: agent command not available');
        return;
      }

      const prompt = "Convert PRD with 'quotes' and /slashes";
      const result = await execAgentCommand(prompt, 'json', 140000); // Increase internal timeout
      
      expect(result).toHaveProperty('stdout');
      expect(result.stdout.length).toBeGreaterThan(0);
    }, 150000);

    it('handles long PRD markdown content', async () => {
      if (!agentAvailable) {
        console.log('Skipping test: agent command not available');
        return;
      }

      const longPRD = '# PRD: Test\n\n' + 'A'.repeat(1000);
      const result = await execAgentCommand(longPRD, 'json', 140000); // Increase internal timeout
      
      expect(result).toHaveProperty('stdout');
      expect(result.stdout.length).toBeGreaterThan(0);
    }, 150000);

    it('respects timeout', async () => {
      if (!agentAvailable) {
        console.log('Skipping test: agent command not available');
        return;
      }

      const prompt = 'Convert this to JSON';
      
      await expect(
        execAgentCommand(prompt, 'json', 1000) // 1 second timeout
      ).rejects.toThrow();
    }, 5000);
  });
});
