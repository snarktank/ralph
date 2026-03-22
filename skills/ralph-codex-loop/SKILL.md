---
name: ralph-codex-loop
description: Set up and run the Ralph-style autonomous coding loop with Codex in any git repository. Use this skill whenever the user wants to copy the Ralph runner into another project, make the loop reusable across repositories, or run Codex-driven iterations from a prd.json task list.
---

# Ralph Codex Loop

Use this skill to make the Ralph loop reusable in another project and to run it with Codex.

## What this skill is for

- Copy the Ralph runner into a new git repository
- Run Codex as the loop engine
- Keep the project memory in `prd.json`, `progress.txt`, and git history
- Explain the shortest repeatable way to use the loop in day-to-day work

## Short answer

The reusable unit is a folder. Put the runner files in `scripts/ralph/`, then call:

```bash
./scripts/ralph/ralph.sh --tool codex 10
```

Use `1` instead of `10` when you want a single pass.

## When to use this skill

- The user asks how to use Ralph with Codex in another project
- The user wants a one-folder setup they can copy between repositories
- The user wants to turn a PRD into repeated Codex iterations
- The user asks for the shortest install-and-run instructions for Ralph

## Setup in a new project

1. Make sure the target project is a git repository.
2. Make sure `jq` and `codex` are installed.
3. Create `scripts/ralph/`.
4. Copy the runner and prompt files into that folder:
   - `ralph.sh`
   - `CODEX.md`
5. Make the runner executable.
6. Add or prepare `prd.json` in the project root.
7. Run the loop with Codex.

```bash
mkdir -p scripts/ralph
cp /path/to/ralph/ralph.sh scripts/ralph/
cp /path/to/ralph/CODEX.md scripts/ralph/
chmod +x scripts/ralph/ralph.sh
./scripts/ralph/ralph.sh --tool codex 10
```

## What the loop does

Each iteration:

1. Reads `prd.json`
2. Picks the highest-priority story with `passes: false`
3. Implements only that story
4. Runs the project's checks
5. Commits the changes if checks pass
6. Marks the story as passed
7. Appends notes to `progress.txt`
8. Stops when all stories pass or the iteration limit is hit

## Keep the instructions honest

- Do not add mock data just to make the loop appear to work.
- Do not hide failing checks behind a fallback.
- Do not mark stories done unless the repo state actually changed.

## Day-to-day commands

| Goal | Command |
|---|---|
| Run the loop | `./scripts/ralph/ralph.sh --tool codex 10` |
| Run one pass | `./scripts/ralph/ralph.sh --tool codex 1` |
| Check script syntax | `bash -n scripts/ralph/ralph.sh` |
| Check whitespace | `git diff --check` |

## If the user also wants PRD tooling

If the new project also needs PRD generation and conversion, copy the Ralph skills as well:

- `skills/prd/`
- `skills/ralph/`

Those are optional. The Codex loop works without them.
