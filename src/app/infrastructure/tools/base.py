"""Base class for agent tools using Pydantic validation."""

from abc import ABC, abstractmethod
from typing import Any, Type, Optional
from pydantic import BaseModel
from app.infrastructure.tools.validation import ToolParameterValidator


class Tool(ABC):
    """
    Abstract base class for agent tools.

    Tools are capabilities that the agent can use to interact with
    the environment, such as reading files, executing commands, etc.
    """

    @property
    @abstractmethod
    def name(self) -> str:
        """Tool name used in function calls."""
        pass

    @property
    @abstractmethod
    def description(self) -> str:
        """Description of what the tool does."""
        pass

    @property
    def parameters(self) -> dict[str, Any]:
        """JSON Schema for tool parameters. Auto-generated from param_model."""
        if self.param_model:
            return ToolParameterValidator.model_to_json_schema(self.param_model)
        return {"type": "object", "properties": {}, "required": []}

    @property
    def param_model(self) -> Optional[Type[BaseModel]]:
        """Pydantic model for tool parameters. Override in subclasses."""
        return None

    @abstractmethod
    async def execute(self, **kwargs: Any) -> str:
        """
        Execute the tool with given parameters.

        Args:
            **kwargs: Tool-specific parameters.

        Returns:
            String result of the tool execution.
        """
        pass

    def validate_params(self, params: dict[str, Any]) -> list[str]:
        """Validate tool parameters using Pydantic. Returns error list (empty if valid)."""
        if not self.param_model:
            # No validation if no model defined
            return []

        _, errors = ToolParameterValidator.validate(params, self.param_model)
        return errors

    def to_schema(self) -> dict[str, Any]:
        """Convert tool to OpenAI function schema format."""
        return {
            "type": "function",
            "function": {
                "name": self.name,
                "description": self.description,
                "parameters": self.parameters,
            },
        }
