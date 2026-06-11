#!/usr/bin/env python3
"""Tests for the Ralph control layer. Stdlib unittest only.

Run:  python3 -m unittest discover -s tests -v
"""

import json
import os
import sys
import tempfile
import unittest
from concurrent.futures import ProcessPoolExecutor

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)
sys.path.insert(0, ROOT)

import ralph_prd          # noqa: E402
import ralph_cost         # noqa: E402
import ralph_schedule     # noqa: E402
import cost_hook          # noqa: E402
import project_audit      # noqa: E402


def _prd_dict(items):
    return {"meta": {"maxAttempts": 3, "coldPriorAttemptsPerItem": 1.5}, "items": items}


def _write_json(obj):
    fd, path = tempfile.mkstemp(suffix=".json")
    with os.fdopen(fd, "w") as fh:
        json.dump(obj, fh)
    return path


# Module-level worker for the concurrent-write test (spawn-safe).
def _hammer(args):
    path, worker_id, n = args
    for i in range(n):
        cost_hook.append_atomic(
            {"ts": "t", "run_id": f"w{worker_id}", "session_id": f"{worker_id}-{i}",
             "tool": "claude", "model": "m", "cost_usd": 0.01,
             "input_tokens": 1, "output_tokens": 1, "cache_read_tokens": 0},
            path,
        )
    return n


class TestDataModel(unittest.TestCase):
    """Block B.1 — optional fields default cleanly."""

    def test_absent_fields_default(self):
        path = _write_json(_prd_dict([
            {"id": "X.1", "block": "X", "blockName": "x", "description": "d",
             "passes": False},  # no attempts/blocked/blockReason
        ]))
        prd = ralph_prd.load_prd(path)
        it = prd.items[0]
        self.assertEqual(it.attempts, 0)
        self.assertFalse(it.blocked)
        self.assertEqual(it.block_reason, "")
        os.unlink(path)

    def test_present_fields_read(self):
        path = _write_json(_prd_dict([
            {"id": "X.1", "block": "X", "blockName": "x", "description": "d",
             "passes": False, "attempts": 3, "blocked": True, "blockReason": "stuck"},
        ]))
        it = ralph_prd.load_prd(path).items[0]
        self.assertEqual(it.attempts, 3)
        self.assertTrue(it.blocked)
        self.assertEqual(it.block_reason, "stuck")
        os.unlink(path)

    def test_legacy_userstories_shape(self):
        path = _write_json({"branchName": "b", "userStories": [
            {"id": "US-1", "title": "t", "description": "d", "passes": True}]})
        prd = ralph_prd.load_prd(path)
        self.assertEqual(len(prd.items), 1)
        self.assertTrue(prd.items[0].passes)
        os.unlink(path)


class TestPartitionAndBackstop(unittest.TestCase):
    """C.3, C.4, D.2 — selection never picks blocked; backstop catches stragglers."""

    def setUp(self):
        self.prd = ralph_prd.Prd(meta={"maxAttempts": 3}, items=[
            ralph_prd.Item("A", "A", "a", "done", passes=True, attempts=1),
            ralph_prd.Item("B", "A", "a", "active", passes=False, attempts=1),
            ralph_prd.Item("C", "A", "a", "blocked", passes=False, blocked=True,
                           attempts=3, block_reason="nope"),
            ralph_prd.Item("D", "A", "a", "straggler", passes=False, attempts=4),
        ])

    def test_active_excludes_blocked(self):
        parts = ralph_prd.partition(self.prd)
        active_ids = {i.id for i in parts["active"]}
        self.assertIn("B", active_ids)
        self.assertNotIn("C", active_ids)        # blocked never active (C.3)
        self.assertEqual({i.id for i in parts["blocked"]}, {"C"})
        self.assertEqual({i.id for i in parts["done"]}, {"A"})

    def test_backstop_flags_unblocked_over_ceiling(self):
        v = ralph_prd.backstop_violations(self.prd)
        ids = {i.id for i in v}
        self.assertIn("D", ids)      # attempts 4 >= K=3, not blocked (C.4)
        self.assertNotIn("C", ids)   # already blocked, not a violation


class TestAuditRender(unittest.TestCase):
    """Block D — rundown content."""

    def setUp(self):
        self.prd = ralph_prd.Prd(meta={"project": "p", "maxAttempts": 3}, items=[
            ralph_prd.Item("A", "A", "Alpha", "done thing", passes=True, attempts=1),
            ralph_prd.Item("B", "A", "Alpha", "active thing", passes=False),
            ralph_prd.Item("C", "B", "Beta", "blocked thing", passes=False,
                           blocked=True, attempts=3, block_reason="needs human"),
        ])

    def test_rundown_sections(self):
        out = project_audit.render(self.prd, "/nonexistent/progress.txt", None, None)
        self.assertIn("UP NEXT", out)
        self.assertIn("active thing", out)
        self.assertIn("BLOCKED / SKIPPED", out)
        self.assertIn("needs human", out)          # D.1 reason shown
        self.assertNotIn("blocked thing", out.split("UP NEXT")[1].split("BLOCKED")[0])  # D.2
        self.assertIn("no cost data yet", out)      # D.4 fallback

    def test_progress_bar_counts_blocked_distinctly(self):
        # 1 done, 1 blocked, 3 total -> bar shows blocked glyph, not dropped (D.3)
        bar = project_audit.bar(done=1, blocked=1, total=3)
        self.assertIn("▒", bar)
        self.assertIn("1 blocked", bar)


class TestCost(unittest.TestCase):
    """Block F — analysis + forecasts."""

    def _cost_file(self, rows):
        fd, path = tempfile.mkstemp(suffix=".jsonl")
        with os.fdopen(fd, "w") as fh:
            for r in rows:
                fh.write(json.dumps(r) + "\n")
        return path

    def test_run_id_filtering(self):
        path = self._cost_file([
            {"run_id": "r1", "cost_usd": 1.0},
            {"run_id": None, "cost_usd": 5.0},     # ad-hoc, untagged (G.4/F.2)
        ])
        loop = ralph_cost.load_lines(path, include_untagged=False)
        allrows = ralph_cost.load_lines(path, include_untagged=True)
        self.assertEqual(len(loop), 1)
        self.assertEqual(ralph_cost.totals(loop)["cost_usd"], 1.0)
        self.assertEqual(ralph_cost.totals(allrows)["cost_usd"], 6.0)  # --all (F.2)
        os.unlink(path)

    def test_cost_per_green(self):
        prd = ralph_prd.Prd(meta={}, items=[
            ralph_prd.Item("A", "A", "a", "d", passes=True),
            ralph_prd.Item("B", "A", "a", "d", passes=True),
            ralph_prd.Item("C", "A", "a", "d", passes=False),
        ])
        rows = [{"run_id": "r", "cost_usd": 2.0}, {"run_id": "r", "cost_usd": 2.0}]
        self.assertEqual(ralph_cost.cost_per_green(rows, prd), 2.0)  # 4.0 / 2 greens

    def test_estimates_labelled_cold_then_warm(self):
        cold = ralph_prd.Prd(meta={"coldPriorAttemptsPerItem": 1.5}, items=[
            ralph_prd.Item("A", "A", "a", "d", passes=False)])
        self.assertEqual(ralph_cost.loops_remaining(cold)["basis"], "cold-prior")  # F.5
        warm = ralph_prd.Prd(meta={}, items=[
            ralph_prd.Item("A", "A", "a", "d", passes=True, attempts=1),
            ralph_prd.Item("B", "A", "a", "d", passes=False)])
        self.assertEqual(ralph_cost.loops_remaining(warm)["basis"], "warm-empirical")

    def test_cost_remaining_burn_in(self):
        prd = ralph_prd.Prd(meta={}, items=[ralph_prd.Item("A", "A", "a", "d", passes=False)])
        cold = ralph_cost.cost_remaining(prd, [])                      # 0 iters
        self.assertEqual(cold["basis"], "cold-prior")                 # F.4
        rows = [{"run_id": "r", "cost_usd": 0.4}] * ralph_cost.BURN_IN_ITERS
        warm = ralph_cost.cost_remaining(prd, rows)
        self.assertEqual(warm["basis"], "warm-empirical")
        self.assertLess(cold["low"], cold["high"])                    # always a range

    def test_one_line_summary_no_data(self):
        self.assertIn("no cost data yet",
                      ralph_cost.one_line_summary(cost_path="/nope/cost.jsonl"))


class TestHook(unittest.TestCase):
    """Block G — capture, tagging, concurrency."""

    def test_record_has_required_fields(self):
        os.environ["RALPH_RUN_ID"] = "run-xyz"
        try:
            rec = cost_hook.build_record(
                {"session_id": "s1", "total_cost_usd": 0.42, "model": "claude-x",
                 "usage": {"input_tokens": 10, "output_tokens": 5,
                           "cache_read_input_tokens": 3}})
        finally:
            del os.environ["RALPH_RUN_ID"]
        for k in ("ts", "run_id", "session_id", "tool", "model", "cost_usd",
                  "input_tokens", "output_tokens", "cache_read_tokens"):
            self.assertIn(k, rec)                  # G.6
        self.assertEqual(rec["run_id"], "run-xyz")  # G.3
        self.assertEqual(rec["cost_usd"], 0.42)     # G.2 authoritative
        self.assertEqual(rec["cache_read_tokens"], 3)

    def test_untagged_when_no_run_id(self):
        os.environ.pop("RALPH_RUN_ID", None)
        rec = cost_hook.build_record({"session_id": "s2", "total_cost_usd": 0.1})
        self.assertIsNone(rec["run_id"])           # G.4

    def test_concurrent_append_no_corruption(self):
        fd, path = tempfile.mkstemp(suffix=".jsonl")
        os.close(fd)
        os.unlink(path)  # let the hook create it
        workers, per = 8, 200
        try:
            with ProcessPoolExecutor(max_workers=workers) as ex:
                list(ex.map(_hammer, [(path, w, per) for w in range(workers)]))
            with open(path, encoding="utf-8") as fh:
                lines = [ln for ln in fh.read().splitlines() if ln.strip()]
            self.assertEqual(len(lines), workers * per)   # no lost writes (G.5)
            for ln in lines:
                json.loads(ln)                            # every line intact
        finally:
            if os.path.exists(path):
                os.unlink(path)


class TestSchedule(unittest.TestCase):
    """Block H — cron entry construction and isolation."""

    def test_validate_cron_rejects_bad(self):
        with self.assertRaises(ValueError):
            ralph_schedule._validate_cron("0 2 * *")   # only 4 fields (H.1)
        ralph_schedule._validate_cron("0 2 * * *")     # ok

    def test_build_command_has_wrapper_and_tag(self):
        cmd = ralph_schedule.build_command("claude", 5)
        self.assertIn("ralph-run.sh", cmd)
        self.assertIn(ralph_schedule.TAG, cmd)
        self.assertIn("--tool claude", cmd)

    def test_only_our_line_removed(self):
        crontab = "\n".join([
            "0 1 * * * /other/job.sh",                          # someone else's
            f"0 2 * * * /x/ralph-run.sh 10 {ralph_schedule.TAG}",
        ])
        others = ralph_schedule._lines_without_ours(crontab)
        self.assertEqual(others, ["0 1 * * * /other/job.sh"])   # H.2 isolation
        self.assertIsNotNone(ralph_schedule.current_entry(crontab))  # H.3

    def test_no_entry_reads_none(self):
        self.assertIsNone(ralph_schedule.current_entry("0 1 * * * /other/job.sh"))


if __name__ == "__main__":
    unittest.main()
