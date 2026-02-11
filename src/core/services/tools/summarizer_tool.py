"""Summarizer Tool for AI agent workflow."""

import logging
from core.entities.agent_state import AgentState
from core.ports.llm_client_port import LLMClientPort
from core.ports.memory_port import MemoryPort

logger = logging.getLogger(__name__)


class SummarizerTool:
    """Tool that summarizes conversation memory and content."""

    def __init__(self, llm_client: LLMClientPort, memory_adapter: MemoryPort):
        self.llm_client = llm_client
        self.memory_adapter = memory_adapter

    async def summarize_conversation(self, state: AgentState) -> AgentState:
        """Summarize the current conversation and memory context."""

        try:
            user_id = state.user_id or "default_user"

            # Get conversation history for summarization
            conversation_history = await self.memory_adapter.get_conversation_history(
                user_id
            )

            if not conversation_history:
                logger.info("No conversation history found to summarize")
                return state

            # Prepare conversation content for summarization
            conversation_content = ""
            for message in conversation_history:
                role = message.get("role", "unknown")
                content = message.get("content", "")
                conversation_content += f"{role}: {content}\n"

            # Create summarization prompt
            summarization_prompt = [
                {
                    "role": "system",
                    "content": """You are a helpful AI assistant that creates concise summaries of conversations.
                    Create a clear, structured summary that captures:
                    1. Main topics discussed
                    2. Key decisions or conclusions
                    3. Important context or information shared
                    4. Any action items or next steps

                    Keep the summary focused and useful for future reference.""",
                },
                {
                    "role": "user",
                    "content": f"Please summarize the following conversation:\n\n{conversation_content}",
                },
            ]

            # Generate summary using LLM
            summary_response = await self.llm_client.chat_completion(
                summarization_prompt, streaming=False
            )
            summary = summary_response.get("response", "No summary generated.")

            # Store the summary in long-term memory
            await self.memory_adapter.store_long_term_memory(
                user_id=user_id,
                content=f"Conversation Summary: {summary}",
                metadata={"type": "summary", "source": "summarizer_tool"},
            )

            # Add summary to state for current context
            state.conversation_summary = summary
            logger.info(f"Generated conversation summary for user {user_id}")

            return state

        except Exception as e:
            logger.error(f"Error summarizing conversation: {str(e)}")
            state.conversation_summary = "Unable to generate summary due to error."
            return state

    async def summarize_memory(self, state: AgentState) -> AgentState:
        """Summarize recalled memory for better context."""

        try:
            recalled_memory = state.recalled_memory

            if not recalled_memory:
                logger.info("No recalled memory to summarize")
                return state

            # Extract memory content
            memory_content = ""
            for memory in recalled_memory:
                role = memory.get("role", "unknown")
                content = memory.get("content", "")
                memory_content += f"{role}: {content}\n"

            # Create memory summarization prompt
            memory_prompt = [
                {
                    "role": "system",
                    "content": """You are an AI assistant that summarizes recalled memory snippets into a short, clean list.
                    Produce a concise summary in this exact format:
                    - Key point one
                    - Key point two
                    - Key point three

                    Focus on:
                    1.Important information from previous conversations
                    2.User preferences or context that matter for future replies
                    3.Any relevant facts needed for context-aware responses
                    Keep the wording simple and to the point. Do not add explanations outside the bullet list.""",
                },
                {
                    "role": "user",
                    "content": f"Please summarize the following recalled memory context:\n\n{memory_content}",
                },
            ]

            # Generate memory summary
            memory_response = await self.llm_client.chat_completion(
                memory_prompt, streaming=False
            )
            memory_summary = memory_response.get(
                "response", "No memory summary generated."
            )

            # Add memory summary to state
            state.memory_summary = memory_summary
            logger.info("Generated memory summary")

            return state

        except Exception as e:
            logger.error(f"Error summarizing memory: {str(e)}")
            state.memory_summary = "Unable to generate memory summary due to error."
            return state

    async def execute_tool(self, state: AgentState) -> AgentState:
        """Execute the summarizer tool based on state context."""

        try:
            # Check if we should summarize conversation or memory
            if state.needs_conversation_summary:
                state = await self.summarize_conversation(state)
            elif state.needs_memory_summary:
                state = await self.summarize_memory(state)
            else:
                # Default: summarize both if we have content
                if state.recalled_memory:
                    state = await self.summarize_memory(state)

                # Check if we have sufficient conversation history to summarize
                user_id = state.user_id or "default_user"
                conversation_history = (
                    await self.memory_adapter.get_conversation_history(user_id)
                )
                if (
                    conversation_history and len(conversation_history) > 5
                ):  # Only summarize if we have enough history
                    state = await self.summarize_conversation(state)

            logger.info("Summarizer tool executed successfully")
            return state

        except Exception as e:
            logger.error(f"Error in summarizer tool: {str(e)}")
            return state
