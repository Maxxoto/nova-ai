"""Pydantic models for session management."""

from datetime import datetime
from typing import Optional, List, Literal
from pydantic import BaseModel, Field, field_validator


class Message(BaseModel):
    """Validated message structure for sessions."""

    role: Literal["user", "assistant", "system"]
    content: str = Field(min_length=1, max_length=50000)
    timestamp: datetime = Field(default_factory=datetime.now)
    tools_used: Optional[List[str]] = Field(default=None)

    @field_validator("content")
    @classmethod
    def content_not_empty(cls, v: str) -> str:
        """Validate content is not just whitespace."""
        if not v.strip():
            raise ValueError("Message content cannot be empty or just whitespace")
        return v.strip()

    class Config:
        json_encoders = {datetime: lambda v: v.isoformat()}


class Session(BaseModel):
    """Validated conversation session."""

    key: str = Field(min_length=1, max_length=200)
    messages: List[Message] = Field(default_factory=list)
    last_consolidated: int = Field(default=0, ge=0)
    created_at: datetime = Field(default_factory=datetime.now)
    updated_at: datetime = Field(default_factory=datetime.now)

    @field_validator("key")
    @classmethod
    def key_format(cls, v: str) -> str:
        """Validate session key format."""
        if ":" not in v:
            raise ValueError("Session key must contain ':' (format: type:id)")
        return v

    def add_message(
        self, role: str, content: str, tools_used: Optional[List[str]] = None
    ) -> None:
        """Add a validated message to the session.

        Args:
            role: Message role (user, assistant, system)
            content: Message content
            tools_used: Optional list of tools used
        """
        message = Message(role=role, content=content, tools_used=tools_used)
        self.messages.append(message)
        self.updated_at = datetime.now()

    def get_history(self, max_messages: Optional[int] = None) -> List[dict]:
        """Get conversation history as list of dicts.

        Args:
            max_messages: Maximum number of recent messages to return

        Returns:
            List of messages as dictionaries (with datetime serialized to ISO strings)
        """
        messages = self.messages
        if max_messages and len(messages) > max_messages:
            messages = messages[-max_messages:]

        # Use mode='json' to serialize datetime to ISO strings
        return [msg.model_dump(mode="json") for msg in messages]

    def clear(self) -> None:
        """Clear all messages in the session."""
        self.messages = []
        self.last_consolidated = 0
        self.updated_at = datetime.now()

    def to_dict(self) -> dict:
        """Convert session to dictionary for serialization.

        Returns:
            Dictionary representation of session (with datetime serialized to ISO strings)
        """
        return {
            "key": self.key,
            "messages": [msg.model_dump(mode="json") for msg in self.messages],
            "last_consolidated": self.last_consolidated,
            "created_at": self.created_at.isoformat(),
            "updated_at": self.updated_at.isoformat(),
        }

    @classmethod
    def from_dict(cls, data: dict) -> "Session":
        """Create session from dictionary.

        Args:
            data: Dictionary containing session data

        Returns:
            Session instance
        """
        messages_data = data.get("messages", [])
        messages = []

        for msg_data in messages_data:
            # Handle timestamp conversion
            if isinstance(msg_data.get("timestamp"), str):
                msg_data["timestamp"] = datetime.fromisoformat(msg_data["timestamp"])
            messages.append(Message(**msg_data))

        created_at = data.get("created_at")
        if isinstance(created_at, str):
            created_at = datetime.fromisoformat(created_at)
        elif created_at is None:
            created_at = datetime.now()

        updated_at = data.get("updated_at")
        if isinstance(updated_at, str):
            updated_at = datetime.fromisoformat(updated_at)
        elif updated_at is None:
            updated_at = datetime.now()

        return cls(
            key=data["key"],
            messages=messages,
            last_consolidated=data.get("last_consolidated", 0),
            created_at=created_at,
            updated_at=updated_at,
        )
