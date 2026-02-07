import json
import aiofiles
from pathlib import Path
from typing import Optional
from ..models.prd import PRD, UserStory, PRDCreate, PRDUpdate
from ..core.config import settings


class PRDService:
    """Service for managing PRD files"""

    def __init__(self, prd_file_path: Path = None):
        self.prd_file_path = prd_file_path or settings.PRD_FILE_PATH

    async def get_prd(self) -> Optional[PRD]:
        """Load the current PRD from file"""
        if not self.prd_file_path.exists():
            return None

        async with aiofiles.open(self.prd_file_path, 'r') as f:
            content = await f.read()
            data = json.loads(content)
            return PRD(**data)

    async def create_prd(self, prd_create: PRDCreate) -> PRD:
        """Create a new PRD"""
        prd = PRD(
            projectName=prd_create.projectName,
            branchName=prd_create.branchName,
            description=prd_create.description,
            userStories=[]
        )
        await self.save_prd(prd)
        return prd

    async def update_prd(self, prd_update: PRDUpdate) -> PRD:
        """Update the existing PRD"""
        current_prd = await self.get_prd()
        if not current_prd:
            raise ValueError("No PRD exists to update")

        # Update fields if provided
        if prd_update.projectName is not None:
            current_prd.projectName = prd_update.projectName
        if prd_update.branchName is not None:
            current_prd.branchName = prd_update.branchName
        if prd_update.description is not None:
            current_prd.description = prd_update.description
        if prd_update.userStories is not None:
            current_prd.userStories = prd_update.userStories

        await self.save_prd(current_prd)
        return current_prd

    async def save_prd(self, prd: PRD):
        """Save PRD to file"""
        # Ensure parent directory exists
        self.prd_file_path.parent.mkdir(parents=True, exist_ok=True)

        async with aiofiles.open(self.prd_file_path, 'w') as f:
            await f.write(prd.model_dump_json(indent=2))

    async def add_user_story(self, user_story: UserStory) -> PRD:
        """Add a user story to the PRD"""
        prd = await self.get_prd()
        if not prd:
            raise ValueError("No PRD exists")

        prd.userStories.append(user_story)
        await self.save_prd(prd)
        return prd

    async def update_user_story(self, story_id: str, passes: bool) -> PRD:
        """Update a user story's passes status"""
        prd = await self.get_prd()
        if not prd:
            raise ValueError("No PRD exists")

        for story in prd.userStories:
            if story.id == story_id:
                story.passes = passes
                break
        else:
            raise ValueError(f"Story {story_id} not found")

        await self.save_prd(prd)
        return prd

    async def delete_prd(self):
        """Delete the PRD file"""
        if self.prd_file_path.exists():
            self.prd_file_path.unlink()

    async def get_next_incomplete_story(self) -> Optional[UserStory]:
        """Get the highest priority story where passes=false"""
        prd = await self.get_prd()
        if not prd:
            return None

        incomplete_stories = [s for s in prd.userStories if not s.passes]
        if not incomplete_stories:
            return None

        # Sort by priority (lower number = higher priority)
        incomplete_stories.sort(key=lambda s: s.priority)
        return incomplete_stories[0]

    async def all_stories_complete(self) -> bool:
        """Check if all stories have passes=true"""
        prd = await self.get_prd()
        if not prd or not prd.userStories:
            return False

        return all(story.passes for story in prd.userStories)


# Global instance
prd_service = PRDService()
