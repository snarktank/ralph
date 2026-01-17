# Ralph Agent Instructions

## Your Task

1. Read `prd.json`
2. Read the task-specific progress file (`progress/{task-id}.md`) provided in state
3. Work on the task specified in the state (or pick the highest-priority story where `"passes": false`)

## Size Gate (MANDATORY)

Before implementing the story, decide if it is **too large for a single iteration**.

Treat the story as TOO LARGE if ANY of the following are true:
- Requires a framework or major dependency upgrade (e.g. Vue 2 → Vue 3, Webpack → Vite)
- Would touch more than ~8 files
- Requires multiple conceptual steps or phases
- Would reasonably take a human more than ~15 minutes to complete safely

### If the story is TOO LARGE:
1. Edit `prd.json` to decompose the story into **ordered subtasks**
   - Each subtask must be small enough to complete in ONE iteration
   - Each subtask must be independently verifiable
   - Each subtask must have `"passes": false`
2. Do NOT implement any code beyond PRD edits
3. Append to the task-specific progress file (`progress/{task-id}.md`):
   - That the story was decomposed
   - The list of subtasks created
   - Which subtask will be tackled next
4. STOP the iteration immediately

### If the story is NOT too large:
Continue below.

## Implementation

4. Implement that ONE story
5. Run typecheck and tests
6. Update `prd.json`:
   - Set `"passes": true` for the finished story
7. Append learnings to the task-specific progress file (`progress/{task-id}.md`)
8. Commit with message: `feat: [ID] - [Title]`

## Progress Format (append to task progress file)

## [Date] - [Story ID]
- What was implemented
- Files changed
- **Learnings:**
  - Patterns discovered
  - Gotchas encountered

## Stop Condition

If all stories (and subtasks) have `"passes": true`, output:
<promise>COMPLETE</promise>