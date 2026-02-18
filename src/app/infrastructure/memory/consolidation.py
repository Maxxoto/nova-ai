"""Memory consolidation logic for archiving old messages."""

import logging
from pathlib import Path

import json_repair

from app.infrastructure.memory.store import MemoryStore
from app.infrastructure.session.manager import Session
from app.domain.ports.llm_client_port import LLMClientPort


logger = logging.getLogger(__name__)


class MemoryConsolidator:
    """Consolidates old session messages into memory."""

    def __init__(
        self,
        workspace: Path,
        provider: LLMClientPort,
        model: str = "groq/openai/gpt-oss-20b",
    ):
        """Initialize memory consolidator.

        Args:
            workspace: Path to workspace directory
            provider: LLM provider for summarization
            model: Model to use for consolidation
        """
        self.workspace = Path(workspace)
        self.provider = provider
        self.model = model
        self.memory_store = MemoryStore(workspace)

    async def consolidate(
        self,
        session: Session,
        keep_count: int = 10,
        archive_all: bool = False,
    ) -> None:
        """Consolidate old messages into memory.

        Args:
            session: Session to consolidate
            keep_count: Number of recent messages to keep
            archive_all: If True, archive all messages (for /new command)
        """
        if archive_all:
            old_messages = session.messages
            logger.info(
                f"Archiving all {len(session.messages)} messages for session {session.key}"
            )
        else:
            if len(session.messages) <= keep_count:
                logger.debug(
                    f"Session {session.key}: No consolidation needed (messages={len(session.messages)}, keep={keep_count})"
                )
                return

            # Get messages to consolidate
            messages_to_process = len(session.messages) - session.last_consolidated
            if messages_to_process <= 0:
                return

            old_messages = session.messages[session.last_consolidated : -keep_count]
            if not old_messages:
                return

            logger.info(
                f"Consolidating {len(old_messages)} messages for session {session.key}"
            )

        # Format conversation
        lines = []
        for msg in old_messages:
            if not msg.get("content"):
                continue

            role = msg.get("role", "unknown")
            timestamp = msg.get("timestamp", "?")[:16]
            tools = (
                f" [tools: {', '.join(msg['tools_used'])}]"
                if msg.get("tools_used")
                else ""
            )
            content = msg.get("content", "")

            lines.append(f"[{timestamp}] {role.upper()}{tools}: {content}")

        conversation = "\n".join(lines)
        current_memory = self.memory_store.read_long_term()

        # Build prompt for LLM
        prompt = f"""You are a memory consolidation agent. Process this conversation and return a JSON object with exactly two keys:

1. "history_entry": A paragraph (2-5 sentences) summarizing the key events/decisions/topics. Start with a timestamp like [YYYY-MM-DD HH:MM]. Include enough detail to be useful when found by grep search later.

2. "memory_update": The updated long-term memory content. Add any new facts: user location, preferences, personal info, habits, project context, technical decisions, tools/services used. If nothing new, return the existing content unchanged.

## Current Long-term Memory
{current_memory or "(empty)"}

## Conversation to Process
{conversation}

Respond with ONLY valid JSON, no markdown fences."""

        try:
            response = await self.provider.chat_completion(
                messages=[
                    {
                        "role": "system",
                        "content": "You are a memory consolidation agent. Respond only with valid JSON.",
                    },
                    {"role": "user", "content": prompt},
                ],
                streaming=False,
            )

            text = (response.get("response") or "").strip()
            if not text:
                logger.warning("Memory consolidation: LLM returned empty response")
                return

            # Clean markdown fences if present
            if text.startswith("```"):
                text = text.split("\n", 1)[-1].rsplit("```", 1)[0].strip()

            # Parse JSON
            result = json_repair.loads(text)
            if not isinstance(result, dict):
                logger.warning("Memory consolidation: unexpected response type")
                return

            # Update history
            if entry := result.get("history_entry"):
                self.memory_store.append_history(entry)

            # Update memory
            if update := result.get("memory_update"):
                if update != current_memory:
                    self.memory_store.write_long_term(update)

            # Update session tracking
            if archive_all:
                session.last_consolidated = 0
                session.messages = []
            else:
                session.last_consolidated = len(session.messages) - keep_count

            logger.info(f"Memory consolidation completed for session {session.key}")

        except Exception as e:
            logger.error(f"Memory consolidation failed: {e}")
