# Ralph

![Ralph](ralph.webp)

Ralph is an autonomous AI agent loop that runs [Claude Code](https://docs.anthropic.com/en/docs/claude-code) repeatedly until all PRD items are complete. Each iteration is a fresh instance with clean context. Memory persists via git history, `progress.txt`, and `prd.json`.

Based on [Geoffrey Huntley's Ralph pattern](https://ghuntley.com/ralph/).

[Read my in-depth article on how I use Ralph](https://x.com/ryancarson/status/2008548371712135632)

## Prerequisites

- [Claude Code](https://docs.anthropic.com/en/docs/claude-code) installed (`npm install -g @anthropic-ai/claude-code`)
- `jq` installed (`brew install jq` on macOS)
- A git repository for your project

## Setup

### Option 1: Use the ralph-init skill (recommended)

If you have the Ralph skills installed, run in your target project:

```
/ralph-init
```

This creates `scripts/ralph/` with all necessary files.

### Option 2: Copy to your project manually

```bash
# From your project root
mkdir -p scripts/ralph
cp /path/to/ralph/ralph.sh scripts/ralph/
cp /path/to/ralph/CLAUDE.md scripts/ralph/
cp /path/to/ralph/prompt.md scripts/ralph/
chmod +x scripts/ralph/ralph.sh
```

### Option 3: Install skills globally

Copy the skills to your Claude config for use across all projects:

```bash
cp -r skills/prd ~/.claude/skills/
cp -r skills/ralph ~/.claude/skills/
cp -r skills/ralph-init ~/.claude/skills/
```

### Option 4: Use as Claude Code Marketplace

Add the Ralph marketplace to Claude Code:

```bash
/plugin marketplace add foxytanuki/ralph
```

Then install the skills:

```bash
/plugin install ralph-skills@ralph-marketplace
```

Available skills after installation:
- `/prd` - Generate Product Requirements Documents
- `/ralph` - Convert PRDs to prd.json format
- `/ralph-init` - Initialize Ralph in a target repository

## Workflow

### 1. Create a PRD

Use the PRD skill to generate a detailed requirements document:

```
/prd [your feature description]
```

Answer the clarifying questions. The skill saves output to `tasks/prd-[feature-name].md`.

### 2. Convert PRD to Ralph format

Use the Ralph skill to convert the markdown PRD to JSON:

```
/ralph convert tasks/prd-[feature-name].md to prd.json
```

This creates `prd.json` with user stories structured for autonomous execution.

### 3. Run Ralph

```bash
./scripts/ralph/ralph.sh [max_iterations]
```

Default is 10 iterations. Tool defaults to `claude`.

With Discord notifications:
```bash
# Via environment variable
DISCORD_WEBHOOK_URL="https://discord.com/api/webhooks/..." ./scripts/ralph/ralph.sh --tool claude

# Or via command line
./scripts/ralph/ralph.sh --tool claude --webhook "https://discord.com/api/webhooks/..."
```

Ralph will:
1. Create a feature branch (from PRD `branchName`)
2. Pick the highest priority story where `passes: false`
3. Implement that single story
4. Run quality checks (typecheck, tests)
5. Commit if checks pass
6. Update `prd.json` to mark story as `passes: true`
7. Append learnings to `progress.txt`
8. Repeat until all stories pass or max iterations reached
9. Send Discord notification on completion or max iterations

## Key Files

| File | Purpose |
|------|---------|
| `ralph.sh` | The bash loop that spawns fresh Claude Code instances |
| `CLAUDE.md` | Prompt template for Claude Code |
| `prd.json` | User stories with `passes` status (the task list) |
| `prd.json.example` | Example PRD format for reference |
| `progress.txt` | Append-only learnings for future iterations |
| `skills/prd/` | Skill for generating PRDs |
| `skills/ralph/` | Skill for converting PRDs to JSON |
| `skills/ralph-init/` | Skill for initializing Ralph in a target repo |
| `.claude-plugin/` | Plugin manifest for Claude Code marketplace discovery |
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

Each iteration spawns a **new Claude Code instance** with clean context. The only memory between iterations is:
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

After each iteration, Ralph updates the relevant `AGENTS.md` files with learnings. This is key because Claude Code automatically reads these files, so future iterations (and future human developers) benefit from discovered patterns, gotchas, and conventions.

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

## Discord Notifications

Ralph can send Discord notifications when:
- **Complete**: All tasks finished successfully (green embed)
- **Max Iterations**: Reached limit without completing (red embed)

Configure via:
- Environment variable: `RALPH_WEBHOOK_URL` (recommended for global config in `.bashrc`/`.zshrc`)
- Environment variable: `DISCORD_WEBHOOK_URL` (fallback)
- Command line: `--webhook URL`

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

## Customizing the Prompt

After copying `CLAUDE.md` to your project, customize it for your project:
- Add project-specific quality check commands
- Include codebase conventions
- Add common gotchas for your stack

## Archiving

Ralph automatically archives previous runs when you start a new feature (different `branchName`). Archives are saved to `archive/YYYY-MM-DD-feature-name/`.

## References

- [Geoffrey Huntley's Ralph article](https://ghuntley.com/ralph/)
- [Claude Code documentation](https://docs.anthropic.com/en/docs/claude-code)
