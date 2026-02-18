"""Filesystem tools for file operations."""

import os
from pathlib import Path
from typing import Any, Optional

from app.infrastructure.tools.base import Tool


class ReadFileTool(Tool):
    """Tool to read file contents."""

    def __init__(self, allowed_dir: Optional[Path] = None):
        """Initialize read file tool.

        Args:
            allowed_dir: If set, restrict file access to this directory
        """
        self.allowed_dir = allowed_dir

    @property
    def name(self) -> str:
        return "read_file"

    @property
    def description(self) -> str:
        desc = "Read the contents of a file."
        if self.allowed_dir:
            desc += f" Access restricted to: {self.allowed_dir}"
        return desc

    @property
    def parameters(self) -> dict[str, Any]:
        return {
            "type": "object",
            "properties": {
                "path": {"type": "string", "description": "Path to the file to read"}
            },
            "required": ["path"],
        }

    def _resolve_path(self, path: str) -> Path:
        """Resolve and validate path."""
        resolved = Path(path).expanduser().resolve()

        if self.allowed_dir:
            allowed = self.allowed_dir.resolve()
            try:
                resolved.relative_to(allowed)
            except ValueError:
                raise PermissionError(
                    f"Access denied: {path} is outside allowed directory {self.allowed_dir}"
                )

        return resolved

    async def execute(self, path: str) -> str:
        """Read file contents.

        Args:
            path: Path to the file

        Returns:
            File contents or error message
        """
        try:
            file_path = self._resolve_path(path)

            if not file_path.exists():
                return f"Error: File not found: {path}"

            if not file_path.is_file():
                return f"Error: Path is not a file: {path}"

            content = file_path.read_text(encoding="utf-8")
            return content

        except PermissionError as e:
            return f"Error: {str(e)}"
        except Exception as e:
            return f"Error reading file: {str(e)}"


class WriteFileTool(Tool):
    """Tool to write/create files."""

    def __init__(self, allowed_dir: Optional[Path] = None):
        """Initialize write file tool.

        Args:
            allowed_dir: If set, restrict file access to this directory
        """
        self.allowed_dir = allowed_dir

    @property
    def name(self) -> str:
        return "write_file"

    @property
    def description(self) -> str:
        desc = "Write or create a file with the given content."
        if self.allowed_dir:
            desc += f" Access restricted to: {self.allowed_dir}"
        return desc

    @property
    def parameters(self) -> dict[str, Any]:
        return {
            "type": "object",
            "properties": {
                "path": {"type": "string", "description": "Path to the file to write"},
                "content": {
                    "type": "string",
                    "description": "Content to write to the file",
                },
            },
            "required": ["path", "content"],
        }

    def _resolve_path(self, path: str) -> Path:
        """Resolve and validate path."""
        resolved = Path(path).expanduser().resolve()

        if self.allowed_dir:
            allowed = self.allowed_dir.resolve()
            try:
                resolved.relative_to(allowed)
            except ValueError:
                raise PermissionError(
                    f"Access denied: {path} is outside allowed directory {self.allowed_dir}"
                )

        return resolved

    async def execute(self, path: str, content: str) -> str:
        """Write file contents.

        Args:
            path: Path to the file
            content: Content to write

        Returns:
            Success message or error
        """
        try:
            file_path = self._resolve_path(path)

            # Create parent directories if they don't exist
            file_path.parent.mkdir(parents=True, exist_ok=True)

            file_path.write_text(content, encoding="utf-8")
            return f"Successfully wrote to {path}"

        except PermissionError as e:
            return f"Error: {str(e)}"
        except Exception as e:
            return f"Error writing file: {str(e)}"


class EditFileTool(Tool):
    """Tool to edit/replace text in files."""

    def __init__(self, allowed_dir: Optional[Path] = None):
        """Initialize edit file tool.

        Args:
            allowed_dir: If set, restrict file access to this directory
        """
        self.allowed_dir = allowed_dir

    @property
    def name(self) -> str:
        return "edit_file"

    @property
    def description(self) -> str:
        desc = "Replace text in a file. Use \\n for newlines in the replacement."
        if self.allowed_dir:
            desc += f" Access restricted to: {self.allowed_dir}"
        return desc

    @property
    def parameters(self) -> dict[str, Any]:
        return {
            "type": "object",
            "properties": {
                "path": {"type": "string", "description": "Path to the file to edit"},
                "old_string": {"type": "string", "description": "Text to replace"},
                "new_string": {"type": "string", "description": "Replacement text"},
            },
            "required": ["path", "old_string", "new_string"],
        }

    def _resolve_path(self, path: str) -> Path:
        """Resolve and validate path."""
        resolved = Path(path).expanduser().resolve()

        if self.allowed_dir:
            allowed = self.allowed_dir.resolve()
            try:
                resolved.relative_to(allowed)
            except ValueError:
                raise PermissionError(
                    f"Access denied: {path} is outside allowed directory {self.allowed_dir}"
                )

        return resolved

    async def execute(self, path: str, old_string: str, new_string: str) -> str:
        """Edit file contents.

        Args:
            path: Path to the file
            old_string: Text to replace
            new_string: Replacement text

        Returns:
            Success message or error
        """
        try:
            file_path = self._resolve_path(path)

            if not file_path.exists():
                return f"Error: File not found: {path}"

            content = file_path.read_text(encoding="utf-8")

            if old_string not in content:
                return f"Error: Could not find text to replace in {path}"

            new_content = content.replace(old_string, new_string, 1)
            file_path.write_text(new_content, encoding="utf-8")

            return f"Successfully edited {path}"

        except PermissionError as e:
            return f"Error: {str(e)}"
        except Exception as e:
            return f"Error editing file: {str(e)}"


class ListDirTool(Tool):
    """Tool to list directory contents."""

    def __init__(self, allowed_dir: Optional[Path] = None):
        """Initialize list directory tool.

        Args:
            allowed_dir: If set, restrict file access to this directory
        """
        self.allowed_dir = allowed_dir

    @property
    def name(self) -> str:
        return "list_dir"

    @property
    def description(self) -> str:
        desc = "List the contents of a directory."
        if self.allowed_dir:
            desc += f" Access restricted to: {self.allowed_dir}"
        return desc

    @property
    def parameters(self) -> dict[str, Any]:
        return {
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the directory to list",
                }
            },
            "required": ["path"],
        }

    def _resolve_path(self, path: str) -> Path:
        """Resolve and validate path."""
        resolved = Path(path).expanduser().resolve()

        if self.allowed_dir:
            allowed = self.allowed_dir.resolve()
            try:
                resolved.relative_to(allowed)
            except ValueError:
                raise PermissionError(
                    f"Access denied: {path} is outside allowed directory {self.allowed_dir}"
                )

        return resolved

    async def execute(self, path: str) -> str:
        """List directory contents.

        Args:
            path: Path to the directory

        Returns:
            Directory listing or error
        """
        try:
            dir_path = self._resolve_path(path)

            if not dir_path.exists():
                return f"Error: Directory not found: {path}"

            if not dir_path.is_dir():
                return f"Error: Path is not a directory: {path}"

            entries = []
            for entry in sorted(dir_path.iterdir()):
                entry_type = "📁" if entry.is_dir() else "📄"
                size = ""
                if entry.is_file():
                    size_bytes = entry.stat().st_size
                    if size_bytes < 1024:
                        size = f" ({size_bytes}B)"
                    elif size_bytes < 1024 * 1024:
                        size = f" ({size_bytes // 1024}KB)"
                    else:
                        size = f" ({size_bytes // (1024 * 1024)}MB)"
                entries.append(f"{entry_type} {entry.name}{size}")

            if not entries:
                return f"Directory {path} is empty"

            return "\n".join(entries)

        except PermissionError as e:
            return f"Error: {str(e)}"
        except Exception as e:
            return f"Error listing directory: {str(e)}"
