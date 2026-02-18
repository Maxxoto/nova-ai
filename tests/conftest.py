"""Test configuration with FastAPI dependency overrides."""

import pytest
from fastapi.testclient import TestClient

from app.interfaces.api.fastapi_app import create_app
from app.dependencies import get_chat_service


@pytest.fixture
def app():
    """Create FastAPI app for testing."""
    return create_app()


@pytest.fixture
def client(app):
    """Create test client."""
    return TestClient(app)


@pytest.fixture
def mock_chat_service():
    """Create mock chat service for testing."""

    # This is a placeholder - implement actual mock as needed
    class MockChatService:
        async def stream_chat_completion(self, messages, thread_id=None):
            yield {"content": "Test response", "thread_id": thread_id or "test-thread"}

        async def chat_completion(self, messages, thread_id=None):
            return {
                "response": "Test response",
                "thread_id": thread_id or "test-thread",
                "memory_used": True,
                "workflow_completed": True,
            }

        async def get_conversation_history(self, user_id):
            return []

        async def clear_conversation_memory(self, user_id):
            return True

    return MockChatService()


@pytest.fixture(autouse=True)
def override_dependencies(app, mock_chat_service):
    """Override dependencies for all tests."""

    async def _get_mock_chat_service():
        return mock_chat_service

    app.dependency_overrides[get_chat_service] = _get_mock_chat_service
    yield
    # Clean up overrides after test
    app.dependency_overrides.clear()
