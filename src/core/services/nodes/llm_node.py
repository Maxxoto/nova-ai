"""LLM Node for AI agent workflow."""

import logging
from core.entities.agent_state import AgentState
from core.ports.llm_client_port import LLMClientPort
from langchain_core.messages import AIMessage

logger = logging.getLogger(__name__)


class LLMNode:
    """Node that interacts with the LLM to generate responses."""

    def __init__(self, llm_client: LLMClientPort):
        self.llm_client = llm_client

    async def generate_response(self, state: AgentState) -> AgentState:
        """Generate response using LLM with enhanced context including memory summaries."""

        try:
            # Convert messages to format expected by LLM client
            messages_for_llm = []

            # Add memory context as system message if available
            memory_context = self._build_memory_context(state)
            if memory_context:
                messages_for_llm.append({"role": "system", "content": memory_context})

            for msg in state.messages:
                if hasattr(msg, "type"):
                    if msg.type == "human":
                        messages_for_llm.append(
                            {"role": "user", "content": msg.content}
                        )
                    elif msg.type == "ai":
                        messages_for_llm.append(
                            {"role": "assistant", "content": msg.content}
                        )
                    elif msg.type == "system":
                        messages_for_llm.append(
                            {"role": "system", "content": msg.content}
                        )

            # Generate response using LLM
            response = await self.llm_client.chat_completion(messages_for_llm)

            ai_message = AIMessage(content=response["response"])
            state.messages.append(ai_message)

            logger.debug("Generated LLM response with memory context")
            return state

        except Exception as e:
            logger.error(f"Error generating LLM response: {str(e)}")
            # Add error message to state
            error_message = AIMessage(
                content="I apologize, but I encountered an error while processing your request."
            )
            state.messages.append(error_message)
            return state

    def _build_memory_context(self, state: AgentState) -> str:
        """Build memory context from summarized memory and conversation history using XML tags."""
        context_parts = []

        # Add memory summary if available
        if state.memory_summary:
            context_parts.append(
                f"<memory_summary>\n{state.memory_summary}\n</memory_summary>"
            )

        # Add conversation summary if available
        if state.conversation_summary:
            context_parts.append(
                f"<conversation_summary>\n{state.conversation_summary}\n</conversation_summary>"
            )

        # Add raw recalled memory if no summary is available
        if state.recalled_memory and not state.memory_summary:
            context_parts.append("<recalled_memory>")
            for memory in state.recalled_memory:
                role = memory.get("role", "unknown")
                content = memory.get("content", "")
                context_parts.append(
                    f'<memory_entry role="{role}">\n{content}\n</memory_entry>'
                )
            context_parts.append("</recalled_memory>")

        if context_parts:
            return "\n\n".join(context_parts)

        return ""

    async def execute_node(self, state: AgentState) -> AgentState:
        """Generate response using LLM."""
        try:
            # Use the actual LLM node to generate response
            return await self.generate_response(state)
        except Exception as e:
            logger.error(f"Error in LLM node: {str(e)}")
            # Add error message to state

            error_message = AIMessage(
                content="I apologize, but I encountered an error while processing your request."
            )
            state.messages.append(error_message)
            return state
