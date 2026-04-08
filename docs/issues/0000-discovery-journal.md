# Ralph Codex Adaptation Journal

## 2026-04-08T11:04:42Z

### Focus

Clone the upstream Ralph repo and turn it into a documented Codex adaptation workbench
with local skills and an explicit future-runner gap.

### What was done

- cloned `snarktank/ralph` into `/home/shetautnetjerheru/ralph`
- inspected upstream:
  - `README.md`
  - `CLAUDE.md`
  - `skills/prd/SKILL.md`
  - `skills/ralph/SKILL.md`
  - `ralph.sh`
  - `prd.json.example`
- created local Codex skills:
  - `~/.codex/skills/ralph-prd/SKILL.md`
  - `~/.codex/skills/ralph-loop/SKILL.md`
- turned the cloned repo into a Codex workbench with:
  - root `AGENTS.md`
  - `docs/product/`
  - `docs/issues/`
  - `docs/tests/`
  - `docs/tasks/`
  - `docs/gaps/`

### Files or areas touched

- `AGENTS.md`
- `docs/product/*`
- `docs/issues/*`
- `docs/tests/*`
- `docs/tasks/*`
- `docs/gaps/*`
- `~/.codex/skills/ralph-prd/SKILL.md`
- `~/.codex/skills/ralph-loop/SKILL.md`

### What changed in understanding

The key translation boundary is now clear:

- Codex can preserve Ralph’s one-story workflow as a skill/workbench model today
- the standalone autonomous runner is a separate runtime problem because there is no
  local `codex` CLI/runtime contract available here yet

### Next step

Validate the local Codex skills against the upstream files line by line and record any
behavior drift in `docs/issues/0001-upstream-behavior-audit.md`.
