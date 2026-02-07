import asyncio
import subprocess
import json
from pathlib import Path
from typing import Dict, Optional
from datetime import datetime
from ..models.project import Project
from .port_manager import port_manager
from ..core.config import settings


class ProjectManager:
    """Manages running projects"""

    def __init__(self):
        self.projects: Dict[str, Project] = {}
        self.processes: Dict[str, subprocess.Popen] = {}
        self.projects_base_path = Path(settings.RALPH_SCRIPT_PATH).parent.parent / "projects"
        self._load_existing_projects()

    def _load_existing_projects(self):
        """Load existing projects from disk"""
        if not self.projects_base_path.exists():
            return

        for project_dir in self.projects_base_path.iterdir():
            if not project_dir.is_dir():
                continue

            # Try to read project metadata
            metadata_file = project_dir / ".project.json"
            if metadata_file.exists():
                try:
                    with open(metadata_file, 'r') as f:
                        data = json.load(f)
                        # Add default values for new fields if missing (backward compatibility)
                        if 'has_prd' not in data:
                            data['has_prd'] = (project_dir / "prd.json").exists()
                        if 'has_ralph_config' not in data:
                            data['has_ralph_config'] = (project_dir / "CLAUDE.md").exists()
                        if 'ralph_status' not in data:
                            data['ralph_status'] = "not_started"

                        project = Project(**data)
                        # Update status to stopped if it was running
                        if project.status == "running":
                            project.status = "stopped"
                        if project.ralph_status == "running":
                            project.ralph_status = "stopped"
                        self.projects[project.id] = project
                except Exception as e:
                    print(f"Error loading project {project_dir.name}: {e}")

    def add_project(self, project: Project):
        """Add a project to the manager"""
        self.projects[project.id] = project
        # Save project metadata
        self._save_project_metadata(project)

    def _save_project_metadata(self, project: Project):
        """Save project metadata to disk"""
        try:
            project_path = Path(project.path)
            metadata_file = project_path / ".project.json"
            with open(metadata_file, 'w') as f:
                json.dump(project.model_dump(), f, indent=2, default=str)
        except Exception as e:
            print(f"Error saving project metadata: {e}")

    def get_project(self, project_id: str) -> Optional[Project]:
        """Get a project by ID"""
        return self.projects.get(project_id)

    def get_all_projects(self) -> list[Project]:
        """Get all projects"""
        return list(self.projects.values())

    async def install_dependencies(self, project: Project) -> bool:
        """Install project dependencies"""
        try:
            project.status = "installing"
            project_path = Path(project.path)

            # Run npm install
            process = await asyncio.create_subprocess_exec(
                "npm", "install",
                cwd=str(project_path),
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.PIPE
            )

            await process.communicate()

            if process.returncode == 0:
                project.status = "ready"
                return True
            else:
                project.status = "error"
                return False

        except Exception as e:
            print(f"Error installing dependencies: {e}")
            project.status = "error"
            return False

    async def start_project(self, project: Project) -> bool:
        """Start the project dev server"""
        try:
            project_path = Path(project.path)

            # Start npm run dev in background
            process = subprocess.Popen(
                ["npm", "run", "dev"],
                cwd=str(project_path),
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True
            )

            self.processes[project.id] = process
            project.status = "running"
            self._save_project_metadata(project)

            # Wait a bit for server to start
            await asyncio.sleep(3)

            return True

        except Exception as e:
            print(f"Error starting project: {e}")
            project.status = "error"
            self._save_project_metadata(project)
            return False

    def stop_project(self, project_id: str) -> bool:
        """Stop a running project"""
        if project_id in self.processes:
            process = self.processes[project_id]
            process.terminate()
            process.wait(timeout=5)
            del self.processes[project_id]

            project = self.projects.get(project_id)
            if project:
                project.status = "stopped"
                port_manager.release_port(project.port)
                self._save_project_metadata(project)

            return True
        return False

    def is_project_running(self, project_id: str) -> bool:
        """Check if a project is running"""
        if project_id in self.processes:
            return self.processes[project_id].poll() is None
        return False


# Global instance
project_manager = ProjectManager()
