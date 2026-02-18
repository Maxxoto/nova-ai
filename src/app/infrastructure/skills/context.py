"""Context builder for assembling prompts with memory and skills."""

import logging
from pathlib import Path
from typing import Optional

from app.infrastructure.memory.store import MemoryStore
from app.infrastructure.skills.loader import SkillsLoader, Skill


logger = logging.getLogger(__name__)


class ContextBuilder:
    """Builds prompts with bootstrap files, memory, and skills."""

    def __init__(self, workspace: Path):
        """Initialize context builder.

        Args:
            workspace: Path to workspace directory
        """
        self.workspace = Path(workspace)
        self.memory_store = MemoryStore(workspace)
        self.skills_loader = SkillsLoader(workspace)
        self.bootstrap_dir = self.workspace  # Bootstrap files are in workspace root

    def _load_bootstrap_file(self, filename: str) -> str:
        """Load a bootstrap file if it exists.

        Args:
            filename: Name of bootstrap file

        Returns:
            File contents or empty string
        """
        filepath = self.bootstrap_dir / filename
        if filepath.exists():
            try:
                return filepath.read_text(encoding="utf-8")
            except Exception as e:
                logger.error(f"Error reading bootstrap file {filename}: {e}")
        return ""

    def _format_skills_context(self, skills: list[Skill]) -> str:
        """Format skills for context.

        Args:
            skills: List of skills

        Returns:
            Formatted skills context
        """
        if not skills:
            return ""

        parts = ["## Skills"]
        for skill in skills:
            parts.append(f"\n### {skill.name}")
            parts.append(f"{skill.description}")
            parts.append(skill.content)

        return "\n\n".join(parts)

    def _format_available_skills(self, skills: list[Skill]) -> str:
        """Format available skills summary.

        Args:
            skills: List of available skills

        Returns:
            Formatted summary
        """
        if not skills:
            return ""

        parts = ["\n## Available Skills"]
        parts.append("You can use these skills by reading their documentation:")

        for skill in skills:
            parts.append(f"- **{skill.name}**: {skill.description}")

        return "\n".join(parts)

    def build_system_prompt(
        self,
        include_memory: bool = True,
        include_skills: bool = True,
    ) -> str:
        """Build the system prompt with bootstrap, memory, and skills.

        Args:
            include_memory: Whether to include long-term memory
            include_skills: Whether to include skills

        Returns:
            Complete system prompt
        """
        parts = []

        # Load bootstrap files (in order of importance)
        soul = self._load_bootstrap_file("SOUL.md")
        agents = self._load_bootstrap_file("AGENTS.md")
        user = self._load_bootstrap_file("USER.md")
        tools = self._load_bootstrap_file("TOOLS.md")

        # Add SOUL first (core identity)
        if soul:
            parts.append(soul)
        else:
            parts.append("# Nova 🤖")
            parts.append("You are Nova, a helpful AI assistant.")

        # Add agents config (instructions)
        if agents:
            parts.append("\n" + agents)

        # Add user info
        if user:
            parts.append("\n## User Profile")
            parts.append(user)

        # Add memory
        if include_memory:
            memory = self.memory_store.read_long_term()
            if memory and memory.strip():
                parts.append("\n## Long-term Memory")
                parts.append("Use this context to remember user preferences and facts:")
                parts.append(memory)

        # Add tools info
        if tools:
            parts.append("\n" + tools)
        else:
            parts.append("\n## Available Tools")
            parts.append("You have access to tools. Use them when appropriate.")

        # Add always-loaded skills
        if include_skills:
            always_loaded = self.skills_loader.get_always_loaded()
            if always_loaded:
                parts.append(self._format_skills_context(always_loaded))

            # Add available skills summary
            available = self.skills_loader.get_available()
            if available:
                parts.append(self._format_available_skills(available))

        return "\n\n".join(parts)

    def build_messages(
        self,
        history: list[dict],
        current_message: str,
        user_id: Optional[str] = None,
        include_memory: bool = True,
        include_skills: bool = True,
    ) -> list[dict]:
        """Build complete message list for LLM.

        Args:
            history: Conversation history
            current_message: Current user message
            user_id: Optional user ID
            include_memory: Whether to include long-term memory
            include_skills: Whether to include skills

        Returns:
            List of messages for LLM
        """
        messages = []

        # Build system prompt
        system_prompt = self.build_system_prompt(
            include_memory=include_memory,
            include_skills=include_skills,
        )
        messages.append({"role": "system", "content": system_prompt})

        # Add history
        for msg in history:
            role = msg.get("role", "user")
            content = msg.get("content", "")
            if role in ["user", "assistant", "system"]:
                messages.append({"role": role, "content": content})

        # Add current message
        messages.append({"role": "user", "content": current_message})

        return messages
