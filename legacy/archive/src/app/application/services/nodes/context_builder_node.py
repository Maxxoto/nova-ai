"""Context builder node for LangGraph workflow."""

import logging
from typing import Dict, Any

from langchain_core.messages import SystemMessage

from app.domain.entities.agent_state import AgentState
from app.infrastructure.skills import ContextBuilder
from app.infrastructure.memory import MemoryStore


logger = logging.getLogger(__name__)


class ContextBuilderNode:
    """Node for building context with skills and memory."""

    def __init__(
        self,
        context_builder: ContextBuilder,
        memory_store: MemoryStore,
    ):
        """Initialize context builder node.

        Args:
            context_builder: Skills context builder
            memory_store: File-based memory store
        """
        self.context_builder = context_builder
        self.memory_store = memory_store

    async def execute_node(self, state: AgentState) -> AgentState:
        """Build context with skills and memory.

        Args:
            state: Current agent state (or dict from LangGraph)

        Returns:
            Updated agent state with context
        """
        logger.info("Building context with skills and memory")

        # Handle both AgentState object and dict formats
        if isinstance(state, dict):
            messages = state.get("messages", [])
        else:
            messages = state.messages

        # Get the latest user message
        latest_message = None
        for msg in reversed(messages):
            if hasattr(msg, "type") and msg.type == "human":
                latest_message = msg.content
                break

        if not latest_message:
            logger.warning("No user message found in state")
            return state

        # Build system prompt with skills context
        system_prompt = self.context_builder.build_system_prompt(
            include_memory=True,
            include_skills=True,
        )

        # Check if first message is already a system message
        has_system = False
        if messages and hasattr(messages[0], "type"):
            if messages[0].type == "system":
                has_system = True

        # Update or add system message
        if has_system:
            messages[0] = SystemMessage(content=system_prompt)
        else:
            messages.insert(0, SystemMessage(content=system_prompt))

        logger.info("Context built successfully")
        return state
