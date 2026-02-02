# Example Feature: Task Comments

This is a complete example showing how structured tasks improve clarity and execution consistency in Ralph.

## The Feature

Add the ability for users to add comments to tasks. Users should be able to:
- View comments on a task
- Add new comments
- See who wrote each comment and when
- Delete their own comments

## Comparison: Unstructured vs Structured

### Unstructured Approach (Common Mistakes)

A poorly structured PRD might have tasks like:

```json
{
  "id": "US-001",
  "title": "Add comments to tasks",
  "description": "Users should be able to comment on tasks",
  "acceptanceCriteria": [
    "Comments work correctly",
    "Users can add and view comments"
  ]
}
```

**Problems:**
- Too vague - what does "work correctly" mean?
- Too big - database, backend, and UI all in one task
- No clear verification steps
- Agent doesn't know where to start

### Structured Approach (This Example)

The structured PRD breaks this into clear phases:

1. **Foundation** - Database schema for comments
2. **Logic** - Server functions to create/read/delete comments
3. **Interface** - UI components to display and add comments
4. **Integration** - Connect UI to backend, handle real-time updates
5. **Polish** - Error handling, edge cases, permissions

Each task is:
- **Small** - Completable in one iteration
- **Specific** - Clear acceptance criteria
- **Verifiable** - Can check if it's done
- **Ordered** - Dependencies are obvious

## The Structured PRD

See `prd.json` for the complete structured PRD with:
- Clear task breakdown by phase
- Specific acceptance criteria
- Optional metadata (`stage`, `focus`, `responsibility`)
- Proper ordering by dependencies

## Key Improvements

### 1. Clear Scope Per Task

Each task has a single, clear responsibility:
- US-001: Just the database schema
- US-002: Just the server functions
- US-003: Just the UI component
- etc.

The agent knows exactly what to work on.

### 2. Verifiable Acceptance Criteria

Instead of "works correctly", we have:
- "Create comments table with: id, taskId, userId, content, createdAt"
- "Component displays list of comments with author name and timestamp"
- "Clicking delete calls deleteComment service and removes comment from UI"

The agent can verify each step.

### 3. Natural Dependencies

Tasks are ordered so dependencies are obvious:
- Schema before server functions
- Server functions before UI
- UI before integration
- Integration before polish

Ralph picks tasks by priority, so this ordering ensures correct execution.

### 4. Metadata for Clarity (Optional)

The `stage`, `focus`, and `responsibility` fields help humans understand the task structure. Ralph ignores these fields - they're purely for documentation.

## Running This Example

1. Copy `prd.json` to your project's ralph directory
2. Run `./scripts/ralph/ralph.sh`
3. Watch how each iteration picks the next task and completes it

Notice how:
- Each iteration has clear focus
- Progress is easy to track
- Dependencies are handled naturally
- The agent knows exactly what "done" means

## Takeaways

Structuring tasks well means:
- **Smaller tasks** - Each fits in one context window
- **Clear criteria** - Easy to verify completion
- **Natural order** - Dependencies are obvious
- **Better outcomes** - Agent stays focused and completes work

You don't need the metadata fields (`stage`, `focus`, `responsibility`) - they're optional. The real value is in breaking work into small, verifiable, ordered tasks.

