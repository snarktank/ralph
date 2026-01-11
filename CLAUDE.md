# Ralph - Claude Code Configuration

This repository has been adapted to work with Claude Code (previously designed for Amp CLI).

## Quick Start

```bash
# Run Ralph autonomous loop
./ralph.sh [max_iterations]

# Install slash commands globally
mkdir -p ~/.claude/commands
cp commands/prd.md ~/.claude/commands/
cp commands/ralph.md ~/.claude/commands/
```

## Slash Commands

- `/prd [feature description]` - Generate a Product Requirements Document
- `/ralph [prd file]` - Convert a PRD to prd.json for autonomous execution

## Key Files

| File | Purpose |
|------|---------|
| `ralph.sh` | Main loop - runs `claude --dangerously-skip-permissions` per iteration |
| `prompt.md` | Instructions given to each Claude instance |
| `prd.json` | Task list with user stories (generated) |
| `progress.txt` | Append-only learnings log (generated) |
| `commands/prd.md` | /prd slash command |
| `commands/ralph.md` | /ralph slash command |

## Workflow

1. Create PRD: `/prd Create a PRD for [feature]`
2. Convert to JSON: `/ralph Convert tasks/prd-[feature].md to prd.json`
3. Run loop: `./ralph.sh`

## How Ralph Works

1. Each iteration spawns a fresh Claude Code instance
2. Claude picks the next story where `passes: false`
3. Implements it, runs quality checks, commits if passing
4. Updates `prd.json` to mark story complete
5. Logs learnings to `progress.txt`
6. Repeats until all stories pass or max iterations reached

## Memory Between Iterations

- **Git history** - code changes persist
- **progress.txt** - learnings persist
- **prd.json** - task status persists
- **AGENTS.md** - patterns discovered persist

## Repository

- **Origin**: https://github.com/sudpoy/ralph (your fork)
- **Upstream**: https://github.com/snarktank/ralph (original)
