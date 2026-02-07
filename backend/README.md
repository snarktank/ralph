# Ralph Web UI - Backend

Python FastAPI backend for Ralph autonomous AI agent loop with WebSocket support.

## Setup

1. Create a virtual environment:
```bash
python -m venv venv
source venv/bin/activate  # On Windows: venv\Scripts\activate
```

2. Install dependencies:
```bash
pip install -r requirements.txt
```

3. Configure environment:
```bash
cp .env.example .env
# Edit .env with your settings
```

4. Run the server:
```bash
python run.py
```

The API will be available at `http://localhost:8000`

## API Documentation

Interactive API docs available at:
- Swagger UI: `http://localhost:8000/docs`
- ReDoc: `http://localhost:8000/redoc`

## Endpoints

### PRD Management
- `GET /api/prd/` - Get current PRD
- `POST /api/prd/` - Create new PRD
- `PUT /api/prd/` - Update PRD
- `DELETE /api/prd/` - Delete PRD
- `POST /api/prd/stories` - Add user story
- `PUT /api/prd/stories/{story_id}` - Update story status
- `GET /api/prd/next-story` - Get next incomplete story
- `GET /api/prd/status` - Get PRD completion status

### Ralph Control
- `POST /api/ralph/start` - Start autonomous loop
- `POST /api/ralph/stop` - Stop running loop
- `GET /api/ralph/status` - Get orchestrator status

### WebSocket
- `WS /ws` - WebSocket connection for real-time updates

## WebSocket Messages

The WebSocket emits messages with this format:
```json
{
  "type": "message_type",
  "data": {},
  "timestamp": "2024-01-01T00:00:00",
  "iteration": 1
}
```

### Message Types
- `orchestrator_message` - User ↔ orchestrator chat
- `subagent_message` - Subagent output (current iteration)
- `tool_call` - Tool being called
- `tool_result` - Tool execution result
- `iteration_start` - New iteration starting
- `iteration_complete` - Iteration finished
- `story_update` - Story status changed
- `progress_update` - Progress log updated
- `git_commit` - Git commit made
- `error` - Error occurred
- `complete` - All stories complete

## Environment Variables

See `.env.example` for all available settings.

Key variables:
- `ANTHROPIC_API_KEY` - Optional, enables API mode
- `HOST` - Server host (default: 0.0.0.0)
- `PORT` - Server port (default: 8000)
- `PRD_FILE_PATH` - Path to prd.json
- `PROGRESS_FILE_PATH` - Path to progress.txt
