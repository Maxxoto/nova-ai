"""Cron tool for scheduled reminders and tasks."""

import json
import logging
import uuid
from dataclasses import dataclass, field
from datetime import datetime, timedelta
from pathlib import Path
from typing import Any, Callable, Optional


logger = logging.getLogger(__name__)


@dataclass
class CronJob:
    """Represents a scheduled cron job."""

    id: str
    name: str
    message: str
    schedule_type: str  # "once", "cron", "interval"
    at: Optional[str] = None  # ISO format for one-time
    cron: Optional[str] = None  # Cron expression
    every: Optional[int] = None  # Seconds for interval
    deliver: bool = False  # Whether to deliver to channel
    to: Optional[str] = None  # User ID for delivery
    channel: Optional[str] = None  # Channel for delivery
    created_at: str = field(default_factory=lambda: datetime.now().isoformat())
    last_run: Optional[str] = None
    run_count: int = 0

    def to_dict(self) -> dict:
        """Convert to dictionary."""
        return {
            "id": self.id,
            "name": self.name,
            "message": self.message,
            "schedule_type": self.schedule_type,
            "at": self.at,
            "cron": self.cron,
            "every": self.every,
            "deliver": self.deliver,
            "to": self.to,
            "channel": self.channel,
            "created_at": self.created_at,
            "last_run": self.last_run,
            "run_count": self.run_count,
        }

    @classmethod
    def from_dict(cls, data: dict) -> "CronJob":
        """Create from dictionary."""
        return cls(
            id=data["id"],
            name=data["name"],
            message=data["message"],
            schedule_type=data["schedule_type"],
            at=data.get("at"),
            cron=data.get("cron"),
            every=data.get("every"),
            deliver=data.get("deliver", False),
            to=data.get("to"),
            channel=data.get("channel"),
            created_at=data.get("created_at", datetime.now().isoformat()),
            last_run=data.get("last_run"),
            run_count=data.get("run_count", 0),
        )

    def should_run(self) -> bool:
        """Check if job should run now."""
        now = datetime.now()

        if self.schedule_type == "once":
            if not self.at:
                return False
            run_time = datetime.fromisoformat(self.at)
            # Run if we're within 1 minute of the scheduled time and haven't run yet
            if self.run_count > 0:
                return False
            time_diff = (now - run_time).total_seconds()
            return -60 <= time_diff <= 60

        elif self.schedule_type == "interval":
            if not self.every:
                return False
            if not self.last_run:
                return True
            last_run = datetime.fromisoformat(self.last_run)
            seconds_since = (now - last_run).total_seconds()
            return seconds_since >= self.every

        elif self.schedule_type == "cron":
            # Simple cron check - check if current minute matches
            if not self.cron:
                return False
            return self._check_cron_match()

        return False

    def _check_cron_match(self) -> bool:
        """Check if current time matches cron expression."""
        try:
            # Basic cron parsing (minute hour day month weekday)
            parts = self.cron.split()
            if len(parts) != 5:
                return False

            now = datetime.now()
            minute, hour, day, month, weekday = parts

            # Check minute
            if minute != "*" and int(minute) != now.minute:
                return False
            # Check hour
            if hour != "*" and int(hour) != now.hour:
                return False
            # Check day
            if day != "*" and int(day) != now.day:
                return False
            # Check month
            if month != "*" and int(month) != now.month:
                return False
            # Check weekday (0-6, Monday=0)
            if weekday != "*" and int(weekday) != now.weekday():
                return False

            return True
        except (ValueError, IndexError):
            return False


class CronService:
    """Service for managing scheduled cron jobs."""

    def __init__(
        self,
        workspace: Path,
        on_job: Optional[Callable[[str, Optional[str], Optional[str]], None]] = None,
    ):
        """Initialize cron service.

        Args:
            workspace: Path to workspace directory
            on_job: Callback when job fires (message, to, channel)
        """
        self.workspace = Path(workspace)
        self.jobs_file = self.workspace / "cron_jobs.json"
        self.on_job = on_job
        self.jobs: list[CronJob] = []
        self._running = False
        self._task: Optional[asyncio.Task] = None

    async def start(self) -> None:
        """Start the cron service."""
        self._load_jobs()
        self._running = True
        import asyncio

        self._task = asyncio.create_task(self._run_loop())
        logger.info(f"Cron service started with {len(self.jobs)} jobs")

    async def stop(self) -> None:
        """Stop the cron service."""
        self._running = False
        if self._task:
            self._task.cancel()
            try:
                await self._task
            except asyncio.CancelledError:
                pass
        logger.info("Cron service stopped")

    async def _run_loop(self) -> None:
        """Main cron loop - checks every minute."""
        import asyncio

        while self._running:
            try:
                await self._check_jobs()
                await asyncio.sleep(60)  # Check every minute
            except asyncio.CancelledError:
                break
            except Exception as e:
                logger.error(f"Error in cron loop: {e}")
                await asyncio.sleep(60)

    async def _check_jobs(self) -> None:
        """Check and execute due jobs."""
        for job in self.jobs:
            if job.should_run():
                logger.info(f"Executing cron job: {job.name}")
                await self._execute_job(job)

    async def _execute_job(self, job: CronJob) -> None:
        """Execute a cron job.

        Args:
            job: Job to execute
        """
        try:
            # Update job stats
            job.last_run = datetime.now().isoformat()
            job.run_count += 1

            # Call callback if provided
            if self.on_job:
                await self.on_job(job.message, job.to, job.channel)

            # Remove one-time jobs after execution
            if job.schedule_type == "once":
                self.jobs.remove(job)

            # Save updated jobs
            self._save_jobs()

        except Exception as e:
            logger.error(f"Error executing cron job {job.name}: {e}")

    def add_job(
        self,
        name: str,
        message: str,
        schedule_type: str = "once",
        at: Optional[str] = None,
        cron: Optional[str] = None,
        every: Optional[int] = None,
        deliver: bool = False,
        to: Optional[str] = None,
        channel: Optional[str] = None,
    ) -> str:
        """Add a new cron job.

        Args:
            name: Job name
            message: Message to deliver
            schedule_type: "once", "cron", or "interval"
            at: ISO datetime for one-time jobs
            cron: Cron expression for recurring jobs
            every: Seconds for interval jobs
            deliver: Whether to deliver to channel
            to: User ID for delivery
            channel: Channel for delivery

        Returns:
            Job ID
        """
        job = CronJob(
            id=str(uuid.uuid4())[:8],
            name=name,
            message=message,
            schedule_type=schedule_type,
            at=at,
            cron=cron,
            every=every,
            deliver=deliver,
            to=to,
            channel=channel,
        )

        self.jobs.append(job)
        self._save_jobs()

        logger.info(f"Added cron job: {name} ({job.id})")
        return job.id

    def remove_job(self, job_id: str) -> bool:
        """Remove a cron job.

        Args:
            job_id: Job ID to remove

        Returns:
            True if removed, False if not found
        """
        for i, job in enumerate(self.jobs):
            if job.id == job_id:
                self.jobs.pop(i)
                self._save_jobs()
                logger.info(f"Removed cron job: {job_id}")
                return True
        return False

    def list_jobs(self) -> list[CronJob]:
        """List all cron jobs."""
        return self.jobs.copy()

    def _load_jobs(self) -> None:
        """Load jobs from file."""
        if not self.jobs_file.exists():
            self.jobs = []
            return

        try:
            data = json.loads(self.jobs_file.read_text(encoding="utf-8"))
            self.jobs = [
                CronJob.from_dict(job_data) for job_data in data.get("jobs", [])
            ]
            logger.debug(f"Loaded {len(self.jobs)} cron jobs")
        except Exception as e:
            logger.error(f"Error loading cron jobs: {e}")
            self.jobs = []

    def _save_jobs(self) -> None:
        """Save jobs to file."""
        try:
            data = {"jobs": [job.to_dict() for job in self.jobs]}
            self.jobs_file.write_text(json.dumps(data, indent=2), encoding="utf-8")
        except Exception as e:
            logger.error(f"Error saving cron jobs: {e}")

    @property
    def is_running(self) -> bool:
        """Check if cron service is running."""
        return self._running
