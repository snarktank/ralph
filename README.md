# Ralph

[![CI](https://github.com/kcirtapfromspace/ralph/actions/workflows/ci.yml/badge.svg)](https://github.com/kcirtapfromspace/ralph/actions/workflows/ci.yml)
[![Docker](https://github.com/kcirtapfromspace/ralph/actions/workflows/docker.yml/badge.svg)](https://github.com/kcirtapfromspace/ralph/actions/workflows/docker.yml)

![Ralph](ralph.webp)

Ralph is an autonomous AI agent loop that runs [Claude Code](https://docs.anthropic.com/en/docs/claude-code) repeatedly until all PRD items are complete. Each iteration is a fresh Claude Code instance with clean context. Memory persists via git history, `progress.txt`, and `prd.json`.

Based on [Geoffrey Huntley's Ralph pattern](https://ghuntley.com/ralph/).

[Read my in-depth article on how I use Ralph](https://x.com/ryancarson/status/2008548371712135632)

## Prerequisites

- [Claude Code CLI](https://docs.anthropic.com/en/docs/claude-code) installed and authenticated
- [Rust](https://rustup.rs) (for building from source)
- A git repository for your project

## Installation

### Homebrew (macOS/Linux)

```bash
brew install --HEAD https://raw.githubusercontent.com/kcirtapfromspace/ralph/main/Formula/ralph.rb
```

### Build from Source

Requires [Rust](https://rustup.rs) to be installed.

```bash
git clone https://github.com/kcirtapfromspace/ralph.git ~/.ralph
cd ~/.ralph
./install.sh
```

This builds the Rust binary and installs it to `/usr/local/bin`. For a different location:

```bash
./install.sh ~/bin        # Install to ~/bin
sudo ./install.sh         # If you need sudo for /usr/local/bin
```

Then add to your shell config (`~/.bashrc`, `~/.zshrc`):

```bash
export RALPH_HOME="$HOME/.ralph"
```

To uninstall:
```bash
./uninstall.sh
```

### Docker / Claude Desktop (MCP)

Prefer Claude Desktop over the terminal? Ralph is also available as an MCP server via Docker:

```bash
docker pull ghcr.io/kcirtapfromspace/ralph:latest
```

See the [Docker MCP Setup Guide](docs/guides/docker-mcp-setup.md) for Claude Desktop configuration.

## Usage

```bash
ralph [OPTIONS] [MAX_ITERATIONS]
ralph <COMMAND>

Commands:
  init    Initialize project with prd.json template
  home    Show Ralph installation directory

Options:
  -d, --dir <PATH>       Working directory (default: current directory)
  -p, --prompt <FILE>    Custom prompt file
  -n, --iterations <N>   Max iterations (default: 10)
  -h, --help             Show help
  -V, --version          Show version

Examples:
  ralph                  Run in current directory with defaults
  ralph 20               Run with 20 max iterations
  ralph -d ./my-project  Run in specified directory
  ralph init             Create prd.json template
```

## Workflow

### 1. Initialize your project

```bash
cd your-project
ralph --init
```

This creates a `prd.json` template.

### 2. Define your tasks

Edit `prd.json` with your user stories. Each story should be small enough to complete in one iteration. See `prd.json.example` for the format.

You can also use Claude to help create PRDs:

```bash
# Use the PRD template
cat $(ralph --home)/skills/prd/SKILL.md | claude --print "Create a PRD for [feature]"

# Then convert to ralph format
cat $(ralph --home)/skills/ralph/SKILL.md | claude --print "Convert this PRD to prd.json"
```

### 3. Run Ralph

```bash
ralph              # Run with defaults (10 iterations)
ralph 20           # Run with 20 max iterations
ralph -d ./other   # Run in different directory
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
| `bin/ralph` | Global CLI binary |
| `install.sh` | Installer script |
| `prompt.md` | Instructions given to each Claude Code instance |
| `prd.json.example` | Example PRD format for reference |
| `skills/prd/` | Prompt template for generating PRDs |
| `skills/ralph/` | Prompt template for converting PRDs to JSON |
| `flowchart/` | Interactive visualization of how Ralph works |

**In your project directory:**

| File | Purpose |
|------|---------|
| `prd.json` | User stories with `passes` status (created by `ralph --init`) |
| `progress.txt` | Append-only learnings for future iterations |
| `archive/` | Previous run archives |

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

Frontend stories must include "Verify in browser" in acceptance criteria. Ralph will navigate to the page, interact with the UI, and confirm changes work.

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

## Docker Deployment

Ralph can also run as an MCP server in Docker for use with Claude Desktop:

| File | Purpose |
|------|---------|
| `Dockerfile` | Multi-stage build for Ralph MCP server |
| `docker-compose.yml` | Local development and testing |
| `.dockerignore` | Build context optimization |
| `examples/` | MCP toolkit configuration examples |
| `docs/guides/docker-mcp-setup.md` | Docker MCP setup guide |

```bash
# Build locally
docker build -t ralph-mcp .

# Run with docker-compose
docker compose up --build
```

Images are published to `ghcr.io/kcirtapfromspace/ralph` on every push to main.

## References

- [Geoffrey Huntley's Ralph article](https://ghuntley.com/ralph/)
- [Claude Code documentation](https://docs.anthropic.com/en/docs/claude-code)
- [Docker MCP Setup Guide](docs/guides/docker-mcp-setup.md)
