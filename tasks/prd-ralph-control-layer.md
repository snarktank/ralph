# PRD: Ralph Control Layer (fork of `snarktank/ralph`)

Human-intent mirror of the spec compiled into `prd.json`. The control layer is a
**trust-and-control wrapper** around the inherited autonomous loop. `ralph.sh` is
never modified — every addition attaches beside it.

## Summary

Four capabilities on top of the upstream loop:

1. **Skip-then-block** — the loop stops starving its queue on one hard task.
2. **Audit-as-rundown** — one command that is also the morning report.
3. **`/add-feature`** — a `/prd`-style skill to append work mid-project.
4. **Cost** — capture, ongoing analysis, and forward estimates of spend/loops.
5. **Opt-in scheduling** — one command to cron the loop and dump a dated rundown.

## Goals

- Proceed past an individually-stuck task instead of burning iterations on it.
- Produce one honest morning artifact: % complete, done, to-do, blocked (reasons).
- Make adding a feature mid-project a one-command operation.
- Track spend accurately even with parallel/ad-hoc sessions; project remaining
  loops and cost as the run progresses.
- Keep every addition standalone, stdlib-Python, runnable via an alias.

## Non-goals

- Editing `ralph.sh`. New harness adapters. A preventive verification gate
  (upstream already runs typecheck/tests pre-commit). Triage/approval/pending
  state. A standalone `bottleneck.md`. A runtime/uptime sys-ops agent.

## Guiding principles

- **Overnight-trust test** — every feature must increase how confidently you can
  leave the loop running unattended.
- **Dumb execution plane** — intelligence lives in the agent + spec, not the
  orchestrator.
- **Honest numbers** — noisy estimates are labelled ranges, never false points.
- **Don't touch `ralph.sh`.**

## Architecture (three planes)

- **Spec plane** — `tasks/prd-*.md` (intent), `prd.json` (machine ledger), tests
  (contract), `features.md` (appended-work mirror).
- **Execution plane** — `ralph.sh`, unchanged.
- **Control plane** — this fork: skip-then-block (CLAUDE.md + data model), the
  audit/rundown (`project_audit.py`), `/add-feature`, the cost layer
  (`cost_hook.py` + `ralph_cost.py`), the scheduler (`ralph_schedule.py`).

| Piece | Attaches via |
|---|---|
| Skip-then-block | `CLAUDE.md` + `prd.json` fields; audit backstop |
| Audit / rundown | `project_audit.py` (`audit` alias) |
| `/add-feature` | `skills/add-feature/` |
| Cost capture | SessionEnd hook in `.claude/settings.json` → `cost_hook.py` |
| Cost analysis | `ralph_cost.py` (`ralph cost`) reading `cost.jsonl` |
| Scheduling | `ralph_schedule.py` (`ralph schedule`) + `ralph-run.sh` |

## Data model (`prd.json` item additions)

`{meta, items}` envelope. Each item adds three optional, backward-compatible
fields (absent ⇒ default): `attempts` (int, 0), `blocked` (bool, false),
`blockReason` (string, ""). `meta.maxAttempts` is K (default 3).

## Feature blocks → items

Blocks A–I, each acceptance criterion is one `prd.json` item:

- **A** Fork & baseline preservation (A.1 fork, A.2 ralph.sh byte-identical, A.3 clean loop run).
- **B** Data model extension (B.1 defaults, B.2 example, B.3 schema note).
- **C** Skip-then-block (C.1 increment attempts, C.2 self-block at K, C.3 never select blocked, C.4 audit backstop, C.5 test-first discipline).
- **D** Audit / rundown (D.1 blocked section, D.2 partition, D.3 distinct bars, D.4 cost line, D.5 no-arg full rundown).
- **E** `/add-feature` (E.1 generate items, E.2 append-only, E.3 features.md, E.4 immediate, E.5 MVP discipline).
- **F** Cost analysis (F.1 ongoing, F.2 run_id-tagged/--all, F.3 loops-remaining, F.4 cost-remaining, F.5 cold/warm labels, F.6 diminishing-returns signal).
- **G** Cost capture (G.1 SessionEnd hook, G.2 authoritative cost, G.3 run-id tagging, G.4 untagged ad-hoc, G.5 atomic concurrent writes, G.6 line fields).
- **H** Scheduling (H.1 install, H.2 --off clean, H.3 print current, H.4 dated rundown, H.5 cron-only).
- **I** Authoring skills (I.1 carried over, I.2 /prd MVP hardening).

## Definition of done

A trivial seeded project runs end-to-end: PRD compiled, loop run, an item made
unsolvable, the run proceeds past it (blocked with reason) to finish the rest;
the rundown shows % complete + done + active + the blocked item + a cost line;
`ralph cost` reports cost-per-green and both estimates with cold/warm labels;
`ralph schedule` installs/removes a cron entry and produces a dated rundown;
`/add-feature` appends an item + features.md entry; `git diff` vs the pinned
upstream commit shows `ralph.sh` unchanged.
