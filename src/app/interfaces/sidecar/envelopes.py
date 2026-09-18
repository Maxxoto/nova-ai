"""JSON-RPC envelopes for the shell <-> brain IPC (RFC-0002 §4.6).

Transport: newline-delimited JSON over stdio. Versioned via ``proto_ver`` so
the contract can evolve without breaking the shell (M0: version 1).
"""

from __future__ import annotations

from enum import IntEnum
from typing import Literal

from pydantic import BaseModel, Field

PROTO_VER = 1
JSONRPC = "2.0"


class RpcError(BaseModel):
    """JSON-RPC error object."""

    code: int
    message: str
    data: dict[str, str] | None = None


class RpcRequest(BaseModel):
    """Inbound request from the shell."""

    jsonrpc: Literal["2.0"]
    id: int
    method: str
    params: dict[str, object] = Field(default_factory=dict)


class RpcResponse(BaseModel):
    """Outbound response to a request."""

    jsonrpc: Literal["2.0"] = "2.0"
    id: int
    result: dict[str, object] | None = None
    error: RpcError | None = None


class RpcNotification(BaseModel):
    """Outbound notification (no id, fire-and-forget)."""

    jsonrpc: Literal["2.0"] = "2.0"
    method: str
    params: dict[str, object] = Field(default_factory=dict)


class ErrorCode(IntEnum):
    """Server-defined error codes (JSON-RPC reserved range -32000..-32099)."""

    ABORTED = -32000
    BUSY = -32001
    METHOD_NOT_FOUND = -32601
    INVALID_PARAMS = -32602
    INTERNAL = -32603


class AskParams(BaseModel):
    """Params for ``session.ask``."""

    transcript: str = Field(min_length=1)
    capture_ids: list[str] = Field(default_factory=list)
    mode: Literal["ask", "save"] = "ask"
