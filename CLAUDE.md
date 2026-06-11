# Ralph Agent Instructions

You are an autonomous coding agent working on a software project.

## Your Task

1. Read the PRD at `prd.json` (in the same directory as this file)
2. Read the progress log at `progress.txt` (check Codebase Patterns section first)
3. Check you're on the correct branch from PRD `branchName`. If not, check it out or create from main.
4. **Select the active item** (see "Item Selection & Skip-Then-Block" below):
   the highest-priority item where `passes:false` **and** `blocked:false`.
5. **Increment that item's `attempts` by 1** in `prd.json` and save, *before* you
   start implementing it. Do this every iteration you work the item.
6. Implement that single item.
7. Run quality checks (e.g., typecheck, lint, test - use whatever your project requires).
8. Update CLAUDE.md files if you discover reusable patterns (see below).
9. If checks pass, commit ALL changes with message: `feat: [Story ID] - [Story Title]`,
   then set `passes: true` for the completed item in `prd.json`.
10. **If checks do NOT pass and `attempts` has reached K** (`meta.maxAttempts`,
    default 3): set the item's `blocked: true` and write a one-line
    `blockReason` diagnosing why it's stuck. Do **not** keep retrying it — the
    next iteration will move on to the next eligible item.
11. Append your progress to `progress.txt`.

## Item Selection & Skip-Then-Block

The loop must not starve its own queue by burning every iteration on one hard
item. The rule:

- **Eligible = `passes:false` AND `blocked:false`.** Always pick the
  highest-priority eligible item as the active task.
- **A `blocked:true` item is never selected** while any eligible item remains
  (C.3). Blocked items are skipped, not retried.
- **Attempt accounting (C.1):** increment the active item's `attempts` each
  iteration you work it (step 5). `attempts` counts how many iterations have
  been spent on it.
- **Self-block at the ceiling (C.2):** when you've worked an item and it still
  can't pass after `attempts` reaches **K** (`meta.maxAttempts`, default 3), set
  `blocked:true` and write a concrete one-line `blockReason` (e.g.
  `"migration fails: column already exists; needs manual reconciliation"`).
  Then stop working it and let the loop proceed.
- **Honesty:** `blockReason` is the message your future self / the human reads
  in the morning rundown. Make it a real diagnosis, not "couldn't do it".

The `audit` rundown has a detective backstop: if it finds an item with
`attempts >= K` that is *not* `blocked`, it flags it. Don't rely on the
backstop — self-block correctly.

## prd.json Item Schema

Items use the `{meta, items}` envelope. Each item:

| Field | Type | Default | Meaning |
|---|---|---|---|
| `id` | string | — | Stable item id (e.g. `C.2`). |
| `block` / `blockName` | string | — | Grouping for the audit. |
| `description` | string | — | Verifiable statement of the work. |
| `passes` | bool | `false` | True once the item is green and committed. |
| `attempts` | int | `0` | Iterations spent on this item (you increment). |
| `blocked` | bool | `false` | Set true when you give up after K attempts. |
| `blockReason` | string | `""` | One-line diagnosis, written when `blocked` flips true. |

`attempts` / `blocked` / `blockReason` are **optional** — an item without them is
read as `0 / false / ""`. `meta.maxAttempts` holds K (default 3). Never renumber
or delete existing items; only flip their fields.

## Progress Report Format

APPEND to progress.txt (never replace, always append):
```
## [Date/Time] - [Story ID]
- What was implemented
- Files changed
- **Learnings for future iterations:**
  - Patterns discovered (e.g., "this codebase uses X for Y")
  - Gotchas encountered (e.g., "don't forget to update Z when changing W")
  - Useful context (e.g., "the evaluation panel is in component X")
---
```

The learnings section is critical - it helps future iterations avoid repeating mistakes and understand the codebase better.

## Consolidate Patterns

If you discover a **reusable pattern** that future iterations should know, add it to the `## Codebase Patterns` section at the TOP of progress.txt (create it if it doesn't exist). This section should consolidate the most important learnings:

```
## Codebase Patterns
- Example: Use `sql<number>` template for aggregations
- Example: Always use `IF NOT EXISTS` for migrations
- Example: Export types from actions.ts for UI components
```

Only add patterns that are **general and reusable**, not story-specific details.

## Update CLAUDE.md Files

Before committing, check if any edited files have learnings worth preserving in nearby CLAUDE.md files:

1. **Identify directories with edited files** - Look at which directories you modified
2. **Check for existing CLAUDE.md** - Look for CLAUDE.md in those directories or parent directories
3. **Add valuable learnings** - If you discovered something future developers/agents should know:
   - API patterns or conventions specific to that module
   - Gotchas or non-obvious requirements
   - Dependencies between files
   - Testing approaches for that area
   - Configuration or environment requirements

**Examples of good CLAUDE.md additions:**
- "When modifying X, also update Y to keep them in sync"
- "This module uses pattern Z for all API calls"
- "Tests require the dev server running on PORT 3000"
- "Field names must match the template exactly"

**Do NOT add:**
- Story-specific implementation details
- Temporary debugging notes
- Information already in progress.txt

Only update CLAUDE.md if you have **genuinely reusable knowledge** that would help future work in that directory.

## Quality Requirements

- ALL commits must pass your project's quality checks (typecheck, lint, test)
- Do NOT commit broken code
- Keep changes focused and minimal
- Follow existing code patterns

## Test-Before-Implementation Discipline

A green `passes:true` is only trustworthy if the tests behind it are real. The
loop commits on green, so a thin test that asserts nothing lets an item
self-certify as done while the behaviour is broken. To guard against this
(there is deliberately **no** separate gating machinery — the discipline lives
here):

- For any item with testable logic, **write the test first** and watch it fail
  for the right reason before implementing. A test that passes before you write
  the code is testing nothing.
- Assert on **behaviour and outputs**, not on the fact that a function ran.
  Cover the acceptance criterion's concrete claim (the value, the edge case,
  the error), not a tautology.
- Never weaken or delete a test to make an item pass. If the test is wrong,
  fix the test deliberately and say so in `progress.txt`.
- If you cannot write an honest passing test within K attempts, **block the
  item** with a `blockReason` saying so — that is the correct outcome, not a
  hollow green.

## Browser Testing (If Available)

For any story that changes UI, verify it works in the browser if you have browser testing tools configured (e.g., via MCP):

1. Navigate to the relevant page
2. Verify the UI changes work as expected
3. Take a screenshot if helpful for the progress log

If no browser tools are available, note in your progress report that manual browser verification is needed.

## Stop Condition

After completing an item, check whether any **eligible** item remains —
`passes:false` AND `blocked:false`.

If **no eligible item remains** (every item is either `passes:true` or
`blocked:true`), the run is done. Reply with:
<promise>COMPLETE</promise>

Blocked items do **not** prevent completion — they are reported, with reasons,
in the `audit` rundown for the human to triage. Do not loop forever on them.

If at least one eligible item remains, end your response normally (another
iteration will pick up the next eligible item).

## Important

- Work on ONE item per iteration
- Increment `attempts` before implementing; self-block at K (`meta.maxAttempts`)
- Never select a `blocked:true` item while an eligible item remains
- Commit frequently
- Keep CI green
- Read the Codebase Patterns section in progress.txt before starting
- For the morning report, run the `audit` rundown (`python3 project_audit.py`)
