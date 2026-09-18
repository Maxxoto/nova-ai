"""W5 M0 tests: brain sidecar JSON-RPC over stdio.

Spawns the real server process and exercises the contract the shell will
rely on: ping/health, session.ask token streaming, abort mid-stream,
unknown-method error, and graceful EOF shutdown.
"""

from __future__ import annotations

import json
import os
import subprocess
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
ENV = {**os.environ, "PYTHONPATH": str(REPO_ROOT / "src")}


class SidecarProcess:
    def __init__(self) -> None:
        self.proc = subprocess.Popen(
            [sys.executable, "-m", "app.interfaces.sidecar"],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            env=ENV,
            cwd=REPO_ROOT,
        )

    def send(self, payload: dict[str, object]) -> None:
        assert self.proc.stdin is not None
        self.proc.stdin.write(json.dumps(payload) + "\n")
        self.proc.stdin.flush()

    def read_msg(self) -> dict[str, object]:
        assert self.proc.stdout is not None
        line = self.proc.stdout.readline()
        assert line, "sidecar closed stdout unexpectedly"
        return json.loads(line)

    def request(self, id: int, method: str, params: dict[str, object] | None = None) -> dict[str, object]:
        self.send({"jsonrpc": "2.0", "id": id, "method": method, "params": params or {}})
        return self.read_msg()

    def close(self) -> None:
        if self.proc.stdin is not None:
            self.proc.stdin.close()
        self.proc.wait(timeout=10)


def test_ping_and_health() -> None:
    sidecar = SidecarProcess()
    try:
        pong = sidecar.request(1, "ping")
        assert pong["id"] == 1
        result = pong["result"]
        assert isinstance(result, dict)
        assert result["pong"] is True
        assert result["proto_ver"] == 1

        health = sidecar.request(2, "health")
        assert health["result"]["status"] == "ok"
    finally:
        sidecar.close()


def test_session_ask_streams_tokens_then_result() -> None:
    sidecar = SidecarProcess()
    try:
        sidecar.request(1, "ping")
        sidecar.send(
            {
                "jsonrpc": "2.0",
                "id": 2,
                "method": "session.ask",
                "params": {"transcript": "explain this", "capture_ids": ["cap_1"]},
            }
        )
        tokens: list[str] = []
        final: dict[str, object] | None = None
        for _ in range(200):
            msg = sidecar.read_msg()
            if msg.get("method") == "agent.token":
                params = msg["params"]
                assert isinstance(params, dict)
                delta = params["delta"]
                assert isinstance(delta, str)
                tokens.append(delta)
            elif msg.get("id") == 2:
                final = msg
                break
        assert final is not None, "no final response for session.ask"
        assert len(tokens) > 1, "expected streamed tokens before the result"
        result = final["result"]
        assert isinstance(result, dict)
        assert str(result["answer_id"]).startswith("ans_")
        assert "explain this" in str(result["answer"])
        assert result["aborted"] is False
    finally:
        sidecar.close()


def test_abort_mid_stream_returns_aborted_error() -> None:
    sidecar = SidecarProcess()
    try:
        sidecar.send(
            {
                "jsonrpc": "2.0",
                "id": 1,
                "method": "session.ask",
                "params": {"transcript": "a longer question to have time to abort"},
            }
        )
        saw_token = False
        for _ in range(200):
            msg = sidecar.read_msg()
            if msg.get("method") == "agent.token":
                saw_token = True
                break
        assert saw_token, "expected at least one token before aborting"
        sidecar.request(2, "session.abort")  # result {"aborting": true}
        final: dict[str, object] | None = None
        for _ in range(200):
            msg = sidecar.read_msg()
            if msg.get("id") == 1:
                final = msg
                break
        assert final is not None, "no final response for the aborted ask"
        error = final["error"]
        assert isinstance(error, dict)
        assert error["code"] == -32000  # ErrorCode.ABORTED
    finally:
        sidecar.close()


def test_unknown_method_is_reported_not_fatal() -> None:
    sidecar = SidecarProcess()
    try:
        resp = sidecar.request(1, "no.such.method")
        error = resp["error"]
        assert isinstance(error, dict)
        assert error["code"] == -32601  # METHOD_NOT_FOUND
        pong = sidecar.request(2, "ping")  # server still alive
        assert pong["result"]["pong"] is True
    finally:
        sidecar.close()


def test_eof_shuts_down_cleanly() -> None:
    sidecar = SidecarProcess()
    sidecar.request(1, "ping")
    sidecar.close()
    assert sidecar.proc.returncode == 0
