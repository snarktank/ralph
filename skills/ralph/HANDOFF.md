# Ralph Handoff Instructions

This file is created when context approaches the limit (~90%) during story execution. It provides state information for continuing work in a new session.

---

## Current State

**Story being worked on:**
- Story ID: `[US-XXX]`
- Story Title: `[Title]`
- Status: `[In Progress / Partially Complete]`

**What has been completed:**
- [List what's done so far]
- [Files that have been modified]
- [Acceptance criteria that have been met]

**What remains to be done:**
- [List what still needs work]
- [Remaining acceptance criteria]
- [Next steps]

---

## Files Changed

**Modified files:**
```
[File path 1]
[File path 2]
[File path 3]
```

**New files created:**
```
[File path 1]
[File path 2]
```

**Files that need modification:**
```
[File path 1 - what needs to be done]
[File path 2 - what needs to be done]
```

---

## Git Status

**Current branch:** `[branch name from prd.json]`

**Last commit:**
```
[Commit hash] feat: [Story ID] - [Story Title] (in progress)
```

**Uncommitted changes:**
- [List of files with uncommitted changes]
- [Note if changes were committed before handoff]

---

## Context for Next Session

**Important patterns discovered:**
- [Pattern 1]
- [Pattern 2]

**Gotchas encountered:**
- [Gotcha 1]
- [Gotcha 2]

**Useful context:**
- [Context 1]
- [Context 2]

**Codebase patterns to remember:**
- [Pattern from Codebase Patterns section]

---

## Next Steps

1. **Read this handoff file** to understand current state
2. **Read prd.json** to see story status
3. **Read progress.txt** (especially Codebase Patterns section)
4. **Check git history** for recent commits
5. **Continue implementation** from where previous session left off
6. **Complete remaining acceptance criteria**
7. **Run quality checks** (typecheck, lint, test)
8. **Commit when complete** with message: `feat: [Story ID] - [Story Title]`
9. **Update prd.json** to set `passes: true`
10. **Append to progress.txt** with learnings

---

## Acceptance Criteria Status

**Completed:**
- [ ] Criterion 1 ✓
- [ ] Criterion 2 ✓

**Remaining:**
- [ ] Criterion 3
- [ ] Criterion 4
- [ ] Typecheck passes
- [ ] Verify in browser (if UI story)

---

## Notes

[Any additional context, decisions made, or important information for the next session]

---

**After successfully continuing from this handoff, delete this file.**
