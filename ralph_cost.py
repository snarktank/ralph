#!/usr/bin/env python3
"""``ralph cost`` — spend analysis & forecasting for the Ralph control layer.

Reads the append-only ``cost.jsonl`` (one line per agent session, written by the
SessionEnd hook in cost_hook.py) and prints:

  * Ongoing analysis  — cumulative spend, burn rate, cost-per-green (F.1).
  * Loops-remaining   — count(active) x expected-attempts-per-item (F.3).
  * Cost-remaining    — loops-remaining x cost-per-loop (F.4).

Loop metrics are computed only over lines tagged with a ``run_id`` (loop spend),
keeping them separable from untagged ad-hoc / co-pilot sessions (F.2). Pass
``--all`` to include untagged sessions in the spend total.

Every forecast is explicitly labelled ``cold-prior`` vs ``warm-empirical`` and
noisy figures are rendered as ranges, never false point estimates (F.5).

Diminishing-returns signal (F.6): watch *cost-per-green*. A blowout means the
loop is spending more and more per shipped item -- it has stopped earning its
keep, and it is time to intervene (split the item, fix the tests, or stop).

Stdlib only.
"""

from __future__ import annotations

import argparse
import json
import os
import sys

import ralph_prd

HERE = os.path.dirname(os.path.abspath(__file__))
DEFAULT_COST_PATH = os.path.join(HERE, "cost.jsonl")

# Cold priors used before enough real data exists (labelled as priors on output).
COLD_PRIOR_COST_PER_ITER = 0.50   # USD per loop iteration, before burn-in
COLD_PRIOR_COST_SPREAD = 0.5      # +/- 50% range around any cold figure
BURN_IN_ITERS = 4                 # iterations before swapping cold -> warm (F.4)


def load_lines(path: str = DEFAULT_COST_PATH, include_untagged: bool = False):
    """Return parsed cost.jsonl rows. Skips malformed lines defensively so a
    single bad write can never break analysis. ``include_untagged=False``
    keeps only loop-tagged (run_id) lines (F.2)."""
    rows = []
    if not os.path.exists(path):
        return rows
    with open(path, encoding="utf-8") as fh:
        for line in fh:
            line = line.strip()
            if not line:
                continue
            try:
                row = json.loads(line)
            except json.JSONDecodeError:
                continue
            if not isinstance(row, dict):
                continue
            if not include_untagged and not row.get("run_id"):
                continue
            rows.append(row)
    return rows


def _num(row, key):
    try:
        return float(row.get(key) or 0)
    except (TypeError, ValueError):
        return 0.0


def totals(rows):
    cost = sum(_num(r, "cost_usd") for r in rows)
    return {
        "sessions": len(rows),
        "cost_usd": cost,
        "input_tokens": sum(_num(r, "input_tokens") for r in rows),
        "output_tokens": sum(_num(r, "output_tokens") for r in rows),
        "cache_read_tokens": sum(_num(r, "cache_read_tokens") for r in rows),
    }


def fmt_money(x: float) -> str:
    return f"${x:,.2f}"


def fmt_range(low: float, high: float, money: bool = True) -> str:
    f = fmt_money if money else (lambda v: f"{v:.1f}")
    return f"{f(low)}–{f(high)}"


def cost_per_green(loop_rows, prd: ralph_prd.Prd):
    """Tagged loop spend / number of green items (F.1).

    Greens are items currently ``passes:true``; per the PRD the precise
    item<->session attribution is derived here rather than stamped per line.
    Returns ``None`` when there are no greens yet (avoids divide-by-zero)."""
    greens = sum(1 for it in prd.items if it.passes)
    if greens == 0:
        return None
    return totals(loop_rows)["cost_usd"] / greens


def loops_remaining(prd: ralph_prd.Prd):
    """(estimate dict) count(active) x expected-attempts-per-item (F.3).

    Multiplier is the cold prior (meta.coldPriorAttemptsPerItem, ~1.5) until at
    least one item is green, then the empirical mean attempts/green item."""
    active = sum(1 for it in prd.items if it.active)
    completed = [it for it in prd.items if it.passes]
    if completed:
        mean_attempts = sum(max(1, it.attempts) for it in completed) / len(completed)
        basis = "warm-empirical"
        mult = mean_attempts
    else:
        mult = prd.cold_prior_attempts
        basis = "cold-prior"
    point = active * mult
    # Render as a range: tighter (+/-20%) when warm, wider (+/-40%) when cold.
    spread = 0.20 if basis == "warm-empirical" else 0.40
    return {
        "active": active,
        "multiplier": mult,
        "basis": basis,
        "low": point * (1 - spread),
        "high": point * (1 + spread),
        "point": point,
    }


def cost_remaining(prd: ralph_prd.Prd, loop_rows):
    """(estimate dict) loops-remaining x cost-per-loop (F.4).

    Cold prior range for the first BURN_IN_ITERS completed iterations, then the
    empirical per-iteration mean with its observed spread."""
    loops = loops_remaining(prd)
    n_iters = len(loop_rows)
    if n_iters >= BURN_IN_ITERS:
        costs = sorted(_num(r, "cost_usd") for r in loop_rows)
        mean = sum(costs) / len(costs)
        lo_iter, hi_iter = costs[0], costs[-1]   # observed envelope
        basis = "warm-empirical"
    else:
        mean = COLD_PRIOR_COST_PER_ITER
        lo_iter = mean * (1 - COLD_PRIOR_COST_SPREAD)
        hi_iter = mean * (1 + COLD_PRIOR_COST_SPREAD)
        basis = "cold-prior"
    return {
        "basis": basis,
        "low": loops["low"] * lo_iter,
        "high": loops["high"] * hi_iter,
        "point": loops["point"] * mean,
        "cost_per_iter": mean,
        "iters_seen": n_iters,
    }


def one_line_summary(prd_path: str | None = None, cost_path: str = DEFAULT_COST_PATH):
    """One-line cost summary for the audit/rundown (D.4).

    'spend so far . burn rate . cost-per-green', or 'no cost data yet' when
    cost.jsonl is absent/empty."""
    rows = load_lines(cost_path, include_untagged=False)
    if not rows:
        return "Cost: no cost data yet (cost.jsonl absent or empty)"
    t = totals(rows)
    burn = t["cost_usd"] / t["sessions"] if t["sessions"] else 0.0
    try:
        prd = ralph_prd.load_prd(prd_path)
        cpg = cost_per_green(rows, prd)
    except (FileNotFoundError, ValueError):
        cpg = None
    cpg_s = fmt_money(cpg) if cpg is not None else "n/a (no greens yet)"
    return (
        f"Cost: {fmt_money(t['cost_usd'])} spend so far · "
        f"{fmt_money(burn)}/loop burn rate · "
        f"{cpg_s} per green"
    )


def render_report(prd: ralph_prd.Prd, cost_path: str, include_all: bool) -> str:
    loop_rows = load_lines(cost_path, include_untagged=False)
    out = ["=" * 60, "RALPH COST", "=" * 60]

    if not loop_rows and not (include_all and load_lines(cost_path, True)):
        out.append("No cost data yet (cost.jsonl absent or empty).")
        out.append("The SessionEnd hook writes one line per agent session.")
        return "\n".join(out)

    # --- Ongoing analysis (F.1) ---
    lt = totals(loop_rows)
    burn = lt["cost_usd"] / lt["sessions"] if lt["sessions"] else 0.0
    cpg = cost_per_green(loop_rows, prd)
    out += [
        "",
        "Ongoing (loop spend, run_id-tagged only):",
        f"  cumulative spend : {fmt_money(lt['cost_usd'])} over {lt['sessions']} loop session(s)",
        f"  burn rate        : {fmt_money(burn)} / loop",
        f"  cost-per-green   : {fmt_money(cpg) if cpg is not None else 'n/a (no greens yet)'}"
        "   <- diminishing-returns signal; a blowout means the loop stopped earning its keep",
    ]

    if include_all:
        at = totals(load_lines(cost_path, include_untagged=True))
        out.append(
            f"  all sessions     : {fmt_money(at['cost_usd'])} over {at['sessions']} "
            "session(s) incl. untagged ad-hoc/co-pilot"
        )

    # --- Forecasts (F.3, F.4, F.5) ---
    loops = loops_remaining(prd)
    costr = cost_remaining(prd, loop_rows)
    out += [
        "",
        "Forecast:",
        f"  loops remaining  : {fmt_range(loops['low'], loops['high'], money=False)} loops "
        f"[{loops['basis']}]  "
        f"({loops['active']} active items x {loops['multiplier']:.2f} attempts/item)",
        f"  cost remaining   : {fmt_range(costr['low'], costr['high'])} "
        f"[{costr['basis']}]  "
        f"(~{fmt_money(costr['cost_per_iter'])}/loop, {costr['iters_seen']} iters observed)",
    ]
    if costr["basis"] == "cold-prior" or loops["basis"] == "cold-prior":
        out.append(
            f"  note: cold-prior figures are priors, not measurements; they tighten "
            f"after burn-in ({BURN_IN_ITERS} loop iterations)."
        )
    return "\n".join(out)


def main(argv=None) -> int:
    p = argparse.ArgumentParser(prog="ralph cost", description=__doc__,
                                formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("--all", action="store_true",
                   help="include untagged ad-hoc/co-pilot sessions in the spend total (F.2)")
    p.add_argument("--prd", default=None, help="path to prd.json")
    p.add_argument("--cost", default=DEFAULT_COST_PATH, help="path to cost.jsonl")
    args = p.parse_args(argv)
    try:
        prd = ralph_prd.load_prd(args.prd)
    except FileNotFoundError:
        # Cost analysis still works without a ledger; greens/forecasts degrade.
        prd = ralph_prd.Prd(meta={}, items=[])
    print(render_report(prd, args.cost, args.all))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
