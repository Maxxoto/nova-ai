"""File-based memory adapter implementing MemoryPort."""

import logging
import uuid
from pathlib import Path
from typing import List, Dict, Any, Optional
from datetime import datetime

from core.ports.memory_port import MemoryPort
from core.ports.llm_client_port import LLMClientPort
from adapters.memory.file_storage import (
    FileStorage,
    JSONLWriter,
    JSONLReader,
    MemoryFileManager
)
from adapters.memory.memory_index import (
    EmbeddingCache,
    MemoryIndex
)

logger = logging.getLogger(__name__)


class FileMemoryAdapter(MemoryPort):
    """File-based memory adapter with semantic search capabilities."""

    def __init__(
        self,
        llm_client: LLMClientPort,
        data_dir: str = "./data/memory",
        embedding_model: str = "all-MiniLM-L6-v2"
    ):
        """Initialize file-based memory adapter.
        
        Args:
            llm_client: LLM client for memory operations
            data_dir: Base directory for memory storage
            embedding_model: Sentence transformer model name
        """
        self.llm_client = llm_client
        self.data_dir = Path(data_dir)
        self.file_manager = MemoryFileManager(self.data_dir)
        
        # Initialize embedding cache
        logger.info("Initializing embedding model...")
        self.embedding_cache = EmbeddingCache(model_name=embedding_model)
        
        logger.info(f"File-based memory initialized at: {self.data_dir}")

    def _get_memory_index(self, user_id: str) -> MemoryIndex:
        """Get memory index for a user."""
        index_file = self.file_manager.get_facts_index_file(user_id)
        return MemoryIndex(index_file, self.embedding_cache)

    async def get_conversation_history(
        self, 
        thread_id: str
    ) -> Optional[List[Dict[str, Any]]]:
        """Get conversation history from JSONL file.
        
        Args:
            thread_id: Thread ID for conversation context
            
        Returns:
            List of conversation messages or None if not found
        """
        try:
            messages_file = self.file_manager.get_messages_file(thread_id)
            
            if not messages_file.exists():
                logger.debug(f"No conversation history found for thread: {thread_id}")
                return None
            
            reader = JSONLReader(messages_file)
            messages = reader.read_all()
            
            logger.debug(f"Loaded {len(messages)} messages for thread: {thread_id}")
            return messages if messages else None
            
        except Exception as e:
            logger.error(f"Error reading conversation history: {e}")
            return None

    def append_message(
        self,
        thread_id: str,
        message: Dict[str, Any],
        user_id: Optional[str] = None
    ) -> bool:
        """Append a message to conversation history.
        
        Args:
            thread_id: Thread ID
            message: Message dict to append
            user_id: Optional user ID for metadata
            
        Returns:
            True if successful
        """
        try:
            messages_file = self.file_manager.get_messages_file(thread_id)
            
            # Create thread metadata if this is the first message
            if not messages_file.exists() and user_id:
                self.file_manager.create_thread_metadata(
                    thread_id=thread_id,
                    user_id=user_id
                )
            
            # Add timestamp if not present
            if "timestamp" not in message:
                message["timestamp"] = datetime.utcnow().isoformat()
            
            # Append to JSONL file
            writer = JSONLWriter(messages_file)
            writer.append(message)
            
            # Update thread metadata
            if user_id:
                self.file_manager.update_thread_metadata(
                    thread_id=thread_id,
                    updates={"message_count": len(JSONLReader(messages_file).read_all())}
                )
            
            return True
            
        except Exception as e:
            logger.error(f"Error appending message: {e}")
            return False

    async def get_longterm_memory(
        self, 
        user_id: str, 
        max_tokens: int = 500
    ) -> Optional[str]:
        """Retrieve long-term memories using semantic search.
        
        Args:
            user_id: User ID
            max_tokens: Maximum tokens for summary (not strictly enforced)
            
        Returns:
            Formatted string of relevant memories or None
        """
        try:
            # Get memory index
            memory_index = self._get_memory_index(user_id)
            
            # Get all facts (for now, return all; can add query-based filtering later)
            facts = memory_index.get_all_facts()
            
            if not facts:
                logger.info(f"No long-term memories found for user: {user_id}")
                return None
            
            # Format facts as readable text
            memory_lines = []
            for fact in facts:
                fact_type = fact.get("fact_type", "INFO")
                content = fact.get("content", "")
                memory_lines.append(f"[{fact_type}]: {content}")
            
            result = "\n".join(memory_lines)
            logger.debug(f"Retrieved {len(facts)} memories for user: {user_id}")
            
            return result
            
        except Exception as e:
            logger.error(f"Error retrieving long-term memory: {e}")
            return None

    async def search_longterm_memory(
        self,
        user_id: str,
        query: str,
        top_k: int = 5,
        min_similarity: float = 0.3
    ) -> Optional[str]:
        """Search long-term memories by semantic similarity.
        
        Args:
            user_id: User ID
            query: Search query
            top_k: Number of results to return
            min_similarity: Minimum similarity threshold
            
        Returns:
            Formatted string of relevant memories or None
        """
        try:
            memory_index = self._get_memory_index(user_id)
            
            # Search using semantic similarity
            results = memory_index.search(
                query=query,
                top_k=top_k,
                min_similarity=min_similarity
            )
            
            if not results:
                logger.info(f"No relevant memories found for query: {query}")
                return None
            
            # Format results
            memory_lines = []
            for fact, similarity in results:
                fact_type = fact.get("fact_type", "INFO")
                content = fact.get("content", "")
                memory_lines.append(f"[{fact_type}] (similarity: {similarity:.2f}): {content}")
            
            result = "\n".join(memory_lines)
            logger.debug(f"Found {len(results)} relevant memories for query: {query}")
            
            return result
            
        except Exception as e:
            logger.error(f"Error searching long-term memory: {e}")
            return None

    async def store_long_term_memory(
        self, 
        user_id: str, 
        content: str, 
        metadata: Optional[Dict[str, Any]] = None
    ) -> bool:
        """Store a memory in long-term storage.
        
        Args:
            user_id: User ID
            content: Memory content
            metadata: Optional metadata (fact_type, etc.)
            
        Returns:
            True if successful
        """
        try:
            # Generate unique fact ID
            fact_id = str(uuid.uuid4())
            
            # Extract fact type from metadata or default
            fact_type = (metadata or {}).get("fact_type", "USER_INFO")
            
            # Compute embedding
            embedding = self.embedding_cache.encode(content)
            
            # Create fact data
            fact_data = {
                "fact_id": fact_id,
                "user_id": user_id,
                "content": content,
                "fact_type": fact_type,
                "embedding": embedding,
                "metadata": metadata or {},
                "created_at": datetime.utcnow().isoformat()
            }
            
            # Save fact file
            fact_file = self.file_manager.get_fact_file(user_id, fact_id)
            FileStorage.atomic_write_json(fact_file, fact_data)
            
            # Update index
            memory_index = self._get_memory_index(user_id)
            memory_index.add_fact(
                fact_id=fact_id,
                content=content,
                fact_type=fact_type,
                metadata=metadata,
                embedding=embedding
            )
            
            logger.info(f"Stored long-term memory for user {user_id}: {content[:50]}...")
            return True
            
        except Exception as e:
            logger.error(f"Error storing long-term memory: {e}")
            return False

    async def clear_conversation_memory(self, thread_id: str) -> bool:
        """Clear/archive conversation memory for a thread.
        
        Args:
            thread_id: Thread ID to clear
            
        Returns:
            True if successful
        """
        try:
            conversation_dir = self.file_manager.get_conversation_dir(thread_id)
            
            if not conversation_dir.exists():
                logger.debug(f"No conversation to clear for thread: {thread_id}")
                return True
            
            # Archive instead of delete (move to archived subdirectory)
            archive_dir = self.data_dir / "archived" / "conversations" / thread_id
            FileStorage.ensure_dir(archive_dir.parent)
            
            # Move conversation directory to archive
            import shutil
            shutil.move(str(conversation_dir), str(archive_dir))
            
            logger.info(f"Archived conversation for thread: {thread_id}")
            return True
            
        except Exception as e:
            logger.error(f"Error clearing conversation memory: {e}")
            return False

    def rebuild_user_index(self, user_id: str) -> bool:
        """Rebuild memory index for a user from fact files.
        
        Args:
            user_id: User ID
            
        Returns:
            True if successful
        """
        try:
            memory_index = self._get_memory_index(user_id)
            facts_dir = self.file_manager.get_facts_dir(user_id)
            
            memory_index.rebuild_from_files(facts_dir)
            logger.info(f"Rebuilt memory index for user: {user_id}")
            return True
            
        except Exception as e:
            logger.error(f"Error rebuilding index: {e}")
            return False

    def get_thread_metadata(self, thread_id: str) -> Optional[Dict[str, Any]]:
        """Get metadata for a conversation thread.
        
        Args:
            thread_id: Thread ID
            
        Returns:
            Metadata dict or None
        """
        metadata_file = self.file_manager.get_thread_metadata_file(thread_id)
        return FileStorage.read_json(metadata_file)

    def list_user_threads(self, user_id: str) -> List[str]:
        """List all thread IDs for a user.
        
        Args:
            user_id: User ID
            
        Returns:
            List of thread IDs
        """
        all_threads = self.file_manager.list_conversation_threads()
        
        # Filter by user_id
        user_threads = []
        for thread_id in all_threads:
            metadata = self.get_thread_metadata(thread_id)
            if metadata and metadata.get("user_id") == user_id:
                user_threads.append(thread_id)
        
        return user_threads
