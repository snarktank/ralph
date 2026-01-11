# Chief Wiggum

An autonomous PRD executor plugin for Claude Code. Orchestrates story execution using the `/ralph-loop:ralph-loop` skill to iterate until each story is complete.

## Installation

### Via Claude Code Plugin System

```bash
claude plugins install github:kobozo/chief-wiggum
```

### Manual Installation

```bash
git clone https://github.com/kobozo/chief-wiggum ~/.claude/plugins/chief-wiggum
```

## Prerequisites

- [Claude Code CLI](https://docs.anthropic.com/en/docs/claude-code) installed and authenticated
- `jq` installed (`brew install jq` on macOS)
- A git repository for your project
- The `/ralph-loop:ralph-loop` skill installed

## Quick Start

1. **Create a PRD** using the `/prd` skill:
   ```
   /prd create a task management feature
   ```

2. **Convert to prd.json** using the `/chief-wiggum` skill:
   ```
   /chief-wiggum convert tasks/prd-task-management.md
   ```

3. **Run Chief Wiggum**:
   ```bash
   /chief-wiggum
   # or directly:
   ~/.claude/plugins/chief-wiggum/commands/chief-wiggum.sh
   ```

## Two-Tier Architecture

```
/chief-wiggum (Outer Orchestrator)
    |
    +-- Reads prd.json from current directory
    +-- Picks highest priority story where passes: false
    +-- Spawns Claude Code with /ralph-loop:ralph-loop
    |
    +-- Detects STORY_COMPLETE or BLOCKED promises
    +-- Updates prd.json (marks passes: true)
    +-- Repeats until all stories complete
```

1. **Chief Wiggum (Outer Loop)**: Orchestrates story execution, tracks progress
2. **Inner Loop**: Each story runs via `/ralph-loop:ralph-loop` with iteration support

## Plugin Structure

```
chief-wiggum/
├── plugin.json                 # Plugin manifest
├── commands/
│   └── chief-wiggum.sh        # Main orchestrator
├── hooks/
│   └── stop-hook.sh           # Optional stop hook
├── skills/
│   ├── prd/SKILL.md           # PRD generation skill
│   └── chief-wiggum/SKILL.md  # PRD-to-JSON converter skill
├── chief-wiggum.config.json   # Configuration
├── story-prompt.template.md   # Prompt template
├── CLAUDE.md                  # Plugin instructions
└── README.md                  # This file
```

## User Project Files

These files are created in your project directory:

| File | Purpose |
|------|---------|
| `prd.json` | User stories with `passes` status |
| `progress.txt` | Append-only learnings log |
| `archive/` | Previous run archives |

## Configuration

Edit `chief-wiggum.config.json`:

```json
{
  "maxIterationsPerStory": 25,
  "completionPromise": "STORY_COMPLETE",
  "blockedPromise": "BLOCKED",
  "qualityChecks": [
    {"name": "typecheck", "command": "npm run typecheck"},
    {"name": "lint", "command": "npm run lint"},
    {"name": "test", "command": "npm run test"}
  ]
}
```

## Workflow

Chief Wiggum will:

1. Create a feature branch (from PRD `branchName`)
2. Pick the highest priority story where `passes: false`
3. Spawn Claude Code with `/ralph-loop:ralph-loop`:
   ```bash
   claude --dangerously-skip-permissions --print "/ralph-loop:ralph-loop \"<prompt>\" --max-iterations 25 --completion-promise STORY_COMPLETE"
   ```
4. Implement that single story with iteration support
5. Run quality checks (typecheck, tests)
6. Commit if checks pass
7. Detect `STORY_COMPLETE` promise and update `prd.json`
8. Append learnings to `progress.txt`
9. Repeat until all stories pass or blocked

## Critical Concepts

### Each Story = Fresh Context

Each story spawns a **new Claude Code instance** with clean context. Memory persists via:
- Git history (commits from previous stories)
- `progress.txt` (learnings and context)
- `prd.json` (which stories are done)

### Small Tasks

Each story must be completable in one context window. Right-sized:
- Add a database column and migration
- Add a UI component to an existing page
- Update a server action with new logic

Too big (split these):
- "Build the entire dashboard"
- "Add authentication"
- "Refactor the API"

### Promise System

- `<promise>STORY_COMPLETE</promise>` - Story successfully implemented
- `<promise>BLOCKED</promise>` - Cannot proceed, needs human intervention

### Browser Verification

UI stories must include "Verify in browser" in acceptance criteria.

## Debugging

```bash
# See which stories are done
cat prd.json | jq '.userStories[] | {id, title, passes}'

# See learnings from previous iterations
cat progress.txt

# Check git history
git log --oneline -10
```

## Customizing story-prompt.template.md

Available placeholders:
- `{{STORY_ID}}`, `{{STORY_TITLE}}`, `{{STORY_DESCRIPTION}}`
- `{{ACCEPTANCE_CRITERIA}}`
- `{{PROJECT_NAME}}`, `{{BRANCH_NAME}}`, `{{PROJECT_DESCRIPTION}}`
- `{{QUALITY_CHECKS}}`
- `{{COMPLETION_PROMISE}}`, `{{BLOCKED_PROMISE}}`

## Archiving

Chief Wiggum automatically archives previous runs when you start a new feature (different `branchName`). Archives are saved to `archive/YYYY-MM-DD-feature-name/`.

## References

- [Claude Code documentation](https://docs.anthropic.com/en/docs/claude-code)
- [Geoffrey Huntley's iteration pattern](https://ghuntley.com/ralph/)
