# Ralph Parallel Mode

Run N containerized Claude Code agents simultaneously against the same PRD. Each agent is sandboxed in Docker with network restrictions, resource limits, and no host access.

## How It Works

1. **Orchestrator** (`ralph-parallel.sh`) builds a Docker image, creates networks, and launches N containers
2. Each container runs the **agent loop** (`docker/agent-loop.sh`) which:
   - Clones the project from a bind-mounted directory
   - Claims a story in `prd.json` via git atomic push
   - Runs Claude Code with the parallel prompt
   - Pushes results and picks the next story
3. The orchestrator monitors container health and recovers stale claims
4. When all stories have `passes: true`, everything shuts down

## Prerequisites

- Docker installed and running
- A Claude Code auth token
- `jq` installed (`brew install jq` on macOS)
- A `prd.json` in the ralph root directory

## Quick Start

```bash
# Set your Claude auth token
export RALPH_CLAUDE_TOKEN='<your-oauth-token-json>'

# Run with 3 builder agents
./parallel/ralph-parallel.sh --agents 3

# Check status
./parallel/status.sh

# Graceful shutdown
./parallel/stop.sh
```

## CLI Options

```
./parallel/ralph-parallel.sh [options] [max_iterations]

Options:
  --project DIR       Project directory with prd.json (default: current dir)
  --image IMAGE       Custom Docker image (default: ralph-agent:latest)
  --agents N          Number of builder agents (default: 2)
  --researcher N      Number of researcher agents with full internet (default: 0)
  --model MODEL       Claude model (default: claude-sonnet-4-5-20250929)
  --memory SIZE       Per-container memory limit (default: 4g)
  --cpus N            Per-container CPU limit (default: 2)
  --allow-domain D    Extra domain to whitelist in firewall (repeatable)

Arguments:
  max_iterations    Per-agent iteration cap (default: 0 = until PRD complete)
```

## Authentication

Token retrieval priority (first wins):

1. **`RALPH_CLAUDE_TOKEN` env var** — set before running
2. **`.ralph/token` file** — write your token here
3. **1Password** via `op read` — interactive, startup only

## Story Claiming

Agents claim stories by modifying `prd.json` and using git's atomic push as a lock:

1. Agent finds highest-priority unclaimed story (`passes: false`, `claimed_by` empty)
2. Sets `claimed_by` and `claimed_at` fields
3. Commits and pushes
4. If push fails (another agent pushed first), rebase and pick a different story

### Stale Claim Recovery

The orchestrator checks for claims older than 30 minutes where the agent's container is no longer running. Stale claims are automatically cleared so other agents can pick up the work.

## Agent Roles

| Role | Network | Purpose |
|------|---------|---------|
| `builder` | API + allowed domains only | Feature implementation, testing, code changes |
| `researcher` | Full internet | Web research, documentation lookup |

Builder agents are restricted via iptables to only reach:
- `api.anthropic.com` (Claude API) — always allowed
- `statsig.anthropic.com` (telemetry) — always allowed
- Any domains passed via `--allow-domain`

Use `--allow-domain` to whitelist package registries your project needs:

```bash
# Node.js
./parallel/ralph-parallel.sh --allow-domain registry.npmjs.org

# Python
./parallel/ralph-parallel.sh \
  --allow-domain pypi.org \
  --allow-domain files.pythonhosted.org

# Go
./parallel/ralph-parallel.sh \
  --allow-domain proxy.golang.org \
  --allow-domain sum.golang.org

# Rust
./parallel/ralph-parallel.sh \
  --allow-domain crates.io \
  --allow-domain static.crates.io
```

## Custom Images

The default `ralph-agent:latest` image is based on `node:20-slim` (Node.js is required for Claude Code). If your project needs additional runtimes (Python, Go, Rust, etc.), extend the base image.

### `Dockerfile.ralph` Convention

The easiest way to make a project "ralph-ready" is to add a `Dockerfile.ralph` to the project root. When ralph detects this file, it automatically builds a project-specific image — no `--image` flag needed.

```
my-project/
├── Dockerfile.ralph    # <-- ralph auto-detects this
├── prd.json
├── src/
└── ...
```

```bash
# Ralph sees Dockerfile.ralph and builds automatically
./parallel/ralph-parallel.sh --project /path/to/my-project --agents 3
```

The image is tagged `ralph-agent-<project-name>:latest` (derived from `prd.json`'s `project` field) so multiple projects don't collide.

### Image Contract

`Dockerfile.ralph` should extend `ralph-agent:latest`. When doing so, you **must**:

- Preserve the `agent` user (UID 1001) — do not delete or change its UID
- Keep the `/opt/ralph/` scripts intact — do not remove or modify them
- Keep the default `ENTRYPOINT` (`/opt/ralph/agent-loop.sh`)
- Switch back to `USER agent` after installing system packages

### Example: Python Project

```dockerfile
# Dockerfile.ralph
FROM ralph-agent:latest
USER root
RUN apt-get update && apt-get install -y --no-install-recommends \
    python3 python3-pip python3-venv \
    && rm -rf /var/lib/apt/lists/*
USER agent
```

### Example: Go Project

```dockerfile
# Dockerfile.ralph
FROM ralph-agent:latest
USER root
RUN curl -fsSL https://go.dev/dl/go1.22.0.linux-$(dpkg --print-architecture).tar.gz \
    | tar -C /usr/local -xz
ENV PATH="/usr/local/go/bin:${PATH}"
USER agent
```

### Image Resolution Order

1. `--image IMAGE` flag — explicit override, used as-is
2. `Dockerfile.ralph` in the project directory — auto-built
3. Default `ralph-agent:latest` — base image with Node.js only

## Multi-PRP Mode

Multi-PRP mode runs independent feature branches simultaneously. Instead of N agents sharing one PRD, each PRP gets its own branch and dedicated agent. Use this when you have multiple independent features to build in parallel.

### How It Works

1. **Branch setup**: The orchestrator reads `branchName` from each PRP JSON file, creates a feature branch per PRP in the bare repo, and commits the PRP file to its branch
2. **Targeted launch**: Each container receives a `RALPH_BRANCH` env var telling it which branch to check out, bypassing the normal prp.json branch discovery
3. **Independent execution**: Agents work on their own branch with no cross-branch coordination or story contention
4. **Completion tracking**: The monitor loop checks each branch independently; when all branches report all stories passing, it fetches branches back and prints `gh pr create` commands

### Quick Start (Multi-PRP)

```bash
./parallel/launch-multi-prp.sh \
  --project /path/to/repo \
  --prp prps/feature-a.json \
  --prp prps/feature-b.json \
  --prp prps/feature-c.json \
  --model claude-sonnet-4-5-20250929
```

### CLI Options (Multi-PRP)

```
./parallel/launch-multi-prp.sh [options]

Required:
  --project DIR         Project git repository
  --prp FILE            PRP JSON file, relative to project dir (repeatable)

Options:
  --model MODEL         Claude model (default: claude-sonnet-4-5-20250929)
  --memory SIZE         Per-container memory limit (default: 4g)
  --cpus N              Per-container CPU limit (default: 2)
  --max-iterations N    Per-agent iteration cap (default: 0 = until done)
  --allow-domain D      Extra domain to whitelist in firewall (repeatable)
```

### PRP File Requirements

Each PRP JSON file must include:
- `branchName`: Feature branch name (e.g., `"ralph/prd-03-summarization"`)
- `userStories`: Array of stories with `id`, `passes`, `dependsOn`, etc.
- `project`: Project identifier (used for logging)

### RALPH_BRANCH Override

The `RALPH_BRANCH` env var is the key mechanism. When set in a container:
- `agent-loop.sh` skips its normal spec-file-based branch discovery
- Goes directly to the specified branch (checkout or create from remote)
- Backward compatible: empty string falls through to existing behavior

This is passed as the 10th parameter to `launch_agent()` in `docker-helpers.sh`.

### PRP Independence Pattern

When multiple PRPs share helper code (e.g., the same utility function), each PRP should include the helper creation as its own story with a note like "skip if file already exists from another branch merge." This makes PRPs fully independent — each agent can complete its work without depending on another branch being merged first.

### Differences from Single-PRP Mode

| | `ralph-parallel.sh` | `launch-multi-prp.sh` |
|---|---|---|
| Branches | One shared branch | One branch per PRP |
| Agents | N agents compete for stories | 1 agent per PRP, no competition |
| Use case | Parallelize within a feature | Parallelize across features |
| Completion | All stories in one PRP done | All branches done independently |
| Output | Stories marked `passes: true` | Branches synced + PR commands |
| Agent targeting | Branch from prp.json | `RALPH_BRANCH` env var override |

### Resource Planning

Each container uses the configured memory and CPU limits. For N PRPs:
- Memory: N × `--memory` (e.g., 6 PRPs × 4GB = 24GB)
- CPUs: N × `--cpus` (e.g., 6 PRPs × 2 = 12 CPUs)
- Token budget: all agents share one OAuth token; plan for the total session duration

### Monitoring

The monitor loop runs every 30 seconds and:
- Checks each branch independently for story completion
- Recovers stale claims on incomplete branches
- Detects auth failures (exit code 2) and halts all agents
- Restarts crashed containers for incomplete branches
- Prints PR creation commands when all branches complete

## File Layout

```
docker/
├── Dockerfile                      # Container image: node:20-slim + claude-code
├── agent-loop.sh                   # Container entrypoint: firewall → auth → clone → loop
├── init-firewall-builder.sh        # iptables: whitelist API + allowed domains
└── init-firewall-researcher.sh     # No-op (full internet)

parallel/
├── ralph-parallel.sh               # Single-PRP: N agents, one branch
├── launch-multi-prp.sh             # Multi-PRP: 1 agent per branch
├── stop.sh                         # Graceful shutdown
├── status.sh                       # Container status + story board + logs
├── CLAUDE-parallel.md              # Parallel-aware prompt for agents
├── README.md                       # This file
└── lib/
    ├── auth.sh                     # Token retrieval: env > file > 1Password
    ├── network-setup.sh            # Docker network create/teardown
    ├── docker-helpers.sh           # Container launch/stop/restart (10-param)
    └── logging.sh                  # Timestamped log helpers
```

## Exit Codes & Auth Failure Halt

Agent containers use distinct exit codes so the orchestrator can respond appropriately:

| Exit Code | Meaning | Orchestrator Action |
|-----------|---------|---------------------|
| 0 | Clean exit (stories complete, stop requested, iteration limit) | No action |
| 2 | Auth failure (credentials expired after 5 retries) | **Halt all agents** |
| Other non-zero | Crash, OOM, or unexpected error | Restart the container |

Since all agents share the same credential volume, a single auth failure means none can authenticate. When exit code 2 is detected, the orchestrator immediately stops all remaining containers, tears down networks, and exits with a message to refresh credentials. No restart loop occurs.

## Per-Agent Progress Files

Instead of all agents appending to one `progress.txt` (merge conflict risk), each agent writes to `progress-<agent-id>.txt`. The parallel prompt instructs agents to read ALL progress files for context and write only to their own.

## Differences from `ralph.sh`

| | `ralph.sh` (original) | `ralph-parallel.sh` (new) |
|---|---|---|
| Runs on | Host, bare metal | Host, launches Docker containers |
| Agents | 1, sequential | N, parallel |
| Prompt | `CLAUDE.md` | `parallel/CLAUDE-parallel.md` |
| Auth | Delegates to CLI | Env var / file / 1Password |
| Network | Unrestricted | iptables firewall per container |
| Progress | `progress.txt` | `progress-<agent-id>.txt` per agent |
| Config | CLI args | CLI args |

## Debugging

```bash
# Check container status
./parallel/status.sh

# View container logs directly
docker logs ralph-agent-1

# View agent log files
ls -lt agent_logs/

# Check prd.json story status
cat prd.json | jq '.userStories[] | {id, title, passes, claimed_by}'

# Force rebuild the Docker image
docker rmi ralph-agent:latest
./parallel/ralph-parallel.sh --agents 1
```
