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
ENV = {
    **os.environ,
    "PYTHONPATH": str(REPO_ROOT / "src"),
    "RUOXI_UPSTREAM_TIMEOUT_S": "0.5",
    # Deterministic streamed answers with no network: the agent loop's mock path.
    "RUOXI_LLM_MOCK": "1",
}


class SidecarProcess:
    def __init__(self, env_overrides: dict[str, str] | None = None) -> None:
        env = {**ENV, **(env_overrides or {})}
        self.proc = subprocess.Popen(
            [sys.executable, "-m", "app.interfaces.sidecar"],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            env=env,
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


def test_ask_with_unresolvable_capture_still_completes() -> None:
    sidecar = SidecarProcess()
    try:
        sidecar.request(1, "ping")
        sidecar.send(
            {
                "jsonrpc": "2.0",
                "id": 2,
                "method": "session.ask",
                "params": {"transcript": "explain this", "capture_ids": ["cap_missing"]},
            }
        )
        final: dict[str, object] | None = None
        for _ in range(200):
            msg = sidecar.read_msg()
            if msg.get("id") == 2:
                final = msg
                break
        assert final is not None, "ask must answer even when a capture cannot resolve"
        result = final["result"]
        assert isinstance(result, dict)
        assert "[mock]" in str(result["answer"])
        assert sidecar.request(3, "ping")["result"]["pong"] is True
    finally:
        sidecar.close()


def test_ask_offline_gate_refuses_without_llm_call() -> None:
    sidecar = SidecarProcess(
        {"RUOXI_LLM_MOCK": "", "RUOXI_OFFLINE": "true"}
    )
    try:
        sidecar.request(1, "ping")
        sidecar.send(
            {
                "jsonrpc": "2.0",
                "id": 2,
                "method": "session.ask",
                "params": {"transcript": "hello", "capture_ids": []},
            }
        )
        final: dict[str, object] | None = None
        for _ in range(200):
            msg = sidecar.read_msg()
            if msg.get("id") == 2:
                final = msg
                break
        assert final is not None
        result = final["result"]
        assert isinstance(result, dict)
        assert "Offline mode" in str(result["answer"])
    finally:
        sidecar.close()


def test_ask_unconfigured_reports_setup_hint() -> None:
    sidecar = SidecarProcess(
        {
            "RUOXI_LLM_MOCK": "",
            "RUOXI_OFFLINE": "false",
            "RUOXI_LLM_BASE_URL": "",
            "RUOXI_LLM_API_KEY": "",
        }
    )
    try:
        sidecar.request(1, "ping")
        sidecar.send(
            {
                "jsonrpc": "2.0",
                "id": 2,
                "method": "session.ask",
                "params": {"transcript": "hello", "capture_ids": []},
            }
        )
        final: dict[str, object] | None = None
        for _ in range(200):
            msg = sidecar.read_msg()
            if msg.get("id") == 2:
                final = msg
                break
        assert final is not None
        result = final["result"]
        assert isinstance(result, dict)
        assert "isn't configured" in str(result["answer"])
    finally:
        sidecar.close()


def test_config_test_reports_unconfigured_as_not_ok() -> None:
    sidecar = SidecarProcess()
    try:
        response = sidecar.request(1, "config.test", {})
        result = response["result"]
        assert isinstance(result, dict)
        assert result["ok"] is False
        assert "not configured" in str(result["message"])
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


def test_memory_block_lists_hits_and_citation_rules() -> None:
    from app.interfaces.sidecar.server import SidecarServer

    hits = [
        {
            "id": "mem_01H",
            "kind": "semantic",
            "snippet": "Krebs cycle summary",
            "source_refs": ["cap_9"],
        },
        {"id": "mem_02H", "kind": "procedural", "snippet": "prefers simple first", "source_refs": []},
    ]
    block = SidecarServer._memory_block(hits)
    assert "[mem_01H] (semantic)" in block
    assert "(captures: cap_9)" in block
    assert "cite it as [mem_id]" in block
    assert SidecarServer._memory_block([]) == ""


def test_strip_bogus_citations_keeps_valid_ids() -> None:
    from app.interfaces.sidecar.server import SidecarServer

    hits = [{"id": "mem_01H", "kind": "semantic", "snippet": "s", "source_refs": []}]
    answer = "Krebs makes ATP [mem_01H] per [cap_C1], see [mem_FAKE]."
    cleaned, bogus = SidecarServer._strip_bogus_citations(answer, hits, ["cap_C1"])
    assert bogus == 1
    assert "[mem_01H]" in cleaned
    assert "[cap_C1]" in cleaned
    assert "[mem_FAKE]" not in cleaned


def test_memory_search_flows_upstream_on_real_path() -> None:
    sidecar = SidecarProcess(
        {
            "RUOXI_LLM_MOCK": "",
            "RUOXI_LLM_BASE_URL": "http://127.0.0.1:9",
            "RUOXI_LLM_API_KEY": "sk-test",
        }
    )
    try:
        sidecar.request(1, "ping")
        sidecar.send(
            {
                "jsonrpc": "2.0",
                "id": 2,
                "method": "session.ask",
                "params": {"transcript": "what did I save about krebs", "capture_ids": []},
            }
        )
        search_seen = False
        final: dict[str, object] | None = None
        for _ in range(200):
            msg = sidecar.read_msg()
            if msg.get("method") == "memory.search":
                assert msg["params"]["query"] == "what did I save about krebs"
                sidecar.send(
                    {
                        "jsonrpc": "2.0",
                        "id": msg["id"],
                        "result": [
                            {
                                "id": "mem_01H",
                                "kind": "semantic",
                                "snippet": "krebs cycle notes",
                                "score": 0.9,
                                "source_refs": [],
                            }
                        ],
                    }
                )
                search_seen = True
            if msg.get("id") == 2:
                final = msg
                break
        assert search_seen, "session.ask must query memory.search upstream on the real path"
        assert final is not None
    finally:
        sidecar.close()
