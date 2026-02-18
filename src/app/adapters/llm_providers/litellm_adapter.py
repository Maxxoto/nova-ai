"""LiteLLM Multi-Provider Adapter using litellm directly.

Supports multiple LLM providers through LiteLLM:
- Groq (groq/openai/gpt-oss-20b)
- OpenAI (openai/gpt-4)
- Anthropic (anthropic/claude-3-opus-20240229)
- Zhipu (zhipu/glm-4)
- And 100+ more providers...

Usage:
    Set LITE_LLM_API_KEY environment variable with your API key.
    The provider is determined by the model prefix.
"""

import logging
from typing import AsyncGenerator, Dict, Any, List, Optional

from litellm import acompletion, completion

from app.domain.ports.llm_client_port import LLMClientPort
from app.adapters.config import settings


logger = logging.getLogger(__name__)


class LiteLLMAdapter(LLMClientPort):
    """LiteLLM adapter using litellm directly (no LangChain wrapper).

    This adapter uses litellm.completion() directly, supporting 100+ providers.
    Just set LITE_LLM_API_KEY and use model format "provider/model-name".

    Examples:
        - groq/openai/gpt-oss-20b
        - openai/gpt-4
        - anthropic/claude-3-opus-20240229
        - zhipu/glm-4
    """

    def __init__(
        self,
        model: Optional[str] = None,
        api_key: Optional[str] = None,
        temperature: float = 0.7,
        max_tokens: int = 4096,
    ):
        """Initialize LiteLLM adapter.

        Args:
            model: Model identifier in format "provider/model-name"
            api_key: API key (defaults to LITE_LLM_API_KEY env var)
            temperature: Sampling temperature
            max_tokens: Maximum tokens to generate
        """
        self.model = model or settings.lite_llm_model
        self.api_key = api_key or settings.lite_llm_api_key
        self.temperature = temperature
        self.max_tokens = max_tokens

        logger.info(f"LiteLLM adapter initialized with model: {self.model}")

    def _get_api_key(self) -> Optional[str]:
        """Get API key, returning None if empty string."""
        if self.api_key and self.api_key.strip():
            return self.api_key
        return None

    def get_llm_client(self):
        """Get the underlying LLM client.

        This method is not used in the direct LiteLLM implementation.
        Use invoke(), chat_completion(), or stream_chat_completion() instead.

        Raises:
            NotImplementedError: This adapter uses litellm directly
        """
        raise NotImplementedError(
            "LiteLLMAdapter uses litellm directly. "
            "Use invoke(), chat_completion(), or stream_chat_completion() instead."
        )

    async def stream_chat_completion(
        self, messages: List[Dict[str, Any]], thread_id: Optional[str] = None
    ) -> AsyncGenerator[Dict[str, Any], None]:
        """Stream chat completion using LiteLLM.

        Args:
            messages: List of chat messages with 'role' and 'content'
            thread_id: Optional thread ID for conversation context

        Yields:
            Dictionary containing content and thread_id
        """
        try:
            response = await acompletion(
                model=self.model,
                messages=messages,
                api_key=self._get_api_key(),
                temperature=self.temperature,
                max_tokens=self.max_tokens,
                stream=True,
            )

            async for chunk in response:
                delta = chunk.choices[0].delta
                if delta and delta.content:
                    yield {
                        "content": delta.content,
                        "thread_id": thread_id or "default",
                    }

        except Exception as e:
            logger.error(f"Error in stream_chat_completion: {e}")
            yield {"content": f"Error: {str(e)}", "thread_id": thread_id or "default"}

    async def chat_completion(
        self,
        messages: List[Dict[str, Any]],
        thread_id: Optional[str] = None,
        model: Optional[str] = None,
        streaming: Optional[bool] = False,
        tools: Optional[List[Dict[str, Any]]] = None,
    ) -> Dict[str, Any]:
        """Non-streaming chat completion using LiteLLM.

        Args:
            messages: List of chat messages with 'role' and 'content'
            thread_id: Optional thread ID for conversation context
            model: Optional model name to override default
            streaming: Whether to stream (default False)
            tools: Optional list of tool definitions for function calling

        Returns:
            Dictionary containing response and thread_id
        """
        import time

        use_model = model or self.model

        try:
            # Build completion kwargs
            kwargs = {
                "model": use_model,
                "messages": messages,
                "api_key": self._get_api_key(),
                "temperature": self.temperature,
                "max_tokens": self.max_tokens,
                "stream": False,
            }

            # Add tools if provided - filter Groq-incompatible fields
            if tools:
                cleaned_tools = []
                for tool in tools:
                    # Clean up tool definitions for Groq
                    cleaned_tool = {}
                    if "type" in tool:
                        cleaned_tool["type"] = tool["type"]
                    if "function" in tool:
                        func = {}
                        if "name" in tool["function"]:
                            func["name"] = tool["function"]["name"]
                        if "description" in tool["function"]:
                            func["description"] = tool["function"]["description"]
                        # Handle parameters - remove Groq-incompatible fields
                        if "parameters" in tool["function"]:
                            params = tool["function"]["parameters"]
                            func["parameters"] = self._clean_groq_parameters(params)
                        cleaned_tool["function"] = func
                    cleaned_tools.append(cleaned_tool)
                kwargs["tools"] = cleaned_tools
                kwargs["tool_choice"] = "auto"

            response = await acompletion(**kwargs)

            message = response.choices[0].message

            # Check for tool calls
            if hasattr(message, "tool_calls") and message.tool_calls:
                return {
                    "response": None,
                    "tool_calls": message.tool_calls,
                    "thread_id": thread_id or "default",
                    "memory_used": False,
                }

            content = message.content

            return {
                "response": content,
                "thread_id": thread_id or "default",
                "memory_used": False,
            }

        except Exception as e:
            logger.error(f"Error in chat_completion: {e}")
            return {
                "response": f"Error: {str(e)}",
                "thread_id": thread_id or "default",
                "memory_used": False,
            }

    def _clean_groq_parameters(self, params: dict) -> dict:
        """Clean parameter schema for Groq compatibility."""
        cleaned = {}
        if "type" in params:
            cleaned["type"] = params["type"]

        if "properties" in params:
            cleaned["properties"] = {}
            for prop_name, prop_def in params["properties"].items():
                cleaned_prop = {}
                if "description" in prop_def:
                    cleaned_prop["description"] = prop_def["description"]

                prop_type = prop_def.get("type", "string")
                if prop_type == "string":
                    cleaned_prop["type"] = "string"
                    if "default" in prop_def:
                        cleaned_prop["default"] = prop_def["default"]
                elif prop_type == "integer":
                    cleaned_prop["type"] = "integer"
                    if "default" in prop_def:
                        cleaned_prop["default"] = prop_def["default"]
                elif prop_type == "number":
                    cleaned_prop["type"] = "number"
                    if "default" in prop_def:
                        cleaned_prop["default"] = prop_def["default"]
                elif prop_type == "boolean":
                    cleaned_prop["type"] = "boolean"
                    if "default" in prop_def:
                        cleaned_prop["default"] = prop_def["default"]

                cleaned["properties"][prop_name] = cleaned_prop

        if "required" in params and params["required"]:
            cleaned["required"] = params["required"]

        return cleaned

    def invoke(self, messages: List[Any]) -> Any:
        """Invoke the LLM with messages (synchronous).

        Args:
            messages: List of messages to send to LLM

        Returns:
            Simple response object with content attribute
        """
        # Convert to simple dict format if needed
        simple_messages = []
        for msg in messages:
            if hasattr(msg, "role") and hasattr(msg, "content"):
                simple_messages.append({"role": msg.role, "content": msg.content})
            elif isinstance(msg, dict):
                simple_messages.append(msg)
            else:
                simple_messages.append({"role": "user", "content": str(msg)})

        response = completion(
            model=self.model,
            messages=simple_messages,
            api_key=self._get_api_key(),
            temperature=self.temperature,
            max_tokens=self.max_tokens,
        )

        # Extract content from response
        content = response.choices[0].message.content
        return type("Response", (), {"content": content})()
