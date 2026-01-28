# Ralph

![Ralph](ralph.webp)

Ralph is an autonomous AI agent loop that runs AI coding tools ([Amp](https://ampcode.com) or [Claude Code](https://docs.anthropic.com/en/docs/claude-code)) repeatedly until all PRD items are complete. Each iteration is a fresh instance with clean context. Memory persists via git history, `progress.txt`, and `prd.json`.

Based on [Geoffrey Huntley's Ralph pattern](https://ghuntley.com/ralph/).

[Read my in-depth article on how I use Ralph](https://x.com/ryancarson/status/2008548371712135632)

## Prerequisites

- One of the following AI coding tools installed and authenticated:
  - [Amp CLI](https://ampcode.com) (default)
  - [Claude Code](https://docs.anthropic.com/en/docs/claude-code) (`npm install -g @anthropic-ai/claude-code`)
- `jq` installed (`brew install jq` on macOS)
- A git repository for your project

## Setup

### Option 1: Copy to your project

Copy the ralph files into your project:

```bash
# From your project root
mkdir -p scripts/ralph
cp /path/to/ralph/ralph.sh scripts/ralph/

# Copy the prompt template for your AI tool of choice:
cp /path/to/ralph/prompt.md scripts/ralph/prompt.md    # For Amp
# OR
cp /path/to/ralph/CLAUDE.md scripts/ralph/CLAUDE.md    # For Claude Code

chmod +x scripts/ralph/ralph.sh
```

### Option 2: Install skills globally

Copy the skills to your Amp or Claude config for use across all projects:

For AMP
```bash
cp -r skills/prd ~/.config/amp/skills/
cp -r skills/ralph ~/.config/amp/skills/
```

For Claude Code
```bash
cp -r skills/prd ~/.claude/skills/
cp -r skills/ralph ~/.claude/skills/
```

### Configure Amp auto-handoff (recommended)

Add to `~/.config/amp/settings.json`:

```json
{
  "amp.experimental.autoHandoff": { "context": 90 }
}
```

This enables automatic handoff when context fills up, allowing Ralph to handle large stories that exceed a single context window.

## Workflow

### 1. Create a PRD

Use the PRD skill to generate a detailed requirements document:

```
Load the prd skill and create a PRD for [your feature description]
```

Answer the clarifying questions. The skill saves output to `tasks/prd-[feature-name].md`.

### 2. Convert PRD to Ralph format

Use the Ralph skill to convert the markdown PRD to JSON:

```
Load the ralph skill and convert tasks/prd-[feature-name].md to prd.json
```

This creates `prd.json` with user stories structured for autonomous execution.

### 3. Run Ralph

```bash
# Using Amp (default)
./scripts/ralph/ralph.sh [max_iterations]

# Using Claude Code
./scripts/ralph/ralph.sh --tool claude [max_iterations]
```

Default is 10 iterations. Use `--tool amp` or `--tool claude` to select your AI coding tool.

Ralph will:
1. Create a feature branch (from PRD `branchName`)
2. **[Claude Code]** Convert PRD to hierarchical tasks (parent stories + child criteria)
3. Pick the next pending task (respecting dependencies)
4. Implement that single task (one acceptance criterion)
5. Run quality checks (typecheck, tests, browser verification)
6. Commit if checks pass
7. Mark task complete; update `prd.json` when story fully complete
8. Append learnings to `progress.txt`
9. Repeat until all tasks complete or max iterations reached

**Task System Benefits (Claude Code only):**
- Granular progress tracking (per acceptance criterion, not just per story)
- Automatic dependency management (schema → backend → UI)
- Shared task list across iterations via `CLAUDE_CODE_TASK_LIST_ID`
- Better visibility into what's done vs pending

## Key Files

| File | Purpose |
|------|---------|
| `ralph.sh` | The bash loop that spawns fresh AI instances (supports `--tool amp` or `--tool claude`) |
| `prompt.md` | Prompt template for Amp |
| `CLAUDE.md` | Prompt template for Claude Code |
| `prd.json` | User stories with `passes` status (the task list) |
| `prd.json.example` | Example PRD format for reference |
| `progress.txt` | Append-only learnings for future iterations |
| `skills/prd/` | Skill for generating PRDs |
| `skills/ralph/` | Skill for converting PRDs to JSON (+ tasks) |
| `flowchart/` | Interactive visualization of how Ralph works |
| `lib/task-converter.js` | **[New]** Converts PRD to hierarchical Claude Code tasks |
| `lib/task-utils.sh` | **[New]** Bash utilities for task system initialization |
| `scripts/prd-to-tasks.js` | **[New]** CLI tool to generate tasks from prd.json |

## Task System Integration (Claude Code)

Ralph now integrates with Claude Code's built-in task management system to provide hierarchical task tracking and dependency management.

### How It Works

**Hierarchical Tasks:**
- Each user story → **Parent task** (e.g., `[US-001] Add status field to database`)
- Each acceptance criterion → **Child task** (e.g., `[US-001-AC1] Add status column with migration`)
- Child tasks are completed sequentially within a story
- Parent task completes when all children complete

**Smart Dependencies:**
The system auto-detects dependencies based on keywords:
- **Schema stories** (`database`, `migration`, `table`) → No dependencies (foundational)
- **Backend stories** (`API`, `endpoint`, `server action`) → Depend on schema stories
- **UI stories** (`component`, `page`, `form`) → Depend on backend + schema stories

**Shared Task List:**
All Ralph iterations work on the same task list via `CLAUDE_CODE_TASK_LIST_ID` environment variable. Task progress persists across sessions.

### Using Tasks

Tasks are automatically created when you run `./ralph.sh --tool claude`:

```bash
# Run Ralph with Claude Code (tasks auto-created)
./ralph.sh --tool claude 10

# View current tasks
claude task list

# Manual task generation (if needed)
node scripts/prd-to-tasks.js prd.json
```

**Task List ID:**
Generated from project name + branch name hash. Stored in `.ralph-task-list-id` file.

### When Tasks Are Used

- **Claude Code**: Tasks used automatically (fallback to prd.json if unavailable)
- **Amp**: prd.json-only mode (tasks not supported)
- **Fallback**: If `CLAUDE_CODE_ENABLE_TASKS=false` or Node.js not installed

### Example Task Structure

```
Story US-001 (Parent Task)
├── [US-001-AC1] Add status column to database (Child Task)
├── [US-001-AC2] Generate and run migration (Child Task - blocked by AC1)
└── [US-001-AC3] Typecheck passes (Child Task - blocked by AC1, AC2)

Story US-002 (Parent Task - blocked by US-001)
├── [US-002-AC1] Display status badge on cards (Child Task)
├── [US-002-AC2] Badge colors match design (Child Task - blocked by AC1)
└── [US-002-AC3] Browser verification (Child Task - blocked by AC1, AC2)
```

### Manual Dependency Override

If auto-detection is insufficient, add `dependencies` field to prd.json:

```json
{
  "id": "US-003",
  "title": "Feature that depends on US-001 and US-002",
  "dependencies": ["US-001", "US-002"],
  "acceptanceCriteria": [...]
}
```

## Flowchart

[![Ralph Flowchart](ralph-flowchart.png)](https://snarktank.github.io/ralph/)

**[View Interactive Flowchart](https://snarktank.github.io/ralph/)** - Click through to see each step with animations.

The `flowchart/` directory contains the source code. To run locally:

```bash
cd flowchart
npm install
npm run dev
```

## Critical Concepts

### Each Iteration = Fresh Context

Each iteration spawns a **new AI instance** (Amp or Claude Code) with clean context. The only memory between iterations is:
- Git history (commits from previous iterations)
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

After each iteration, Ralph updates the relevant `AGENTS.md` files with learnings. This is key because AI coding tools automatically read these files, so future iterations (and future human developers) benefit from discovered patterns, gotchas, and conventions.

Examples of what to add to AGENTS.md:
- Patterns discovered ("this codebase uses X for Y")
- Gotchas ("do not forget to update Z when changing W")
- Useful context ("the settings panel is in component X")

### Feedback Loops

Ralph only works if there are feedback loops:
- Typecheck catches type errors
- Tests verify behavior
- CI must stay green (broken code compounds across iterations)

### Browser Verification for UI Stories

Frontend stories must include "Verify in browser using dev-browser skill" in acceptance criteria. Ralph will use the dev-browser skill to navigate to the page, interact with the UI, and confirm changes work.

### Stop Condition

When all stories have `passes: true`, Ralph outputs `<promise>COMPLETE</promise>` and the loop exits.

## Debugging

Check current state:

```bash
# See which stories are done (prd.json)
cat prd.json | jq '.userStories[] | {id, title, passes}'

# See task status (Claude Code)
claude task list

# See detailed task info
claude task list --json | jq '.[] | select(.metadata.type == "parent")'

# See which tasks are blocked
claude task list --json | jq '.[] | select(.blockedBy | length > 0)'

# Check task list ID
cat .ralph-task-list-id

# See learnings from previous iterations
cat progress.txt

# Check git history
git log --online -10

# Check current branch
git branch
```

### Task Troubleshooting

**Tasks not being created:**
- Check Node.js installed: `node --version`
- Check Claude Code installed: `claude --version`
- Check task system enabled: `echo $CLAUDE_CODE_ENABLE_TASKS`
- Manually create: `node scripts/prd-to-tasks.js prd.json`

**Task/PRD out of sync:**
- Tasks are source of truth for execution
- prd.json updated when parent task completes
- To reset: delete `~/.claude/tasks/[task-list-id]` and re-run conversion

**Circular dependency error:**
- Check prd.json story ordering (dependencies → dependents)
- Add manual `dependencies` field to override auto-detection
- Ensure no story depends on a later-priority story

**Wrong task list:**
- Check `.ralph-task-list-id` matches expected ID
- Delete `.ralph-task-list-id` to regenerate
- Manually set: `export CLAUDE_CODE_TASK_LIST_ID=your-id`

## Customizing the Prompt

After copying `prompt.md` (for Amp) or `CLAUDE.md` (for Claude Code) to your project, customize it for your project:
- Add project-specific quality check commands
- Include codebase conventions
- Add common gotchas for your stack

## Archiving

Ralph automatically archives previous runs when you start a new feature (different `branchName`). Archives are saved to `archive/YYYY-MM-DD-feature-name/`.

## References

- [Geoffrey Huntley's Ralph article](https://ghuntley.com/ralph/)
- [Amp documentation](https://ampcode.com/manual)
- [Claude Code documentation](https://docs.anthropic.com/en/docs/claude-code)
