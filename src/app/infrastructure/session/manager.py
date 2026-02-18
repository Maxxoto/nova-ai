"""Session management for conversation persistence."""

import json
import logging
from dataclasses import dataclass, field
from datetime import datetime
from pathlib import Path
from typing import Any, Optional


logger = logging.getLogger(__name__)


@dataclass
class Session:
    """Conversation session."""

    key: str  # Format: "user_id" or "channel:chat_id"
    messages: list[dict] = field(default_factory=list)
    last_consolidated: int = 0  # Index of last archived message
    created_at: str = field(default_factory=lambda: datetime.now().isoformat())
    updated_at: str = field(default_factory=lambda: datetime.now().isoformat())

    def add_message(
        self, role: str, content: str, tools_used: Optional[list[str]] = None
    ) -> None:
        """Add a message to the session.

        Args:
            role: Message role (user, assistant, system)
            content: Message content
            tools_used: Optional list of tools used
        """
        message = {
            "role": role,
            "content": content,
            "timestamp": datetime.now().isoformat(),
        }
        if tools_used:
            message["tools_used"] = tools_used

        self.messages.append(message)
        self.updated_at = datetime.now().isoformat()

    def get_history(self, max_messages: Optional[int] = None) -> list[dict]:
        """Get conversation history.

        Args:
            max_messages: Maximum number of recent messages to return

        Returns:
            List of messages
        """
        if max_messages and len(self.messages) > max_messages:
            return self.messages[-max_messages:]
        return self.messages.copy()

    def clear(self) -> None:
        """Clear all messages."""
        self.messages = []
        self.last_consolidated = 0
        self.updated_at = datetime.now().isoformat()

    def to_dict(self) -> dict:
        """Convert session to dictionary."""
        return {
            "key": self.key,
            "messages": self.messages,
            "last_consolidated": self.last_consolidated,
            "created_at": self.created_at,
            "updated_at": self.updated_at,
        }

    @classmethod
    def from_dict(cls, data: dict) -> "Session":
        """Create session from dictionary."""
        return cls(
            key=data["key"],
            messages=data.get("messages", []),
            last_consolidated=data.get("last_consolidated", 0),
            created_at=data.get("created_at", datetime.now().isoformat()),
            updated_at=data.get("updated_at", datetime.now().isoformat()),
        )


class SessionManager:
    """Manager for conversation sessions."""

    def __init__(self, workspace: Path):
        """Initialize session manager.

        Args:
            workspace: Path to workspace directory
        """
        self.workspace = Path(workspace)
        self.sessions_dir = self.workspace / "sessions"

        # Ensure directory exists
        self.sessions_dir.mkdir(parents=True, exist_ok=True)

        # In-memory cache
        self._cache: dict[str, Session] = {}

    def _get_session_path(self, key: str) -> Path:
        """Get file path for session.

        Args:
            key: Session key

        Returns:
            Path to session file
        """
        # Sanitize key for filename
        safe_key = key.replace(":", "_").replace("/", "_")
        return self.sessions_dir / f"{safe_key}.json"

    def get_or_create(self, key: str) -> Session:
        """Get existing session or create new one.

        Args:
            key: Session key

        Returns:
            Session instance
        """
        # Check cache first
        if key in self._cache:
            return self._cache[key]

        # Try to load from disk
        session_path = self._get_session_path(key)
        if session_path.exists():
            try:
                data = json.loads(session_path.read_text(encoding="utf-8"))
                session = Session.from_dict(data)
                self._cache[key] = session
                logger.info(f"Loaded session: {key}")
                return session
            except Exception as e:
                logger.error(f"Error loading session {key}: {e}")

        # Create new session
        session = Session(key=key)
        self._cache[key] = session
        logger.info(f"Created new session: {key}")
        return session

    def save(self, session: Session) -> None:
        """Save session to disk.

        Args:
            session: Session to save
        """
        try:
            session_path = self._get_session_path(session.key)
            session.updated_at = datetime.now().isoformat()
            session_path.write_text(
                json.dumps(session.to_dict(), indent=2, ensure_ascii=False),
                encoding="utf-8",
            )
            logger.debug(f"Saved session: {session.key}")
        except Exception as e:
            logger.error(f"Error saving session {session.key}: {e}")

    def invalidate(self, key: str) -> None:
        """Remove session from cache.

        Args:
            key: Session key
        """
        if key in self._cache:
            del self._cache[key]
            logger.debug(f"Invalidated session from cache: {key}")

    def list_sessions(self) -> list[str]:
        """List all session keys.

        Returns:
            List of session keys
        """
        sessions = []
        for session_file in self.sessions_dir.glob("*.json"):
            key = session_file.stem.replace("_", ":", 1)
            sessions.append(key)
        return sessions

    def delete_session(self, key: str) -> bool:
        """Delete a session.

        Args:
            key: Session key

        Returns:
            True if deleted, False otherwise
        """
        try:
            session_path = self._get_session_path(key)
            if session_path.exists():
                session_path.unlink()

            if key in self._cache:
                del self._cache[key]

            logger.info(f"Deleted session: {key}")
            return True

        except Exception as e:
            logger.error(f"Error deleting session {key}: {e}")
            return False
