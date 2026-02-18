"""Skills loader for markdown-based skills with YAML frontmatter."""

import logging
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Optional

import frontmatter
import yaml


logger = logging.getLogger(__name__)


@dataclass
class Skill:
    """Represents a loaded skill."""

    name: str
    description: str
    content: str
    always_load: bool
    requirements: dict
    metadata: dict
    source: Path


class SkillsLoader:
    """Loader for markdown-based skills."""

    def __init__(self, workspace: Path):
        """Initialize skills loader.

        Args:
            workspace: Path to workspace directory
        """
        self.workspace = Path(workspace)
        self.builtin_path = Path(__file__).parent / "builtin"
        self.user_path = self.workspace / "skills"

        # Ensure directories exist
        self._ensure_directories()

    def _ensure_directories(self) -> None:
        """Create skills directories if they don't exist."""
        self.user_path.mkdir(parents=True, exist_ok=True)

        # Copy built-in skills if not exists
        if self.builtin_path.exists():
            for skill_dir in self.builtin_path.iterdir():
                if skill_dir.is_dir() and skill_dir.name != "bootstrap":
                    target = self.user_path / skill_dir.name
                    if not target.exists():
                        import shutil

                        shutil.copytree(skill_dir, target)
                        logger.info(f"Copied built-in skill: {skill_dir.name}")

        # Copy bootstrap files to workspace root
        self._copy_bootstrap_files()

    def _copy_bootstrap_files(self) -> None:
        """Copy bootstrap template files to workspace."""
        bootstrap_path = self.builtin_path / "bootstrap"
        if not bootstrap_path.exists():
            return

        for template_file in bootstrap_path.iterdir():
            if template_file.is_file() and template_file.suffix == ".md":
                target = self.workspace / template_file.name
                if not target.exists():
                    import shutil

                    shutil.copy2(template_file, target)
                    logger.info(f"Copied bootstrap file: {template_file.name}")

    def _load_skill_file(self, skill_path: Path) -> Optional[Skill]:
        """Load a single skill from markdown file.

        Args:
            skill_path: Path to skill markdown file

        Returns:
            Skill instance or None if invalid
        """
        try:
            # Read frontmatter
            post = frontmatter.load(str(skill_path))

            # Extract metadata
            metadata = post.metadata

            # Get skill info
            name = metadata.get("name", skill_path.parent.name)
            description = metadata.get("description", "")
            always_load = metadata.get("always_load", False)
            requirements = metadata.get("requirements", {})

            # Content is the markdown body
            content = post.content

            return Skill(
                name=name,
                description=description,
                content=content,
                always_load=always_load,
                requirements=requirements,
                metadata=metadata,
                source=skill_path,
            )

        except Exception as e:
            logger.error(f"Error loading skill from {skill_path}: {e}")
            return None

    def load_all(self) -> list[Skill]:
        """Load all skills from user directory.

        Returns:
            List of loaded skills
        """
        skills = []

        if not self.user_path.exists():
            return skills

        for skill_dir in self.user_path.iterdir():
            if not skill_dir.is_dir():
                continue

            # Look for SKILL.md
            skill_file = skill_dir / "SKILL.md"
            if skill_file.exists():
                skill = self._load_skill_file(skill_file)
                if skill:
                    skills.append(skill)
                    logger.debug(f"Loaded skill: {skill.name}")

        logger.info(f"Loaded {len(skills)} skills")
        return skills

    def get_skill(self, name: str) -> Optional[Skill]:
        """Get a specific skill by name.

        Args:
            name: Skill name

        Returns:
            Skill instance or None
        """
        skill_file = self.user_path / name / "SKILL.md"
        if skill_file.exists():
            return self._load_skill_file(skill_file)
        return None

    def get_always_loaded(self) -> list[Skill]:
        """Get skills that should always be loaded.

        Returns:
            List of always-loaded skills
        """
        all_skills = self.load_all()
        return [s for s in all_skills if s.always_load]

    def get_available(self) -> list[Skill]:
        """Get skills that are available but not always loaded.

        Returns:
            List of available skills
        """
        all_skills = self.load_all()
        return [s for s in all_skills if not s.always_load]

    def check_requirements(self, skill: Skill) -> tuple[bool, list[str]]:
        """Check if skill requirements are met.

        Args:
            skill: Skill to check

        Returns:
            Tuple of (is_valid, list_of_errors)
        """
        errors = []

        # Check for required binaries
        bins = skill.requirements.get("bins", [])
        for binary in bins:
            import shutil

            if not shutil.which(binary):
                errors.append(f"Required binary not found: {binary}")

        # Check for required environment variables
        env_vars = skill.requirements.get("env", [])
        import os

        for env_var in env_vars:
            if not os.getenv(env_var):
                errors.append(f"Required environment variable not set: {env_var}")

        return len(errors) == 0, errors
