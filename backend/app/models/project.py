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


class ProjectList(BaseModel):
    """List of projects"""
    projects: List[Project]
