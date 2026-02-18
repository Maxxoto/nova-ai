"""Agent Loop - Nanobot-style simple agent loop with tool calling.

This implements a simplified agent loop that:
1. Consumes messages from the message bus
2. Runs agent iteration loop with LLM + tools
3. Publishes responses back to the bus
"""

import asyncio
import json
import logging
from pathlib import Path
from typing import Optional, Callable, Awaitable, List, Dict, Any

from app.infrastructure.bus.events import InboundMessage, OutboundMessage
from app.infrastructure.bus.queue import MessageBus
from app.domain.ports.llm_client_port import LLMClientPort
from app.infrastructure.tools import ToolRegistry
from app.infrastructure.skills import ContextBuilder
from app.infrastructure.session import SessionManager
from app.infrastructure.memory import MemoryStore


logger = logging.getLogger(__name__)


class AgentLoop:
    """Simple agent loop with tool calling (nanobot-style).

    This uses a simple while loop for tool calling:
    - Build context (memory + skills)
    - Call LLM with tools
    - If tools called -> execute -> add results -> loop
    - If no tools -> return response
    """

    def __init__(
        self,
        bus: MessageBus,
        llm_client: LLMClientPort,
        workspace: Path,
        tool_registry: ToolRegistry,
        max_iterations: int = 10,
        memory_window: int = 50,
    ):
        self.bus = bus
        self.llm_client = llm_client
        self.workspace = workspace
        self.tool_registry = tool_registry
        self.max_iterations = max_iterations
        self.memory_window = memory_window

        self.context_builder = ContextBuilder(workspace)
        self.session_manager = SessionManager(workspace)
        self.memory_store = MemoryStore(workspace)

        self._running = False
        self._task = None

    async def start(self) -> None:
        self._running = True
        self._task = asyncio.create_task(self._run_loop())
        logger.info("AgentLoop started")

    async def stop(self) -> None:
        self._running = False
        if self._task:
            self._task.cancel()
            try:
                await self._task
            except asyncio.CancelledError:
                pass
        logger.info("AgentLoop stopped")

    async def _run_loop(self) -> None:
        while self._running:
            try:
                msg = await asyncio.wait_for(self.bus.consume_inbound(), timeout=1.0)

                response = await self._process_message(msg)
                if response:
                    await self.bus.publish_outbound(response)

            except asyncio.TimeoutError:
                continue
            except asyncio.CancelledError:
                break
            except Exception as e:
                logger.error(f"Error in AgentLoop: {e}", exc_info=True)

    async def _process_message(self, msg: InboundMessage) -> Optional[OutboundMessage]:
        try:
            logger.info(f"Processing message from {msg.channel}:{msg.sender_id}")

            session = self.session_manager.get_or_create(msg.session_key)

            system_prompt = self.context_builder.build_system_prompt(
                include_memory=True,
                include_skills=True,
            )

            messages = [{"role": "system", "content": system_prompt}]

            history = session.get_history(max_messages=self.memory_window)
            for h in history:
                messages.append({"role": h["role"], "content": h["content"]})

            messages.append({"role": "user", "content": msg.content})

            final_content, tools_used = await self._run_agent_loop(messages=messages)

            if not final_content:
                final_content = "I apologize, but I couldn't generate a response."

            session.add_message("user", msg.content)
            session.add_message("assistant", final_content, tools_used=tools_used)
            self.session_manager.save(session)

            if len(session.messages) > self.memory_window:
                asyncio.create_task(self._consolidate_memory(session))

            logger.info(
                f"Response generated: {len(final_content)} chars, tools: {tools_used}"
            )

            return OutboundMessage(
                channel=msg.channel,
                chat_id=msg.chat_id,
                content=final_content,
                metadata=msg.metadata,
            )

        except Exception as e:
            logger.error(f"Error processing message: {e}", exc_info=True)
            return OutboundMessage(
                channel=msg.channel,
                chat_id=msg.chat_id,
                content=f"Sorry, I encountered an error: {str(e)}",
                metadata=msg.metadata,
            )

    async def _run_agent_loop(
        self,
        messages: List[Dict[str, Any]],
        on_progress: Optional[Callable[[str], Awaitable[None]]] = None,
    ) -> tuple[Optional[str], List[str]]:
        iteration = 0
        tools_used: List[str] = []
        final_content = None

        while iteration < self.max_iterations:
            iteration += 1

            tool_definitions = self.tool_registry.get_definitions()

            response = await self.llm_client.chat_completion(
                messages=messages,
                tools=tool_definitions if tool_definitions else None,
                streaming=False,
            )

            tool_calls = response.get("tool_calls")
            if tool_calls:
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

                messages.append(
                    {
                        "role": "assistant",
                        "content": response.get("response", ""),
                        "tool_calls": tool_call_dicts,
                    }
                )

                for tc in tool_calls:
                    tool_name = None
                    tool_args = {}
                    tool_id = f"call_{len(tools_used)}"

                    if hasattr(tc, "function"):
                        tool_name = tc.function.name
                        tool_args = (
                            tc.function.arguments
                            if isinstance(tc.function.arguments, dict)
                            else json.loads(tc.function.arguments)
                        )
                        tool_id = getattr(tc, "id", tool_id)
                    elif isinstance(tc, dict):
                        func_info = tc.get("function", {})
                        tool_name = func_info.get("name", "")
                        tool_args = func_info.get("arguments", {})
                        tool_id = tc.get("id", tool_id)

                    if not tool_name:
                        logger.warning(f"Skipping tool call with no name: {tc}")
                        continue

                    logger.info(f"Executing tool: {tool_name}({tool_args})")

                    try:
                        result = await self.tool_registry.execute(tool_name, tool_args)
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
            else:
                final_content = response.get("response", "")
                logger.info(f"Agent loop completed in {iteration} iterations")
                break

        if iteration >= self.max_iterations:
            logger.warning(
                f"Agent loop exceeded max iterations ({self.max_iterations})"
            )
            final_content = (
                "I apologize, but I needed too many iterations to complete this task."
            )

        return final_content, tools_used

    async def _consolidate_memory(self, session) -> None:
        try:
            keep_count = self.memory_window // 2
            if len(session.messages) <= keep_count:
                return

            old_messages = session.messages[:-keep_count]
            if not old_messages:
                return

            logger.info(f"Memory consolidation: {len(old_messages)} messages")

            lines = []
            for m in old_messages:
                if not m.get("content"):
                    continue
                tools = (
                    f" [tools: {', '.join(m.get('tools_used', []))}]"
                    if m.get("tools_used")
                    else ""
                )
                lines.append(
                    f"[{m.get('timestamp', '?')[:16]}] {m['role'].upper()}{tools}: {m['content']}"
                )

            conversation = "\n".join(lines)
            current_memory = self.memory_store.read_long_term()

            prompt = f"""Consolidate this conversation into memory. Return JSON with:
{{
  "history_entry": "Brief summary for HISTORY.md",
  "memory_update": "Updated MEMORY.md content (add new facts, keep existing)"
}}

Current MEMORY.md:
{current_memory or "(empty)"}

Conversation:
{conversation}"""

            response = await self.llm_client.chat_completion(
                messages=[
                    {
                        "role": "system",
                        "content": "You are a memory consolidation agent. Respond only with valid JSON.",
                    },
                    {"role": "user", "content": prompt},
                ],
                streaming=False,
            )

            result = json.loads(response.get("response", "{}"))

            if entry := result.get("history_entry"):
                self.memory_store.append_history(entry)

            if update := result.get("memory_update"):
                self.memory_store.write_long_term(update)

            session.last_consolidated = len(session.messages) - keep_count
            logger.info("Memory consolidation completed")

        except Exception as e:
            logger.error(f"Memory consolidation failed: {e}")

    @property
    def is_running(self) -> bool:
        return self._running
