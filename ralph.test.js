import test from "node:test";
import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import os from "node:os";
import { PassThrough } from "node:stream";
import { fileURLToPath } from "node:url";
import { parseArgs, archiveIfBranchChanged, runLoop } from "./ralph.js";

const scriptPath = path.join(path.dirname(fileURLToPath(import.meta.url)), "ralph.js");

test("parseArgs supports --tool amp|claude", () => {
  assert.deepEqual(parseArgs(["--tool", "amp"]), {
    tool: "amp",
    maxIterations: 10,
  });

  assert.deepEqual(parseArgs(["--tool", "claude"]), {
    tool: "claude",
    maxIterations: 10,
  });
});

test("parseArgs supports --tool=amp syntax", () => {
  assert.deepEqual(parseArgs(["--tool=amp"]), {
    tool: "amp",
    maxIterations: 10,
  });
});

test("parseArgs supports max_iterations positional arg", () => {
  assert.deepEqual(parseArgs(["7"]), {
    tool: "amp",
    maxIterations: 7,
  });

  assert.deepEqual(parseArgs(["--tool", "claude", "3"]), {
    tool: "claude",
    maxIterations: 3,
  });
});

test("parseArgs returns defaults for empty args", () => {
  assert.deepEqual(parseArgs([]), {
    tool: "amp",
    maxIterations: 10,
  });
});

test("archiveIfBranchChanged archives and resets when branch changes", () => {
  const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "ralph-test-"));
  const date = new Date("2026-01-22T10:20:30Z");

  const branchName = "ralph/new-feature";
  const prdContent = JSON.stringify({ branchName });
  fs.writeFileSync(path.join(tmpDir, "prd.json"), prdContent);

  const progressContent = "Old progress log content";
  fs.writeFileSync(path.join(tmpDir, "progress.txt"), progressContent);

  fs.writeFileSync(path.join(tmpDir, ".last-branch"), "ralph/old-feature");

  archiveIfBranchChanged(tmpDir, date);

  const lastBranch = fs.readFileSync(path.join(tmpDir, ".last-branch"), "utf-8");
  assert.equal(lastBranch, branchName);

  const archiveDir = path.join(tmpDir, "archive");
  assert.ok(fs.existsSync(archiveDir));

  const files = fs.readdirSync(archiveDir);
  assert.equal(files.length, 1);

  const folderName = files[0];
  const today = "2026-01-22";
  assert.equal(folderName, `${today}-new-feature`);

  const archivedPrd = fs.readFileSync(path.join(archiveDir, folderName, "prd.json"), "utf-8");
  assert.equal(archivedPrd, prdContent);

  const archivedProgress = fs.readFileSync(path.join(archiveDir, folderName, "progress.txt"), "utf-8");
  assert.equal(archivedProgress, progressContent);

  const newProgress = fs.readFileSync(path.join(tmpDir, "progress.txt"), "utf-8");
  assert.ok(newProgress.includes("# Ralph Progress Log"));
  assert.ok(newProgress.includes(`Started: ${today}`));
  assert.ok(newProgress.includes("---"));
  assert.ok(!newProgress.includes("Old progress log content"));
});

test("archiveIfBranchChanged does nothing if branch is same", () => {
  const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "ralph-test-same-"));
  const date = new Date("2026-01-22T10:20:30Z");

  const branchName = "ralph/same-feature";
  fs.writeFileSync(path.join(tmpDir, "prd.json"), JSON.stringify({ branchName }));
  fs.writeFileSync(path.join(tmpDir, "progress.txt"), "Current progress");
  fs.writeFileSync(path.join(tmpDir, ".last-branch"), branchName);

  archiveIfBranchChanged(tmpDir, date);

  assert.ok(!fs.existsSync(path.join(tmpDir, "archive")));

  const progress = fs.readFileSync(path.join(tmpDir, "progress.txt"), "utf-8");
  assert.equal(progress, "Current progress");
});

test("archiveIfBranchChanged handles missing files", () => {
  const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "ralph-test-missing-"));
  const date = new Date("2026-01-22T10:20:30Z");

  const branchName = "ralph/feature-missing";
  fs.writeFileSync(path.join(tmpDir, "prd.json"), JSON.stringify({ branchName }));

  archiveIfBranchChanged(tmpDir, date);

  const archiveDir = path.join(tmpDir, "archive");
  assert.ok(fs.existsSync(archiveDir));
  const files = fs.readdirSync(archiveDir);
  const folderName = files[0];

  assert.ok(fs.existsSync(path.join(archiveDir, folderName, "prd.json")));
  assert.ok(!fs.existsSync(path.join(archiveDir, folderName, "progress.txt")));

  const newProgress = fs.readFileSync(path.join(tmpDir, "progress.txt"), "utf-8");
  assert.ok(newProgress.includes("# Ralph Progress Log"));
  assert.ok(newProgress.includes("Started: 2026-01-22"));
});

test("archiveIfBranchChanged does nothing when prd.json is missing", () => {
  const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "ralph-test-no-prd-"));
  const date = new Date("2026-01-22T10:20:30Z");

  archiveIfBranchChanged(tmpDir, date);

  assert.ok(!fs.existsSync(path.join(tmpDir, "archive")));
  assert.ok(!fs.existsSync(path.join(tmpDir, "progress.txt")));
  assert.ok(!fs.existsSync(path.join(tmpDir, ".last-branch")));
});

test("runLoop spawns tool process on each iteration", async () => {
  const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "ralph-test-runloop-"));
  const promptPath = path.join(tmpDir, "prompt.md");
  const claudePath = path.join(tmpDir, "CLAUDE.md");
  fs.writeFileSync(promptPath, "test prompt");
  fs.writeFileSync(claudePath, "test claude instructions");

  const callLog = [];
  const mockSpawn = (command, args) => {
    const processNumber = callLog.length + 1;

    callLog.push({ command, args });

    let stdoutOutput = "";
    let stderrOutput = "";
    let closeCode = 0;

    return {
      stdout: {
        on: (event, handler) => {
          if (event === "data") {
            stdoutOutput = `Iteration ${processNumber}: Processing...\n`;
            handler(Buffer.from(stdoutOutput));
          }
        },
        pipe: (dest) => {
          dest.write(stdoutOutput);
        },
      },
      stderr: {
        on: (event, handler) => {
          if (event === "data") {
            stderrOutput = `Iteration ${processNumber}: stderr\n`;
            handler(Buffer.from(stderrOutput));
          }
        },
        pipe: (dest) => {
          dest.write(stderrOutput);
        },
      },
      on: (event, handler) => {
        if (event === "close") handler(closeCode);
      },
      stdin: {
        write: () => {},
        end: () => {},
      },
    };
  };

  const result = await runLoop(
    { tool: "amp", maxIterations: 3 },
    tmpDir,
    {
      spawn: mockSpawn,
      stdout: new PassThrough(),
      stderr: new PassThrough(),
      createReadStream: () => ({ pipe: () => {} }),
    }
  );

  assert.equal(callLog.length, 3);
  assert.equal(callLog[0].command, "amp");
  assert.equal(callLog[1].command, "amp");
  assert.equal(callLog[2].command, "amp");
  assert.equal(result.completed, false);
});

test("runLoop breaks iteration when COMPLETE signal found", async () => {
  const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "ralph-test-complete-"));
  const promptPath = path.join(tmpDir, "prompt.md");
  const claudePath = path.join(tmpDir, "CLAUDE.md");
  fs.writeFileSync(promptPath, "test prompt");
  fs.writeFileSync(claudePath, "test claude instructions");

  const callLog = [];

  const mockSpawn = (command, args) => {
    const processNumber = callLog.length + 1;
    callLog.push({ command, args });

    let stdoutOutput = "";
    let stderrOutput = "";
    let closeCode = 0;

    return {
      stdout: {
        on: (event, handler) => {
          if (event === "data") {
            if (processNumber === 2) {
              stdoutOutput =
                "Processing...\n<promise>COMPLETE</promise>\n";
            } else {
              stdoutOutput = `Iteration ${processNumber}: Processing...\n`;
            }
            handler(Buffer.from(stdoutOutput));
          }
        },
        pipe: (dest) => {
          dest.write(stdoutOutput);
        },
      },
      stderr: {
        on: () => {},
        pipe: () => {},
      },
      on: (event, handler) => {
        if (event === "close") handler(closeCode);
      },
      stdin: {
        write: () => {},
        end: () => {},
      },
    };
  };

  const result = await runLoop(
    { tool: "claude", maxIterations: 5 },
    tmpDir,
    {
      spawn: mockSpawn,
      stdout: new PassThrough(),
      stderr: new PassThrough(),
      createReadStream: () => ({ pipe: () => {} }),
    }
  );

  assert.equal(callLog.length, 2, "Should stop after 2 iterations");
  assert.equal(result.completed, true);
  assert.equal(result.iterations, 2);
});

test("runLoop exits after maxIterations without COMPLETE signal", async () => {
  const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "ralph-test-maxiter-"));
  const promptPath = path.join(tmpDir, "prompt.md");
  const claudePath = path.join(tmpDir, "CLAUDE.md");
  fs.writeFileSync(promptPath, "test prompt");
  fs.writeFileSync(claudePath, "test claude instructions");

  const callLog = [];

  const mockSpawn = (command, args) => {
    const processNumber = callLog.length + 1;
    callLog.push({ command, args });

    let stdoutOutput = "Normal output - no completion\n";
    let closeCode = 0;

    return {
      stdout: {
        on: (event, handler) => {
          if (event === "data") {
            handler(Buffer.from(stdoutOutput));
          }
        },
        pipe: (dest) => {
          dest.write(stdoutOutput);
        },
      },
      stderr: {
        on: () => {},
        pipe: () => {},
      },
      on: (event, handler) => {
        if (event === "close") handler(closeCode);
      },
      stdin: {
        write: () => {},
        end: () => {},
      },
    };
  };

  const result = await runLoop(
    { tool: "amp", maxIterations: 3 },
    tmpDir,
    {
      spawn: mockSpawn,
      stdout: new PassThrough(),
      stderr: new PassThrough(),
      createReadStream: () => ({ pipe: () => {} }),
    }
  );

  assert.equal(callLog.length, 3, "Should run exactly maxIterations");
  assert.equal(result.completed, false);
  assert.equal(result.iterations, 3);
});

test("runLoop handles non-zero exit code from tool", async () => {
  const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "ralph-test-exitcode-"));
  const promptPath = path.join(tmpDir, "prompt.md");
  fs.writeFileSync(promptPath, "test prompt");

  const mockSpawn = () => {
    return {
      stdout: {
        on: () => {},
        pipe: () => {},
      },
      stderr: {
        on: () => {},
        pipe: () => {},
      },
      on: (event, handler) => {
        if (event === "close") handler(1);
      },
      stdin: {
        write: () => {},
        end: () => {},
      },
    };
  };

  const result = await runLoop(
    { tool: "amp", maxIterations: 1 },
    tmpDir,
    {
      spawn: mockSpawn,
      stdout: new PassThrough(),
      stderr: new PassThrough(),
      createReadStream: () => ({ pipe: () => {} }),
    }
  );

  assert.equal(result.completed, false);
  assert.equal(result.iterations, 1);
});

test("runLoop continues after spawn error", { timeout: 500 }, async () => {
  const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "ralph-test-spawn-error-"));
  const promptPath = path.join(tmpDir, "prompt.md");
  fs.writeFileSync(promptPath, "test prompt");

  let iteration = 0;
  const mockSpawn = () => {
    iteration += 1;
    const listeners = {};

    return {
      stdout: {
        on: () => {},
        pipe: () => {},
      },
      stderr: {
        on: () => {},
        pipe: () => {},
      },
      on: (event, handler) => {
        listeners[event] = handler;
        if (event === "error") {
          handler(new Error("spawn error"));
        }
      },
      stdin: {
        write: () => {},
        end: () => {},
      },
    };
  };

  const result = await runLoop(
    { tool: "amp", maxIterations: 2 },
    tmpDir,
    {
      spawn: mockSpawn,
      stdout: new PassThrough(),
      stderr: new PassThrough(),
      createReadStream: () => ({ pipe: () => {} }),
    }
  );

  assert.equal(iteration, 2);
  assert.equal(result.completed, false);
  assert.equal(result.iterations, 2);
});

test("script entrypoint exits successfully on COMPLETE", async () => {
  const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "ralph-test-cli-complete-"));
  const env = {
    ...process.env,
    RALPH_TEST_MODE: "complete",
  };

  const result = await new Promise((resolve) => {
    const child = spawn(process.execPath, [scriptPath], { env, cwd: tmpDir });
    let output = "";

    child.stdout.on("data", (chunk) => {
      output += chunk.toString();
    });

    child.stderr.on("data", (chunk) => {
      output += chunk.toString();
    });

    child.on("close", (code) => {
      resolve({ code, output });
    });
  });

  assert.equal(result.code, 0);
  assert.ok(result.output.includes("<promise>COMPLETE</promise>"));
});

test("script entrypoint exits successfully after maxIterations", async () => {
  const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "ralph-test-cli-maxiter-"));
  const env = {
    ...process.env,
    RALPH_TEST_MODE: "no-complete",
  };

  const result = await new Promise((resolve) => {
    const child = spawn(process.execPath, [scriptPath, "2"], { env, cwd: tmpDir });
    let output = "";

    child.stdout.on("data", (chunk) => {
      output += chunk.toString();
    });

    child.stderr.on("data", (chunk) => {
      output += chunk.toString();
    });

    child.on("close", (code) => {
      resolve({ code, output });
    });
  });

  assert.equal(result.code, 0);
  assert.ok(result.output.includes("Iteration 2"));
});
