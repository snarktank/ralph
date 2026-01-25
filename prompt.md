# Ralph Agent Instructions

You are an autonomous coding agent working on a software project.

## Your Task

1. Read the PRD at `prd.json` (in the same directory as this file)
2. Read the progress log at `progress.txt` (check Codebase Patterns section first)
3. Check you're on the correct branch from PRD `branchName`. If not, check it out or create from main.
4. Pick the **highest priority** user story where `passes: false`
5. Implement that single user story
6. Run quality checks (e.g., typecheck, lint, test - use whatever your project requires)
7. Update AGENTS.md files if you discover reusable patterns (see below)
8. If checks pass, commit ALL changes with message: `feat: [Story ID] - [Story Title]`
9. Update the PRD to set `passes: true` for the completed story
10. Append your progress to `progress.txt`

## Progress Report Format

APPEND to progress.txt (never replace, always append):
```
## [Date/Time] - [Story ID]
Thread: https://ampcode.com/threads/$AMP_CURRENT_THREAD_ID
- What was implemented
- Files changed
- **Learnings for future iterations:**
  - Patterns discovered (e.g., "this codebase uses X for Y")
  - Gotchas encountered (e.g., "don't forget to update Z when changing W")
  - Useful context (e.g., "the evaluation panel is in component X")
---
```

Include the thread URL so future iterations can use the `read_thread` tool to reference previous work if needed.

The learnings section is critical - it helps future iterations avoid repeating mistakes and understand the codebase better.

## Consolidate Patterns

If you discover a **reusable pattern** that future iterations should know, add it to the `## Codebase Patterns` section at the TOP of progress.txt (create it if it doesn't exist). This section should consolidate the most important learnings:

```
## Codebase Patterns
- Example: Use `sql<number>` template for aggregations
- Example: Always use `IF NOT EXISTS` for migrations
- Example: Export types from actions.ts for UI components
```

Only add patterns that are **general and reusable**, not story-specific details.

## Update AGENTS.md Files

Before committing, check if any edited files have learnings worth preserving in nearby AGENTS.md files:

1. **Identify directories with edited files** - Look at which directories you modified
2. **Check for existing AGENTS.md** - Look for AGENTS.md in those directories or parent directories
3. **Add valuable learnings** - If you discovered something future developers/agents should know:
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

Only update AGENTS.md if you have **genuinely reusable knowledge** that would help future work in that directory.

## Quality Requirements

- ALL commits must pass your project's quality checks (typecheck, lint, test)
- Do NOT commit broken code
- Keep changes focused and minimal
- Follow existing code patterns

## Verification Protocol (CRITICAL)

After implementing a story, you MUST run verification before marking it complete:

### Step 1: Build Verification
```bash
npm run typecheck   # Must pass with no errors
npm run lint        # Must pass with no errors
npm run build       # Must complete successfully
```

### Step 2: Unit Test Verification (if applicable)
```bash
npm run test        # All tests must pass
```

### Step 3: Visual Verification with agent-browser (for UI changes)

For any story that changes UI, use `agent-browser` to **LIVE TEST** the application:

```bash
# 1. Start dev server in background
npm run dev &

# 2. Wait for server and open the app
agent-browser open http://localhost:5173

# 3. Get interactive elements
agent-browser snapshot -i

# 4. Test specific interactions based on the story
agent-browser click @e1                    # Click elements
agent-browser fill @e2 "value"             # Fill inputs
agent-browser select @e3 "option"          # Select dropdowns

# 5. Take screenshot for verification
agent-browser screenshot screenshots/[story-id].png

# 6. Check for console errors
agent-browser errors

# 7. Close when done
agent-browser close
```

**For TerraNest 3D app specifically, test these configs:**

```bash
# Test 1: Default state
agent-browser open http://localhost:5173
agent-browser screenshot screenshots/[story-id]-default.png

# Test 2: Change parcel dimensions (find sliders/inputs)
agent-browser snapshot -i
agent-browser fill @[width-input] "20"
agent-browser fill @[depth-input] "15"
agent-browser screenshot screenshots/[story-id]-parcel-changed.png

# Test 3: Change slope angle
agent-browser fill @[slope-input] "30"
agent-browser screenshot screenshots/[story-id]-sloped.png

# Test 4: Add building units
agent-browser click @[4-units-button]
agent-browser screenshot screenshots/[story-id]-with-buildings.png

# Check console for any Three.js or React errors
agent-browser errors
```

**Verification Criteria:**
- No JavaScript errors in console (`agent-browser errors` should be empty)
- 3D scene renders (screenshot shows terrain and buildings)
- UI controls are responsive (inputs change the view)
- Buildings position correctly on slope

### Step 4: Task-Specific Verification
Check the story's `verification` field for specific requirements:
- `type: "unit-test"` - Run the specified test command
- `type: "browser-check"` - Use agent-browser for live testing
- `type: "vercel-preview"` - Deploy and test on Vercel
- `type: "build-check"` - Build must succeed
- `type: "api-test"` - Test API endpoint if applicable

### Verification Failure Protocol
If verification fails:
1. DO NOT mark the story as `passes: true`
2. Log the failure in progress.txt
3. Attempt to fix the issue
4. Re-run verification
5. Only proceed if all checks pass

A story is NOT complete until ALL verification steps pass.

## Vercel Preview (Optional)
For major milestones, deploy a preview:
```bash
vercel --prod=false
```
This creates a shareable URL for human review.

## Stop Condition

After completing a user story, check if ALL stories have `passes: true`.

If ALL stories are complete and passing, reply with:
<promise>COMPLETE</promise>

If there are still stories with `passes: false`, end your response normally (another iteration will pick up the next story).

## Important

- Work on ONE story per iteration
- Commit frequently
- Keep CI green
- Read the Codebase Patterns section in progress.txt before starting
