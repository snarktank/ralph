# Ralph

![Ralph](ralph.webp)

Ralph is an autonomous AI agent loop that runs AI coding tools ([Amp](https://ampcode.com) or [Claude Code](https://docs.anthropic.com/en/docs/claude-code)) repeatedly until all PRD items are complete. Each iteration is a fresh instance with clean context. Memory persists via git history, `progress.txt`, and `prd.json`.

Based on [Geoffrey Huntley's Ralph pattern](https://ghuntley.com/ralph/).

[Read my in-depth article on how I use Ralph](https://x.com/ryancarson/status/2008548371712135632)

## Quick Start

```bash
cp .env.example .env         # Configure API key
./ralph.sh [max_iterations]  # Run with Amp (default)
./ralph.sh --tool claude 10  # Run with Claude Code
./ralph.sh --llm-provider nvidia --llm-api-key KEY 10  # Run with NVIDIA API
```

📖 **[Full setup & usage guide → SETUP.md](SETUP.md)**

## Key Files

| File | Purpose |
|------|---------|
| `ralph.sh` | Main agent loop (supports Amp, Claude Code, NVIDIA API) |
| `prompt.md` | Prompt template for Amp |
| `CLAUDE.md` | Prompt template for Claude Code |
| `prd.json.example` | Example PRD format |
| `.env.example` | Environment variables template |
| `SETUP.md` | Full setup & usage guide |
| `skills/` | PRD generation and conversion skills |
| `docs/` | Verification guide, test instructions |
| `flowchart/` | Interactive visualization ([live](https://snarktank.github.io/ralph/)) |

## How It Works

[![Ralph Flowchart](ralph-flowchart.png)](https://snarktank.github.io/ralph/)

**[View Interactive Flowchart](https://snarktank.github.io/ralph/)**

1. Ralph picks the highest priority story where `passes: false`
2. Spawns a **fresh AI instance** with clean context
3. The AI implements the story, runs checks, commits
4. Marks story as `passes: true`, appends learnings to `progress.txt`
5. Repeats until all stories pass or max iterations reached
6. Outputs `<promise>COMPLETE</promise>` when done

## Critical Concepts

### Small Tasks

Each PRD item should be small enough to complete in one context window.

✅ Right-sized: _"Add a database column"_, _"Add a UI component"_, _"Update server action"_
❌ Too big: _"Build the entire dashboard"_, _"Add authentication"_

### Feedback Loops

Ralph only works with feedback loops: typecheck, tests, CI. Broken code compounds across iterations.

### Verification / TDD Mode

Stories can include a `verificationCommand` in PRD. Ralph runs it after `finish` — pass marks the story done, fail retries. See [`docs/VERIFICATION_GUIDE.md`](docs/VERIFICATION_GUIDE.md).

### Self-Modification Sandbox

NVIDIA LLM agent can edit `ralph.sh` via staging copy. Changes validated with `bash -n` before applying — syntax errors are rejected.

### AGENTS.md Updates

After each iteration, Ralph updates `AGENTS.md` with discovered patterns and gotchas for future iterations.

## References

- [Geoffrey Huntley's Ralph article](https://ghuntley.com/ralph/)
- [Amp documentation](https://ampcode.com/manual)
- [Claude Code documentation](https://docs.anthropic.com/en/docs/claude-code)
- [NVIDIA API documentation](https://docs.nvidia.com/nim/large-language-models/latest/)