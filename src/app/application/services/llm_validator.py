"""LLM response validation using Pydantic AI."""

import logging
from typing import TypeVar, Type, Optional, Any
from pydantic import BaseModel, ValidationError
import json

from app.adapters.config import settings
from app.adapters.llm_providers.litellm_adapter import LiteLLMAdapter


logger = logging.getLogger(__name__)


T = TypeVar("T", bound=BaseModel)


class LLMResponseValidator:
    """Validates and structures LLM responses using Pydantic."""

    def __init__(
        self,
        llm_client: Optional[LiteLLMAdapter] = None,
        max_retries: Optional[int] = None,
        debug: Optional[bool] = None,
    ):
        """Initialize validator.

        Args:
            llm_client: LLM client to use for validation
            max_retries: Maximum retry attempts (defaults to PYDANTIC_AI_MAX_RETRIES)
            debug: Enable debug logging (defaults to PYDANTIC_AI_VALIDATION_DEBUG)
        """
        self.llm_client = llm_client
        self.max_retries = max_retries or settings.pydantic_ai_max_retries
        self.debug = (
            debug if debug is not None else settings.pydantic_ai_validation_debug
        )
        self.async_only = settings.pydantic_ai_async_only

        if self.debug:
            logger.info("[Pydantic AI] Validator initialized")
            logger.info(f"[Pydantic AI] Max retries: {self.max_retries}")
            logger.info(f"[Pydantic AI] Async only: {self.async_only}")
            logger.info(f"[Pydantic AI] Debug mode: {self.debug}")

    async def validate_tool_result_async(
        self,
        tool_name: str,
        raw_result: str,
        result_model: Type[T],
    ) -> T:
        """Validate tool execution result asynchronously.

        Args:
            tool_name: Name of tool that was executed
            raw_result: Raw string result from tool
            result_model: Expected Pydantic model

        Returns:
            Validated result model

        Raises:
            ValueError: If validation fails after max retries
        """
        for attempt in range(self.max_retries):
            try:
                if self.debug:
                    logger.debug(
                        f"[Pydantic AI] Validating {tool_name} result (attempt {attempt + 1}/{self.max_retries})"
                    )

                # Try to parse as JSON first
                try:
                    parsed_data = json.loads(raw_result)
                    validated = result_model(**parsed_data)
                except json.JSONDecodeError:
                    # Not JSON, try direct validation
                    validated = result_model.model_validate(raw_result)

                if self.debug:
                    logger.debug(
                        f"[Pydantic AI] ✓ Validation successful for {tool_name}"
                    )
                    logger.debug(
                        f"[Pydantic AI] Validated data: {validated.model_dump()}"
                    )

                return validated

            except (ValidationError, ValueError, TypeError) as e:
                if attempt < self.max_retries - 1:
                    if self.debug:
                        logger.debug(
                            f"[Pydantic AI] ✗ Validation failed (attempt {attempt + 1}): {e}"
                        )
                    continue
                else:
                    error_msg = f"Validation failed for {tool_name} after {self.max_retries} attempts: {str(e)}"
                    logger.error(f"[Pydantic AI] {error_msg}")
                    raise ValueError(error_msg)

        # Should not reach here, but just in case
        raise ValueError(f"Validation failed for {tool_name}")

    async def validate_structured_response_async(
        self,
        prompt: str,
        response_model: Type[T],
        context: Optional[str] = None,
    ) -> T:
        """Get and validate structured response from LLM (async).

        Args:
            prompt: Prompt to send to LLM
            response_model: Pydantic model to validate against
            context: Optional context/instructions

        Returns:
            Validated Pydantic model instance

        Raises:
            ValueError: If validation fails or LLM is not available
        """
        if not self.llm_client:
            raise ValueError(
                "LLM client not available for structured response validation"
            )

        for attempt in range(self.max_retries):
            try:
                if self.debug:
                    logger.debug(
                        f"[Pydantic AI] Requesting structured response (attempt {attempt + 1}/{self.max_retries})"
                    )

                # Build messages
                messages = []
                if context:
                    messages.append({"role": "system", "content": context})
                messages.append({"role": "user", "content": prompt})

                # Get response from LLM
                response = await self.llm_client.chat_completion(
                    messages=messages,
                    streaming=False,
                )

                # Parse response
                response_text = response.get("response", "")

                if self.debug:
                    logger.debug(
                        f"[Pydantic AI] LLM response received (length: {len(response_text)} chars)"
                    )

                # Try to parse as JSON
                try:
                    parsed = json.loads(response_text)
                    validated = response_model(**parsed)
                except json.JSONDecodeError:
                    # Try direct validation
                    validated = response_model.model_validate_json(response_text)

                if self.debug:
                    logger.debug(
                        f"[Pydantic AI] ✓ Structured response validation successful"
                    )
                    logger.debug(
                        f"[Pydantic AI] Validated model: {validated.model_dump()}"
                    )

                return validated

            except (ValidationError, ValueError, TypeError) as e:
                if attempt < self.max_retries - 1:
                    if self.debug:
                        logger.debug(
                            f"[Pydantic AI] Structured response validation failed (attempt {attempt + 1}): {e}"
                        )
                    continue
                else:
                    error_msg = f"Structured response validation failed after {self.max_retries} attempts: {str(e)}"
                    logger.error(f"[Pydantic AI] {error_msg}")
                    raise ValueError(error_msg)

        raise ValueError("Failed to get validated structured response")
