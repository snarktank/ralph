#!/usr/bin/env node
/**
 * Ralph CLI - Autonomous AI agent loop installer and runner
 * 
 * Commands:
 *   ralph init [--worker amp|cursor|both] [--force] [--cursor-rules] [--cursor-cli]
 *   ralph run [--worker amp|cursor] [--iterations N]
 * 
 * Init options:
 *   --worker: Install files for 'amp', 'cursor', or 'both' (default: 'both')
 *   --force: Overwrite existing files
 *   --cursor-rules: Also install .cursor/rules/ralph-prd.mdc
 *   --cursor-cli: Also install .cursor/cli.json template
 */

import { fileURLToPath } from 'url';
import { dirname, join } from 'path';
import { readFileSync, writeFileSync, mkdirSync, existsSync, statSync, copyFileSync } from 'fs';
import { chmodSync } from 'fs';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const PACKAGE_ROOT = join(__dirname, '..');
const TEMPLATES_DIR = join(PACKAGE_ROOT, 'templates');

// Get command and args
const [,, command, ...args] = process.argv;

if (!command) {
  console.error('Usage: ralph <init|run> [options]');
  process.exit(1);
}

if (command === 'init') {
  handleInit(args);
} else if (command === 'run') {
  handleRun(args);
} else {
  console.error(`Unknown command: ${command}`);
  console.error('Usage: ralph <init|run> [options]');
  process.exit(1);
}

function handleInit(args) {
  const flags = parseFlags(args);
  const force = flags.has('--force');
  const cursorRules = flags.has('--cursor-rules');
  const cursorCli = flags.has('--cursor-cli');
  const worker = flags.get('--worker') || 'both'; // 'amp', 'cursor', or 'both' (default)
  
  if (worker !== 'amp' && worker !== 'cursor' && worker !== 'both') {
    console.error(`Error: --worker must be 'amp', 'cursor', or 'both' (got: ${worker})`);
    process.exit(1);
  }
  
  const repoRoot = process.cwd();
  const targetDir = join(repoRoot, 'scripts', 'ralph');
  
  // Create scripts/ralph directory
  if (!existsSync(targetDir)) {
    mkdirSync(targetDir, { recursive: true });
    console.log(`Created: ${targetDir}`);
  }
  
  // Always required files (needed for both workers)
  const alwaysRequired = [
    { src: 'scripts/ralph/ralph.sh', dest: 'scripts/ralph/ralph.sh', executable: true },
    { src: 'scripts/ralph/prd.json.example', dest: 'scripts/ralph/prd.json.example', executable: false },
  ];
  
  // Amp-specific files
  const ampFiles = [
    { src: 'scripts/ralph/prompt.md', dest: 'scripts/ralph/prompt.md', executable: false },
  ];
  
  // Cursor-specific files
  const cursorFiles = [
    { src: 'scripts/ralph/cursor/prompt.cursor.md', dest: 'scripts/ralph/cursor/prompt.cursor.md', executable: false },
    { src: 'scripts/ralph/cursor/prompt.convert-to-prd-json.md', dest: 'scripts/ralph/cursor/prompt.convert-to-prd-json.md', executable: false },
    { src: 'scripts/ralph/cursor/prompt.generate-prd.md', dest: 'scripts/ralph/cursor/prompt.generate-prd.md', executable: false },
    { src: 'scripts/ralph/cursor/convert-to-prd-json.sh', dest: 'scripts/ralph/cursor/convert-to-prd-json.sh', executable: true },
  ];
  
  // Build file list based on worker selection
  let requiredFiles = [...alwaysRequired];
  if (worker === 'amp' || worker === 'both') {
    requiredFiles = [...requiredFiles, ...ampFiles];
  }
  if (worker === 'cursor' || worker === 'both') {
    requiredFiles = [...requiredFiles, ...cursorFiles];
  }
  
  const created = [];
  const skipped = [];
  
  for (const file of requiredFiles) {
    const srcPath = join(TEMPLATES_DIR, file.src);
    const destPath = join(repoRoot, file.dest);
    const destDir = dirname(destPath);
    
    // Create subdirectory if needed
    if (!existsSync(destDir)) {
      mkdirSync(destDir, { recursive: true });
    }
    
    if (existsSync(destPath) && !force) {
      skipped.push(file.dest);
      continue;
    }
    
    copyFileSync(srcPath, destPath);
    if (file.executable) {
      chmodSync(destPath, 0o755);
    }
    created.push(file.dest);
  }
  
  // Optional: .cursor/rules/ralph-prd.mdc (only relevant for cursor)
  if (cursorRules && (worker === 'cursor' || worker === 'both')) {
    const cursorRulesDir = join(repoRoot, '.cursor', 'rules');
    const cursorRulesFile = join(cursorRulesDir, 'ralph-prd.mdc');
    
    if (!existsSync(cursorRulesDir)) {
      mkdirSync(cursorRulesDir, { recursive: true });
    }
    
    if (existsSync(cursorRulesFile) && !force) {
      skipped.push('.cursor/rules/ralph-prd.mdc');
    } else {
      const srcRules = join(PACKAGE_ROOT, '.cursor', 'rules', 'ralph-prd.mdc');
      if (existsSync(srcRules)) {
        copyFileSync(srcRules, cursorRulesFile);
        created.push('.cursor/rules/ralph-prd.mdc');
      }
    }
  }
  
  // Optional: .cursor/cli.json (only relevant for cursor)
  if (cursorCli && (worker === 'cursor' || worker === 'both')) {
    const cursorCliFile = join(repoRoot, '.cursor', 'cli.json');
    const cursorDir = dirname(cursorCliFile);
    
    if (!existsSync(cursorDir)) {
      mkdirSync(cursorDir, { recursive: true });
    }
    
    if (existsSync(cursorCliFile) && !force) {
      skipped.push('.cursor/cli.json');
    } else {
      // Create a basic template
      const cliTemplate = {
        "mcpServers": {
          "cursor-ide-browser": {
            "command": "node",
            "args": []
          }
        }
      };
      writeFileSync(cursorCliFile, JSON.stringify(cliTemplate, null, 2) + '\n');
      created.push('.cursor/cli.json');
    }
  }
  
  // Print summary
  console.log('\nSummary:');
  if (worker !== 'both') {
    console.log(`Worker setup: ${worker}`);
  }
  if (created.length > 0) {
    console.log(`\nCreated ${created.length} file(s):`);
    created.forEach(f => console.log(`  - ${f}`));
  }
  if (skipped.length > 0) {
    console.log(`\nSkipped ${skipped.length} file(s) (already exist, use --force to overwrite):`);
    skipped.forEach(f => console.log(`  - ${f}`));
  }
  console.log('\nRalph initialized! Run `ralph run` to start.');
}

async function handleRun(args) {
  const flags = parseFlags(args);
  const worker = flags.get('--worker') || 'amp';
  const iterations = flags.get('--iterations') || '10';
  
  if (worker !== 'amp' && worker !== 'cursor') {
    console.error(`Error: Worker must be 'amp' or 'cursor' (got: ${worker})`);
    process.exit(1);
  }
  
  const repoRoot = process.cwd();
  const runnerScript = join(repoRoot, 'scripts', 'ralph', 'ralph.sh');
  
  if (!existsSync(runnerScript)) {
    console.error('Error: Ralph not initialized in this repository.');
    console.error('Run `ralph init` first to set up Ralph.');
    process.exit(1);
  }
  
  // Execute the runner script with appropriate arguments
  const { spawn } = await import('child_process');
  
  const child = spawn('bash', [runnerScript, iterations, '--worker', worker], {
    stdio: 'inherit',
    cwd: repoRoot,
  });
  
  child.on('exit', (code) => {
    process.exit(code || 0);
  });
}

function parseFlags(args) {
  const flags = new Map();
  for (let i = 0; i < args.length; i++) {
    const arg = args[i];
    if (arg.startsWith('--')) {
      if (i + 1 < args.length && !args[i + 1].startsWith('--')) {
        flags.set(arg, args[i + 1]);
        i++;
      } else {
        flags.set(arg, true);
      }
    }
  }
  return flags;
}
