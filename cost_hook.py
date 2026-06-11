#!/usr/bin/env python3
"""Cost-capture hook for the Ralph control layer (Block G).

Wired to Claude Code's **SessionEnd** event in ``.claude/settings.json`` (config
only — ralph.sh is never touched, G.1). Fires once per agent session, reads the
hook payload from stdin, and appends exactly one JSONL line to ``cost.jsonl``
(G.2). The write is a single ``O_APPEND`` os.write of a sub-PIPE_BUF record, so
concurrent sessions cannot interleave/corrupt the file (G.5).

Run-attribution (G.3/G.4): if ``RALPH_RUN_ID`` is set in the environment (the
loop launch wrapper / cron entry sets it and it propagates down the process
tree) the line is tagged with it = loop spend. Ad-hoc/co-pilot sessions have no
``RALPH_RUN_ID`` and are written untagged, so analysis can separate them.

Line shape (G.6 — at minimum ts, run_id, session_id, tool, model, cost_usd,
tokens)::

    {"ts": "...Z", "run_id": "ralph-...", "session_id": "...", "tool": "claude",
     "model": "...", "cost_usd": 0.42, "input_tokens": 0, "output_tokens": 0,
     "cache_read_tokens": 0}

The hook is intentionally *dumb*: it logs raw cost facts only. Item-level
pass/fail attribution is derived later in ralph_cost.py, not stamped here.

NB (PRD §11.1): the exact SessionEnd field names should be confirmed against the
current Claude Code hooks reference. Claude Code reports ``total_cost_usd`` as
the authoritative spend; this hook prefers it and never reconstructs cost from
token×price. We read it from the payload if present, else from the session
transcript, and degrade gracefully (cost_usd=0) rather than ever crashing the
session. Cost extraction is isolated in ``extract_cost_and_tokens`` so the
binding is easy to adjust once confirmed.
"""

from __future__ import annotations

import datetime
import json
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
DEFAULT_COST_PATH = os.path.join(HERE, "cost.jsonl")


def _now_iso() -> str:
    return datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def read_payload() -> dict:
    try:
        raw = sys.stdin.read()
        data = json.loads(raw) if raw.strip() else {}
        return data if isinstance(data, dict) else {}
    except (json.JSONDecodeError, ValueError):
        return {}


def _first(d: dict, *keys, default=None):
    for k in keys:
        if isinstance(d, dict) and d.get(k) is not None:
            return d[k]
    return default


def _scan_transcript(path: str):
    """Best-effort: pull total_cost_usd (last/max seen) and sum token usage from
    a Claude Code transcript JSONL. Returns (cost_usd, tokens-dict)."""
    cost = None
    inp = out = cache = 0
    if not path or not os.path.exists(path):
        return cost, {"input_tokens": 0, "output_tokens": 0, "cache_read_tokens": 0}
    try:
        with open(path, encoding="utf-8") as fh:
            for line in fh:
                line = line.strip()
                if not line:
                    continue
                try:
                    obj = json.loads(line)
                except json.JSONDecodeError:
                    continue
                tc = _first(obj, "total_cost_usd")
                if tc is None and isinstance(obj.get("cost"), dict):
                    tc = obj["cost"].get("total_cost_usd")
                if isinstance(tc, (int, float)):
                    cost = float(tc)  # later lines win (cumulative)
                usage = None
                msg = obj.get("message")
                if isinstance(msg, dict):
                    usage = msg.get("usage")
                usage = usage or obj.get("usage")
                if isinstance(usage, dict):
                    inp += int(usage.get("input_tokens") or 0)
                    out += int(usage.get("output_tokens") or 0)
                    cache += int(
                        usage.get("cache_read_input_tokens")
                        or usage.get("cache_read_tokens")
                        or 0
                    )
    except OSError:
        pass
    return cost, {"input_tokens": inp, "output_tokens": out, "cache_read_tokens": cache}


def extract_cost_and_tokens(payload: dict):
    """Resolve authoritative cost_usd + token counts from the hook payload,
    falling back to the transcript. Isolated so the field binding (§11.1) is
    trivial to update once confirmed against the live hooks reference."""
    cost = _first(payload, "total_cost_usd")
    if cost is None and isinstance(payload.get("cost"), dict):
        cost = payload["cost"].get("total_cost_usd")

    tokens = {"input_tokens": 0, "output_tokens": 0, "cache_read_tokens": 0}
    usage = payload.get("usage")
    if isinstance(usage, dict):
        tokens["input_tokens"] = int(usage.get("input_tokens") or 0)
        tokens["output_tokens"] = int(usage.get("output_tokens") or 0)
        tokens["cache_read_tokens"] = int(
            usage.get("cache_read_input_tokens") or usage.get("cache_read_tokens") or 0
        )

    if cost is None or not any(tokens.values()):
        t_cost, t_tokens = _scan_transcript(payload.get("transcript_path", ""))
        if cost is None:
            cost = t_cost
        if not any(tokens.values()):
            tokens = t_tokens

    try:
        cost = float(cost) if cost is not None else 0.0
    except (TypeError, ValueError):
        cost = 0.0
    return cost, tokens


def build_record(payload: dict) -> dict:
    cost, tokens = extract_cost_and_tokens(payload)
    return {
        "ts": _now_iso(),
        "run_id": os.environ.get("RALPH_RUN_ID") or None,  # G.3 / G.4
        "session_id": _first(payload, "session_id", "sessionId", default=""),
        "tool": os.environ.get("RALPH_TOOL", "claude"),
        "model": _first(payload, "model", default="")
        or os.environ.get("ANTHROPIC_MODEL", ""),
        "cost_usd": round(cost, 6),
        **tokens,
    }


def append_atomic(record: dict, path: str = DEFAULT_COST_PATH) -> None:
    """Single O_APPEND write of one line. The record is small (< PIPE_BUF =
    4096 bytes), so the write is atomic and concurrent sessions never corrupt
    the file (G.5)."""
    line = (json.dumps(record, separators=(",", ":")) + "\n").encode("utf-8")
    fd = os.open(path, os.O_WRONLY | os.O_APPEND | os.O_CREAT, 0o644)
    try:
        os.write(fd, line)
    finally:
        os.close(fd)


def main() -> int:
    payload = read_payload()
    try:
        append_atomic(build_record(payload))
    except Exception:
        # Never break the session because cost logging hiccuped.
        pass
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
