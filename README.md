# Ralph for Cursor

![Ralph](ralph.webp)

Ralph is an autonomous AI agent loop that runs Cursor CLI repeatedly until all PRD items are complete. Each iteration is a fresh Cursor CLI agent instance with clean context. Memory persists via git history, `progress.txt`, and `prd.json`.

Based on [Geoffrey Huntley's Ralph pattern](https://ghuntley.com/ralph/).

[Read my in-depth article on how I use Ralph](https://x.com/ryancarson/status/2008548371712135632)

## Prerequisites

- [Cursor CLI](https://cursor.com/docs/cli) installed and authenticated
- `jq` installed (`brew install jq` on macOS, `choco install jq` on Windows)
- A git repository for your project

### Windows Users

Ralph includes a PowerShell version (`ralph.ps1`) for native Windows support:

```powershell
# Run Ralph on Windows
.\ralph.ps1 [max_iterations]

# Or with explicit iteration count
.\ralph.ps1 -MaxIterations 12
```

Alternatively, use WSL (Windows Subsystem for Linux) to run `ralph.sh`.

## Setup

### Option 1: Copy to your project

Copy the ralph files into your project:

**macOS/Linux:**
```bash
# From your project root
mkdir -p scripts/ralph
cp /path/to/ralph/ralph.sh scripts/ralph/
cp /path/to/ralph/prompt.md scripts/ralph/
chmod +x scripts/ralph/ralph.sh
```

**Windows (PowerShell):**
```powershell
# From your project root
mkdir -Force scripts\ralph
Copy-Item C:\path\to\ralph\ralph.ps1 scripts\ralph\
Copy-Item C:\path\to\ralph\prompt.md scripts\ralph\
```

### Option 2: Install skills globally

Copy the skills to your Cursor CLI config for use across all projects:

```bash
cp -r skills/prd ~/.cursor/skills/
cp -r skills/ralph ~/.cursor/skills/
```

Note: Cursor CLI automatically handles context management, so no additional configuration is needed.

### Option 3: Copy ralph_install.sh to project and run
Copy the ralph_install.sh from copy_to_project/ralph_install.sh

### Option 4: Use the PRD Web UI

Ralph includes a web-based UI for creating PRDs and converting them to JSON format. This provides a user-friendly alternative to using Cursor CLI skills.

See [PRD UI Documentation](prd-ui/README.md) for setup and usage instructions.

## Workflow

### 1. Create a PRD

**Option A: Using the Web UI (Recommended)**

1. Start the PRD UI (see [PRD UI Documentation](prd-ui/README.md))
2. Navigate to "Create PRD"
3. Select your project directory
4. Follow the guided wizard to create your PRD
5. The PRD will be saved to `tasks/prd-[feature-name].md`

**Option B: Using Cursor CLI Skills**

Use the PRD skill to generate a detailed requirements document:

```
Load the prd skill and create a PRD for [your feature description]
```

Answer the clarifying questions. The skill saves output to `tasks/prd-[feature-name].md`.

### 2. Convert PRD to Ralph format

**Option A: Using the Web UI (Recommended)**

1. In the PRD UI, navigate to "Convert to JSON"
2. Select your project directory
3. Choose an existing PRD file or paste PRD content
4. Review the generated JSON
5. Save `prd.json` to your project root

**Option B: Using Cursor CLI Skills**

Use the Ralph skill to convert the markdown PRD to JSON:

```
Load the ralph skill and convert tasks/prd-[feature-name].md to prd.json
```

This creates `prd.json` with user stories structured for autonomous execution.

### 3. Run Ralph

```bash
./scripts/ralph/ralph.sh [max_iterations]
```

Default is 10 iterations.

**Note**: `ralph.sh` uses Cursor CLI agent with `--print --force` flags to enable shell execution. This allows Ralph to run git commands, quality checks, and commits automatically.

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
| `ralph.sh` | The bash loop that spawns fresh Cursor CLI agent instances (macOS/Linux) |
| `ralph.ps1` | PowerShell version for Windows |
| `prompt.md` | Instructions given to each Cursor CLI agent instance |
| `prd.json` | User stories with `passes` status (the task list) |
| `prd.json.example` | Example PRD format for reference |
| `progress.txt` | Append-only learnings for future iterations |
| `skills/prd/` | Skill for generating PRDs |
| `skills/ralph/` | Skill for converting PRDs to JSON |
| `flowchart/` | Interactive visualization of how Ralph works |
| `prd-ui/` | Web UI for creating PRDs and converting to JSON |

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

Each iteration spawns a **new Cursor CLI agent instance** with clean context. The only memory between iterations is:
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

After each iteration, Ralph updates the relevant `AGENTS.md` files with learnings. This is key because Cursor CLI automatically reads these files, so future iterations (and future human developers) benefit from discovered patterns, gotchas, and conventions.

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

### Troubleshooting

**Shell execution unavailable error:**
- Ensure Cursor CLI is installed and authenticated: `agent --version` and `agent status`
- Verify `ralph.sh` uses `--print --force` flags (should be automatic)
- Check that `agent` command is in your PATH
- The script automatically checks for required commands (`agent` and `jq`) on startup

**Agent command not found:**
- Install Cursor CLI: https://cursor.com/docs/cli
- Verify installation: `agent --version`
- Ensure `agent` is in your PATH
- Test agent command: `agent --print --force --output-format text "Test"`

**PRD generation or JSON conversion fails in PRD UI:**
- Check that Cursor CLI agent is available and authenticated
- The PRD UI uses `spawn` for reliable agent command execution
- Agent output is automatically parsed (handles wrapped JSON, markdown code fences, etc.)
- The UI automatically falls back to template-based methods if agent fails
- See [PRD UI Documentation](prd-ui/README.md) for detailed troubleshooting

**jq command not found:**
- Install jq: `brew install jq` (macOS) or see https://stedolan.github.io/jq/download/
- Verify installation: `jq --version`

## Customizing prompt.md

Edit `prompt.md` to customize Ralph's behavior for your project:
- Add project-specific quality check commands
- Include codebase conventions
- Add common gotchas for your stack

## Archiving

Ralph automatically archives previous runs when you start a new feature (different `branchName`). Archives are saved to `archive/YYYY-MM-DD-feature-name/`.

## PRD Web UI

Ralph includes a full-stack web application for creating and managing PRDs through a user-friendly interface. The PRD UI provides:

- **Guided PRD Creation**: Multi-step wizard with intelligent question generation
- **PRD to JSON Conversion**: Convert markdown PRDs to Ralph's JSON format
- **Cursor CLI Integration**: Uses Cursor CLI agent for enhanced generation (with template fallback)
- **Project Management**: Point to any project directory to manage PRDs
- **Real-time Preview**: See PRD markdown and JSON previews as you work

### Technical Highlights

- Uses Node.js `spawn` for reliable agent command execution (no shell escaping issues)
- Handles agent output formats (wrapped JSON, markdown code fences, etc.)
- Automatic fallback to template-based methods if agent unavailable
- Comprehensive test suite with unit and integration tests

See the [PRD UI Documentation](prd-ui/README.md) for detailed setup and usage instructions.

## References

- [Geoffrey Huntley's Ralph article](https://ghuntley.com/ralph/)
- [Cursor CLI documentation](https://cursor.com/docs/cli)
