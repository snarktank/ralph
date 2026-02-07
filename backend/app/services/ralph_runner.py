import asyncio
import json
import os
from pathlib import Path
from typing import Dict, Optional, Callable
from datetime import datetime
import subprocess
import re
from .websocket_manager import manager as ws_manager
from .ralph_event_logger import ralph_event_logger


class RalphRunner:
    """Manages Ralph loop execution for individual projects"""

    def __init__(self):
        self.running_processes: Dict[str, asyncio.Task] = {}
        self.project_conversations: Dict[str, dict] = {}
        self.current_story_ids: Dict[str, Optional[str]] = {}  # Track current story per project

    async def start_ralph_loop(
        self,
        project_id: str,
        project_path: str,
        on_message: Optional[Callable] = None,
        project_obj = None  # Optional project object to update
    ) -> bool:
        """
        Start Ralph autonomous loop for a project

        Args:
            project_id: Unique project identifier
            project_path: Path to project directory
            on_message: Callback for streaming messages (async function)

        Returns:
            True if started successfully
        """
        if project_id in self.running_processes:
            return False

        # Initialize event logger for this project
        events_file = ralph_event_logger.initialize_project(project_id, project_path)

        # Update project with events path if provided
        if project_obj:
            project_obj.ralph_events_path = str(events_file)
            project_obj.ralph_last_event_time = datetime.now()

        # Log system start event
        ralph_event_logger.log_system(
            project_id,
            "Ralph autonomous agent started",
            data={"project_path": project_path, "events_file": str(events_file)}
        )

        # Initialize conversation storage
        self.project_conversations[project_id] = {
            "orchestrator": [],
            "subagents": {},
            "status": "running",
            "started_at": datetime.now().isoformat()
        }

        # Initialize current story tracker
        self.current_story_ids[project_id] = None

        # Create async task to run Ralph
        task = asyncio.create_task(
            self._run_ralph_process(project_id, project_path, on_message)
        )
        self.running_processes[project_id] = task

        return True

    async def stop_ralph_loop(self, project_id: str) -> bool:
        """Stop Ralph loop for a project"""
        if project_id not in self.running_processes:
            return False

        task = self.running_processes[project_id]
        task.cancel()

        try:
            await task
        except asyncio.CancelledError:
            pass

        del self.running_processes[project_id]

        if project_id in self.project_conversations:
            self.project_conversations[project_id]["status"] = "stopped"

        # Log stop event
        ralph_event_logger.log_system(project_id, "Ralph autonomous agent stopped by user")

        # Clean up current story tracker
        if project_id in self.current_story_ids:
            del self.current_story_ids[project_id]

        return True

    def is_running(self, project_id: str) -> bool:
        """Check if Ralph is running for a project"""
        return project_id in self.running_processes

    def get_conversation(self, project_id: str) -> Optional[dict]:
        """Get conversation history for a project"""
        return self.project_conversations.get(project_id)

    async def _run_ralph_process(
        self,
        project_id: str,
        project_path: str,
        on_message: Optional[Callable]
    ):
        """
        Execute Ralph loop and stream updates

        This runs the Claude Code CLI in autonomous mode, monitoring the project
        """
        try:
            # Find claude-code binary
            claude_binary = self._find_claude_binary()
            if not claude_binary:
                await self._send_message(
                    project_id,
                    "error",
                    "Claude Code CLI not found. Please ensure it's installed.",
                    on_message
                )
                return

            # Prepare Ralph loop command
            # We'll run Claude in the project directory with instructions to read CLAUDE.md
            cmd = [
                str(claude_binary),
                "--dangerously-skip-update-check",
                "--prompt", "Please read CLAUDE.md and start working on the highest priority user story with passes: false in prd.json. Follow all instructions in CLAUDE.md exactly."
            ]

            await self._send_message(
                project_id,
                "system",
                f"Starting Ralph autonomous agent in {project_path}",
                on_message
            )

            # Start process
            process = await asyncio.create_subprocess_exec(
                *cmd,
                cwd=project_path,
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.PIPE,
                stdin=asyncio.subprocess.PIPE,
                env={**os.environ, "ANTHROPIC_API_KEY": os.getenv("ANTHROPIC_API_KEY", "")}
            )

            # Stream output
            await asyncio.gather(
                self._stream_output(process.stdout, project_id, "stdout", on_message),
                self._stream_output(process.stderr, project_id, "stderr", on_message),
            )

            # Wait for completion
            await process.wait()

            if process.returncode == 0:
                await self._send_message(
                    project_id,
                    "system",
                    "Ralph loop completed successfully",
                    on_message
                )
                self.project_conversations[project_id]["status"] = "completed"
                ralph_event_logger.log_system(
                    project_id,
                    "Ralph loop completed successfully",
                    data={"exit_code": 0}
                )
            else:
                await self._send_message(
                    project_id,
                    "error",
                    f"Ralph loop exited with code {process.returncode}",
                    on_message
                )
                self.project_conversations[project_id]["status"] = "error"
                ralph_event_logger.log_error(
                    project_id,
                    f"Ralph loop exited with code {process.returncode}",
                    error_details=f"Process exit code: {process.returncode}"
                )

        except asyncio.CancelledError:
            await self._send_message(
                project_id,
                "system",
                "Ralph loop stopped by user",
                on_message
            )
            self.project_conversations[project_id]["status"] = "stopped"
            raise
        except Exception as e:
            await self._send_message(
                project_id,
                "error",
                f"Error running Ralph: {str(e)}",
                on_message
            )
            self.project_conversations[project_id]["status"] = "error"

    async def _stream_output(self, stream, project_id: str, stream_type: str, on_message: Optional[Callable]):
        """Stream process output line by line"""
        try:
            while True:
                line = await stream.readline()
                if not line:
                    break

                text = line.decode('utf-8').strip()
                if text:
                    # Parse Claude output for structured messages
                    message = self._parse_claude_output(text)

                    # Store in conversation history
                    self.project_conversations[project_id]["orchestrator"].append({
                        "timestamp": datetime.now().isoformat(),
                        "type": stream_type,
                        "content": text,
                        "parsed": message
                    })

                    # Log events based on output
                    current_story = self.current_story_ids.get(project_id)
                    ralph_event_logger.parse_claude_output_for_events(
                        project_id,
                        text,
                        current_story_id=current_story
                    )

                    # Check for story ID updates
                    self._update_current_story(project_id, text)

                    # Prepare message data
                    msg_data = {
                        "type": "message",
                        "stream": stream_type,
                        "content": text,
                        "parsed": message,
                        "timestamp": datetime.now().isoformat()
                    }

                    # Send to callback
                    if on_message:
                        await on_message(project_id, msg_data)

                    # Broadcast via WebSocket
                    await ws_manager.broadcast_project_ralph_message(
                        project_id,
                        "message",
                        msg_data
                    )
        except Exception as e:
            print(f"Error streaming output: {e}")

    def _update_current_story(self, project_id: str, text: str):
        """Update the current story ID being worked on"""
        # Look for story ID patterns like US-001, Story #1, etc.
        story_pattern = r"(US-\d+|Story\s+#?(\d+))"
        match = re.search(story_pattern, text, re.IGNORECASE)
        if match:
            story_id = match.group(1)
            if self.current_story_ids.get(project_id) != story_id:
                self.current_story_ids[project_id] = story_id

    def _parse_claude_output(self, text: str) -> Optional[dict]:
        """
        Parse Claude Code CLI output to extract structured information

        Returns dict with parsed message or None if not parseable
        """
        # Look for tool uses, file operations, etc.
        patterns = {
            "file_read": r"Reading file: (.+)",
            "file_write": r"Writing file: (.+)",
            "file_edit": r"Editing file: (.+)",
            "bash_command": r"Running command: (.+)",
            "task_complete": r"✓ (.+)",
            "task_start": r"→ (.+)",
        }

        for pattern_type, pattern in patterns.items():
            match = re.search(pattern, text)
            if match:
                return {
                    "type": pattern_type,
                    "detail": match.group(1)
                }

        return None

    def _find_claude_binary(self) -> Optional[Path]:
        """Find Claude Code CLI binary"""
        # Check common locations
        common_paths = [
            Path.home() / ".local" / "bin" / "claude",
            Path("/usr/local/bin/claude"),
            Path("/usr/bin/claude"),
        ]

        for path in common_paths:
            if path.exists() and path.is_file():
                return path

        # Try which command
        try:
            result = subprocess.run(
                ["which", "claude"],
                capture_output=True,
                text=True
            )
            if result.returncode == 0:
                return Path(result.stdout.strip())
        except:
            pass

        return None

    async def _send_message(
        self,
        project_id: str,
        msg_type: str,
        content: str,
        on_message: Optional[Callable]
    ):
        """Send a message through callback and WebSocket"""
        message_data = {
            "type": msg_type,
            "content": content,
            "timestamp": datetime.now().isoformat()
        }

        # Send to callback if provided
        if on_message:
            await on_message(project_id, message_data)

        # Also broadcast via WebSocket
        await ws_manager.broadcast_project_ralph_message(
            project_id,
            msg_type,
            message_data
        )


# Global instance
ralph_runner = RalphRunner()
