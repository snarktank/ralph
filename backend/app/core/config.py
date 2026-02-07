from pydantic_settings import BaseSettings
from pathlib import Path
from typing import Optional


class Settings(BaseSettings):
    """Application settings"""

    # API settings
    HOST: str = "0.0.0.0"
    PORT: int = 8000

    # Anthropic API
    ANTHROPIC_API_KEY: Optional[str] = None

    # Ralph settings
    RALPH_SCRIPT_PATH: Path = Path("../ralph.sh")
    PRD_FILE_PATH: Path = Path("../prd.json")
    PROGRESS_FILE_PATH: Path = Path("../progress.txt")
    DEFAULT_MAX_ITERATIONS: int = 10

    # CORS settings
    CORS_ORIGINS: list[str] = ["http://localhost:5173", "http://localhost:3000"]

    class Config:
        env_file = ".env"
        case_sensitive = True


settings = Settings()
