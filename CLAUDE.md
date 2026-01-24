# CLAUDE.md - AI Assistant Guide for Ralph

## Project Overview

Ralph is an autonomous AI agent loop system for the [Amp CLI](https://ampcode.com). It repeatedly spawns fresh Amp instances to execute tasks from a PRD (Product Requirements Document) until all items are complete. Each iteration has clean context, with memory persisting only through git history, `progress.txt`, and `prd.json`.

Based on [Geoffrey Huntley's Ralph pattern](https://ghuntley.com/ralph/).

## Repository Structure

```
ralph/
├── ralph.sh              # Main bash loop (spawns Amp instances)
├── prompt.md             # System prompt for each Amp iteration
├── prd.json.example      # Example PRD format for reference
├── AGENTS.md             # Agent-specific instructions
├── README.md             # User documentation
├── skills/               # Amp skills for PRD generation
│   ├── prd/SKILL.md      # Generates markdown PRD documents
│   └── ralph/SKILL.md    # Converts PRDs to prd.json format
├── flowchart/            # Interactive React visualization
│   ├── src/App.tsx       # Main flowchart component
│   ├── package.json      # Dependencies (React 19, @xyflow/react)
│   └── vite.config.ts    # Build config (base: '/ralph/')
└── .github/workflows/    # GitHub Pages deployment
```

### Files NOT in Git (Generated at Runtime)
- `prd.json` - Active task list with story status
- `progress.txt` - Append-only learning log
- `.last-branch` - Tracks current branch for archiving

## Key Concepts

### Fresh Context Per Iteration
Each Amp instance starts with **zero memory** of previous iterations. The only continuity comes from:
- Git history (committed code)
- `progress.txt` (learnings and patterns)
- `prd.json` (story completion status)

### Memory Persistence Hierarchy
1. **prd.json** - Which stories are complete (`passes: true/false`)
2. **progress.txt** - Learnings, gotchas, thread URLs for reference
3. **Git commits** - All implemented code
4. **AGENTS.md** - Reusable patterns for the codebase

### Story Sizing (Critical for Success)

**Right-sized stories** (can complete in one context window):
- Add a database column and migration
- Add a UI component to an existing page
- Update a server action with new logic
- Add a filter dropdown to a list

**Too big** (must be split):
- "Build the entire dashboard"
- "Add authentication"
- "Refactor the API"

Rule: If you can't describe it in 2-3 sentences, it's too big.

### Story Ordering
Stories must be ordered by dependencies:
1. Schema/database changes first
2. Backend logic second
3. UI components third
4. Dashboard/integration views last

No forward dependencies allowed.

## Commands

### Running Ralph
```bash
./ralph.sh [max_iterations]  # Default: 10 iterations
```

### Flowchart Development
```bash
cd flowchart
npm install        # Install dependencies
npm run dev        # Development server with HMR
npm run build      # Production build (TypeScript + Vite)
npm run lint       # ESLint check
npm run preview    # Preview production build
```

### Debugging
```bash
# See which stories are done
cat prd.json | jq '.userStories[] | {id, title, passes}'

# See learnings from previous iterations
cat progress.txt

# Check git history
git log --oneline -10
```

## Technology Stack

### Core
- **Bash** - Orchestration script
- **jq** - JSON processing (required dependency)
- **Amp CLI** - AI code generation tool
- **Git** - Version control and memory persistence

### Flowchart Visualization
- **React 19** - UI framework
- **@xyflow/react 12** - Interactive node graph library
- **Vite 7** - Build tool
- **TypeScript 5.9** - Type safety
- **ESLint 9** - Linting

## PRD Format

```json
{
  "project": "ProjectName",
  "branchName": "ralph/feature-name",
  "description": "Feature description",
  "userStories": [
    {
      "id": "US-001",
      "title": "Story title",
      "description": "User story description",
      "acceptanceCriteria": [
        "Specific verifiable criteria",
        "Typecheck passes",
        "Verify in browser using dev-browser skill"
      ],
      "priority": 1,
      "passes": false,
      "notes": ""
    }
  ]
}
```

### Acceptance Criteria Requirements
- Must be **verifiable** (not vague)
- Always include "Typecheck passes"
- For testable logic: include "Tests pass"
- For UI changes: include "Verify in browser using dev-browser skill"

## Conventions for AI Assistants

### When Working on Ralph Itself

1. **Preserve the fresh-context model** - Don't add state that persists between iterations except through the defined channels (git, progress.txt, prd.json)

2. **Keep prompt.md focused** - Instructions given to each Amp instance should be clear and actionable

3. **Skills are templates** - The skills in `skills/` are documentation for Amp, not executable code

4. **Flowchart is standalone** - The React app in `flowchart/` is purely for visualization/presentation

### When Using Ralph in Another Project

1. **One story per iteration** - Never try to complete multiple stories
2. **Commit format**: `feat: [Story ID] - [Story Title]`
3. **Update progress.txt** - Always append learnings, never replace
4. **Update AGENTS.md** - Document reusable patterns discovered
5. **Quality gates** - All commits must pass typecheck/lint/tests

### Progress Report Format
```markdown
## [Date/Time] - [Story ID]
Thread: https://ampcode.com/threads/$AMP_CURRENT_THREAD_ID
- What was implemented
- Files changed
- **Learnings for future iterations:**
  - Patterns discovered
  - Gotchas encountered
  - Useful context
---
```

### Codebase Patterns Section
At the TOP of progress.txt, maintain a `## Codebase Patterns` section with consolidated learnings:
```markdown
## Codebase Patterns
- Use `sql<number>` template for aggregations
- Always use `IF NOT EXISTS` for migrations
- Export types from actions.ts for UI components
```

## Quality Requirements

- ALL commits must pass quality checks (typecheck, lint, test)
- Do NOT commit broken code (compounds across iterations)
- Keep changes focused and minimal
- Follow existing code patterns in the target project

## Stop Condition

When all stories have `passes: true`, output:
```
<promise>COMPLETE</promise>
```

This signals ralph.sh to exit successfully.

## Archiving

Ralph automatically archives previous runs when `branchName` changes:
- Archives saved to `archive/YYYY-MM-DD-feature-name/`
- Contains `prd.json` and `progress.txt` from previous run
- Progress file is reset for new feature

## Deployment

The flowchart visualization deploys to GitHub Pages:
- **URL**: https://snarktank.github.io/ralph/
- **Trigger**: Push to main or manual dispatch
- **Build**: `npm run build` outputs to `flowchart/dist/`
- **Base path**: `/ralph/` (configured in vite.config.ts)

## Common Patterns

### Modifying ralph.sh
- Use `set -e` for error handling
- Capture output with `tee /dev/stderr` for visibility
- Check for `<promise>COMPLETE</promise>` to detect completion

### Modifying prompt.md
- Keep instructions numbered and clear
- Reference specific file paths relative to script directory
- Include quality check requirements

### Modifying the Flowchart
- Custom node types defined in App.tsx
- Colors indicate phase: setup (blue), loop (green), decision (yellow), done (green)
- Animation controlled by visibility state
- Click advances through steps

## References

- [Geoffrey Huntley's Ralph article](https://ghuntley.com/ralph/)
- [Amp documentation](https://ampcode.com/manual)
- [Interactive Flowchart](https://snarktank.github.io/ralph/)
