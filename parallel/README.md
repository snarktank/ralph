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

## File Layout

```
docker/
├── Dockerfile                      # Container image: node:20-slim + claude-code
├── agent-loop.sh                   # Container entrypoint: firewall → auth → clone → loop
├── init-firewall-builder.sh        # iptables: whitelist API + allowed domains
└── init-firewall-researcher.sh     # No-op (full internet)

parallel/
├── ralph-parallel.sh               # Host orchestrator: launch, monitor, restart
├── stop.sh                         # Graceful shutdown
├── status.sh                       # Container status + story board + logs
├── CLAUDE-parallel.md              # Parallel-aware prompt for agents
├── README.md                       # This file
└── lib/
    ├── auth.sh                     # Token retrieval: env > file > 1Password
    ├── network-setup.sh            # Docker network create/teardown
    ├── docker-helpers.sh           # Container launch/stop/restart
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
