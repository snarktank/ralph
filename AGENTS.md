# AGENTS.md

This file provides guidance to Claude Code (claude.ai/code) and other AI coding tools when working with code in this repository.

## Overview

Ralph is an autonomous AI agent loop framework that runs AI coding tools (Amp or Claude Code) repeatedly until all PRD items are complete. Each iteration spawns a fresh instance with clean context. Memory persists via git history, `progress.txt`, and `prd.json`.

**Note**: The `CLAUDE.md` file in this repo is the prompt template piped into Claude Code during Ralph loop execution. This `AGENTS.md` file contains development guidelines.

## Commands

```bash
# Run Ralph with Amp (default)
./ralph.sh [max_iterations]

# Run Ralph with Claude Code
./ralph.sh --tool claude [max_iterations]

# Flowchart development
cd flowchart && npm install   # Install dependencies
cd flowchart && npm run dev   # Start dev server (Vite)
cd flowchart && npm run build # Build (tsc + vite build)
cd flowchart && npm run lint  # Run ESLint

# Debug current Ralph state
cat prd.json | jq '.userStories[] | {id, title, passes}'
cat progress.txt
git log --oneline -10
```

## Architecture

```
ralph/
├── ralph.sh              # Main bash loop - spawns AI instances iteratively
├── CLAUDE.md             # Prompt template for Claude Code iterations
├── prompt.md             # Prompt template for Amp iterations
├── lerty.sh              # Lerty integration (notifications, approvals, Live Activities)
├── lerty-setup.sh        # Setup wizard for Lerty API
├── prd.json              # Runtime task list with passes status
├── prd.json.example      # Example PRD format
├── progress.txt          # Append-only learnings log
├── .lerty/               # Lerty config (gitignored, contains API keys)
├── skills/               # Reusable skills for PRD creation
│   ├── prd/SKILL.md      # Generate PRDs from feature descriptions
│   └── ralph/SKILL.md    # Convert PRDs to prd.json format
└── flowchart/            # Interactive React visualization
    ├── src/App.tsx       # Main React Flow component
    └── vite.config.ts    # Vite configuration
```

## Ralph Loop Mechanics

1. **Setup**: User creates PRD → converts to `prd.json` → runs `ralph.sh`
2. **Loop**: Each iteration picks highest priority story where `passes: false`, implements it, runs quality checks, commits, updates `prd.json` to `passes: true`, logs to `progress.txt`
3. **Exit**: When all stories pass, outputs `<promise>COMPLETE</promise>`

Memory between iterations:
- Git history (commits from previous iterations)
- `progress.txt` (learnings and patterns)
- `prd.json` (story completion status)

## Flowchart Component

The `flowchart/` directory contains an interactive React Flow visualization:
- **React 19 + TypeScript 5.9 + Vite 7**
- **@xyflow/react** for node/edge diagrams
- 10-step walkthrough with Next/Previous navigation
- Nodes are color-coded by phase (setup=blue, loop=gray, decision=yellow, done=green)

## Key Patterns

- Stories must be small enough to complete in one context window
- Each story should have verifiable acceptance criteria including "Typecheck passes"
- UI stories should include browser verification in acceptance criteria
- Commits follow format: `feat: [Story ID] - [Story Title]`
- Learnings go in `progress.txt`, reusable patterns in the Codebase Patterns section at top
- Branch naming: `ralph/[feature-name-kebab-case]`
