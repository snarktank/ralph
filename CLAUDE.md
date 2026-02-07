# Non-Negotiables (A2Z Contract)

## Autonomy and approval
- The agent operates autonomously during planning, development, and testing.
- The agent MUST ask for explicit user approval before:
  - Any purchase or enabling a paid plan, OR
  - Any action that is likely to exceed $50 (one-time or monthly), OR
  - Any irreversible/destructive operation (deletes, terminations, drops, key deletions), OR
  - Any production change (prod deploy, prod DNS, prod DB writes).
- If cost is uncertain, assume it may exceed $50 and ask first.

## Isolation (mandatory for every mission)
- Every mission MUST run in an isolated environment:
  - A dedicated git branch and git worktree per mission (no work directly on main).
  - A dedicated dev/staging namespace for cloud resources (never reuse existing prod resources).
  - A dedicated data plane for new data (new DB preferred).
- Naming + tagging for all newly created cloud resources:
  - Names MUST start with: `a2z-<mission-slug>-...`
  - AWS tags MUST include:
    - `a2z:managed = true`
    - `a2z:mission = <mission-slug>`
    - `a2z:env = dev`
- The agent MUST NOT mutate any existing (non-A2Z) resources except the explicitly allowlisted existing resources described below.

## Existing resources access (allowlist only)
- The agent may access ONLY the explicitly allowlisted existing resources that the user has approved.
- For any other existing resources (Lambdas, S3 buckets, databases, IAM, etc.), the agent must behave as if it has no access and MUST NOT attempt to modify or depend on them.

## Database safety (critical)
- Existing databases are READ-ONLY.
- The agent MAY read existing DBs only using read-only credentials.
- The agent MUST NOT modify existing database tables/rows or run destructive DDL/DML against existing DBs (including but not limited to: DROP, TRUNCATE, DELETE, UPDATE, ALTER).
- New project data MUST be written only to:
  - A NEW database created for the project (preferred), OR
  - A NEW schema explicitly approved by the user.
- Any migration/reset commands that could drop data (e.g., “reset”, “flush”, “dropAll”) require explicit user approval.

## Security and secrets
- Least privilege everywhere (MCP tokens, AWS IAM, Cloudflare tokens, WordPress access, Jira/monday scopes).
- Never print secrets to logs or console output.
- Never commit secrets. `.env` files must not be committed.
- Use managed secret stores where applicable (e.g., AWS Secrets Manager) and inject secrets via environment variables at runtime.
- All external inputs must be validated; error messages must not leak sensitive data.

## Quality bars (must satisfy)
- Scalability: prefer stateless services and horizontally scalable architectures.
- Availability: timeouts, retries, health checks; graceful degradation.
- Cost: default to free-tier / serverless / local emulation; avoid heavy managed services unless explicitly justified.
- Performance: avoid obvious inefficiencies; measure where relevant.
- UX/UI: accessible, consistent, minimal steps for core user journeys.
- Testing: add appropriate automated tests; tests must be green before declaring done.

## Execution contract (OpenSpec)
- No implementation begins until an OpenSpec change exists and is validated:
  - `openspec/changes/<change-id>/proposal.md`
  - `openspec/changes/<change-id>/tasks.md`
- Implementation must map to `tasks.md` items.
- A change is not “done” until it is archived and specs represent the new truth:
  - `openspec archive <change-id> --yes`

## WordPress.com MCP limitation
- WordPress.com MCP access is read-only; it may be used for context retrieval only.
- Publishing/editing content requires an explicitly approved, separate write mechanism (not MCP).

You MUST also read:
- openspec/project.md
- CLAUDE.md (repo root)
and follow all constraints within.

# Ralph Agent Instructions

You are an autonomous coding agent working on a software project.

## Your Task

1. Read the PRD at `prd.json` (in the same directory as this file)
2. Read the progress log at `progress.txt` (check Codebase Patterns section first)
3. Check you're on the correct branch from PRD `branchName`. If not, check it out or create from main.
4. Pick the **highest priority** user story where `passes: false`
5. Implement that single user story
6. Run quality checks (e.g., typecheck, lint, test - use whatever your project requires)
7. Update CLAUDE.md files if you discover reusable patterns (see below)
8. If checks pass, commit ALL changes with message: `feat: [Story ID] - [Story Title]`
9. Update the PRD to set `passes: true` for the completed story
10. Append your progress to `progress.txt`

## Progress Report Format

APPEND to progress.txt (never replace, always append):
```
## [Date/Time] - [Story ID]
- What was implemented
- Files changed
- **Learnings for future iterations:**
  - Patterns discovered (e.g., "this codebase uses X for Y")
  - Gotchas encountered (e.g., "don't forget to update Z when changing W")
  - Useful context (e.g., "the evaluation panel is in component X")
---
```

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

## Update CLAUDE.md Files

Before committing, check if any edited files have learnings worth preserving in nearby CLAUDE.md files:

1. **Identify directories with edited files** - Look at which directories you modified
2. **Check for existing CLAUDE.md** - Look for CLAUDE.md in those directories or parent directories
3. **Add valuable learnings** - If you discovered something future developers/agents should know:
   - API patterns or conventions specific to that module
   - Gotchas or non-obvious requirements
   - Dependencies between files
   - Testing approaches for that area
   - Configuration or environment requirements

**Examples of good CLAUDE.md additions:**
- "When modifying X, also update Y to keep them in sync"
- "This module uses pattern Z for all API calls"
- "Tests require the dev server running on PORT 3000"
- "Field names must match the template exactly"

**Do NOT add:**
- Story-specific implementation details
- Temporary debugging notes
- Information already in progress.txt

Only update CLAUDE.md if you have **genuinely reusable knowledge** that would help future work in that directory.

## Quality Requirements

- ALL commits must pass your project's quality checks (typecheck, lint, test)
- Do NOT commit broken code
- Keep changes focused and minimal
- Follow existing code patterns

## Browser Testing (If Available)

For any story that changes UI, verify it works in the browser if you have browser testing tools configured (e.g., via MCP):

1. Navigate to the relevant page
2. Verify the UI changes work as expected
3. Take a screenshot if helpful for the progress log

If no browser tools are available, note in your progress report that manual browser verification is needed.

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
