# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Ralph is an **autonomous AI agent loop system** that runs AI coding tools (Amp or Claude Code) repeatedly until all PRD items are complete. Each iteration spawns a **fresh AI instance** with clean context. Memory persists via git history, `progress.txt`, and `prd.json`.

Key insight: Ralph solves the context window problem by breaking large features into small, completable stories that execute sequentially across multiple fresh instances.

## Essential Commands

### Running Ralph

```bash
# Run with Amp (default)
./ralph.sh [max_iterations]

# Run with Claude Code
./ralph.sh --tool claude [max_iterations]

# Default is 10 iterations
```

### Flowchart Development

```bash
# Run the interactive visualization locally
cd flowchart
npm install
npm run dev

# Build for production
npm run build

# Lint
npm run lint
```

## Architecture & Key Files

### Core Loop Files

| File | Purpose | Modified By |
|------|---------|-------------|
| `ralph.sh` | Bash loop that spawns fresh AI instances | Manual |
| `prompt.md` | Instructions template for Amp instances | Manual |
| `CLAUDE.md` | Instructions template for Claude Code instances | Manual (this file) |
| `prd.json` | User stories with `passes` status - the task list | AI agent updates `passes` field after each story |
| `progress.txt` | Append-only learnings log | AI agent appends after each story |

### Supporting Files

| File | Purpose |
|------|---------|
| `prd.json.example` | Example PRD format for reference |
| `AGENTS.md` | Repository-level patterns and learnings |
| `skills/prd/SKILL.md` | Skill for generating PRDs from requirements |
| `skills/ralph/SKILL.md` | Skill for converting markdown PRDs to prd.json |
| `flowchart/` | Interactive React Flow visualization of how Ralph works |

### Archive System

When starting a new feature (different `branchName` in prd.json), Ralph automatically archives the previous run to `archive/YYYY-MM-DD-feature-name/` containing the old `prd.json` and `progress.txt`.

## How Ralph Works

### The Fresh Context Model

**Critical concept:** Each iteration = fresh AI instance with no memory of previous iterations.

Memory ONLY persists via:
- **Git history** - Commits from previous iterations
- **progress.txt** - Learnings and context (append-only)
- **prd.json** - Which stories are done (`passes: true/false`)

### Iteration Workflow

When `ralph.sh` runs, each iteration:

1. Reads `prd.json` to find the next incomplete story (`passes: false`)
2. Reads `progress.txt` for context (especially "Codebase Patterns" section)
3. Checks current git branch matches `prd.json` `branchName`
4. Implements **one single story**
5. Runs quality checks (typecheck, lint, tests)
6. Commits changes: `feat: [Story ID] - [Story Title]`
7. Updates `prd.json` to set `passes: true` for completed story
8. Appends learnings to `progress.txt`
9. If all stories pass, outputs `<promise>COMPLETE</promise>` to exit loop

### Story Size Guidelines

**Rule:** Each story must be completable in ONE context window.

**Right-sized:**
- Add a database column and migration
- Add a UI component to an existing page
- Update a server action with new logic
- Add a filter dropdown to a list

**Too big (split these):**
- "Build the entire dashboard"
- "Add authentication"
- "Refactor the API"

### Story Ordering Rules

Stories execute in priority order. Dependencies must come first:

1. Schema/database changes (migrations)
2. Server actions / backend logic
3. UI components using the backend
4. Dashboard/summary views aggregating data

## PRD Format (prd.json)

```json
{
  "project": "ProjectName",
  "branchName": "ralph/feature-name-kebab-case",
  "description": "Feature description",
  "userStories": [
    {
      "id": "US-001",
      "title": "Story title",
      "description": "As a [user], I want [feature] so that [benefit]",
      "acceptanceCriteria": [
        "Verifiable criterion 1",
        "Verifiable criterion 2",
        "Typecheck passes",
        "Verify in browser using dev-browser skill"  // For UI stories
      ],
      "priority": 1,
      "passes": false,
      "notes": ""
    }
  ]
}
```

### Acceptance Criteria Rules

**Always include:**
- `"Typecheck passes"` - Every story
- `"Tests pass"` - Stories with testable logic
- `"Verify in browser using dev-browser skill"` - UI stories (mandatory for frontend work)

**Must be verifiable, not vague:**
- ✅ "Filter dropdown has options: All, Active, Completed"
- ✅ "Clicking delete shows confirmation dialog"
- ❌ "Works correctly"
- ❌ "Good UX"

## Progress Log Format (progress.txt)

The progress.txt file has two critical sections:

### 1. Codebase Patterns (at top)

Consolidated reusable patterns that ALL future iterations need:

```
## Codebase Patterns
- Use `sql<number>` template for aggregations
- Always use `IF NOT EXISTS` for migrations
- Export types from actions.ts for UI components
```

### 2. Iteration Logs (chronological)

```
## [Date/Time] - [Story ID]
- What was implemented
- Files changed
- **Learnings for future iterations:**
  - Patterns discovered
  - Gotchas encountered
  - Useful context
---
```

## Skills System

Ralph includes two skills for Amp/Claude Code:

### /prd skill
Generate a Product Requirements Document from natural language description.

**Workflow:**
1. Describe feature to AI
2. AI asks clarifying questions
3. AI generates structured PRD
4. Saves to `tasks/prd-[feature-name].md`

### /ralph skill
Convert markdown PRD to `prd.json` format for autonomous execution.

**Workflow:**
1. Load skill: "Load the ralph skill and convert tasks/prd-X.md to prd.json"
2. AI analyzes PRD and splits into right-sized stories
3. AI generates `prd.json` with proper dependency ordering
4. Archives previous run if different `branchName`

## Critical Success Factors

### 1. Small Stories
If a story can't be completed in one context window, Ralph produces broken code. Size stories carefully.

### 2. Feedback Loops Required
Ralph ONLY works with automated quality checks:
- Typecheck catches type errors
- Tests verify behavior
- CI must stay green (broken code compounds)

### 3. AGENTS.md Updates
After each iteration, update relevant AGENTS.md files with:
- Patterns discovered ("this codebase uses X for Y")
- Gotchas ("don't forget to update Z when changing W")
- Useful context ("settings panel is in component X")

AI coding tools automatically read AGENTS.md, so learnings propagate.

### 4. Browser Verification for UI
Frontend stories MUST include "Verify in browser using dev-browser skill" in acceptance criteria. Ralph uses browser automation to verify UI changes actually work.

## Debugging Ralph Runs

```bash
# Check which stories are done
cat prd.json | jq '.userStories[] | {id, title, passes}'

# View learnings from iterations
cat progress.txt

# Check recent commits
git log --oneline -10

# See current branch
git branch
```

## Common Patterns

### Starting a New Feature

1. Generate PRD: Use `/prd` skill or manually create markdown PRD
2. Convert to JSON: Use `/ralph` skill to create `prd.json`
3. Run Ralph: `./ralph.sh --tool claude 10`
4. Monitor: Ralph commits after each successful story

### Customizing for Your Project

After copying Ralph to your project, customize `CLAUDE.md` (this file) with:
- Project-specific quality check commands
- Codebase conventions and patterns
- Common gotchas for your stack
- Where to find key components

### When Ralph Gets Stuck

**Story too big:** Split into smaller stories in prd.json
**Quality checks failing:** Fix manually, commit, Ralph continues from that point
**Wrong branch:** Ralph auto-creates/checks out from `prd.json` `branchName`
**Context too large:** Story size is too big, needs splitting

## Implementation Guidelines for Autonomous Agents

When working as Ralph (invoked via `ralph.sh --tool claude`):

### Task-Based Workflow (Recommended)

Ralph now uses Claude Code's Task system for hierarchical task tracking. Each user story becomes a parent task, and each acceptance criterion becomes a child task.

**Iteration Workflow:**

1. **Check for tasks** - Use `TaskList` to see if tasks exist
   - If no tasks exist: Convert prd.json to tasks using `node scripts/prd-to-tasks.js prd.json`
   - Tasks are automatically created when Ralph starts (if Node.js available)

2. **Find next task** - Use `TaskList` to find next pending child task
   - Look for status=`pending` and type=`child` in metadata
   - Check `blockedBy` is empty (no blocking dependencies)
   - Pick the lowest task ID if multiple available

3. **Read context**
   - Read `progress.txt` for Codebase Patterns section
   - Check git branch matches PRD `branchName`
   - Review task description and metadata for requirements

4. **Mark task in progress** - `TaskUpdate` with `status: in_progress`

5. **Implement the criterion**
   - Work on ONLY this one acceptance criterion
   - Follow existing code patterns
   - Keep changes focused and minimal

6. **Run quality checks**
   - Typecheck (if criterion requires it)
   - Tests (if criterion requires it)
   - Browser verification (if criterion requires it)
   - Must pass before committing

7. **Commit changes** - `feat: [Story ID-ACn] - [Criterion description]`
   - Example: `feat: [US-001-AC1] - Add status column to database`
   - Include Co-Authored-By trailer

8. **Mark task completed** - `TaskUpdate` with `status: completed`

9. **Check if story complete**
   - Get parent task ID from child metadata (`parentTaskId`)
   - Use `TaskList` to check all sibling child tasks (same `parentTaskId`)
   - If ALL siblings are `completed`:
     - Mark parent task `completed`
     - Update prd.json: set story `passes: true`
     - Commit prd.json update

10. **Append to progress.txt**
    - What was implemented (the criterion)
    - Files changed
    - Learnings for future iterations

11. **Check completion**
    - Use `TaskList` to get all tasks
    - Filter for type=`parent` in metadata
    - If ALL parent tasks have status=`completed`:
      - Output `<promise>COMPLETE</promise>`
    - Otherwise, continue (next iteration picks up next task)

### Fallback: prd.json-Only Mode

If task system is unavailable (no Claude Code, or `CLAUDE_CODE_ENABLE_TASKS=false`):

1. **Read prd.json** - Find highest priority story where `passes: false`
2. **Read progress.txt Codebase Patterns** - Essential context from previous iterations
3. **Check branch** - Ensure on correct branch from `prd.json`
4. **Work on ONE story only** - Complete it fully, don't start the next
5. **Run quality checks** - Must pass before committing
6. **Update AGENTS.md** if you discover reusable patterns
7. **Commit** - `feat: [Story ID] - [Story Title]` (with Co-Authored-By trailer)
8. **Update prd.json** - Set `passes: true` for completed story
9. **Append to progress.txt** - What changed + learnings
10. **Check completion** - If all `passes: true`, output `<promise>COMPLETE</promise>`

### Key Differences: Task vs prd.json Mode

**Task Mode Benefits:**
- Granular tracking: Each acceptance criterion is a separate task
- Dependency enforcement: Tasks are blocked automatically based on dependencies
- Progress visibility: Can see which criteria are done vs pending
- Collaboration: Multiple sessions can share same task list via `CLAUDE_CODE_TASK_LIST_ID`

**When to Use Each:**
- **Task mode**: Default for Claude Code (automatically enabled)
- **prd.json mode**: Fallback when tasks unavailable, or for Amp compatibility

## Deployment

The `flowchart/` visualization auto-deploys to GitHub Pages via `.github/workflows/deploy.yml` when pushing to main.
