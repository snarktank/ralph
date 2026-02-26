---
name: ralph
description: "Convert PRDs to prp.json (or prd.json) format for the Ralph autonomous agent system. Use when you have an existing PRD and need to convert it to Ralph's JSON format. Triggers on: convert this prd, turn this into ralph format, create prp.json from this, ralph json."
user-invocable: true
---

# Ralph PRP Converter

Converts existing PRDs to the `prp.json` format that Ralph uses for autonomous execution.

> **Filename:** Output to `prp.json` (preferred). Ralph also supports `prd.json` as a legacy fallback — agents and scripts check for `prp.json` first, then `prd.json`.

---

## The Job

Take a PRD (markdown file or text) and convert it to `prp.json` in your ralph directory.

---

## Output Format

```json
{
  "project": "[Project Name]",
  "branchName": "ralph/[feature-name-kebab-case]",
  "description": "[Feature description from PRD title/intro]",
  "version": 1,
  "previousVersion": null,

  "constraints": [
    "Constraint 1 from PRD",
    "Constraint 2 from PRD"
  ],

  "nonGoals": [
    "Non-goal 1 from PRD",
    "Non-goal 2 from PRD"
  ],

  "glossary": {
    "term": "Definition from PRD glossary"
  },

  "userStories": [
    {
      "id": "US-001",
      "title": "[Story title]",
      "description": "As a [user], I want [feature] so that [benefit]",
      "acceptanceCriteria": [
        "Testable assertion 1",
        "Testable assertion 2",
        "Typecheck passes"
      ],
      "verificationCommands": [
        "npx tsc --noEmit"
      ],
      "dependsOn": [],
      "priority": 1,
      "passes": false,
      "notes": "",
      "context": {
        "relevantFiles": ["src/path/to/file.ts"],
        "hints": ["Implementation guidance"],
        "examples": ["// Code snippet showing pattern to follow"]
      }
    }
  ]
}
```

---

## Story Size: The Number One Rule

**Each story must be completable in ONE Ralph iteration (one context window).**

Ralph spawns a fresh agent instance per iteration with no memory of previous work. If a story is too big, the LLM runs out of context before finishing and produces broken code.

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

For explicit dependency control, use the `dependsOn` field — an array of story IDs that must have `passes: true` before this story can be claimed. This complements priority ordering by enforcing hard prerequisites, which is especially useful for parallel agents where multiple stories run concurrently.

**If a story requires another story to be completed first, list the prerequisite IDs in `dependsOn`.** Stories without dependencies use an empty array `[]`.

**Correct order:**
1. Schema/database changes (migrations)
2. Server actions / backend logic
3. UI components that use the backend — `dependsOn: ["US-001"]`
4. Dashboard/summary views that aggregate data — `dependsOn: ["US-002", "US-003"]`

**Wrong order:**
1. UI component (depends on schema that does not exist yet)
2. Schema change

---

## Acceptance Criteria: Must Be Testable Assertions

Each criterion must be a **testable assertion** that an agent can check pass/fail, not a description or aspiration.

### Good criteria (testable assertions):
- "Add `status` column to tasks table with default 'pending'"
- "Filter dropdown has options: All, Active, Completed"
- "Clicking delete shows confirmation dialog"
- "TaskCard renders Badge with color: red for high, yellow for medium"
- "Typecheck passes"
- "Tests pass"

### Bad criteria (vague or descriptive):
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

### Project-Level Fields

1. **project**: Project name from PRD title
2. **branchName**: Derive from feature name, kebab-case, prefixed with `ralph/`
3. **description**: Feature description from PRD intro
4. **version**: Integer starting at 1 (see Version Evolution below)
5. **previousVersion**: `null` for new specs (see Version Evolution below)
6. **constraints**: Extract from Constraints section of the PRD. If no explicit Constraints section, extract architectural decisions from Technical Considerations (e.g., "Use X for Y", "Follow pattern Z"). Array of strings.
7. **nonGoals**: Extract from Non-Goals section. Array of strings.
8. **glossary**: Extract from Glossary section. If no Glossary section but domain terms are used, generate definitions from context. Object with string keys/values. Omit if empty.

### Story-Level Fields

1. **Each user story becomes one JSON entry**
2. **IDs**: Sequential (US-001, US-002, etc.)
3. **Priority**: Based on dependency order, then document order
4. **dependsOn**: If a story requires another story first, list prerequisite IDs in `dependsOn`. Use `[]` for stories with no prerequisites.
5. **All stories**: `passes: false` and empty `notes`
6. **Always add**: "Typecheck passes" to every story's acceptance criteria
7. **verificationCommands**: Extract from acceptance criteria that reference commands. Always include at least the project's typecheck command (e.g., `npx tsc --noEmit`). If the PRD mentions specific test commands, include those. Array of strings. Omit if only typecheck.
8. **context.relevantFiles**: Extract file references from the PRD's Design/Technical Considerations sections, or from story descriptions mentioning specific files. Array of strings.
9. **context.hints**: Extract implementation notes, approach guidance, or "reuse X" suggestions from the PRD. Array of strings.
10. **context.examples**: Extract code snippets from the PRD that show patterns to follow. Array of strings (each string is a code snippet).
11. **context**: Omit the entire context object if all three sub-fields would be empty.

---

## Version Evolution

### New Feature (version 1)

For a brand new feature with no existing spec:
- Set `version: 1`
- Set `previousVersion: null`
- Standard conversion — all stories `passes: false`

### Feature Revision (version 2+)

When converting a PRD that revises an existing feature:

1. **Check for existing spec**: Look for `prp.json` (or `prd.json`) in the ralph directory
2. **Compare branch names**: If the existing spec has the same `branchName` (same feature), this is a revision
3. **Archive the current spec**:
   - Read the `version` from the existing spec (default 1 if absent)
   - Create archive: `archive/{feature-name}/v{N}.prp.json`
   - Archive progress file: `archive/{feature-name}/v{N}.progress.txt`
4. **Write the new spec**:
   - Set `version` to previous version + 1
   - Set `previousVersion` to the archive path (e.g., `"archive/task-priority/v1.prp.json"`)
5. **Diff stories** between old and new:
   - Story ID exists in new but not old → new story, `passes: false`
   - Story ID exists in both, `acceptanceCriteria` unchanged → carry over `passes: true` from old
   - Story ID exists in both, `acceptanceCriteria` changed → modified, `passes: false`
   - Story ID exists in old but not new → removed (not in new spec, no action needed)

### Different Feature

If the existing spec has a **different** `branchName`, this is a new feature — archive the old spec using the standard archive flow (date-based), then write a fresh v1 spec.

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

## Constraints
- Use drizzle ORM for database changes
- Use shadcn/ui for all new components

## Non-Goals
- No status-based notifications
- No automatic status transitions

## Requirements
- Toggle between pending/in-progress/done on task list
- Filter list by status
- Show status badge on each task
- Persist status in database
```

**Output prp.json:**
```json
{
  "project": "TaskApp",
  "branchName": "ralph/task-status",
  "description": "Task Status Feature - Track task progress with status indicators",
  "version": 1,
  "previousVersion": null,
  "constraints": [
    "Use drizzle ORM for database changes",
    "Use shadcn/ui for all new components"
  ],
  "nonGoals": [
    "No status-based notifications",
    "No automatic status transitions"
  ],
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
      "verificationCommands": [
        "npx tsc --noEmit"
      ],
      "dependsOn": [],
      "priority": 1,
      "passes": false,
      "notes": "",
      "context": {
        "relevantFiles": [
          "src/db/schema.ts"
        ],
        "hints": [
          "Follow existing column patterns in schema.ts"
        ],
        "examples": []
      }
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
      "dependsOn": ["US-001"],
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
        "Verify in browser using dev-browser skill"
      ],
      "dependsOn": ["US-001"],
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
        "Verify in browser using dev-browser skill"
      ],
      "dependsOn": ["US-002", "US-003"],
      "priority": 4,
      "passes": false,
      "notes": ""
    }
  ]
}
```

---

## Archiving Previous Runs

**Before writing a new prp.json, check if there is an existing one from a different feature:**

1. Read the current `prp.json` (or `prd.json`) if it exists
2. Check if `branchName` differs from the new feature's branch name
3. If **same feature** (same branchName): This is a version evolution — see Version Evolution above
4. If **different feature** AND `progress.txt` has content beyond the header:
   - Create archive folder: `archive/YYYY-MM-DD-feature-name/`
   - Copy current spec and `progress.txt` to archive
   - Reset `progress.txt` with fresh header

**The ralph.sh script handles this automatically** when you run it, but if you are manually updating the spec between runs, archive first.

---

## Backward Compatibility

All new fields are **optional**. The contract:
- If `prp.json` exists, use it. If not, fall back to `prd.json`. Both formats are identical.
- If new fields (`constraints`, `nonGoals`, `glossary`, `version`, `context`, `verificationCommands`) exist, agents use them. If not, current behavior is unchanged.
- The jq story-selection queries are UNCHANGED — they only touch `passes`, `claimed_by`, `dependsOn`, `priority`.
- Existing `prd.json` files from previous runs continue to work.
- `version` defaults to 1 if absent. `previousVersion` defaults to null.

---

## Checklist Before Saving

Before writing prp.json, verify:

- [ ] **Previous run archived** (if spec exists with different branchName, archive it first)
- [ ] **Version evolution handled** (if same branchName, archive and increment version)
- [ ] Each story is completable in one iteration (small enough)
- [ ] Stories are ordered by dependency (schema to backend to UI)
- [ ] Stories with prerequisites have correct `dependsOn` arrays
- [ ] Every story has "Typecheck passes" as criterion
- [ ] UI stories have "Verify in browser using dev-browser skill" as criterion
- [ ] Acceptance criteria are **testable assertions** (not vague)
- [ ] No story depends on a later story
- [ ] `constraints` captures architectural decisions from the PRD
- [ ] `nonGoals` captures scope boundaries from the PRD
- [ ] `context` blocks included for stories with relevant files/hints in the PRD
