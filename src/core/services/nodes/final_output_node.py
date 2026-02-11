import logging
from core.entities.agent_state import AgentState


logger = logging.getLogger(__name__)


class FinalOutputNode:
    def get_final_output(self, state: AgentState) -> AgentState:
        """Get latest ai message from message agent state"""
        latest_ai_message = None
        for msg in reversed(state.messages):
            if hasattr(msg, "type") and msg.type == "ai":
                latest_ai_message = msg.content
                break

        if latest_ai_message is None:
            logger.error("Cannot parse final output")
        state.final_output = latest_ai_message

        return state
