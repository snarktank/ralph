# Ralph Web UI

A beautiful web interface for Ralph autonomous AI agent loop. Create PRDs, watch real-time execution, and track progress as Ralph autonomously implements your features.

## Features

- **PRD Editor**: Create and manage Product Requirements Documents directly in the browser
- **Live Dashboard**: Real-time view of orchestrator ↔ subagent conversations
- **Progress Tracking**: Visual progress indicators and story completion status
- **WebSocket Streaming**: Live updates as Ralph executes each iteration
- **Dual Mode**: Switch between interactive dashboard and workflow flowchart visualization
- **API + CLI Support**: Use Anthropic API or CLI (amp/claude) for execution

## Architecture

```
┌─────────────────────────────────────────────────┐
│                Ralph Web UI                     │
├─────────────────────────────────────────────────┤
│                                                 │
│  Frontend (React + TypeScript + Vite)          │
│  - Dashboard view with real-time updates       │
│  - PRD editor with JSON preview                │
│  - Progress tracking and story management      │
│  - WebSocket client for live streaming         │
│                                                 │
├─────────────────────────────────────────────────┤
│                                                 │
│  Backend (Python FastAPI)                       │
│  - RESTful API for PRD management              │
│  - WebSocket server for real-time updates      │
│  - Ralph orchestrator service                  │
│  - Claude API + CLI integration                │
│  - File system monitoring                      │
│                                                 │
└─────────────────────────────────────────────────┘
```

## Quick Start

### 1. Start the Backend

```bash
cd backend
python -m venv venv
source venv/bin/activate  # On Windows: venv\Scripts\activate
pip install -r requirements.txt

# Configure environment
cp .env.example .env
# Edit .env with your settings (API key optional)

# Run the server
python run.py
```

Backend will be available at `http://localhost:8000`

### 2. Start the Frontend

```bash
cd flowchart
npm install

# Configure environment
cp .env.example .env
# Defaults should work for local development

# Run the dev server
npm run dev
```

Frontend will be available at `http://localhost:5173`

## Usage

### Creating a PRD

1. Open the web UI at `http://localhost:5173`
2. Click "Create PRD" in the dashboard
3. Fill in:
   - Project Name
   - Branch Name (e.g., `ralph/my-feature`)
   - Description
4. Click "Create"

### Adding User Stories

1. Once PRD is created, click "Add Story"
2. Enter:
   - Story title
   - Description
   - Acceptance criteria (comma-separated)
   - Priority (number, lower = higher priority)
3. Story will be added with `passes: false`

### Running Ralph

1. Configure max iterations (default: 10)
2. Choose execution mode:
   - **API Mode**: Uses Anthropic API (requires `ANTHROPIC_API_KEY` in backend `.env`)
   - **CLI Mode**: Uses `claude` or `amp` CLI (must be installed and authenticated)
3. Click "Start Ralph"
4. Watch real-time execution in:
   - **Orchestrator Chat**: Your conversation with Ralph
   - **Current Iteration**: Live subagent output
   - **Progress Dashboard**: Story completion status

### Monitoring Progress

- **Progress Overview**: See total/completed/pending stories
- **Story List**: Visual checkboxes for completed stories
- **Iteration Status**: Current iteration number and max
- **Live Messages**: Real-time streaming of agent actions

## API Documentation

Once the backend is running, visit:
- **Swagger UI**: `http://localhost:8000/docs`
- **ReDoc**: `http://localhost:8000/redoc`

### Key Endpoints

**PRD Management**
- `GET /api/prd/` - Get current PRD
- `POST /api/prd/` - Create new PRD
- `PUT /api/prd/` - Update PRD
- `POST /api/prd/stories` - Add user story
- `GET /api/prd/status` - Get completion status

**Ralph Control**
- `POST /api/ralph/start` - Start autonomous loop
- `POST /api/ralph/stop` - Stop running loop
- `GET /api/ralph/status` - Get orchestrator status

**WebSocket**
- `WS /ws` - Real-time updates

## WebSocket Messages

The WebSocket emits different message types:

- `orchestrator_message` - User ↔ orchestrator conversation
- `subagent_message` - Subagent output (current iteration)
- `iteration_start` - New iteration beginning
- `iteration_complete` - Iteration finished
- `story_update` - Story status changed
- `tool_call` - Tool being executed
- `tool_result` - Tool execution result
- `git_commit` - Git commit notification
- `progress_update` - Progress log updated
- `error` - Error occurred
- `complete` - All stories done

## Configuration

### Backend (.env)

```env
# Anthropic API Key (optional - will fall back to CLI if not set)
ANTHROPIC_API_KEY=your_api_key_here

# Server settings
HOST=0.0.0.0
PORT=8000

# Ralph settings
RALPH_SCRIPT_PATH=../ralph.sh
PRD_FILE_PATH=../prd.json
PROGRESS_FILE_PATH=../progress.txt
DEFAULT_MAX_ITERATIONS=10
```

### Frontend (.env)

```env
VITE_API_URL=http://localhost:8000
VITE_WS_URL=ws://localhost:8000/ws
```

## Development

### Backend

```bash
cd backend
source venv/bin/activate
python run.py  # Auto-reload enabled
```

### Frontend

```bash
cd flowchart
npm run dev  # Hot module replacement enabled
```

### Building for Production

**Backend:**
```bash
cd backend
pip install -r requirements.txt
uvicorn app.main:app --host 0.0.0.0 --port 8000
```

**Frontend:**
```bash
cd flowchart
npm run build
npm run preview
```

## Troubleshooting

**WebSocket won't connect**
- Ensure backend is running at `http://localhost:8000`
- Check CORS settings in `backend/app/core/config.py`
- Verify `VITE_WS_URL` in frontend `.env`

**Ralph won't start**
- Check that `CLAUDE.md` exists in project root
- Verify CLI tool is installed: `claude --version` or `amp --version`
- Check API key if using API mode
- Look at backend console for errors

**PRD not loading**
- Ensure `prd.json` path is correct in backend `.env`
- Check file permissions
- Verify JSON is valid

## Features in Detail

### Live Streaming

Ralph streams all output in real-time via WebSocket:
- See tool calls as they happen
- Watch code being written and tested
- Get notified of git commits immediately
- Track progress updates live

### Dual View Mode

Switch between:
- **Dashboard**: Interactive workspace with PRD editor, chat, and progress
- **Flowchart**: Visual explanation of how Ralph works (step-by-step)

### Progress Tracking

- Visual progress bar
- Story-by-story completion status
- Iteration counter
- Real-time status updates

## Contributing

See main [README.md](README.md) for the Ralph project overview.

## License

MIT License - see [LICENSE](LICENSE)
