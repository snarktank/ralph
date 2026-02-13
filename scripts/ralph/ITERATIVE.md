# Iterative User Stories Guide

> This file is read ONLY when the current User Story has an `iterationMode` field.
> For standard (non-iterative) stories, this file is not loaded.

These rules SUPPLEMENT the standard workflow in `scripts/ralph/CLAUDE.md`. Follow all CLAUDE.md steps as usual, with the overrides below.

---

## When to Use Iterative Mode

Iterative mode is designed for tasks where **multiple passes improve quality** and **context rot would degrade a single-pass attempt**. Each session gets a fresh context, so later iterations naturally compensate for earlier ones.

Good candidates:
- Auditing/reviewing N files or components (spread across iterations)
- Complex refactors where a review pass catches mistakes from the implementation pass
- Tasks with high error surface where self-correction across sessions matters

**NOT** for stories that are simply too large — split those into multiple standard stories instead.

## Iteration Modes

The `iterationMode` field determines completion behavior:

| Mode | Completion Criterion | Use Case |
|---|---|---|
| `fixed` | `currentIteration >= maxIterations` | Predetermined batch work: audit N files, process N items, multi-pass review |

> Future modes (e.g., `adaptive` — agent decides when done) may be added. Currently only `fixed` is supported.

---

## CRITICAL: One Iteration = One Session

**This is the most important rule for iterative stories.**

You MUST:
1. Complete exactly ONE iteration of meaningful progress
2. Follow all CLAUDE.md steps (standard ordering: quality checks → commit → progress.md → update PRD)
3. **STOP and END the session** — a new session handles the next iteration

NEVER attempt multiple iterations in a single session. A new session will be spawned for the next iteration.

---

## Iteration Strategy

Before starting work, plan how to distribute effort across `maxIterations`:

1. **Read the story's `notes` field (in `scripts/ralph/prd.json`) and `scripts/ralph/progress.md`** to understand what previous iterations did
2. **Distribute work logically**: each iteration should focus on a distinct subset or aspect
3. **Reserve later iterations for review**: if `maxIterations` is 3+, the last 1-2 iterations should review and fix issues from earlier ones rather than doing net-new work
4. **Avoid redundancy**: if a previous iteration already handled something, move on

Example distribution for `maxIterations: 5` on an audit task:
- Iterations 1-3: Audit and fix files in batches
- Iteration 4: Cross-file consistency review, fix regressions
- Iteration 5: Final validation pass, edge cases, documentation

---

## Commit Format

For iterative stories, append the iteration number to the standard commit format:

```
feat: [Story ID] - [Story Title] (iteration N)
```

---

## Handling No-Change Iterations

If a review iteration finds no issues to fix:
- This is a **valid outcome** — it means earlier iterations did their job
- Update `notes` with: `"Iteration X/N: Reviewed [scope] — no changes required"`
- Commit only the `scripts/ralph/prd.json` update (the PRD change itself is the meaningful output)
- Do NOT invent unnecessary changes to satisfy "meaningful progress"

---

## Quality Check Failures

If quality checks fail during an iterative session:
- Attempt to fix the issue within the same session
- If the fix succeeds: proceed normally (commit, update progress, update PRD)
- If you cannot fix it: document the failure in `scripts/ralph/progress.md`, do **NOT** increment `currentIteration`, and end the session. The next session will retry the same iteration with fresh context.

---

## PRD Update Rules (overrides the standard PRD update step in `scripts/ralph/CLAUDE.md`)

At the END of each session, update `scripts/ralph/prd.json` for the current story:

1. Increment `currentIteration` (e.g., 0 → 1)
2. Append to `notes` using format: `"Iteration X/N: [what was done]"` — separate entries with ` | `
3. CRITICAL: Set `passes: true` ONLY when `currentIteration >= maxIterations`

NEVER set `passes: true` just because tests pass or work appears complete early. The iteration count is the **sole completion criterion** for `fixed` mode.

**Example (mid-iteration):**
```json
{
  "currentIteration": 2,
  "passes": false,
  "notes": "Iteration 1/5: Audited files A, B, C | Iteration 2/5: Fixed naming in A, reformatted B"
}
```

**Example (final iteration):**
```json
{
  "currentIteration": 5,
  "passes": true,
  "notes": "Iteration 1/5: Audited files A, B, C | ... | Iteration 5/5: Final review pass — all clean. Complete."
}
```