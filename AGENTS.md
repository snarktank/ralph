# Ralph Codex Workbench

This repo is the upstream Ralph source plus a local Codex adaptation workbench.

Use it for two things:

- understanding the original Ralph behavior from `CLAUDE.md`, `ralph.sh`,
  `skills/prd/SKILL.md`, and `skills/ralph/SKILL.md`
- building and validating the Codex-native adaptation without losing fidelity to the
  original one-story loop

## Read order

1. Read `README.md`.
2. Read:
   - `CLAUDE.md`
   - `skills/prd/SKILL.md`
   - `skills/ralph/SKILL.md`
   - `ralph.sh`
   - `prd.json.example`
3. Read:
   - `docs/product/README.md`
   - `docs/product/01_prd.md`
   - `docs/product/02_hld.md`
   - `docs/product/03_lld.md`
4. For current adaptation state and handoff context, read:
   - `docs/issues/0000-discovery-journal.md`
   - `docs/issues/0001-upstream-behavior-audit.md`

## Local Codex skills

Use the installed local Codex skills instead of re-deriving the workflow from scratch
every time:

- `ralph-prd`
- `ralph-loop`

These are the Codex-native equivalents of:

- upstream `skills/prd/SKILL.md`
- upstream `CLAUDE.md` plus `skills/ralph/SKILL.md`

## Core behavior to preserve

- one story per iteration
- `prd.json` is the task ledger
- `progress.txt` is append-only memory
- reusable patterns belong at the top of `progress.txt`
- nearby `AGENTS.md` files should capture genuinely reusable local knowledge
- commit only after checks pass
- output `COMPLETE` only when all stories pass

## Doc routing

- `docs/product/`
  - Codex adaptation specs
- `docs/issues/`
  - behavior mismatches, discoveries, and the UTC handoff journal
- `docs/tests/`
  - validation procedures and adaptation checks
- `docs/tasks/`
  - bounded adaptation work items
- `docs/tasks/done/`
  - completed work records
- `docs/gaps/`
  - unresolved runtime or product gaps, especially the future standalone Codex runner

## Handoff discipline

After each meaningful work loop, append a UTC entry to:

- `docs/issues/0000-discovery-journal.md`

Minimum fields:

- `utc_timestamp`
- `focus`
- `what_was_done`
- `files_or_areas_touched`
- `what_changed_in_understanding`
- `next_step`

## Important constraints

- Keep the upstream file contract unchanged in the first pass:
  - `prd.json`
  - `progress.txt`
  - `tasks/prd-<feature>.md`
- The Codex adaptation should stay faithful to Ralph, not become a generic autonomous
  agent framework.
- There is no confirmed local `codex` CLI/runtime contract here yet, so the standalone
  loop runner remains a documented future gap rather than an implemented shell loop.
