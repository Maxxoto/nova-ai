"""Tool Registry for managing and executing tools with Pydantic AI validation."""

import logging
from typing import Any, Optional

from app.infrastructure.tools.base import Tool
from app.infrastructure.tools.results import (
    WebSearchResults,
    WebFetchResult,
    FileReadResult,
    FileWriteResult,
    FileEditResult,
    DirectoryListResult,
    ExecResult,
)


logger = logging.getLogger(__name__)


class ToolRegistry:
    """Registry for managing and executing tools with result validation."""

    def __init__(
        self,
        enable_validation: bool = False,  # Disabled by default - tools return strings
        async_validation: bool = True,
    ):
        """Initialize registry.

        Args:
            enable_validation: Whether to validate tool results (default: False)
                              Note: Tools return strings, not structured data, so
                              validation is disabled by default.
            async_validation: Whether to use async validation
        """
        self._tools: dict[str, Tool] = {}
        self.enable_validation = enable_validation
        self.async_validation = async_validation

        # Map tool names to their result validators
        # Note: These validators expect structured data (dict/Pydantic model),
        # but tools currently return strings. Validation is disabled by default.
        self.result_validators = {
            "web_search": WebSearchResults,
            "read_file": FileReadResult,
            "write_file": FileWriteResult,
            "edit_file": FileEditResult,
            "list_dir": DirectoryListResult,
            "exec": ExecResult,
            "bash": ExecResult,  # Alias for exec
            "web_fetch": WebFetchResult,
        }

    def register(self, tool: Tool) -> None:
        """Register a tool.

        Args:
            tool: Tool instance to register
        """
        self._tools[tool.name] = tool
        logger.debug(f"Registered tool: {tool.name}")

    def get(self, name: str) -> Tool | None:
        """Get a tool by name.

        Args:
            name: Tool name

        Returns:
            Tool instance or None if not found
        """
        return self._tools.get(name)

    def list_tools(self) -> list[str]:
        """List all registered tool names.

        Returns:
            List of tool names
        """
        return list(self._tools.keys())

    def get_definitions(self) -> list[dict[str, Any]]:
        """Get OpenAI function definitions for all tools.

        Returns:
            List of tool schemas in OpenAI format
        """
        return [tool.to_schema() for tool in self._tools.values()]

    async def execute(
        self,
        name: str,
        arguments: dict[str, Any],
    ) -> str:
        """Execute a tool by name with optional result validation.

        Args:
            name: Tool name
            arguments: Tool arguments

        Returns:
            Tool execution result as string (validated or raw)
        """
        tool = self._tools.get(name)
        if not tool:
            error_msg = f"Tool not found: {name}"
            logger.error(error_msg)
            return error_msg

        # Validate parameters
        errors = tool.validate_params(arguments)
        if errors:
            error_msg = f"Parameter validation failed for {name}: {'; '.join(errors)}"
            logger.error(error_msg)
            return error_msg

        # Execute tool
        try:
            logger.info(f"Executing tool: {name} with args: {arguments}")
            raw_result = await tool.execute(**arguments)

            # Validate result if validator available and validation is enabled
            if self.enable_validation and name in self.result_validators:
                if self.async_validation:
                    from app.application.services.llm_validator import (
                        LLMResponseValidator,
                    )

                    validator = LLMResponseValidator(
                        None
                    )  # No LLM needed for tool results
                    try:
                        logger.debug(f"[Tool Registry] Validating result for {name}")

                        # Try to parse raw_result as structured data
                        try:
                            validated = await validator.validate_tool_result_async(
                                name,
                                raw_result,
                                self.result_validators[name],
                            )
                            # Return as JSON string
                            result_json = validated.model_dump_json(indent=2)
                            logger.info(
                                f"Tool {name} executed and validated successfully"
                            )
                            return result_json
                        except Exception as validation_error:
                            # Validation failed, but tool executed successfully
                            logger.warning(
                                f"Result validation failed for {name}: {validation_error}. Using raw result."
                            )
                            logger.info(
                                f"Tool {name} executed successfully (raw result)"
                            )
                            return raw_result

                    except Exception as e:
                        # Validator error, fall back to raw result
                        logger.warning(
                            f"Validator error for {name}: {e}. Using raw result."
                        )
                        logger.info(f"Tool {name} executed successfully")
                        return raw_result

            logger.info(f"Tool {name} executed successfully")
            return raw_result

        except Exception as e:
            error_msg = f"Error executing tool {name}: {str(e)}"
            logger.error(error_msg)
            return error_msg
