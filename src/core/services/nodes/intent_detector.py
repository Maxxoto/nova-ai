"""Intent Detector Service for AI agent workflow."""

import logging
from core.entities.agent_state import AgentState
from core.ports.llm_client_port import LLMClientPort
from core.services.parsers.think_cleaner_parser import ThinkCleanerParser

logger = logging.getLogger(__name__)


class IntentDetector:
    """Detects user intent from messages using LLM for autonomous classification."""

    def __init__(self, llm_client: LLMClientPort):
        self.llm_client = llm_client
        self.think_cleaner = ThinkCleanerParser()

    async def detect_intent(self, state: AgentState) -> AgentState:
        """Detect user intent using LLM for autonomous classification."""

        # Get the latest user message
        latest_message = None
        for msg in reversed(state.messages):
            if hasattr(msg, "type") and msg.type == "human":
                latest_message = msg.content
                break

        if not latest_message:
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
            else:
                # Default to general_chat for any other response
                state.needs_planning = False
                state.intent = "general_chat"

            logger.info(f"LLM detected intent: {state.intent}")

        except Exception as e:
            logger.error(f"Error detecting intent with LLM: {str(e)}")
            # Fallback to general_chat on error
            state.needs_planning = False
            state.intent = "general_chat"

        return state
