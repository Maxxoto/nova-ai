"""LangGraph application entry point for LangGraph CLI integration."""

from adapters.llm_providers.groq import GroqLLMAdapter
from dotenv import load_dotenv

# Load environment variables
load_dotenv()


def create_agent():
    """Create and return the LangGraph agent instance without custom checkpointer."""

    # Initialize the LLM client
    llm_client = GroqLLMAdapter()

    # Create a simplified version without custom checkpointer for LangGraph API
    # This avoids the "custom checkpointer" error from LangGraph API
    from langchain_core.messages import SystemMessage
    from langgraph.graph import StateGraph, START, END
    from core.entities.agent_state import AgentState

    def agent_node(state: AgentState):
        system_prompt = (
            "You are a helpful AI assistant. Provide clear, concise, and helpful responses. "
            "Use the conversation history to maintain context and provide relevant responses."
        )
        messages = state.messages
        # Ensure system_prompt is included in the conversation
        if messages and not isinstance(messages[0], SystemMessage):
            messages_to_send = [system_prompt] + messages
        else:
            messages_to_send = messages

        # Use the LLM client to get the response
        response = llm_client.invoke(messages_to_send)
        return {"messages": [response]}

    # Build the graph WITHOUT custom checkpointer for LangGraph API
    workflow = StateGraph(AgentState)
    workflow.add_node("chatbot", agent_node)
    workflow.add_edge(START, "chatbot")
    workflow.add_edge("chatbot", END)

    # Compile without checkpointer - LangGraph API handles persistence automatically
    return workflow.compile()


# Export the graph for LangGraph CLI
graph = create_agent()
