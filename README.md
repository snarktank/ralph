# Ralph

![Ralph](ralph.webp)

Ralph is an autonomous AI agent loop that runs [Codex CLI](https://developers.openai.com/codex/) repeatedly until all PRD items are complete. Each iteration is a fresh Codex instance with clean context. Memory persists via git history, `progress.txt`, and `prd.json`. (Amp is still supported as an optional engine.)

Based on [Geoffrey Huntley's Ralph pattern](https://ghuntley.com/ralph/).

[Read my in-depth article on how I use Ralph](https://x.com/ryancarson/status/2008548371712135632)

## Prerequisites

- [Codex CLI](https://developers.openai.com/codex/) installed and authenticated
  - `codex login` for local use, or set `CODEX_API_KEY` in CI
- `jq` installed (`brew install jq` on macOS)
- A git repository for your project
- (Optional) [Amp CLI](https://ampcode.com) if you want to use the legacy engine

## Setup

### Option 1: Copy to your project

Copy the ralph files into your project:

```bash
# From your project root
mkdir -p scripts/ralph
cp /path/to/ralph/ralph.sh scripts/ralph/
cp /path/to/ralph/prompt.md scripts/ralph/
chmod +x scripts/ralph/ralph.sh
```

### Option 2: Install skills globally (Codex)

Copy the skills to your Codex config for use across all projects:

```bash
cp -r .codex/skills/ralph-codex ~/.codex/skills/
# Optional: make the PRD and PRD->JSON converter skills available to Codex
cp -r skills/prd ~/.codex/skills/
cp -r skills/ralph ~/.codex/skills/
```

### Configure Amp auto-handoff (Amp only, recommended)

Add to `~/.config/amp/settings.json`:

```json
{
  "amp.experimental.autoHandoff": { "context": 90 }
}
```

This enables automatic handoff when context fills up, allowing Ralph to handle large stories that exceed a single context window.

## Codex Skills

Codex automatically discovers skills from:
- `.codex/skills` in your repo
- `~/.codex/skills` globally

Optional: install curated skills with the `$skill-installer` skill (restart Codex after installing):

```
Load the $skill-installer skill and install [skill-name] from openai/skills
```

## Workflow

### 1. Create a PRD

Use the PRD skill to generate a detailed requirements document (install `skills/prd` into `~/.codex/skills` or copy into `.codex/skills` first):

```
Load the prd skill and create a PRD for [your feature description]
```

Answer the clarifying questions. The skill saves output to `tasks/prd-[feature-name].md`.

### 2. Convert PRD to Ralph format

Use the Ralph skill to convert the markdown PRD to JSON (install `skills/ralph` into `~/.codex/skills` or copy into `.codex/skills` first):

```
Load the ralph skill and convert tasks/prd-[feature-name].md to prd.json
```

This creates `prd.json` with user stories structured for autonomous execution.

### 3. Run Ralph

```bash
./scripts/ralph/ralph.sh [max_iterations]
```

Default is 10 iterations.

Codex runs in read-only mode by default. For edits, enable full auto:

```bash
RALPH_CODEX_FULL_AUTO=1 ./scripts/ralph/ralph.sh [max_iterations]
```

You can override the sandbox policy if you need broader access:

```bash
RALPH_CODEX_SANDBOX=workspace-write ./scripts/ralph/ralph.sh [max_iterations]
```

To use Amp instead of Codex:

```bash
RALPH_ENGINE=amp ./scripts/ralph/ralph.sh [max_iterations]
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

## Key Files

| File | Purpose |
|------|---------|
| `ralph.sh` | The bash loop that spawns fresh Codex instances |
| `prompt.md` | Instructions given to each Codex instance |
| `prd.json` | User stories with `passes` status (the task list) |
| `prd.json.example` | Example PRD format for reference |
| `progress.txt` | Append-only learnings for future iterations |
| `.codex/skills/ralph-codex/` | Repo-local Codex skill for Ralph conventions |
| `skills/prd/` | Skill for generating PRDs (copy to `~/.codex/skills` if using Codex) |
| `skills/ralph/` | Skill for converting PRDs to JSON (copy to `~/.codex/skills` if using Codex) |
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

Each iteration spawns a **new Codex instance** with clean context. The only memory between iterations is:
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

After each iteration, Ralph updates the relevant `AGENTS.md` files with learnings. This is key because Codex automatically reads these files, so future iterations (and future human developers) benefit from discovered patterns, gotchas, and conventions.

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

## Customizing prompt.md

Edit `prompt.md` to customize Ralph's behavior for your project:
- Add project-specific quality check commands
- Include codebase conventions
- Add common gotchas for your stack

## Archiving

Ralph automatically archives previous runs when you start a new feature (different `branchName`). Archives are saved to `archive/YYYY-MM-DD-feature-name/`.

## References

- [Geoffrey Huntley's Ralph article](https://ghuntley.com/ralph/)
- [Codex CLI docs](https://developers.openai.com/codex/)
- [Codex CLI reference](https://developers.openai.com/codex/cli/reference/)
- [Codex skills](https://developers.openai.com/codex/skills/)
- [Codex AGENTS.md guide](https://developers.openai.com/codex/guides/agents-md/)
- [Amp documentation (legacy)](https://ampcode.com/manual)
