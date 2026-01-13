# Context Management Guide

This guide provides detailed procedures for detecting context limits and managing handoffs during Ralph execution.

---

## Context Detection

### Monitoring Token Usage

Claude's context window has limits. During story execution, monitor usage:

**Indicators of approaching limit:**
- Response length decreasing
- Difficulty maintaining conversation history
- System warnings about context
- Estimated token usage approaching ~90% of window

**Typical context window sizes:**
- Claude Sonnet: ~200k tokens
- Claude Opus: ~200k tokens
- Other models vary

**Threshold for handoff:** ~90% of available context

---

## When to Prepare Handoff

Prepare handoff when:

1. **Token usage approaches ~90%** - Estimate based on conversation length and file reads
2. **Story is partially complete** - Some work done but not finished
3. **Complex implementation remaining** - Significant work still needed
4. **Multiple file reads anticipated** - Will need to read many files to continue

**Do NOT handoff if:**
- Story is nearly complete (can finish in current session)
- Only simple tasks remain
- Context usage is still low (~70% or less)

---

## Handoff Preparation Workflow

### Step 1: Commit Current Work

Even if story is incomplete, commit what you have:

```bash
git add .
git commit -m "feat: [Story ID] - [Story Title] (in progress)"
```

**Why commit incomplete work?**
- Preserves progress if handoff fails
- Provides git history for next session
- Allows rollback if needed

### Step 2: Update prd.json

Add progress notes to current story:

```json
{
  "id": "US-001",
  "title": "Story Title",
  "passes": false,
  "notes": "In progress: Completed [X, Y]. Remaining: [Z, W]. See HANDOFF.md for details."
}
```

### Step 3: Append to progress.txt

Document current state:

```
## [Date/Time] - [Story ID] (Handoff)

**Status:** In progress - context limit reached

**Completed:**
- [What's done]

**Remaining:**
- [What needs work]

**Files changed:**
- [List files]

**Next steps:**
- [What to do next]

See HANDOFF.md for detailed handoff instructions.
---
```

### Step 4: Create HANDOFF.md

Use the template in [HANDOFF.md](HANDOFF.md) and include:

- Current story ID and title
- What's completed vs. remaining
- Files changed
- Git status
- Context for next session
- Next steps
- Acceptance criteria status

**Location:** Project root (same level as prd.json)

### Step 5: Signal to User

Tell the user:

```
Context approaching limit (~90%). Handoff file created at HANDOFF.md.

Please start a new session and:
1. Load HANDOFF.md to understand current state
2. Continue implementation from where we left off
3. Complete remaining acceptance criteria
4. Delete HANDOFF.md after successful continuation
```

---

## Session Bootstrap After Handoff

When starting a new session with a handoff:

### Step 1: Read Handoff File

```bash
cat HANDOFF.md
```

Understand:
- What story was being worked on
- What's done vs. what remains
- Files that were changed
- Important context and patterns

### Step 2: Load State Files

**Read prd.json:**
```bash
cat prd.json
```
- See current story status
- Check notes field for progress info

**Read progress.txt:**
```bash
cat progress.txt
```
- Focus on `## Codebase Patterns` section at top
- Review recent iteration entries

**Check git status:**
```bash
git log --oneline -10
git status
```
- See recent commits
- Check for uncommitted changes

### Step 3: Verify Branch

```bash
git branch --show-current
```

Ensure on correct branch from prd.json `branchName`.

### Step 4: Continue Implementation

1. **Review what's done** - Understand completed work
2. **Identify remaining work** - Check acceptance criteria status
3. **Continue implementation** - Complete remaining tasks
4. **Follow existing patterns** - Use Codebase Patterns from progress.txt
5. **Complete story** - Finish all acceptance criteria
6. **Run quality checks** - Typecheck, lint, test
7. **Commit when complete** - Standard commit message
8. **Update state** - prd.json and progress.txt
9. **Delete HANDOFF.md** - Clean up after successful continuation

---

## Best Practices

### Prevent Premature Handoffs

- **Estimate remaining work** - If story is 80%+ complete, try to finish it
- **Optimize file reads** - Read only necessary files, not entire codebase
- **Be concise in responses** - Don't repeat information unnecessarily
- **Use progressive disclosure** - Load detailed guides only when needed

### Effective Handoff Documentation

- **Be specific** - List exact files and what was changed
- **Document decisions** - Note why certain approaches were chosen
- **Include context** - Patterns, gotchas, useful information
- **Clear next steps** - Explicit list of what needs to be done

### Smooth Continuation

- **Read handoff thoroughly** - Don't skip context
- **Verify state** - Check git, prd.json, progress.txt match handoff
- **Continue naturally** - Pick up where previous session left off
- **Clean up** - Delete HANDOFF.md after successful continuation

---

## Example Handoff Scenario

**Situation:** Implementing US-003 (Add status toggle to task list rows). Context at ~88%.

**Handoff preparation:**
1. Committed partial work: `feat: US-003 - Add status toggle (in progress)`
2. Updated prd.json notes: "Completed: Added dropdown component. Remaining: Save logic, UI update."
3. Appended to progress.txt with status
4. Created HANDOFF.md with:
   - Story: US-003
   - Completed: Dropdown component added to TaskRow
   - Remaining: Save handler, optimistic UI update, tests
   - Files: `components/TaskRow.tsx`, `actions/tasks.ts`
   - Next: Implement save handler, add optimistic update

**Next session:**
1. Read HANDOFF.md
2. See dropdown is done, need save logic
3. Implement save handler
4. Add optimistic UI update
5. Run tests
6. Complete story
7. Delete HANDOFF.md

---

## Troubleshooting

**Handoff file missing:**
- Check project root
- May have been deleted already
- Check git history for last commit message

**State mismatch:**
- Compare HANDOFF.md with current git/prd.json/progress.txt
- Resolve differences before continuing
- May need to adjust based on current state

**Cannot continue:**
- Review handoff context carefully
- Check if story requirements changed
- May need to restart story if too much context lost

---

**This guide is loaded only when context management is needed, following progressive disclosure principles.**
