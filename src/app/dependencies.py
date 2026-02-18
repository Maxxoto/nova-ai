"""FastAPI Dependency Injection Container."""

import logging
import os
from typing import AsyncGenerator

from fastapi import Depends
from langgraph.store.base import BaseStore
from langgraph.checkpoint.memory import InMemorySaver
from langchain.embeddings import init_embeddings
from psycopg_pool import AsyncConnectionPool

from app.domain.ports.llm_client_port import LLMClientPort
from app.domain.ports.memory_port import MemoryPort
from app.adapters.llm_providers.groq import GroqLLMAdapter
from app.adapters.memory.in_memory_adapter import InMemoryMemoryAdapter
from app.application.usecases.chat_service import ChatService


logger = logging.getLogger(__name__)

# Cache for expensive async resources
_llm_client: GroqLLMAdapter | None = None
_longterm_store: BaseStore | None = None
_thread_memory: InMemorySaver | None = None


def get_llm_client() -> LLMClientPort:
    """Get or create LLM client (cached)."""
    global _llm_client
    if _llm_client is None:
        _llm_client = GroqLLMAdapter()
    return _llm_client


async def get_longterm_memory_store() -> BaseStore:
    """Get or create long-term memory store (cached)."""
    global _longterm_store
    if _longterm_store is None:
        embeddings = init_embeddings("ollama:nomic-embed-text")
        conn_pool = AsyncConnectionPool(
            conninfo=os.getenv("POSTGRES_URL"),
            min_size=1,
            max_size=10,
            open=False,
        )
        await conn_pool.open()

        from langgraph.store.postgres import AsyncPostgresStore

        _longterm_store = AsyncPostgresStore(
            conn=conn_pool,
            index={
                "dims": 768,
                "embed": embeddings,
                "fields": ["content"],
                "distance_type": "cosine",
            },
        )
        await _longterm_store.setup()

    return _longterm_store


def get_thread_memory() -> InMemorySaver:
    """Get or create thread memory (cached)."""
    global _thread_memory
    if _thread_memory is None:
        _thread_memory = InMemorySaver()
    return _thread_memory


async def get_memory(
    llm: LLMClientPort = Depends(get_llm_client),
    longterm_store: BaseStore = Depends(get_longterm_memory_store),
    thread_memory: InMemorySaver = Depends(get_thread_memory),
) -> MemoryPort:
    """Get memory adapter with all dependencies."""
    return InMemoryMemoryAdapter(
        llm_client=llm,
        thread_memory_saver=thread_memory,
        longterm_memory_store=longterm_store,
    )


async def get_chat_service(
    llm: LLMClientPort = Depends(get_llm_client),
    memory: MemoryPort = Depends(get_memory),
    thread_memory: InMemorySaver = Depends(get_thread_memory),
    longterm_memory: BaseStore = Depends(get_longterm_memory_store),
) -> ChatService:
    """Get chat service with all dependencies."""
    return ChatService(
        llm_client=llm,
        memory=memory,
        thread_memory=thread_memory,
        longterm_memory=longterm_memory,
    )


async def init_chat_service() -> ChatService:
    """Initialize chat service for CLI/non-FastAPI contexts.

    This function manually resolves dependencies without using FastAPI's Depends().
    """
    llm = get_llm_client()
    thread_memory = get_thread_memory()
    longterm_memory = await get_longterm_memory_store()
    memory = InMemoryMemoryAdapter(
        llm_client=llm,
        thread_memory_saver=thread_memory,
        longterm_memory_store=longterm_memory,
    )
    return ChatService(
        llm_client=llm,
        memory=memory,
        thread_memory=thread_memory,
        longterm_memory=longterm_memory,
    )
