---
name: ralph
description: "Convert PRDs to prd.json format for the Ralph autonomous agent system. Use when you have an existing PRD and need to convert it to Ralph's JSON format. Triggers on: convert this prd, turn this into ralph format, create prd.json from this, ralph json."
user-invocable: true
---

# Ralph PRD Converter

Converts existing PRDs to the prd.json format that Ralph uses for autonomous execution.

---

## The Job

Take a PRD (markdown file or text) and convert it to `prd.json` in your ralph directory.

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
      "model": "opus",
      "failures": 0,
      "passes": false,
      "notes": ""
    }
  ]
}
```

### Model Field

The `model` field specifies which Claude model executes the story: `"opus"`, `"sonnet"`, or `"haiku"`. See the **Model Assignment** section below for assignment rules.

### Failures Field

The `failures` field (default `0`) tracks how many times a story has failed to complete. Ralph uses this for automatic model escalation - if a story fails twice, it escalates to a more capable model. See **Auto-Escalation** below.
```

---

## Story Size: The Number One Rule

**Each story must be completable in ONE Ralph iteration (one context window).**

Ralph spawns a fresh Amp instance per iteration with no memory of previous work. If a story is too big, the LLM runs out of context before finishing and produces broken code.

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
"Verify in browser using dev-browser skill"
```

Frontend stories are NOT complete until visually verified. Ralph will use the dev-browser skill to navigate to the page, interact with the UI, and confirm changes work.

---

## Conversion Rules

1. **Each user story becomes one JSON entry**
2. **IDs**: Sequential (US-001, US-002, etc.)
3. **Priority**: Based on dependency order, then document order
4. **All stories**: `passes: false`, `failures: 0`, and empty `notes`
5. **branchName**: Derive from feature name, kebab-case, prefixed with `ralph/`
6. **Always add**: "Typecheck passes" to every story's acceptance criteria
7. **Model**: Assign `opus`, `sonnet`, or `haiku` based on complexity (see Model Assignment section)

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
      "model": "opus",
      "failures": 0,
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
        "Verify in browser using dev-browser skill"
      ],
      "priority": 2,
      "model": "sonnet",
      "failures": 0,
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
        "Verify in browser using dev-browser skill"
      ],
      "priority": 3,
      "model": "sonnet",
      "failures": 0,
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
        "Verify in browser using dev-browser skill"
      ],
      "priority": 4,
      "model": "sonnet",
      "failures": 0,
      "passes": false,
      "notes": ""
    }
  ]
}
```

**Model assignment summary (cost-efficient mode):**
```
  US-001: opus    - Database migration (schema change)
  US-002: sonnet  - Display status badge (standard UI component)
  US-003: sonnet  - Add status toggle (form handling)
  US-004: sonnet  - Filter tasks (standard UI component)
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

**The ralph.sh script handles this automatically** when you run it, but if you are manually updating prd.json between runs, archive first.

---

## Model Assignment

After generating all user stories, assign a Claude model to each story based on complexity and risk. Ralph supports two modes:

### Max Quality Mode (Default)

When the user specifies `mode=max-quality` (or no mode specified), assign `"opus"` to every story. No analysis needed.

### Cost Efficient Mode

When the user specifies `mode=cost-efficient`, analyze each story and assign the appropriate model.

**Decision principle: When in doubt, round UP.**
- If you're unsure between haiku and sonnet → assign `sonnet`
- If you're unsure between sonnet and opus → assign `opus`

Only assign a lower-tier model when you're **confident** the task is straightforward enough.

#### Assign `opus` when story involves:
- Database schema changes or migrations
- Authentication/authorization logic
- Payment, billing, or financial calculations
- Security-sensitive code
- Complex state management
- New architectural patterns
- Multi-service or multi-file coordination
- First story in a critical dependency chain
- **Any uncertainty about complexity or risk**

#### Assign `sonnet` when story involves:
- Standard CRUD operations
- Typical UI components with clear patterns
- Moderate business logic with well-defined scope
- API endpoint implementation
- Form handling and validation
- Data fetching and display
- **Confident the scope is clear and bounded**

#### Assign `haiku` when story involves:
- Trivial text or copy changes
- Pure documentation updates
- Config file tweaks
- Obvious pattern repetition (copy-paste with minor changes)
- Single-file, single-function changes with no business logic
- Styling-only changes
- **Confident the task is trivially simple**

### Output Summary

After assigning models, output a summary:

```
Model assignments (cost-efficient mode):
  US-001: opus    - Database migration (schema change)
  US-002: sonnet  - Display priority badge (standard UI)
  US-003: sonnet  - Add priority selector (form handling)
  US-004: haiku   - Update button text (trivial change)
```

---

## Auto-Escalation

Ralph automatically escalates to more capable models when a story fails repeatedly. This self-corrects bad model assignments without manual intervention.

### How It Works

If a story doesn't complete in an iteration, Ralph increments its `failures` count. After 2 failures, it escalates to the next model tier:

**Starting from haiku:**
| Failures | Effective Model |
|----------|-----------------|
| 0-1 | haiku |
| 2-3 | sonnet |
| 4+ | opus |

**Starting from sonnet:**
| Failures | Effective Model |
|----------|-----------------|
| 0-1 | sonnet |
| 2+ | opus |

**Starting from opus:**
| Failures | Effective Model |
|----------|-----------------|
| Any | opus (can't escalate) |

### Example

A story assigned `haiku` that keeps failing:
- Attempt 1: haiku fails → failures: 1
- Attempt 2: haiku fails → failures: 2
- Attempt 3: sonnet (escalated) fails → failures: 3
- Attempt 4: sonnet fails → failures: 4
- Attempt 5: opus (escalated) → continues until success or max iterations

This ensures Ralph eventually uses the most capable model if simpler ones can't handle the task.

---

## Checklist Before Saving

Before writing prd.json, verify:

- [ ] **Previous run archived** (if prd.json exists with different branchName, archive it first)
- [ ] Each story is completable in one iteration (small enough)
- [ ] Stories are ordered by dependency (schema to backend to UI)
- [ ] Every story has "Typecheck passes" as criterion
- [ ] UI stories have "Verify in browser using dev-browser skill" as criterion
- [ ] Acceptance criteria are verifiable (not vague)
- [ ] No story depends on a later story
- [ ] **Every story has a model assigned** (opus, sonnet, or haiku)
