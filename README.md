# Ralph

![Ralph](ralph.webp)

Ralph is a long-running autonomous task loop for [Codex CLI](https://github.com/openai/codex). It runs one fresh Codex instance per PRD story until every story in `prd.json` is complete. Memory persists through git history, `progress.txt`, `prd.json`, and structured run logs under `runs/`.

Ralph still supports Amp and Claude Code as legacy tools, but Codex CLI is the default path.

Based on [Geoffrey Huntley's Ralph pattern](https://ghuntley.com/ralph/).

## Prerequisites

- Codex CLI installed and authenticated.
- `jq` installed.
- Bash available. On Windows, use Git Bash or WSL.
- A git repository for the project you want Ralph to work in.

Legacy optional tools:

- [Amp CLI](https://ampcode.com)
- [Claude Code](https://docs.anthropic.com/en/docs/claude-code)

## Setup

Copy the Ralph files into your project:

```bash
mkdir -p scripts/ralph
cp /path/to/ralph/ralph.sh scripts/ralph/
cp /path/to/ralph/CODEX.md scripts/ralph/
cp /path/to/ralph/prd.json.example scripts/ralph/
chmod +x scripts/ralph/ralph.sh
```

For legacy tools, you can also copy `prompt.md` for Amp or `CLAUDE.md` for Claude Code.

### Install Skills In Codex

This repository includes a Codex plugin manifest at `.codex-plugin/plugin.json` and two skills:

- `prd` - Generate Product Requirements Documents.
- `ralph` - Convert PRDs to `prd.json`.

You can use the skills from this repository or copy them into your Codex skills directory.

Claude Code marketplace support is still present under `.claude-plugin/`.

## Workflow

### 1. Create A PRD

Use the PRD skill to generate a detailed requirements document:

```text
Load the prd skill and create a PRD for [your feature description]
```

The skill saves output to `tasks/prd-[feature-name].md`.

### 2. Convert The PRD To Ralph JSON

Use the Ralph skill to convert the markdown PRD to JSON:

```text
Load the ralph skill and convert tasks/prd-[feature-name].md to prd.json
```

Place the generated `prd.json` in the same directory as `ralph.sh`.

### 3. Run Ralph With Codex CLI

```bash
./scripts/ralph/ralph.sh [max_iterations]
```

Default max iterations is 10.

On Windows Git Bash, if `codex` does not resolve correctly, point Ralph at the `.cmd` shim:

```bash
CODEX_BIN=codex.cmd ./scripts/ralph/ralph.sh 20
```

Ralph runs Codex with:

```bash
codex exec --dangerously-bypass-approvals-and-sandbox --cd <project-root> --color never
```

This is intentionally powerful for unattended automation. Run it only in repositories and environments where you are comfortable giving Codex full command execution access.

### Legacy Tool Selection

```bash
./scripts/ralph/ralph.sh --tool codex 20
./scripts/ralph/ralph.sh --tool amp 20
./scripts/ralph/ralph.sh --tool claude 20
```

## Runner Behavior

Ralph will:

1. Check required tools and validate `prd.json`.
2. Acquire `.ralph.lock` so only one loop runs in the directory.
3. Create `runs/YYYYMMDD-HHMMSS/` for structured logs.
4. Pick the highest priority story where `passes: false`.
5. Launch a fresh Codex CLI instance with `CODEX.md` plus runtime context.
6. Let Codex implement one story, run checks, update `prd.json`, append `progress.txt`, and commit.
7. Save each iteration's prompt, output, last message, git status, diff stat, and `status.json`.
8. Retry transient CLI, network, auth, or permission failures.
9. Stop when all stories pass or max iterations is reached.

If Codex finishes normally but does not update the PRD or does not create a commit for the completed story, Ralph records a story failure and gives the next fresh context a chance to repair it. If the same story fails repeatedly, Ralph stops after `RALPH_MAX_STORY_FAILURES`.

## Configuration

| Variable | Purpose |
| --- | --- |
| `CODEX_BIN` | Codex executable, such as `codex` or `codex.cmd` |
| `RALPH_MODEL` | Optional model passed to `codex exec --model` |
| `RALPH_PROFILE` | Optional profile passed to `codex exec --profile` |
| `RALPH_CODEX_FLAGS` | Extra flags appended to `codex exec` |
| `RALPH_MAX_RETRIES` | Retry count for transient CLI failures, default `2` |
| `RALPH_MAX_STORY_FAILURES` | Consecutive failures allowed for one story, default `3` |
| `RALPH_REQUIRE_CLEAN` | Set to `1` to stop when git status is dirty |

## Key Files

| File | Purpose |
| --- | --- |
| `ralph.sh` | Bash runner that launches fresh agent instances |
| `CODEX.md` | Prompt template for Codex CLI iterations |
| `prompt.md` | Legacy prompt template for Amp |
| `CLAUDE.md` | Legacy prompt template for Claude Code |
| `prd.json` | Runtime story list with `passes` status |
| `prd.json.example` | Example PRD format |
| `progress.txt` | Append-only cross-iteration memory |
| `runs/` | Per-run structured logs |
| `skills/prd/` | Skill for generating PRDs |
| `skills/ralph/` | Skill for converting PRDs to JSON |
| `.codex-plugin/` | Codex plugin manifest |
| `.claude-plugin/` | Claude Code marketplace manifest |

## Critical Concepts

### Each Iteration Has Fresh Context

Every iteration starts a new Codex CLI process. The only durable memory is:

- Git history
- `progress.txt`
- `prd.json`
- Structured logs in `runs/`

### Keep Stories Small

Each PRD story should fit in one focused Codex context.

Right-sized stories:

- Add a database column and migration.
- Add a UI component to an existing page.
- Update one server action.
- Add one filter dropdown.

Too large:

- Build the entire dashboard.
- Add authentication end to end.
- Refactor the API.

### AGENTS.md Updates Matter

When an iteration discovers reusable codebase knowledge, it should update relevant `AGENTS.md` files. This lets future Codex iterations and human developers reuse discovered patterns.

Useful additions include:

- Local API conventions.
- Files that must be changed together.
- Testing requirements for a module.
- Non-obvious setup or environment constraints.

### Browser Verification For UI Stories

Frontend stories should include browser verification in acceptance criteria. Codex should use available browser tools when present. If browser tooling is unavailable, the progress entry must say manual browser verification is still needed.

### Stop Condition

Ralph stops when `prd.json` has all stories marked with `passes: true`. Codex may also output:

```text
<promise>COMPLETE</promise>
```

Ralph verifies PRD completion before accepting a complete run.

## Debugging

Check current state:

```bash
cat prd.json | jq '.userStories[] | {id, title, passes}'
cat progress.txt
git log --oneline -10
```

Inspect a specific run:

```bash
ls runs/
cat runs/<run-id>/iteration-1/status.json
cat runs/<run-id>/iteration-1/output.log
cat runs/<run-id>/iteration-1/git-status.txt
```

## Archiving

Ralph automatically archives previous runtime files when a new `prd.json` uses a different `branchName`. Archives are saved to `archive/YYYY-MM-DD-feature-name/`.

## References

- [Geoffrey Huntley's Ralph article](https://ghuntley.com/ralph/)
- [Codex CLI](https://github.com/openai/codex)
- [Amp documentation](https://ampcode.com/manual)
- [Claude Code documentation](https://docs.anthropic.com/en/docs/claude-code)
