"""Event types for message bus."""

from dataclasses import dataclass, field
from typing import Any, Optional


@dataclass
class InboundMessage:
    """Message received from a channel."""

    channel: str  # e.g., "telegram", "cli"
    sender_id: str  # User ID
    chat_id: str  # Chat/channel ID
    content: str  # Message text
    media: Optional[list] = None  # Media attachments
    metadata: Optional[dict] = None  # Extra channel-specific data

    @property
    def session_key(self) -> str:
        """Generate unique session key for this chat."""
        return f"{self.channel}:{self.chat_id}"


@dataclass
class OutboundMessage:
    """Message to be sent to a channel."""

    channel: str  # Target channel
    chat_id: str  # Target chat ID
    content: str  # Message text
    media: Optional[list] = None  # Media attachments
    metadata: Optional[dict] = None  # Extra channel-specific data
