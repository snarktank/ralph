from fastapi import APIRouter, HTTPException, BackgroundTasks
from typing import List
from ..models.project import Project, ProjectCreate, ProjectList
from ..services.project_generator import project_generator
from ..services.project_manager import project_manager
from ..services.ralph_runner import ralph_runner

router = APIRouter(prefix="/api/projects", tags=["projects"])


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

    return {"message": f"Started Ralph loop for project {project_id}"}


@router.post("/{project_id}/ralph/stop")
async def stop_ralph_loop(project_id: str):
    """Stop Ralph autonomous loop for a project"""
    success = await ralph_runner.stop_ralph_loop(project_id)

    if not success:
        raise HTTPException(status_code=404, detail="Ralph is not running for this project")

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
