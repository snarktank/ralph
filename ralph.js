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

  for (let i = 0; i < args.length; i++) {
    const arg = args[i];

    if (arg === "--tool") {
      if (i + 1 < args.length) {
        tool = args[i + 1];
        i++;
      }
    } else if (arg.startsWith("--tool=")) {
      tool = arg.substring(7);
    } else if (/^\d+$/.test(arg)) {
      maxIterations = parseInt(arg, 10);
    }
  }

  return { tool, maxIterations };
}

export function runLoop(
  { tool, maxIterations },
  baseDir = scriptDir,
  options = {}
) {
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

      const inputFile = tool === "claude" ? "CLAUDE.md" : "prompt.md";
      const inputPath = path.join(baseDir, inputFile);
      const child = spawnImpl(tool, [], { stdio: ["pipe", "pipe", "pipe"] });

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

const isEntrypoint = fileURLToPath(import.meta.url) === path.resolve(process.argv[1] ?? "");

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
