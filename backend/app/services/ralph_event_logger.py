import json
import os
from pathlib import Path
from typing import Dict, List, Optional, Any
from datetime import datetime
from collections import deque
import re


class RalphEventLogger:
    """
    Logs Ralph execution events to JSON file and maintains in-memory cache

    Events are written in JSON Lines format (one JSON object per line)
    for efficient append-only logging and easy parsing.
    """

    # Event types
    EVENT_STORY_START = "story_start"
    EVENT_STORY_COMPLETE = "story_complete"
    EVENT_TOOL_CALL = "tool_call"
    EVENT_COMMIT = "commit"
    EVENT_ERROR = "error"
    EVENT_QUALITY_CHECK = "quality_check"
    EVENT_PROGRESS_UPDATE = "progress_update"
    EVENT_SYSTEM = "system"

    def __init__(self, max_memory_events: int = 500):
        """
        Initialize event logger

        Args:
            max_memory_events: Maximum number of events to keep in memory cache
        """
        self.project_events: Dict[str, deque] = {}  # project_id -> deque of events
        self.project_files: Dict[str, Path] = {}  # project_id -> event file path
        self.max_memory_events = max_memory_events

    def initialize_project(self, project_id: str, project_path: str) -> Path:
        """
        Initialize event logging for a project

        Args:
            project_id: Unique project identifier
            project_path: Path to project directory

        Returns:
            Path to events file
        """
        # Create events file path
        events_file = Path(project_path) / "ralph_events.jsonl"

        # Initialize in-memory cache
        self.project_events[project_id] = deque(maxlen=self.max_memory_events)
        self.project_files[project_id] = events_file

        # Create file if it doesn't exist
        if not events_file.exists():
            events_file.touch()

        return events_file

    def log_event(
        self,
        project_id: str,
        event_type: str,
        message: str,
        story_id: Optional[str] = None,
        data: Optional[Dict[str, Any]] = None
    ) -> Dict[str, Any]:
        """
        Log an event for a project

        Args:
            project_id: Project identifier
            event_type: Type of event (use EVENT_* constants)
            message: Human-readable message
            story_id: Optional user story ID
            data: Optional additional data

        Returns:
            The logged event dict
        """
        if project_id not in self.project_files:
            raise ValueError(f"Project {project_id} not initialized. Call initialize_project first.")

        event = {
            "timestamp": datetime.now().isoformat(),
            "event_type": event_type,
            "message": message,
            "story_id": story_id,
            "data": data or {}
        }

        # Add to in-memory cache
        self.project_events[project_id].append(event)

        # Append to file
        self._write_event_to_file(project_id, event)

        return event

    def log_tool_call(
        self,
        project_id: str,
        tool_name: str,
        details: str,
        story_id: Optional[str] = None,
        files_affected: Optional[List[str]] = None
    ):
        """Log a tool call event"""
        data = {
            "tool": tool_name,
            "details": details,
            "files_affected": files_affected or []
        }
        return self.log_event(
            project_id,
            self.EVENT_TOOL_CALL,
            f"Tool: {tool_name} - {details}",
            story_id=story_id,
            data=data
        )

    def log_commit(
        self,
        project_id: str,
        commit_message: str,
        story_id: Optional[str] = None,
        files_changed: Optional[List[str]] = None,
        commit_sha: Optional[str] = None
    ):
        """Log a git commit event"""
        data = {
            "commit_message": commit_message,
            "files_changed": files_changed or [],
            "commit_sha": commit_sha
        }
        return self.log_event(
            project_id,
            self.EVENT_COMMIT,
            f"Commit: {commit_message}",
            story_id=story_id,
            data=data
        )

    def log_story_start(self, project_id: str, story_id: str, story_title: str):
        """Log start of user story implementation"""
        data = {
            "story_title": story_title
        }
        return self.log_event(
            project_id,
            self.EVENT_STORY_START,
            f"Started: {story_id} - {story_title}",
            story_id=story_id,
            data=data
        )

    def log_story_complete(
        self,
        project_id: str,
        story_id: str,
        story_title: str,
        passed: bool,
        files_changed: Optional[List[str]] = None
    ):
        """Log completion of user story"""
        data = {
            "story_title": story_title,
            "passed": passed,
            "files_changed": files_changed or []
        }
        status = "✓ Passed" if passed else "✗ Failed"
        return self.log_event(
            project_id,
            self.EVENT_STORY_COMPLETE,
            f"Completed: {story_id} - {story_title} [{status}]",
            story_id=story_id,
            data=data
        )

    def log_error(
        self,
        project_id: str,
        error_message: str,
        error_details: Optional[str] = None,
        story_id: Optional[str] = None
    ):
        """Log an error event"""
        data = {
            "error_message": error_message,
            "error_details": error_details
        }
        return self.log_event(
            project_id,
            self.EVENT_ERROR,
            f"Error: {error_message}",
            story_id=story_id,
            data=data
        )

    def log_quality_check(
        self,
        project_id: str,
        check_type: str,
        passed: bool,
        output: Optional[str] = None,
        story_id: Optional[str] = None
    ):
        """Log quality check result (typecheck, lint, test, etc.)"""
        data = {
            "check_type": check_type,
            "passed": passed,
            "output": output
        }
        status = "✓ Passed" if passed else "✗ Failed"
        return self.log_event(
            project_id,
            self.EVENT_QUALITY_CHECK,
            f"Quality Check ({check_type}): {status}",
            story_id=story_id,
            data=data
        )

    def log_progress_update(
        self,
        project_id: str,
        update_message: str,
        story_id: Optional[str] = None,
        metadata: Optional[Dict[str, Any]] = None
    ):
        """Log a progress update"""
        return self.log_event(
            project_id,
            self.EVENT_PROGRESS_UPDATE,
            update_message,
            story_id=story_id,
            data=metadata or {}
        )

    def log_system(self, project_id: str, message: str, data: Optional[Dict[str, Any]] = None):
        """Log a system message"""
        return self.log_event(
            project_id,
            self.EVENT_SYSTEM,
            message,
            data=data
        )

    def get_events(
        self,
        project_id: str,
        limit: Optional[int] = None,
        event_type: Optional[str] = None,
        story_id: Optional[str] = None
    ) -> List[Dict[str, Any]]:
        """
        Get events from memory cache

        Args:
            project_id: Project identifier
            limit: Maximum number of events to return (most recent first)
            event_type: Filter by event type
            story_id: Filter by story ID

        Returns:
            List of event dictionaries
        """
        if project_id not in self.project_events:
            return []

        events = list(self.project_events[project_id])

        # Apply filters
        if event_type:
            events = [e for e in events if e["event_type"] == event_type]

        if story_id:
            events = [e for e in events if e.get("story_id") == story_id]

        # Sort by timestamp (most recent first)
        events.sort(key=lambda e: e["timestamp"], reverse=True)

        # Apply limit
        if limit:
            events = events[:limit]

        return events

    def get_all_events_from_file(self, project_id: str) -> List[Dict[str, Any]]:
        """
        Read all events from file (useful for pagination or full history)

        Args:
            project_id: Project identifier

        Returns:
            List of all events from file
        """
        if project_id not in self.project_files:
            return []

        events_file = self.project_files[project_id]
        if not events_file.exists():
            return []

        events = []
        try:
            with open(events_file, 'r') as f:
                for line in f:
                    line = line.strip()
                    if line:
                        events.append(json.loads(line))
        except Exception as e:
            print(f"Error reading events file: {e}")

        return events

    def get_summary(self, project_id: str) -> Dict[str, Any]:
        """
        Get summary statistics for project events

        Returns:
            Dict with event counts, story progress, etc.
        """
        events = self.get_all_events_from_file(project_id)

        summary = {
            "total_events": len(events),
            "event_types": {},
            "stories": {},
            "last_event_time": None,
            "total_commits": 0,
            "total_errors": 0,
            "stories_completed": 0,
            "stories_in_progress": 0
        }

        story_status: Dict[str, str] = {}  # story_id -> status

        for event in events:
            # Count by type
            event_type = event["event_type"]
            summary["event_types"][event_type] = summary["event_types"].get(event_type, 0) + 1

            # Track stories
            story_id = event.get("story_id")
            if story_id:
                if story_id not in summary["stories"]:
                    summary["stories"][story_id] = {
                        "story_id": story_id,
                        "status": "in_progress",
                        "events": 0,
                        "commits": 0,
                        "errors": 0
                    }

                summary["stories"][story_id]["events"] += 1

                if event_type == self.EVENT_STORY_START:
                    story_status[story_id] = "in_progress"
                    summary["stories"][story_id]["status"] = "in_progress"
                    summary["stories"][story_id]["title"] = event["data"].get("story_title", "")

                elif event_type == self.EVENT_STORY_COMPLETE:
                    passed = event["data"].get("passed", False)
                    story_status[story_id] = "completed" if passed else "failed"
                    summary["stories"][story_id]["status"] = story_status[story_id]

                elif event_type == self.EVENT_COMMIT:
                    summary["stories"][story_id]["commits"] += 1

                elif event_type == self.EVENT_ERROR:
                    summary["stories"][story_id]["errors"] += 1

            # Count commits and errors
            if event_type == self.EVENT_COMMIT:
                summary["total_commits"] += 1
            elif event_type == self.EVENT_ERROR:
                summary["total_errors"] += 1

        # Count story statuses
        for status in story_status.values():
            if status == "completed":
                summary["stories_completed"] += 1
            elif status == "in_progress":
                summary["stories_in_progress"] += 1

        # Get last event time
        if events:
            summary["last_event_time"] = events[-1]["timestamp"]

        return summary

    def parse_claude_output_for_events(
        self,
        project_id: str,
        output_text: str,
        current_story_id: Optional[str] = None
    ) -> Optional[Dict[str, Any]]:
        """
        Parse Claude Code CLI output and automatically log relevant events

        Args:
            project_id: Project identifier
            output_text: Raw output text from Claude
            current_story_id: Current user story being worked on

        Returns:
            Logged event if one was created, None otherwise
        """
        # Detect file operations
        file_patterns = {
            "read": r"Reading\s+(?:file\s+)?['\"]?([^'\":\n]+)['\"]?",
            "write": r"(?:Writing|Creating)\s+(?:file\s+)?['\"]?([^'\":\n]+)['\"]?",
            "edit": r"Editing\s+(?:file\s+)?['\"]?([^'\":\n]+)['\"]?"
        }

        for tool, pattern in file_patterns.items():
            match = re.search(pattern, output_text, re.IGNORECASE)
            if match:
                file_path = match.group(1)
                return self.log_tool_call(
                    project_id,
                    tool,
                    file_path,
                    story_id=current_story_id,
                    files_affected=[file_path]
                )

        # Detect bash commands
        bash_pattern = r"(?:Running|Executing)\s+(?:command|bash)?:?\s*['\"]?([^'\":\n]+)['\"]?"
        match = re.search(bash_pattern, output_text, re.IGNORECASE)
        if match:
            command = match.group(1).strip()
            return self.log_tool_call(
                project_id,
                "bash",
                command,
                story_id=current_story_id
            )

        # Detect commits
        commit_pattern = r"git\s+commit.*?-m\s+['\"]([^'\"]+)['\"]"
        match = re.search(commit_pattern, output_text, re.IGNORECASE)
        if match:
            commit_msg = match.group(1)
            return self.log_commit(
                project_id,
                commit_msg,
                story_id=current_story_id
            )

        # Detect story references
        story_pattern = r"(US-\d+|Story\s+#?\d+)"
        match = re.search(story_pattern, output_text, re.IGNORECASE)
        if match and "start" in output_text.lower():
            story_ref = match.group(1)
            return self.log_story_start(project_id, story_ref, output_text)

        # Detect errors
        if "error" in output_text.lower() or "failed" in output_text.lower():
            return self.log_error(
                project_id,
                output_text[:200],  # Truncate long error messages
                error_details=output_text,
                story_id=current_story_id
            )

        return None

    def _write_event_to_file(self, project_id: str, event: Dict[str, Any]):
        """Write event to JSON Lines file"""
        try:
            events_file = self.project_files[project_id]
            with open(events_file, 'a') as f:
                f.write(json.dumps(event) + '\n')
        except Exception as e:
            print(f"Error writing event to file: {e}")

    def cleanup_project(self, project_id: str):
        """Clean up resources for a project"""
        if project_id in self.project_events:
            del self.project_events[project_id]
        if project_id in self.project_files:
            del self.project_files[project_id]


# Global instance
ralph_event_logger = RalphEventLogger()
