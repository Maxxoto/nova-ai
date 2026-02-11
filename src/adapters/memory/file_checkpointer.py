"""File-based checkpointer for LangGraph compatibility."""

import logging
from pathlib import Path
from typing import Any, Dict, Optional, Sequence
from langgraph.checkpoint.base import BaseCheckpointSaver, Checkpoint, CheckpointMetadata
from adapters.memory.file_storage import FileStorage, MemoryFileManager

logger = logging.getLogger(__name__)


class FileCheckpointer(BaseCheckpointSaver):
    """File-based checkpoint saver for LangGraph state persistence."""

    def __init__(self, data_dir: str = "./data/memory"):
        """Initialize file-based checkpointer.
        
        Args:
            data_dir: Base directory for memory storage
        """
        super().__init__()
        self.data_dir = Path(data_dir)
        self.file_manager = MemoryFileManager(self.data_dir)
        logger.info(f"FileCheckpointer initialized at: {self.data_dir}")

    def get(self, config: Dict[str, Any]) -> Optional[Checkpoint]:
        """Get checkpoint for a thread.
        
        Args:
            config: Config dict with thread_id in configurable
            
        Returns:
            Checkpoint or None
        """
        try:
            thread_id = config.get("configurable", {}).get("thread_id")
            if not thread_id:
                return None
            
            state_file = self.file_manager.get_thread_state_file(thread_id)
            state_data = FileStorage.read_json(state_file)
            
            if state_data is None:
                return None
            
            # Convert stored data back to Checkpoint
            # Note: This is a simplified implementation
            # LangGraph's Checkpoint structure may need adjustment
            return state_data
            
        except Exception as e:
            logger.error(f"Error getting checkpoint: {e}")
            return None

    def put(
        self, 
        config: Dict[str, Any], 
        checkpoint: Checkpoint,
        metadata: CheckpointMetadata
    ) -> Dict[str, Any]:
        """Save checkpoint for a thread.
        
        Args:
            config: Config dict with thread_id
            checkpoint: Checkpoint to save
            metadata: Checkpoint metadata
            
        Returns:
            Updated config
        """
        try:
            thread_id = config.get("configurable", {}).get("thread_id")
            if not thread_id:
                raise ValueError("thread_id required in config")
            
            state_file = self.file_manager.get_thread_state_file(thread_id)
            
            # Save checkpoint state
            # Note: Checkpoint serialization may need adjustment
            FileStorage.atomic_write_json(state_file, checkpoint)
            
            logger.debug(f"Saved checkpoint for thread: {thread_id}")
            return config
            
        except Exception as e:
            logger.error(f"Error saving checkpoint: {e}")
            return config

    def list(
        self, 
        config: Dict[str, Any],
        before: Optional[Dict[str, Any]] = None,
        limit: Optional[int] = None
    ) -> Sequence[Checkpoint]:
        """List checkpoints for a thread.
        
        Note: This simplified implementation returns only the latest checkpoint.
        Full versioning would require additional implementation.
        """
        checkpoint = self.get(config)
        return [checkpoint] if checkpoint else []
