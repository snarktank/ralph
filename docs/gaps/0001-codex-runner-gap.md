# Codex Runner Gap

## Missing piece

Ralph’s upstream `ralph.sh` repeatedly spawns `amp` or `claude` non-interactively.

There is no confirmed local `codex` CLI/runtime contract available in this environment
to replace that behavior 1:1 yet.

## What is already solved

- Codex-native PRD generation skill
- Codex-native one-story loop skill
- repo-local Codex workbench documentation

## What remains for future work

- define the invocation contract for a standalone Codex runner
- decide how the runner executes repeated isolated iterations
- preserve:
  - max-iteration limit
  - branch discipline
  - progress logging
  - story completion updates
  - `COMPLETE` stop condition
