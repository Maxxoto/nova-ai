"""Main entry point for Nova Agent.

This initializes all components and starts the system with:
- Message bus for channel communication
- AgentLoop for processing messages
- Channels (Telegram, etc.) for user interaction
"""

import asyncio
import logging
import os
import sys
from pathlib import Path
from typing import List

# Add src to path
sys.path.insert(0, str(Path(__file__).parent.parent))

from app.adapters.llm_providers.litellm_adapter import LiteLLMAdapter
from app.adapters.config import settings

# ============================================================================
# ORCHESTRATOR CHOICE: Choose one of the two approaches below
# ============================================================================
#
# OPTION 1: Nanobot-style (CURRENT) - Simple while loop for tool calling
#   - Simple, easy to debug
#   - Direct tool execution
#   - Good for basic agents
from app.application.services.agent_loop import AgentLoop
#
# OPTION 2: LangGraph-style (ADVANCED) - StateGraph with workflow nodes
#   - More extensible with nodes
#   - Better for complex workflows
#   - Visual graph debugging
#   - To switch: Comment out AgentLoop above, uncomment below:
#
# from app.application.services.enhanced_orchestrator import EnhancedLangGraphOrchestrator
#
# ============================================================================

from app.infrastructure.bus.queue import MessageBus
from app.infrastructure.channels.telegram import TelegramChannel
from app.infrastructure.tools import ToolRegistry
from app.infrastructure.tools.filesystem import (
    ReadFileTool,
    WriteFileTool,
    EditFileTool,
    ListDirTool,
)
from app.infrastructure.tools.shell import ExecTool
from app.infrastructure.tools.web import WebFetchTool


# Setup logging
logging.basicConfig(
    level=logging.INFO, format="%(asctime)s - %(name)s - %(levelname)s - %(message)s"
)
logger = logging.getLogger(__name__)


async def main():
    """Initialize and start Nova Agent."""

    logger.info("🚀 Starting Nova Agent...")

    # 1. Initialize Message Bus
    bus = MessageBus()
    bus.start()
    logger.info("✓ Message bus initialized")

    # 2. Setup workspace
    workspace = Path(os.getenv("NOVA_WORKSPACE", Path.home() / ".nova"))
    workspace.mkdir(parents=True, exist_ok=True)
    logger.info(f"✓ Workspace: {workspace}")

    # 3. Initialize LLM Client
    api_key = os.getenv("LITE_LLM_API_KEY")
    if not api_key:
        logger.error("❌ LITE_LLM_API_KEY not set!")
        logger.error("Set it with: export LITE_LLM_API_KEY=your-key")
        sys.exit(1)

    llm = LiteLLMAdapter(
        model=os.getenv("LITE_LLM_MODEL", settings.lite_llm_model),
        api_key=api_key,
        temperature=settings.litellm_temperature,
        max_tokens=settings.litellm_max_tokens,
    )
    logger.info(f"✓ LLM initialized: {settings.lite_llm_model}")

    # 4. Initialize Tool Registry
    tool_registry = ToolRegistry()
    allowed_dir = workspace  # Restrict file operations to workspace

    # Register filesystem tools
    tool_registry.register(ReadFileTool(allowed_dir=allowed_dir))
    tool_registry.register(WriteFileTool(allowed_dir=allowed_dir))
    tool_registry.register(EditFileTool(allowed_dir=allowed_dir))
    tool_registry.register(ListDirTool(allowed_dir=allowed_dir))

    # Register shell tool
    tool_registry.register(ExecTool(allowed_dir=allowed_dir))

    # Register web tool
    tool_registry.register(WebFetchTool())

    logger.info(
        f"✓ Tool registry initialized with {len(tool_registry.list_tools())} tools"
    )

    # ============================================================================
    # ORCHESTRATOR INITIALIZATION - Choose one below
    # ============================================================================
    #
    # OPTION 1: Nanobot-style (CURRENT)
    #   - Simple while loop that calls LLM with tools
    #   - Handles tool execution in loop
    agent_loop = AgentLoop(
        bus=bus,
        llm_client=llm,
        workspace=workspace,
        tool_registry=tool_registry,
    )
    logger.info("✓ AgentLoop initialized (nanobot-style)")
    #
    # OPTION 2: LangGraph-style (ADVANCED)
    #   - To switch: Comment out AgentLoop above, uncomment below
    #   - Then update startup section (see below)
    #
    # orchestrator = EnhancedLangGraphOrchestrator(
    #     llm_client=llm,
    #     workspace=workspace,
    # )
    # logger.info("✓ EnhancedLangGraphOrchestrator initialized")
    #
    # ============================================================================

    # 6. Initialize Channels
    channels: List[TelegramChannel] = []

    # Telegram (if configured)
    telegram_token = os.getenv("TELEGRAM_TOKEN")
    if telegram_token:
        # Get allowed user IDs from environment
        # Format: TELEGRAM_ALLOW_LIST="123456789,987654321"
        allow_list_str = os.getenv("TELEGRAM_ALLOW_LIST", "")
        allow_list = [x.strip() for x in allow_list_str.split(",") if x.strip()] or None

        if allow_list:
            logger.info(f"🔒 Telegram access restricted to user IDs: {allow_list}")
        else:
            logger.warning(
                "⚠️  Telegram allow_list not set - bot will accept messages from ANYONE!"
            )
            logger.info("   Set TELEGRAM_ALLOW_LIST='your_user_id' to restrict access")

        telegram = TelegramChannel(
            token=telegram_token,
            bus=bus,
            allow_list=allow_list,
        )
        channels.append(telegram)
        logger.info("✓ Telegram channel initialized")
    else:
        logger.info("○ Telegram not configured (set TELEGRAM_TOKEN)")

    if not channels:
        logger.warning(
            "⚠️  No channels configured! Set TELEGRAM_TOKEN to enable Telegram."
        )
        logger.info("Nova will run but won't receive any messages.")

    # ============================================================================
    # STARTUP - Choose one section below based on your orchestrator choice
    # ============================================================================
    try:
        # ----------------------------------------------------------------------
        # OPTION 1: Nanobot-style startup (CURRENT)
        # ----------------------------------------------------------------------
        # Start AgentLoop (consumes from bus and processes messages)
        await agent_loop.start()

        # Start all channels
        channel_tasks = []
        for channel in channels:
            task = asyncio.create_task(channel.start())
            channel_tasks.append(task)

        logger.info("\n🎉 Nova Agent is running!")
        logger.info("Press Ctrl+C to stop\n")

        # Keep running until interrupted
        while True:
            await asyncio.sleep(1)

        # ----------------------------------------------------------------------
        # OPTION 2: LangGraph-style startup (ADVANCED)
        # ----------------------------------------------------------------------
        # To switch: Comment out OPTION 1 above, uncomment OPTION 2 below:
        #
        # # Create message processing task that uses orchestrator
        # async def process_messages():
        #     from app.infrastructure.bus.events import OutboundMessage
        #     while True:
        #         try:
        #             msg = await asyncio.wait_for(bus.consume_inbound(), timeout=1.0)
        #             logger.info(f"Processing message from {msg.channel}:{msg.sender_id}")
        #
        #             # Call orchestrator
        #             response = await orchestrator.chat_completion(
        #                 messages=[{"role": "user", "content": msg.content}],
        #                 thread_id=msg.session_key,
        #             )
        #
        #             # Publish response
        #             await bus.publish_outbound(
        #                 OutboundMessage(
        #                     channel=msg.channel,
        #                     chat_id=msg.chat_id,
        #                     content=response.get("response", "No response generated"),
        #                     metadata=msg.metadata,
        #                 )
        #             )
        #             logger.info(f"Response sent: {len(response.get('response', ''))} chars")
        #
        #         except asyncio.TimeoutError:
        #             continue
        #         except Exception as e:
        #             logger.error(f"Error processing message: {e}", exc_info=True)
        #
        # # Start message processing
        # processing_task = asyncio.create_task(process_messages())
        #
        # # Start all channels
        # channel_tasks = []
        # for channel in channels:
        #     task = asyncio.create_task(channel.start())
        #     channel_tasks.append(task)
        #
        # logger.info("\n🎉 Nova Agent is running with LangGraph!")
        # logger.info("Press Ctrl+C to stop\n")
        #
        # # Keep running until interrupted
        # while True:
        #     await asyncio.sleep(1)
        #
        # ============================================================================

    except KeyboardInterrupt:
        logger.info("\n🛑 Shutting down...")
    except Exception as e:
        logger.error(f"Error: {e}", exc_info=True)
    finally:
        # Cleanup - OPTION 1 (AgentLoop)
        await agent_loop.stop()
        #
        # Cleanup - OPTION 2 (LangGraph)
        # Uncomment below if using LangGraph:
        # processing_task.cancel()
        #
        for channel in channels:
            await channel.stop()
        bus.stop()
        logger.info("✓ Shutdown complete")


if __name__ == "__main__":
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        print("\n👋 Goodbye!")
        sys.exit(0)
