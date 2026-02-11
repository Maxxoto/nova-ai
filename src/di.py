"""Dependency Injection for hexagonal architecture with singleton pattern."""

import logging
import os
from core.ports.llm_client_port import LLMClientPort
from core.ports.memory_port import MemoryPort
from core.services.chat_service import ChatService
from adapters.llm_providers.groq import GroqLLMAdapter
from adapters.memory.file_memory_adapter import FileMemoryAdapter
from adapters.memory.file_checkpointer import FileCheckpointer

logger = logging.getLogger(__name__)


class DependencyContainer:
    """Singleton dependency container for managing all service instances."""

    _instance = None
    _initialized = False

    def __new__(cls):
        if cls._instance is None:
            cls._instance = super().__new__(cls)
        return cls._instance

    def __init__(self):
        if not self._initialized:
            self._llm_client = None
            self._memory = None
            self._chat_service = None
            self._thread_memory_store = None
            self._initialized = True

    def get_llm_client(self) -> LLMClientPort:
        """Get LLM client instance."""
        if self._llm_client is None:
            self._llm_client = GroqLLMAdapter()

        return self._llm_client

    def get_thread_memory(self):
        """Get file-based checkpointer instance."""
        if self._thread_memory_store is None:
            memory_data_dir = os.getenv("MEMORY_DATA_DIR", "./data/memory")
            self._thread_memory_store = FileCheckpointer(data_dir=memory_data_dir)
        return self._thread_memory_store

    async def get_memory(self) -> MemoryPort:
        """Get file-based memory instance."""
        if self._memory is None:
            memory_data_dir = os.getenv("MEMORY_DATA_DIR", "./data/memory")
            embedding_model = os.getenv("EMBEDDING_MODEL", "all-MiniLM-L6-v2")
            
            self._memory = FileMemoryAdapter(
                llm_client=self.get_llm_client(),
                data_dir=memory_data_dir,
                embedding_model=embedding_model
            )
        return self._memory

    async def get_chat_service(self) -> ChatService:
        """Get chat service instance."""
        if self._chat_service is None:
            memory = await self.get_memory()
            self._chat_service = ChatService(
                llm_client=self.get_llm_client(),
                memory=memory,
                thread_memory=self.get_thread_memory(),
                longterm_memory=None,  # File-based memory handles this internally
            )

        return self._chat_service


# Global singleton instance
_container = DependencyContainer()


# Public API functions for backward compatibility
def get_llm_client() -> LLMClientPort:
    """Get LLM client instance."""
    return _container.get_llm_client()


async def get_memory() -> MemoryPort:
    """Get memory instance."""
    return await _container.get_memory()


async def get_chat_service() -> ChatService:
    """Get chat service instance."""
    return await _container.get_chat_service()


def get_thread_memory():
    """Get file-based checkpointer instance."""
    return _container.get_thread_memory()

