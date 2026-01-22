#!/usr/bin/env node
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { spawn } from "node:child_process";
import { PassThrough } from "node:stream";

const scriptDir = path.dirname(fileURLToPath(import.meta.url));

export function formatDate(date) {
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, "0");
  const day = String(date.getDate()).padStart(2, "0");
  return `${year}-${month}-${day}`;
}

export function stripPrefix(branchName) {
  return branchName.replace(/^ralph\//, "");
}

export function resolveEntrypointPath(entrypointPath) {
  const resolved = path.resolve(entrypointPath);
  const normalized = resolved.replace(/[\\/]node_modules[\\/]ralph[\\/]ralph\.js$/, `${path.sep}ralph.js`);

  if (normalized !== resolved && fs.existsSync(normalized)) {
    return normalized;
  }

  return resolved;
}

export function pathsAreEqual(pathA, pathB) {
  if (!pathA || !pathB) return false;
  const a = path.resolve(pathA);
  const b = path.resolve(pathB);
  if (process.platform === "win32") {
    return a.toLowerCase() === b.toLowerCase();
  }
  return a === b;
}

export function readBranchName(prdPath) {
  const prdContent = fs.readFileSync(prdPath, "utf-8");
  const prd = JSON.parse(prdContent);
  return prd.branchName;
}

export function readLastBranch(lastBranchPath) {
  return fs.readFileSync(lastBranchPath, "utf-8").trim();
}

export function writeLastBranch(lastBranchPath, branchName) {
  fs.writeFileSync(lastBranchPath, branchName);
}

export function resetProgress(progressPath, date = new Date()) {
  const today = formatDate(date);
  const content = `# Ralph Progress Log
Started: ${today}
---
`;
  fs.writeFileSync(progressPath, content);
}

export function copyIfExists(src, dest) {
  if (fs.existsSync(src)) {
    fs.copyFileSync(src, dest);
  }
}

export function createArchiveFolder(baseDir, branchName, date = new Date()) {
  const folderName = `${formatDate(date)}-${stripPrefix(branchName)}`;
  const folderPath = path.join(baseDir, folderName);
  fs.mkdirSync(folderPath, { recursive: true });
  return folderPath;
}

export function archiveIfBranchChanged(baseDir = scriptDir, date = new Date()) {
  const prdPath = path.join(baseDir, "prd.json");
  if (!fs.existsSync(prdPath)) {
    return;
  }
  const lastBranchPath = path.join(baseDir, ".last-branch");
  const progressPath = path.join(baseDir, "progress.txt");
  const archivePath = path.join(baseDir, "archive");

  const currentBranch = readBranchName(prdPath);
  const hasLastBranch = fs.existsSync(lastBranchPath);
  const lastBranch = hasLastBranch ? readLastBranch(lastBranchPath) : null;

  if (hasLastBranch && lastBranch === currentBranch) {
    return;
  }

  fs.mkdirSync(archivePath, { recursive: true });
  const archiveFolder = createArchiveFolder(archivePath, currentBranch, date);

  copyIfExists(prdPath, path.join(archiveFolder, "prd.json"));
  copyIfExists(progressPath, path.join(archiveFolder, "progress.txt"));

  resetProgress(progressPath, date);
  writeLastBranch(lastBranchPath, currentBranch);
}

export function parseArgs(args) {
  let tool = "amp";
  let maxIterations = 10;
  let toolArgs = [];
  let promptFile = null;

  for (let i = 0; i < args.length; i++) {
    const arg = args[i];

    if (arg === "--tool") {
      if (i + 1 < args.length) {
        tool = args[i + 1];
        i++;
      }
    } else if (arg.startsWith("--tool=")) {
      tool = arg.substring(7);
    } else if (arg === "--tool-args") {
      if (i + 1 < args.length) {
        toolArgs.push(args[i + 1]);
        i++;
      }
    } else if (arg === "--prompt-file") {
      if (i + 1 < args.length) {
        promptFile = args[i + 1];
        i++;
      }
    } else if (arg.startsWith("--prompt-file=")) {
      promptFile = arg.substring(14);
    } else if (/^\d+$/.test(arg)) {
      maxIterations = parseInt(arg, 10);
    }
  }

  // Set default prompt file if not provided
  if (!promptFile) {
    promptFile = tool === "claude" ? "CLAUDE.md" : "prompt.md";
  }

  return { tool, maxIterations, toolArgs, promptFile };
}

export function runLoop(
  { tool, maxIterations, toolArgs = [], promptFile },
  baseDir = scriptDir,
  options = {}
) {
  const resolvedPromptFile =
    promptFile ?? (tool === "claude" ? "CLAUDE.md" : "prompt.md");
  const spawnImpl = options.spawn ?? spawn;
  const stdoutStream = options.stdout ?? process.stdout;
  const stderrStream = options.stderr ?? process.stderr;
  const createReadStreamImpl = options.createReadStream ?? fs.createReadStream;

  return new Promise((resolve) => {
    let iteration = 1;
    let completed = false;

    const runIteration = () => {
      if (iteration > maxIterations) {
        resolve({ completed, iterations: iteration - 1 });
        return;
      }

      const inputPath = path.isAbsolute(resolvedPromptFile)
        ? resolvedPromptFile
        : path.join(baseDir, resolvedPromptFile);
      const child = spawnImpl(tool, toolArgs, { stdio: ["pipe", "pipe", "pipe"] });

      let outputBuffer = "";
      let finished = false;

      const handleData = (chunk) => {
        const text = chunk.toString();
        outputBuffer += text;
      };

      const finishIteration = () => {
        if (finished) {
          return;
        }
        finished = true;
        if (outputBuffer.includes("<promise>COMPLETE</promise>")) {
          completed = true;
          resolve({ completed, iterations: iteration });
          return;
        }
        iteration += 1;
        runIteration();
      };

      child.stdout.on("data", handleData);
      child.stderr.on("data", handleData);

      child.stdout.pipe(stdoutStream);
      child.stderr.pipe(stderrStream);

      if (fs.existsSync(inputPath)) {
        const stream = createReadStreamImpl(inputPath);
        stream.pipe(child.stdin);
      } else {
        child.stdin.end();
      }

      child.on("error", () => {
        finishIteration();
      });

      child.on("close", () => {
        finishIteration();
      });
    };

    runIteration();
  });
}

const testMode = process.env.RALPH_TEST_MODE;

const entrypoint = resolveEntrypointPath(process.argv[1] ?? "");
const meta = fileURLToPath(import.meta.url);
const isEntrypoint = pathsAreEqual(meta, entrypoint);

const createTestSpawn = () => {
  let iteration = 0;

  return () => {
    iteration += 1;
    const output = testMode === "complete"
      ? `Iteration ${iteration}\n<promise>COMPLETE</promise>\n`
      : `Iteration ${iteration}: Normal output\n`;

    const stdoutStream = new PassThrough();
    const stderrStream = new PassThrough();

    stdoutStream.end(output);

    return {
      stdout: stdoutStream,
      stderr: stderrStream,
      on: (event, handler) => {
        if (event === "close") {
          setImmediate(() => handler(0));
        }
      },
      stdin: new PassThrough(),
    };
  };
};

if (isEntrypoint) {
  const options = {};

  if (testMode) {
    options.spawn = createTestSpawn();
  }

  runLoop(parseArgs(process.argv.slice(2)), process.cwd(), options).then(() => {
    process.exit(0);
  });
}
