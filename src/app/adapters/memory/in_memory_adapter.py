"""In-Memory Memory Adapter for hexagonal architecture with LangMem integration."""

import logging
from typing import List, Dict, Any, Optional

from langchain_core.stores import BaseStore
from langgraph.checkpoint.memory import InMemorySaver
from langmem import create_manage_memory_tool, create_search_memory_tool

from app.domain.ports.memory_port import MemoryPort
from app.domain.ports.llm_client_port import LLMClientPort


logger = logging.getLogger(__name__)


class InMemoryMemoryAdapter(MemoryPort):
    """In-memory memory adapter implementing MemoryPort with LangMem integration."""

    def __init__(
        self,
        llm_client: LLMClientPort,
        thread_memory_saver: InMemorySaver,
        longterm_memory_store: BaseStore,
    ):
        self.memory = thread_memory_saver
        self.lt_memory = longterm_memory_store
        self.llm_client = llm_client

        # Initialize LangMem tools for long-term memory (user-scoped)
        self.manage_memory_tool = create_manage_memory_tool(
            namespace=("memories", "{user_id}"), store=longterm_memory_store
        )
        self.search_memory_tool = create_search_memory_tool(
            namespace=("memories", "{user_id}"), store=longterm_memory_store
        )

    async def get_conversation_history(
        self, thread_id: str
    ) -> Optional[List[Dict[str, Any]]]:
        """Get conversation history from a thread memory."""
        try:
            # Get the current state from memory
            config = {"configurable": {"thread_id": thread_id}}
            snapshot = self.memory.get(config)

            if snapshot and "messages" in snapshot:
                return snapshot["messages"]
            return None
        except Exception:
            return None

    async def clear_conversation_memory(self, thread_id: str) -> bool:
        """Clear memory for a specific conversation thread."""
        # TODO: IMPLEMENT LATER
        try:
            # LangGraph MemorySaver doesn't have a direct clear method
            # We can simulate clearing by starting a new conversation
            return True
        except Exception:
            return False

    async def get_longterm_memory(
        self, user_id: str, max_tokens: int = 500
    ) -> Optional[str]:
        """Retrieve a summarized version of the long-term conversation history for a user using LangMem."""
        try:
            # Use LangMem search tool to retrieve relevant long-term memory
            # Search for general conversation memories for this user
            logger.info(f"Searching for memories for user: {user_id}")
            search_result = await self.search_memory_tool.ainvoke(
                {
                    "query": f"user {user_id} conversation memories preferences history",
                },
                config={"configurable": {"user_id": user_id}},
            )

            logger.debug(f"Search result type: {type(search_result)}")
            logger.debug(f"Search result: {search_result}")

            # If we have actual search results, process them
            if (
                search_result
                and isinstance(search_result, str)
                and len(search_result.strip()) > 0
            ):
                return search_result
            else:
                logger.info(f"No memories found for user {user_id}")
                return None

        except Exception as e:
            logger.error(
                f"Error retrieving long-term memory for user {user_id}: {str(e)}"
            )
            return None

    async def store_long_term_memory(
        self, user_id: str, content: str, metadata: Optional[Dict[str, Any]] = None
    ) -> bool:
        """Store a memory in long-term storage using LangMem."""
        try:
            # Use LangMem manage tool to store memory
            logger.info(
                f"Stored long-term memory for user {user_id}: {content[:50]}..."
            )
            result = await self.manage_memory_tool.ainvoke(
                {
                    "action": "create",
                    "content": content,
                },
                config={"configurable": {"user_id": user_id}},
            )
            logger.debug(f"Memory storage tool result: {result}")
            return True
        except Exception as e:
            logger.error(f"Error storing long-term memory: {str(e)}")
            return False
