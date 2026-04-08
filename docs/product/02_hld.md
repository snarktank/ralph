# Ralph Codex Adaptation HLD

## System role

The Codex adaptation is a workflow translation layer over Ralph’s existing model.

It keeps Ralph’s core state surfaces:

- `tasks/prd-<feature>.md`
- `prd.json`
- `progress.txt`
- nearby `AGENTS.md`

## Main subsystems

### PRD authoring

- local skill: `ralph-prd`
- generates markdown PRDs in `tasks/`

### Execution loop

- local skill: `ralph-loop`
- reads `prd.json`
- chooses one pending story
- implements only that story
- runs checks
- updates progress and story state

### Progress memory

- `progress.txt` remains append-only
- `Codebase Patterns` stays near the top and accumulates reusable knowledge

### Local knowledge capture

- nearby `AGENTS.md` files capture durable local conventions and gotchas

### Future standalone runner

- a later runtime wrapper can drive repeated Codex iterations
- it should orchestrate `ralph-loop`, not redefine the workflow

## Data flow

1. create markdown PRD
2. convert PRD into `prd.json`
3. start one iteration
4. read `prd.json` and `progress.txt`
5. select highest-priority incomplete story
6. implement exactly one story
7. run checks
8. if passing, commit and mark story complete
9. append progress
10. repeat until `COMPLETE`

## Invariants

- one story at a time
- no completion on failed checks
- file contract remains unchanged
- reusable knowledge goes into `progress.txt` and nearby `AGENTS.md`
