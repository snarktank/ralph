# Ralph Agent Instructions

## Overview

Ralph is an autonomous AI agent loop that runs Codex CLI repeatedly until all PRD items are complete. Each iteration is a fresh Codex instance with clean context.

## Commands

```bash
# Run the flowchart dev server
cd flowchart && npm run dev

# Build the flowchart
cd flowchart && npm run build

# Run Ralph (from your project that has prd.json)
./ralph.sh [max_iterations]

# Use Amp instead of Codex (optional)
RALPH_ENGINE=amp ./ralph.sh [max_iterations]
```

## Key Files

- `ralph.sh` - The bash loop that spawns fresh Codex instances
- `prompt.md` - Instructions given to each Codex instance
- `prd.json.example` - Example PRD format
- `flowchart/` - Interactive React Flow diagram explaining how Ralph works
- `.codex/skills/ralph-codex/` - Repo-local Codex skill for Ralph conventions

## Flowchart

The `flowchart/` directory contains an interactive visualization built with React Flow. It's designed for presentations - click through to reveal each step with animations.

To run locally:
```bash
cd flowchart
npm install
npm run dev
```

## Loop Rules

- Work on a single highest-priority story where `passes: false`
- Run the project's quality checks before committing
- Update `prd.json` and append to `progress.txt` every iteration
- Emit `<promise>COMPLETE</promise>` when all stories pass

## Codex Settings

- Default is read-only; enable edits with `RALPH_CODEX_FULL_AUTO=1`
- Override sandbox with `RALPH_CODEX_SANDBOX=workspace-write` or `danger-full-access`

## Patterns

- Each iteration spawns a fresh Codex instance with clean context
- Memory persists via git history, `progress.txt`, and `prd.json`
- Stories should be small enough to complete in one context window
- Always update AGENTS.md with discovered patterns for future iterations
