"""Advanced LangGraph Orchestrator with workflow nodes."""

import logging
import uuid
import os
from typing import AsyncGenerator, Dict, Any, List, Optional

from core.services.nodes.final_output_node import FinalOutputNode
from core.services.nodes.intent_detector import IntentDetector
from core.services.nodes.llm_node import LLMNode
from core.services.nodes.memory_recall_node import MemoryRecallNode
from core.services.nodes.memory_write_node import MemoryWriteNode
from core.services.nodes.memory_gate_node import MemoryGateNode
from core.services.tools.multiply_tool import multiply
from core.services.tools.summarizer_tool import SummarizerTool
from langchain_core.messages import (
    AIMessageChunk,
    SystemMessage,
    HumanMessage,
    AIMessage,
)
from langgraph.func import START
from langgraph.graph import StateGraph, END
from opik.integrations.langchain import OpikTracer

from core.entities.agent_state import AgentState
from core.ports.llm_client_port import LLMClientPort
from core.ports.memory_port import MemoryPort

from core.entities.plan import Plan  # Import Plan


logger = logging.getLogger(__name__)


class LangGraphOrchestrator:
    """Advanced LangGraph orchestrator with workflow nodes for AI agent."""

    def __init__(
        self,
        llm_client: LLMClientPort,
        memory_adapter: MemoryPort,
        thread_memory=None,
        longterm_memory=None,
    ):
        self.llm_client = llm_client

        # Tools bind here

        # summarize_conversation = summarizer_tool.summarize_conversation
        # summarize_memory = self.summarizer_tool.summarize_memory

        tools = [multiply]
        llm_with_tools = llm_client.get_llm_client()
        self.llm_client = llm_with_tools.bind_tools(tools)
        self.memory_adapter = memory_adapter
        self.memory = thread_memory
        self.lt_memory = longterm_memory

        # Initialize observability
        self._setup_observability()

        # Initialize workflow nodes
        self.summarizer_tool = SummarizerTool(llm_client, memory_adapter)
        self.intent_detector = IntentDetector(llm_client)
        self.memory_gate_node = MemoryGateNode(llm_client, memory_adapter)
        self.memory_recall_node = MemoryRecallNode(llm_client, memory_adapter)
        self.llm_node = LLMNode(llm_client)
        self.memory_write_node = MemoryWriteNode(llm_client, memory_adapter)
        self.final_output_node = FinalOutputNode()

        self.graph = self._build_graph()

    def _route_memory_operation(self, state: AgentState) -> str:
        """Route workflow based on memory operation needs."""

        # Call memory recall when chat initialization for personalization
        if len(state.messages) == 1 and state.messages[0].type == "human":
            state.needs_memory_recall = True

        if state.needs_memory_recall:
            logger.info("Routing to memory recall node")
            return "memory_recall_node"
        elif state.needs_memory_write:
            logger.info("Routing to memory write node")
            return "memory_write_node"
        else:
            logger.info("Routing to planner node (no memory operation needed)")
            return "planner_node"

    def _create_plan_and_prompt(self, state: AgentState) -> AgentState:
        """Create execution plan and build enhanced prompt internally."""
        latest_message = None
        for msg in reversed(state.messages):
            if hasattr(msg, "type") and msg.type == "human":
                latest_message = msg.content
                break

        if not latest_message:
            return state

        plan = Plan(
            name=f"Plan for: {latest_message[:50]}...",
            description="Generated plan based on user request",
        )

        if state.needs_planning:
            plan.add_step(
                description="Analyze the user's request and requirements",
                action="analyze_request",
            )
            plan.add_step(
                description="Generate step-by-step approach", action="generate_approach"
            )
            plan.add_step(
                description="Execute the plan and provide results",
                action="execute_plan",
            )

        state.plan = plan

        enhanced_prompt = self._build_enhanced_prompt(state, latest_message)

        state.messages.insert(0, SystemMessage(content=enhanced_prompt))

        logger.info(f"Created internal plan with {len(plan.steps)} steps")
        return state

    def _build_enhanced_prompt(self, state: AgentState, user_message: str) -> str:
        """Build enhanced prompt with memory, summaries, and plan context internally."""
        prompt_parts = [
            "You are a helpful AI assistant. Provide clear, concise, and helpful responses.",
            "Use the conversation history and context below to maintain context and provide relevant responses.",
        ]

        # Add conversation summary if available
        if state.conversation_summary:
            prompt_parts.append(
                f"\nConversation Summary:\n{state.conversation_summary}"
            )

        # Add memory summary if available
        if state.memory_summary:
            prompt_parts.append(f"\nMemory Context Summary:\n{state.memory_summary}")

        if state.recalled_memory:
            prompt_parts.append("\nPrevious conversation context:")
            for memory in state.recalled_memory:
                role = memory.get("role", "unknown")
                content = memory.get("content", "")
                prompt_parts.append(f"{role}: {content}")

        if state.plan:
            plan = state.plan
            prompt_parts.append(f"\nCurrent plan: {plan.name}")
            if plan.description:
                prompt_parts.append(f"Plan description: {plan.description}")

        prompt_parts.append(f"\nCurrent user message: {user_message}")

        return "\n".join(prompt_parts)

    def _build_graph(self):
        """Build the advanced LangGraph state graph with workflow nodes."""

        # Build the graph with workflow nodes
        workflow = StateGraph(AgentState)

        # Add nodes
        workflow.add_node("intent_detector", self.intent_detector.detect_intent)
        workflow.add_node("memory_gate_node", self.memory_gate_node.execute_node)
        workflow.add_node("memory_recall_node", self.memory_recall_node.execute_node)
        workflow.add_node("planner_node", self._create_plan_and_prompt)
        workflow.add_node("llm_node", self.llm_node.execute_node)
        workflow.add_node("memory_write_node", self.memory_write_node.execute_node)
        workflow.add_node("summarizer_node", self.summarizer_tool.execute_tool)
        workflow.add_node("final_output_node", self.final_output_node.get_final_output)

        # Define workflow edges
        workflow.add_edge(START, "intent_detector")
        workflow.add_edge("intent_detector", "memory_gate_node")
        workflow.add_conditional_edges(
            "memory_gate_node",
            self._route_memory_operation,  # Use memory operation routing method
            {
                "memory_recall_node": "memory_recall_node",
                "memory_write_node": "memory_write_node",
                "planner_node": "planner_node",
            },
        )

        workflow.add_edge(
            "memory_recall_node", "summarizer_node"
        )  # After memory recall, summarize
        workflow.add_edge(
            "summarizer_node", "llm_node"
        )  # After summarization, go to LLM
        workflow.add_edge("planner_node", "llm_node")  # After planning, go to LLM
        workflow.add_edge(
            "memory_write_node", "llm_node"
        )  # After memory write, go to LLM
        workflow.add_edge("llm_node", "final_output_node")
        workflow.add_edge("final_output_node", END)

        compiled_graph = workflow.compile(
            checkpointer=self.memory, store=self.lt_memory
        )

        # Get the PNG image bytes
        png_data = compiled_graph.get_graph().draw_mermaid_png()
        # Write it to a file
        with open("mermaid_graph.png", "wb") as f:
            f.write(png_data)

        return compiled_graph

    async def stream_chat_completion(
        self,
        messages: List[Dict[str, Any]],
        thread_id: Optional[str] = None,
        user_id: Optional[str] = None,
    ) -> AsyncGenerator[Dict[str, Any], None]:
        """Stream chat completion using advanced workflow."""

        thread_id = thread_id or "default"
        if thread_id == "default":
            thread_id = str(uuid.uuid4())

        if not user_id:
            user_id = "default_user"
        # Convert message dictionaries to proper LangChain message objects
        langchain_messages = self._convert_messages_to_langchain_format(messages)

        # Prepare initial state
        initial_state = AgentState(
            messages=langchain_messages, thread_id=thread_id, user_id=user_id
        )
        config = {"configurable": {"thread_id": thread_id, "user_id": user_id}}

        # Add OpikTracer to config if available
        if hasattr(self, "opik_tracer") and self.opik_tracer:
            config["callbacks"] = [self.opik_tracer]

        # Collect complete response
        complete_response = ""

        # Execute graph with memory to stream messages
        async for chunk in self.graph.astream(
            initial_state, config, stream_mode="messages"
        ):
            metadata: dict = chunk[1]

            if (
                chunk
                and isinstance(chunk[0], AIMessageChunk)
                and metadata.get("langgraph_node") == "llm_node"
            ):
                chunk_content = chunk[0].content
                complete_response += chunk_content
                yield {"content": chunk_content, "thread_id": thread_id}

        logger.info(f"Completed advanced workflow for thread {thread_id}")

        # Log workflow metrics for streaming completion
        self._log_workflow_metrics(initial_state, thread_id, complete_response)

    async def chat_completion(
        self, messages: List[Dict[str, Any]], thread_id: Optional[str] = None
    ) -> Dict[str, Any]:
        """Non-streaming chat completion using advanced workflow."""

        thread_id = thread_id or "default"

        # Convert message dictionaries to proper LangChain message objects
        langchain_messages = self._convert_messages_to_langchain_format(messages)

        initial_state = AgentState(messages=langchain_messages, thread_id=thread_id)
        config = {"configurable": {"thread_id": thread_id}}

        # Add OpikTracer to config if available
        if hasattr(self, "opik_tracer") and self.opik_tracer:
            config["callbacks"] = [self.opik_tracer]

        # Execute the graph with memory
        final_state = self.graph.invoke(initial_state, config)

        # Extract response from messages
        response_content = ""
        if final_state.messages:
            # Get the last AI message content
            for msg in reversed(final_state.messages):
                if hasattr(msg, "content") and msg.content:
                    response_content = msg.content
                    break

        # Log workflow metrics for non-streaming completion
        self._log_workflow_metrics(final_state, thread_id, response_content)

        return {
            "response": response_content,
            "thread_id": thread_id,
            "memory_used": True,
            "workflow_completed": True,
        }

    def _setup_observability(self):
        """Setup Opik observability."""
        try:
            # Setup Opik Tracer
            opik_api_key = os.getenv("OPIK_API_KEY")
            if opik_api_key:
                # Configure Opik first - only API key is needed
                import opik

                opik.configure(api_key=opik_api_key, workspace="ahmat-dani-setiawan")

                # Then create the tracer
                self.opik_tracer = OpikTracer()
                logger.info("Opik tracer initialized successfully")
            else:
                logger.warning("OPIK_API_KEY not found, Opik observability disabled")
                self.opik_tracer = None

        except Exception as e:
            logger.error(f"Failed to initialize observability: {str(e)}")
            self.opik_tracer = None

    def _convert_messages_to_langchain_format(
        self, messages: List[Dict[str, Any]]
    ) -> List[Any]:
        """Convert message dictionaries to proper LangChain message objects.

        Args:
            messages: List of message dictionaries with 'role' and 'content' keys

        Returns:
            List of LangChain message objects (HumanMessage or AIMessage)
        """
        langchain_messages = []
        for msg in messages:
            role = msg.get("role", "user")
            content = msg.get("content", "")
            if role == "ai":
                langchain_messages.append(AIMessage(content=content))
            else:
                langchain_messages.append(HumanMessage(content=content))
        return langchain_messages

    def _log_workflow_metrics(
        self, state: AgentState, thread_id: str, response_content: str
    ):
        """Log workflow metrics."""
        # Opik automatically logs metrics through the tracer
        logger.info(
            f"Workflow completed for thread {thread_id}, response length: {len(response_content)}"
        )
