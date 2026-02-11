"""Memory Write Node for AI agent workflow with LangMem integration."""

import logging
from typing import List, Dict, Any
from core.entities.agent_state import AgentState
from core.ports.llm_client_port import LLMClientPort
from core.ports.memory_port import MemoryPort

logger = logging.getLogger(__name__)


class MemoryWriteNode:
    """Node that writes conversation to memory using LangMem for long-term storage."""

    def __init__(self, llm_client: LLMClientPort, memory_adapter: MemoryPort):
        self.memory_adapter = memory_adapter

    async def write_memory(self, state: AgentState, user_id: str) -> AgentState:
        """Write conversation to memory with LangMem for long-term storage."""

        try:
            # Get the latest messages
            latest_messages = self._extract_latest_messages(state.messages)

            if not latest_messages:
                logger.info("No new messages to store")
                return state

            # Store important messages as long-term memories using LangMem
            stored_count = await self._store_important_memories(
                latest_messages, user_id
            )

            logger.info(
                f"Stored {stored_count} important memories to long-term storage for user {user_id}"
            )

            return state

        except Exception as e:
            logger.error(f"Error writing to memory: {str(e)}")
            return state

    async def execute_node(self, state: AgentState) -> AgentState:
        """Write conversation to memory using LangMem."""
        user_id = state.user_id
        try:
            # Use the MemoryWriteNode to write important memories
            state = await self.write_memory(state, user_id)
            logger.info(f"Memory write node executed for user {user_id}")
        except Exception as e:
            logger.error(f"Error in memory write node: {str(e)}")

        return state

    def _extract_latest_messages(self, messages: List[Any]) -> List[Dict[str, Any]]:
        """Extract the latest user and assistant messages from the conversation."""
        extracted_messages = []

        for msg in messages:
            if hasattr(msg, "type"):
                if msg.type == "human":
                    extracted_messages.append(
                        {"role": "user", "content": msg.content, "type": "human"}
                    )
                elif msg.type == "ai":
                    extracted_messages.append(
                        {"role": "assistant", "content": msg.content, "type": "ai"}
                    )

        return extracted_messages

    async def _store_important_memories(
        self, messages: List[Dict[str, Any]], user_id: str
    ) -> int:
        """Store important conversation memories using LangMem."""
        stored_count = 0

        for msg in messages:
            # Only store user messages and significant assistant responses
            if msg["type"] == "human":
                # Store user queries as long-term memories
                success = await self.memory_adapter.store_long_term_memory(
                    user_id=user_id,
                    content=msg["content"],
                    metadata={"role": "user", "type": "query"},
                )
                if success:
                    stored_count += 1

            elif msg["type"] == "ai" and self._is_significant_response(msg["content"]):
                # Store significant assistant responses
                success = await self.memory_adapter.store_long_term_memory(
                    user_id=user_id,
                    content=msg["content"],
                    metadata={"role": "assistant", "type": "response"},
                )
                if success:
                    stored_count += 1

        return stored_count

    def _is_significant_response(self, content: str) -> bool:
        """Determine if an assistant response is significant enough to store as long-term memory."""
        # Consider responses significant if they contain explanations, instructions, or important information
        significant_keywords = [
            "explain",
            "how to",
            "important",
            "key",
            "summary",
            "conclusion",
            "recommend",
            "suggest",
            "advice",
            "should",
            "must",
            "need to",
        ]

        content_lower = content.lower()
        return any(keyword in content_lower for keyword in significant_keywords)
