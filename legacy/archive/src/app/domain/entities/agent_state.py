"""Agent State Entity for hexagonal architecture with Pydantic validation."""

from typing import Annotated, Dict, Any, Optional, List, Union
from pydantic import BaseModel, Field
from langgraph.graph import add_messages
from langchain_core.messages import AnyMessage
from app.domain.entities.plan import Plan
from app.domain.entities.agent_state_models import ToolCall, ToolResult, MemoryEntry


class AgentState(BaseModel):
    """State for the chat graph with enhanced workflow support."""

    user_id: Optional[str] = Field(default=None)
    thread_id: Optional[str] = Field(default=None)
    messages: Annotated[List[AnyMessage], add_messages]
    # Classification intent
    intent: Optional[str] = Field(default=None)
    # Plan Mode
    needs_planning: bool = Field(default=False)
    plan: Optional[Plan] = Field(default=None)
    # Memory - Now using validated models
    recalled_memory: Optional[Union[List[MemoryEntry], str]] = Field(default=None)
    needs_memory_recall: bool = Field(default=False)
    needs_memory_write: bool = Field(default=False)
    memory_operation: Optional[str] = Field(default=None)
    # Summarization
    memory_summary: Optional[str] = Field(default=None)
    conversation_summary: Optional[str] = Field(default=None)
    needs_conversation_summary: bool = Field(default=False)
    needs_memory_summary: bool = Field(default=False)

    # Tools - Now using validated models
    tool_calls: Optional[List[ToolCall]] = Field(default=None)
    tool_results: Optional[List[ToolResult]] = Field(default=None)
    should_call_tools: bool = Field(default=False)

    final_output: Optional[str] = Field(default=None)

    class Config:
        arbitrary_types_allowed = True
