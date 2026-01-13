# Ralph PRD UI

A full-stack web application for creating Product Requirements Documents (PRDs) and converting them to Ralph's JSON format.

## Features

- **PRD Creation Wizard**: Guided multi-step process to create PRDs
- **PRD to JSON Conversion**: Convert markdown PRDs to Ralph's `prd.json` format
- **Project Management**: Point to any project directory to manage PRDs
- **Real-time Preview**: See PRD markdown and JSON previews as you work
- **Folder Selection**: Browse and select project folders (with manual path entry fallback)

## Prerequisites

- Node.js 18+ 
- pnpm (package manager)

## Setup

### Install Dependencies

From the root directory:

```bash
pnpm install
```

This will install dependencies for both backend and frontend.

### Environment Variables

Copy the example environment file:

```bash
cp backend/.env.example backend/.env
```

Edit `backend/.env` if needed (defaults should work for local development).

## Development

### Run Backend

```bash
cd backend
pnpm run dev
```

Backend runs on `http://localhost:3001` using nodemon for hot reload.

### Run Frontend

```bash
cd frontend
pnpm run dev
```

Frontend runs on `http://localhost:5173`

### Run Both (from root)

```bash
pnpm run dev
```

This runs both backend and frontend in parallel.

## Testing

### Run All Tests

```bash
pnpm test
```

### Run Frontend Tests

```bash
pnpm test:frontend
# or
cd frontend && pnpm test
```

### Run Backend Tests

```bash
pnpm test:backend
# or
cd backend && pnpm test
```

### Test Coverage

```bash
pnpm test:coverage
```

### Frontend Test UI

```bash
cd frontend
pnpm test:ui
```

Opens Vitest UI for interactive test running.

## Usage

1. **Start the application**: Run both backend and frontend servers
2. **Open browser**: Navigate to `http://localhost:5173`
3. **Create PRD**: 
   - Click "Create PRD"
   - enter path manually
   - Follow the wizard to create a PRD
4. **Convert to JSON**:
   - Click "Convert to JSON"
   - Select your project directory
   - Choose an existing PRD file or paste PRD content
   - Review the generated JSON
   - Save `prd.json` to your project root

## Project Structure

```
prd-ui/
├── backend/          # Express API server
│   ├── routes/       # API route handlers
│   ├── services/     # Business logic
│   └── utils/        # Utility functions
├── frontend/         # React + Vite frontend
│   └── src/
│       ├── components/  # React components
│       ├── hooks/       # Custom React hooks
│       ├── services/    # API client
│       ├── types/       # TypeScript types
│       └── test/        # Test setup
└── README.md
```

## API Endpoints

### Project
- `POST /api/project/validate` - Validate project path
- `GET /api/project/prds` - List PRD files

### PRD
- `POST /api/prd/generate-questions` - Generate clarifying questions
- `POST /api/prd/create` - Create PRD file
- `GET /api/prd/read` - Read existing PRD
- `PUT /api/prd/update` - Update PRD

### Convert
- `POST /api/convert` - Convert PRD to JSON
- `POST /api/convert/save` - Save prd.json

## Building for Production

```bash
cd frontend
pnpm run build
```

Built files will be in `frontend/dist/`

## Security Notes

- All file paths are validated to prevent directory traversal attacks
- File operations are restricted to specified project directories
- Input validation is performed on all user inputs

## Troubleshooting

**Backend won't start / Port already in use:**
- If you see `EADDRINUSE: address already in use :::3001`:
  ```bash
  cd backend
  pnpm run kill-port  # Kills process on port 3001
  # Or manually: lsof -ti:3001 | xargs kill -9
  ```
- Or change the port in `backend/.env`: `PORT=3002`
- Verify Node.js version is 18+

**Frontend won't start:**
- Check that port 5173 is not in use
- Clear `node_modules` and reinstall: `pnpm install`

**API calls fail:**
- Ensure backend is running on port 3001
- Check CORS settings in `backend/server.js`
- Verify backend server started successfully (check console for errors)

**Tests fail:**
- Run `pnpm install` to ensure all dependencies are installed
- Check that test files are in the correct locations

**Nodemon crashes:**
- Check the error message in the console
- Ensure all dependencies are installed: `pnpm install`
- Check that `.env` file exists (or create from `.env.example`)
- Verify no syntax errors in server.js or route files

## License

Part of the Ralph project.
