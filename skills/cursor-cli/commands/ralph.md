# Ralph Command

Execute the Ralph autonomous agent loop to implement user stories iteratively, or convert PRDs to prd.json format.

## Usage

### Executing Stories

Use `/ralph` or say "run ralph" or "execute stories from prd.json" to start implementing stories:

```
/ralph execute stories from prd.json
```

Or use natural language:
```
run ralph
execute stories from prd.json
implement stories from prd.json
```

### Converting PRDs

Use `/ralph` to convert a PRD markdown file to `prd.json`:

```
/ralph convert tasks/prd-task-priority.md to prd.json
```

Or use natural language:
```
convert this PRD to ralph format
create prd.json from this PRD
```

## What This Does

### When Executing Stories

1. Reads `prd.json` and `progress.txt`
2. Checks you're on the correct branch
3. Picks the highest priority story where `passes: false`
4. Implements that single story
5. Runs quality checks (typecheck, lint, test)
6. Updates AGENTS.md files if patterns discovered
7. Commits if checks pass: `feat: [Story ID] - [Story Title]`
8. Updates `prd.json` to mark story as `passes: true`
9. Appends learnings to `progress.txt`
10. Continues until all stories pass or you stop it

### When Converting PRDs

Converts a markdown PRD file to `prd.json` format with user stories structured for autonomous execution.

## Completion

When all stories have `passes: true`, Ralph signals `<promise>COMPLETE</promise>`.

## Full Documentation

See `.cursor/rules/ralph.md` for complete execution workflow, story sizing guidelines, and examples.
