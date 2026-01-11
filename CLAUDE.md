# Chief Wiggum - Claude Code Plugin

## Overview

Chief Wiggum is an autonomous PRD executor plugin for Claude Code. It uses the `/ralph-loop` skill to execute user stories from a PRD with iterative completion support. Two-tier architecture:

1. **Chief Wiggum (Outer Loop)**: Orchestrates story execution, tracks progress, manages state
2. **Ralph Loop (Inner Loop)**: Each story executes via `/ralph-loop` with iteration support

## Installation

```bash
# First, install the required ralph-loop plugin
claude plugins install ralph-loop

# Then install chief-wiggum
claude plugins install github:kobozo/chief-wiggum

# Or clone manually
git clone https://github.com/kobozo/chief-wiggum ~/.claude/plugins/chief-wiggum
```

## Architecture

```
/chief-wiggum
    │
    ├── Executes chief-wiggum.sh
    │
    └── For each story in prd.json:
        ├── Renders prompt from story-prompt.template.md
        ├── Spawns: claude --print "/ralph-loop <prompt>"
        ├── Detects STORY_COMPLETE or BLOCKED
        ├── Updates prd.json (passes: true)
        ├── Archives previous runs when branch changes
        └── Continues to next story
```

## Plugin Structure

```
chief-wiggum/
├── .claude-plugin/
│   └── plugin.json              # Plugin manifest
├── commands/
│   └── chief-wiggum.md          # /chief-wiggum command → executes chief-wiggum.sh
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
└── story-prompt.template.md     # Prompt template for stories
```

## User Project Files

These files live in your project directory (not the plugin):

| File | Purpose |
|------|---------|
| `prd.json` | User stories with `passes` status |
| `progress.txt` | Append-only learnings log |
| `archive/` | Previous run archives |

## Usage

```bash
# Execute all stories
/chief-wiggum

# Limit to N stories
/chief-wiggum 5
```

## Commands & Skills

| Command/Skill | Description |
|---------------|-------------|
| `/chief-wiggum` | Execute stories from prd.json via ralph-loop |
| `/chief-wiggum 5` | Execute max 5 stories |
| `/prd` | Generate a PRD document |
| `/chief-wiggum:chief-wiggum` | Convert PRD markdown to prd.json |

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

## Story Lifecycle

1. `/chief-wiggum` executes `chief-wiggum.sh`
2. Script reads `prd.json` from current directory
3. Picks highest priority story where `passes: false`
4. Renders `story-prompt.template.md` with story data
5. Spawns Claude with `/ralph-loop`
6. On `STORY_COMPLETE`: marks story as passed, continues
7. On `BLOCKED`: stops and logs blocker
8. On timeout: logs and continues to next story

## Promise System

- `<promise>STORY_COMPLETE</promise>`: Story implemented and verified (stops loop immediately)
- `<promise>BLOCKED</promise>`: Cannot proceed, needs human intervention

**Note:** The `/ralph-loop` plugin only detects `STORY_COMPLETE` as the completion promise. If Claude outputs `BLOCKED`, the loop continues until `max-iterations`, then Chief Wiggum detects the blocked status.

## Quality Requirements

- All commits must pass configured quality checks
- Never commit broken code
- Keep changes focused and minimal
- Follow existing code patterns

## Memory and Context

Each Claude Code invocation is fresh. Memory persists via:
- Git history (commits from previous stories)
- `progress.txt` (learnings and patterns)
- `prd.json` (story completion status)
- `CLAUDE.md` files (codebase patterns)

## Skills

### PRD Skill (`/prd`)
Generates detailed Product Requirements Documents from feature descriptions.

### Chief Wiggum Skill (`/chief-wiggum:chief-wiggum`)
Converts PRD markdown files to `prd.json` format for Chief Wiggum execution.

## Best Practices

1. **Small Stories**: Each story should complete in one context window
2. **Clear Criteria**: Acceptance criteria must be verifiable
3. **Dependency Order**: Schema -> Backend -> UI
4. **Update CLAUDE.md**: Record reusable patterns
5. **Browser Testing**: UI stories must include browser verification

## Credits

Forked from [snarktank/ralph](https://github.com/snarktank/ralph) - the original autonomous PRD executor for Claude Code.
