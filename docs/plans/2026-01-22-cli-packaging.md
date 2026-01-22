# CLI Packaging Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a local npm-based CLI package so `ralph` can be installed globally and run from any directory.

**Architecture:** Add a minimal root `package.json` with a `bin` mapping to `ralph.js`, add a shebang to `ralph.js`, and change the script base directory to `process.cwd()` so it operates on the current project. Keep zero dependencies and existing behavior.

**Tech Stack:** Node.js (ESM), npm local install (`npm -g ./repo`), node:test (existing tests).

---

### Task 1: Add root package.json for CLI

**Files:**
- Create: `/home/eros/src/ralph/package.json`

**Step 1: Write the failing test (smoke check for CLI availability)**

Add a test to `/home/eros/src/ralph/ralph.test.js` that runs the CLI via Node (using `spawn`) and asserts it exits with code 0 when invoked with `RALPH_TEST_MODE=complete`.

```js
import { spawn } from "node:child_process";
import { fileURLToPath } from "node:url";

// ... in test suite

test("cli entrypoint works via node", async () => {
  const scriptPath = path.join(path.dirname(fileURLToPath(import.meta.url)), "ralph.js");
  const env = { ...process.env, RALPH_TEST_MODE: "complete" };

  const result = await new Promise((resolve) => {
    const child = spawn(process.execPath, [scriptPath], { env });
    child.on("close", (code) => resolve(code));
  });

  assert.equal(result, 0);
});
```

**Step 2: Run test to verify it fails**

Run:
```
node /home/eros/src/ralph/ralph.test.js
```
Expected: PASS or FAIL depending on current code (if already passing, keep this test as regression).

**Step 3: Create minimal root package.json**

Create `/home/eros/src/ralph/package.json` with:

```json
{
  "name": "ralph",
  "private": true,
  "version": "0.0.0",
  "type": "module",
  "bin": {
    "ralph": "./ralph.js"
  },
  "engines": {
    "node": ">=18"
  }
}
```

**Step 4: Run test to verify it passes**

Run:
```
node /home/eros/src/ralph/ralph.test.js
```
Expected: PASS

**Step 5: Commit**

```
git add /home/eros/src/ralph/package.json /home/eros/src/ralph/ralph.test.js
git commit -m "feat: add root CLI package.json"
```

---

### Task 2: Make ralph.js executable and use cwd as base

**Files:**
- Modify: `/home/eros/src/ralph/ralph.js:1-220`
- Test: `/home/eros/src/ralph/ralph.test.js`

**Step 1: Write failing test (cwd-based baseDir)**

Add a test that sets `process.chdir(tmpDir)` and verifies `runLoop` uses `tmpDir` for `prompt.md`/`CLAUDE.md` via the entrypoint (CLI) or by calling `runLoop` with baseDir from `process.cwd()`.

```js
test("cli uses cwd as baseDir", async () => {
  const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "ralph-test-cwd-"));
  const promptPath = path.join(tmpDir, "prompt.md");
  fs.writeFileSync(promptPath, "test prompt");

  const originalCwd = process.cwd();
  process.chdir(tmpDir);

  const result = await runLoop({ tool: "amp", maxIterations: 1 }, process.cwd(), {
    spawn: () => ({
      stdout: { on: () => {}, pipe: () => {} },
      stderr: { on: () => {}, pipe: () => {} },
      on: (event, handler) => event === "close" && handler(0),
      stdin: { write: () => {}, end: () => {} },
    }),
    stdout: new PassThrough(),
    stderr: new PassThrough(),
    createReadStream: () => ({ pipe: () => {} }),
  });

  process.chdir(originalCwd);
  assert.equal(result.iterations, 1);
});
```

**Step 2: Run test to verify it fails**

Run:
```
node /home/eros/src/ralph/ralph.test.js
```
Expected: FAIL if baseDir is not from cwd (adjust as needed).

**Step 3: Add shebang and switch base directory to cwd in entrypoint**

Update `/home/eros/src/ralph/ralph.js`:

```js
#!/usr/bin/env node
```

And in the entrypoint, use `process.cwd()` as the base directory:

```js
if (isEntrypoint) {
  const options = {};
  if (testMode) {
    options.spawn = createTestSpawn();
  }
  runLoop(parseArgs(process.argv.slice(2)), process.cwd(), options).then(() => {
    process.exit(0);
  });
}
```

**Step 4: Run test to verify it passes**

Run:
```
node /home/eros/src/ralph/ralph.test.js
```
Expected: PASS

**Step 5: Commit**

```
git add /home/eros/src/ralph/ralph.js /home/eros/src/ralph/ralph.test.js
git commit -m "feat: use cwd for CLI execution"
```

---

### Task 3: Update documentation for global install

**Files:**
- Modify: `/home/eros/src/ralph/README.md`
- Modify: `/home/eros/src/ralph/AGENTS.md`

**Step 1: Add global install instructions**

Update README:

```md
## Install (local global)

```bash
npm install -g /path/to/ralph
ralph --tool claude 5
```
```

Update AGENTS.md commands section with:

```md
# Install CLI globally (local)
npm install -g /path/to/ralph

# Run Ralph from any project root
ralph --tool claude 5
```

**Step 2: Verify docs formatting**

Ensure code fences are correct and no broken markdown.

**Step 3: Commit**

```
git add /home/eros/src/ralph/README.md /home/eros/src/ralph/AGENTS.md
git commit -m "docs: add global install instructions"
```
