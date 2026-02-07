from typing import Dict, List, Optional
from datetime import datetime
import uuid
from ..models.conversation import Message, Conversation
from .websocket_manager import manager


class ConversationManager:
    """Manages conversation history for orchestrator and subagents"""

    def __init__(self):
        # Main orchestrator conversation (you ↔ Ralph)
        self.orchestrator_conversation = Conversation(
            id="orchestrator-main",
            type="orchestrator",
            messages=[],
            created_at=datetime.now(),
            updated_at=datetime.now()
        )

        # Subagent conversations (one per iteration)
        self.subagent_conversations: Dict[int, Conversation] = {}

    def add_orchestrator_message(
        self,
        role: str,
        content: str,
        tool_calls: Optional[List[dict]] = None,
        tool_results: Optional[List[dict]] = None
    ) -> Message:
        """Add a message to the orchestrator conversation"""
        message = Message(
            id=str(uuid.uuid4()),
            role=role,
            content=content,
            timestamp=datetime.now(),
            tool_calls=tool_calls,
            tool_results=tool_results
        )

        self.orchestrator_conversation.messages.append(message)
        self.orchestrator_conversation.updated_at = datetime.now()

        # Broadcast via WebSocket
        manager.broadcast_orchestrator_message(role, content)

        return message

    def add_subagent_message(
        self,
        iteration: int,
        story_id: str,
        role: str,
        content: str,
        tool_calls: Optional[List[dict]] = None,
        tool_results: Optional[List[dict]] = None
    ) -> Message:
        """Add a message to a subagent conversation"""

        # Create conversation for this iteration if it doesn't exist
        if iteration not in self.subagent_conversations:
            self.subagent_conversations[iteration] = Conversation(
                id=f"subagent-iteration-{iteration}",
                type="subagent",
                iteration=iteration,
                story_id=story_id,
                messages=[],
                created_at=datetime.now(),
                updated_at=datetime.now()
            )

        message = Message(
            id=str(uuid.uuid4()),
            role=role,
            content=content,
            timestamp=datetime.now(),
            tool_calls=tool_calls,
            tool_results=tool_results
        )

        self.subagent_conversations[iteration].messages.append(message)
        self.subagent_conversations[iteration].updated_at = datetime.now()

        # Broadcast via WebSocket
        manager.broadcast_subagent_message(role, content, iteration)

        return message

    def get_orchestrator_conversation(self) -> Conversation:
        """Get the full orchestrator conversation"""
        return self.orchestrator_conversation

    def get_subagent_conversation(self, iteration: int) -> Optional[Conversation]:
        """Get a specific subagent conversation by iteration"""
        return self.subagent_conversations.get(iteration)

    def get_all_subagent_conversations(self) -> List[Conversation]:
        """Get all subagent conversations sorted by iteration"""
        return sorted(
            self.subagent_conversations.values(),
            key=lambda c: c.iteration or 0
        )

    def get_conversation_summary(self) -> dict:
        """Get summary of all conversations"""
        return {
            "orchestrator": self.orchestrator_conversation.model_dump(),
            "subagents": [
                conv.model_dump()
                for conv in self.get_all_subagent_conversations()
            ]
        }

    def clear_orchestrator_conversation(self):
        """Clear the orchestrator conversation"""
        self.orchestrator_conversation = Conversation(
            id="orchestrator-main",
            type="orchestrator",
            messages=[],
            created_at=datetime.now(),
            updated_at=datetime.now()
        )

    def clear_subagent_conversations(self):
        """Clear all subagent conversations"""
        self.subagent_conversations.clear()

    def clear_all_conversations(self):
        """Clear all conversations"""
        self.clear_orchestrator_conversation()
        self.clear_subagent_conversations()


# Global instance
conversation_manager = ConversationManager()
