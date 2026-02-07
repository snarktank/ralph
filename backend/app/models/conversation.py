from pydantic import BaseModel
from typing import List, Optional, Literal
from datetime import datetime


class Message(BaseModel):
    """Single message in a conversation"""
    id: str
    role: Literal["user", "assistant", "system"]
    content: str
    timestamp: datetime
    tool_calls: Optional[List[dict]] = None
    tool_results: Optional[List[dict]] = None


class Conversation(BaseModel):
    """Full conversation history"""
    id: str
    type: Literal["orchestrator", "subagent"]
    iteration: Optional[int] = None  # For subagent conversations
    story_id: Optional[str] = None  # For subagent conversations
    messages: List[Message] = []
    created_at: datetime
    updated_at: datetime


class ConversationSummary(BaseModel):
    """Summary of all conversations"""
    orchestrator: Conversation
    subagents: List[Conversation]
