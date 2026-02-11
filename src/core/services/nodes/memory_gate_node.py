"""Memory Gate Node for AI agent workflow - acts as a judge for memory operations."""

import logging
from core.entities.agent_state import AgentState
from core.ports.llm_client_port import LLMClientPort
from core.ports.memory_port import MemoryPort
from core.services.parsers.think_cleaner_parser import ThinkCleanerParser

logger = logging.getLogger(__name__)


class MemoryGateNode:
    """Node that acts as a judge to decide memory operations (save, recall, or nothing)."""

    def __init__(self, llm_client: LLMClientPort, memory_adapter: MemoryPort):
        self.llm_client = llm_client
        self.memory_adapter = memory_adapter
        self.think_cleaner = ThinkCleanerParser()

    async def judge_memory_operation(self, state: AgentState) -> AgentState:
        """Judge whether to save memory, recall memory, or do nothing."""

        # Get the latest user message
        latest_message = None
        for msg in reversed(state.messages):
            if hasattr(msg, "type") and msg.type == "human":
                latest_message = msg.content
                break

        if not latest_message:
            state.memory_operation = "none"
            return state

        # Use LLM to judge memory operation
        judgment_prompt = f"""
        Analyze the following user message and conversation context to determine the appropriate memory operation.
        Choose ONE of these operations:

        - "save_memory": The conversation contains important information worth saving for future reference
        - "recall_memory": The user is asking about or referencing previous conversations, memories, or history
        - "none": No memory operation is needed - this is general conversation without significant memory value

        Consider these factors:
        - Is the user asking about past conversations or memories? (choose recall_memory)
        - Does the conversation contain important information, decisions, or learnings? (choose save_memory)
        - Is this just casual conversation without lasting value? (choose none)

        User message: "{latest_message}"

        Respond with ONLY the operation name (save_memory, recall_memory, or none) and nothing else.
        """

        try:
            # Call LLM to judge memory operation
            response = await self.llm_client.chat_completion(
                [
                    {
                        "role": "system",
                        "content": "You are a memory operation judge. Respond with only the operation name.",
                    },
                    {"role": "user", "content": judgment_prompt},
                ],
                streaming=False,
            )

            # Clean the response from <think> tokens
            cleaned_response = self.think_cleaner.parse(response["response"])
            operation = cleaned_response.strip().lower()

            logger.debug(f"Memory gate judgment: {operation}")

            # Set the appropriate flags based on judgment
            if operation == "save_memory":
                state.needs_memory_write = True
                state.needs_memory_recall = False
                state.memory_operation = "save_memory"
            elif operation == "recall_memory":
                state.needs_memory_write = False
                state.needs_memory_recall = True
                state.memory_operation = "recall_memory"
            else:
                # Default to none for any other response
                state.needs_memory_write = False
                state.needs_memory_recall = False
                state.memory_operation = "none"

            logger.info(f"Memory gate decided: {state.memory_operation}")

        except Exception as e:
            logger.error(f"Error judging memory operation with LLM: {str(e)}")
            # Fallback to none on error
            state.needs_memory_write = False
            state.needs_memory_recall = False
            state.memory_operation = "none"

        return state

    async def execute_node(self, state: AgentState) -> AgentState:
        """Execute memory gate judgment."""
        try:
            state = await self.judge_memory_operation(state)
            logger.info(
                f"Memory gate node executed, operation: {state.memory_operation or 'none'}"
            )
        except Exception as e:
            logger.error(f"Error in memory gate node: {str(e)}")
            state.needs_memory_write = False
            state.needs_memory_recall = False
            state.memory_operation = "none"

        return state
