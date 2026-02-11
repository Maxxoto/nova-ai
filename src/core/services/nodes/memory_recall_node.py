"""Memory Recall Node for AI agent workflow with LangMem integration."""

import logging
from core.entities.agent_state import AgentState
from core.ports.llm_client_port import LLMClientPort
from core.ports.memory_port import MemoryPort

logger = logging.getLogger(__name__)


class MemoryRecallNode:
    """Node that retrieves relevant memory for the conversation using LangMem."""

    def __init__(self, llm_client: LLMClientPort, memory_adapter: MemoryPort):
        self.llm_client = llm_client
        self.memory_adapter = memory_adapter

    async def recall_memory(self, state: AgentState) -> AgentState:
        """Recall relevant conversation memory using LangMem for long-term memory."""

        user_id = state.user_id
        try:
            # Get long-term memory summary using LangMem
            get_longterm_memory = await self.memory_adapter.get_longterm_memory(
                user_id, max_tokens=300
            )

            # Combine short-term and long-term memory
            recalled_memory = []

            # Add long-term memory summary
            if get_longterm_memory:
                recalled_memory.append(
                    {
                        "role": "system",
                        "content": f"{get_longterm_memory}",
                    }
                )
                logger.info(f"Recalled long-term memory, ${get_longterm_memory[:50]}")

            state.recalled_memory = recalled_memory

            if not recalled_memory:
                logger.info("No memory found for user")
            return state

        except Exception as e:
            logger.error(f"Error recalling memory: {str(e)}")
            state.recalled_memory = []
            return state

    async def execute_node(self, state: AgentState) -> AgentState:
        """Recall relevant memory from longterm"""
        user_id = state.user_id or "default_user"

        logger.info(f"Initiating memory recall for user: {user_id}")
        try:
            # Use the MemoryRecallNode to recall relevant memory
            state = await self.recall_memory(state)
            logger.info(
                f"Memory recall node executed for user {user_id}, recalled {len(state.recalled_memory or [])} memory entries"
            )
        except Exception as e:
            logger.error(f"Error in memory recall node: {str(e)}")
            state.recalled_memory = []

        return state
