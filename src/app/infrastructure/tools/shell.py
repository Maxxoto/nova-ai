"""Shell execution tool with safety guards."""

import asyncio
import os
import shlex
from pathlib import Path
from typing import Any, Optional

from app.infrastructure.tools.base import Tool


class ExecTool(Tool):
    """Tool to execute shell commands with safety guards."""

    # Dangerous commands that are blocked by default
    DANGEROUS_PATTERNS = [
        "rm -rf /",
        "rm -rf /*",
        "rm -rf ~",
        "> /dev/sda",
        "mkfs.",
        "dd if=",
        ":(){ :|:& };:",
        "chmod -R 777 /",
        "chown -R",
    ]

    def __init__(
        self,
        working_dir: Optional[str] = None,
        timeout: int = 30,
        restrict_to_workspace: bool = False,
        allowed_dir: Optional[Path] = None,
    ):
        """Initialize shell execution tool.

        Args:
            working_dir: Default working directory for commands
            timeout: Command timeout in seconds
            restrict_to_workspace: If True, restrict to workspace directory
            allowed_dir: If set, restrict to this directory
        """
        self.working_dir = working_dir or os.getcwd()
        self.timeout = timeout
        self.restrict_to_workspace = restrict_to_workspace
        self.allowed_dir = allowed_dir or (Path(working_dir) if working_dir else None)

    @property
    def name(self) -> str:
        return "exec"

    @property
    def description(self) -> str:
        desc = "Execute a shell command. Use with caution."
        if self.restrict_to_workspace and self.allowed_dir:
            desc += f" Restricted to: {self.allowed_dir}"
        return desc

    @property
    def parameters(self) -> dict[str, Any]:
        return {
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "Shell command to execute",
                },
                "cwd": {
                    "type": "string",
                    "description": "Working directory (optional, defaults to workspace)",
                },
            },
            "required": ["command"],
        }

    def _is_dangerous(self, command: str) -> bool:
        """Check if command contains dangerous patterns."""
        cmd_lower = command.lower()
        for pattern in self.DANGEROUS_PATTERNS:
            if pattern.lower() in cmd_lower:
                return True
        return False

    def _validate_cwd(self, cwd: Optional[str]) -> str:
        """Validate and resolve working directory."""
        if cwd:
            resolved = Path(cwd).expanduser().resolve()
        else:
            resolved = Path(self.working_dir).resolve()

        if self.restrict_to_workspace and self.allowed_dir:
            allowed = self.allowed_dir.resolve()
            try:
                resolved.relative_to(allowed)
            except ValueError:
                raise PermissionError(
                    f"Access denied: {cwd or resolved} is outside allowed directory {self.allowed_dir}"
                )

        return str(resolved)

    async def execute(self, command: str, cwd: Optional[str] = None) -> str:
        """Execute shell command.

        Args:
            command: Shell command to execute
            cwd: Working directory (optional)

        Returns:
            Command output or error
        """
        # Check for dangerous commands
        if self._is_dangerous(command):
            return f"Error: Command blocked for safety: {command}"

        try:
            working_dir = self._validate_cwd(cwd)

            # Create subprocess
            process = await asyncio.create_subprocess_shell(
                command,
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.PIPE,
                cwd=working_dir,
            )

            # Wait for completion with timeout
            try:
                stdout, stderr = await asyncio.wait_for(
                    process.communicate(), timeout=self.timeout
                )
            except asyncio.TimeoutError:
                process.kill()
                return f"Error: Command timed out after {self.timeout} seconds"

            # Decode output
            stdout_str = stdout.decode("utf-8", errors="replace") if stdout else ""
            stderr_str = stderr.decode("utf-8", errors="replace") if stderr else ""

            # Build result
            result_parts = []
            if stdout_str:
                result_parts.append(f"STDOUT:\n{stdout_str}")
            if stderr_str:
                result_parts.append(f"STDERR:\n{stderr_str}")

            if process.returncode != 0:
                result_parts.append(f"Exit code: {process.returncode}")

            if not result_parts:
                return "Command executed successfully (no output)"

            return "\n\n".join(result_parts)

        except PermissionError as e:
            return f"Error: {str(e)}"
        except Exception as e:
            return f"Error executing command: {str(e)}"
