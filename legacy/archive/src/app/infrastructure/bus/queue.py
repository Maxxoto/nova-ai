"""Message bus for decoupled channel-agent communication."""

import asyncio
import logging
from typing import Optional

from app.infrastructure.bus.events import InboundMessage, OutboundMessage


logger = logging.getLogger(__name__)


class MessageBus:
    """Async message bus for decoupling channels from agent."""

    def __init__(self):
        """Initialize message bus with inbound and outbound queues."""
        self.inbound: asyncio.Queue[InboundMessage] = asyncio.Queue()
        self.outbound: asyncio.Queue[OutboundMessage] = asyncio.Queue()
        self._running = False

    async def publish_inbound(self, msg: InboundMessage) -> None:
        """Publish message to inbound queue (channel -> agent).

        Args:
            msg: Inbound message
        """
        await self.inbound.put(msg)
        logger.debug(f"Published inbound message from {msg.channel}:{msg.sender_id}")

    async def consume_inbound(self) -> InboundMessage:
        """Consume message from inbound queue.

        Returns:
            Inbound message
        """
        msg = await self.inbound.get()
        self.inbound.task_done()
        return msg

    async def publish_outbound(self, msg: OutboundMessage) -> None:
        """Publish message to outbound queue (agent -> channel).

        Args:
            msg: Outbound message
        """
        await self.outbound.put(msg)
        logger.debug(f"Published outbound message to {msg.channel}:{msg.chat_id}")

    async def consume_outbound(self) -> OutboundMessage:
        """Consume message from outbound queue.

        Returns:
            Outbound message
        """
        msg = await self.outbound.get()
        self.outbound.task_done()
        return msg

    def start(self) -> None:
        """Mark bus as running."""
        self._running = True
        logger.info("Message bus started")

    def stop(self) -> None:
        """Mark bus as stopped."""
        self._running = False
        logger.info("Message bus stopped")

    @property
    def is_running(self) -> bool:
        """Check if bus is running."""
        return self._running
