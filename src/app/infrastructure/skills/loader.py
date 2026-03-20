"""Skills loader for markdown-based skills with YAML frontmatter and Pydantic validation."""

import logging
from pathlib import Path
from typing import Optional

import frontmatter
import yaml

from app.infrastructure.skills.models import Skill, SkillRequirements, SkillMetadata


logger = logging.getLogger(__name__)


class SkillsLoader:
    """Loader for markdown-based skills with Pydantic validation."""

    def __init__(self, workspace: Path):
        """Initialize skills loader.

        Args:
            workspace: Path to workspace directory
        """
        self.workspace = Path(workspace)
        self.builtin_path = Path(__file__).parent / "builtin"
        self.user_path = self.workspace / "skills"

        # Cache for loaded skills
        self._cached_skills: Optional[list[Skill]] = None

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

            # Get skill info with proper type handling
            name = str(metadata.get("name", skill_path.parent.name))
            description = str(metadata.get("description", ""))
            always_load = bool(metadata.get("always_load", False))
            requirements_raw = metadata.get("requirements", {})

            # Content is the markdown body
            content = post.content

            # Build structured metadata with type safety
            raw_tags = metadata.get("tags", [])
            tags_list = list(raw_tags) if isinstance(raw_tags, (list, tuple)) else []

            skill_metadata = SkillMetadata(
                author=str(metadata.get("author")) if metadata.get("author") else None,
                version=str(metadata.get("version", "1.0.0")),
                tags=[str(t) for t in tags_list],
                created=str(metadata.get("created"))
                if metadata.get("created")
                else None,
                updated=str(metadata.get("updated"))
                if metadata.get("updated")
                else None,
            )

            # Build structured requirements with type safety
            reqs = requirements_raw if isinstance(requirements_raw, dict) else {}
            skill_requirements = SkillRequirements(
                tools=[str(t) for t in reqs.get("tools", [])],
                apis=[str(a) for a in reqs.get("apis", [])],
                skills=[str(s) for s in reqs.get("skills", [])],
            )

            return Skill(
                name=name,
                description=description,
                content=content,
                always_load=always_load,
                requirements=skill_requirements,
                metadata=skill_metadata,
                source=skill_path,
            )

        except Exception as e:
            logger.error(f"Error loading skill from {skill_path}: {e}")
            return None

    def load_all(self, force_reload: bool = False) -> list[Skill]:
        """Load all skills from user directory.

        Args:
            force_reload: If True, bypass cache and reload from disk

        Returns:
            List of loaded skills
        """
        # Return cached skills if available and not forcing reload
        if self._cached_skills is not None and not force_reload:
            return self._cached_skills

        skills = []

        if not self.user_path.exists():
            self._cached_skills = skills
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
        self._cached_skills = skills
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

    Note: This only checks for required skills. Tool and API availability
    must be checked by the caller with access to tool registry and env vars.

    Args:
        skill: Skill to check

    Returns:
        Tuple of (is_valid, list_of_errors)
    """
    errors = []

    # Check for required skills
    for required_skill in skill.requirements.skills:
        if not self.get_skill(required_skill):
            errors.append(f"Required skill not found: {required_skill}")

    return len(errors) == 0, errors
