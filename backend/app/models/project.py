from pydantic import BaseModel
from typing import Optional, List
from datetime import datetime


class ProjectCreate(BaseModel):
    """Request to create a new project"""
    name: str
    description: str
    user_request: str  # Natural language description


class Project(BaseModel):
    """Project model"""
    id: str
    name: str
    description: str
    path: str
    port: int
    stack: str  # "react-vite", "next", "vue", etc.
    status: str  # "created", "installing", "running", "stopped"
    created_at: datetime
    url: Optional[str] = None
    prd_path: Optional[str] = None
    has_prd: bool = False
    has_ralph_config: bool = False
    ralph_status: str = "not_started"  # "not_started", "running", "stopped", "completed"
    ralph_events_path: Optional[str] = None  # Path to ralph_events.jsonl file
    ralph_last_event_time: Optional[datetime] = None  # Timestamp of last event


class ProjectList(BaseModel):
    """List of projects"""
    projects: List[Project]


class PRDUserStory(BaseModel):
    """User story in a PRD"""
    id: str
    title: str
    description: str
    acceptanceCriteria: List[str]
    priority: int
    passes: bool = False


class PRDCreate(BaseModel):
    """Request to create a PRD for a project"""
    projectName: str
    branchName: str
    description: str
    userStories: List[PRDUserStory]


class PRDResponse(BaseModel):
    """PRD response"""
    projectName: str
    branchName: str
    description: str
    userStories: List[PRDUserStory]
    prd_path: str
