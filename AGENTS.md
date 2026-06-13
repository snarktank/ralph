# Ralph Harness — Agent Instructions

## Overview

Ralph Harness is an autonomous AI agent loop that runs AI coding tools (Amp or Claude Code) repeatedly until all PRD items are complete. Each iteration is a fresh instance with clean context.

Ralph supports two modes:

| Mode | Architecture | When to use |
|------|-------------|-------------|
| `simple` | Single agent (self-implement, self-check) | Quick tasks, backend-only stories, well-defined small changes |
| `harness` | Generator + Evaluator (dual-agent with contract) | UI-heavy features, complex stories, when quality is critical |

## Architecture: Harness Mode

```
ralph.sh orchestrator
  │
  ├── Planner (prd.json)
  │     Defines user stories, acceptance criteria, dependencies
  │
  ├── Generator (generator-prompt.md)
  │     Drafts sprint contracts → Implements stories → Fixes based on feedback
  │
  └── Evaluator (evaluator-prompt.md)
        Reviews contracts → Signs/locks → Tests in browser → Scores → Writes feedback
```

### Per-Story Flow

1. **Contract Negotiation**: Generator drafts `contract.json` → Evaluator reviews → Back-and-forth until Evaluator signs → Contract locked (immutable)
2. **Build**: Generator reads locked contract → implements → runs typecheck/lint/test → commits
3. **Evaluate**: Evaluator reads locked contract → tests in browser (Playwright MCP) → scores on 4 dimensions → writes `evaluation.json`
4. **Retry or Advance**: If all scores pass thresholds → mark story done → next story. If any fail → feedback to Generator → retry (up to `--max-retries`)

### Contract Lifecycle

```
proposed → evaluator_review → locked (immutable)
              │                    ↑
              └→ generator_revise ─┘
```

Once `locked`, neither agent can modify `contract.json`. Evaluation is strict: only judge against what the contract says, not what you think should have been in it.

### Evaluation Dimensions

| Dimension | Weight | Threshold | What it checks |
|-----------|--------|-----------|----------------|
| Functionality | 30% | 70 | Do all acceptance criteria actually work? |
| Code Quality | 25% | 60 | Does code follow project patterns? |
| Design Quality | 25% | 65 | Is the UI coherent and original? |
| Product Depth | 20% | 50 | Is there real functionality, not just a shell? |

## Commands

```bash
# Harness mode (default) — Generator + Evaluator
./ralph.sh --mode harness --tool claude

# Simple mode — original single-agent behavior
./ralph.sh --mode simple --tool claude

# Tune retry behavior
./ralph.sh --mode harness --max-retries 5 --max-contract-rounds 3

# Run the flowchart dev server
cd flowchart && npm run dev
```

## Key Files

| File | Role |
|------|------|
| `ralph.sh` | Orchestrator — spawns AI instances in correct order with phase control |
| `generator-prompt.md` | Instructions for Generator agent (contract drafting + implementation) |
| `evaluator-prompt.md` | Instructions for Evaluator agent (contract review + QA scoring) |
| `CLAUDE.md` | Instructions for simple mode (original single-agent behavior) |
| `prd.json` | Planner — user stories with status and evaluation data |
| `contract.json` | Sprint contract — Generator proposes, Evaluator signs, then locked |
| `evaluation.json` | QA results — scores, per-criterion verification, feedback |
| `progress.txt` | Append-only learnings log |
| `.ralph-phase` | Phase indicator file — tells agents which mode they're in |

## Patterns

- Simple mode: Each iteration = one AI instance, self-contained
- Harness mode: Each story = 2-6+ AI instances (contract rounds + build + evaluate + retries)
- Memory persists via git history, `progress.txt`, and `prd.json`
- Stories should be small enough to complete in one context window
- Contract negotiation prevents scope creep — "done" is defined before work starts
- Evaluator is tuned to be skeptical — must catch what Generator misses
- Never modify a locked contract.json — this is enforced by both prompts and ralph.sh itself
