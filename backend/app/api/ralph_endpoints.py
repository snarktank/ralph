from fastapi import APIRouter, HTTPException, BackgroundTasks
from pydantic import BaseModel
from typing import Optional
from ..services.ralph_orchestrator import ralph_orchestrator

router = APIRouter(prefix="/api/ralph", tags=["ralph"])


class StartRalphRequest(BaseModel):
    max_iterations: Optional[int] = 10
    use_cli: bool = False


@router.post("/start")
async def start_ralph(request: StartRalphRequest, background_tasks: BackgroundTasks):
    """Start the Ralph autonomous loop"""
    if ralph_orchestrator.is_running():
        raise HTTPException(status_code=400, detail="Ralph is already running")

    # Run in background
    background_tasks.add_task(
        ralph_orchestrator.start_loop,
        max_iterations=request.max_iterations,
        use_cli=request.use_cli
    )

    return {
        "message": "Ralph loop started",
        "max_iterations": request.max_iterations,
        "use_cli": request.use_cli
    }


@router.post("/stop")
async def stop_ralph():
    """Stop the Ralph autonomous loop"""
    if not ralph_orchestrator.is_running():
        raise HTTPException(status_code=400, detail="Ralph is not running")

    await ralph_orchestrator.stop_loop()
    return {"message": "Ralph loop stop requested"}


@router.get("/status")
async def get_ralph_status():
    """Get Ralph orchestrator status"""
    return {
        "running": ralph_orchestrator.is_running(),
        "current_iteration": ralph_orchestrator.get_current_iteration(),
        "max_iterations": ralph_orchestrator.max_iterations
    }
