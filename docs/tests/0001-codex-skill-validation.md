# Codex Skill Validation

Validate the Codex adaptation against upstream Ralph behavior.

## Checkpoints

- `ralph-prd` preserves:
  - right-sized story guidance
  - verifiable acceptance criteria
  - markdown PRD output in `tasks/`
- `ralph-loop` preserves:
  - one story per iteration
  - branch alignment with `branchName`
  - progress append discipline
  - nearby `AGENTS.md` update policy
  - `COMPLETE` stop condition
- `prd.json.example` remains the file-shape reference
- `docs/gaps/0001-codex-runner-gap.md` clearly states what is not implemented yet
