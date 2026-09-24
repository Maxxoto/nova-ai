"""Unit tests for the shared tool loop (application layer, no subprocess).

Guards the behaviours the desktop contract depends on: thinking-model
reasoning passthrough, the exhausted-budget wrap-up, and tool progress.
"""

from __future__ import annotations

import asyncio
import json
from typing import Any

from app.application.services.tool_loop import WRAP_UP_INSTRUCTION, run_tool_loop


class FakeRegistry:
    def __init__(self) -> None:
        self.calls: list[tuple[str, Any]] = []

    def get_definitions(self) -> list[dict[str, Any]]:
        return [
            {
                "type": "function",
                "function": {
                    "name": "memory_search",
                    "description": "search",
                    "parameters": {"type": "object", "properties": {}},
                },
            }
        ]

    async def execute(self, name: str, arguments: Any) -> str:
        self.calls.append((name, arguments))
        return json.dumps({"ok": True})


def _tool_call(name: str, arguments: dict[str, Any], call_id: str = "c1") -> Any:
    call = type("ToolCall", (), {})()
    call.id = call_id
    call.function = type("Function", (), {})()
    call.function.name = name
    call.function.arguments = json.dumps(arguments)
    return call


class FakeClient:
    """Replays a list of turns and records every request it receives."""

    def __init__(self, turns: list[dict[str, Any]]) -> None:
        self._turns = turns
        self.seen: list[list[dict[str, Any]]] = []

    async def chat_completion(
        self, messages: list[dict[str, Any]], tools: Any = None, **kwargs: Any
    ) -> dict[str, Any]:
        self.seen.append([dict(message) for message in messages])
        index = min(len(self.seen) - 1, len(self._turns) - 1)
        return self._turns[index]


def test_reasoning_content_is_echoed_on_tool_turns() -> None:
    """DeepSeek-style thinking models reject an assistant turn without it."""
    client = FakeClient(
        [
            {
                "response": None,
                "tool_calls": [_tool_call("memory_search", {"query": "x"})],
                "reasoning_content": "the user wants saved notes",
            },
            {"response": "final answer", "tool_calls": [], "reasoning_content": None},
        ]
    )
    registry = FakeRegistry()
    text, used = asyncio.run(
        run_tool_loop(
            llm_client=client,
            tool_registry=registry,
            messages=[{"role": "user", "content": "hi"}],
            max_iterations=3,
        )
    )

    assert text == "final answer"
    assert used == ["memory_search"]
    assert registry.calls == [("memory_search", {"query": "x"})]

    second_request = client.seen[1]
    assistant_turns = [m for m in second_request if m.get("role") == "assistant"]
    assert assistant_turns, "assistant turn must be replayed to the provider"
    assert assistant_turns[-1].get("reasoning_content") == "the user wants saved notes"
    tool_turns = [m for m in second_request if m.get("role") == "tool"]
    assert tool_turns and "ok" in tool_turns[-1]["content"]


def test_exhausted_budget_forces_a_wrap_up_turn() -> None:
    tool_turn = {
        "response": None,
        "tool_calls": [_tool_call("memory_search", {"query": "again"})],
        "reasoning_content": None,
    }
    client = FakeClient([tool_turn, tool_turn, tool_turn, {"response": "wrapped up", "tool_calls": []}])
    registry = FakeRegistry()

    text, used = asyncio.run(
        run_tool_loop(
            llm_client=client,
            tool_registry=registry,
            messages=[{"role": "user", "content": "keep searching"}],
            max_iterations=3,
            wrap_up_on_exhaustion=True,
        )
    )

    assert text == "wrapped up"
    assert len(used) == 3, "the hard budget must stop tool execution at three"
    final_request = client.seen[-1]
    assert any(
        WRAP_UP_INSTRUCTION in str(message.get("content"))
        for message in final_request
        if message.get("role") == "system"
    ), "the wrap-up instruction must precede the final turn"


def test_progress_fires_per_tool_and_never_breaks_the_loop() -> None:
    seen: list[str] = []

    async def progress(tool_name: str) -> None:
        seen.append(tool_name)
        if tool_name == "memory_search":
            raise RuntimeError("progress sink exploded")

    client = FakeClient(
        [
            {
                "response": None,
                "tool_calls": [_tool_call("memory_search", {"query": "x"})],
                "reasoning_content": None,
            },
            {"response": "done", "tool_calls": []},
        ]
    )
    text, _ = asyncio.run(
        run_tool_loop(
            llm_client=client,
            tool_registry=FakeRegistry(),
            messages=[{"role": "user", "content": "hi"}],
            max_iterations=3,
            on_progress=progress,
        )
    )

    assert seen == ["memory_search"]
    assert text == "done", "a failing progress sink must not abort the answer"
