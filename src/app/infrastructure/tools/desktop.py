"""Desktop tools: the shell's own data, reached over the sidecar reverse-RPC.

Each tool proxies one router method implemented in Rust (`memory.search`,
`memory.lookup`, `capture.lookup`, `timeline.query`) through an injected
async callable, so the tool layer stays transport-free and testable.
"""

from __future__ import annotations

import json
from typing import Any, Awaitable, Callable, Optional

from pydantic import BaseModel, Field

from app.infrastructure.tools.base import Tool

Upstream = Callable[[str, dict[str, Any]], Awaitable[Any]]

_MAX_RESULT_CHARS = 4000


class MemorySearchParams(BaseModel):
    query: str = Field(description="What to look for in the user's memory")
    types: Optional[list[str]] = Field(
        default=None,
        description="Restrict to memory types: episodic, semantic, procedural",
    )
    k: Optional[int] = Field(default=None, description="Maximum hits (1-10)")


class MemoryLookupParams(BaseModel):
    id: str = Field(description="Memory id, e.g. mem_01H...")


class CaptureLookupParams(BaseModel):
    id: str = Field(description="Capture id, e.g. cap_01H...")


class TimelineQueryParams(BaseModel):
    day: Optional[str] = Field(default=None, description="Filter by day, YYYY-MM-DD")
    app: Optional[str] = Field(default=None, description="Filter by app name")
    limit: Optional[int] = Field(default=None, description="Maximum rows (default 50)")


class _DesktopTool(Tool):
    """Shared plumbing: proxy a router method and return a short string."""

    def __init__(self, upstream: Upstream):
        self._upstream = upstream

    async def _call(self, method: str, params: dict[str, Any]) -> str:
        try:
            result = await self._upstream(method, params)
        except Exception as exc:
            return json.dumps({"error": str(exc)})
        payload = json.dumps(result, ensure_ascii=False, default=str)
        return payload[:_MAX_RESULT_CHARS]


class MemorySearchTool(_DesktopTool):
    @property
    def name(self) -> str:
        return "memory_search"

    @property
    def description(self) -> str:
        return (
            "Search the user's local memory (saved notes, facts, past Q/A). "
            "Cite results as [mem_id] in the answer."
        )

    @property
    def param_model(self):
        return MemorySearchParams

    async def execute(
        self, query: str, types: Optional[list[str]] = None, k: Optional[int] = None
    ) -> str:
        params: dict[str, Any] = {"query": query}
        if types:
            params["types"] = types
        if k:
            params["k"] = k
        return await self._call("memory.search", params)


class MemoryLookupTool(_DesktopTool):
    @property
    def name(self) -> str:
        return "memory_lookup"

    @property
    def description(self) -> str:
        return "Fetch one memory entry in full by its id."

    @property
    def param_model(self):
        return MemoryLookupParams

    async def execute(self, id: str) -> str:
        return await self._call("memory.lookup", {"id": id})


class CaptureLookupTool(_DesktopTool):
    @property
    def name(self) -> str:
        return "capture_lookup"

    @property
    def description(self) -> str:
        return (
            "Fetch a screenshot's metadata and file path by capture id. "
            "Cite it as [cap_id]."
        )

    @property
    def param_model(self):
        return CaptureLookupParams

    async def execute(self, id: str) -> str:
        return await self._call("capture.lookup", {"id": id})


class TimelineQueryTool(_DesktopTool):
    @property
    def name(self) -> str:
        return "timeline_query"

    @property
    def description(self) -> str:
        return "List the user's recent screen captures, newest first."

    @property
    def param_model(self):
        return TimelineQueryParams

    async def execute(
        self, day: Optional[str] = None, app: Optional[str] = None, limit: Optional[int] = None
    ) -> str:
        params: dict[str, Any] = {}
        if day:
            params["day"] = day
        if app:
            params["app"] = app
        if limit:
            params["limit"] = limit
        return await self._call("timeline.query", params)


def desktop_tools(upstream: Upstream) -> list[Tool]:
    """The shell-backed tool set, bound to one upstream caller."""
    return [
        MemorySearchTool(upstream),
        MemoryLookupTool(upstream),
        CaptureLookupTool(upstream),
        TimelineQueryTool(upstream),
    ]
