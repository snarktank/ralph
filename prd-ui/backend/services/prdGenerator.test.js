import { describe, it, expect, beforeAll } from '@jest/globals';
import { generateQuestions, generatePRD, execAgentCommand } from './prdGenerator.js';
import { spawn } from 'child_process';

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
    it('generates PRD markdown from feature description', async () => {
      const description = 'Add user authentication';
      const answers = {
        goal: 'Improve security',
        targetUser: 'All users',
        scope: 'Full-featured implementation',
      };

      const prd = await generatePRD(description, answers, 'TestProject');
      const prdLower = prd.toLowerCase();

      expect(prd).toContain('# PRD:');
      expect(prdLower).toContain('## introduction');
      expect(prdLower).toContain('## goals');
      expect(prdLower).toContain('## user stories');
      // Agent may use different casing for the description
      expect(prdLower).toContain(description.toLowerCase());
    }, 150000); // Increase timeout to 150 seconds for agent execution

    it('includes all required sections', async () => {
      const prd = await generatePRD('Test feature', {}, 'Test');
      const prdLower = prd.toLowerCase();

      // Core sections that should always be present (case-insensitive)
      const requiredSections = [
        'introduction',
        'goals',
        'user stories',
        'functional requirements',
        'non-goals',
        'technical considerations',
        'success metrics',
        'open questions',
      ];

      requiredSections.forEach((section) => {
        expect(prdLower).toContain(`## ${section}`);
      });
    }, 150000); // Increase timeout to 150 seconds for agent execution

    it('includes user stories with acceptance criteria', async () => {
      const prd = await generatePRD('Test feature', {}, 'Test');
      const prdLower = prd.toLowerCase();

      // PRD should contain user stories section
      expect(prdLower).toContain('## user stories');
      
      // Stories can be in different formats:
      // - "### US-001:" (template format)
      // - "- **US-1:" (agent format with US-n)
      // - "- **Story 1:" (agent format)
      // - "**Story 1:" (agent format variant)
      const hasTemplateFormat = prd.includes('### US-');
      const hasAgentUSFormat = /- \*\*US-\d+:/.test(prd);
      const hasAgentStoryFormat = /\*\*Story \d+:/.test(prd) || /- \*\*Story \d+:/.test(prd);
      expect(hasTemplateFormat || hasAgentUSFormat || hasAgentStoryFormat).toBe(true);
      
      // Stories should have acceptance criteria in some form (case-insensitive)
      const hasAcceptanceCriteria = prdLower.includes('acceptance criteria');
      expect(hasAcceptanceCriteria).toBe(true);
    }, 150000); // Increase timeout to 150 seconds for agent execution
  });

  describe('execAgentCommand', () => {
    let agentAvailable = false;
    let agentWorking = false;

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
      
      if (agentAvailable) {
        // Test if agent command actually works (not just available)
        try {
          const testResult = await execAgentCommand('Say hello', 'text', 10000);
          if (testResult.stdout && testResult.stdout.length > 0) {
            agentWorking = true;
          }
        } catch (error) {
          // Agent available but not working in this environment
          agentWorking = false;
        }
      }
    }, 20000); // Increase beforeAll timeout to 20 seconds

    it('executes agent command with simple prompt', async () => {
      if (!agentAvailable || !agentWorking) {
        console.log('Skipping test: agent command not available or not working in test environment');
        return;
      }

      const prompt = 'Say hello';
      const result = await execAgentCommand(prompt, 'text', 30000);
      
      expect(result).toHaveProperty('stdout');
      expect(result).toHaveProperty('stderr');
      expect(typeof result.stdout).toBe('string');
      expect(result.stdout.length).toBeGreaterThan(0);
    }, 35000); // 35 second timeout for test

    it('handles prompts with special characters', async () => {
      if (!agentAvailable || !agentWorking) {
        console.log('Skipping test: agent command not available or not working in test environment');
        return;
      }

      const prompt = "Test with 'quotes' and /slashes and $dollars";
      const result = await execAgentCommand(prompt, 'text', 30000);
      
      expect(result).toHaveProperty('stdout');
      expect(result.stdout.length).toBeGreaterThan(0);
    }, 35000);

    it('handles long prompts', async () => {
      if (!agentAvailable || !agentWorking) {
        console.log('Skipping test: agent command not available or not working in test environment');
        return;
      }

      const longPrompt = 'Create a PRD for: ' + 'A'.repeat(500);
      const result = await execAgentCommand(longPrompt, 'text', 30000);
      
      expect(result).toHaveProperty('stdout');
      expect(result.stdout.length).toBeGreaterThan(0);
    }, 35000);

    it('respects timeout', async () => {
      if (!agentAvailable) {
        console.log('Skipping test: agent command not available');
        return;
      }

      // Use a very short timeout to test timeout handling
      const prompt = 'Say hello';
      
      await expect(
        execAgentCommand(prompt, 'text', 1000) // 1 second timeout
      ).rejects.toThrow();
    }, 5000);

    it('works with json output format', async () => {
      if (!agentAvailable || !agentWorking) {
        console.log('Skipping test: agent command not available or not working in test environment');
        return;
      }

      const prompt = 'Convert this to JSON: {"test": "value"}';
      const result = await execAgentCommand(prompt, 'json', 30000);
      
      expect(result).toHaveProperty('stdout');
      expect(result.stdout.length).toBeGreaterThan(0);
    }, 35000);
  });
});
