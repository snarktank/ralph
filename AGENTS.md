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

## Control Layer (this fork)

A trust-and-control layer sits **beside** the loop (`ralph.sh` is never
modified). All scripts are standalone stdlib-Python, run via the `ralph`
dispatcher or direct aliases.

```bash
./ralph audit              # morning rundown (project_audit.py)
./ralph cost [--all]       # spend analysis + forecasts (ralph_cost.py)
./ralph schedule "0 2 * * *"   # cron the loop;  --off to remove (ralph_schedule.py)
./ralph run --tool claude 10   # tagged loop run + dated rundown (ralph-run.sh)
python3 -m unittest discover -s tests   # control-layer tests
```

- `prd.json` uses the `{meta, items}` envelope. Items carry three optional
  fields — `attempts` (int), `blocked` (bool), `blockReason` (str) — absent ⇒
  `0 / false / ""`. `meta.maxAttempts` is K (default 3). Semantics live in
  `ralph_prd.py` (the shared loader; import it, don't re-parse prd.json).
- **Skip-then-block:** the agent increments `attempts` each iteration and
  self-blocks at K (rules in `CLAUDE.md`). Never select a `blocked:true` item
  while an eligible (`passes:false & !blocked`) item remains.
- **Cost capture:** SessionEnd hook (`.claude/settings.json` → `cost_hook.py`)
  appends one atomic line per session to `cost.jsonl`. `RALPH_RUN_ID` (set by
  `ralph-run.sh`) tags loop spend; untagged lines = ad-hoc/co-pilot sessions.
- **`/add-feature`** appends items to `prd.json` + `features.md` without altering
  existing items.

When changing data-model semantics, update `ralph_prd.py`, the `CLAUDE.md` schema
note, and `prd.json.example` together.
