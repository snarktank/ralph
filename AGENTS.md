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
./scripts/ralph/ralph.sh [max_sessions]

# Run Ralph with Claude Code
./scripts/ralph/ralph.sh --tool claude [max_sessions]
```

## Key Files

- `scripts/ralph/ralph.sh` - The bash loop that spawns fresh AI instances (supports `--tool amp` or `--tool claude`)
- `scripts/ralph/prompt.md` - Instructions given to each AMP instance
- `scripts/ralph/CLAUDE.md` - Instructions given to each Claude Code instance
- `scripts/ralph/prd.json.example` - Example PRD format
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

- Each session spawns a fresh AI instance (Amp or Claude Code) with clean context
- Memory persists via git history, `scripts/ralph/progress.md`, `scripts/ralph/prd.json`, and `scripts/ralph/state.json`
- Stories should be small enough to complete in one context window
- Always update Institutional Memory in scripts/ralph/CLAUDE.md with discovered patterns for future sessions
