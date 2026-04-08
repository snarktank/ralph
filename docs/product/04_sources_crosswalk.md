# Ralph Codex Sources Crosswalk

## Upstream sources -> Codex adaptation

- `CLAUDE.md`
  - execution-loop behavior and progress discipline
- `skills/prd/SKILL.md`
  - PRD generation behavior
- `skills/ralph/SKILL.md`
  - PRD-to-`prd.json` conversion rules and story sizing
- `ralph.sh`
  - outer iteration model and stop condition
- `prd.json.example`
  - execution-ledger shape

## Local outputs

- local skill `ralph-prd`
  - upstream PRD behavior translated for Codex
- local skill `ralph-loop`
  - upstream Claude loop behavior translated for Codex
- `docs/product/*`
  - implementation-facing Codex adaptation spec
- `docs/gaps/0001-codex-runner-gap.md`
  - future standalone runner specification gap
