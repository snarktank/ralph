# Ralph Agent Instructions

## Overview

Ralph is an autonomous AI agent loop that runs AI coding tools (Amp or Claude Code) repeatedly until all PRD items are complete. Each iteration is a fresh instance with clean context.

## Commands

```bash
# Run the flowchart dev server
cd flowchart && npm run dev

# Build the flowchart
cd flowchart && npm run build

# Run Ralph with Amp (default)
./ralph.sh [max_iterations]

# Run Ralph with Claude Code
./ralph.sh --tool claude [max_iterations]
```

## Key Files

- `ralph.sh` - The bash loop that spawns fresh AI instances (supports `--tool amp` or `--tool claude`)
- `prompt.md` - Instructions given to each AMP instance
-  `CLAUDE.md` - Instructions given to each Claude Code instance
- `prd.json.example` - Example PRD format
- `flowchart/` - Interactive React Flow diagram explaining how Ralph works

## Flowchart

The `flowchart/` directory contains an interactive visualization built with React Flow. It's designed for presentations - click through to reveal each step with animations.

To run locally:
```bash
cd flowchart
npm install
npm run dev
```

## Patterns

- Each iteration spawns a fresh AI instance (Amp or Claude Code) with clean context
- Memory persists via git history, `progress.txt`, and `prd.json`
- Stories should be small enough to complete in one context window
- Always update AGENTS.md with discovered patterns for future iterations

## Parallel Mode

Ralph supports running multiple agents in parallel via Docker containers. See `parallel/README.md` for details.

- Parallel scripts live in `parallel/` — orchestrator, status, stop
- Docker image and container entrypoint live in `docker/`
- Agents claim stories via `claimed_by` field in prd.json using git atomic push
- Each agent writes to its own `progress-<agent-id>.txt` to avoid merge conflicts
- Builder agents have restricted network access (Claude API + npm only)
- Researcher agents have full internet access

### Multi-PRP Mode

`launch-multi-prp.sh` runs independent feature branches simultaneously (1 agent per PRP):

- Each PRP file specifies its own `branchName`; the orchestrator pre-creates branches
- `RALPH_BRANCH` env var tells each agent which branch to check out
- No story contention — each agent has its own story pool
- On completion, branches are fetched back and `gh pr create` commands are printed
- Use `--prp FILE` (repeatable) to specify which PRPs to run

### Lessons Learned

- **macOS bash 3.x**: Don't use `declare -A` (associative arrays). Use indexed parallel arrays.
- **Agent containers are independent**: The monitor loop crashing doesn't affect running agents.
- **Foundation stories take longer**: Types + API client stories are slower than incremental tool stories.
- **Token expiry planning**: All agents share one OAuth token. If it expires mid-run, agents that haven't finished enter a retry loop. Plan token validity for the full session.
- **PRP independence**: When two PRPs need the same helper, inline it in both as separate stories with "skip if already exists" guidance.
