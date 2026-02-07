from fastapi import APIRouter, HTTPException
from typing import List
from ..models.prd import PRD, UserStory, PRDCreate, PRDUpdate
from ..services.prd_service import prd_service

router = APIRouter(prefix="/api/prd", tags=["prd"])


@router.get("/", response_model=PRD)
async def get_prd():
    """Get the current PRD"""
    prd = await prd_service.get_prd()
    if not prd:
        raise HTTPException(status_code=404, detail="No PRD found")
    return prd


@router.post("/", response_model=PRD)
async def create_prd(prd_create: PRDCreate):
    """Create a new PRD"""
    try:
        return await prd_service.create_prd(prd_create)
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@router.put("/", response_model=PRD)
async def update_prd(prd_update: PRDUpdate):
    """Update the existing PRD"""
    try:
        return await prd_service.update_prd(prd_update)
    except ValueError as e:
        raise HTTPException(status_code=404, detail=str(e))
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@router.delete("/")
async def delete_prd():
    """Delete the PRD"""
    try:
        await prd_service.delete_prd()
        return {"message": "PRD deleted successfully"}
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@router.post("/stories", response_model=PRD)
async def add_user_story(user_story: UserStory):
    """Add a user story to the PRD"""
    try:
        return await prd_service.add_user_story(user_story)
    except ValueError as e:
        raise HTTPException(status_code=404, detail=str(e))
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@router.put("/stories/{story_id}", response_model=PRD)
async def update_user_story(story_id: str, passes: bool):
    """Update a user story's passes status"""
    try:
        return await prd_service.update_user_story(story_id, passes)
    except ValueError as e:
        raise HTTPException(status_code=404, detail=str(e))
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@router.get("/next-story", response_model=UserStory)
async def get_next_story():
    """Get the next incomplete story"""
    story = await prd_service.get_next_incomplete_story()
    if not story:
        raise HTTPException(status_code=404, detail="No incomplete stories found")
    return story


@router.get("/status")
async def get_status():
    """Get PRD completion status"""
    prd = await prd_service.get_prd()
    if not prd:
        return {
            "exists": False,
            "total_stories": 0,
            "completed_stories": 0,
            "incomplete_stories": 0,
            "all_complete": False
        }

    completed = sum(1 for story in prd.userStories if story.passes)
    total = len(prd.userStories)

    return {
        "exists": True,
        "total_stories": total,
        "completed_stories": completed,
        "incomplete_stories": total - completed,
        "all_complete": completed == total and total > 0
    }
