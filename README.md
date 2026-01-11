# Chief Wiggum

An autonomous PRD executor plugin for Claude Code. Orchestrates story execution using the `/ralph-loop` skill to iterate until each story is complete.

## Installation

### Via Claude Code Plugin System

```bash
# First, install the required ralph-loop plugin
claude plugins install ralph-loop

# Then install chief-wiggum
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
- The `ralph-loop` plugin installed

## Quick Start

1. **Create a PRD** using the `/prd` skill:
   ```
   /prd create a task management feature
   ```

2. **Convert to prd.json** using the chief-wiggum skill:
   ```
   /chief-wiggum:chief-wiggum convert tasks/prd-task-management.md
   ```

3. **Run Chief Wiggum**:
   ```
   /chief-wiggum
   ```

## Two-Tier Architecture

```
/chief-wiggum
    │
    ├── Executes chief-wiggum.sh
    │
    └── For each story in prd.json:
        ├── Spawns: claude --print "/ralph-loop <prompt>"
        ├── Detects STORY_COMPLETE or BLOCKED
        ├── Updates prd.json (passes: true)
        └── Continues to next story
```

1. **Chief Wiggum (Outer Loop)**: Orchestrates story execution, tracks progress
2. **Ralph Loop (Inner Loop)**: Each story runs with iteration support until complete

## Plugin Structure

```
chief-wiggum/
├── .claude-plugin/
│   └── plugin.json              # Plugin manifest
├── commands/
│   └── chief-wiggum.md          # /chief-wiggum command
├── agents/
│   └── story-executor.md        # Optional agent for story execution
├── skills/
│   ├── prd/
│   │   └── SKILL.md             # PRD generation skill
│   └── chief-wiggum/
│       └── SKILL.md             # PRD-to-JSON converter skill
├── hooks/
│   ├── hooks.json               # Hook configuration
│   └── stop-hook.sh             # Stop event handler
├── chief-wiggum.sh              # Main orchestrator script
├── chief-wiggum.config.json     # Configuration
├── story-prompt.template.md     # Prompt template
├── CLAUDE.md                    # Plugin instructions
└── README.md                    # This file
```

## Commands & Skills

| Command/Skill | Description |
|---------------|-------------|
| `/chief-wiggum` | Execute all stories from prd.json |
| `/chief-wiggum 5` | Execute max 5 stories |
| `/prd` | Generate a PRD document |
| `/chief-wiggum:chief-wiggum` | Convert PRD to prd.json format |

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

1. Read `prd.json` from current directory
2. Pick the highest priority story where `passes: false`
3. Spawn Claude Code with `/ralph-loop`:
   ```bash
   claude --dangerously-skip-permissions --print "/ralph-loop \"<prompt>\" --max-iterations 25 --completion-promise STORY_COMPLETE"
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

**Note:** The `/ralph-loop` plugin only detects `STORY_COMPLETE` as the completion promise. If Claude outputs `BLOCKED`, the loop will continue until `max-iterations` is reached.

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

## Credits

This plugin is forked from [snarktank/ralph](https://github.com/snarktank/ralph), which pioneered the autonomous PRD execution pattern for Claude Code.

## References

- [Claude Code documentation](https://docs.anthropic.com/en/docs/claude-code)
- [Geoffrey Huntley's iteration pattern](https://ghuntley.com/ralph/)
- [snarktank/ralph](https://github.com/snarktank/ralph) - Original implementation
