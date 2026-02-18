"""Agent State Entity for hexagonal architecture."""

from typing import Annotated, Dict, Any, Optional, List, Union
from pydantic import BaseModel, Field
from langgraph.graph import add_messages
from langchain_core.messages import AnyMessage
from app.domain.entities.plan import Plan


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
    # Memory
    recalled_memory: Optional[Union[List[Dict[str, Any]], str]] = Field(default=None)
    needs_memory_recall: bool = Field(default=False)
    needs_memory_write: bool = Field(default=False)
    memory_operation: Optional[str] = Field(default=None)
    # Summarization
    memory_summary: Optional[str] = Field(default=None)
    conversation_summary: Optional[str] = Field(default=None)
    needs_conversation_summary: bool = Field(default=False)
    needs_memory_summary: bool = Field(default=False)

    # Tools
    tool_calls: Optional[List[Dict[str, Any]]] = Field(default=None)
    tool_results: Optional[List[Dict[str, Any]]] = Field(default=None)
    should_call_tools: bool = Field(default=False)

    final_output: Optional[str] = Field(default=None)

    class Config:
        arbitrary_types_allowed = True
