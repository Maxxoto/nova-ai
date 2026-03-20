"""Memory Port for hexagonal architecture."""

from abc import ABC, abstractmethod
from typing import List, Dict, Any, Optional


class MemoryPort(ABC):
    """Abstract base class for memory implementations."""

    @abstractmethod
    async def get_conversation_history(
        self, user_id: str
    ) -> Optional[List[Dict[str, Any]]]:
        """Get conversation history for a user.

        Args:
            user_id: User ID for conversation context

        Returns:
            List of conversation messages or None if not found
        """
        pass

    @abstractmethod
    async def get_longterm_memory(
        self, user_id: str, max_tokens: int = 500
    ) -> Optional[str]:
        """Retrieve a summarized version of the long-term conversation history for a thread.

        Args:
            user_id: The ID of the conversation thread.
            max_tokens: The maximum number of tokens for the summary.

        Returns:
            A summarized string of the conversation history, or None if no history.
        """
        pass

    @abstractmethod
    async def clear_conversation_memory(self, user_id: str) -> bool:
        """Clear memory for a specific user.

        Args:
            user_id: User ID to clear

        Returns:
            True if successful, False otherwise
        """
        pass

    @abstractmethod
    async def store_long_term_memory(
        self, user_id: str, content: str, metadata: Optional[Dict[str, Any]] = None
    ) -> bool:
        """Store a memory in long-term storage.

        Args:
            user_id: The ID of the conversation thread.
            content: The content to store as memory.
            metadata: Optional metadata for the memory.

        Returns:
            True if successful, False otherwise.
        """
        pass
