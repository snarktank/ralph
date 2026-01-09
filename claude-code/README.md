# Ralph for Claude Code

![Ralph](../ralph.webp)

Ralph is an autonomous AI agent loop that runs [Claude Code](https://docs.anthropic.com/en/docs/claude-code) repeatedly until all PRD items are complete. Each iteration is a fresh Claude instance with clean context. Memory persists via git history, `progress.txt`, and `prd.json`.

Based on [Geoffrey Huntley's Ralph pattern](https://ghuntley.com/ralph/).

## Prerequisites

- [Claude Code CLI](https://docs.anthropic.com/en/docs/claude-code) installed and authenticated
- `jq` installed (`brew install jq` on macOS, `apt install jq` on Linux)
- A git repository for your project

## Quick Start

### 1. Copy Ralph to your project

```bash
# From your project root
mkdir -p scripts/ralph
cp /path/to/ralph/claude-code/* scripts/ralph/
chmod +x scripts/ralph/ralph.sh
```

### 2. Create a PRD

Create `prd.json` in your ralph directory. Use `prd.json.example` as a template:

```json
{
  "project": "MyApp",
  "branchName": "ralph/feature-name",
  "description": "Feature description",
  "userStories": [
    {
      "id": "US-001",
      "title": "Story title",
      "description": "As a user, I want X so that Y",
      "acceptanceCriteria": [
        "Criterion 1",
        "Criterion 2",
        "Typecheck passes"
      ],
      "priority": 1,
      "passes": false,
      "notes": ""
    }
  ]
}
```

### 3. Run Ralph

```bash
./scripts/ralph/ralph.sh [max_iterations]
```

Default is 10 iterations.

## How It Works

Ralph spawns a fresh Claude Code instance for each iteration:

```
┌─────────────────────────────────────────────────────────────┐
│                      ralph.sh loop                          │
├─────────────────────────────────────────────────────────────┤
│  for i in 1..max_iterations:                                │
│    1. Read prompt.md                                        │
│    2. Spawn: claude --dangerously-skip-permissions --print  │
│    3. Claude reads prd.json, picks next story               │
│    4. Implements story, runs tests, commits                 │
│    5. Updates prd.json (passes: true)                       │
│    6. Appends to progress.txt                               │
│    7. If all done → outputs "RALPH_COMPLETE" → exit         │
│    8. Otherwise → next iteration                            │
└─────────────────────────────────────────────────────────────┘
```

Each iteration:
1. Creates/checks out the feature branch (from PRD `branchName`)
2. Picks the highest priority story where `passes: false`
3. Implements that single story
4. Runs quality checks (typecheck, tests)
5. Commits if checks pass
6. Updates `prd.json` to mark story as `passes: true`
7. Appends learnings to `progress.txt`
8. Repeats until all stories pass or max iterations reached

## Key Files

| File | Purpose |
|------|---------|
| `ralph.sh` | The bash loop that spawns fresh Claude instances |
| `prompt.md` | Instructions given to each Claude instance |
| `prd.json` | User stories with `passes` status (your task list) |
| `prd.json.example` | Example PRD format for reference |
| `progress.txt` | Append-only learnings for future iterations |
| `CLAUDE.md` | Claude Code context file with patterns |

## Story Sizing: Critical

**Each story must be completable in ONE iteration (one context window).**

Claude spawns fresh each iteration with no memory of previous work. If a story is too big, Claude runs out of context before finishing.

Good story size examples:
- Add a database column and migration
- Create a single component
- Add one API endpoint
- Write tests for one module

Too big:
- "Build the entire auth system"
- "Refactor the whole codebase"

## Memory Persistence

Since each iteration is a fresh Claude instance, memory persists through:

1. **Git history** - Previous commits and their messages
2. **prd.json** - Which stories are complete (`passes: true`)
3. **progress.txt** - Learnings and patterns from previous iterations
4. **CLAUDE.md files** - Consolidated patterns for the codebase

## Progress.txt Format

Ralph appends to `progress.txt` after each story:

```markdown
## Codebase Patterns
- Pattern 1 discovered
- Pattern 2 discovered

---

## 2024-01-15 10:30 - US-001
- What was implemented
- Files changed
- **Learnings for future iterations:**
  - Patterns discovered
  - Gotchas encountered
---
```

## Differences from Amp Version

| Feature | Amp | Claude Code |
|---------|-----|-------------|
| CLI | `amp` | `claude` |
| Permissions | `--dangerously-allow-all` | `--dangerously-skip-permissions` |
| Output mode | piped stdin | `--print` flag |
| Completion signal | `<promise>COMPLETE</promise>` | `RALPH_COMPLETE` |
| Context files | `AGENTS.md` | `CLAUDE.md` |
| Skills | `~/.config/amp/skills/` | Natural prompting |

## Troubleshooting

### Claude not found
Make sure Claude Code CLI is installed and in your PATH:
```bash
which claude
claude --version
```

### Permission denied
```bash
chmod +x ralph.sh
```

### Story too large
If Ralph keeps failing on a story, break it into smaller stories in `prd.json`.

### Stuck in loop
Check `progress.txt` for what's happening. You may need to manually intervene and update `prd.json`.

## Safety Notes

The `--dangerously-skip-permissions` flag allows Claude to:
- Execute shell commands without confirmation
- Read/write files without prompting
- Make git commits

Only run Ralph in trusted environments on code you're willing to have modified.
