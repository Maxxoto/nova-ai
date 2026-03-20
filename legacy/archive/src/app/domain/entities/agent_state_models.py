"""Pydantic models for validated agent state fields."""

from typing import List, Dict, Any, Optional
from pydantic import BaseModel, Field, field_validator


class ToolCall(BaseModel):
    """Validated tool call structure."""

    id: str = Field(description="Unique tool call identifier")
    type: str = Field(default="function", description="Tool call type")
    function: "ToolFunction" = Field(description="Function to call")

    @field_validator("id")
    @classmethod
    def id_format(cls, v: str) -> str:
        """Validate tool call ID format."""
        if not v:
            raise ValueError("Tool call ID cannot be empty")
        return v

    @field_validator("type")
    @classmethod
    def type_valid(cls, v: str) -> str:
        """Validate tool call type."""
        if v not in ["function"]:
            raise ValueError(f"Invalid tool call type: {v}")
        return v


class ToolFunction(BaseModel):
    """Tool function specification."""

    name: str = Field(description="Function name")
    arguments: Dict[str, Any] = Field(description="Function arguments")

    @field_validator("name")
    @classmethod
    def name_not_empty(cls, v: str) -> str:
        """Validate function name."""
        if not v.strip():
            raise ValueError("Function name cannot be empty")
        return v


class ToolResult(BaseModel):
    """Validated tool result structure."""

    tool_call_id: str = Field(description="Corresponding tool call ID")
    name: str = Field(description="Tool name")
    content: str = Field(description="Tool result content")

    @field_validator("content")
    @classmethod
    def content_not_empty(cls, v: str) -> str:
        """Validate tool result content."""
        if not v:
            return "Tool executed successfully (no output)"
        return v


class MemoryEntry(BaseModel):
    """Validated memory entry."""

    content: str = Field(description="Memory content")
    timestamp: str = Field(description="When memory was created")
    importance: int = Field(
        default=5, ge=1, le=10, description="Memory importance score"
    )
    tags: List[str] = Field(default_factory=list, description="Memory tags")

    @field_validator("content")
    @classmethod
    def content_meaningful(cls, v: str) -> str:
        """Validate memory content is meaningful."""
        if len(v.strip()) < 5:
            raise ValueError("Memory content too short (min 5 chars)")
        return v.strip()


# Update forward references
ToolCall.model_rebuild()


class AgentStateModels:
    """Container for validated agent state models.

    This provides a clean namespace for all agent state-related Pydantic models.
    """

    ToolCall = ToolCall
    ToolFunction = ToolFunction
    ToolResult = ToolResult
    MemoryEntry = MemoryEntry
