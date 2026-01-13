# Ralph Skills

Agent Skills for Claude that enable the Ralph autonomous agent pattern. These skills allow Claude to create PRDs, convert them to executable format, and autonomously implement user stories iteratively.

Based on [Geoffrey Huntley's Ralph pattern](https://ghuntley.com/ralph/).

## Prerequisites

- Claude Desktop (Claude Code) or Claude API with Skills support
- A git repository for your project
- Project with quality checks (typecheck, lint, test)

## Setup

### Option 1: Install in Claude Code

Copy skills to your Claude Code skills directory:

```bash
# For project-specific skills
cp -r skills/prd .claude/skills/
cp -r skills/ralph .claude/skills/

# For global skills (all projects)
cp -r skills/prd ~/.claude/skills/
cp -r skills/ralph ~/.claude/skills/
```

### Option 2: Use via Claude API

Upload skills via the Skills API (`/v1/skills` endpoints). Skills are shared organization-wide via the API.

## Skills Overview

### `prd` Skill

Generates detailed Product Requirements Documents (PRDs) with user stories, acceptance criteria, and technical considerations.

**Use when:** Planning a feature, starting a new project, or creating requirements.

**Triggers on:** "create a prd", "write prd for", "plan this feature", "requirements for", "spec out"

### `ralph` Skill

Executes the Ralph autonomous agent loop to implement user stories iteratively. Also converts PRDs to `prd.json` format.

**Use when:** You have `prd.json` and want to implement stories autonomously, or need to convert a PRD to JSON format.

**Triggers on:** "run ralph", "execute ralph", "implement stories from prd.json", "convert this prd", "ralph json"

## Workflow

### 1. Create a PRD

Use the `prd` skill to generate a requirements document:

```
Load the prd skill and create a PRD for [your feature description]
```

The skill will:
1. Ask 3-5 clarifying questions with lettered options
2. Generate a structured PRD based on your answers
3. Save to `tasks/prd-[feature-name].md`

**Important:** Stories are automatically sized to fit within one context window.

### 2. Convert PRD to Ralph Format

Use the `ralph` skill to convert the markdown PRD to JSON:

```
Load the ralph skill and convert tasks/prd-[feature-name].md to prd.json
```

This creates `prd.json` with user stories structured for autonomous execution.

### 3. Execute Stories

Use the `ralph` skill to implement stories autonomously:

```
Load the ralph skill and execute stories from prd.json
```

Ralph will:
1. Read `prd.json` and `progress.txt`
2. Check you're on the correct branch (from PRD `branchName`)
3. Pick the highest priority story where `passes: false`
4. Implement that single story
5. Run quality checks (typecheck, lint, test)
6. Update AGENTS.md files if patterns discovered
7. Commit if checks pass: `feat: [Story ID] - [Story Title]`
8. Update `prd.json` to mark story as `passes: true`
9. Append learnings to `progress.txt`
10. Continue until all stories pass or you stop it

**Completion:** When all stories have `passes: true`, Ralph signals `<promise>COMPLETE</promise>`.

## Key Files

| File | Purpose |
|------|---------|
| `prd/SKILL.md` | Instructions for generating PRDs |
| `ralph/SKILL.md` | Main execution workflow and PRD conversion |
| `ralph/HANDOFF.md` | Handoff template (loaded when context limit reached) |
| `ralph/CONTEXT.md` | Context management guide (loaded when needed) |
| `prd.json` | User stories with `passes` status (created in your project) |
| `progress.txt` | Append-only learnings for future iterations (created in your project) |

## Critical Concepts

### Story Sizing

Each story must be completable in **one context window**. If a story is too big, Claude runs out of context before finishing and produces broken code.

**Right-sized stories:**
- Add a database column and migration
- Add a UI component to an existing page
- Update a server action with new logic
- Add a filter dropdown to a list

**Too big (split these):**
- "Build the entire dashboard" → Split into: schema, queries, UI components, filters
- "Add authentication" → Split into: schema, middleware, login UI, session handling
- "Refactor the API" → Split into one story per endpoint

**Rule of thumb:** If you cannot describe the change in 2-3 sentences, it is too big.

### Context Management

Unlike Amp's auto-handoff, Claude Desktop requires manual context management:

- **Monitor token usage** throughout execution
- **At ~90% context:** Commit work, update state, create `HANDOFF.md`, signal user to start new session
- **Next session:** Read `HANDOFF.md`, continue from where previous session left off

See `ralph/CONTEXT.md` for detailed procedures (loaded automatically when needed).

### State Persistence

Ralph persists state across sessions via:

- **`prd.json`** - Current story status and project info
- **`progress.txt`** - Append-only log with Codebase Patterns section at top
- **Git history** - Each story gets a commit when complete
- **AGENTS.md files** - Reusable patterns discovered during implementation
- **HANDOFF.md** - Temporary file created when context limit reached

### AGENTS.md Updates

After each iteration, Ralph updates relevant `AGENTS.md` files with reusable patterns:

- API patterns or conventions specific to that module
- Gotchas or non-obvious requirements
- Dependencies between files
- Testing approaches for that area

This helps future iterations (and human developers) understand the codebase better.

### Browser Verification

Frontend stories must include "Verify in browser using dev-browser skill" in acceptance criteria. Ralph will use the dev-browser skill to navigate, interact with the UI, and confirm changes work.

### Feedback Loops

Ralph only works if there are feedback loops:

- Typecheck catches type errors
- Tests verify behavior
- CI must stay green (broken code compounds across iterations)

## Usage Examples

### Creating a PRD

```
Load the prd skill and create a PRD for adding task priority levels

[Answer clarifying questions: 1A, 2C, 3B]

PRD saved to tasks/prd-task-priority.md
```

### Converting to JSON

```
Load the ralph skill and convert tasks/prd-task-priority.md to prd.json

prd.json created with 4 user stories
```

### Executing Stories

```
Load the ralph skill and execute stories from prd.json

[Claude implements US-001: Add priority field to database]
✓ Typecheck passes
✓ Migration successful
✓ Committed: feat: US-001 - Add priority field to database
✓ Updated prd.json: US-001 passes: true
✓ Appended to progress.txt

[Claude continues with US-002, US-003, etc.]
```

### Handling Context Overflow

```
[Context approaching ~90%]

✓ Committed in-progress work
✓ Updated prd.json with progress notes
✓ Created HANDOFF.md with state snapshot
→ Please start a new session and load HANDOFF.md to continue
```

## Debugging

Check current state:

```bash
# See which stories are done
cat prd.json | jq '.userStories[] | {id, title, passes}'

# See learnings from previous iterations
cat progress.txt

# Check git history
git log --oneline -10

# Check for handoff file
cat HANDOFF.md
```

## Progressive Disclosure

The skills use progressive disclosure to minimize context usage:

- **Level 1:** Skill metadata (name, description) - always loaded
- **Level 2:** Main SKILL.md instructions - loaded when skill triggered
- **Level 3:** Supporting files (HANDOFF.md, CONTEXT.md) - loaded only when needed

This ensures only relevant content occupies the context window at any given time.

## Archiving

Before creating a new `prd.json` for a different feature:

1. Check if existing `prd.json` has different `branchName`
2. If different AND `progress.txt` has content:
   - Create archive: `archive/YYYY-MM-DD-feature-name/`
   - Copy `prd.json` and `progress.txt` to archive
   - Reset `progress.txt` with fresh header

The `ralph` skill handles this automatically when converting PRDs.

## Differences from Amp Version

| Feature | Amp Version | Skills Version |
|---------|------------|----------------|
| Loop execution | External bash script (`ralph.sh`) | Claude executes loop directly |
| Context handoff | Automatic (`amp.experimental.autoHandoff`) | Manual (create HANDOFF.md) |
| Session spawning | Script spawns fresh instances | User starts new sessions |
| State files | Same (`prd.json`, `progress.txt`) | Same |
| Workflow | Same (PRD → JSON → Execute) | Same |

The skills version provides the same functionality but requires manual session management instead of automatic loop execution.

## References

- [Geoffrey Huntley's Ralph article](https://ghuntley.com/ralph/)
- [Claude Agent Skills documentation](https://docs.anthropic.com/en/docs/agents-and-tools/agent-skills)
- [Original Ralph repository](https://github.com/snarktank/ralph)
