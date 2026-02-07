import asyncio
import subprocess
import re
from pathlib import Path
from typing import Optional, Callable
from datetime import datetime
from anthropic import Anthropic
from ..core.config import settings
from ..services.websocket_manager import manager
from ..services.prd_service import prd_service
from ..services.conversation_manager import conversation_manager


class RalphOrchestrator:
    """Orchestrates the Ralph autonomous loop with real-time WebSocket updates"""

    def __init__(self):
        self.running = False
        self.current_iteration = 0
        self.max_iterations = settings.DEFAULT_MAX_ITERATIONS
        self.anthropic_client = None
        self.use_api = False

        # Initialize Anthropic client if API key is available
        if settings.ANTHROPIC_API_KEY:
            self.anthropic_client = Anthropic(api_key=settings.ANTHROPIC_API_KEY)
            self.use_api = True

    async def start_loop(self, max_iterations: int = None, use_cli: bool = False):
        """Start the Ralph autonomous loop"""
        if self.running:
            await manager.broadcast_error("Ralph is already running")
            return

        self.running = True
        self.current_iteration = 0
        self.max_iterations = max_iterations or settings.DEFAULT_MAX_ITERATIONS

        # Override API usage if CLI is explicitly requested
        if use_cli:
            self.use_api = False

        try:
            # Add to conversation history
            conversation_manager.add_orchestrator_message(
                "assistant",
                f"Starting Ralph autonomous loop (max {self.max_iterations} iterations)"
            )

            for iteration in range(1, self.max_iterations + 1):
                if not self.running:
                    await manager.broadcast_orchestrator_message(
                        "assistant",
                        "Ralph loop stopped by user"
                    )
                    break

                self.current_iteration = iteration
                conversation_manager.add_orchestrator_message(
                    "assistant",
                    f"Starting iteration {iteration} of {self.max_iterations}"
                )

                # Get next incomplete story
                next_story = await prd_service.get_next_incomplete_story()
                if not next_story:
                    conversation_manager.add_orchestrator_message(
                        "assistant",
                        "No incomplete stories found. All done!"
                    )
                    await manager.broadcast_complete()
                    break

                # Broadcast iteration start
                await manager.broadcast_iteration_start(
                    iteration,
                    next_story.id,
                    next_story.title
                )

                # Run iteration (using API or CLI)
                success = False
                if self.use_api and self.anthropic_client:
                    success = await self._run_iteration_with_api(iteration, next_story)
                else:
                    success = await self._run_iteration_with_cli(iteration, next_story)

                # Broadcast iteration complete
                await manager.broadcast_iteration_complete(iteration, next_story.id, success)

                if not success:
                    await manager.broadcast_error(
                        f"Iteration {iteration} failed for story {next_story.id}",
                        iteration
                    )

                # Check if all stories are complete
                if await prd_service.all_stories_complete():
                    conversation_manager.add_orchestrator_message(
                        "assistant",
                        "All stories completed! Ralph is done."
                    )
                    await manager.broadcast_complete()
                    break

                # Small delay between iterations
                await asyncio.sleep(2)

            if self.current_iteration >= self.max_iterations:
                conversation_manager.add_orchestrator_message(
                    "assistant",
                    f"Reached max iterations ({self.max_iterations}). Some stories may still be incomplete."
                )

        except Exception as e:
            await manager.broadcast_error(f"Ralph orchestrator error: {str(e)}")
        finally:
            self.running = False
            self.current_iteration = 0

    async def _run_iteration_with_api(self, iteration: int, story) -> bool:
        """Run a single iteration using Anthropic API"""
        try:
            conversation_manager.add_subagent_message(
                iteration,
                story.id,
                "system",
                f"Running iteration {iteration} with Anthropic API for story: {story.id}"
            )

            # Read the CLAUDE.md prompt
            claude_md_path = Path(settings.RALPH_SCRIPT_PATH).parent / "CLAUDE.md"
            if not claude_md_path.exists():
                await manager.broadcast_error(f"CLAUDE.md not found at {claude_md_path}", iteration)
                return False

            with open(claude_md_path, 'r') as f:
                prompt = f.read()

            conversation_manager.add_subagent_message(
                iteration,
                story.id,
                "user",
                prompt
            )

            # Call Claude API with streaming
            response_text = ""
            async with self.anthropic_client.messages.stream(
                model="claude-sonnet-4-5-20250929",
                max_tokens=8000,
                messages=[{"role": "user", "content": prompt}]
            ) as stream:
                async for text in stream.text_stream:
                    response_text += text

            # Add complete response to conversation history
            conversation_manager.add_subagent_message(
                iteration,
                story.id,
                "assistant",
                response_text
            )

            # Check for completion signal
            if "<promise>COMPLETE</promise>" in response_text:
                return True

            # Check if story was marked as complete in prd.json
            prd = await prd_service.get_prd()
            for s in prd.userStories:
                if s.id == story.id and s.passes:
                    await manager.broadcast_story_update(story.id, True)
                    return True

            return False

        except Exception as e:
            await manager.broadcast_error(f"API iteration error: {str(e)}", iteration)
            return False

    async def _run_iteration_with_cli(self, iteration: int, story) -> bool:
        """Run a single iteration using CLI (amp or claude)"""
        try:
            conversation_manager.add_subagent_message(
                iteration,
                story.id,
                "system",
                f"Running iteration {iteration} with CLI for story: {story.id}"
            )

            # Determine which CLI to use (default to claude)
            cli_command = ["claude", "--dangerously-skip-permissions", "--print"]
            claude_md_path = Path(settings.RALPH_SCRIPT_PATH).parent / "CLAUDE.md"

            if not claude_md_path.exists():
                await manager.broadcast_error(f"CLAUDE.md not found at {claude_md_path}", iteration)
                return False

            # Run CLI command with input from CLAUDE.md
            with open(claude_md_path, 'r') as f:
                prompt_input = f.read()

            process = await asyncio.create_subprocess_exec(
                *cli_command,
                stdin=asyncio.subprocess.PIPE,
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.PIPE
            )

            # Send prompt and get output
            stdout, stderr = await process.communicate(input=prompt_input.encode())

            output = stdout.decode()
            if stderr:
                error_output = stderr.decode()
                conversation_manager.add_subagent_message(
                    iteration,
                    story.id,
                    "system",
                    f"STDERR: {error_output}"
                )

            # Add output to conversation history
            conversation_manager.add_subagent_message(
                iteration,
                story.id,
                "assistant",
                output
            )

            # Check for completion signal
            if "<promise>COMPLETE</promise>" in output:
                return True

            # Check if story was marked as complete
            prd = await prd_service.get_prd()
            for s in prd.userStories:
                if s.id == story.id and s.passes:
                    await manager.broadcast_story_update(story.id, True)
                    return True

            return False

        except Exception as e:
            await manager.broadcast_error(f"CLI iteration error: {str(e)}", iteration)
            return False

    async def stop_loop(self):
        """Stop the running loop"""
        self.running = False
        await manager.broadcast_orchestrator_message(
            "assistant",
            "Stopping Ralph loop..."
        )

    def is_running(self) -> bool:
        """Check if the loop is currently running"""
        return self.running

    def get_current_iteration(self) -> int:
        """Get the current iteration number"""
        return self.current_iteration


# Global instance
ralph_orchestrator = RalphOrchestrator()
