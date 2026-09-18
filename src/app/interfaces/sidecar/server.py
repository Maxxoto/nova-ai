"""Sidecar server: dispatches JSON-RPC requests from the shell (M0 stub).

M0 scope (plans/m0-build-plan.md W5): the transport, envelopes, supervision
surface, and abort semantics are real; ``session.ask`` streams a canned
answer so the shell can exercise the full panel path before the agent loop
is wired in M1.
"""

from __future__ import annotations

import asyncio
import sys
import time
import uuid
from typing import TextIO

from app.interfaces.sidecar.envelopes import (
    PROTO_VER,
    AskParams,
    ErrorCode,
    RpcError,
    RpcNotification,
    RpcRequest,
    RpcResponse,
)

_STARTED_AT = time.monotonic()
_TOKEN_PAUSE_S = 0.02
_MAX_CONCURRENT_ASKS = 1


class SidecarServer:
    """Single-reader JSON-RPC dispatcher over stdio."""

    def __init__(self, reader: TextIO, writer: TextIO) -> None:
        self._reader = reader
        self._writer = writer
        self._write_lock = asyncio.Lock()
        self._ask_task: asyncio.Task[None] | None = None
        self._abort: asyncio.Event = asyncio.Event()
        self._busy = False

    async def _send(self, message: RpcResponse | RpcNotification) -> None:
        line = message.model_dump_json(exclude_none=True)
        async with self._write_lock:
            self._writer.write(line + "\n")
            self._writer.flush()

    async def _notify_token(self, answer_id: str, delta: str) -> None:
        await self._send(
            RpcNotification(
                method="agent.token",
                params={"answer_id": answer_id, "delta": delta},
            )
        )

    async def serve(self) -> None:
        """Read requests until EOF; graceful shutdown on stdin close."""
        while True:
            line = await asyncio.to_thread(self._reader.readline)
            if not line:
                break
            line = line.strip()
            if not line:
                continue
            await self._handle_line(line)
        if self._ask_task is not None:
            self._abort.set()
            await self._ask_task

    async def _handle_line(self, line: str) -> None:
        import json

        try:
            raw = json.loads(line)
            request = RpcRequest.model_validate(raw)
        except Exception:
            # Not parseable as a request: there is no id to answer, skip.
            return
        try:
            await self._dispatch(request)
        except Exception as exc:  # internal handler failure
            await self._send(
                RpcResponse(
                    id=request.id,
                    error=RpcError(
                        code=int(ErrorCode.INTERNAL), message=f"internal: {exc}"
                    ),
                )
            )

    async def _dispatch(self, request: RpcRequest) -> None:
        method = request.method
        if method == "ping":
            await self._send(
                RpcResponse(
                    id=request.id,
                    result={
                        "pong": True,
                        "proto_ver": PROTO_VER,
                        "uptime_s": round(time.monotonic() - _STARTED_AT, 3),
                    },
                )
            )
        elif method == "health":
            await self._send(
                RpcResponse(
                    id=request.id,
                    result={"status": "ok", "proto_ver": PROTO_VER, "stub": True},
                )
            )
        elif method == "session.ask":
            await self._start_ask(request)
        elif method == "session.abort":
            self._abort.set()
            await self._send(
                RpcResponse(id=request.id, result={"aborting": True})
            )
        else:
            await self._send(
                RpcResponse(
                    id=request.id,
                    error=_err(
                        ErrorCode.METHOD_NOT_FOUND, f"unknown method: {method}"
                    )
                )
            )

    async def _start_ask(self, request: RpcRequest) -> None:
        try:
            params = AskParams.model_validate(request.params)
        except Exception as exc:
            await self._send(
                RpcResponse(
                    id=request.id,
                    error=_err(ErrorCode.INVALID_PARAMS, f"bad params: {exc}"),
                )
            )
            return
        if self._busy:
            await self._send(
                RpcResponse(id=request.id, error=_err(ErrorCode.BUSY, "ask in flight"))
            )
            return
        self._abort.clear()
        self._busy = True
        answer_id = f"ans_{uuid.uuid4().hex[:12]}"
        self._ask_task = asyncio.create_task(
            self._run_ask(request.id, answer_id, params)
        )

    async def _run_ask(
        self, request_id: int, answer_id: str, params: AskParams
    ) -> None:
        try:
            canned = (
                f"[M0 stub] Heard: {params.transcript!r} with "
                f"{len(params.capture_ids)} capture(s). The real agent loop "
                "arrives in M1."
            )
            for word in canned.split(" "):
                if self._abort.is_set():
                    await self._send(
                        RpcResponse(
                            id=request_id,
                            error=_err(ErrorCode.ABORTED, "aborted by user"),
                        )
                    )
                    return
                await self._notify_token(answer_id, word + " ")
                await asyncio.sleep(_TOKEN_PAUSE_S)
            await self._send(
                RpcResponse(
                    id=request_id,
                    result={
                        "answer_id": answer_id,
                        "answer": canned,
                        "steps": [],
                        "aborted": False,
                    },
                )
            )
        finally:
            self._busy = False
            self._ask_task = None


def _err(code: ErrorCode, message: str) -> RpcError:
    return RpcError(code=int(code), message=message)


async def serve(reader: TextIO | None = None, writer: TextIO | None = None) -> None:
    """Entry point: serve stdio until EOF (see ``python -m app.interfaces.sidecar``)."""
    server = SidecarServer(
        reader if reader is not None else sys.stdin,
        writer if writer is not None else sys.stdout,
    )
    await server.serve()
