"""LLM clients for the sidecar.

The loop itself is transport-agnostic (`application.services.tool_loop`); this
module provides the sidecar's concrete clients:

- `ScriptedLLMClient` — deterministic responses for tests, no network.
- `RoutingLLMClient`  — text vs vision model per request (RFC-0007 §4.4).
- `ResilientLLMClient` — retries down a capability ladder when a provider
  rejects a request shape (tools+image → image → text), preserving the
  failure in the log instead of failing the ask.
"""

from __future__ import annotations

import json
import logging
import os
from pathlib import Path
from typing import Any, Dict, List, Optional

logger = logging.getLogger("ruoxi.py")


def _has_image(messages: List[Dict[str, Any]]) -> bool:
    for message in messages:
        content = message.get("content")
        if isinstance(content, list):
            if any(
                isinstance(part, dict) and part.get("type") == "image_url"
                for part in content
            ):
                return True
    return False


def strip_images(messages: List[Dict[str, Any]]) -> List[Dict[str, Any]]:
    """Replace image parts with a placeholder so text-only models still answer."""
    stripped: List[Dict[str, Any]] = []
    for message in messages:
        content = message.get("content")
        if not isinstance(content, list):
            stripped.append(message)
            continue
        parts: List[Any] = []
        for part in content:
            if isinstance(part, dict) and part.get("type") == "image_url":
                parts.append({"type": "text", "text": "[image omitted]"})
            else:
                parts.append(part)
        stripped.append({**message, "content": parts})
    return stripped


class ScriptedLLMClient:
    """Serves responses from a JSON script — deterministic, offline tests.

    Each entry may carry `content` and a compact `tool_calls` list
    (`{id, name, arguments}`); `{"raise": "..."}` simulates a provider error.
    """

    def __init__(self, script_path: Path):
        self._entries: List[Dict[str, Any]] = json.loads(
            Path(script_path).read_text(encoding="utf-8")
        )
        self._cursor = 0

    async def chat_completion(
        self,
        messages: List[Dict[str, Any]],
        thread_id: Optional[str] = None,
        model: Optional[str] = None,
        streaming: Optional[bool] = False,
        tools: Optional[List[Dict[str, Any]]] = None,
    ) -> Dict[str, Any]:
        entry = self._entries[min(self._cursor, len(self._entries) - 1)]
        self._cursor += 1
        logger.info("llm.call | scripted turn tools=%s", bool(tools))
        if entry.get("raise"):
            raise RuntimeError(str(entry["raise"]))
        calls = [
            {
                "id": call.get("id", f"call_{i}"),
                "type": "function",
                "function": {
                    "name": call.get("name", ""),
                    "arguments": call.get("arguments", "{}"),
                },
            }
            for i, call in enumerate(entry.get("tool_calls") or [])
        ]
        return {
            "response": entry.get("content", ""),
            "tool_calls": calls,
            "reasoning_content": entry.get("reasoning_content"),
        }


class RoutingLLMClient:
    """Picks the vision model when a request carries images, else the text model."""

    def __init__(self, text_client: Any, vision_client: Any):
        self._text = text_client
        self._vision = vision_client

    async def chat_completion(self, messages: List[Dict[str, Any]], **kwargs: Any) -> Dict[str, Any]:
        client = self._vision if _has_image(messages) else self._text
        return await client.chat_completion(messages=messages, **kwargs)


class ResilientLLMClient:
    """Retries a rejected request with fewer capabilities before giving up."""

    def __init__(self, inner: Any):
        self._inner = inner

    async def chat_completion(
        self,
        messages: List[Dict[str, Any]],
        tools: Optional[List[Dict[str, Any]]] = None,
        **kwargs: Any,
    ) -> Dict[str, Any]:
        image_present = _has_image(messages)
        attempts: List[tuple[bool, bool]] = []
        if tools and image_present:
            attempts = [(True, True), (False, True), (False, False)]
        elif tools:
            attempts = [(True, False), (False, False)]
        elif image_present:
            attempts = [(False, True), (False, False)]
        else:
            attempts = [(False, False)]

        last_exc: Optional[BaseException] = None
        for attempt_tools, keep_images in attempts:
            attempt_messages = messages if keep_images else strip_images(messages)
            try:
                outcome = await self._inner.chat_completion(
                    messages=attempt_messages,
                    tools=tools if attempt_tools else None,
                    **kwargs,
                )
                if (attempt_tools, keep_images) != attempts[0]:
                    logger.warning(
                        "llm.call degraded to tools=%s image=%s after provider rejection",
                        attempt_tools,
                        keep_images,
                    )
                return outcome
            except Exception as exc:
                last_exc = exc
                logger.warning(
                    "llm.call tools=%s image=%s failed: %s",
                    attempt_tools,
                    keep_images,
                    exc,
                )
        assert last_exc is not None
        raise last_exc


def build_byok_clients(model: str, vision_model: str, api_key: str, api_base: str) -> RoutingLLMClient:
    """Build the BYOK client pair from the shell-injected configuration."""
    import litellm

    # litellm prints error banners to stdout — our JSON-RPC wire — so one stray
    # line desyncs the protocol (seen with connection errors).
    litellm.suppress_debug_info = True

    from app.adapters.llm_providers.litellm_adapter import LiteLLMAdapter

    def qualify(name: str) -> str:
        # litellm needs a provider prefix to honour a custom api_base.
        return name if "/" in name else f"openai/{name}"

    def make(name: str) -> LiteLLMAdapter:
        return LiteLLMAdapter(
            model=qualify(name),
            api_key=api_key,
            api_base=api_base or None,
            temperature=float(os.environ.get("RUOXI_LLM_TEMPERATURE", "0.7")),
        )

    return RoutingLLMClient(make(model), make(vision_model))
