### Cursor Support for Ralph — Execution Steps

This is a checklist-style execution plan derived from `master-plan.md`, formatted as steps with acceptance criteria (similar spirit to `prd.json`, but in markdown with checkboxes).

### Step 1 — Templates live in `scripts/ralph/` (packaging is separate work)
- [x] **Acceptance criteria**
  - [x] All repo-local templates for Cursor support are placed under `scripts/ralph/` (runner + prompts + examples).
  - [x] `ralph/plan/installation/plan.md` remains the separate follow-up plan for packaging/distribution (npm/brew), not implemented here.

### Step 2 — Add Cursor prompt contract (repo-local template)
- [x] **Acceptance criteria**
  - [x] A repo-local Cursor prompt template exists (e.g., `scripts/ralph/prompt.cursor.md`).
  - [x] Cursor prompt contract mirrors Ralph conventions:
    - [x] Read `prd.json`
    - [x] Read `progress.txt` (check “Codebase Patterns” section first)
    - [x] Work on ONE story per iteration (`passes:false` highest priority)
    - [x] Run quality checks
    - [x] Commit if checks pass
    - [x] Update `prd.json` (`passes:true`) and append to `progress.txt`
  - [x] Stop condition remains `<promise>COMPLETE</promise>` when all stories pass.

### Step 3 — Add worker selection to the runner (Amp default, Cursor optional)
- [x] **Acceptance criteria**
  - [x] Runner supports selecting worker (`amp` default; `cursor` optional).
  - [x] Selection mechanism is documented in-code (e.g., env var like `RALPH_WORKER=cursor` or a `--worker cursor` flag).
  - [x] When worker is `amp`, the runner still uses `prompt.md` and behaves like current Ralph.
  - [x] When worker is `cursor`, the runner uses the Cursor prompt contract (see Step 2) and executes Cursor CLI.

### Step 4 — Implement Cursor execution invocation (always `--approve-mcps`)
- [x] **Acceptance criteria**
  - [x] Cursor worker uses non-interactive headless mode with file edits enabled:
    - [x] `cursor --model auto --print --force --approve-mcps "<PROMPT_TEXT>"`
  - [x] Cursor worker always uses normal spawn (never PTY).
  - [x] stdin is closed (no interactive prompts).

### Step 5 — Keep progress tracking aligned with current Ralph (`progress.txt`)
- [x] **Acceptance criteria**
  - [x] Cursor worker uses single `progress.txt` file (no split) for the initial implementation.
  - [x] Cursor prompt instructs: read “Codebase Patterns” section first.
  - [x] Cursor appends progress entries (never overwrites) after completing a story.

### Step 6 — Browser verification parity rules for Cursor
- [x] **Acceptance criteria**
  - [x] `steps.md`/plan documents that Amp’s `dev-browser` skill is Amp-provided.
  - [x] Cursor worker behavior:
    - [x] If no browser tools are configured, browser verification is skipped and the story is marked “needs manual verification” unless tests cover it.
    - [x] If a browser MCP server is configured, browser verification can run via MCP (since `--approve-mcps` is always included).

### Step 7 — Reliability guardrails (match Ralph minimalism; borrow select cursor-runner ideas)
- [x] **Acceptance criteria**
  - [x] Cursor worker has a per-iteration hard timeout (wall-clock).
  - [x] Cursor worker supports MCP cleanup after each run (when MCPs are used).
  - [x] `prd.json` validation stays lightweight:
    - [x] No strict schema gate in the loop
    - [x] Optional best-effort “is valid JSON” check is allowed
    - [x] Best-effort reads (like `jq -r '.branchName // empty' ...` with error suppression) are acceptable

### Step 8 — PRD flow support (Cursor equivalents of Amp skills)
- [x] **Acceptance criteria**
  - [x] “Initial prompt → PRD markdown” flow is represented for Cursor:
    - [x] Provide a repo Cursor rule: `.cursor/rules/ralph-prd.mdc`
  - [x] “PRD markdown → `prd.json`” flow is represented for Cursor:
    - [x] Provide a repo-local prompt template (e.g., `prompt.convert-to-prd-json.md`) that writes/updates `prd.json` following `prd.json.example`
  - [x] PRD conversion output guidelines match Ralph conventions:
    - [x] Stories are small enough for one iteration
    - [x] Stories are ordered by dependency
    - [x] Acceptance criteria are verifiable

### Step 9 — Automated tests (required)
- [x] **Acceptance criteria**
  - [x] Tests run in CI/local without requiring real Amp or real Cursor (use stub binaries injected via `PATH`).
  - [x] **Worker selection tests**
    - [x] Default worker is Amp when no worker is specified.
    - [x] Cursor worker is used only when explicitly selected.
  - [x] **Cursor invocation tests**
    - [x] Cursor command includes `--model auto --print --force --approve-mcps`.
    - [x] Cursor invocation uses normal spawning assumptions (no PTY mode).
  - [x] **Stop condition tests**
    - [x] If worker output contains `<promise>COMPLETE</promise>`, the loop exits successfully.
    - [x] If worker output does NOT contain `<promise>COMPLETE</promise>`, the loop proceeds to the next iteration (up to max iterations).
  - [x] **Progress/PRD handling tests**
    - [x] `progress.txt` is append-only (never overwritten by a normal iteration).
    - [x] `prd.json` best-effort parsing failures (missing/invalid JSON) do not crash the runner (match current Ralph conventions).

### Step 10 — Verify Amp behavior is unchanged (final regression check)
- [x] **Acceptance criteria**
  - [x] `amp` worker behavior is unchanged (same `amp` invocation + same `prompt.md` semantics).
  - [x] Amp remains the default worker unless explicitly switched.
  - [x] No existing Amp skills/flow are removed or renamed as part of Cursor support.

