# Ralph Agent Instructions

> **Ralph directory: `scripts/ralph/`** — All Ralph files live here. Your working directory is the repo root; always use the `scripts/ralph/` prefix for Ralph file operations.

## Step 0: Read State (ALWAYS FIRST)

1. Read `scripts/ralph/state.json`.
   - If it exists: orient from `currentStory`, `nextAction`, `branch`, `compactionNeeded`.
   - If missing: read `scripts/ralph/BOOTSTRAP.md` and follow it to create `scripts/ralph/state.json`, then continue.
2. If `compactionNeeded` is `true`: read `scripts/ralph/COMPACTION.md` and follow it. Do NOT work on stories. STOP after.
3. Proceed to **Your Task** below.

## Your Task

1. Check you're on the correct branch from `state.json`. If not, check it out or create from main.
2. Read `scripts/ralph/progress.md` in two reads: the `## Codebase Patterns` section from the top (~60 lines), then recent sessions from the bottom (last ~80 lines). Never read the entire file.
3. Read `scripts/ralph/prd.json` and locate the story matching `state.json.currentStory` — read its acceptance criteria and description.
4. **If story has `iterationMode` field**: read `scripts/ralph/ITERATIVE.md` BEFORE starting work.
5. Implement that single user story.
6. Run quality checks (typecheck, lint, test; plus `validationCommand` if defined). Fix failures before committing. If unfixable: document in `scripts/ralph/progress.md`, do NOT set `passes: true`, end session.
7. **Frontend verification** (if UI): verify in browser using available tools. If none available, note in `scripts/ralph/progress.md`.
8. Update Institutional Memory (below) if you discover reusable conventions or gotchas. Optionally update project-level `CLAUDE.md` files for directory-scoped knowledge.
9. Commit ALL changes: `feat: [Story ID] - [Story Title]`
10. **APPEND** to `scripts/ralph/progress.md`: date, story ID, what was done, files changed, learnings for future sessions.
    **PROMOTE (mandatory)**: re-read your learnings — any learning useful beyond the current story MUST go to `## Codebase Patterns` (top of `scripts/ralph/progress.md`). Cross-PRD conventions → Institutional Memory. Format: `scripts/ralph/PROGRESS-GUIDE.md`.
11. Update PRD: set `passes: true` for the completed story in `scripts/ralph/prd.json`. If final iteration, compact notes to max 2 lines (~200 chars). Details in `scripts/ralph/PROGRESS-GUIDE.md`.
12. **Update `scripts/ralph/state.json`** (see State Update Rules below).
13. **STOP** — end response immediately. The shell loop detects completion from `scripts/ralph/prd.json`; a new session handles remaining work.

## State Update Rules

At END of every session, update `scripts/ralph/state.json`:

1. Scan `scripts/ralph/prd.json` for the next incomplete story (first with `passes: false`, ordered by `priority`)
2. `currentStory`: that story's ID (or `null` if all complete)
3. `storyTitle`: that story's title (or `"all stories complete"`)
4. `nextAction`:
   - If iterative with `currentIteration > 0`: `"iteration N/M (review pass)"` or `"iteration N/M"`
   - If iterative with `currentIteration == 0`: `"iteration 1/M"`
   - If standard: `"implement"`
   - If all complete: `"all stories complete"`
5. `branch`: keep as-is unless the PRD changes
6. `progressTokens`: estimate current size of `scripts/ralph/progress.md` (~15 tokens/line)
7. `compactionNeeded`: set to `true` if `progressTokens` exceeds `compactionThreshold`
8. `lastUpdated`: current date

## Critical Rules

These rules MUST be followed every session, even if earlier context is summarized.

1. **ALWAYS** read `scripts/ralph/state.json` FIRST, before any other file.
2. **ALWAYS** read `scripts/ralph/progress.md` (Codebase Patterns first) before starting work.
3. **ALWAYS** work on exactly ONE story per session.
4. The story to work on comes from `state.json.currentStory` — NEVER scan `scripts/ralph/prd.json` to find it.
5. **ALWAYS** append to `scripts/ralph/progress.md` before ending a session. NEVER replace its contents (except during compaction — see Step 0.2).
6. **ALWAYS** commit all changes before ending a session. Commits MUST pass quality checks.
7. For iterative stories, all iteration-specific rules are in `scripts/ralph/ITERATIVE.md`.
8. After steps 9-13, **STOP**. End the session. A new session handles remaining work.
9. **ALWAYS** update `scripts/ralph/state.json` at the end of every session.
10. If `compactionNeeded` is true, follow `scripts/ralph/COMPACTION.md` — do NOT work on stories.

## Institutional Memory

> Format: `- **Name**: description` | Budget: 40 lines max
> Rules: Never remove unless proven wrong. No story-specific details or timestamps.
