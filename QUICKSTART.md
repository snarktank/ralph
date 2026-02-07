# Ralph Web UI - Quick Start Guide

Get the Ralph Web UI up and running in 5 minutes!

## Prerequisites

- Python 3.9+
- Node.js 18+
- (Optional) Anthropic API key for API mode
- (Optional) Claude CLI or Amp CLI installed for CLI mode

## 1. Clone and Setup

```bash
cd ralph-with-ui
```

## 2. Start the Backend (Terminal 1)

```bash
# Navigate to backend
cd backend

# Create virtual environment
python -m venv venv

# Activate it
source venv/bin/activate  # On Windows: venv\Scripts\activate

# Install dependencies
pip install -r requirements.txt

# Configure environment (optional API key)
cp .env.example .env
# Edit .env if you want to use Anthropic API instead of CLI

# Start the server
python run.py
```

You should see:
```
INFO:     Uvicorn running on http://0.0.0.0:8000
```

## 3. Start the Frontend (Terminal 2)

```bash
# Navigate to frontend
cd flowchart

# Install dependencies
npm install

# Copy environment file (defaults work for local dev)
cp .env.example .env

# Start dev server
npm run dev
```

You should see:
```
  VITE v7.x.x  ready in xxx ms

  ➜  Local:   http://localhost:5173/
```

## 4. Open the Web UI

Open your browser to: **http://localhost:5173**

You should see the Ralph dashboard with two view options:
- **Dashboard** (default) - Interactive workspace
- **Flowchart** - Visual explanation

## 5. Create Your First PRD

1. Click **"Create PRD"** button
2. Fill in:
   - **Project Name:** "My Test Feature"
   - **Branch Name:** "ralph/test-feature"
   - **Description:** "Testing Ralph Web UI"
3. Click **"Create"**

## 6. Add a User Story

1. Click **"Add Story"**
2. Enter when prompted:
   - **Title:** "Add hello world function"
   - **Description:** "Create a simple hello world function in Python"
   - **Criteria:** "Function prints Hello World, Has a test, Test passes"
   - **Priority:** 1
3. Story appears in the PRD editor and progress dashboard

## 7. Run Ralph

1. In the **Control Panel** (top right):
   - **Max Iterations:** 10 (default is fine)
   - **Use CLI:** Check this if you don't have an API key
2. Click **"Start Ralph"**
3. Watch the magic happen! 🎉

### What You'll See:

**Orchestrator Chat (bottom left)**
- Your conversation with Ralph
- "Starting Ralph autonomous loop..."
- "Starting iteration 1 of 10"

**Current Iteration (bottom right)**
- Real-time subagent output
- Tool calls and results
- Code being written and tested

**Progress Dashboard (top right)**
- Stats update as stories complete
- Story checkboxes turn green
- Progress bar fills up

## 8. Monitor Progress

As Ralph runs, you'll see:
- ✓ Stories marked complete (green)
- Git commits in the subagent panel
- Progress updates in real-time
- Iteration counter advancing

When complete, you'll see:
- **"All stories completed! Ralph is done."**
- Progress bar at 100%
- All stories with green checkmarks

## Troubleshooting

### WebSocket Won't Connect
```bash
# Check backend is running
curl http://localhost:8000/health
# Should return: {"status":"healthy"}
```

### Backend Won't Start
```bash
# Check Python version
python --version  # Should be 3.9+

# Reinstall dependencies
pip install -r requirements.txt --force-reinstall
```

### Frontend Won't Start
```bash
# Check Node version
node --version  # Should be 18+

# Clear cache and reinstall
rm -rf node_modules package-lock.json
npm install
```

### Ralph Won't Start (CLI Mode)
```bash
# Verify CLI is installed
claude --version
# or
amp --version

# Make sure you're authenticated
```

### Ralph Won't Start (API Mode)
```bash
# Check your .env file
cat backend/.env

# Make sure ANTHROPIC_API_KEY is set
# Test the key works
python -c "from anthropic import Anthropic; print(Anthropic().messages.create(model='claude-sonnet-4-5-20250929', messages=[{'role':'user','content':'hi'}], max_tokens=10).content)"
```

## Next Steps

Now that you have it running:

1. **Explore the Dashboard**
   - Try the JSON view in PRD editor
   - Watch the flowchart visualization
   - See live WebSocket updates

2. **Create a Real PRD**
   - Use the `/prd` skill in Claude Code to generate a proper PRD
   - Convert it with `/ralph` skill to prd.json
   - Upload it via the UI or place in project root

3. **Customize**
   - Edit `backend/.env` for your project paths
   - Adjust max iterations
   - Configure CORS for remote access

4. **Deploy** (Optional)
   - See `README_WEB_UI.md` for production deployment
   - Use nginx/apache for frontend static files
   - Run backend with proper WSGI server

## Useful URLs

- **Frontend:** http://localhost:5173
- **Backend API:** http://localhost:8000
- **API Docs (Swagger):** http://localhost:8000/docs
- **API Docs (ReDoc):** http://localhost:8000/redoc
- **Health Check:** http://localhost:8000/health

## Getting Help

- **Backend logs:** Check Terminal 1 for Python errors
- **Frontend logs:** Check Terminal 2 for build errors
- **Browser console:** Press F12 to see JavaScript errors
- **WebSocket:** Check Network tab in DevTools for WS connection

## What's Next?

Check out:
- `README_WEB_UI.md` - Detailed documentation
- `IMPLEMENTATION_SUMMARY.md` - Technical architecture
- `README.md` - Original Ralph documentation
- API docs at http://localhost:8000/docs

Enjoy building with Ralph! 🚀
