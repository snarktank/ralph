from __future__ import annotations

from pathlib import Path
from typing import Sequence

import pytest
from _pytest.capture import CaptureFixture
from _pytest.monkeypatch import MonkeyPatch

import ralphython


def test_parse_args_requires_agent() -> None:
    with pytest.raises(SystemExit) as exc:
        ralphython._parse_args([])
    assert exc.value.code == 2


def test_parse_args_valid_agent() -> None:
    args = ralphython._parse_args(["--agent", "amp", "3"])
    assert args.agent == "amp"
    assert args.max_iterations == 3


def test_parse_args_tool_deprecated_sets_agent(
    capsys: CaptureFixture[str],
) -> None:
    args = ralphython._parse_args(["--tool", "claude", "2"])
    captured = capsys.readouterr()
    assert "deprecated" in captured.err
    assert args.agent == "claude"
    assert args.max_iterations == 2


def test_prd_copies_before_run(monkeypatch: MonkeyPatch, tmp_path: Path) -> None:
    prd_src = tmp_path / "incoming.json"
    prd_src.write_text('{"branchName":"ralph/test"}')

    monkeypatch.setattr(ralphython, "__file__", str(tmp_path / "ralphython.py"))

    def fake_run(_cmd: Sequence[str], stdin_path: Path | None = None) -> str:
        return "<promise>COMPLETE</promise>"

    monkeypatch.setattr(ralphython, "_run_and_capture", fake_run)
    monkeypatch.setattr("ralphython.time.sleep", lambda _seconds: None)

    rc = ralphython.main(["--agent", "amp", "--prd", str(prd_src), "1"])
    assert rc == 0
    prd_dest = tmp_path / "prd.json"
    assert prd_dest.read_text() == prd_src.read_text()
