# Ralph

![Ralph](ralph.webp)

Ralph is an autonomous AI agent loop that runs AI coding tools ([Amp](https://ampcode.com) or [Claude Code](https://docs.anthropic.com/en/docs/claude-code)) repeatedly until all PRD items are complete. Each session is a fresh instance with clean context. Memory persists via git history, `state.json`, `progress.md`, and `prd.json`.

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
cp /path/to/ralph/scripts/ralph/ralph.sh scripts/ralph/
cp /path/to/ralph/scripts/ralph/BOOTSTRAP.md scripts/ralph/
cp /path/to/ralph/scripts/ralph/COMPACTION.md scripts/ralph/
cp /path/to/ralph/scripts/ralph/PROGRESS-GUIDE.md scripts/ralph/
cp /path/to/ralph/scripts/ralph/ITERATIVE.md scripts/ralph/
cp /path/to/ralph/scripts/ralph/prd.json.example scripts/ralph/

# Copy the prompt template for your AI tool of choice:
cp /path/to/ralph/scripts/ralph/prompt.md scripts/ralph/prompt.md    # For Amp
# OR
cp /path/to/ralph/scripts/ralph/CLAUDE.md scripts/ralph/CLAUDE.md    # For Claude Code

chmod +x scripts/ralph/ralph.sh
```

### Option 2: Install skills globally (Amp)

Copy the skills to your Amp or Claude config for use across all projects:

For AMP
```bash
cp -r skills/ralph ~/.config/amp/skills/
```

For Claude Code (manual)
```bash
cp -r skills/ralph ~/.claude/skills/
```

### Option 3: Use as Claude Code Marketplace

Add the Ralph marketplace to Claude Code:

```bash
/plugin marketplace add snarktank/ralph
```

Then install the skills:

```bash
/plugin install ralph-skills@ralph-marketplace
```

Available skills after installation:
- `/ralph` - Generate PRD markdown + Ralph-compatible `prd.json` in one step

Skills are automatically invoked when you ask Claude to:
- "create a prd", "write prd for", "plan this feature", "spec out"

### Configure Amp auto-handoff (recommended)

Add to `~/.config/amp/settings.json`:

```json
{
  "amp.experimental.autoHandoff": { "context": 90 }
}
```

This enables automatic handoff when context fills up, allowing Ralph to handle large stories that exceed a single context window.

## Terminology

| Term | Definition |
|------|------------|
| **Session** | One top-level loop cycle in `ralph.sh`. Each session is a fresh context window with no memory of prior sessions except what is persisted in the memory layers below. |
| **Iteration** | Repeating the SAME user story across multiple sessions (ITERATIVE feature). One iteration = one session = one context window. Used for batch/multi-pass work with a predetermined count. |

## Workflow

### 1. Create a PRD

Use the `/ralph` skill to generate both artifacts in one step:

```
Load the ralph skill and create a PRD for [your feature description]
```

Answer the clarifying questions. The skill produces two outputs:
- `tasks/prd-[feature-name].md` — human-readable requirements document
- `scripts/ralph/prd.json` — machine-readable story list for Ralph

### 2. Run Ralph

```bash
# Using Amp (default)
./scripts/ralph/ralph.sh [max_sessions]

# Using Claude Code
./scripts/ralph/ralph.sh --tool claude [max_sessions]
```

Default is 10 sessions. Use `--tool amp` or `--tool claude` to select your AI coding tool.

Ralph will:
1. Read `state.json` for current story, branch, and next action
2. Create/checkout the feature branch (from `state.json`)
3. Pick the current story (from `state.json`, NOT by scanning prd.json)
4. Implement that single story (or one iteration of an iterative story)
5. Run quality checks (typecheck, tests)
6. Commit if checks pass
7. Append learnings to `progress.md`
8. Update `prd.json` to mark story as `passes: true`
9. Update `state.json` with next story
10. Repeat until all stories pass or max sessions reached

## Standard vs Iterative Stories

| Mode | When to Use | Behavior |
|------|-------------|----------|
| **Standard** | Most stories -- single-pass work that fits in one session | Story is implemented and marked `passes: true` in one session |
| **Iterative** | Batch/multi-pass work with a predetermined count | Story has `iterationMode` + `maxIterations` in `prd.json`; one iteration per session; `passes: true` only when `currentIteration >= maxIterations` |

Iterative mode is for work that is inherently batched (e.g., audit N sections, process N files), **NOT** for stories that are too large. Stories that are too large should be split into smaller stories.

When a story has `iterationMode`, the agent loads `scripts/ralph/ITERATIVE.md` as a delta-only supplement to `scripts/ralph/CLAUDE.md`. See `scripts/ralph/ITERATIVE.md` for the full rules.

## Memory Model

Ralph persists knowledge across sessions via four layers:

| Layer | File | Purpose | Lifespan |
|-------|------|---------|----------|
| **Operational** | `state.json` | Current story, next action, branch, compaction status | Per-session (updated every session) |
| **Structural** | `prd.json` | Story status, iteration counters, notes | Per-PRD |
| **Experiential** | `progress.md` | Session logs, codebase patterns, learnings | Per-PRD, append-only (auto-compacted) |
| **Institutional** | `scripts/ralph/CLAUDE.md` | Reusable conventions, gotchas, cross-file dependencies | Permanent — survives across PRDs |

See the "Institutional Memory" section in `scripts/ralph/CLAUDE.md` for details on what goes where.

## Key Files

| File | Purpose |
|------|---------|
| `scripts/ralph/ralph.sh` | The bash loop that spawns fresh AI sessions (supports `--tool amp` or `--tool claude`) |
| `scripts/ralph/prompt.md` | Prompt template for Amp |
| `scripts/ralph/CLAUDE.md` | Core agent instructions for Claude Code (includes institutional memory) |
| `scripts/ralph/BOOTSTRAP.md` | State bootstrap for first session (loaded conditionally) |
| `scripts/ralph/COMPACTION.md` | Progress compaction rules (loaded conditionally) |
| `scripts/ralph/PROGRESS-GUIDE.md` | Progress report format and learnings guide (reference) |
| `scripts/ralph/ITERATIVE.md` | Supplemental rules for iterative stories (loaded conditionally) |
| `scripts/ralph/prd.json` | User stories with `passes` status (the task list) |
| `scripts/ralph/prd.json.example` | Example PRD format for reference |
| `scripts/ralph/progress.md` | Append-only learnings for future sessions |
| `scripts/ralph/state.json` | Session orientation file — tells the agent which story, branch, and action to take |
| `skills/ralph/` | Skill for generating PRD markdown + `prd.json` (works with Amp and Claude Code) |
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

### Each Session = Fresh Context

Each session spawns a **new AI instance** (Amp or Claude Code) with clean context. The only memory between sessions is:
- Git history (commits from previous sessions)
- `state.json` (current story orientation and next action)
- `progress.md` (learnings and context)
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

### Institutional Memory

After each session, Ralph updates the institutional memory section in `scripts/ralph/CLAUDE.md` with reusable patterns and gotchas. This is key because AI coding tools automatically read these files, so future sessions (and future human developers) benefit from discovered conventions and gotchas.

Examples of what to persist:
- Patterns discovered ("this codebase uses X for Y")
- Gotchas ("do not forget to update Z when changing W")
- Useful context ("the settings panel is in component X")

### Feedback Loops

Ralph only works if there are feedback loops:
- Typecheck catches type errors
- Tests verify behavior
- CI must stay green (broken code compounds across sessions)

### Browser Verification for UI Stories

Frontend stories must include "Verify in browser using dev-browser skill" in acceptance criteria. Ralph will use the dev-browser skill to navigate to the page, interact with the UI, and confirm changes work.

### Stop Condition

When all stories have `passes: true`, `ralph.sh` detects completion by checking `prd.json` directly and the loop exits.

## Debugging

Check current state:

```bash
# See current state
cat scripts/ralph/state.json | jq .

# See which stories are done
cat scripts/ralph/prd.json | jq '.userStories[] | {id, title, passes}'

# See learnings from previous sessions
cat scripts/ralph/progress.md

# Check git history
git log --oneline -10
```

## Customizing the Prompt

After copying `scripts/ralph/prompt.md` (for Amp) or `scripts/ralph/CLAUDE.md` (for Claude Code) to your project, customize it for your project:
- Add project-specific quality check commands
- Include codebase conventions
- Add common gotchas for your stack

## Archiving

Ralph automatically archives previous runs when you start a new feature (different `branchName`). Archives are saved to `scripts/ralph/archive/YYYY-MM-DD-feature-name/`.

## References

- [Geoffrey Huntley's Ralph article](https://ghuntley.com/ralph/)
- [Amp documentation](https://ampcode.com/manual)
- [Claude Code documentation](https://docs.anthropic.com/en/docs/claude-code)
