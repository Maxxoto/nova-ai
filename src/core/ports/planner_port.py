"""Planner port interface for planning capabilities."""

from abc import ABC, abstractmethod
from typing import List, Optional
from uuid import UUID

from ..entities.plan import Plan
from ..entities.agent_state import AgentState


class PlannerPort(ABC):
    """Interface for planning capabilities."""
    
    @abstractmethod
    async def create_plan(self, goal: str, agent_state: AgentState) -> Plan:
        """Create a new execution plan based on a goal and agent state."""
        pass
    
    @abstractmethod
    async def update_plan(self, plan_id: UUID, updates: dict) -> Optional[Plan]:
        """Update an existing plan."""
        pass
    
    @abstractmethod
    async def get_plan(self, plan_id: UUID) -> Optional[Plan]:
        """Get a plan by ID."""
        pass
    
    @abstractmethod
    async def list_plans(self, agent_id: Optional[UUID] = None) -> List[Plan]:
        """List all plans, optionally filtered by agent ID."""
        pass