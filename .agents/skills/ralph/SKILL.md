---
name: ralph
description: "Codex-native PRD-to-prd.json converter for Ralph. Use when you have a markdown PRD and want the executable Ralph task list. Invoke with $ralph."
user-invocable: true
---

# Ralph PRD Converter

Use this skill in Codex by typing `$ralph`.

Convert an existing PRD into Ralph's `prd.json` format so `./ralph.sh` can execute it.

## The Job

1. Read the markdown PRD.
2. Break it into dependency-ordered user stories.
3. Ensure each story is small enough for one Ralph iteration.
4. Save the result to `./prd.json` by default.

If the user explicitly wants a different location, use that location instead. Otherwise prefer `./prd.json`.

## Output Format

Use this shape:

```json
{
  "project": "[Project Name]",
  "branchName": "ralph/[feature-name-kebab-case]",
  "description": "[Feature description]",
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

## Story Sizing

Each story must be completable in one Ralph iteration.

Good story shapes:

- add a database column and migration
- add one UI component to an existing page
- update one server action with clear logic
- add one filter or control to an existing view

Bad story shapes:

- build an entire dashboard
- add authentication end to end
- refactor a large API surface in one step

If a story is too large, split it.

## Ordering Rules

Earlier stories must not depend on later ones.

Typical order:

1. schema and migration changes
2. backend logic
3. UI that depends on the backend
4. aggregate views and polish

## Acceptance Criteria Rules

- Every story must include `Typecheck passes`.
- Add `Tests pass` when the behavior is testable.
- Add `Verify in browser using dev-browser skill` for any UI story.
- Criteria must be verifiable, not vague.

Good criteria examples:

- `Add status column with default 'pending'`
- `Filter dropdown shows All | Active | Completed`
- `Clicking delete shows a confirmation dialog`

Bad criteria examples:

- `Works correctly`
- `Has good UX`
- `Handles edge cases`

## Save Rules

- IDs are sequential: `US-001`, `US-002`, ...
- `priority` follows dependency order, then document order
- Every story starts with `passes: false`
- `notes` starts empty
- `branchName` uses the `ralph/feature-name` convention

After writing `./prd.json`, tell the user to run `./ralph.sh`.
