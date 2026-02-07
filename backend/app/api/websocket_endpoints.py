from fastapi import APIRouter, WebSocket, WebSocketDisconnect
from ..services.websocket_manager import manager
import json

router = APIRouter()


@router.websocket("/ws")
async def websocket_endpoint(websocket: WebSocket):
    """WebSocket endpoint for real-time updates"""
    await manager.connect(websocket)
    try:
        while True:
            # Receive messages from client (if any)
            data = await websocket.receive_text()
            # Echo back for now (can add client->server commands later)
            message_data = json.loads(data)
            print(f"Received from client: {message_data}")

    except WebSocketDisconnect:
        manager.disconnect(websocket)
        print("Client disconnected")
    except Exception as e:
        print(f"WebSocket error: {e}")
        manager.disconnect(websocket)


@router.websocket("/ws/project/{project_id}")
async def project_websocket_endpoint(websocket: WebSocket, project_id: str):
    """WebSocket endpoint for project-specific Ralph updates"""
    await manager.connect_to_project(websocket, project_id)
    try:
        while True:
            # Keep connection alive and receive messages
            data = await websocket.receive_text()
            message_data = json.loads(data)
            print(f"Received from project {project_id} client: {message_data}")

            # Handle client commands if needed (e.g., start/stop Ralph)
            if message_data.get("command") == "ping":
                await websocket.send_text(json.dumps({"type": "pong"}))

    except WebSocketDisconnect:
        manager.disconnect_from_project(websocket, project_id)
        print(f"Client disconnected from project {project_id}")
    except Exception as e:
        print(f"WebSocket error for project {project_id}: {e}")
        manager.disconnect_from_project(websocket, project_id)
