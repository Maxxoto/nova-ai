"""Executor port interface for plan execution capabilities."""

from abc import ABC, abstractmethod
from typing import AsyncGenerator, Dict, Any, Optional
from uuid import UUID

from ..entities.plan import Plan, PlanStep
from ..entities.agent_state import AgentState


class ExecutorPort(ABC):
    """Interface for plan execution capabilities."""
    
    @abstractmethod
    async def execute_step(self, step: PlanStep, agent_state: AgentState) -> Dict[str, Any]:
        """Execute a single plan step."""
        pass
    
    @abstractmethod
    async def execute_plan(self, plan: Plan, agent_state: AgentState) -> AsyncGenerator[Dict[str, Any], None]:
        """Execute a complete plan and stream progress."""
        pass
    
    @abstractmethod
    async def validate_step(self, step: PlanStep, agent_state: AgentState) -> bool:
        """Validate if a step can be executed with current agent state."""
        pass
    
    @abstractmethod
    async def get_execution_status(self, plan_id: UUID) -> Optional[Dict[str, Any]]:
        """Get the current execution status of a plan."""
        pass