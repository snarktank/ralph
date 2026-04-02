# Ralph Agent Instructions

## Overview

Ralph is a Codex-first autonomous agent loop. Each iteration starts a fresh agent, completes at most one story, and writes memory back to files next to `ralph.sh`.

In Codex, the native entry points for this repo are `$prd`, `$ralph`, and `./ralph.sh`.

## Commands

```bash
# Run the flowchart dev server
cd flowchart && npm run dev

# Build the flowchart
cd flowchart && npm run build

# Run Ralph with Codex (default)
./ralph.sh [max_iterations]

# Run non-destructive preflight checks
./ralph.sh --doctor

# Run Ralph with Claude Code
./ralph.sh --tool claude [max_iterations]
```

## Key Files

- `ralph.sh` - The bash loop that spawns fresh AI instances (supports `--tool claude` or `--tool codex`)
- `README.md` - Main usage guide; keep it Codex-first and focused on in-project usage
- `CLAUDE.md` - Instructions given to each Claude Code instance
- `CODEX.md` - Instructions given to each Codex instance
- `.agents/skills/prd/SKILL.md` - Codex-native PRD skill
- `.agents/skills/ralph/SKILL.md` - Codex-native PRD-to-`prd.json` skill
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

- Codex is the default runtime. Keep docs and examples Codex-first; treat Claude as an optional compatibility path.
- Codex-native workflow in this repo is: `$prd` -> `$ralph` -> `./ralph.sh`.
- Repo-local Codex skills live in `.agents/skills/`. Keep those paths as the primary Codex integration surface.
- Ralph state lives next to `ralph.sh`. In this repo that means `./prd.json`, `./progress.txt`, and `./archive/`.
- Non-destructive CLI modes in `ralph.sh` should parse and exit before validating tool binaries, prompt files, or Ralph state. That keeps setup and recovery commands usable on broken local state.
- Diagnostic modes should validate `prd.json` directly, but treat `progress.txt` and `archive/` as lazily created state paths and check readiness instead of requiring them to exist up front.
- `ralph.sh` injects absolute Ralph state paths into the agent prompt, so prompt files should reference runtime context rather than assume the current working directory.
- Ease-of-use work should prioritize copy-paste happy paths, non-destructive diagnostics, and explicit recovery steps over architectural changes.
- When regenerating `prd.json` from a revised PRD, reflect the repo's current baseline and keep only the remaining executable work instead of replaying already-landed stories.
- If Ralph gets a global `ralph` command, keep it as a thin wrapper around repo-local `ralph.sh` so flags, defaults, and docs stay in sync.
- Each iteration spawns a fresh AI instance with clean context
- Memory persists via git history, `progress.txt`, and `prd.json`
- Stories should be small enough to complete in one context window
- Always update AGENTS.md with discovered patterns for future iterations
