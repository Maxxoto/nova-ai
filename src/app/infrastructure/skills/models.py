"""Pydantic models for skills validation."""

from pathlib import Path
from typing import List, Optional
from pydantic import BaseModel, Field, field_validator


class SkillRequirements(BaseModel):
    """Skill dependency requirements."""

    tools: List[str] = Field(default_factory=list, description="Required tools")
    apis: List[str] = Field(default_factory=list, description="Required API keys")
    skills: List[str] = Field(default_factory=list, description="Required skills")

    class Config:
        json_encoders = {Path: lambda v: str(v)}


class SkillMetadata(BaseModel):
    """Skill metadata."""

    author: Optional[str] = Field(default=None, description="Skill author")
    version: str = Field(default="1.0.0", description="Skill version")
    tags: List[str] = Field(default_factory=list, description="Skill tags")
    created: Optional[str] = Field(default=None, description="Creation date")
    updated: Optional[str] = Field(default=None, description="Last update date")


class Skill(BaseModel):
    """Validated skill structure."""

    name: str = Field(min_length=1, max_length=100, description="Skill name")
    description: str = Field(
        min_length=1, max_length=500, description="Skill description"
    )
    content: str = Field(description="Skill content/instructions")
    always_load: bool = Field(default=False, description="Always load skill")
    requirements: SkillRequirements = Field(default_factory=SkillRequirements)
    metadata: SkillMetadata = Field(default_factory=SkillMetadata)
    source: Optional[Path] = Field(default=None, description="Skill source file")

    @field_validator("name")
    @classmethod
    def name_alphanumeric(cls, v: str) -> str:
        """Validate skill name is alphanumeric with underscores/hyphens."""
        import re

        if not re.match(r"^[a-zA-Z0-9_-]+$", v):
            raise ValueError(
                "Skill name must contain only letters, numbers, underscores, or hyphens"
            )
        return v

    @field_validator("description")
    @classmethod
    def description_meaningful(cls, v: str) -> str:
        """Validate description is meaningful."""
        if len(v.strip()) < 10:
            raise ValueError("Description must be at least 10 characters")
        return v.strip()

    class Config:
        arbitrary_types_allowed = True
        json_encoders = {Path: lambda v: str(v)}
