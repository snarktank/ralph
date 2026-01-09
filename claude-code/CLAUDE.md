# Ralph Agent Instructions (Claude Code)

## Overview

Ralph is an autonomous AI agent loop that runs Claude Code repeatedly until all PRD items are complete. Each iteration is a fresh Claude instance with clean context.

## Commands

```bash
# Run Ralph (from this directory)
./ralph.sh [max_iterations]
```

## Key Files

- `ralph.sh` - The bash loop that spawns fresh Claude instances
- `prompt.md` - Instructions given to each Claude instance
- `prd.json` - Your project's user stories (copy from prd.json.example)
- `progress.txt` - Append-only log of completed work and learnings

## Patterns

- Each iteration spawns a fresh Claude instance with clean context
- Memory persists via git history, `progress.txt`, and `prd.json`
- Stories should be small enough to complete in one context window
- Always update CLAUDE.md with discovered patterns for future iterations
- The completion signal is `RALPH_COMPLETE` (grep'd by the loop)
