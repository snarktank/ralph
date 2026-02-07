from pydantic import BaseModel
from typing import Literal, Optional, Any
from datetime import datetime


class WebSocketMessage(BaseModel):
    """WebSocket message format"""
    type: Literal[
        "orchestrator_message",
        "subagent_message",
        "tool_call",
        "tool_result",
        "iteration_start",
        "iteration_complete",
        "story_update",
        "progress_update",
        "git_commit",
        "error",
        "complete"
    ]
    data: Any
    timestamp: datetime = datetime.now()
    iteration: Optional[int] = None


class OrchestratorMessage(BaseModel):
    """Message from orchestrator (user chat)"""
    role: Literal["user", "assistant"]
    content: str


class SubagentMessage(BaseModel):
    """Message from subagent (current iteration)"""
    role: Literal["user", "assistant", "system"]
    content: str


class ToolCall(BaseModel):
    """Tool call information"""
    tool_name: str
    parameters: dict
    description: Optional[str] = None


class ToolResult(BaseModel):
    """Tool execution result"""
    tool_name: str
    output: str
    success: bool


class IterationStatus(BaseModel):
    """Iteration status update"""
    iteration: int
    story_id: str
    story_title: str
    status: Literal["started", "implementing", "testing", "committing", "complete", "failed"]


class StoryUpdate(BaseModel):
    """Story completion status update"""
    story_id: str
    passes: bool


class ProgressUpdate(BaseModel):
    """Progress log update"""
    entry: str


class GitCommit(BaseModel):
    """Git commit notification"""
    commit_hash: str
    message: str
    story_id: str
