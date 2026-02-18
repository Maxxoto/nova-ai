"""File-based markdown memory system with two-layer storage.

Two-layer memory system:
- MEMORY.md: Long-term facts and preferences (loaded into context)
- HISTORY.md: Searchable archive of past conversations (not loaded, searched on demand)
"""

import logging
from datetime import datetime
from pathlib import Path
from typing import Optional


logger = logging.getLogger(__name__)


class MemoryStore:
    """Two-layer memory storage using markdown files."""

    def __init__(self, workspace: Path):
        """Initialize memory store.

        Args:
            workspace: Path to workspace directory
        """
        self.workspace = Path(workspace)
        self.memory_dir = self.workspace / "memory"
        self.memory_file = self.memory_dir / "MEMORY.md"
        self.history_file = self.memory_dir / "HISTORY.md"

        # Ensure directories exist
        self._ensure_directories()

    def _ensure_directories(self) -> None:
        """Create memory directories if they don't exist."""
        self.memory_dir.mkdir(parents=True, exist_ok=True)

        # Create empty files if they don't exist
        if not self.memory_file.exists():
            self.memory_file.write_text(
                "# Long-term Memory\n\n"
                "This file contains long-term facts and preferences.\n\n"
                "## User Information\n\n"
                "## Preferences\n\n"
                "## Projects\n\n"
            )

        if not self.history_file.exists():
            self.history_file.write_text(
                "# Conversation History\n\n"
                "This file contains a searchable archive of past conversations.\n\n"
            )

    def read_long_term(self) -> str:
        """Read long-term memory (MEMORY.md).

        Returns:
            Contents of MEMORY.md
        """
        try:
            return self.memory_file.read_text(encoding="utf-8")
        except Exception as e:
            logger.error(f"Error reading memory file: {e}")
            return ""

    def write_long_term(self, content: str) -> None:
        """Write long-term memory (MEMORY.md).

        Args:
            content: New content for MEMORY.md
        """
        try:
            self.memory_file.write_text(content, encoding="utf-8")
            logger.info("Updated long-term memory")
        except Exception as e:
            logger.error(f"Error writing memory file: {e}")

    def append_history(self, entry: str) -> None:
        """Append entry to history (HISTORY.md).

        Args:
            entry: History entry to append
        """
        try:
            timestamp = datetime.now().strftime("%Y-%m-%d %H:%M")
            formatted_entry = f"\n## [{timestamp}]\n\n{entry}\n"

            with open(self.history_file, "a", encoding="utf-8") as f:
                f.write(formatted_entry)

            logger.info("Appended to history")
        except Exception as e:
            logger.error(f"Error appending to history: {e}")

    def search_history(self, query: str) -> list[str]:
        """Search history for matching entries.

        Args:
            query: Search query

        Returns:
            List of matching history entries
        """
        try:
            content = self.history_file.read_text(encoding="utf-8")
            lines = content.split("\n")

            matches = []
            current_entry = []
            in_entry = False

            for line in lines:
                if line.startswith("## ["):
                    # Start of new entry
                    if in_entry and current_entry:
                        entry_text = "\n".join(current_entry)
                        if query.lower() in entry_text.lower():
                            matches.append(entry_text)
                    current_entry = [line]
                    in_entry = True
                elif in_entry:
                    current_entry.append(line)

            # Don't forget the last entry
            if in_entry and current_entry:
                entry_text = "\n".join(current_entry)
                if query.lower() in entry_text.lower():
                    matches.append(entry_text)

            return matches

        except Exception as e:
            logger.error(f"Error searching history: {e}")
            return []

    def update_section(self, section: str, content: str) -> None:
        """Update a specific section in MEMORY.md.

        Args:
            section: Section name (e.g., "User Information")
            content: New content for the section
        """
        try:
            current = self.read_long_term()

            # Find section
            section_header = f"## {section}"
            if section_header not in current:
                # Add section if it doesn't exist
                current += f"\n## {section}\n\n{content}\n"
            else:
                # Replace section content
                parts = current.split(section_header)
                if len(parts) == 2:
                    before = parts[0]
                    after_parts = parts[1].split("\n## ", 1)
                    after = "\n## " + after_parts[1] if len(after_parts) > 1 else ""
                    current = before + section_header + "\n\n" + content + "\n" + after

            self.write_long_term(current)

        except Exception as e:
            logger.error(f"Error updating section: {e}")
