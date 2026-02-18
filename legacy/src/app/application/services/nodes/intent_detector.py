"""Intent Detector Service for AI agent workflow."""

import logging
from typing import Optional
from app.domain.entities.agent_state import AgentState
from app.domain.ports.llm_client_port import LLMClientPort
from app.domain.parsers.think_cleaner_parser import ThinkCleanerParser

logger = logging.getLogger(__name__)


class IntentDetector:
    """Detects user intent from messages using LLM for autonomous classification."""

    def __init__(self, llm_client: LLMClientPort):
        self.llm_client = llm_client
        self.think_cleaner = ThinkCleanerParser()

    async def detect_intent(self, state: AgentState) -> AgentState:
        """Detect user intent using LLM for autonomous classification.

        Args:
            state: Current agent state containing messages

        Returns:
            Updated agent state with detected intent
        """

        # Get the latest user message
        latest_message: Optional[str] = None
        for msg in reversed(state.messages):
            # Handle different message types and formats
            if hasattr(msg, "type") and hasattr(msg, "content"):
                if msg.type == "human":  # type: ignore
                    latest_message = str(msg.content)  # type: ignore
                    break
            elif hasattr(msg, "role") and hasattr(msg, "content"):
                if msg.role == "user":  # type: ignore
                    latest_message = str(msg.content)  # type: ignore
                    break

        if not latest_message:
            logger.warning("No human/user message found in state.messages")
            # Set default intent
            state.needs_planning = False
            state.intent = "general_chat"
            return state

        # Use LLM to classify intent
        intent_prompt = f"""
        Analyze the following user message and classify its intent. Choose ONE of these categories:

        - "planning": The user wants help with planning, strategy, step-by-step approaches, or complex tasks
        - "general_chat": The user wants general conversation, simple questions, or assistance without specific planning needs

        User message: "{latest_message}"

        Respond with ONLY the category name (planning or general_chat) and nothing else.
        """

        try:
            # Call LLM to classify intent with specific model
            response = await self.llm_client.chat_completion(
                [
                    {
                        "role": "system",
                        "content": "You are an intent classification assistant. Respond with only the category name.",
                    },
                    {"role": "user", "content": intent_prompt},
                ],
                streaming=False,
            )

            # Clean the response from <think> tokens
            cleaned_response = self.think_cleaner.parse(response["response"])
            intent = cleaned_response.strip().lower()

            # Validate and map intent
            if intent == "planning":
                state.needs_planning = True
                state.intent = "planning"
                logger.info(
                    f"LLM detected planning intent for message: {latest_message[:100]}..."
                )
            elif intent == "general_chat":
                state.needs_planning = False
                state.intent = "general_chat"
                logger.info(
                    f"LLM detected general_chat intent for message: {latest_message[:100]}..."
                )
            else:
                # Handle unexpected responses
                logger.warning(
                    f"Unexpected intent response: '{intent}'. Defaulting to general_chat."
                )
                state.needs_planning = False
                state.intent = "general_chat"

            logger.info(f"Intent detection completed: {state.intent}")

        except Exception as e:
            logger.error(f"Error detecting intent with LLM: {str(e)}")
            # Fallback to general_chat on error
            state.needs_planning = False
            state.intent = "general_chat"

        return state
