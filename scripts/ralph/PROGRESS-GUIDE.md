# Progress Guide

> Reference for writing progress reports and managing the experiential memory layer.
> Read this file when you need the full format details for `scripts/ralph/progress.md` or `scripts/ralph/prd.json` notes.

## Progress Report Format

APPEND to `scripts/ralph/progress.md` (NEVER replace, ALWAYS append):
```markdown
## [Date/Time] - [Story ID]
- What was implemented
- Files changed
- **Learnings for future sessions:**
  - Patterns discovered (e.g., "this codebase uses X for Y")
  - Gotchas encountered (e.g., "don't forget to update Z when changing W")
  - Useful context (e.g., "the evaluation panel is in component X")
---
```

The learnings section is **critical** — it helps future sessions avoid repeating mistakes.

## Codebase Patterns Section

Maintain a `## Codebase Patterns` section at the **TOP** of `scripts/ralph/progress.md` (right after the header). Create it if missing:
```markdown
## Codebase Patterns
> Reusable patterns discovered during implementation. Read this FIRST every session.

- **Pattern Name**: description
```

Budget: **30 lines max**. When approaching the limit, merge related patterns. Never add story-specific details or timestamps.

## Persist Learnings

After implementing a story, persist reusable discoveries to two places:

- **Codebase Patterns** (top of `scripts/ralph/progress.md`): General, reusable patterns
- **Institutional Memory** (bottom of `scripts/ralph/CLAUDE.md`): Conventions, gotchas, cross-file dependencies
- Optionally update project-level `CLAUDE.md` files in relevant directories for directory-scoped knowledge
- NEVER add: story-specific details, temporary debugging notes, or info already elsewhere
- Only add patterns that are **genuinely reusable** across stories

## Notes Compaction (prd.json)

When writing the FINAL notes for a story (the one that sets `passes: true`):
- Compact ALL previous notes into a single summary
- Final `notes` field must be **max 2 lines** (~200 chars)
- Keep: what was done, files changed, key ambiguities
- Remove: line numbers, audit scores, citation counts, verification details
- Format: `"[Summary of what was integrated/changed]. [Key ambiguity if any]. Complete."`
