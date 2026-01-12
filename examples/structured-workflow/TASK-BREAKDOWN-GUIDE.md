# Task Breakdown Guide

A practical guide to breaking work into tasks that fit naturally into Ralph's execution model.

## The Core Principle

**Each task should be completable in one Ralph iteration (one context window).**

Ralph spawns a fresh agent instance per iteration. If a task is too big, the agent runs out of context before finishing and produces broken code.

## The Phased Approach

Break work into phases that flow naturally:

### Phase 1: Foundation
**What:** Schema, data structures, core types, migrations

**Examples:**
- Add a database column and migration
- Create a new table
- Define TypeScript types/interfaces
- Set up configuration files

**Characteristics:**
- No dependencies on other work
- Changes are structural, not behavioral
- Easy to verify (migration runs, types compile)

### Phase 2: Logic
**What:** Business rules, server actions, API endpoints, core functions

**Examples:**
- Create a service function to process data
- Add validation logic
- Implement a calculation or transformation
- Create an API endpoint

**Characteristics:**
- Depends on foundation (uses schema/types)
- Changes are behavioral, not visual
- Verifiable with tests or manual testing

### Phase 3: Interface
**What:** UI components, forms, displays, user-facing elements

**Examples:**
- Create a component to display data
- Add a form for user input
- Build a dropdown or filter
- Create a modal or dialog

**Characteristics:**
- May depend on logic (calls services)
- Changes are visual and interactive
- Requires browser verification

### Phase 4: Integration
**What:** Connecting pieces, data fetching, event handling, end-to-end flows

**Examples:**
- Connect UI component to backend service
- Add data fetching to a page
- Handle user interactions (clicks, form submissions)
- Wire up real-time updates

**Characteristics:**
- Depends on both logic and interface
- Makes the feature actually work end-to-end
- Requires browser verification

### Phase 5: Polish
**What:** Error handling, edge cases, refinement, final touches

**Examples:**
- Handle network errors gracefully
- Add loading states
- Handle empty states
- Optimize performance
- Add accessibility improvements

**Characteristics:**
- Depends on integration being complete
- Improves robustness and user experience
- Often involves testing edge cases

## Task Sizing Rules

### Right-Sized Tasks

A task is right-sized if you can:
1. Describe what needs to be done in 2-3 sentences
2. List 3-5 specific acceptance criteria
3. Complete it in one focused session
4. Verify it's done without ambiguity

**Examples:**
- ✅ "Add priority column to tasks table with migration"
- ✅ "Create service function to get user notifications"
- ✅ "Add notification bell icon to header with unread count badge"
- ✅ "Connect notification dropdown to backend service"

### Tasks That Are Too Big

If you find yourself saying "and then" or "also need to", the task is too big.

**Examples:**
- ❌ "Add authentication system" → Split into: schema, middleware, login UI, session handling
- ❌ "Build the dashboard" → Split into: data queries, chart components, filters, layout
- ❌ "Refactor the API" → Split into one task per endpoint or pattern

**How to split:**
1. Identify the distinct pieces
2. Order them by dependencies
3. Make each piece its own task
4. Ensure each task is independently verifiable

## Dependency Ordering

Tasks should be ordered so that:
- Earlier tasks don't depend on later ones
- Dependencies are obvious from the task order
- Ralph can pick tasks by priority without breaking things

**Correct order:**
1. Schema changes (foundation)
2. Server functions (logic)
3. UI components (interface)
4. Connecting them (integration)
5. Error handling (polish)

**Wrong order:**
1. UI component (needs schema that doesn't exist)
2. Schema change
3. Server function

## Acceptance Criteria

Each task needs verifiable acceptance criteria. "Works correctly" is not verifiable. "Button shows confirmation dialog before deleting" is verifiable.

### Good Criteria

- Specific and measurable
- Can be checked automatically or manually
- Clear pass/fail condition
- Includes quality checks (typecheck, tests)

**Examples:**
- "Add `status` column to tasks table with default 'pending'"
- "Filter dropdown has options: All, Active, Completed"
- "Clicking delete shows confirmation dialog"
- "Typecheck passes"
- "Tests pass"
- "Verify in browser using dev-browser skill" (for UI tasks)

### Bad Criteria

- Vague or subjective
- Cannot be verified
- No clear completion condition

**Examples:**
- "Works correctly"
- "User can do X easily"
- "Good UX"
- "Handles edge cases" (too vague - which edge cases?)

## Metadata Fields (Optional)

You can add optional metadata to tasks for documentation:

- `stage`: "foundation" | "logic" | "interface" | "integration" | "polish"
- `focus`: More specific focus area (e.g., "data-model", "ui-component", "error-handling")
- `responsibility`: "schema" | "backend" | "frontend" | "full-stack"

**Important:** Ralph ignores these fields. They're purely for human understanding. The task will work identically with or without them.

## Example: Breaking Down a Feature

**Feature:** "Add user notifications"

**Unstructured (too big):**
- "Add notification system"

**Structured (right-sized):**

1. **Foundation:** Add notifications table to database
2. **Logic:** Create notification service functions
3. **Interface:** Add notification bell icon to header
4. **Interface:** Create notification dropdown panel
5. **Integration:** Connect dropdown to backend service
6. **Integration:** Add mark-as-read functionality
7. **Integration:** Handle notification creation events
8. **Polish:** Add error handling and edge cases

Each task is:
- Small enough for one iteration
- Clearly verifiable
- Properly ordered by dependencies
- Independently completable

## Checklist

Before adding a task to your PRD, verify:

- [ ] Task can be completed in one context window
- [ ] Task has 3-5 specific acceptance criteria
- [ ] All criteria are verifiable (not vague)
- [ ] Task doesn't depend on later tasks
- [ ] Task includes quality checks (typecheck, tests if applicable)
- [ ] UI tasks include browser verification requirement

## Remember

Ralph executes tasks. Structure comes from task definition, not from orchestration. Well-structured tasks lead to better outcomes because:

- The agent has clear focus
- Progress is easy to track
- Dependencies are obvious
- Completion is verifiable

Keep tasks small, specific, and ordered. That's the key to effective Ralph usage.

