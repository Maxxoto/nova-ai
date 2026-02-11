"""Tests for file-based memory adapter."""

import pytest
import tempfile
import shutil
from pathlib import Path
from adapters.memory.file_memory_adapter import FileMemoryAdapter
from adapters.llm_providers.groq import GroqLLMAdapter


@pytest.fixture
def temp_memory_dir():
    """Create a temporary directory for memory storage."""
    temp_dir = tempfile.mkdtemp()
    yield temp_dir
    # Cleanup
    shutil.rmtree(temp_dir)


@pytest.fixture
def memory_adapter(temp_memory_dir):
    """Create a file memory adapter with temp directory."""
    llm_client = GroqLLMAdapter()
    adapter = FileMemoryAdapter(
        llm_client=llm_client,
        data_dir=temp_memory_dir,
        embedding_model="all-MiniLM-L6-v2"
    )
    return adapter


@pytest.mark.asyncio
async def test_store_and_retrieve_long_term_memory(memory_adapter):
    """Test storing and retrieving long-term memories."""
    user_id = "test_user_001"
    content = "The user's favorite color is blue"
    
    # Store memory
    success = await memory_adapter.store_long_term_memory(
        user_id=user_id,
        content=content,
        metadata={"fact_type": "USER_PREFERENCE"}
    )
    
    assert success is True
    
    # Retrieve memory
    memories = await memory_adapter.get_longterm_memory(user_id=user_id)
    
    assert memories is not None
    assert "blue" in memories
    assert "USER_PREFERENCE" in memories


@pytest.mark.asyncio
async def test_semantic_search(memory_adapter):
    """Test semantic search functionality."""
    user_id = "test_user_002"
    
    # Store multiple memories
    await memory_adapter.store_long_term_memory(
        user_id=user_id,
        content="The user loves pizza",
        metadata={"fact_type": "USER_PREFERENCE"}
    )
    
    await memory_adapter.store_long_term_memory(
        user_id=user_id,
        content="The user works as a software engineer",
        metadata={"fact_type": "USER_INFO"}
    )
    
    await memory_adapter.store_long_term_memory(
        user_id=user_id,
        content="The user enjoys hiking on weekends",
        metadata={"fact_type": "USER_PREFERENCE"}
    )
    
    # Search for food-related memories
    results = await memory_adapter.search_longterm_memory(
        user_id=user_id,
        query="What food does the user like?",
        top_k=2
    )
    
    assert results is not None
    assert "pizza" in results


@pytest.mark.asyncio
async def test_conversation_history(memory_adapter):
    """Test conversation history storage and retrieval."""
    thread_id = "test_thread_001"
    user_id = "test_user_003"
    
    # Initially no history
    history = await memory_adapter.get_conversation_history(thread_id)
    assert history is None
    
    # Append messages
    message1 = {"role": "user", "content": "Hello!"}
    message2 = {"role": "assistant", "content": "Hi there!"}
    
    success1 = memory_adapter.append_message(thread_id, message1, user_id)
    success2 = memory_adapter.append_message(thread_id, message2, user_id)
    
    assert success1 is True
    assert success2 is True
    
    # Retrieve history
    history = await memory_adapter.get_conversation_history(thread_id)
    
    assert history is not None
    assert len(history) == 2
    assert history[0]["content"] == "Hello!"
    assert history[1]["content"] == "Hi there!"


@pytest.mark.asyncio
async def test_clear_conversation_memory(memory_adapter):
    """Test clearing/archiving conversation memory."""
    thread_id = "test_thread_002"
    user_id = "test_user_004"
    
    # Add a message
    message = {"role": "user", "content": "Test message"}
    memory_adapter.append_message(thread_id, message, user_id)
    
    # Verify it exists
    history = await memory_adapter.get_conversation_history(thread_id)
    assert history is not None
    
    # Clear memory
    success = await memory_adapter.clear_conversation_memory(thread_id)
    assert success is True
    
    # Verify it's cleared
    history = await memory_adapter.get_conversation_history(thread_id)
    assert history is None


@pytest.mark.asyncio
async def test_rebuild_index(memory_adapter, temp_memory_dir):
    """Test rebuilding memory index from files."""
    user_id = "test_user_005"
    
    # Store some memories
    await memory_adapter.store_long_term_memory(
        user_id=user_id,
        content="Test fact 1",
        metadata={"fact_type": "TEST"}
    )
    
    await memory_adapter.store_long_term_memory(
        user_id=user_id,
        content="Test fact 2",
        metadata={"fact_type": "TEST"}
    )
    
    # Rebuild index
    success = memory_adapter.rebuild_user_index(user_id)
    assert success is True
    
    # Verify facts are still retrievable
    memories = await memory_adapter.get_longterm_memory(user_id=user_id)
    assert memories is not None
    assert "Test fact 1" in memories
    assert "Test fact 2" in memories


@pytest.mark.asyncio
async def test_list_user_threads(memory_adapter):
    """Test listing threads for a user."""
    user_id = "test_user_006"
    
    # Create multiple threads
    thread1 = "thread_001"
    thread2 = "thread_002"
    
    memory_adapter.append_message(thread1, {"role": "user", "content": "Hi"}, user_id)
    memory_adapter.append_message(thread2, {"role": "user", "content": "Hello"}, user_id)
    
    # List threads
    threads = memory_adapter.list_user_threads(user_id)
    
    assert len(threads) == 2
    assert thread1 in threads
    assert thread2 in threads
