---
name: ralph-codex
description: "Operate the Ralph fresh-context loop with Codex CLI. Use when running or maintaining the Ralph loop in this repo."
metadata:
  short-description: "Ralph loop conventions for Codex"
---

# Ralph Codex Loop

This skill defines the working agreement for Ralph when running with Codex CLI.

## Working Agreement

1. Each iteration starts with fresh context; memory persists via git history, `progress.txt`, and `prd.json`.
2. Select the highest-priority user story where `passes: false`, then implement only that story.
3. Run quality checks (typecheck, lint, test as required) before committing.
4. Commit with `feat: [Story ID] - [Story Title]` only if checks pass.
5. Update `prd.json` and append to `progress.txt` every iteration.
6. Record reusable patterns in relevant `AGENTS.md` files.

## Stop Condition

If all stories have `passes: true`, respond with:

```
<promise>COMPLETE</promise>
```

## Codex Execution Notes

- Default `codex exec` is read-only; use `--full-auto` for edits.
- Use `--output-last-message` to capture the final message for loop control.
- Favor limited sandbox permissions; only widen access when explicitly requested.
