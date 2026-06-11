#!/usr/bin/env python3
"""``audit`` — the Ralph morning rundown.

Run with no arguments it produces the full morning artifact (D.5): percent
complete, the done list, the active to-do, the blocked-with-reasons list, recent
progress, and a one-line cost summary. It is a CLI executable, run via the
``audit`` alias, and is the thing you read after an overnight run.

Sections:
  * Progress bars      — overall + per-block, blocked counted as
                         remaining-but-not-actionable, shown distinctly (D.3).
  * Up Next            — active items only; blocked work never appears here (D.2).
  * Blocked / Skipped  — every blocked item with its blockReason (D.1).
  * Backstop warnings  — items at/over the attempt ceiling not self-blocked (C.4).
  * Recent progress    — tail of progress.txt.
  * Cost               — one line from ralph_cost (D.4).

Stdlib only.
"""

from __future__ import annotations

import argparse
import os

import ralph_prd

HERE = os.path.dirname(os.path.abspath(__file__))
DEFAULT_PROGRESS = os.path.join(HERE, "progress.txt")
BAR_WIDTH = 24


def bar(done: int, blocked: int, total: int, width: int = BAR_WIDTH) -> str:
    """Progress bar where done is solid, blocked is shown distinctly (not
    silently dropped, D.3), and the rest is empty."""
    if total <= 0:
        return "[" + " " * width + "] n/a"
    done_n = round(width * done / total)
    blocked_n = round(width * blocked / total)
    # Never let rounding overflow the bar.
    blocked_n = min(blocked_n, width - done_n)
    empty_n = width - done_n - blocked_n
    bar_str = "█" * done_n + "▒" * blocked_n + "·" * empty_n
    pct = 100 * done / total
    suffix = f"{pct:5.1f}%  ({done}/{total} done"
    if blocked:
        suffix += f", {blocked} blocked"
    suffix += ")"
    return f"[{bar_str}] {suffix}"


def by_block(prd: ralph_prd.Prd):
    blocks: dict[str, dict] = {}
    for it in prd.items:
        b = blocks.setdefault(
            it.block, {"name": it.block_name, "done": 0, "blocked": 0, "total": 0}
        )
        b["total"] += 1
        if it.passes:
            b["done"] += 1
        elif it.blocked:
            b["blocked"] += 1
    return blocks


def tail(path: str, n_chars: int = 1200) -> str:
    if not os.path.exists(path):
        return "(no progress.txt yet)"
    with open(path, encoding="utf-8") as fh:
        data = fh.read()
    return data[-n_chars:].strip() if len(data) > n_chars else data.strip()


def render(prd: ralph_prd.Prd, progress_path: str, cost_path: str | None,
           prd_path: str | None) -> str:
    parts = ralph_prd.partition(prd)
    done, active, blocked = parts["done"], parts["active"], parts["blocked"]
    total = len(prd.items)

    out = ["=" * 64, "RALPH RUNDOWN", "=" * 64]
    meta = prd.meta
    if meta.get("project") or meta.get("branchName"):
        out.append(
            f"{meta.get('project', '?')}  ·  branch {meta.get('branchName', '?')}"
            f"  ·  K={prd.max_attempts}"
        )

    # --- Overall + per-block progress (D.3) ---
    out += ["", "PROGRESS", bar(len(done), len(blocked), total)]
    out.append("legend: █ done   ▒ blocked (remaining, not actionable)   · to-do")
    out.append("")
    for bid in sorted(by_block(prd)):
        b = by_block(prd)[bid]
        label = f"  {bid} {b['name']}"[:34].ljust(34)
        out.append(f"{label} {bar(b['done'], b['blocked'], b['total'], width=16)}")

    # --- Up Next: active only, never blocked (D.2) ---
    out += ["", "UP NEXT (active — blocked items excluded)"]
    if active:
        for it in active[:12]:
            att = f"  [attempts {it.attempts}/{prd.max_attempts}]" if it.attempts else ""
            out.append(f"  ☐ {it.id} — {it.description}{att}")
        if len(active) > 12:
            out.append(f"  … and {len(active) - 12} more active")
    else:
        out.append("  (nothing active — all remaining work is blocked or complete)")

    # --- Blocked / Skipped (D.1) ---
    out += ["", f"BLOCKED / SKIPPED ({len(blocked)})"]
    if blocked:
        for it in blocked:
            reason = it.block_reason or "(no reason recorded)"
            out.append(f"  ✗ {it.id} — {it.description}")
            out.append(f"      reason: {reason}  [attempts {it.attempts}/{prd.max_attempts}]")
    else:
        out.append("  (none)")

    # --- Detective backstop (C.4) ---
    violations = ralph_prd.backstop_violations(prd)
    if violations:
        out += ["", "⚠ BACKSTOP — at/over attempt ceiling but NOT self-blocked (C.4)"]
        for it in violations:
            out.append(
                f"  ! {it.id} — attempts {it.attempts} ≥ K={prd.max_attempts}, "
                f"blocked=false. Agent likely failed to self-block."
            )

    # --- Done list ---
    out += ["", f"DONE ({len(done)}/{total})"]
    if done:
        out.append("  " + ", ".join(it.id for it in done))
    else:
        out.append("  (none yet)")

    # --- Recent progress ---
    out += ["", "RECENT PROGRESS", tail(progress_path)]

    # --- Cost (D.4) ---
    cost_line = "Cost: no cost data yet (cost.jsonl absent or empty)"
    try:
        import ralph_cost
        cp = cost_path or ralph_cost.DEFAULT_COST_PATH
        cost_line = ralph_cost.one_line_summary(prd_path=prd_path, cost_path=cp)
    except Exception:  # cost layer is best-effort for the rundown
        pass
    out += ["", cost_line, "=" * 64]
    return "\n".join(out)


def main(argv=None) -> int:
    p = argparse.ArgumentParser(prog="audit", description=__doc__,
                                formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("--prd", default=None, help="path to prd.json")
    p.add_argument("--progress", default=DEFAULT_PROGRESS, help="path to progress.txt")
    p.add_argument("--cost", default=None, help="path to cost.jsonl")
    args = p.parse_args(argv)

    try:
        prd = ralph_prd.load_prd(args.prd)
    except FileNotFoundError:
        print("No prd.json found — nothing to audit. "
              "Compile a PRD with the /ralph skill first.")
        return 1
    print(render(prd, args.progress, args.cost, args.prd))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
