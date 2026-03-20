"""Plan entity representing execution plans for agents."""

from datetime import datetime
from typing import List, Optional
from uuid import UUID, uuid4

from pydantic import BaseModel, Field


class PlanStep(BaseModel):
    """A single step in an execution plan."""

    id: UUID = Field(default_factory=uuid4)
    description: str
    action: str
    parameters: dict = Field(default_factory=dict)
    status: str = Field(
        default="pending", pattern="^(pending|in_progress|completed|failed)$"
    )
    result: Optional[str] = None
    created_at: datetime = Field(default_factory=datetime.utcnow)
    updated_at: datetime = Field(default_factory=datetime.utcnow)


class Plan(BaseModel):
    """Execution plan for an agent containing multiple steps."""

    id: UUID = Field(default_factory=uuid4)
    name: str
    description: Optional[str] = None
    steps: List[PlanStep] = Field(default_factory=list)
    status: str = Field(default="draft", pattern="^(draft|active|completed|cancelled)$")
    agent_id: Optional[UUID] = None
    created_at: datetime = Field(default_factory=datetime.utcnow)
    updated_at: datetime = Field(default_factory=datetime.utcnow)

    def add_step(
        self, description: str, action: str, parameters: dict = None
    ) -> PlanStep:
        """Add a new step to the plan."""
        step = PlanStep(
            description=description, action=action, parameters=parameters or {}
        )
        self.steps.append(step)
        self.updated_at = datetime.utcnow()
        return step

    def update_step_status(
        self, step_id: UUID, status: str, result: str = None
    ) -> bool:
        """Update the status of a specific step."""
        for step in self.steps:
            if step.id == step_id:
                step.status = status
                if result is not None:
                    step.result = result
                step.updated_at = datetime.utcnow()
                self.updated_at = datetime.utcnow()
                return True
        return False

    def get_pending_steps(self) -> List[PlanStep]:
        """Get all pending steps in the plan."""
        return [step for step in self.steps if step.status == "pending"]

    def get_completed_steps(self) -> List[PlanStep]:
        """Get all completed steps in the plan."""
        return [step for step in self.steps if step.status == "completed"]
