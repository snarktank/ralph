from fastapi import APIRouter, HTTPException, BackgroundTasks
from typing import List
from pydantic import BaseModel
from ..models.project import Project, ProjectCreate, ProjectList, PRDCreate, PRDResponse
from ..services.project_generator import project_generator
from ..services.project_manager import project_manager
from ..services.ralph_runner import ralph_runner
from ..services.prd_generator import prd_generator
import json
import os
from pathlib import Path

router = APIRouter(prefix="/api/projects", tags=["projects"])


class PRDGenerateRequest(BaseModel):
    """Request to generate a PRD using AI"""
    user_prompt: str


class PRDUpdateRequest(BaseModel):
    """Request to update a PRD using AI"""
    update_prompt: str


@router.post("/", response_model=Project)
async def create_project(project_create: ProjectCreate, background_tasks: BackgroundTasks):
    """Create a new project with UI"""
    try:
        # Generate project
        project = await project_generator.create_project(
            name=project_create.name,
            description=project_create.description,
            user_request=project_create.user_request
        )

        # Add to manager
        project_manager.add_project(project)

        # Install dependencies in background
        background_tasks.add_task(project_manager.install_dependencies, project)

        return project

    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@router.get("/", response_model=ProjectList)
async def get_projects():
    """Get all projects"""
    projects = project_manager.get_all_projects()
    return ProjectList(projects=projects)


@router.get("/{project_id}", response_model=Project)
async def get_project(project_id: str):
    """Get a specific project"""
    project = project_manager.get_project(project_id)
    if not project:
        raise HTTPException(status_code=404, detail="Project not found")
    return project


@router.post("/{project_id}/start")
async def start_project(project_id: str, background_tasks: BackgroundTasks):
    """Start a project"""
    project = project_manager.get_project(project_id)
    if not project:
        raise HTTPException(status_code=404, detail="Project not found")

    if project.status == "created":
        raise HTTPException(status_code=400, detail="Project dependencies not installed yet")

    # Start in background
    background_tasks.add_task(project_manager.start_project, project)

    return {"message": f"Starting project {project_id}", "url": project.url}


@router.post("/{project_id}/stop")
async def stop_project(project_id: str):
    """Stop a project"""
    success = project_manager.stop_project(project_id)
    if not success:
        raise HTTPException(status_code=404, detail="Project not found or not running")

    return {"message": f"Stopped project {project_id}"}


@router.get("/{project_id}/status")
async def get_project_status(project_id: str):
    """Get project status"""
    project = project_manager.get_project(project_id)
    if not project:
        raise HTTPException(status_code=404, detail="Project not found")

    is_running = project_manager.is_project_running(project_id)

    return {
        "project_id": project_id,
        "status": project.status,
        "running": is_running,
        "url": project.url if is_running else None,
        "port": project.port
    }


@router.post("/{project_id}/ralph/start")
async def start_ralph_loop(project_id: str):
    """Start Ralph autonomous loop for a project"""
    project = project_manager.get_project(project_id)
    if not project:
        raise HTTPException(status_code=404, detail="Project not found")

    if ralph_runner.is_running(project_id):
        raise HTTPException(status_code=400, detail="Ralph is already running for this project")

    # Start Ralph loop (will send updates via WebSocket)
    success = await ralph_runner.start_ralph_loop(
        project_id=project_id,
        project_path=project.path
    )

    if not success:
        raise HTTPException(status_code=500, detail="Failed to start Ralph loop")

    # Update project status
    project.ralph_status = "running"

    return {"message": f"Started Ralph loop for project {project_id}"}


@router.post("/{project_id}/ralph/stop")
async def stop_ralph_loop(project_id: str):
    """Stop Ralph autonomous loop for a project"""
    project = project_manager.get_project(project_id)
    if not project:
        raise HTTPException(status_code=404, detail="Project not found")

    success = await ralph_runner.stop_ralph_loop(project_id)

    if not success:
        raise HTTPException(status_code=404, detail="Ralph is not running for this project")

    # Update project status
    project.ralph_status = "stopped"

    return {"message": f"Stopped Ralph loop for project {project_id}"}


@router.get("/{project_id}/ralph/status")
async def get_ralph_status(project_id: str):
    """Get Ralph status for a project"""
    project = project_manager.get_project(project_id)
    if not project:
        raise HTTPException(status_code=404, detail="Project not found")

    is_running = ralph_runner.is_running(project_id)
    conversation = ralph_runner.get_conversation(project_id)

    return {
        "project_id": project_id,
        "ralph_running": is_running,
        "conversation": conversation
    }


@router.post("/{project_id}/prd/generate", response_model=PRDResponse)
async def generate_prd(project_id: str, request: PRDGenerateRequest):
    """Generate a PRD using AI for a project"""
    project = project_manager.get_project(project_id)
    if not project:
        raise HTTPException(status_code=404, detail="Project not found")

    try:
        # Generate PRD using AI
        prd_data = await prd_generator.generate_prd(
            project_name=project.name,
            description=project.description,
            user_prompt=request.user_prompt
        )

        # Save to file
        prd_path = os.path.join(project.path, "prd.json")
        prd_dict = prd_data.model_dump()

        with open(prd_path, 'w') as f:
            json.dump(prd_dict, f, indent=2)

        # Update project metadata
        project.prd_path = prd_path
        project.has_prd = True

        return PRDResponse(**prd_dict, prd_path=prd_path)

    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Failed to generate PRD: {str(e)}")


@router.put("/{project_id}/prd/update", response_model=PRDResponse)
async def update_prd_with_ai(project_id: str, request: PRDUpdateRequest):
    """Update an existing PRD using AI"""
    project = project_manager.get_project(project_id)
    if not project:
        raise HTTPException(status_code=404, detail="Project not found")

    if not project.has_prd or not project.prd_path:
        raise HTTPException(status_code=400, detail="PRD must exist before updating")

    try:
        # Read current PRD
        with open(project.prd_path, 'r') as f:
            prd_dict = json.load(f)

        current_prd = PRDCreate(**prd_dict)

        # Update using AI
        updated_prd = await prd_generator.update_prd_from_prompt(
            current_prd=current_prd,
            update_prompt=request.update_prompt
        )

        # Save updated PRD
        updated_dict = updated_prd.model_dump()
        with open(project.prd_path, 'w') as f:
            json.dump(updated_dict, f, indent=2)

        return PRDResponse(**updated_dict, prd_path=project.prd_path)

    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Failed to update PRD: {str(e)}")


@router.post("/{project_id}/prd", response_model=PRDResponse)
async def create_prd(project_id: str, prd_data: PRDCreate):
    """Create or update a PRD file for a project (manual edit)"""
    project = project_manager.get_project(project_id)
    if not project:
        raise HTTPException(status_code=404, detail="Project not found")

    try:
        # Create PRD JSON file in project directory
        prd_path = os.path.join(project.path, "prd.json")

        # Convert to dict for JSON serialization
        prd_dict = prd_data.model_dump()

        with open(prd_path, 'w') as f:
            json.dump(prd_dict, f, indent=2)

        # Update project metadata
        project.prd_path = prd_path
        project.has_prd = True

        return PRDResponse(**prd_dict, prd_path=prd_path)

    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Failed to create PRD: {str(e)}")


@router.get("/{project_id}/prd", response_model=PRDResponse)
async def get_prd(project_id: str):
    """Get the PRD for a project"""
    project = project_manager.get_project(project_id)
    if not project:
        raise HTTPException(status_code=404, detail="Project not found")

    if not project.has_prd or not project.prd_path:
        raise HTTPException(status_code=404, detail="PRD not found for this project")

    try:
        with open(project.prd_path, 'r') as f:
            prd_data = json.load(f)

        return PRDResponse(**prd_data, prd_path=project.prd_path)

    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Failed to read PRD: {str(e)}")


@router.post("/{project_id}/ralph-config")
async def create_ralph_config(project_id: str):
    """Create Ralph configuration files (CLAUDE.md, progress.txt) from PRD"""
    project = project_manager.get_project(project_id)
    if not project:
        raise HTTPException(status_code=404, detail="Project not found")

    if not project.has_prd or not project.prd_path:
        raise HTTPException(status_code=400, detail="PRD must be created first")

    try:
        # Read the PRD
        with open(project.prd_path, 'r') as f:
            prd_data = json.load(f)

        # Create CLAUDE.md file
        claude_md_path = os.path.join(project.path, "CLAUDE.md")
        claude_md_content = """# Ralph Agent Instructions

You are an autonomous coding agent working on a software project.

## Your Task

1. Read the PRD at `prd.json` (in the same directory as this file)
2. Read the progress log at `progress.txt` (check Codebase Patterns section first)
3. Check you're on the correct branch from PRD `branchName`. If not, check it out or create from main.
4. Pick the **highest priority** user story where `passes: false`
5. Implement that single user story
6. Run quality checks (e.g., typecheck, lint, test - use whatever your project requires)
7. Update CLAUDE.md files if you discover reusable patterns (see below)
8. If checks pass, commit ALL changes with message: `feat: [Story ID] - [Story Title]`
9. Update the PRD to set `passes: true` for the completed story
10. Append your progress to `progress.txt`

## Progress Report Format

APPEND to progress.txt (never replace, always append):
```
## [Date/Time] - [Story ID]
- What was implemented
- Files changed
- **Learnings for future iterations:**
  - Patterns discovered (e.g., "this codebase uses X for Y")
  - Gotchas encountered (e.g., "don't forget to update Z when changing W")
  - Useful context (e.g., "the evaluation panel is in component X")
---
```

The learnings section is critical - it helps future iterations avoid repeating mistakes and understand the codebase better.

## Consolidate Patterns

If you discover a **reusable pattern** that future iterations should know, add it to the `## Codebase Patterns` section at the TOP of progress.txt (create it if it doesn't exist).

## Quality Requirements

- ALL commits must pass your project's quality checks (typecheck, lint, test)
- Do NOT commit broken code
- Keep changes focused and minimal
- Follow existing code patterns

## Stop Condition

After completing a user story, check if ALL stories have `passes: true`.

If ALL stories are complete and passing, reply with:
<promise>COMPLETE</promise>

If there are still stories with `passes: false`, end your response normally (another iteration will pick up the next story).

## Important

- Work on ONE story per iteration
- Commit frequently
- Keep CI green
- Read the Codebase Patterns section in progress.txt before starting
"""

        with open(claude_md_path, 'w') as f:
            f.write(claude_md_content)

        # Create initial progress.txt if it doesn't exist
        progress_path = os.path.join(project.path, "progress.txt")
        if not os.path.exists(progress_path):
            with open(progress_path, 'w') as f:
                f.write("## Codebase Patterns\n\n---\n")

        # Update project metadata
        project.has_ralph_config = True

        return {
            "message": "Ralph configuration created successfully",
            "claude_md_path": claude_md_path,
            "progress_txt_path": progress_path,
            "prd_path": project.prd_path
        }

    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Failed to create Ralph config: {str(e)}")
