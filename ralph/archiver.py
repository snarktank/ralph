from __future__ import annotations

import shutil
from datetime import datetime
from pathlib import Path

from returns.result import Failure, Result, Success


def _project_root() -> Path:
    return Path(__file__).resolve().parent.parent


def _last_branch_path() -> Path:
    return _project_root() / ".last-branch"


def _default_prd_path() -> Path:
    return _project_root() / "prd.json"


def _default_progress_path() -> Path:
    return _project_root() / "progress.txt"


def _archive_dir() -> Path:
    return _project_root() / "archive"


def _get_last_branch() -> str | None:
    """Read the last branch from .last-branch file."""
    last_branch_file = _last_branch_path()
    if not last_branch_file.exists():
        return None
    try:
        return last_branch_file.read_text().strip()
    except Exception:
        return None


def _get_current_git_branch() -> str | None:
    """Get the current git branch name."""
    try:
        import subprocess

        result = subprocess.run(
            ["git", "branch", "--show-current"],
            capture_output=True,
            text=True,
            check=True,
            cwd=_project_root(),
        )
        return result.stdout.strip() or None
    except Exception:
        return None


def check_branch_change() -> bool:
    """
    Check if the git branch has changed since the last run.

    Returns True if the branch has changed, False otherwise.
    """
    last_branch = _get_last_branch()
    current_branch = _get_current_git_branch()

    if last_branch is None:
        # First run, no previous branch
        return False

    if current_branch is None:
        # Can't determine current branch
        return False

    return last_branch != current_branch


def archive_previous_run(
    last_branch: str,
    current_branch: str,
    prd_path: Path | None = None,
    progress_path: Path | None = None,
) -> Result[None, Exception]:
    """
    Archive the previous run's prd.json and progress.txt.

    Archives to: archive/{date}-{branch-name}/
    """
    prd_source = prd_path or _default_prd_path()
    progress_source = progress_path or _default_progress_path()

    try:
        # Create archive directory if it doesn't exist
        archive_root = _archive_dir()
        archive_root.mkdir(exist_ok=True)

        # Create timestamped directory: YYYY-MM-DD-{branch-name}
        date_str = datetime.now().strftime("%Y-%m-%d")
        # Sanitize branch name for filesystem (replace / with -)
        safe_branch = last_branch.replace("/", "-")
        archive_name = f"{date_str}-{safe_branch}"
        archive_path = archive_root / archive_name

        # If directory exists, add a counter
        counter = 1
        original_archive_path = archive_path
        while archive_path.exists():
            archive_path = Path(f"{original_archive_path}-{counter}")
            counter += 1

        archive_path.mkdir(parents=True)

        # Copy files if they exist
        if prd_source.exists():
            shutil.copy2(prd_source, archive_path / "prd.json")

        if progress_source.exists():
            shutil.copy2(progress_source, archive_path / "progress.txt")

        return Success(None)
    except Exception as exc:
        return Failure(exc)


def update_last_branch(branch_name: str) -> Result[None, Exception]:
    """Update the .last-branch file with the current branch name."""
    try:
        _last_branch_path().write_text(f"{branch_name}\n")
        return Success(None)
    except Exception as exc:
        return Failure(exc)


def reset_progress_file(progress_path: Path | None = None) -> Result[None, Exception]:
    """Reset progress.txt with a new header on branch change."""
    target = progress_path or _default_progress_path()
    try:
        target.write_text(
            "# Ralph Progress Log\n"
            f"Started: {datetime.now()}\n"
            "---\n"
        )
        return Success(None)
    except Exception as exc:
        return Failure(exc)
