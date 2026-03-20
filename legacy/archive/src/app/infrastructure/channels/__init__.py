"""Channel infrastructure for agent."""

from app.infrastructure.channels.base import BaseChannel
from app.infrastructure.channels.telegram import TelegramChannel

__all__ = [
    "BaseChannel",
    "TelegramChannel",
]
