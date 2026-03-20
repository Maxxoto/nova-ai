"""Pydantic models for memory consolidation validation."""

from pydantic import BaseModel, Field
from typing import List, Optional
from datetime import datetime


class MemorySummary(BaseModel):
    """Structured and validated memory summary."""

    summary: str = Field(description="Consolidated summary of the conversation")
    key_topics: List[str] = Field(
        description="Key topics discussed in the conversation"
    )
    action_items: List[str] = Field(
        default_factory=list,
        description="Action items or follow-ups identified",
    )
    user_preferences: List[str] = Field(
        default_factory=list,
        description="User preferences or requirements mentioned",
    )
    entities_mentioned: List[str] = Field(
        default_factory=list,
        description="Important entities (people, places, technologies) mentioned",
    )
    timestamp: datetime = Field(
        default_factory=datetime.now,
        description="When this summary was created",
    )
    message_count: int = Field(description="Number of conversation messages summarized")
    conversation_start: Optional[datetime] = Field(
        default=None,
        description="When the conversation started",
    )
    conversation_end: Optional[datetime] = Field(
        default=None,
        description="When the conversation ended (for archived conversations)",
    )

    class Config:
        json_encoders = {
            datetime: lambda v: v.isoformat(),
        }


class MemoryExtract(BaseModel):
    """Important information extracted from conversation."""

    facts_learned: List[str] = Field(
        description="Important facts or information learned during conversation",
    )
    entities: List[str] = Field(
        description="Entities mentioned (people, organizations, technologies, etc.)",
    )
    decisions_made: List[str] = Field(
        default_factory=list,
        description="Decisions or conclusions reached",
    )
    questions_asked: List[str] = Field(
        default_factory=list,
        description="Questions the user asked (for follow-up)",
    )
    user_goals: List[str] = Field(
        default_factory=list,
        description="User goals or objectives identified",
    )
    technical_context: List[str] = Field(
        default_factory=list,
        description="Technical details or context discussed",
    )


class MemoryConsolidationPlan(BaseModel):
    """Plan for what to do with consolidated information."""

    should_store_long_term: bool = Field(
        description="Whether to store in long-term memory"
    )
    should_update_user_profile: bool = Field(
        default=False,
        description="Whether to update user profile/preferences",
    )
    suggested_followups: List[str] = Field(
        default_factory=list,
        description="Suggested follow-up actions or questions",
    )
    priority_level: str = Field(
        default="medium",
        description="Priority level of the conversation content",
    )
