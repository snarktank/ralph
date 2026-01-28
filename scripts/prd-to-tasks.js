#!/usr/bin/env node
/**
 * prd-to-tasks.js
 * CLI tool to convert prd.json to Claude Code tasks
 *
 * Usage: node scripts/prd-to-tasks.js [prd.json]
 */

import { convertPrdToTasks, checkTaskSystemAvailable } from '../lib/task-converter.js';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';
import fs from 'fs';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

async function main() {
  const prdPath = process.argv[2] || join(__dirname, '..', 'prd.json');

  console.log('Ralph PRD to Tasks Converter');
  console.log('============================\n');

  // Check if task system is available
  if (!checkTaskSystemAvailable()) {
    console.error('✗ Claude Code task system is not available');
    console.error('  Make sure Claude Code is installed and task system is enabled');
    console.error('  Set CLAUDE_CODE_ENABLE_TASKS=true if needed');
    process.exit(1);
  }

  // Check if PRD file exists
  if (!fs.existsSync(prdPath)) {
    console.error(`✗ PRD file not found: ${prdPath}`);
    console.error('  Usage: node scripts/prd-to-tasks.js [prd.json]');
    process.exit(1);
  }

  // Check if PRD is valid JSON
  let prd;
  try {
    prd = JSON.parse(fs.readFileSync(prdPath, 'utf8'));
  } catch (error) {
    console.error(`✗ Invalid JSON in PRD file: ${error.message}`);
    process.exit(1);
  }

  // Validate PRD structure
  if (!prd.userStories || !Array.isArray(prd.userStories)) {
    console.error('✗ PRD must have a userStories array');
    process.exit(1);
  }

  if (prd.userStories.length === 0) {
    console.error('✗ PRD has no user stories');
    process.exit(1);
  }

  console.log(`PRD: ${prd.project || 'Unknown'}`);
  console.log(`Branch: ${prd.branchName || 'Unknown'}`);
  console.log(`Stories: ${prd.userStories.length}`);
  console.log();

  try {
    const stats = await convertPrdToTasks(prdPath);

    console.log('\n✓ Task conversion successful!\n');
    console.log('Summary:');
    console.log(`  ${stats.parentTasks} user stories`);
    console.log(`  ${stats.childTasks} acceptance criteria`);
    console.log(`  ${stats.dependencies} dependencies`);
    console.log();
    console.log('Next steps:');
    console.log('  1. Review tasks: claude task list');
    console.log('  2. Run Ralph: ./ralph.sh --tool claude');
    console.log();

  } catch (error) {
    console.error('\n✗ Task conversion failed:', error.message);
    console.error();
    console.error('Troubleshooting:');
    console.error('  - Check that Claude Code is installed');
    console.error('  - Verify task system is enabled (CLAUDE_CODE_ENABLE_TASKS=true)');
    console.error('  - Try running: claude task list');
    process.exit(1);
  }
}

main().catch(error => {
  console.error('Unexpected error:', error);
  process.exit(1);
});
