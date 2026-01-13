# Ralph Autonomous Agent

Execute the Ralph autonomous agent loop to implement user stories iteratively, or convert PRDs to prd.json format for execution.

---

## Primary Use: Executing Stories

When you have a `prd.json` file, use this workflow to execute the Ralph autonomous loop:

1. **Read state files** - Load prd.json and progress.txt
2. **Pick next story** - Select highest priority story where `passes: false`
3. **Implement story** - Complete the story following acceptance criteria
4. **Verify and commit** - Run quality checks, commit if passing
5. **Update state** - Mark story as complete, append progress
6. **Continue or complete** - Loop until all stories pass

See [Executing Stories](#executing-stories) section below for complete workflow.

---

## Secondary Use: Converting PRDs

Take a PRD (markdown file or text) and convert it to `prd.json` in your ralph directory.

---

## Executing Stories

When executing the Ralph loop, follow this workflow for each iteration:

### Step 1: Bootstrap Session

**If starting a new session, check for handoff state:**

1. Check for `HANDOFF.md` file in the project root
2. If exists, read it to understand previous state and where to continue
3. Read `prd.json` to see current story status
4. Read `progress.txt` (especially the `## Codebase Patterns` section at the top)
5. Check git history for recent commits: `git log --oneline -10`

**Always read these files at session start:**
- `prd.json` - Current story status
- `progress.txt` - Previous learnings and patterns
- Git branch status - Ensure on correct branch

### Step 2: Select Next Story

1. Read `prd.json` from project root (or `ralph/prd.json` if using ralph directory)
2. Find the **highest priority** user story where `passes: false`
3. If multiple stories have same priority, pick the first one (lowest ID)
4. Verify the story's acceptance criteria are clear and verifiable

**If all stories have `passes: true`, signal completion:**
```
<promise>COMPLETE</promise>
```

### Step 3: Check Branch

1. Read `branchName` from `prd.json`
2. Check current git branch: `git branch --show-current`
3. If not on correct branch:
   - Check if branch exists: `git branch -a | grep <branchName>`
   - If exists: `git checkout <branchName>`
   - If not exists: `git checkout -b <branchName>` (from main or master)

### Step 4: Implement the Story

1. **Read Codebase Patterns** - Check the `## Codebase Patterns` section in `progress.txt` first
2. **Understand requirements** - Review the story's description and acceptance criteria
3. **Implement changes** - Make code changes to satisfy all acceptance criteria
4. **Follow existing patterns** - Use patterns discovered in previous iterations
5. **Keep changes focused** - Only modify what's needed for this story

### Step 5: Run Quality Checks

Run your project's standard quality checks:

- **Typecheck**: `npm run typecheck` or `tsc --noEmit` or equivalent
- **Lint**: `npm run lint` or equivalent
- **Tests**: `npm test` or equivalent (if story has testable logic)
- **Build**: `npm run build` or equivalent (if applicable)

**Do NOT commit if checks fail.** Fix issues first.

### Step 6: Browser Verification (UI Stories Only)

For any story that changes UI:

1. Use the cursor-ide-browser MCP tools (automatically available in Cursor CLI)
2. Start dev server if not running: `npm run dev`
3. Navigate to the relevant page using `browser_navigate`
4. Verify UI changes work as expected using `browser_snapshot` and `browser_click`
5. Interact with the feature to confirm it meets acceptance criteria
6. Take a screenshot if helpful for documentation using `browser_take_screenshot`

**A frontend story is NOT complete until browser verification passes.**

### Step 7: Update AGENTS.md Files

Before committing, check if any edited files have learnings worth preserving:

1. **Identify directories with edited files** - Note which directories you modified
2. **Check for existing AGENTS.md** - Look for AGENTS.md in those directories or parent directories
3. **Add valuable learnings** - If you discovered reusable patterns:
   - API patterns or conventions specific to that module
   - Gotchas or non-obvious requirements
   - Dependencies between files
   - Testing approaches for that area
   - Configuration or environment requirements

**Examples of good AGENTS.md additions:**
- "When modifying X, also update Y to keep them in sync"
- "This module uses pattern Z for all API calls"
- "Tests require the dev server running on PORT 3000"
- "Field names must match the template exactly"

**Do NOT add:**
- Story-specific implementation details
- Temporary debugging notes
- Information already in progress.txt

Only update AGENTS.md if you have **genuinely reusable knowledge** that would help future work.

### Step 8: Commit Changes

If all quality checks pass:

1. Stage all changes: `git add .`
2. Commit with message: `feat: [Story ID] - [Story Title]`
   - Example: `feat: US-001 - Add priority field to database`
3. Verify commit succeeded: `git log -1 --oneline`

**Do NOT commit broken code.** All commits must pass quality checks.

### Step 9: Update State Files

**Update prd.json:**
1. Read current `prd.json`
2. Find the completed story by ID
3. Set `passes: true` for that story
4. Optionally add notes if there were important learnings
5. Write updated `prd.json` back

**Append to progress.txt:**
1. Read current `progress.txt`
2. Append new entry at the end:
```
## [Date/Time] - [Story ID]
- What was implemented
- Files changed
- **Learnings for future iterations:**
  - Patterns discovered (e.g., "this codebase uses X for Y")
  - Gotchas encountered (e.g., "don't forget to update Z when changing W")
  - Useful context (e.g., "the settings panel is in component X")
---
```

**Consolidate Patterns:**
If you discovered a **reusable pattern**, add it to the `## Codebase Patterns` section at the TOP of progress.txt (create it if it doesn't exist):

```
## Codebase Patterns
- Pattern 1: Description
- Pattern 2: Description
```

Only add patterns that are **general and reusable**, not story-specific details.

### Step 10: Check Completion

After updating state:

1. Read updated `prd.json`
2. Check if ALL stories have `passes: true`
3. If all complete, signal: `<promise>COMPLETE</promise>`
4. If more stories remain, end response normally (user will continue loop)

### Step 11: Context Management

**Monitor context usage throughout execution:**

- Be aware of token usage as you work
- If approaching ~90% of context window:
  1. **Commit current work** - Even if story not complete, commit what you have
  2. **Update prd.json** - Mark current story progress in `notes` field
  3. **Append to progress.txt** - Document what was done and what remains
  4. **Create HANDOFF.md** - See [Context Detection & Handoff](#context-detection--handoff) section
  5. **Signal handoff** - Tell user to start new session with handoff file

For detailed context management procedures, see the ralph-context rule when needed.

---

## Context Detection & Handoff

When context is filling up (~90% threshold), prepare for handoff:

### Handoff Preparation

1. **Commit in-progress work:**
   - Stage all changes: `git add .`
   - Commit with message: `feat: [Story ID] - [Story Title] (in progress)`
   - Document what's done vs. what remains

2. **Update prd.json:**
   - Add progress notes to current story's `notes` field
   - Document what's complete and what still needs work

3. **Append to progress.txt:**
   - Document current state
   - List files changed so far
   - Note what remains to be done

4. **Create HANDOFF.md:**
   - See the ralph-handoff rule template for structure
   - Include current story being worked on
   - List files changed
   - Document next steps needed
   - Include state snapshot

5. **Signal to user:**
   - "Context approaching limit. Handoff file created at HANDOFF.md"
   - "Please start a new session and load HANDOFF.md to continue"

### Session Bootstrap

When starting a new session after handoff:

1. **Read HANDOFF.md** - Understand previous state
2. **Read prd.json** - See current story status
3. **Read progress.txt** - Load Codebase Patterns section
4. **Check git history** - See recent commits
5. **Continue from handoff point** - Resume implementation

---

## State Persistence

Ralph persists state across sessions via:

### prd.json
- Location: Project root or `ralph/prd.json`
- Contains: Project info, branch name, user stories with `passes` status
- Updated: After each story completion

### progress.txt
- Location: Project root or `ralph/progress.txt`
- Contains: Append-only log of iterations and learnings
- Structure:
  - `## Codebase Patterns` section at top (consolidated patterns)
  - Individual iteration entries below
- Updated: After each iteration

### Git History
- Commits: Each story gets a commit when complete
- Format: `feat: [Story ID] - [Story Title]`
- Purpose: Provides code history and context for future sessions

### AGENTS.md Files
- Location: Throughout codebase (near relevant code)
- Contains: Reusable patterns and gotchas for specific modules
- Updated: When discovering reusable patterns

### HANDOFF.md (temporary)
- Location: Project root
- Created: When context limit approached
- Contains: State snapshot for session continuation
- Deleted: After successful handoff (or manually)

---

## Output Format

```json
{
  "project": "[Project Name]",
  "branchName": "ralph/[feature-name-kebab-case]",
  "description": "[Feature description from PRD title/intro]",
  "userStories": [
    {
      "id": "US-001",
      "title": "[Story title]",
      "description": "As a [user], I want [feature] so that [benefit]",
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

---

## Story Size: The Number One Rule

**Each story must be completable in ONE Ralph iteration (one context window).**

If a story is too big, the agent runs out of context before finishing and produces broken code.

### Right-sized stories:
- Add a database column and migration
- Add a UI component to an existing page
- Update a server action with new logic
- Add a filter dropdown to a list

### Too big (split these):
- "Build the entire dashboard" - Split into: schema, queries, UI components, filters
- "Add authentication" - Split into: schema, middleware, login UI, session handling
- "Refactor the API" - Split into one story per endpoint or pattern

**Rule of thumb:** If you cannot describe the change in 2-3 sentences, it is too big.

---

## Story Ordering: Dependencies First

Stories execute in priority order. Earlier stories must not depend on later ones.

**Correct order:**
1. Schema/database changes (migrations)
2. Server actions / backend logic
3. UI components that use the backend
4. Dashboard/summary views that aggregate data

**Wrong order:**
1. UI component (depends on schema that does not exist yet)
2. Schema change

---

## Acceptance Criteria: Must Be Verifiable

Each criterion must be something Ralph can CHECK, not something vague.

### Good criteria (verifiable):
- "Add `status` column to tasks table with default 'pending'"
- "Filter dropdown has options: All, Active, Completed"
- "Clicking delete shows confirmation dialog"
- "Typecheck passes"
- "Tests pass"

### Bad criteria (vague):
- "Works correctly"
- "User can do X easily"
- "Good UX"
- "Handles edge cases"

### Always include as final criterion:
```
"Typecheck passes"
```

For stories with testable logic, also include:
```
"Tests pass"
```

### For stories that change UI, also include:
```
"Verify in browser using cursor-ide-browser MCP"
```

Frontend stories are NOT complete until visually verified. Use the cursor-ide-browser MCP tools to navigate to the page, interact with the UI, and confirm changes work.

---

## Conversion Rules

1. **Each user story becomes one JSON entry**
2. **IDs**: Sequential (US-001, US-002, etc.)
3. **Priority**: Based on dependency order, then document order
4. **All stories**: `passes: false` and empty `notes`
5. **branchName**: Derive from feature name, kebab-case, prefixed with `ralph/`
6. **Always add**: "Typecheck passes" to every story's acceptance criteria

---

## Splitting Large PRDs

If a PRD has big features, split them:

**Original:**
> "Add user notification system"

**Split into:**
1. US-001: Add notifications table to database
2. US-002: Create notification service for sending notifications
3. US-003: Add notification bell icon to header
4. US-004: Create notification dropdown panel
5. US-005: Add mark-as-read functionality
6. US-006: Add notification preferences page

Each is one focused change that can be completed and verified independently.

---

## Example

**Input PRD:**
```markdown
# Task Status Feature

Add ability to mark tasks with different statuses.

## Requirements
- Toggle between pending/in-progress/done on task list
- Filter list by status
- Show status badge on each task
- Persist status in database
```

**Output prd.json:**
```json
{
  "project": "TaskApp",
  "branchName": "ralph/task-status",
  "description": "Task Status Feature - Track task progress with status indicators",
  "userStories": [
    {
      "id": "US-001",
      "title": "Add status field to tasks table",
      "description": "As a developer, I need to store task status in the database.",
      "acceptanceCriteria": [
        "Add status column: 'pending' | 'in_progress' | 'done' (default 'pending')",
        "Generate and run migration successfully",
        "Typecheck passes"
      ],
      "priority": 1,
      "passes": false,
      "notes": ""
    },
    {
      "id": "US-002",
      "title": "Display status badge on task cards",
      "description": "As a user, I want to see task status at a glance.",
      "acceptanceCriteria": [
        "Each task card shows colored status badge",
        "Badge colors: gray=pending, blue=in_progress, green=done",
        "Typecheck passes",
        "Verify in browser using cursor-ide-browser MCP"
      ],
      "priority": 2,
      "passes": false,
      "notes": ""
    },
    {
      "id": "US-003",
      "title": "Add status toggle to task list rows",
      "description": "As a user, I want to change task status directly from the list.",
      "acceptanceCriteria": [
        "Each row has status dropdown or toggle",
        "Changing status saves immediately",
        "UI updates without page refresh",
        "Typecheck passes",
        "Verify in browser using cursor-ide-browser MCP"
      ],
      "priority": 3,
      "passes": false,
      "notes": ""
    },
    {
      "id": "US-004",
      "title": "Filter tasks by status",
      "description": "As a user, I want to filter the list to see only certain statuses.",
      "acceptanceCriteria": [
        "Filter dropdown: All | Pending | In Progress | Done",
        "Filter persists in URL params",
        "Typecheck passes",
        "Verify in browser using cursor-ide-browser MCP"
      ],
      "priority": 4,
      "passes": false,
      "notes": ""
    }
  ]
}
```

---

## Archiving Previous Runs

**Before writing a new prd.json, check if there is an existing one from a different feature:**

1. Read the current `prd.json` if it exists
2. Check if `branchName` differs from the new feature's branch name
3. If different AND `progress.txt` has content beyond the header:
   - Create archive folder: `archive/YYYY-MM-DD-feature-name/`
   - Copy current `prd.json` and `progress.txt` to archive
   - Reset `progress.txt` with fresh header

**When converting a PRD to prd.json**, always check for existing runs and archive if needed.

---

## Checklist Before Saving

Before writing prd.json, verify:

- [ ] **Previous run archived** (if prd.json exists with different branchName, archive it first)
- [ ] Each story is completable in one iteration (small enough)
- [ ] Stories are ordered by dependency (schema to backend to UI)
- [ ] Every story has "Typecheck passes" as criterion
- [ ] UI stories have "Verify in browser using cursor-ide-browser MCP" as criterion
- [ ] Acceptance criteria are verifiable (not vague)
- [ ] No story depends on a later story
