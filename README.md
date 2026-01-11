# Chief Wiggum

![Chief Wiggum](ralph.webp)

Chief Wiggum is an autonomous AI agent orchestrator that runs [Claude Code](https://docs.anthropic.com/en/docs/claude-code) with the `/ralph-loop:ralph-loop` skill repeatedly until all PRD items are complete. Each iteration spawns a fresh Claude Code instance with clean context. Memory persists via git history, `progress.txt`, and `prd.json`.

Based on [Geoffrey Huntley's Ralph pattern](https://ghuntley.com/ralph/) and forked from [snarktank/ralph](https://github.com/snarktank/ralph).

[Read my in-depth article on how I use Ralph](https://x.com/ryancarson/status/2008548371712135632)

## Two-Tier Architecture

Chief Wiggum implements a two-tier autonomous execution model:

1. **Chief Wiggum (Outer Loop)**: Orchestrates story execution, tracks progress in `prd.json`, manages state
2. **Ralph Loop (Inner Loop)**: `/ralph-loop:ralph-loop` skill provides iteration support for each story

## Prerequisites

- [Claude Code CLI](https://docs.anthropic.com/en/docs/claude-code) installed and authenticated
- `jq` installed (`brew install jq` on macOS)
- A git repository for your project
- The `/ralph-loop:ralph-loop` skill installed

## Setup

### Option 1: Copy to your project

Copy the Chief Wiggum files into your project:

```bash
# From your project root
mkdir -p scripts/chief-wiggum
cp /path/to/chief-wiggum/chief-wiggum.sh scripts/chief-wiggum/
cp /path/to/chief-wiggum/chief-wiggum.config.json scripts/chief-wiggum/
cp /path/to/chief-wiggum/story-prompt.template.md scripts/chief-wiggum/
chmod +x scripts/chief-wiggum/chief-wiggum.sh
```

### Option 2: Install skills globally

Copy the skills to your Claude Code config for use across all projects:

```bash
cp -r skills/prd ~/.config/claude-code/skills/
cp -r skills/ralph ~/.config/claude-code/skills/
```

## Workflow

### 1. Create a PRD

Use the PRD skill to generate a detailed requirements document:

```
Load the prd skill and create a PRD for [your feature description]
```

Answer the clarifying questions. The skill saves output to `tasks/prd-[feature-name].md`.

### 2. Convert PRD to Chief Wiggum format

Use the Ralph skill to convert the markdown PRD to JSON:

```
Load the ralph skill and convert tasks/prd-[feature-name].md to prd.json
```

This creates `prd.json` with user stories structured for autonomous execution.

### 3. Run Chief Wiggum

```bash
./scripts/chief-wiggum/chief-wiggum.sh [max_stories]
```

Default processes all stories.

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
7. Detect `STORY_COMPLETE` promise and update `prd.json` to mark story as `passes: true`
8. Append learnings to `progress.txt`
9. Repeat until all stories pass or blocked

## Key Files

| File | Purpose |
|------|---------|
| `chief-wiggum.sh` | The bash orchestrator that spawns Claude Code instances |
| `chief-wiggum.config.json` | Configuration for iterations, promises, quality checks |
| `story-prompt.template.md` | Template for generating story prompts |
| `prd.json` | User stories with `passes` status (the task list) |
| `prd.json.example` | Example PRD format for reference |
| `progress.txt` | Append-only learnings for future iterations |
| `skills/prd/` | Skill for generating PRDs |
| `skills/ralph/` | Skill for converting PRDs to JSON |
| `flowchart/` | Interactive visualization of how Chief Wiggum works |

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

## Flowchart

[![Chief Wiggum Flowchart](ralph-flowchart.png)](https://snarktank.github.io/ralph/)

**[View Interactive Flowchart](https://snarktank.github.io/ralph/)** - Click through to see each step with animations.

The `flowchart/` directory contains the source code. To run locally:

```bash
cd flowchart
npm install
npm run dev
```

## Critical Concepts

### Each Story = Fresh Context

Each story spawns a **new Claude Code instance** with clean context via `/ralph-loop:ralph-loop`. The only memory between stories is:
- Git history (commits from previous stories)
- `progress.txt` (learnings and context)
- `prd.json` (which stories are done)

### Small Tasks

Each PRD item should be small enough to complete in one context window. If a task is too big, the LLM runs out of context before finishing and produces poor code.

Right-sized stories:
- Add a database column and migration
- Add a UI component to an existing page
- Update a server action with new logic
- Add a filter dropdown to a list

Too big (split these):
- "Build the entire dashboard"
- "Add authentication"
- "Refactor the API"

### AGENTS.md Updates Are Critical

After each iteration, Chief Wiggum updates the relevant `AGENTS.md` files with learnings. This is key because Claude Code automatically reads these files, so future iterations (and future human developers) benefit from discovered patterns, gotchas, and conventions.

Examples of what to add to AGENTS.md:
- Patterns discovered ("this codebase uses X for Y")
- Gotchas ("do not forget to update Z when changing W")
- Useful context ("the settings panel is in component X")

### Feedback Loops

Chief Wiggum only works if there are feedback loops:
- Typecheck catches type errors
- Tests verify behavior
- CI must stay green (broken code compounds across iterations)

### Browser Verification for UI Stories

Frontend stories must include "Verify in browser" in acceptance criteria. Claude Code will navigate to the page, interact with the UI, and confirm changes work.

### Promise System

Chief Wiggum uses promises to detect story completion:
- `<promise>STORY_COMPLETE</promise>` - Story successfully implemented
- `<promise>BLOCKED</promise>` - Cannot proceed, needs human intervention

When all stories have `passes: true`, Chief Wiggum exits successfully.

## Debugging

Check current state:

```bash
# See which stories are done
cat prd.json | jq '.userStories[] | {id, title, passes}'

# See learnings from previous iterations
cat progress.txt

# Check git history
git log --oneline -10
```

## Customizing story-prompt.template.md

Edit `story-prompt.template.md` to customize Claude Code's behavior for your project:
- Add project-specific quality check commands
- Include codebase conventions
- Add common gotchas for your stack

Available placeholders:
- `{{STORY_ID}}`, `{{STORY_TITLE}}`, `{{STORY_DESCRIPTION}}`
- `{{ACCEPTANCE_CRITERIA}}`
- `{{PROJECT_NAME}}`, `{{BRANCH_NAME}}`, `{{PROJECT_DESCRIPTION}}`
- `{{QUALITY_CHECKS}}`
- `{{COMPLETION_PROMISE}}`, `{{BLOCKED_PROMISE}}`

## Archiving

Chief Wiggum automatically archives previous runs when you start a new feature (different `branchName`). Archives are saved to `archive/YYYY-MM-DD-feature-name/`.

## References

- [Geoffrey Huntley's Ralph article](https://ghuntley.com/ralph/)
- [Claude Code documentation](https://docs.anthropic.com/en/docs/claude-code)
- [Ralph Loop skill](https://github.com/anthropics/claude-code-plugins)
