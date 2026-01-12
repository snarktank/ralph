# Ralph

![Ralph](ralph.webp)

Ralph is an autonomous AI agent loop that runs an AI worker (default: [Amp](https://ampcode.com), optional: Cursor CLI) repeatedly until all PRD items are complete. Each iteration is a fresh worker invocation with clean context. Memory persists via git history, `scripts/ralph/progress.txt`, and `scripts/ralph/prd.json`.

Based on [Geoffrey Huntley's Ralph pattern](https://ghuntley.com/ralph/).

[Read my in-depth article on how I use Ralph](https://x.com/ryancarson/status/2008548371712135632)

## Prerequisites

- One worker installed:
  - [Amp CLI](https://ampcode.com) installed and authenticated, and/or
  - Cursor CLI (`cursor`) installed and authenticated
- `jq` installed (`brew install jq` on macOS)
- A git repository for your project

## Setup

### Option 1: Copy to your project

Copy the Ralph templates into your project:

```bash
# From your project root
mkdir -p scripts/ralph
cp -R /path/to/ralph/scripts/ralph/* scripts/ralph/
chmod +x scripts/ralph/ralph.sh
chmod +x scripts/ralph/cursor/convert-to-prd-json.sh
```

### Option 2: Install skills globally

Copy the skills to your Amp config for use across all projects:

```bash
cp -r skills/prd ~/.config/amp/skills/
cp -r skills/ralph ~/.config/amp/skills/
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

If you use Amp skills, use the PRD skill to generate a detailed requirements document:

```
Load the prd skill and create a PRD for [your feature description]
```

Answer the clarifying questions. The skill saves output to `tasks/prd-[feature-name].md`.

If you use Cursor in the IDE, you can also generate a PRD using the repo's Cursor rules (see `.cursor/rules/`).

### 2. Convert PRD to Ralph format

If you use Amp skills, use the Ralph skill to convert the markdown PRD to JSON:

```
Load the ralph skill and convert tasks/prd-[feature-name].md to prd.json
```

Alternatively, you can convert PRD markdown to `scripts/ralph/prd.json` using the Cursor helper script:

```bash
./scripts/ralph/cursor/convert-to-prd-json.sh tasks/prd-[feature-name].md
```

This creates `scripts/ralph/prd.json` with user stories structured for autonomous execution.

### 3. Run Ralph

```bash
./scripts/ralph/ralph.sh [max_iterations] [--worker amp|cursor] [--cursor-timeout SECONDS]
```

Default is 10 iterations.

The runner loop will invoke the selected worker repeatedly. The worker prompt instructs it to:
- Read `scripts/ralph/prd.json` and `scripts/ralph/progress.txt`
- Implement one story per iteration, run checks, commit, and update `passes: true`
- Stop by outputting `<promise>COMPLETE</promise>` when all stories pass

Examples:

```bash
# Default worker is Amp
./scripts/ralph/ralph.sh 10

# Run with Cursor CLI (with a per-iteration timeout)
./scripts/ralph/ralph.sh 10 --worker cursor --cursor-timeout 1800
```

Note: `--cursor-timeout` only applies if a `timeout` binary is available on your PATH. If it isn't, Ralph will run Cursor without a hard timeout.

## Key Files

| File | Purpose |
|------|---------|
| `scripts/ralph/ralph.sh` | The bash loop that spawns fresh worker invocations |
| `scripts/ralph/amp/prompt.md` | Instructions given to each Amp iteration |
| `scripts/ralph/cursor/prompt.cursor.md` | Instructions given to each Cursor iteration |
| `scripts/ralph/cursor/convert-to-prd-json.sh` | Convert PRD markdown → `scripts/ralph/prd.json` via Cursor CLI |
| `scripts/ralph/prd.json` | User stories with `passes` status (the task list) |
| `scripts/ralph/prd.json.example` | Example PRD format for reference |
| `scripts/ralph/progress.txt` | Append-only learnings for future iterations |
| `skills/prd/` | Skill for generating PRDs |
| `skills/ralph/` | Skill for converting PRDs to JSON |
| `flowchart/` | Interactive visualization of how Ralph works |

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

Each iteration spawns a **new worker invocation** (Amp or Cursor) with clean context. The only memory between iterations is:
- Git history (commits from previous iterations)
- `scripts/ralph/progress.txt` (learnings and context)
- `scripts/ralph/prd.json` (which stories are done)

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

After each iteration, Ralph updates the relevant `AGENTS.md` files with learnings. This is key because Amp automatically reads these files, so future iterations (and future human developers) benefit from discovered patterns, gotchas, and conventions.

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
# See which stories are done
cat scripts/ralph/prd.json | jq '.userStories[] | {id, title, passes}'

# See learnings from previous iterations
cat scripts/ralph/progress.txt

# Check git history
git log --oneline -10
```

## Customizing prompts

Edit the worker prompt(s) to customize Ralph's behavior for your project:
- Add project-specific quality check commands
- Include codebase conventions
- Add common gotchas for your stack

Worker prompt locations:
- Amp: `scripts/ralph/amp/prompt.md`
- Cursor: `scripts/ralph/cursor/prompt.cursor.md`

## Archiving

Ralph automatically archives previous runs when you start a new feature (different `branchName`). Archives are saved to `scripts/ralph/archive/YYYY-MM-DD-feature-name/`.

## References

- [Geoffrey Huntley's Ralph article](https://ghuntley.com/ralph/)
- [Amp documentation](https://ampcode.com/manual)
