from fastapi import WebSocket
from typing import List, Dict
import json
from datetime import datetime
from ..models.websocket import WebSocketMessage


class ConnectionManager:
    """Manages WebSocket connections and broadcasts messages"""

    def __init__(self):
        self.active_connections: List[WebSocket] = []
        self.connection_metadata: Dict[WebSocket, dict] = {}
        # Project-specific connections: project_id -> list of websockets
        self.project_connections: Dict[str, List[WebSocket]] = {}

    async def connect(self, websocket: WebSocket, client_id: str = None):
        """Accept and store a new WebSocket connection"""
        await websocket.accept()
        self.active_connections.append(websocket)
        self.connection_metadata[websocket] = {
            "client_id": client_id,
            "connected_at": datetime.now()
        }

    def disconnect(self, websocket: WebSocket):
        """Remove a WebSocket connection"""
        if websocket in self.active_connections:
            self.active_connections.remove(websocket)
        if websocket in self.connection_metadata:
            del self.connection_metadata[websocket]

    async def send_personal_message(self, message: WebSocketMessage, websocket: WebSocket):
        """Send a message to a specific client"""
        await websocket.send_text(message.model_dump_json())

    async def broadcast(self, message: WebSocketMessage):
        """Broadcast a message to all connected clients"""
        disconnected = []
        for connection in self.active_connections:
            try:
                await connection.send_text(message.model_dump_json())
            except Exception as e:
                print(f"Error broadcasting to client: {e}")
                disconnected.append(connection)

        # Clean up disconnected clients
        for conn in disconnected:
            self.disconnect(conn)

    async def broadcast_orchestrator_message(self, role: str, content: str):
        """Broadcast an orchestrator message"""
        message = WebSocketMessage(
            type="orchestrator_message",
            data={"role": role, "content": content}
        )
        await self.broadcast(message)

    async def broadcast_subagent_message(self, role: str, content: str, iteration: int = None):
        """Broadcast a subagent message"""
        message = WebSocketMessage(
            type="subagent_message",
            data={"role": role, "content": content},
            iteration=iteration
        )
        await self.broadcast(message)

    async def broadcast_tool_call(self, tool_name: str, parameters: dict, iteration: int = None):
        """Broadcast a tool call"""
        message = WebSocketMessage(
            type="tool_call",
            data={
                "tool_name": tool_name,
                "parameters": parameters
            },
            iteration=iteration
        )
        await self.broadcast(message)

    async def broadcast_tool_result(self, tool_name: str, output: str, success: bool, iteration: int = None):
        """Broadcast a tool result"""
        message = WebSocketMessage(
            type="tool_result",
            data={
                "tool_name": tool_name,
                "output": output,
                "success": success
            },
            iteration=iteration
        )
        await self.broadcast(message)

    async def broadcast_iteration_start(self, iteration: int, story_id: str, story_title: str):
        """Broadcast iteration start"""
        message = WebSocketMessage(
            type="iteration_start",
            data={
                "iteration": iteration,
                "story_id": story_id,
                "story_title": story_title,
                "status": "started"
            },
            iteration=iteration
        )
        await self.broadcast(message)

    async def broadcast_iteration_complete(self, iteration: int, story_id: str, success: bool):
        """Broadcast iteration completion"""
        message = WebSocketMessage(
            type="iteration_complete",
            data={
                "iteration": iteration,
                "story_id": story_id,
                "status": "complete" if success else "failed"
            },
            iteration=iteration
        )
        await self.broadcast(message)

    async def broadcast_story_update(self, story_id: str, passes: bool):
        """Broadcast story status update"""
        message = WebSocketMessage(
            type="story_update",
            data={
                "story_id": story_id,
                "passes": passes
            }
        )
        await self.broadcast(message)

    async def broadcast_progress_update(self, entry: str):
        """Broadcast progress log update"""
        message = WebSocketMessage(
            type="progress_update",
            data={"entry": entry}
        )
        await self.broadcast(message)

    async def broadcast_git_commit(self, commit_hash: str, message: str, story_id: str):
        """Broadcast git commit notification"""
        message_obj = WebSocketMessage(
            type="git_commit",
            data={
                "commit_hash": commit_hash,
                "message": message,
                "story_id": story_id
            }
        )
        await self.broadcast(message_obj)

    async def broadcast_error(self, error_message: str, iteration: int = None):
        """Broadcast an error"""
        message = WebSocketMessage(
            type="error",
            data={"message": error_message},
            iteration=iteration
        )
        await self.broadcast(message)

    async def broadcast_complete(self):
        """Broadcast completion of all stories"""
        message = WebSocketMessage(
            type="complete",
            data={"message": "All stories completed!"}
        )
        await self.broadcast(message)

    # Project-specific connection management
    async def connect_to_project(self, websocket: WebSocket, project_id: str, client_id: str = None):
        """Connect a client to a specific project's Ralph dashboard"""
        await websocket.accept()

        if project_id not in self.project_connections:
            self.project_connections[project_id] = []

        self.project_connections[project_id].append(websocket)
        self.connection_metadata[websocket] = {
            "client_id": client_id,
            "project_id": project_id,
            "connected_at": datetime.now()
        }

    def disconnect_from_project(self, websocket: WebSocket, project_id: str):
        """Disconnect a client from a project"""
        if project_id in self.project_connections:
            if websocket in self.project_connections[project_id]:
                self.project_connections[project_id].remove(websocket)

            # Clean up empty project lists
            if not self.project_connections[project_id]:
                del self.project_connections[project_id]

        if websocket in self.connection_metadata:
            del self.connection_metadata[websocket]

    async def broadcast_to_project(self, project_id: str, message: WebSocketMessage):
        """Broadcast a message to all clients watching a specific project"""
        if project_id not in self.project_connections:
            return

        disconnected = []
        for connection in self.project_connections[project_id]:
            try:
                await connection.send_text(message.model_dump_json())
            except Exception as e:
                print(f"Error broadcasting to project client: {e}")
                disconnected.append(connection)

        # Clean up disconnected clients
        for conn in disconnected:
            self.disconnect_from_project(conn, project_id)

    async def broadcast_project_ralph_message(self, project_id: str, msg_type: str, data: dict):
        """Broadcast a Ralph message for a specific project"""
        message = WebSocketMessage(
            type=f"project_ralph_{msg_type}",
            data={
                "project_id": project_id,
                **data
            }
        )
        await self.broadcast_to_project(project_id, message)


# Global instance
manager = ConnectionManager()
