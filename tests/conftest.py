"""Test configuration for Nova Agent CLI."""

import pytest
from pathlib import Path
import tempfile
import os


@pytest.fixture
def temp_workspace():
    """Create a temporary workspace directory for testing."""
    with tempfile.TemporaryDirectory() as tmpdir:
        workspace = Path(tmpdir) / ".nova"
        workspace.mkdir(parents=True, exist_ok=True)
        (workspace / "workspace").mkdir(exist_ok=True)
        (workspace / "memory").mkdir(exist_ok=True)
        (workspace / "sessions").mkdir(exist_ok=True)
        (workspace / "bootstrap").mkdir(exist_ok=True)
        yield workspace


@pytest.fixture
def mock_env(temp_workspace, monkeypatch):
    """Set up environment variables for testing."""
    monkeypatch.setenv("NOVA_WORKSPACE", str(temp_workspace))
    monkeypatch.setenv("LITE_LLM_API_KEY", "test-api-key")
    monkeypatch.setenv("LITE_LLM_MODEL", "groq/test-model")
    yield
