# Monitoring Guide

This guide helps operators monitor Ralph Wiggum during autonomous runs and know when to intervene.

## Red Flags: When to Intervene

Watch for these patterns that indicate Ralph needs human intervention:

### 1. Repeated Failures on Same Story
```bash
# Check progress.txt for repeated story attempts
grep -c "US-00X" progress.txt
```
If the same story ID appears more than 3 times, Ralph is likely stuck.

### 2. Typecheck/Lint Loops
```bash
# Watch for repeated error patterns
tail -f progress.txt | grep -E "(typecheck|lint|error)"
```
Repeated cycles of "fixing" the same error indicates a fundamental misunderstanding.

### 3. File Thrashing
```bash
# Check git for excessive changes to same file
git log --oneline --follow -20 -- path/to/file.ts
```
Multiple commits to the same file in quick succession suggests trial-and-error debugging.

### 4. Scope Creep
```bash
# Check for unexpected file changes
git diff --stat HEAD~5
```
If Ralph is modifying files unrelated to the current story, it may have lost focus.

### 5. Silent Failures
```bash
# Check if progress is being made
ls -la progress.txt
cat prd.json | jq '.userStories[] | select(.passes == true) | .id'
```
If progress.txt hasn't been updated but Ralph is still running, something may be wrong.

### 6. Credential Warnings
```bash
# Monitor for any credential-related output
grep -i -E "(password|secret|key|token|credential)" progress.txt
```
Any mention of credentials in logs requires immediate review.

### 7. Network Activity
```bash
# Check for unexpected network calls (if using network monitoring)
lsof -i -P | grep ralph
```
Unexpected network activity could indicate Ralph is accessing external services.

## Monitoring Commands

Use these commands to monitor Ralph in real-time:

### Real-Time Progress
```bash
# Follow progress updates
tail -f progress.txt

# Watch for story completions
watch -n 5 'cat prd.json | jq ".userStories[] | select(.passes == true) | .id"'
```

### Story Status Dashboard
```bash
# Show all story statuses
cat prd.json | jq -r '.userStories[] | "\(.id): \(.title) - passes: \(.passes)"'

# Count completed vs total
echo "Completed: $(cat prd.json | jq '[.userStories[] | select(.passes == true)] | length')/$(cat prd.json | jq '.userStories | length')"
```

### Git Activity
```bash
# Watch for new commits
watch -n 10 'git log --oneline -10'

# Check uncommitted changes
git status --short

# View recent diffs
git diff HEAD~1 --stat
```

### Resource Usage
```bash
# Monitor CPU/memory usage
top -l 1 | grep -E "(ralph|claude|amp)"

# Check disk usage in project
du -sh .
```

## When to Stop and Regenerate Plan

Stop Ralph and regenerate the plan when:

1. **Same error appears 3+ times** - The current approach isn't working
2. **Story takes more than 5 iterations** - Requirements may be unclear or impossible
3. **Multiple stories fail in sequence** - There may be a fundamental issue with the plan
4. **Unexpected side effects** - Ralph is breaking previously working features
5. **Tests start failing** - Regression indicates architectural problems
6. **Budget threshold reached** - Cost is exceeding the value of the feature

### How to Stop and Reassess

```bash
# 1. Stop Ralph gracefully
touch .ralph-stop
# OR
Ctrl+C

# 2. Review current state
git log --oneline -10
git diff
cat progress.txt | tail -50

# 3. Check which stories are problematic
cat prd.json | jq '.userStories[] | select(.passes == false) | {id, title, notes}'

# 4. Consider if PRD needs revision
# - Are acceptance criteria clear and achievable?
# - Are there missing dependencies between stories?
# - Is the scope realistic?
```

## Intervention Checklist

Before intervening, run through this checklist:

- [ ] **Is Ralph actually stuck?** - Wait at least 2 minutes for complex operations
- [ ] **Check the logs** - Review progress.txt for context on what Ralph is attempting
- [ ] **Review recent commits** - Understand what changes have been made
- [ ] **Check story notes** - Ralph may have added notes explaining difficulties
- [ ] **Verify acceptance criteria** - Ensure they are actually achievable
- [ ] **Check for external dependencies** - Does the story require services Ralph can't access?
- [ ] **Review error messages** - Are there clear errors indicating the problem?
- [ ] **Consider partial progress** - Can you help Ralph past a specific blocker?

### Post-Intervention Actions

After intervening:

1. **Document the intervention** - Add a note to progress.txt explaining what you did
2. **Update story notes** - Add context to prd.json if helpful
3. **Consider PRD changes** - Split complex stories or clarify criteria if needed
4. **Restart cleanly** - Ensure Ralph has a clear starting point
5. **Monitor closely** - Watch the first few iterations after intervention

## Alert Thresholds

Configure these alerts for autonomous monitoring:

| Metric | Warning | Critical |
|--------|---------|----------|
| Same story iterations | 3 | 5 |
| Time on single story | 15 min | 30 min |
| Consecutive failures | 2 | 3 |
| Files changed per commit | 10 | 20 |
| API cost per story | $1 | $5 |
| Total run cost | $10 | $25 |
