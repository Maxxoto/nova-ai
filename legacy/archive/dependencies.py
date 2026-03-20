"""FastAPI Dependency Injection Container."""

import logging
import os
from pathlib import Path

from fastapi import Depends
from langgraph.store.base import BaseStore
from langgraph.checkpoint.memory import InMemorySaver
from langchain.embeddings import init_embeddings
from psycopg_pool import AsyncConnectionPool

from app.domain.ports.llm_client_port import LLMClientPort
from app.domain.ports.memory_port import MemoryPort
from app.adapters.llm_providers.litellm_adapter import LiteLLMAdapter
from app.adapters.memory.in_memory_adapter import InMemoryMemoryAdapter
from app.application.services.enhanced_orchestrator import EnhancedLangGraphOrchestrator
from app.adapters.config import settings


logger = logging.getLogger(__name__)

# Cache for expensive async resources
_llm_client: LiteLLMAdapter | None = None
_longterm_store: BaseStore | None = None
_thread_memory: InMemorySaver | None = None
_orchestrator: EnhancedLangGraphOrchestrator | None = None


def get_llm_client() -> LLMClientPort:
    """Get or create LLM client (cached)."""
    global _llm_client
    if _llm_client is None:
        _llm_client = LiteLLMAdapter(
            model=settings.lite_llm_model,
            api_key=settings.lite_llm_api_key,
            temperature=settings.litellm_temperature,
            max_tokens=settings.litellm_max_tokens,
        )
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


async def get_orchestrator(
    llm: LLMClientPort = Depends(get_llm_client),
) -> EnhancedLangGraphOrchestrator:
    """Get enhanced orchestrator with all dependencies."""
    global _orchestrator
    if _orchestrator is None:
        workspace = Path(os.getenv("NOVA_WORKSPACE", Path.home() / ".nova"))
        _orchestrator = EnhancedLangGraphOrchestrator(
            llm_client=llm,
            workspace=workspace,
        )
    return _orchestrator
