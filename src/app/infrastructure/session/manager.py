"""Session management for conversation persistence with Pydantic validation."""

import json
import logging
from datetime import datetime
from pathlib import Path
from typing import Optional

from app.infrastructure.session.models import Session


logger = logging.getLogger(__name__)


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
            session.updated_at = datetime.now()
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
        sessions: list[str] = []
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
