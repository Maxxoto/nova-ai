"""Tool Registry for managing and executing tools."""

import logging
from typing import Any

from app.infrastructure.tools.base import Tool


logger = logging.getLogger(__name__)


class ToolRegistry:
    """Registry for managing and executing tools."""

    def __init__(self):
        """Initialize empty tool registry."""
        self._tools: dict[str, Tool] = {}

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

    async def execute(self, name: str, arguments: dict[str, Any]) -> str:
        """Execute a tool by name.

        Args:
            name: Tool name
            arguments: Tool arguments

        Returns:
            Tool execution result as string

        Raises:
            ValueError: If tool not found
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

        try:
            logger.info(f"Executing tool: {name} with args: {arguments}")
            result = await tool.execute(**arguments)
            logger.info(f"Tool {name} executed successfully")
            return result
        except Exception as e:
            error_msg = f"Error executing tool {name}: {str(e)}"
            logger.error(error_msg)
            return error_msg
