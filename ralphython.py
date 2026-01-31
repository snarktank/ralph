#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = [
#   "fastmcp>=3.0.0b1",
# ]
# ///

import argparse
import datetime as _dt
import json
import os
import shlex
import subprocess
import sys
import time
from pathlib import Path
from typing import Any

from fastmcp import FastMCP

# Initialize FastMCP server
mcp = FastMCP("Ralph Wiggum 🎯")


def _warn_tool_deprecated() -> None:
    print("⚠️  Warning: --tool is deprecated, use --agent instead", file=sys.stderr)


def _parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        prog="ralph.sh",
        description="Ralph Wiggum - Long-running AI agent loop",
    )
    parser.add_argument("--agent", choices=("amp", "claude", "codex"))
    parser.add_argument("--prd", help="Path to prd.json to ingest before running")
    parser.add_argument("--tool", choices=("amp", "claude", "codex"), help=argparse.SUPPRESS)
    parser.add_argument("max_iterations", nargs="?", type=int, default=10)

    args = parser.parse_args(argv)

    if args.tool:
        _warn_tool_deprecated()
        if not args.agent:
            args.agent = args.tool

    if not args.agent:
        parser.error("--agent is required. Use --agent amp|claude|codex.")

    return args


def _read_branch_name(prd_file: Path) -> str:
    try:
        data = json.loads(prd_file.read_text())
    except Exception:
        return ""
    branch = data.get("branchName")
    return branch or ""


def _ensure_progress_file(progress_file: Path) -> None:
    if progress_file.exists():
        return
    progress_file.write_text(
        "# Ralph Progress Log\n"
        f"Started: {_dt.datetime.now()}\n"
        "---\n"
    )


def _archive_previous_run(
    prd_file: Path, progress_file: Path, archive_dir: Path, last_branch_file: Path
) -> None:
    if not prd_file.exists() or not last_branch_file.exists():
        return

    current_branch = _read_branch_name(prd_file)
    try:
        last_branch = last_branch_file.read_text().strip()
    except Exception:
        last_branch = ""

    if not current_branch or not last_branch or current_branch == last_branch:
        return

    date_str = _dt.date.today().isoformat()
    folder_name = last_branch.removeprefix("ralph/")
    archive_folder = archive_dir / f"{date_str}-{folder_name}"

    print(f"Archiving previous run: {last_branch}")
    archive_folder.mkdir(parents=True, exist_ok=True)
    if prd_file.exists():
        (archive_folder / prd_file.name).write_text(prd_file.read_text())
    if progress_file.exists():
        (archive_folder / progress_file.name).write_text(progress_file.read_text())
    print(f"   Archived to: {archive_folder}")

    progress_file.write_text(
        "# Ralph Progress Log\n"
        f"Started: {_dt.datetime.now()}\n"
        "---\n"
    )


def _track_current_branch(prd_file: Path, last_branch_file: Path) -> None:
    if not prd_file.exists():
        return
    current_branch = _read_branch_name(prd_file)
    if current_branch:
        last_branch_file.write_text(current_branch)


def _run_and_capture(cmd: list[str], stdin_path: Path | None = None) -> str:
    stdin = None
    try:
        if stdin_path is not None:
            stdin = stdin_path.open("r")
        proc = subprocess.Popen(
            cmd,
            stdin=stdin,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
            bufsize=1,
        )
        assert proc.stdout is not None
        output_chunks: list[str] = []
        for line in proc.stdout:
            sys.stderr.write(line)
            output_chunks.append(line)
        proc.wait()
        return "".join(output_chunks)
    finally:
        if stdin is not None:
            stdin.close()


def main(argv: list[str]) -> int:
    args = _parse_args(argv)

    script_dir = Path(__file__).resolve().parent
    prd_file = script_dir / "prd.json"
    progress_file = script_dir / "progress.txt"
    archive_dir = script_dir / "archive"
    last_branch_file = script_dir / ".last-branch"

    _codex_prompt_file = Path(
        os.environ.get("CODEX_PROMPT_FILE", str(script_dir / "prompt.md"))
    )
    codex_model = os.environ.get("CODEX_MODEL", "gpt-5.2-codex")
    codex_reasoning_effort = os.environ.get("CODEX_REASONING_EFFORT", "high")
    codex_sandbox = os.environ.get("CODEX_SANDBOX", "workspace-write")
    codex_extra_args = os.environ.get("CODEX_EXTRA_ARGS", "")

    if args.prd:
        prd_src = Path(args.prd).expanduser()
        if not prd_src.exists():
            print(f"Error: PRD file not found: {prd_src}", file=sys.stderr)
            return 1
        prd_file.write_text(prd_src.read_text())

    _archive_previous_run(prd_file, progress_file, archive_dir, last_branch_file)
    _track_current_branch(prd_file, last_branch_file)
    _ensure_progress_file(progress_file)

    print(f"Starting Ralph - Agent: {args.agent} - Max iterations: {args.max_iterations}")

    for i in range(1, args.max_iterations + 1):
        print("")
        print("===============================================================")
        print(f"  Ralph Iteration {i} of {args.max_iterations} ({args.agent})")
        print("===============================================================")

        if args.agent == "amp":
            output = _run_and_capture(
                ["amp", "--dangerously-allow-all"],
                stdin_path=script_dir / "prompt.md",
            )
        elif args.agent == "codex":
            codex_args = [
                "codex",
                "exec",
                "-m",
                codex_model,
                "--config",
                f"model_reasoning_effort=\"{codex_reasoning_effort}\"",
                "--sandbox",
                codex_sandbox,
                "--dangerously-bypass-approvals-and-sandbox",
                "--cd",
                str(script_dir),
            ]
            if codex_extra_args:
                codex_args.extend(shlex.split(codex_extra_args))
            codex_args.append("@ralph-next")
            output = _run_and_capture(codex_args)
        else:
            output = _run_and_capture(
                [
                    "claude",
                    "--model",
                    "sonnet",
                    "--dangerously-skip-permissions",
                    "--print",
                ],
                stdin_path=script_dir / "CLAUDE.md",
            )
            sys.stdout.write(output)

        if "<promise>COMPLETE</promise>" in output:
            print("")
            print("Ralph completed all tasks!")
            print(f"Completed at iteration {i} of {args.max_iterations}")
            return 0

        print(f"Iteration {i} complete. Continuing...")
        time.sleep(2)

    print("")
    print(
        f"Ralph reached max iterations ({args.max_iterations}) without completing all tasks."
    )
    print(f"Check {progress_file} for status.")
    return 1


@mcp.tool()
def run_ralph_iteration(
    agent: str = "codex",
    max_iterations: int = 1,
    prd_path: str | None = None
) -> dict[str, str | int]:
    """
    Run Ralph autonomous agent for specified iterations.
    
    Args:
        agent: Agent to use (amp, claude, or codex)
        max_iterations: Maximum number of iterations to run
        prd_path: Optional path to PRD JSON file
        
    Returns:
        Status dict with exit_code, output, and iterations_completed
    """
    args = ["--agent", agent, str(max_iterations)]
    if prd_path:
        args.extend(["--prd", prd_path])
    
    exit_code = main(args)
    script_dir = Path(__file__).parent
    progress_file = script_dir / "progress.txt"
    
    return {
        "exit_code": exit_code,
        "status": "complete" if exit_code == 0 else "incomplete",
        "max_iterations": max_iterations,
        "progress_file": str(progress_file),
    }


@mcp.tool()
def get_ralph_status() -> dict[str, Any]:
    """
    Get current Ralph execution status from progress.txt.
    
    Returns:
        Dict with latest progress information
    """
    script_dir = Path(__file__).parent
    progress_file = script_dir / "progress.txt"
    
    if not progress_file.exists():
        return {"status": "no_progress_file", "message": "No progress.txt found"}
    
    content = progress_file.read_text()
    lines = content.strip().split("\n")
    
    return {
        "status": "active",
        "progress_file": str(progress_file),
        "last_lines": lines[-10:] if len(lines) > 10 else lines,
        "total_lines": len(lines),
    }


@mcp.tool()
def get_prd_status(prd_path: str | None = None) -> dict[str, Any]:
    """
    Get PRD completion status.
    
    Args:
        prd_path: Optional path to PRD file (defaults to ./prd.json)
        
    Returns:
        Dict with PRD metadata and story completion status
    """
    script_dir = Path(__file__).parent
    prd_file = Path(prd_path) if prd_path else (script_dir / "prd.json")
    
    if not prd_file.exists():
        return {"status": "not_found", "path": str(prd_file)}
    
    with open(prd_file) as f:
        prd_data = json.load(f)
    
    total = len(prd_data["userStories"])
    completed = sum(1 for story in prd_data["userStories"] if story.get("passes", False))
    incomplete = [
        {"id": s["id"], "title": s["title"]}
        for s in prd_data["userStories"]
        if not s.get("passes", False)
    ]
    
    return {
        "status": "loaded",
        "project": prd_data.get("project", "Unknown"),
        "total_stories": total,
        "completed_stories": completed,
        "completion_percentage": round((completed / total) * 100, 1) if total > 0 else 0,
        "incomplete_stories": incomplete[:5],  # First 5 incomplete
    }


@mcp.resource("ralph://prd")
def get_prd_resource() -> str:
    """Get the current PRD as a resource."""
    script_dir = Path(__file__).parent
    prd_file = script_dir / "prd.json"
    
    if not prd_file.exists():
        return "No PRD file found"
    
    return prd_file.read_text()


@mcp.resource("ralph://progress")
def get_progress_resource() -> str:
    """Get the current progress log as a resource."""
    script_dir = Path(__file__).parent
    progress_file = script_dir / "progress.txt"
    
    if not progress_file.exists():
        return "No progress file found"
    
    return progress_file.read_text()


if __name__ == "__main__":
    # Check if running as MCP server or CLI
    if "--mcp" in sys.argv:
        sys.argv.remove("--mcp")
        mcp.run()
    else:
        raise SystemExit(main(sys.argv[1:]))
