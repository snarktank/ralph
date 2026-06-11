#!/usr/bin/env python3
"""``ralph schedule`` — opt-in cron scheduling for the Ralph loop (Block H).

  ralph schedule "0 2 * * *"     install: run the loop nightly at 02:00 (H.1)
  ralph schedule                 print the current scheduled entry, or "none" (H.3)
  ralph schedule --off           remove the fork's entry, leave others alone (H.2)

The installed crontab line invokes the launch wrapper (ralph-run.sh), which
mints a fresh RALPH_RUN_ID per run and, on completion, dumps the full audit to
rundown/YYYY-MM-DD.txt (H.4). No daemon, no always-on process — cron only (H.5).

Our entry is identified by a trailing ``# RALPH-CONTROL`` marker so we only ever
touch our own line. Stdlib only.
"""

from __future__ import annotations

import argparse
import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
WRAPPER = os.path.join(HERE, "ralph-run.sh")
RUNDOWN_DIR = os.path.join(HERE, "rundown")
TAG = "# RALPH-CONTROL"


def _read_crontab() -> str:
    res = subprocess.run(["crontab", "-l"], capture_output=True, text=True)
    # `crontab -l` exits non-zero with "no crontab" when empty — treat as blank.
    if res.returncode != 0 and "no crontab" not in (res.stderr or "").lower():
        if res.stdout.strip() == "" and res.stderr.strip():
            return ""
    return res.stdout


def _write_crontab(text: str) -> None:
    text = text if text.endswith("\n") or text == "" else text + "\n"
    proc = subprocess.run(["crontab", "-"], input=text, text=True,
                          capture_output=True)
    if proc.returncode != 0:
        raise RuntimeError(f"failed to write crontab: {proc.stderr.strip()}")


def _lines_without_ours(text: str) -> list[str]:
    return [ln for ln in text.splitlines() if TAG not in ln]


def current_entry(text: str | None = None) -> str | None:
    text = _read_crontab() if text is None else text
    for ln in text.splitlines():
        if TAG in ln:
            return ln
    return None


def _validate_cron(expr: str) -> None:
    fields = expr.split()
    if len(fields) != 5:
        raise ValueError(
            f"cron expression must have 5 fields (m h dom mon dow); got {len(fields)}: "
            f"{expr!r}"
        )


def build_command(tool: str, iterations: int) -> str:
    os.makedirs(RUNDOWN_DIR, exist_ok=True)
    log = os.path.join(RUNDOWN_DIR, "cron.log")
    # ralph-run.sh mints a fresh RALPH_RUN_ID itself, so each cron firing is a
    # distinct, separately-attributable loop run.
    return (
        f"cd {HERE} && RALPH_TOOL={tool} {WRAPPER} --tool {tool} {iterations} "
        f">> {log} 2>&1 {TAG}"
    )


def install(expr: str, tool: str, iterations: int) -> str:
    _validate_cron(expr)
    if not os.path.exists(WRAPPER):
        raise FileNotFoundError(f"launch wrapper not found: {WRAPPER}")
    others = _lines_without_ours(_read_crontab())
    line = f"{expr} {build_command(tool, iterations)}"
    new = "\n".join([*others, line]).strip("\n") + "\n"
    _write_crontab(new)
    return line


def off() -> bool:
    text = _read_crontab()
    if current_entry(text) is None:
        return False
    others = _lines_without_ours(text)
    _write_crontab(("\n".join(others).strip("\n") + "\n") if others else "")
    return True


def main(argv=None) -> int:
    p = argparse.ArgumentParser(prog="ralph schedule", description=__doc__,
                                formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("cron", nargs="?", help='5-field cron expression, e.g. "0 2 * * *"')
    p.add_argument("--off", action="store_true", help="remove the fork's crontab entry")
    p.add_argument("--tool", default="claude", choices=["claude", "amp"])
    p.add_argument("--iterations", type=int, default=10)
    args = p.parse_args(argv)

    if args.off:
        removed = off()
        print("Removed Ralph schedule." if removed else "No Ralph schedule to remove.")
        return 0

    if not args.cron:
        entry = current_entry()
        print(entry if entry else "No Ralph schedule installed (none).")
        return 0

    try:
        line = install(args.cron, args.tool, args.iterations)
    except (ValueError, FileNotFoundError, RuntimeError) as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 1
    print("Installed Ralph schedule:")
    print(f"  {line}")
    print(f"A dated rundown will be written to {RUNDOWN_DIR}/ after each run.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
