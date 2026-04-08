# Ralph Codex Adaptation PRD

## Product

Create a Codex-native adaptation of Ralph that preserves the one-story autonomous loop
discipline and the original file contract.

## Purpose

The adaptation should:

- preserve Ralph’s one-story-per-iteration model
- preserve `prd.json` as the execution ledger
- preserve `progress.txt` as append-only memory
- preserve nearby `AGENTS.md` updates as reusable repo knowledge
- let Codex users run the Ralph workflow without depending on Claude-specific prompts

## Users

- Codex users running Ralph-style iterative development
- repo owners who want a PRD-to-loop workflow in Codex
- future implementers of a standalone Codex runner

## Required capabilities

- PRD generation into `tasks/prd-<feature>.md`
- PRD-to-`prd.json` conversion
- one-story loop discipline against `prd.json`
- progress and pattern logging in `progress.txt`
- nearby `AGENTS.md` learning capture
- branch/check/commit discipline

## Non-goals for this pass

- replacing Ralph’s file contract
- inventing a different task ledger
- implementing a standalone Codex CLI runner before a runtime contract exists
- turning Ralph into a generic autonomous framework

## Success criteria

- local Codex skills exist for PRD generation and the one-story loop
- the cloned Ralph repo documents the Codex adaptation clearly
- every major behavior from the upstream Claude/Amp workflow is either:
  - implemented as a local Codex skill/workflow
  - or recorded as an explicit future runner gap
