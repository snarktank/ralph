#!/usr/bin/env python3
"""Shared PRD ledger helpers for the Ralph control layer.

The control-plane scripts (project_audit.py, ralph_cost.py, ralph_schedule.py)
all read prd.json through this module so the data-model semantics live in one
place. Stdlib only.

prd.json schema (the fork's ``{meta, items}`` envelope)::

    {
      "meta": {
        "project": "ralph-control",
        "branchName": "...",
        "maxAttempts": 3,            # K, the attempt ceiling (section 11.3)
        "coldPriorAttemptsPerItem": 1.5,
        "v1Scope": ["A", "B", ...],
        "deferred": {"blocks": []},
        "upstream": {"repo": "snarktank/ralph", "pinnedCommit": "..."}
      },
      "items": [
        {
          "id": "B.1",
          "block": "B",
          "blockName": "Data model extension",
          "description": "...",
          "passes": false,
          "attempts": 0,        # optional, default 0  (Block B.1)
          "blocked": false,     # optional, default false
          "blockReason": ""     # optional, default ""
        }
      ]
    }

Items lacking ``attempts`` / ``blocked`` / ``blockReason`` are read as
``0 / False / ""`` with no error (acceptance criterion B.1).
"""

from __future__ import annotations

import json
import os
from dataclasses import dataclass, field
from typing import Any

DEFAULT_MAX_ATTEMPTS = 3
DEFAULT_COLD_PRIOR_ATTEMPTS = 1.5

HERE = os.path.dirname(os.path.abspath(__file__))
DEFAULT_PRD_PATH = os.path.join(HERE, "prd.json")


@dataclass
class Item:
    """One PRD item with the three optional fork fields defaulted (B.1)."""

    id: str
    block: str
    block_name: str
    description: str
    passes: bool
    attempts: int = 0
    blocked: bool = False
    block_reason: str = ""
    raw: dict[str, Any] = field(default_factory=dict, repr=False)

    @property
    def active(self) -> bool:
        """Eligible to be picked as the next task: not done, not blocked."""
        return not self.passes and not self.blocked

    @property
    def remaining(self) -> bool:
        """Not yet green (whether or not it is blocked)."""
        return not self.passes


@dataclass
class Prd:
    meta: dict[str, Any]
    items: list[Item]

    @property
    def max_attempts(self) -> int:
        try:
            return int(self.meta.get("maxAttempts", DEFAULT_MAX_ATTEMPTS))
        except (TypeError, ValueError):
            return DEFAULT_MAX_ATTEMPTS

    @property
    def cold_prior_attempts(self) -> float:
        try:
            return float(
                self.meta.get("coldPriorAttemptsPerItem", DEFAULT_COLD_PRIOR_ATTEMPTS)
            )
        except (TypeError, ValueError):
            return DEFAULT_COLD_PRIOR_ATTEMPTS


def _coerce_item(raw: dict[str, Any]) -> Item:
    return Item(
        id=str(raw.get("id", "")),
        block=str(raw.get("block", "")),
        block_name=str(raw.get("blockName", raw.get("block_name", ""))),
        description=str(raw.get("description", "")),
        passes=bool(raw.get("passes", False)),
        attempts=int(raw.get("attempts", 0) or 0),
        blocked=bool(raw.get("blocked", False)),
        block_reason=str(raw.get("blockReason", raw.get("block_reason", "")) or ""),
        raw=raw,
    )


def load_prd(path: str | None = None) -> Prd:
    """Load prd.json into a :class:`Prd`. Supports the ``{meta, items}``
    envelope; tolerates a bare list or the legacy ``userStories`` key."""
    path = path or DEFAULT_PRD_PATH
    with open(path, encoding="utf-8") as fh:
        data = json.load(fh)

    if isinstance(data, list):
        return Prd(meta={}, items=[_coerce_item(x) for x in data])

    meta = data.get("meta", {}) or {}
    raw_items = data.get("items")
    if raw_items is None:
        # Legacy upstream shape used "userStories"; map it through so the audit
        # still works against an un-migrated ledger.
        raw_items = data.get("userStories", [])
        if not meta:
            meta = {k: v for k, v in data.items() if k != "userStories"}
    items = [_coerce_item(x) for x in raw_items]
    return Prd(meta=meta, items=items)


def partition(prd: Prd) -> dict[str, list[Item]]:
    """Split items into done / active / blocked. ``active`` never contains a
    blocked item (C.3, D.2); blocked items are surfaced separately (D.1)."""
    done, active, blocked = [], [], []
    for item in prd.items:
        if item.passes:
            done.append(item)
        elif item.blocked:
            blocked.append(item)
        else:
            active.append(item)
    return {"done": done, "active": active, "blocked": blocked}


def backstop_violations(prd: Prd) -> list[Item]:
    """Detective backstop (C.4): items that hit the attempt ceiling but were
    never self-blocked by the agent."""
    k = prd.max_attempts
    return [
        it
        for it in prd.items
        if not it.passes and not it.blocked and it.attempts >= k
    ]


def first_pass_rate(prd: Prd) -> float | None:
    """Empirical first-pass success rate over completed items: fraction of
    green items that needed <= 1 attempt. ``None`` until at least one item is
    green (so callers can fall back to the cold prior)."""
    completed = [it for it in prd.items if it.passes]
    if not completed:
        return None
    first_try = sum(1 for it in completed if it.attempts <= 1)
    return first_try / len(completed)
