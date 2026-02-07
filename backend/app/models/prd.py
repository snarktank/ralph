from pydantic import BaseModel, Field
from typing import List, Optional


class AcceptanceCriteria(BaseModel):
    """Single acceptance criteria item"""
    description: str


class UserStory(BaseModel):
    """User story model matching prd.json format"""
    id: str
    title: str
    description: str
    acceptanceCriteria: List[str]
    priority: int
    passes: bool = False


class PRD(BaseModel):
    """Product Requirements Document model"""
    projectName: str
    branchName: str
    description: str
    userStories: List[UserStory]


class PRDCreate(BaseModel):
    """Request model for creating a new PRD"""
    projectName: str
    branchName: str
    description: str


class PRDUpdate(BaseModel):
    """Request model for updating PRD"""
    projectName: Optional[str] = None
    branchName: Optional[str] = None
    description: Optional[str] = None
    userStories: Optional[List[UserStory]] = None
