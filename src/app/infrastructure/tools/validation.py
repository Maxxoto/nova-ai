"""Pydantic-based validation for tool parameters."""

from typing import Any, Type, TypeVar, Optional
from pydantic import BaseModel, ValidationError
import logging


logger = logging.getLogger(__name__)


T = TypeVar("T", bound=BaseModel)


class ToolParameterValidator:
    """Validator using Pydantic models for parameter validation."""

    @staticmethod
    def validate(
        params: dict[str, Any], model_class: Type[T]
    ) -> tuple[Optional[T], list[str]]:
        """Validate parameters against a Pydantic model.

        Args:
            params: Parameters to validate
            model_class: Pydantic BaseModel class

        Returns:
            Tuple of (validated_model or None, list of error messages)
        """
        try:
            validated = model_class(**params)
            return validated, []
        except ValidationError as e:
            errors = []
            for error in e.errors():
                # Build readable error path
                loc = ".".join(str(x) for x in error["loc"] if x != "__root__")
                if not loc:
                    loc = "parameters"

                # Get human-readable message
                msg = error["msg"]
                if error["type"] == "missing":
                    msg = f"required field '{loc}' is missing"
                elif error["type"] == "greater_than_equal":
                    msg = (
                        f"must be greater than or equal to {error['ctx'].get('ge', '')}"
                    )
                elif error["type"] == "less_than_equal":
                    msg = f"must be less than or equal to {error['ctx'].get('le', '')}"

                errors.append(f"{loc}: {msg}")

            logger.debug(f"Validation errors: {errors}")
            return None, errors

    @staticmethod
    def model_to_json_schema(model_class: Type[BaseModel]) -> dict[str, Any]:
        """Convert Pydantic model to JSON Schema format.

        Args:
            model_class: Pydantic BaseModel class

        Returns:
            JSON Schema dictionary compatible with OpenAI function calling
        """
        schema = model_class.model_json_schema()

        # Extract properties and required fields
        properties = schema.get("properties", {})
        required = schema.get("required", [])

        # Convert Pydantic Field defaults to JSON Schema format
        for prop_name, prop_schema in properties.items():
            # Move default to top level for JSON Schema compatibility
            if "default" in prop_schema and prop_schema["default"] is not None:
                # Keep it, it's already in the right format
                pass

        return {
            "type": "object",
            "properties": properties,
            "required": required,
        }

    @staticmethod
    def model_to_openai_schema(
        model_class: Type[BaseModel], name: str, description: str
    ) -> dict[str, Any]:
        """Convert Pydantic model to OpenAI function schema format.

        Args:
            model_class: Pydantic BaseModel class
            name: Tool name
            description: Tool description

        Returns:
            OpenAI function schema dictionary
        """
        return {
            "type": "function",
            "function": {
                "name": name,
                "description": description,
                "parameters": ToolParameterValidator.model_to_json_schema(model_class),
            },
        }
