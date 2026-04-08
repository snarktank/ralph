# Upstream Behavior Audit

This note tracks how the Codex adaptation maps to upstream Ralph behavior.

## Preserved directly

- one story per iteration
- `prd.json` as the execution ledger
- `progress.txt` as append-only memory
- codebase-pattern consolidation at the top of `progress.txt`
- nearby `AGENTS.md` updates for reusable local knowledge
- stop with `COMPLETE` only when all stories pass

## Adapted for Codex

- upstream `CLAUDE.md` is translated into the `ralph-loop` skill rather than used as a
  raw prompt file
- upstream PRD generation is translated into the `ralph-prd` skill
- the standalone shell runner is documented as a future gap rather than implemented

## Current known gap

- no local non-interactive `codex` runtime contract exists here, so `ralph.sh` cannot
  be replaced 1:1 yet
