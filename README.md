# Ralph

![Ralph](ralph.webp)

Ralph is an autonomous AI agent loop that runs AI coding tools ([Amp](https://ampcode.com) by default) repeatedly until all PRD items are complete. Each iteration is a fresh instance of the agent with clean context to prevent context rot. Memory persists via git history, `progress.txt`, and `prd.json`.

Based on [Geoffrey Huntley's Ralph pattern](https://ghuntley.com/ralph/).

[Read my in-depth article on how I use Ralph](https://x.com/ryancarson/status/2008548371712135632)

## Prerequisites

- One of the following AI coding tools installed and authenticated:
  - [Amp CLI](https://ampcode.com) (default)
  - [Claude Code](https://docs.anthropic.com/en/docs/claude-code)
  - [OpenCode](https://github.com/AmruthPillai/OpenCode)
- Common shell utilities:
  - `jq` for JSON manipulation (`brew install jq` on macOS, `apt-get install jq` on Ubuntu)
  - `sponge` from moreutils for in-place file updates (`brew install moreutils` on macOS, `apt-get install moreutils` on Ubuntu)
- A git repository for your project

## Setup

### Option 1: Install globally (Recommended)

Download and install ralph.sh to your PATH:

```bash
# Download and install ralph.sh to your PATH
curl -o ~/.local/bin/ralph.sh https://raw.githubusercontent.com/snarktank/ralph/main/ralph.sh
chmod +x ~/.local/bin/ralph.sh

# Ensure ~/.local/bin is in PATH (add to ~/.bashrc, ~/.zshrc, or your shell's config)
export PATH="$HOME/.local/bin:$PATH"
```

### Option 2: Install skills

Copy the skills to your Amp or Claude config for use across all projects:

**For Amp:**
```bash
# From local clone
cp -r skills/prd ~/.config/amp/skills/
cp -r skills/ralph ~/.config/amp/skills/

# Or via curl (no clone needed)
mkdir -p ~/.config/amp/skills/{prd,ralph}
curl -o ~/.config/amp/skills/prd/SKILL.md https://raw.githubusercontent.com/snarktank/ralph/main/skills/prd/SKILL.md
curl -o ~/.config/amp/skills/ralph/SKILL.md https://raw.githubusercontent.com/snarktank/ralph/main/skills/ralph/SKILL.md
```

**For Claude Code:**
```bash
# From local clone
cp -r skills/prd ~/.claude/skills/
cp -r skills/ralph ~/.claude/skills/

# Or via curl (no clone needed)
mkdir -p ~/.claude/skills/{prd,ralph}
curl -o ~/.claude/skills/prd/SKILL.md https://raw.githubusercontent.com/snarktank/ralph/main/skills/prd/SKILL.md
curl -o ~/.claude/skills/ralph/SKILL.md https://raw.githubusercontent.com/snarktank/ralph/main/skills/ralph/SKILL.md
```

### Option 3: Copy to your project (Alternative)

If you prefer to keep ralph.sh in your project directory:

```bash
# From your project root
mkdir -p scripts/ralph
cp /path/to/ralph/ralph.sh scripts/ralph/
chmod +x scripts/ralph/ralph.sh
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
# If ralph.sh is in your PATH (recommended)
ralph.sh [OPTIONS]

# If ralph.sh is in your project directory
./scripts/ralph/ralph.sh [OPTIONS]

# Examples
ralph.sh                           # Amp, default iterations
ralph.sh --tool claude 20          # Claude Code, 20 iterations
ralph.sh --tool opencode           # OpenCode, default iterations
ralph.sh --custom-prompt ./my-prompt.md  # With custom prompt
```

Run `ralph.sh --help` for all options.

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
| `ralph.sh` | The bash loop that spawns fresh AI instances (supports `--tool amp`, `--tool claude`, or `--tool opencode`) |
| `prompt-template.md` | Template for creating custom prompts (copy and modify for your project) |
| `prd.json` | User stories with `passes` status (the task list) |
| `prd.json.example` | Example PRD format for reference |
| `progress.txt` | Append-only learnings for future iterations |
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

Ralph looks for prompts in this order:
1. `--custom-prompt <file>` - Explicit flag takes highest priority
2. `.agents/ralph.md` - Project-local template (if exists)
3. Embedded default prompt

To customize for your project:
1. Copy `prompt-template.md` to `.agents/ralph.md` in your project root
2. Modify it for your needs  
3. Ralph will automatically use it

Example:
```bash
# Copy the template
cp /path/to/ralph/prompt-template.md .agents/ralph.md

# Edit it for your project
vim .agents/ralph.md

# Ralph will now use it automatically
./ralph.sh
```

### Post-Completion Cleanup

When all stories are complete, Ralph automatically removes working files in a final commit:
- `prd.json` - The task list
- `progress.txt` - The iteration log
- `.last-branch` - Branch tracking file
- The source PRD file (if specified in `prd.json`)

**This cleanup commit can be reverted:**
```bash
# Undo the cleanup
git revert HEAD

# Recover just the source PRD
git checkout HEAD~1 -- plans/my-feature.md
```

**To disable cleanup:** Create a custom prompt template (`.agents/ralph.md`) without the cleanup instructions in the Stop Condition section.

## Archiving

Ralph automatically archives previous runs when you start a new feature (different `branchName`). Archives are saved to `archive/YYYY-MM-DD-feature-name/`.

## References

- [Geoffrey Huntley's Ralph article](https://ghuntley.com/ralph/)
- [Amp documentation](https://ampcode.com/manual)
- [Claude Code documentation](https://docs.anthropic.com/en/docs/claude-code)
