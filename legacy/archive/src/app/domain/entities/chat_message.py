"""Chat Message Entity for hexagonal architecture."""

from typing import List
from pydantic import BaseModel


class ChatMessage(BaseModel):
    """Chat message entity."""

    role: str
    content: str


class ChatCompletionRequest(BaseModel):
    """Chat completion request entity."""

    messages: List[ChatMessage]
    thread_id: str = None
