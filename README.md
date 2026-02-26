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

### Option 2: Install skills globally (Amp)

Copy the skills to your Amp or Claude config for use across all projects:

For AMP
```bash
cp -r skills/prd ~/.config/amp/skills/
cp -r skills/ralph ~/.config/amp/skills/
```

For Claude Code (manual)
```bash
cp -r skills/prd ~/.claude/skills/
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
- `/prd` - Generate Product Requirements Documents
- `/ralph` - Convert PRDs to prd.json format

Skills are automatically invoked when you ask Claude to:
- "create a prd", "write prd for", "plan this feature"
- "convert this prd", "turn into ralph format", "create prd.json"

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
2. Pick the highest priority story where `passes: false`
3. Implement that single story
4. Run quality checks (typecheck, tests)
5. Commit if checks pass
6. Update `prd.json` to mark story as `passes: true`
7. Append learnings to `progress.txt`
8. Repeat until all stories pass or max iterations reached

## Key Files

| File | Purpose |
|------|---------|
| `ralph.sh` | The bash loop that spawns fresh AI instances (supports `--tool amp` or `--tool claude`) |
| `prompt.md` | Prompt template for Amp |
| `CLAUDE.md` | Prompt template for Claude Code |
| `prd.json` | User stories with `passes` status (the task list) |
| `prd.json.example` | Example PRD format for reference |
| `progress.txt` | Append-only learnings for future iterations |
| `skills/prd/` | Skill for generating PRDs (works with Amp and Claude Code) |
| `skills/ralph/` | Skill for converting PRDs to JSON (works with Amp and Claude Code) |
| `.claude-plugin/` | Plugin manifest for Claude Code marketplace discovery |
| `flowchart/` | Interactive visualization of how Ralph works |
| `docker/` | Dockerfile and container scripts for parallel mode |
| `parallel/` | Parallel mode orchestrator, status, and stop scripts |

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
# See which stories are done
cat prd.json | jq '.userStories[] | {id, title, passes}'

# See learnings from previous iterations
cat progress.txt

# Check git history
git log --oneline -10
```

## Customizing the Prompt

After copying `prompt.md` (for Amp) or `CLAUDE.md` (for Claude Code) to your project, customize it for your project:
- Add project-specific quality check commands
- Include codebase conventions
- Add common gotchas for your stack

## Archiving

Ralph automatically archives previous runs when you start a new feature (different `branchName`). Archives are saved to `archive/YYYY-MM-DD-feature-name/`.

## Parallel Mode (Docker)

Ralph includes a parallel mode that runs N containerized Claude Code agents simultaneously against the same PRD. Each agent runs in a Docker container with:

- **Network restrictions** — builder agents can only reach Claude API and npm registry
- **Resource limits** — configurable memory and CPU caps per container
- **Story claiming** — agents claim stories via git atomic push to avoid duplicate work
- **Automatic recovery** — stale claims are cleared, crashed containers are restarted

### Prerequisites (Parallel Mode)

- Docker installed and running
- A Claude Code auth token (env var, file, or 1Password)
- `jq` installed

### Quick Start (Parallel Mode)

```bash
# Set your Claude auth token
export RALPH_CLAUDE_TOKEN='<your-token>'

# Run 3 agents in parallel
./parallel/ralph-parallel.sh --agents 3

# Check status
./parallel/status.sh

# Graceful shutdown
./parallel/stop.sh
```

### Options

```bash
./parallel/ralph-parallel.sh \
  --agents 3 \              # number of builder agents (default: 2)
  --model claude-sonnet-4-5-20250929 \   # model (default: sonnet)
  --memory 4g \             # per-container memory limit
  --cpus 2 \                # per-container CPU limit
  --researcher 1 \          # researcher agents with full internet access
  [max_iterations]           # per-agent iteration cap (default: 0 = until PRD complete)
```

### Auth Token

Priority order (first wins):
1. `RALPH_CLAUDE_TOKEN` environment variable
2. `.ralph/token` file in the project directory
3. 1Password via `op read` (interactive, startup only)

See [parallel/README.md](parallel/README.md) for full documentation.

## Multi-PRP Mode (Docker)

Multi-PRP mode extends parallel mode for running **independent feature branches simultaneously**. Instead of N agents competing for stories on one PRD, each PRP file gets its own feature branch and dedicated agent. This is ideal for batching multiple independent features in a single launch.

### How It Works

1. You provide multiple PRP JSON files, each with its own `branchName`
2. The orchestrator pre-creates a feature branch per PRP in the bare repo
3. One container launches per PRP, targeted to its branch via `RALPH_BRANCH`
4. Agents work independently — no competition, no cross-branch coordination
5. On completion, branches are fetched back and PR creation commands are printed

### Quick Start (Multi-PRP)

```bash
# Launch 6 agents, one per PRP file
./parallel/launch-multi-prp.sh \
  --project /path/to/my-repo \
  --prp prps/prp-03.json \
  --prp prps/prp-04.json \
  --prp prps/prp-05.json \
  --prp prps/prp-06.json \
  --prp prps/prp-07.json \
  --model claude-sonnet-4-5-20250929
```

### Options (Multi-PRP)

```bash
./parallel/launch-multi-prp.sh \
  --project DIR \          # project git repo (required)
  --prp FILE \             # PRP JSON file, relative to project (repeatable, required)
  --model MODEL \          # Claude model (default: claude-sonnet-4-5-20250929)
  --memory SIZE \          # per-container memory (default: 4g)
  --cpus N \               # per-container CPUs (default: 2)
  --max-iterations N \     # per-agent iteration cap (default: 0 = until done)
  --allow-domain DOMAIN    # extra firewall whitelist (repeatable)
```

### Single-PRP vs Multi-PRP

| | `ralph-parallel.sh` | `launch-multi-prp.sh` |
|---|---|---|
| Branches | One shared branch | One branch per PRP |
| Agents | N agents compete for stories | 1 agent per PRP, no competition |
| Use case | Parallelize within a feature | Parallelize across features |
| Story claiming | Git atomic push (contention possible) | No contention (isolated branches) |
| Completion | All stories done → exit | All branches done → exit |
| Output | Stories marked `passes: true` | Branches synced + PR commands printed |

See [parallel/README.md](parallel/README.md) for full documentation.

## References

- [Geoffrey Huntley's Ralph article](https://ghuntley.com/ralph/)
- [Amp documentation](https://ampcode.com/manual)
- [Claude Code documentation](https://docs.anthropic.com/en/docs/claude-code)
