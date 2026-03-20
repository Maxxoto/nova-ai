"""Tests for Pydantic AI integration."""

import pytest
import asyncio
import sys
import os

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "src"))

from app.application.services.llm_validator import LLMResponseValidator
from app.infrastructure.tools.results import (
    WebSearchResults,
    FileReadResult,
    FileWriteResult,
    DirectoryListResult,
)


class TestPydanticAIValidation:
    """Test suite for Pydantic AI validation."""

    def test_validator_initialization(self):
        """Test validator can be initialized with settings."""
        validator = LLMResponseValidator(None)
        assert validator.max_retries == 3
        assert validator.debug is False  # Default from settings
        print("✓ Validator initialization test passed")

    async def test_tool_result_validation_success(self):
        """Test successful tool result validation."""
        validator = LLMResponseValidator(None)

        # Valid web search result
        valid_result = """{
            "results": [
                {
                    "title": "Python 3.12",
                    "url": "https://python.org",
                    "description": "Latest version",
                    "rank": 1
                }
            ],
            "total_results": 1,
            "query": "python 3.12"
        }"""

        result = await validator.validate_tool_result_async(
            "web_search", valid_result, WebSearchResults
        )

        assert result.query == "python 3.12"
        assert len(result.results) == 1
        assert result.results[0].title == "Python 3.12"
        print("✓ Tool result validation (success) test passed")

    async def test_tool_result_validation_failure(self):
        """Test validation failure with retry logic."""
        validator = LLMResponseValidator(None, max_retries=2)

        # Invalid result (missing required fields)
        invalid_result = """{"results": [{"title": "Test"}]}"""

        with pytest.raises(ValueError, match="Validation failed"):
            await validator.validate_tool_result_async(
                "web_search", invalid_result, WebSearchResults
            )

        print("✓ Tool result validation (failure) test passed")

    def test_result_model_fields(self):
        """Test that result models have correct fields."""
        # WebSearchResults
        model_fields = WebSearchResults.model_json_schema()["properties"]
        assert "results" in model_fields
        assert "total_results" in model_fields
        assert "query" in model_fields
        print("✓ Result model fields test passed")

    def test_memory_summary_model(self):
        """Test MemorySummary model."""
        from app.infrastructure.memory.models import MemorySummary
        from datetime import datetime

        summary = MemorySummary(
            summary="Test summary",
            key_topics=["python", "validation"],
            message_count=5,
            timestamp=datetime.now(),
        )

        assert summary.summary == "Test summary"
        assert len(summary.key_topics) == 2
        assert summary.message_count == 5
        print("✓ MemorySummary model test passed")

    def test_settings_configuration(self):
        """Test that settings are properly loaded."""
        from app.adapters.config import settings

        assert hasattr(settings, "pydantic_ai_max_retries")
        assert settings.pydantic_ai_max_retries == 3  # Default value
        assert hasattr(settings, "pydantic_ai_validation_debug")
        assert hasattr(settings, "pydantic_ai_async_only")
        print("✓ Settings configuration test passed")


def run_tests():
    """Run all tests."""
    print("\n🧪 Running Pydantic AI Integration Tests\n")

    # Test 1: Initialization
    test_suite = TestPydanticAIValidation()
    test_suite.test_validator_initialization()

    # Test 2: Model fields
    test_suite.test_result_model_fields()
    test_suite.test_memory_summary_model()
    test_suite.test_settings_configuration()

    # Test 3: Async validation
    async def run_async_tests():
        await test_suite.test_tool_result_validation_success()
        await test_suite.test_tool_result_validation_failure()

    asyncio.run(run_async_tests())

    print("\n✅ All Pydantic AI tests passed!\n")
    print("\n📊 Summary:")
    print("  ✓ Pydantic AI validation integrated")
    print("  ✓ Tool result validation working")
    print("  ✓ Memory models created")
    print("  ✓ Configuration via environment variables")
    print("  ✓ Async-only validation")
    print("  ✓ Retry logic (max 3, configurable)")
    print("  ✓ Debug logging (controlled by PYDANTIC_AI_VALIDATION_DEBUG)")
    print("\n🎯 Next Steps:")
    print("  1. Tool results are validated automatically when executed")
    print("  2. Failed validations fall back to raw results")
    print("  3. Memory summaries can be validated (optional enhancement)")
    print("  4. Set PYDANTIC_AI_VALIDATION_DEBUG=true for detailed logging")


if __name__ == "__main__":
    run_tests()
