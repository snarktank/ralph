### Master Plan: Add Cursor support to Ralph (replace Amp as the “worker”)

### Goals
- **Add Cursor as an optional second worker** (`cursor` / “cursor-cli”) while keeping Amp support unchanged.
- Keep Ralph’s core paradigm: **one small story per iteration**, “done” tracked in `prd.json`, with durable carryover via a lightweight memory artifact.
- Preserve Ralph’s “portable kit” vibe: minimal infra, repo-local files, easy to drop into a client project.

### Non-goals
- Building a full web UI / multi-tenant queue system (that’s closer to `cursor-runner`/`cursor-agents` territory).
- Achieving perfect feature parity with Amp “skills” without any prompt/template work (Cursor doesn’t have Amp’s skill runtime).

### Current Ralph architecture (Amp-shaped)
- **Loop**: `ralph.sh` shells out to `amp` each iteration, piping `prompt.md`.
- **Task list**: `prd.json` with `userStories[]`, sorted by `priority`, marked complete with `passes: true`.
- **Memory**: git + `progress.txt` + “update AGENTS.md” conventions.
- **Upstream planning tools**: Amp skills in `skills/`:
  - PRD generator skill writes `tasks/prd-*.md`.
  - PRD→`prd.json` converter skill.

### Target architecture (dual-worker: Amp + Cursor)
Ralph becomes a thin orchestrator that can run one of multiple “workers”:
- **Amp worker**: existing behavior stays unchanged (same prompts/skills/flow).
- **Cursor worker**: new option that runs the same single-iteration prompt contract via Cursor CLI.

This is best implemented as:
- A small abstraction layer in the runner script: `WORKER=amp|cursor`
- Repo-local templates under `scripts/ralph/` (prompts, runner) scaffolded by `ralph init` (see `ralph/plan/installation/plan.txt`).

### Proposed worker abstraction
#### Interface: “run one iteration”
Inputs (repo-local):
- `prompt.md` (or `prompt.cursor.md`) – instructions for the worker agent
- `prd.json` – next story selection + completion tracking
- `progress.txt` – append-only log with "Codebase Patterns" section at top (current Ralph pattern)

Outputs:
- STDOUT/STDERR of the worker CLI invocation
- Worker updates `prd.json` + progress artifacts + git commits (same as today)
- Worker prints `<promise>COMPLETE</promise>` when all stories pass

#### Runner selection (minimal)
Add one of:
- **Env var**: `RALPH_WORKER=cursor` (default `amp`)
- **CLI flag**: `./ralph.sh --worker cursor`

### Cursor CLI invocation strategy
#### Baseline command shape
Use non-interactive headless mode and allow edits:
- `cursor --model auto --print --force --approve-mcps "<PROMPT_TEXT>"`

#### Borrowed stability defaults from `cursor-runner` (recommended)
These are concrete, battle-tested behaviors from `cursor-runner/src/cursor-cli.ts` that are worth copying into a “Cursor Ralph” runner:
- **Always normal spawn (never PTY)**: always use normal process spawning; do not support PTY mode in Cursor-Ralph (simplifies behavior and avoids PTY/MCP/stdin-stdout quirks).
- **Hard timeout**: kill the worker if it exceeds a wall-clock budget (cursor-runner enforces `CURSOR_CLI_TIMEOUT`).
- **Output size cap**: abort if stdout/stderr grows beyond a threshold (prevents log explosions and disk pressure).
- **Process-group termination**: on POSIX, spawn detached and kill the whole process group (prevents orphaned descendants).
- **Keep stdin closed**: run headless; don’t leave stdin open (reduces “hung waiting for input” edge cases).
- **Post-run MCP cleanup**: clean up stray/orphaned MCP processes after each run

#### Workspace trust & permissions
Cursor CLI often depends on:
- workspace trust settings
- `.cursor/cli.json` permissions for shell/file ops

For client portability, decide one:
- **Option A (repo-local config)**: ship a minimal `.cursor/cli.json` template in the target repo (recommended for consistency).

### Replace Amp “skills” with Cursor-native equivalents
Ralph currently “uses Amp” for three practical capabilities:

#### 1) Feature conversation → PRD markdown
Amp today: PRD skill (`skills/prd`).

Cursor replacement (chosen approach):
- **Cursor rule**: a `.cursor/rules/ralph-prd.mdc` rule that standardizes PRD generation instructions across repos.

#### 2) PRD markdown → `prd.json`
Amp today: converter skill (`skills/ralph`).

Cursor replacement options:
- **Prompt template**: `prompt.convert-to-prd-json.md` that instructs Cursor to:
  - read `tasks/prd-*.md`
  - produce/overwrite `prd.json` following `prd.json.example`
  - enforce “small enough for one iteration” and dependency ordering
- **Non-AI helper** (optional): a tiny schema validator script that checks `prd.json` shape and fails fast if invalid JSON or missing keys.

Key design choice:
- Target “relatively small” stories (one iteration), not “smallest possible” micro-steps.

#### 3) Execute one story per iteration until complete
Amp today: `prompt.md` instructs Amp to:
- pick next `passes:false` story
- implement + run checks
- commit + mark `passes:true`
- append to progress log
- emit completion marker when all stories done

Cursor replacement:
- A Cursor-specific `prompt.cursor.md` with the same contract, but tuned for Cursor:
  - rely on `.cursor/rules/*` for repo conventions (instead of stuffing everything in the prompt)
  - be explicit about printing `<promise>COMPLETE</promise>` only when *all* stories pass
  - specify the exact quality commands expected (typecheck/tests) for the target repo

### Progress tracking (current Ralph pattern)
For the initial cursor-support implementation, use the current Ralph pattern:
- **Single `progress.txt` file** with:
  - `## Codebase Patterns` section at the TOP (curated, reusable learnings)
  - Append-only log entries below (iteration history)
- **Prompt contract**: instruct Cursor to "check Codebase Patterns section first" when reading `progress.txt`
- **Future optimization**: See [progress-split.md](./progress-split.md) for splitting into separate files (defer to later phases)

### Browser verification parity
Amp workflow mentions "Verify in browser using dev-browser skill".

Cursor replacement options (choose per client):
- **No tools configured**: browser verification is skipped; require Playwright tests or deterministic UI tests for automated verification; otherwise mark as "needs manual verification" with a checklist item.
- **MCP-based**: if the environment has a browser MCP server configured, browser verification will run automatically (since `--approve-mcps` is always included).

### Error handling & reliability (what to borrow from cursor-runner)
Ralph (bash) is intentionally minimal. Cursor CLI can hang or be noisy; to make Cursor-Ralph robust on small VPSes, consider porting a few “platform” guardrails:
- **Hard timeout** per iteration (kill the worker if exceeded).
- **MCP cleanup** if MCP servers are used (avoid orphan processes).

Additional reliability features from `cursor-runner/src/cursor-execution-service.ts` that map well to this plan:
- **MCP config verification (diagnostic-only)**: log what was written (server names, sizes) to make debugging tool issues faster.

Implementation note: guardrails can be done in bash (simpler) or in a tiny Node/Python wrapper (more reliable process control and process-group handling).

### Proposed delivery plan (incremental, low risk)
#### Phase 1 — Minimal Cursor worker (proof of concept)
- Add `RALPH_WORKER=cursor` to runner.
- Implement Cursor invocation: `cursor --model auto --print --force --approve-mcps`.
- Add `prompt.cursor.md` (Cursor-tuned prompt contract).
- Keep the stop condition: grep for `<promise>COMPLETE</promise>`.

#### Phase 1.1 — Preserve Amp (no behavioral changes)
- Do not change `amp` invocation, Amp skills, or current Amp prompt behavior.
- Cursor support must be additive: `amp` remains the default worker unless explicitly switched.

#### Phase 2 — Replace “skills” with prompt templates
- Add PRD prompt template (feature → `tasks/prd-*.md`).
- Add converter prompt template (PRD → `prd.json`).
- Follow existing Ralph conventions: do not introduce strict schema validation in the runner; rely on `prd.json.example` + prompt instructions (optional: best-effort “is valid JSON” check only).

#### Phase 3 — Reliability hardening
- Add per-iteration timeout + output cap.
- Add workspace trust/permissions bootstrap (repo-local `.cursor/cli.json` recommended).

#### Phase 4 — Team/client packaging
- Keep templates repo-local under `scripts/ralph/` (runner + prompts + examples).

### Open questions (decide early)
- **Worker choice**: keep Amp support unchanged; add Cursor as a second option (do not go Cursor-only).
- **Templates location (decided)**: store templates repo-locally under `scripts/ralph/`. (Packaging/install automation is a separate follow-up plan.)
- **`prd.json` validation (decided; match existing Ralph)**: keep validation lightweight. Assume `prd.json` follows `prd.json.example` and is maintained by the agent. The runner may do best-effort reads (e.g., `jq -r '.branchName // empty' ...` with error suppression) but should not enforce a strict schema gate in the loop.
- **Progress file pattern**: use single `progress.txt` with "Codebase Patterns" section (current Ralph pattern).

