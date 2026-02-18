#!/usr/bin/env python3
"""
Command-line interface for Nova Agent.
This script provides an interactive chat interface using the hexagonal architecture.
"""

import asyncio
import sys
import logging
from typing import List, Dict, Any

from app.dependencies import init_chat_service

# Configure logging
logging.basicConfig(
    level=logging.ERROR,
    format="%(asctime)s - %(name)s - %(levelname)s - %(message)s",
    handlers=[logging.StreamHandler(sys.stdout)],
)


class ChatInterface:
    """Interactive chat interface for Nova Agent."""

    def __init__(self):
        self.chat_service = None
        self.conversation_history: List[Dict[str, Any]] = []
        self.thread_id = None

    async def initialize(self):
        """Initialize the chat service"""
        if self.chat_service is None:
            self.chat_service = await init_chat_service()

    async def chat_streaming(self, message: str) -> str:
        """Send a message and stream the response."""
        print("\n🤖 Agent: ", end="", flush=True)

        # Prepare messages for the API
        messages = [{"role": "user", "content": message}]

        # Collect the complete response
        complete_response = ""

        try:
            # Stream the response
            async for chunk in self.chat_service.stream_chat_completion(
                messages=messages, thread_id=self.thread_id
            ):
                chunk_content = chunk["content"]
                print(chunk_content, end="", flush=True)
                complete_response += chunk_content

                # Store thread_id from the first chunk
                if self.thread_id is None:
                    self.thread_id = chunk["thread_id"]

            print()  # New line after response
            return complete_response

        except Exception as e:
            print(f"\n❌ Error: {str(e)}")
            return ""

    async def chat_non_streaming(self, message: str) -> str:
        """Send a message and get the complete response at once."""
        print("\n🤖 Agent: ", end="", flush=True)

        # Prepare messages for the API
        messages = [{"role": "user", "content": message}]

        try:
            # Get complete response
            response = await self.chat_service.chat_completion(
                messages=messages, thread_id=self.thread_id
            )

            print(response["response"])

            # Store thread_id if this is the first message
            if self.thread_id is None:
                self.thread_id = response["thread_id"]

            return response["response"]

        except Exception as e:
            print(f"\n❌ Error: {str(e)}")
            return ""

    async def get_conversation_history(self) -> List[Dict[str, Any]]:
        """Get the conversation history for the current thread."""
        if self.thread_id:
            return await self.chat_service.get_conversation_history(self.thread_id)
        return []

    async def clear_conversation(self) -> bool:
        """Clear the conversation memory."""
        if self.thread_id:
            success = await self.chat_service.clear_conversation_memory(self.thread_id)
            if success:
                self.thread_id = None
                self.conversation_history = []
                return True
        return False


async def main():
    """Main chat loop."""
    print("🚀 Welcome to Nova Agent Chat Interface!")
    print("Type 'quit', 'exit', or 'bye' to end the conversation")
    print("Type 'clear' to clear conversation history")
    print("Type 'history' to view conversation history")
    print("Type 'stream' to toggle streaming mode (default: on)")
    print("-" * 50)

    chat = ChatInterface()
    await chat.initialize()
    streaming_mode = True

    while True:
        try:
            user_input = input("\n👤 You: ").strip()

            if user_input.lower() in ["quit", "exit", "bye"]:
                print("👋 Goodbye!")
                break

            elif user_input.lower() == "clear":
                if await chat.clear_conversation():
                    print("🗑️ Conversation history cleared")
                else:
                    print("❌ No conversation to clear")
                continue

            elif user_input.lower() == "history":
                history = await chat.get_conversation_history()
                if history:
                    print("\n📜 Conversation History:")
                    for msg in history:
                        role = "👤 You" if msg.get("role") == "user" else "🤖 Agent"
                        print(f"{role}: {msg.get('content', '')}")
                else:
                    print("📜 No conversation history available")
                continue

            elif user_input.lower() == "stream":
                streaming_mode = not streaming_mode
                mode = "ON" if streaming_mode else "OFF"
                print(f"🔄 Streaming mode: {mode}")
                continue

            elif not user_input:
                continue

            # Process the user message
            if streaming_mode:
                await chat.chat_streaming(user_input)
            else:
                await chat.chat_non_streaming(user_input)

        except KeyboardInterrupt:
            print("\n👋 Goodbye!")
            break
        except Exception as e:
            print(f"\n❌ Unexpected error: {str(e)}")


if __name__ == "__main__":
    # Run the chat interface
    asyncio.run(main())
