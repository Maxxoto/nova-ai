"""Sidecar server: dispatches JSON-RPC requests from the shell (M0 stub).

M0 scope (plans/m0-build-plan.md W5): the transport, envelopes, supervision
surface, and abort semantics are real; ``session.ask`` streams a canned
answer so the shell can exercise the full panel path before the agent loop
is wired in M1.
"""

from __future__ import annotations

import asyncio
import json
import os
import sys
import time
import uuid
from typing import Any, TextIO

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
_UPSTREAM_TIMEOUT_S = float(os.environ.get("RUOXI_UPSTREAM_TIMEOUT_S", "5.0"))


class SidecarServer:
    """JSON-RPC peer over stdio: serves the shell's requests and issues
    upstream requests (Rust-backed tools) back over the same transport."""

    def __init__(self, reader: TextIO, writer: TextIO) -> None:
        self._reader = reader
        self._writer = writer
        self._write_lock = asyncio.Lock()
        self._ask_task: asyncio.Task[None] | None = None
        self._abort: asyncio.Event = asyncio.Event()
        self._busy = False
        self._pending: dict[str, asyncio.Future[dict[str, Any]]] = {}
        self._upstream_seq = 0

    async def _send(self, message: RpcResponse | RpcNotification) -> None:
        line = message.model_dump_json(exclude_none=True)
        async with self._write_lock:
            self._writer.write(line + "\n")
            self._writer.flush()

    async def _send_raw(self, payload: dict[str, Any]) -> None:
        async with self._write_lock:
            self._writer.write(json.dumps(payload) + "\n")
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
        try:
            raw = json.loads(line)
        except Exception:
            return
        if not isinstance(raw, dict):
            return
        if "method" in raw:
            await self._handle_request(raw)
        elif "id" in raw:
            self._resolve_upstream(raw)

    async def _handle_request(self, raw: dict[str, Any]) -> None:
        try:
            request = RpcRequest.model_validate(raw)
        except Exception:
            return
        try:
            await self._dispatch(request)
        except Exception as exc:
            await self._send(
                RpcResponse(
                    id=request.id,
                    error=RpcError(
                        code=int(ErrorCode.INTERNAL), message=f"internal: {exc}"
                    ),
                )
            )

    def _resolve_upstream(self, raw: dict[str, Any]) -> None:
        rid = str(raw.get("id"))
        future = self._pending.get(rid)
        if future is not None and not future.done():
            future.set_result(raw)

    async def _request_upstream(
        self, method: str, params: dict[str, Any]
    ) -> dict[str, Any] | None:
        self._upstream_seq += 1
        rid = f"py-{self._upstream_seq}"
        future: asyncio.Future[dict[str, Any]] = (
            asyncio.get_running_loop().create_future()
        )
        self._pending[rid] = future
        await self._send_raw(
            {"jsonrpc": "2.0", "id": rid, "method": method, "params": params}
        )
        try:
            return await asyncio.wait_for(future, _UPSTREAM_TIMEOUT_S)
        except asyncio.TimeoutError:
            return None
        finally:
            self._pending.pop(rid, None)

    async def _lookup_capture(self, capture_id: str) -> str:
        response = await self._request_upstream(
            "capture.lookup", {"id": capture_id}
        )
        result = response.get("result") if response else None
        if not isinstance(result, dict):
            return f"[capture {capture_id} unavailable]"
        app = result.get("app") or "unknown app"
        title = result.get("window_title") or "untitled"
        return f"{app} — {title}"

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
        elif method == "config.test":
            result = await asyncio.to_thread(llm_config_test, request.params if isinstance(request.params, dict) else None)
            await self._send(RpcResponse(id=request.id, result=result))
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
            if self._abort.is_set():
                await self._send_aborted(request_id)
                return
            subjects = [await self._lookup_capture(c) for c in params.capture_ids]
            about = ", ".join(subjects) if subjects else "no captures"
            canned = (
                f"[M0 stub] Heard: {params.transcript!r} about {about}. "
                "The real agent loop arrives in M1."
            )
            for word in canned.split(" "):
                if self._abort.is_set():
                    await self._send_aborted(request_id)
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

    async def _send_aborted(self, request_id: int) -> None:
        await self._send(
            RpcResponse(id=request_id, error=_err(ErrorCode.ABORTED, "aborted by user"))
        )


def llm_config_test(params: dict | None = None) -> dict[str, object]:
    """One-token LiteLLM call against the BYOK config — explicit params
    (typed in the UI) override the env-injected values; no built-in keys."""
    params = params or {}
    base_url = str(params.get("base_url") or "").strip() or os.environ.get("RUOXI_LLM_BASE_URL", "").strip()
    api_key = str(params.get("api_key") or "").strip() or os.environ.get("RUOXI_LLM_API_KEY", "").strip()
    model = str(params.get("model") or "").strip() or os.environ.get("RUOXI_LLM_MODEL", "").strip()
    if not base_url or not api_key:
        return {"ok": False, "message": "LLM not configured — set base URL and API key in Settings"}
    try:
        import litellm

        response = litellm.completion(
            model=f"openai/{model or 'gpt-4o-mini'}",
            api_base=base_url,
            api_key=api_key,
            messages=[{"role": "user", "content": "ping"}],
            max_tokens=1,
            timeout=15,
        )
        reply = (response.choices[0].message.content or "")[:40]
        return {"ok": True, "model": model, "reply": reply}
    except Exception as exc:  # noqa: BLE001
        return {"ok": False, "model": model, "message": str(exc)[:300]}


def _err(code: ErrorCode, message: str) -> RpcError:
    return RpcError(code=int(code), message=message)


async def serve(reader: TextIO | None = None, writer: TextIO | None = None) -> None:
    """Entry point: serve stdio until EOF (see ``python -m app.interfaces.sidecar``)."""
    server = SidecarServer(
        reader if reader is not None else sys.stdin,
        writer if writer is not None else sys.stdout,
    )
    await server.serve()
