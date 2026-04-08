# Ralph Codex Adaptation LLD

## Local skills

### `ralph-prd`

Responsibilities:

- ask only essential clarifying questions
- generate a markdown PRD
- keep stories small and verifiable
- write to `tasks/prd-<feature>.md`

### `ralph-loop`

Responsibilities:

- read `prd.json`
- read `progress.txt`
- ensure branch alignment with `branchName`
- pick highest-priority `passes: false` story
- implement exactly one story
- run repo checks
- update nearby `AGENTS.md` if reusable learnings were found
- commit with `feat: [Story ID] - [Story Title]`
- set `passes: true`
- append progress
- emit `COMPLETE` only when all stories pass

## Future standalone runner spec

The later runner should:

- accept target repo path and max iterations
- repeatedly invoke the Ralph loop behavior
- stop on `COMPLETE`
- stop on max-iteration exhaustion
- preserve the same `prd.json` and `progress.txt` contract

Because no local `codex` CLI/runtime contract is available here, the runner is a future
runtime concern rather than a current implementation target.

## Validation targets

- story sizing rules match upstream Ralph
- `prd.json` shape stays compatible with `prd.json.example`
- progress append format stays stable
- codebase pattern consolidation behavior is preserved
- nearby `AGENTS.md` update policy matches upstream `CLAUDE.md`
