"""Base channel interface."""

from abc import ABC, abstractmethod
from typing import Optional

from app.infrastructure.bus.events import InboundMessage, OutboundMessage


class BaseChannel(ABC):
    """Abstract base class for chat channels."""

    name: str = "base"

    @abstractmethod
    async def start(self) -> None:
        """Start the channel (connect, listen for messages)."""
        pass

    @abstractmethod
    async def stop(self) -> None:
        """Stop the channel (cleanup, disconnect)."""
        pass

    @abstractmethod
    async def send(self, msg: OutboundMessage) -> None:
        """Send a message through this channel.

        Args:
            msg: Message to send
        """
        pass

    async def receive(self) -> Optional[InboundMessage]:
        """Receive a message (optional, for polling-based channels).

        Returns:
            Received message or None
        """
        return None

    def is_allowed(
        self, sender_id: str, allow_list: Optional[list[str]] = None
    ) -> bool:
        """Check if sender is allowed to interact.

        Args:
            sender_id: Sender identifier
            allow_list: Optional list of allowed sender IDs

        Returns:
            True if allowed, False otherwise
        """
        if not allow_list:
            return True
        return sender_id in allow_list
