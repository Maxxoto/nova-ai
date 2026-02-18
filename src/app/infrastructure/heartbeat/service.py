"""Heartbeat service for periodic task checking.

Checks HEARTBEAT.md every 30 minutes for recurring tasks.
"""

import asyncio
import logging
from pathlib import Path
from typing import Callable, Optional


logger = logging.getLogger(__name__)

# Constants
DEFAULT_HEARTBEAT_INTERVAL_S = 30 * 60  # 30 minutes
HEARTBEAT_PROMPT = """Read HEARTBEAT.md in your workspace (if it exists).
Follow any instructions or tasks listed there.
If nothing needs attention, reply with just: HEARTBEAT_OK"""
HEARTBEAT_OK_TOKEN = "HEARTBEAT_OK"


class HeartbeatService:
    """Service to periodically check HEARTBEAT.md for tasks."""

    def __init__(
        self,
        workspace: Path,
        on_heartbeat: Callable[[str], str],
        interval_s: int = DEFAULT_HEARTBEAT_INTERVAL_S,
        enabled: bool = True,
    ):
        """Initialize heartbeat service.

        Args:
            workspace: Path to workspace directory
            on_heartbeat: Callback function to execute heartbeat
            interval_s: Interval between heartbeats in seconds (default: 30 min)
            enabled: Whether heartbeat is enabled
        """
        self.workspace = Path(workspace)
        self.heartbeat_file = self.workspace / "HEARTBEAT.md"
        self.on_heartbeat = on_heartbeat
        self.interval_s = interval_s
        self.enabled = enabled
        self._running = False
        self._task: Optional[asyncio.Task] = None

    async def start(self) -> None:
        """Start the heartbeat service."""
        if not self.enabled:
            logger.info("Heartbeat service is disabled")
            return

        self._running = True
        self._task = asyncio.create_task(self._run_loop())
        logger.info(f"Heartbeat service started (interval: {self.interval_s}s)")

    async def stop(self) -> None:
        """Stop the heartbeat service."""
        self._running = False
        if self._task:
            self._task.cancel()
            try:
                await self._task
            except asyncio.CancelledError:
                pass
        logger.info("Heartbeat service stopped")

    async def _run_loop(self) -> None:
        """Main heartbeat loop."""
        while self._running:
            try:
                await self._check_heartbeat()
                await asyncio.sleep(self.interval_s)
            except asyncio.CancelledError:
                break
            except Exception as e:
                logger.error(f"Error in heartbeat loop: {e}")
                await asyncio.sleep(self.interval_s)

    async def _check_heartbeat(self) -> None:
        """Check HEARTBEAT.md and execute if needed."""
        if not self.heartbeat_file.exists():
            logger.debug("HEARTBEAT.md not found, skipping")
            return

        try:
            content = self.heartbeat_file.read_text(encoding="utf-8")

            # Check if file has actionable content (skip empty, headers only, comments)
            if not self._has_actionable_content(content):
                logger.debug("HEARTBEAT.md has no actionable content")
                return

            logger.info("Executing heartbeat check")

            # Execute heartbeat through callback
            response = await self.on_heartbeat(HEARTBEAT_PROMPT)

            if HEARTBEAT_OK_TOKEN in response:
                logger.debug("Heartbeat check completed: no action needed")
            else:
                logger.info(
                    f"Heartbeat executed tasks, response length: {len(response)}"
                )

        except Exception as e:
            logger.error(f"Error checking heartbeat: {e}")

    def _has_actionable_content(self, content: str) -> bool:
        """Check if heartbeat file has actionable tasks.

        Args:
            content: File content

        Returns:
            True if there are actionable tasks
        """
        lines = content.strip().split("\n")

        for line in lines:
            line = line.strip()
            # Skip empty lines, headers, comments
            if not line:
                continue
            if line.startswith("#"):
                continue
            if line.startswith("<!--"):
                continue
            if line.startswith("-->"):
                continue
            # Check for task markers
            if line.startswith("- [ ]") or line.startswith("- [x]"):
                return True
            # Any non-header content is actionable
            return True

        return False

    def trigger_now(self) -> None:
        """Manually trigger a heartbeat check."""
        if self._running:
            asyncio.create_task(self._check_heartbeat())

    @property
    def is_running(self) -> bool:
        """Check if heartbeat service is running."""
        return self._running
