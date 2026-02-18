"""Updated LangGraph Orchestrator with new tools, memory, and skills integration."""

import json
import logging
import uuid
import os
from pathlib import Path
from typing import AsyncGenerator, Dict, Any, List, Optional

from langchain_core.messages import (
    SystemMessage,
    HumanMessage,
    AIMessage,
)
from langgraph.func import START
from langgraph.graph import StateGraph, END
from langgraph.checkpoint.memory import InMemorySaver

from app.domain.entities.agent_state import AgentState
from app.domain.ports.llm_client_port import LLMClientPort
from app.infrastructure.tools import ToolRegistry
from app.infrastructure.memory import MemoryStore, MemoryConsolidator
from app.infrastructure.session import SessionManager
from app.infrastructure.skills import ContextBuilder
from app.adapters.config import settings

from app.application.services.nodes.tool_execution_node import ToolExecutionNode
from app.application.services.nodes.context_builder_node import ContextBuilderNode


logger = logging.getLogger(__name__)


class EnhancedLangGraphOrchestrator:
    """Enhanced LangGraph orchestrator with new infrastructure integration."""

    def __init__(
        self,
        llm_client: LLMClientPort,
        workspace: Optional[Path] = None,
    ):
        """Initialize enhanced orchestrator.

        Args:
            llm_client: LLM client for completions
            workspace: Path to workspace directory (default: ~/.nova)
        """
        self.llm_client = llm_client
        self.workspace = Path(workspace or settings.workspace_dir)

        # Initialize new infrastructure components
        self._init_infrastructure()

        # Initialize nodes
        self._init_nodes()

        # Build graph
        self.graph = self._build_graph()

    def _init_infrastructure(self) -> None:
        """Initialize new infrastructure components."""
        # Memory and session management
        self.memory_store = MemoryStore(self.workspace)
        self.session_manager = SessionManager(self.workspace)
        self.memory_consolidator = MemoryConsolidator(
            self.workspace,
            self.llm_client,
        )

        # Skills and context
        self.context_builder = ContextBuilder(self.workspace)

        # Tool registry
        self.tool_registry = self._setup_tools()

        # Thread memory for LangGraph
        self.thread_memory = InMemorySaver()

        logger.info("Infrastructure initialized")

    def _setup_tools(self) -> ToolRegistry:
        """Setup and configure tools."""
        from app.infrastructure.tools import (
            ReadFileTool,
            WriteFileTool,
            EditFileTool,
            ListDirTool,
            ExecTool,
            WebSearchTool,
            WebFetchTool,
        )

        registry = ToolRegistry()

        # Filesystem tools with workspace restriction
        allowed_dir = self.workspace / "workspace"
        registry.register(ReadFileTool(allowed_dir=allowed_dir))
        registry.register(WriteFileTool(allowed_dir=allowed_dir))
        registry.register(EditFileTool(allowed_dir=allowed_dir))
        registry.register(ListDirTool(allowed_dir=allowed_dir))

        # Shell tool with safety
        registry.register(
            ExecTool(
                working_dir=str(allowed_dir),
                restrict_to_workspace=True,
                allowed_dir=allowed_dir,
            )
        )

        # Web tools
        brave_api_key = os.getenv("BRAVE_API_KEY")
        registry.register(WebSearchTool(api_key=brave_api_key))
        registry.register(WebFetchTool())

        logger.info(f"Registered {len(registry.list_tools())} tools")
        return registry

    def _init_nodes(self) -> None:
        """Initialize workflow nodes."""
        self.tool_execution_node = ToolExecutionNode(self.tool_registry)
        self.context_builder_node = ContextBuilderNode(
            self.context_builder,
            self.memory_store,
        )

    def _build_graph(self):
        """Build the enhanced LangGraph state graph."""
        workflow = StateGraph(AgentState)

        # Add nodes
        workflow.add_node("context_builder", self.context_builder_node.execute_node)
        workflow.add_node("llm_node", self._llm_node)
        workflow.add_node("tool_executor", self.tool_execution_node.execute_node)
        workflow.add_node("memory_consolidator", self._memory_consolidator_node)
        workflow.add_node("final_output", self._final_output_node)

        # Define workflow edges
        workflow.add_edge(START, "context_builder")
        workflow.add_edge("context_builder", "llm_node")

        # Conditional edge: check if tools need to be called
        workflow.add_conditional_edges(
            "llm_node",
            self._should_call_tools,
            {
                "tools": "tool_executor",
                "end": "memory_consolidator",
            },
        )

        # After tool execution, go back to LLM
        workflow.add_edge("tool_executor", "llm_node")

        # After memory consolidation, end
        workflow.add_edge("memory_consolidator", "final_output")
        workflow.add_edge("final_output", END)

        # Compile graph
        compiled_graph = workflow.compile(checkpointer=self.thread_memory)

        logger.info("Graph compiled successfully")
        return compiled_graph

    def _should_call_tools(self, state: AgentState) -> str:
        """Determine if tools should be called.

        Args:
            state: Current agent state (or dict from LangGraph)

        Returns:
            "tools" if tools need to be called, "end" otherwise
        """
        # Handle both AgentState object and dict formats
        if isinstance(state, dict):
            should_call = state.get("should_call_tools", False)
            tool_calls = state.get("tool_calls", [])
        else:
            should_call = state.should_call_tools
            tool_calls = state.tool_calls

        if should_call and tool_calls:
            logger.info(f"Routing to tool executor with {len(tool_calls)} tool calls")
            return "tools"
        return "end"

    async def _llm_node(self, state: AgentState) -> AgentState:
        """LLM node with tool calling support.

        Args:
            state: Current agent state (or dict from LangGraph)

        Returns:
            Updated state with LLM response
        """
        try:
            # Handle both AgentState object and dict formats
            if isinstance(state, dict):
                state_messages = state.get("messages", [])
                thread_id = state.get("thread_id", "default")
            else:
                state_messages = state.messages
                thread_id = state.thread_id

            # Get tool definitions
            tool_definitions = self.tool_registry.get_definitions()

            # Convert messages to dict format for LiteLLM
            messages = []
            for msg in state_messages:
                if hasattr(msg, "role") and hasattr(msg, "content"):
                    messages.append(
                        {
                            "role": msg.role,
                            "content": msg.content,
                        }
                    )
                elif isinstance(msg, dict):
                    messages.append(msg)

            # Call LLM with tools if available
            response = await self.llm_client.chat_completion(
                messages=messages,
                thread_id=thread_id,
                streaming=False,
                tools=tool_definitions if tool_definitions else None,
            )

            # Check for tool calls in response
            tool_calls = response.get("tool_calls")
            if tool_calls:
                logger.info(f"LLM requested {len(tool_calls)} tool calls")
                # Convert tool calls to our format
                formatted_tool_calls = []
                for tc in tool_calls:
                    if hasattr(tc, "function"):
                        formatted_tool_calls.append(
                            {
                                "name": tc.function.name,
                                "arguments": tc.function.arguments
                                if isinstance(tc.function.arguments, dict)
                                else json.loads(tc.function.arguments),
                            }
                        )
                    elif isinstance(tc, dict):
                        formatted_tool_calls.append(
                            {
                                "name": tc.get("function", {}).get("name"),
                                "arguments": tc.get("function", {}).get(
                                    "arguments", {}
                                ),
                            }
                        )

                if isinstance(state, dict):
                    state["tool_calls"] = formatted_tool_calls
                    state["should_call_tools"] = True
                else:
                    state.tool_calls = formatted_tool_calls
                    state.should_call_tools = True
            else:
                if isinstance(state, dict):
                    state["should_call_tools"] = False
                    state["tool_calls"] = []
                else:
                    state.should_call_tools = False
                    state.tool_calls = []

            # Add response to messages
            from langchain_core.messages import AIMessage

            content = response.get("response", "")

            if isinstance(state, dict):
                state_messages = state.get("messages", [])
                state_messages.append(AIMessage(content=content or ""))
                state["messages"] = state_messages
            else:
                state.messages.append(AIMessage(content=content or ""))

            logger.info(f"LLM node completed, response length: {len(content or '')}")

        except Exception as e:
            logger.error(f"Error in LLM node: {e}")
            # Handle both AgentState object and dict formats
            if isinstance(state, dict):
                state_messages = state.get("messages", [])
                state_messages.append(AIMessage(content=f"Error: {str(e)}"))
                state["messages"] = state_messages
            else:
                state.messages.append(AIMessage(content=f"Error: {str(e)}"))

        return state

    def _parse_tool_calls(self, content: str, state: AgentState) -> AgentState:
        """Parse tool calls from LLM response.

        Args:
            content: LLM response content
            state: Current state (or dict from LangGraph)

        Returns:
            Updated state with tool calls
        """
        import re
        import json

        tool_calls = []

        # Simple regex to parse tool calls
        # Format: TOOL: tool_name
        # Arguments: {...}
        pattern = r"TOOL:\s*(\w+)\s*\n?Arguments:\s*(\{[^}]+\})"
        matches = re.findall(pattern, content, re.IGNORECASE | re.DOTALL)

        for match in matches:
            tool_name = match[0].strip()
            try:
                args = json.loads(match[1])
                tool_calls.append(
                    {
                        "name": tool_name,
                        "arguments": args,
                    }
                )
            except json.JSONDecodeError:
                logger.warning(f"Failed to parse arguments for tool {tool_name}")

        # Handle both AgentState object and dict formats
        if isinstance(state, dict):
            state["tool_calls"] = tool_calls
            state["should_call_tools"] = len(tool_calls) > 0
        else:
            state.tool_calls = tool_calls
            state.should_call_tools = len(tool_calls) > 0

        return state

    async def _memory_consolidator_node(self, state: AgentState) -> AgentState:
        """Memory consolidation node.

        Args:
            state: Current agent state (or dict from LangGraph)

        Returns:
            Updated state
        """
        try:
            # Handle both AgentState object and dict formats
            if isinstance(state, dict):
                thread_id = state.get("thread_id", "default")
            else:
                thread_id = state.thread_id

            session = self.session_manager.get_or_create(thread_id)

            # Consolidate if we have enough messages
            if len(session.messages) > 50:
                await self.memory_consolidator.consolidate(session)
                self.session_manager.save(session)
                logger.info("Memory consolidated")

        except Exception as e:
            logger.error(f"Error in memory consolidation: {e}")

        return state

    async def _final_output_node(self, state: AgentState) -> AgentState:
        """Final output node.

        Args:
            state: Current agent state (or dict from LangGraph)

        Returns:
            Final state
        """
        # Save session
        try:
            # Handle both AgentState object and dict formats
            if isinstance(state, dict):
                thread_id = state.get("thread_id", "default")
                messages = state.get("messages", [])
            else:
                thread_id = state.thread_id
                messages = state.messages

            session = self.session_manager.get_or_create(thread_id)

            # Extract last AI message
            last_ai_message = None
            for msg in reversed(messages):
                if hasattr(msg, "type") and msg.type == "ai":
                    last_ai_message = msg.content
                    break

            if last_ai_message:
                session.add_message("assistant", last_ai_message)

            self.session_manager.save(session)

        except Exception as e:
            logger.error(f"Error saving session: {e}")

        return state

    async def stream_chat_completion(
        self,
        messages: List[Dict[str, Any]],
        thread_id: Optional[str] = None,
        user_id: Optional[str] = None,
    ) -> AsyncGenerator[Dict[str, Any], None]:
        """Stream chat completion using enhanced workflow.

        Args:
            messages: List of chat messages
            thread_id: Optional thread ID
            user_id: Optional user ID

        Yields:
            Response chunks
        """
        thread_id = thread_id or str(uuid.uuid4())
        user_id = user_id or "default_user"

        # Convert messages to LangChain format
        langchain_messages = self._convert_messages(messages)

        # Get or create session
        session = self.session_manager.get_or_create(thread_id)

        # Add user message to session
        if messages:
            last_msg = messages[-1]
            if last_msg.get("role") == "user":
                session.add_message("user", last_msg.get("content", ""))

        # Prepare initial state
        initial_state = AgentState(
            messages=langchain_messages,
            thread_id=thread_id,
            user_id=user_id,
        )

        config = {"configurable": {"thread_id": thread_id, "user_id": user_id}}

        # Execute graph
        try:
            final_state = None
            async for chunk in self.graph.astream(
                initial_state, config, stream_mode="values"
            ):
                final_state = chunk

            # Extract and yield response
            # Handle both AgentState object and dict formats
            messages = None
            if isinstance(final_state, dict):
                messages = final_state.get("messages", [])
            elif hasattr(final_state, "messages"):
                messages = final_state.messages

            if messages:
                for msg in reversed(messages):
                    if hasattr(msg, "type") and msg.type == "ai":
                        yield {"content": msg.content, "thread_id": thread_id}
                        break

        except Exception as e:
            logger.error(f"Error in workflow: {e}")
            yield {"content": f"Error: {str(e)}", "thread_id": thread_id}

        logger.info(f"Workflow completed for thread {thread_id}")

    async def chat_completion(
        self,
        messages: List[Dict[str, Any]],
        thread_id: Optional[str] = None,
    ) -> Dict[str, Any]:
        """Non-streaming chat completion.

        Args:
            messages: List of chat messages
            thread_id: Optional thread ID

        Returns:
            Response dictionary
        """
        thread_id = thread_id or str(uuid.uuid4())

        # Collect all chunks
        response_parts = []
        async for chunk in self.stream_chat_completion(messages, thread_id):
            response_parts.append(chunk.get("content", ""))

        full_response = "".join(response_parts)

        return {
            "response": full_response,
            "thread_id": thread_id,
            "memory_used": True,
            "tools_used": bool(self.tool_registry.list_tools()),
        }

    def _convert_messages(self, messages: List[Dict[str, Any]]) -> List[Any]:
        """Convert message dictionaries to LangChain format.

        Args:
            messages: List of message dictionaries

        Returns:
            List of LangChain message objects
        """
        langchain_messages = []
        for msg in messages:
            role = msg.get("role", "user")
            content = msg.get("content", "")

            if role == "assistant" or role == "ai":
                langchain_messages.append(AIMessage(content=content))
            elif role == "system":
                langchain_messages.append(SystemMessage(content=content))
            else:
                langchain_messages.append(HumanMessage(content=content))

        return langchain_messages
