"""Transport-agnostic tool-calling loop.

Extracted from `AgentLoop` so non-bus consumers (the desktop sidecar) can run
the same loop without a message bus or a workspace-bound service. RFC-0007
§4.1: evolve the existing loop/registry/LLM seams rather than fork them.
"""

from __future__ import annotations

import json
import logging
from typing import Any, Awaitable, Callable, Dict, List, Optional, Tuple

logger = logging.getLogger(__name__)

ProgressCallback = Callable[[str], Awaitable[None]]


def _parse_arguments(raw: Any) -> Dict[str, Any]:
    """Tool arguments arrive as a JSON string on the OpenAI wire format; accept
    either that or an already-decoded mapping."""
    if isinstance(raw, dict):
        return raw
    if isinstance(raw, str):
        text = raw.strip()
        if not text:
            return {}
        try:
            decoded = json.loads(text)
        except json.JSONDecodeError:
            return {}
        return decoded if isinstance(decoded, dict) else {}
    return {}

WRAP_UP_INSTRUCTION = (
    "Tool budget exhausted — answer now from what you already have."
)


async def run_tool_loop(
    *,
    llm_client: Any,
    tool_registry: Any,
    messages: List[Dict[str, Any]],
    max_iterations: int = 10,
    on_progress: Optional[ProgressCallback] = None,
    wrap_up_on_exhaustion: bool = False,
) -> Tuple[Optional[str], List[str]]:
    """Run the tool-calling loop and return (final_content, tools_used).

    `on_progress` fires with each executed tool name (the desktop sidecar maps
    it to `agent.tool_step`). With `wrap_up_on_exhaustion`, an exhausted budget
    gets one final tools-disabled turn asking the model to answer from what it
    has, instead of the canned apology.
    """
    iteration = 0
    tools_used: List[str] = []
    final_content: Optional[str] = None

    while iteration < max_iterations:
        iteration += 1

        tool_definitions = tool_registry.get_definitions()

        response = await llm_client.chat_completion(
            messages=messages,
            tools=tool_definitions if tool_definitions else None,
            streaming=False,
        )

        tool_calls = response.get("tool_calls")
        if not tool_calls:
            final_content = response.get("response", "")
            logger.info(f"Agent loop completed in {iteration} iterations")
            break

        logger.info(
            f"LLM requested {len(tool_calls)} tool calls (iteration {iteration})"
        )

        tool_call_dicts = []
        for tc in tool_calls:
            if hasattr(tc, "function"):
                tool_call_dicts.append(
                    {
                        "id": getattr(tc, "id", f"call_{len(tools_used)}"),
                        "type": "function",
                        "function": {
                            "name": tc.function.name,
                            "arguments": json.dumps(tc.function.arguments),
                        },
                    }
                )
                tools_used.append(tc.function.name)
            elif isinstance(tc, dict):
                tool_call_dicts.append(tc)

        assistant_turn: Dict[str, Any] = {
            "role": "assistant",
            "content": response.get("response", ""),
            "tool_calls": tool_call_dicts,
        }
        # Thinking models require their own reasoning echoed back verbatim.
        if response.get("reasoning_content"):
            assistant_turn["reasoning_content"] = response["reasoning_content"]
        messages.append(assistant_turn)

        for tc in tool_calls:
            tool_name = None
            tool_args: Any = {}
            tool_id = f"call_{len(tools_used)}"

            if hasattr(tc, "function"):
                tool_name = tc.function.name
                tool_args = _parse_arguments(tc.function.arguments)
                tool_id = getattr(tc, "id", tool_id)
            elif isinstance(tc, dict):
                func_info = tc.get("function", {})
                tool_name = func_info.get("name", "")
                tool_args = _parse_arguments(func_info.get("arguments"))
                tool_id = tc.get("id", tool_id)

            if not tool_name:
                logger.warning(f"Skipping tool call with no name: {tc}")
                continue

            if on_progress is not None:
                try:
                    await on_progress(tool_name)
                except Exception as exc:  # progress must never break the loop
                    logger.warning(f"on_progress failed for {tool_name}: {exc}")

            logger.info(f"Executing tool: {tool_name}({tool_args})")

            try:
                result = await tool_registry.execute(tool_name, tool_args)
                tool_result = str(result)
            except Exception as e:
                tool_result = f"Error executing {tool_name}: {str(e)}"
                logger.error(tool_result)

            messages.append(
                {
                    "role": "tool",
                    "tool_call_id": tool_id,
                    "name": tool_name,
                    "content": tool_result,
                }
            )

    if final_content is None:
        if wrap_up_on_exhaustion:
            messages.append({"role": "system", "content": WRAP_UP_INSTRUCTION})
            wrap_up = await llm_client.chat_completion(
                messages=messages, tools=None, streaming=False
            )
            final_content = wrap_up.get("response", "")
        else:
            logger.warning(
                f"Agent loop exceeded max iterations ({max_iterations})"
            )
            final_content = (
                "I apologize, but I needed too many iterations to complete this task."
            )

    return final_content, tools_used
