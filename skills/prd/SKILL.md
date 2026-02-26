---
name: prd
description: "Generate a Product Requirements Document (PRD) for a new feature. Use when planning a feature, starting a new project, or when asked to create a PRD. Triggers on: create a prd, write prd for, plan this feature, requirements for, spec out."
user-invocable: true
---

# PRD Generator

Create detailed Product Requirements Documents that are clear, actionable, and suitable for implementation.

---

## The Job

1. Receive a feature description from the user
2. Ask 3-5 essential clarifying questions (with lettered options)
3. Generate a structured PRD based on answers
4. Save to `tasks/prd-[feature-name].md`

**Important:** Do NOT start implementing. Just create the PRD.

---

## Step 1: Clarifying Questions

Ask only critical questions where the initial prompt is ambiguous. Focus on:

- **Problem/Goal:** What problem does this solve?
- **Core Functionality:** What are the key actions?
- **Scope/Boundaries:** What should it NOT do?
- **Success Criteria:** How do we know it's done?

### Format Questions Like This:

```
1. What is the primary goal of this feature?
   A. Improve user onboarding experience
   B. Increase user retention
   C. Reduce support burden
   D. Other: [please specify]

2. Who is the target user?
   A. New users only
   B. Existing users only
   C. All users
   D. Admin users only

3. What is the scope?
   A. Minimal viable version
   B. Full-featured implementation
   C. Just the backend/API
   D. Just the UI
```

This lets users respond with "1A, 2C, 3B" for quick iteration. Remember to indent the options.

---

## Step 2: PRD Structure

Generate the PRD with these sections:

### 1. Introduction/Overview
Brief description of the feature and the problem it solves.

### 2. Goals
Specific, measurable objectives (bullet list).

### 3. User Stories
Each story needs:
- **Title:** Short descriptive name
- **Description:** "As a [user], I want [feature] so that [benefit]"
- **Acceptance Criteria:** Verifiable checklist of what "done" means
- **Implementation Context** (optional): Relevant files, hints, and code pattern examples
- **Verification Commands** (optional): Shell commands to validate the story

Each story should be small enough to implement in one focused session.

**Format:**
```markdown
### US-001: [Title]
**Description:** As a [user], I want [feature] so that [benefit].

**Acceptance Criteria:**
- [ ] Specific testable assertion (not a description)
- [ ] Another testable assertion
- [ ] Typecheck/lint passes
- [ ] **[UI stories only]** Verify in browser using dev-browser skill

**Verification Commands:** (optional)
- `npx tsc --noEmit`
- `npm test -- --grep "priority"`

**Implementation Context:** (optional)
- **Relevant files:** `src/components/TaskCard.tsx`, `src/components/ui/Badge.tsx`
- **Hints:** Reuse existing Badge component — it already supports color variants
- **Pattern to follow:**
  ```tsx
  <Badge color={statusColors[task.status]}>{task.status}</Badge>
  ```
```

**Acceptance Criteria Rules:**
- Each criterion must be a **testable assertion**, not a description. An agent must be able to check pass/fail.
- Good: "TaskCard renders a Badge with color prop: red for high, yellow for medium, gray for low"
- Bad: "Priority is displayed nicely" or "Works correctly"
- **For any story with UI changes:** Always include "Verify in browser using dev-browser skill" as acceptance criteria.

**Verification Commands:**
- Include explicit shell commands that validate the story (e.g., `npx tsc --noEmit`, `npm test -- --grep "feature"`)
- These give agents a concrete way to verify their work beyond manual inspection

**Implementation Context:**
- **Relevant files**: List specific files the implementor should read or modify
- **Hints**: Brief guidance on approach — what to reuse, what pattern to follow
- **Code examples**: Show existing patterns from the codebase that the new code should match

### 4. Functional Requirements
Numbered list of specific functionalities:
- "FR-1: The system must allow users to..."
- "FR-2: When a user clicks X, the system must..."

Be explicit and unambiguous.

### 5. Non-Goals (Out of Scope)
What this feature will NOT include. **This section is REQUIRED** — explicit scope boundaries prevent agents from gold-plating or building unrequested features.

### 6. Constraints
Architectural decisions and technology requirements that agents MUST follow. This captures intent, not just current state.

Examples:
- "Use drizzle ORM for all database operations"
- "Use server actions for mutations, not API routes"
- "All new components must use shadcn/ui"
- "Follow existing naming conventions in src/actions/"

### 7. Glossary (Optional)
Define domain-specific terms used in the PRD. Helps agents (and junior developers) understand the vocabulary without guessing.

Format:
- **term**: Definition

### 8. Design Considerations (Optional)
- UI/UX requirements
- Link to mockups if available
- Relevant existing components to reuse

### 9. Technical Considerations (Optional)
- Known constraints or dependencies
- Integration points with existing systems
- Performance requirements

### 10. Success Metrics
How will success be measured?
- "Reduce time to complete X by 50%"
- "Increase conversion rate by 10%"

### 11. Open Questions
Remaining questions or areas needing clarification.

---

## Writing for Junior Developers

The PRD reader may be a junior developer or AI agent. Therefore:

- Be explicit and unambiguous
- Avoid jargon or explain it
- Provide enough detail to understand purpose and core logic
- Number requirements for easy reference
- Use concrete examples where helpful

---

## Output

- **Format:** Markdown (`.md`)
- **Location:** `tasks/`
- **Filename:** `prd-[feature-name].md` (kebab-case)

---

## Example PRD

```markdown
# PRD: Task Priority System

## Introduction

Add priority levels to tasks so users can focus on what matters most. Tasks can be marked as high, medium, or low priority, with visual indicators and filtering to help users manage their workload effectively.

## Goals

- Allow assigning priority (high/medium/low) to any task
- Provide clear visual differentiation between priority levels
- Enable filtering and sorting by priority
- Default new tasks to medium priority

## User Stories

### US-001: Add priority field to database
**Description:** As a developer, I need to store task priority so it persists across sessions.

**Acceptance Criteria:**
- [ ] Add priority column to tasks table: 'high' | 'medium' | 'low' (default 'medium')
- [ ] Generate and run migration successfully
- [ ] Typecheck passes

### US-002: Display priority indicator on task cards
**Description:** As a user, I want to see task priority at a glance so I know what needs attention first.

**Acceptance Criteria:**
- [ ] Each task card shows colored priority badge (red=high, yellow=medium, gray=low)
- [ ] Priority visible without hovering or clicking
- [ ] Typecheck passes
- [ ] Verify in browser using dev-browser skill

### US-003: Add priority selector to task edit
**Description:** As a user, I want to change a task's priority when editing it.

**Acceptance Criteria:**
- [ ] Priority dropdown in task edit modal
- [ ] Shows current priority as selected
- [ ] Saves immediately on selection change
- [ ] Typecheck passes
- [ ] Verify in browser using dev-browser skill

### US-004: Filter tasks by priority
**Description:** As a user, I want to filter the task list to see only high-priority items when I'm focused.

**Acceptance Criteria:**
- [ ] Filter dropdown with options: All | High | Medium | Low
- [ ] Filter persists in URL params
- [ ] Empty state message when no tasks match filter
- [ ] Typecheck passes
- [ ] Verify in browser using dev-browser skill

## Functional Requirements

- FR-1: Add `priority` field to tasks table ('high' | 'medium' | 'low', default 'medium')
- FR-2: Display colored priority badge on each task card
- FR-3: Include priority selector in task edit modal
- FR-4: Add priority filter dropdown to task list header
- FR-5: Sort by priority within each status column (high to medium to low)

## Non-Goals

- No priority-based notifications or reminders
- No automatic priority assignment based on due date
- No priority inheritance for subtasks

## Constraints

- Use drizzle ORM for all database operations
- Use server actions in src/actions/ for mutations, not API routes
- Use shadcn/ui for all new UI components

## Glossary

- **priority**: Task urgency level — high, medium, or low
- **badge**: Colored label component showing task metadata

## Technical Considerations

- Reuse existing badge component with color variants
- Filter state managed via URL search params
- Priority stored in database, not computed

## Success Metrics

- Users can change priority in under 2 clicks
- High-priority tasks immediately visible at top of lists
- No regression in task list performance

## Open Questions

- Should priority affect task ordering within a column?
- Should we add keyboard shortcuts for priority changes?
```

---

## Revision Mode

When updating an existing feature (the user references a prior PRD or says "add X to feature Y"):

1. Read the existing PRD for that feature
2. Note which user stories are unchanged vs new/modified
3. Generate the updated PRD as a **complete spec** (not a diff)
4. Mark unchanged stories clearly so the `/ralph` converter can carry over their status
5. Include a brief "Changes from v{N}" section at the top listing what's new

This supports PRP versioning — the `/ralph` converter will handle archiving and version numbering.

---

## Checklist

Before saving the PRD:

- [ ] Asked clarifying questions with lettered options
- [ ] Incorporated user's answers
- [ ] User stories are small and specific
- [ ] Acceptance criteria are **testable assertions** (not descriptions)
- [ ] Functional requirements are numbered and unambiguous
- [ ] Non-goals section defines clear boundaries (**required**)
- [ ] Constraints section captures architectural decisions
- [ ] UI stories have "Verify in browser using dev-browser skill" as criterion
- [ ] Verification commands included where applicable
- [ ] Implementation context included for non-trivial stories
- [ ] Saved to `tasks/prd-[feature-name].md`
