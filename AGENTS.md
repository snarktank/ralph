# Ralph Agent Instructions

## Overview

Ralph is an autonomous AI agent loop that runs AI coding tools (Amp, Claude Code, or OpenCode) repeatedly until all PRD items are complete. Each iteration is a fresh instance with clean context.

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

# Run Ralph with OpenCode
./ralph.sh --tool opencode [max_iterations]
```

## Key Files

- `ralph.sh` - The bash loop that spawns fresh AI instances (supports `--tool amp`, `--tool claude`, or `--tool opencode`). The agent prompt is embedded directly in this script.
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

- Each iteration spawns a fresh AI instance (Amp, Claude Code, or OpenCode) with clean context
- Memory persists via git history, `progress.txt`, and `prd.json`
- Stories should be small enough to complete in one context window
- Always update AGENTS.md with discovered patterns for future iterations
- **Early exit optimization**: The loop checks if `prd.json` exists at the start of each iteration. When the agent completes all stories, it removes `prd.json` as part of cleanup, so subsequent iterations exit immediately without invoking the AI tool (saves tokens)
- **Failure detection**: Iterations completing in less than 4 seconds are flagged as potential failures. After 5 consecutive quick failures, Ralph exits with an error to prevent rate limiting and catch configuration issues (e.g., invalid model names)
- **Default iterations**: Set to 50 (up from 10) since early exit optimization prevents wasted iterations when work is complete
