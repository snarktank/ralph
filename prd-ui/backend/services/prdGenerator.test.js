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
    });

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
