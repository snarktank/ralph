# Ralph Agent Instructions

## Overview

Ralph is an autonomous AI agent loop that runs AI coding tools (Amp or Claude Code) repeatedly until all PRD items are complete. Each session is a fresh instance with clean context—**sessions do not share memory with each other**.

## Commands

```bash
# Run the flowchart dev server
cd flowchart && npm run dev

# Build the flowchart
cd flowchart && npm run build

# Run Ralph with Amp (default)
./ralph.sh [max_sessions]

# Run Ralph with Claude Code
./ralph.sh --tool claude [max_sessions]
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

- Each session spawns a fresh AI instance (Amp or Claude Code) with clean context
- Sessions do NOT share memory—all persistent knowledge must be written to files (git history, `progress.txt`, `prd.json`, and `AGENTS.md`)
- Stories should be small enough to complete in one context window
- Always update AGENTS.md with discovered patterns so future sessions can benefit (they cannot read your memory, only files)
