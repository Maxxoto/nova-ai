"""Tool execution node for LangGraph workflow."""

import json
import logging
from typing import Dict, Any, List

from app.domain.entities.agent_state import AgentState
from app.infrastructure.tools import ToolRegistry


logger = logging.getLogger(__name__)


class ToolExecutionNode:
    """Node for executing tools based on LLM requests."""

    def __init__(self, tool_registry: ToolRegistry):
        """Initialize tool execution node.

        Args:
            tool_registry: Registry of available tools
        """
        self.tool_registry = tool_registry

    async def execute_node(self, state: AgentState) -> AgentState:
        """Execute tools requested by the LLM.

        Args:
            state: Current agent state with tool_calls (or dict from LangGraph)

        Returns:
            Updated agent state with tool results
        """
        # Handle both AgentState object and dict formats
        if isinstance(state, dict):
            tool_calls = state.get("tool_calls", [])
            messages = state.get("messages", [])
        else:
            tool_calls = state.tool_calls
            messages = state.messages

        if not tool_calls:
            logger.info("No tool calls to execute")
            return state

        tool_results = []

        for tool_call in tool_calls:
            tool_name = tool_call.get("name", "")
            tool_args = tool_call.get("arguments", {})

            logger.info(f"Executing tool: {tool_name} with args: {tool_args}")

            try:
                result = await self.tool_registry.execute(tool_name, tool_args)
                tool_results.append(
                    {
                        "tool": tool_name,
                        "arguments": tool_args,
                        "result": result,
                        "success": True,
                    }
                )
                logger.info(f"Tool {tool_name} executed successfully")

            except Exception as e:
                error_msg = f"Error executing {tool_name}: {str(e)}"
                logger.error(error_msg)
                tool_results.append(
                    {
                        "tool": tool_name,
                        "arguments": tool_args,
                        "result": error_msg,
                        "success": False,
                    }
                )

        # Store tool results in state
        if isinstance(state, dict):
            state["tool_results"] = tool_results
            state["should_call_tools"] = False  # Reset flag
            messages = state.get("messages", [])
        else:
            state.tool_results = tool_results
            state.should_call_tools = False  # Reset flag
            messages = state.messages

        # Add tool results to messages for LLM context
        results_text = "\n".join(
            [f"Tool: {r['tool']}\nResult: {r['result']}" for r in tool_results]
        )

        from langchain_core.messages import ToolMessage

        messages.append(
            ToolMessage(
                content=f"Tool execution results:\n{results_text}",
                tool_call_id="tool_execution",
            )
        )

        return state
