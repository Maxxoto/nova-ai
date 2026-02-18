"""Cron tool for agent to schedule reminders."""

import logging
from pathlib import Path
from typing import Any, Optional

from app.infrastructure.tools.base import Tool


logger = logging.getLogger(__name__)


class CronTool(Tool):
    """Tool to manage scheduled cron jobs."""

    def __init__(self, cron_service=None):
        """Initialize cron tool.

        Args:
            cron_service: Optional CronService instance
        """
        self.cron_service = cron_service

    @property
    def name(self) -> str:
        return "cron"

    @property
    def description(self) -> str:
        return """Manage scheduled reminders and tasks. Use this to:
- Add one-time reminders at specific times
- Add recurring reminders using cron expressions
- Add interval-based reminders (every X seconds)
- List or remove scheduled jobs"""

    @property
    def parameters(self) -> dict[str, Any]:
        return {
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["add", "remove", "list"],
                    "description": "Action to perform: add, remove, or list jobs",
                },
                "name": {
                    "type": "string",
                    "description": "Job name (required for add)",
                },
                "message": {
                    "type": "string",
                    "description": "Message to deliver (required for add)",
                },
                "schedule_type": {
                    "type": "string",
                    "enum": ["once", "cron", "interval"],
                    "description": "Schedule type: once (at specific time), cron (cron expression), or interval (every X seconds)",
                },
                "at": {
                    "type": "string",
                    "description": "ISO datetime for one-time jobs (e.g., '2025-01-31T15:00:00')",
                },
                "cron": {
                    "type": "string",
                    "description": "Cron expression for recurring jobs (e.g., '0 9 * * *' for daily at 9am)",
                },
                "every": {
                    "type": "integer",
                    "description": "Seconds for interval jobs (e.g., 7200 for every 2 hours)",
                },
                "job_id": {
                    "type": "string",
                    "description": "Job ID to remove (required for remove action)",
                },
            },
            "required": ["action"],
        }

    async def execute(
        self,
        action: str,
        name: Optional[str] = None,
        message: Optional[str] = None,
        schedule_type: str = "once",
        at: Optional[str] = None,
        cron: Optional[str] = None,
        every: Optional[int] = None,
        job_id: Optional[str] = None,
    ) -> str:
        """Execute cron tool.

        Args:
            action: Action to perform
            name: Job name
            message: Message to deliver
            schedule_type: Schedule type
            at: ISO datetime
            cron: Cron expression
            every: Interval in seconds
            job_id: Job ID to remove

        Returns:
            Result message
        """
        if not self.cron_service:
            return "Error: Cron service not available"

        if action == "add":
            return await self._add_job(name, message, schedule_type, at, cron, every)
        elif action == "remove":
            return await self._remove_job(job_id)
        elif action == "list":
            return await self._list_jobs()
        else:
            return f"Error: Unknown action '{action}'"

    async def _add_job(
        self,
        name: Optional[str],
        message: Optional[str],
        schedule_type: str,
        at: Optional[str],
        cron: Optional[str],
        every: Optional[int],
    ) -> str:
        """Add a new cron job."""
        if not name or not message:
            return "Error: 'name' and 'message' are required for add action"

        # Validate schedule parameters
        if schedule_type == "once" and not at:
            return "Error: 'at' parameter required for one-time jobs"
        if schedule_type == "cron" and not cron:
            return "Error: 'cron' parameter required for cron jobs"
        if schedule_type == "interval" and not every:
            return "Error: 'every' parameter required for interval jobs"

        try:
            job_id = self.cron_service.add_job(
                name=name,
                message=message,
                schedule_type=schedule_type,
                at=at,
                cron=cron,
                every=every,
            )
            return f"✅ Scheduled '{name}' (ID: {job_id})"
        except Exception as e:
            return f"Error adding job: {str(e)}"

    async def _remove_job(self, job_id: Optional[str]) -> str:
        """Remove a cron job."""
        if not job_id:
            return "Error: 'job_id' parameter required for remove action"

        if self.cron_service.remove_job(job_id):
            return f"✅ Removed job {job_id}"
        else:
            return f"Error: Job {job_id} not found"

    async def _list_jobs(self) -> str:
        """List all cron jobs."""
        jobs = self.cron_service.list_jobs()

        if not jobs:
            return "No scheduled jobs"

        lines = ["📅 Scheduled Jobs:"]
        for job in jobs:
            schedule_info = ""
            if job.schedule_type == "once":
                schedule_info = f"at {job.at}"
            elif job.schedule_type == "cron":
                schedule_info = f"cron: {job.cron}"
            elif job.schedule_type == "interval":
                hours = job.every // 3600 if job.every else 0
                mins = (job.every % 3600) // 60 if job.every else 0
                schedule_info = f"every {hours}h {mins}m"

            lines.append(f"  • {job.name} ({job.id}): {schedule_info}")
            lines.append(f"    Message: {job.message[:50]}...")

        return "\n".join(lines)
