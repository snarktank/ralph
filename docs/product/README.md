# Ralph Codex Adaptation Docs

This folder is the implementation-facing spec layer for the local Codex adaptation of
Ralph.

## What this folder is for

Use these docs to understand how Ralph’s upstream Claude/Amp workflow is being
translated into Codex:

- `01_prd.md`
- `02_hld.md`
- `03_lld.md`
- `04_sources_crosswalk.md`

## What this folder is not

This is not the upstream source of truth.

Upstream behavior still comes from:

- `CLAUDE.md`
- `ralph.sh`
- `skills/prd/SKILL.md`
- `skills/ralph/SKILL.md`
- `prd.json.example`

## Current adaptation boundary

The current pass delivers:

- local Codex skills
- repo-local Codex workflow guidance
- a documented future standalone runner spec gap

It does not yet deliver a runnable non-interactive Codex loop runner.
