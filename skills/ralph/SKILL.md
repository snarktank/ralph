---
name: ralph
description: "Generate a Product Requirements Document and Ralph-compatible prd.json for a new feature. Use when planning a feature, starting a new project, or when asked to create a PRD. Triggers on: create a prd, write prd for, plan this feature, requirements for, spec out."
user-invocable: true
---

# PRD Generator

Produce two outputs for every feature request:

1. **Human-readable PRD**: `tasks/prd-[feature-name].md`
2. **Machine-readable PRD**: `scripts/ralph/prd.json` (consumed by Ralph agent loop)

**IMPORTANT:** Do NOT start implementing. Only create the PRD and JSON.

---

## The Job

1. Receive a feature description from the user
2. Ask 3-5 clarifying questions (lettered options for quick answers)
3. **Archive previous run** if `prd.json` already exists with a different `branchName` (see Archiving below)
4. Generate the PRD markdown file
5. Generate the `prd.json` file
6. Save both files

---

## Step 1: Clarifying Questions

Ask only where the initial prompt is ambiguous. Format with lettered options so the user can reply "1A, 2C, 3B":

```
1. What is the primary goal?
   A. [option]  B. [option]  C. [option]  D. Other: [specify]

2. Who is the target user?
   A. [option]  B. [option]  C. [option]

3. What is the scope?
   A. Minimal viable version  B. Full-featured  C. Backend only  D. UI only
```

Focus on: problem/goal, core functionality, scope boundaries, success criteria.

---

## Step 2: PRD Structure (`tasks/prd-[feature-name].md`)

Required sections in order:

| # | Section | Content |
|---|---------|---------|
| 1 | **Title + Summary** | One-paragraph overview of the feature and problem it solves |
| 2 | **Goals** | 3-6 bullet points, specific and measurable |
| 3 | **User Stories** | See Story Format below |
| 4 | **Functional Requirements** | Numbered list: "FR-1: The system must..." |
| 5 | **Non-Goals** | What this feature will NOT include |
| 6 | **Technical Notes** | Rule file references, gotchas, file paths, dependencies |
| 7 | **Open Questions** | Remaining unknowns |

**CRITICAL: PRD must stay under 150 lines.** Be concise. Use tables over prose.

### Story Format

```markdown
### US-001: Short title (under 50 chars)
**Description:** As a [role], I want [feature] so that [benefit].

**Acceptance Criteria:**
- Specific verifiable statement
- Another verifiable statement
- Quality gate (e.g., "Typecheck passes", "Tests pass", "Build succeeds")

**Iteration (only if iterative):**
- Mode: fixed
- Max iterations: N
- Current iteration: 0
- Validation: `command` (optional)
```

### Story Rules

| Rule | Detail |
|------|--------|
| **Scope** | ALWAYS completable in a single context window |
| **Criteria count** | 3-6 verifiable statements per story |
| **Quality gates** | ALWAYS include at least one project-appropriate quality gate criterion (e.g., "Typecheck passes", "Tests pass", "Build succeeds") |
| **Browser verification** | UI stories ALWAYS include a browser verification criterion (e.g., "Verify in browser using available tools") |
| **ID format** | `US-NNN` (zero-padded, sequential) |
| **Title length** | Under 50 characters (must work as commit message suffix) |
| **Splitting** | NEVER use iterations for oversized stories — split them instead |

**NEVER** write vague acceptance criteria. "Works correctly" is bad. "Returns 404 when resource not found" is good.

### Standard vs Iterative Stories

| Type | When to Use | Iteration Fields |
|------|-------------|-----------------|
| **Standard** | Single-pass work (most stories) | None |
| **Iterative** | Batch/multi-pass work with known count (audit N files, process N items) | Required (see below) |

**IMPORTANT:** Iterative stories are for batch work, NOT for stories that are too big. If a story is too big, split it into multiple standard stories.

For iterative story rules, see `scripts/ralph/ITERATIVE.md`.

---

## Step 3: Ralph JSON (`scripts/ralph/prd.json`)

**CRITICAL:** This file is the machine-readable contract consumed by Ralph. Every field matters.

### Schema

```json
{
  "project": "ProjectName",
  "branchName": "ralph/feature-name",
  "description": "Brief description of the feature",
  "userStories": [
    {
      "id": "US-001",
      "title": "Short title under 50 chars",
      "description": "As a [role], I want [feature] so that [benefit].",
      "acceptanceCriteria": [
        "Verifiable statement as plain string",
        "Another verifiable statement",
        "Quality gate passes"
      ],
      "priority": 1,
      "passes": false,
      "notes": ""
    }
  ]
}
```

### Project-Level Fields

| Field | Type | Rules |
|-------|------|-------|
| `project` | string | Project name (from repo name, CLAUDE.md, or user input) |
| `branchName` | string | **ALWAYS** `ralph/[feature-name]` (kebab-case) |
| `description` | string | One-line feature summary |

### Story-Level Fields (`userStories[]`)

| Field | Type | Rules |
|-------|------|-------|
| `id` | string | `US-NNN` format, sequential |
| `title` | string | Under 50 chars, commit-message friendly |
| `description` | string | "As a [role], I want [feature] so that [benefit]" format |
| `acceptanceCriteria` | string[] | Plain strings, NOT checkbox markdown |
| `priority` | number | Execution order (1 = first). Use dependency order: if US-B depends on US-A's output, US-A must have higher priority (lower number). For independent stories, order by risk or complexity (hardest first). |
| `passes` | boolean | **ALWAYS** starts `false` |
| `notes` | string | **ALWAYS** starts `""` |

### Additional Fields for Iterative Stories

| Field | Type | Rules |
|-------|------|-------|
| `iterationMode` | string | **ALWAYS** `"fixed"` |
| `maxIterations` | number | Total passes required |
| `currentIteration` | number | **ALWAYS** starts `0` |
| `validationCommand` | string | Optional command to verify each iteration |

**CRITICAL:** `passes` becomes `true` ONLY when `currentIteration >= maxIterations`. This is enforced by Ralph, not by the PRD author.

---

## Subagent Usage Policy

When writing PRDs that may involve subagent delegation during implementation:

**Use subagents when:**
- High-volume output (test suites, log analysis, codebase-wide exploration)
- Self-contained, isolated tasks with clear start/end
- Parallel independent investigations
- Context preservation (exploration in subagent frees main context)

**Do NOT use subagents when:**
- Task requires iteration or back-and-forth
- Shared context across steps is critical
- Trivial or single-file scope
- Latency-sensitive simple changes

**Key trade-off:** Subagents isolate context but lose continuity. Use for isolated high-volume tasks only. Note this guidance in Technical Notes if relevant to the feature.

---

## Example Skeleton

```markdown
# PRD: User Notification Preferences

## Summary
Allow users to configure which notifications they receive and through which channels.

## Goals
- Users can toggle notification types on/off
- Support email and in-app channels
- Persist preferences across sessions

## User Stories

### US-001: Add preferences table to database
**Description:** As a developer, I need to store notification preferences per user.
**Acceptance Criteria:**
- Preferences table created with user_id FK
- Default preferences seeded on user creation
- Migration runs successfully
- Typecheck passes

### US-002: Build notification settings panel
**Description:** As a user, I want a settings panel to manage my notification preferences.
**Acceptance Criteria:**
- Settings panel accessible from user menu
- Toggle switches for each notification type
- Save persists changes to backend
- Verify in browser using available tools
- Typecheck passes

### US-003: Audit existing notification triggers
**Description:** As a developer, I need to verify all notification triggers respect user preferences.
**Acceptance Criteria:**
- Each trigger checked against preferences table
- Non-compliant triggers updated
- All findings documented
- Lint passes after each iteration
**Iteration:**
- Mode: fixed
- Max iterations: 3
- Current iteration: 0

## Functional Requirements
- FR-1: Preferences table with columns per notification type
- FR-2: API endpoint GET/PUT /api/preferences
- FR-3: Notification dispatch checks preferences before sending

## Non-Goals
- Push notifications (future phase)
- Notification scheduling/digest mode

## Technical Notes
- See relevant rule files for database and API conventions
- Preferences must have RLS policies
- Subagent recommended for US-003 (batch audit across multiple files)

## Open Questions
- Should preferences sync across devices in real-time?
```

---

## Archiving Previous Runs

**Before writing a new `prd.json`, check if one already exists from a different feature:**

1. Read the current `prd.json` if it exists
2. Check if `branchName` differs from the new feature's branch name
3. If different AND `progress.md` has content beyond the header:
   - Create archive folder: `archive/YYYY-MM-DD-feature-name/`
   - Copy current `prd.json`, `progress.md`, and `state.json` to the archive
   - Reset `progress.md` with a fresh header

The `ralph.sh` script handles this automatically when run (including archiving and resetting `state.json` when the branch changes), but if you are manually creating `prd.json`, archive first to avoid data loss.

---

## Checklist

Before saving, verify:

- [ ] Previous `prd.json` archived if it exists with a different `branchName`
- [ ] Asked clarifying questions with lettered options
- [ ] Incorporated user's answers
- [ ] Each story completable in one context window
- [ ] Story IDs follow `US-NNN` pattern
- [ ] Story titles under 50 characters
- [ ] 3-6 verifiable acceptance criteria per story
- [ ] Every story includes at least one project-appropriate quality gate (e.g., "Typecheck passes", "Tests pass", "Build succeeds")
- [ ] UI stories include browser verification criterion (e.g., "Verify in browser using available tools")
- [ ] Iterative stories used ONLY for batch/multi-pass work (not oversized stories)
- [ ] Iterative stories have all required fields (`iterationMode`, `maxIterations`, `currentIteration: 0`)
- [ ] Priorities reflect dependency order (dependencies first = lower number)
- [ ] Functional requirements numbered and unambiguous
- [ ] Non-goals section defines clear boundaries
- [ ] `prd.json` has `project`, `branchName`, `description` fields
- [ ] All stories in JSON have `priority`, `passes: false`, `notes: ""`
- [ ] `acceptanceCriteria` in JSON are string arrays (no checkboxes)
- [ ] Saved PRD to `tasks/prd-[feature-name].md`
- [ ] Saved JSON to `scripts/ralph/prd.json`