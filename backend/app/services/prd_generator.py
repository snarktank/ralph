"""Service to generate PRDs using AI"""
import json
from typing import Optional
from anthropic import Anthropic
from ..core.config import settings
from ..models.project import PRDCreate, PRDUserStory


class PRDGenerator:
    """Generates PRDs from natural language descriptions using AI"""

    def __init__(self):
        self.client = None
        if settings.ANTHROPIC_API_KEY:
            self.client = Anthropic(api_key=settings.ANTHROPIC_API_KEY)

    async def generate_prd(self, project_name: str, description: str, user_prompt: str) -> PRDCreate:
        """
        Generate a PRD from a user prompt using Claude AI

        Args:
            project_name: Name of the project
            description: Short description of the project
            user_prompt: Natural language description of what to build

        Returns:
            PRDCreate object with generated user stories
        """
        if not self.client:
            raise ValueError("Anthropic API key not configured")

        # Create the system prompt
        system_prompt = """You are a product requirements expert. Generate a detailed PRD (Product Requirements Document) in JSON format.

The PRD should include:
1. projectName: Name of the project
2. branchName: A git branch name (format: feature/description)
3. description: Clear description of the project
4. userStories: Array of user stories with:
   - id: Story ID (format: US-001, US-002, etc.)
   - title: Short title (5-8 words)
   - description: What needs to be built
   - acceptanceCriteria: Array of specific, testable criteria
   - priority: Number (1 = highest priority)
   - passes: false (always start as false)

Make user stories:
- Specific and implementable
- Ordered by priority (1 is highest)
- Include clear acceptance criteria
- Focus on one feature per story

Return ONLY valid JSON, no markdown formatting."""

        user_message = f"""Generate a PRD for this project:

Project Name: {project_name}
Description: {description}

Requirements:
{user_prompt}

Generate 3-5 user stories that cover the main features."""

        try:
            # Call Claude API
            response = self.client.messages.create(
                model="claude-sonnet-4-20250514",
                max_tokens=4096,
                system=system_prompt,
                messages=[
                    {"role": "user", "content": user_message}
                ]
            )

            # Extract JSON from response
            prd_text = response.content[0].text.strip()

            # Remove markdown code blocks if present
            if prd_text.startswith("```"):
                lines = prd_text.split("\n")
                prd_text = "\n".join(lines[1:-1])
            if prd_text.startswith("json"):
                prd_text = prd_text[4:].strip()

            # Parse JSON
            prd_data = json.loads(prd_text)

            # Validate and create PRDCreate object
            return PRDCreate(**prd_data)

        except Exception as e:
            raise ValueError(f"Failed to generate PRD: {str(e)}")

    async def update_prd_from_prompt(self, current_prd: PRDCreate, update_prompt: str) -> PRDCreate:
        """
        Update an existing PRD based on a user prompt

        Args:
            current_prd: The current PRD
            update_prompt: Instructions for what to change

        Returns:
            Updated PRDCreate object
        """
        if not self.client:
            raise ValueError("Anthropic API key not configured")

        system_prompt = """You are a product requirements expert. Update the given PRD based on user instructions.

Return the UPDATED PRD in JSON format with the same structure:
- projectName
- branchName
- description
- userStories (array with id, title, description, acceptanceCriteria, priority, passes)

You can:
- Add new user stories
- Modify existing stories
- Remove stories
- Change priorities
- Update acceptance criteria

Return ONLY valid JSON, no markdown formatting."""

        current_prd_json = current_prd.model_dump_json(indent=2)

        user_message = f"""Current PRD:
{current_prd_json}

Update instructions:
{update_prompt}

Return the updated PRD."""

        try:
            response = self.client.messages.create(
                model="claude-sonnet-4-20250514",
                max_tokens=4096,
                system=system_prompt,
                messages=[
                    {"role": "user", "content": user_message}
                ]
            )

            # Extract JSON from response
            prd_text = response.content[0].text.strip()

            # Remove markdown code blocks if present
            if prd_text.startswith("```"):
                lines = prd_text.split("\n")
                prd_text = "\n".join(lines[1:-1])
            if prd_text.startswith("json"):
                prd_text = prd_text[4:].strip()

            # Parse JSON
            prd_data = json.loads(prd_text)

            # Validate and create PRDCreate object
            return PRDCreate(**prd_data)

        except Exception as e:
            raise ValueError(f"Failed to update PRD: {str(e)}")


# Global instance
prd_generator = PRDGenerator()
