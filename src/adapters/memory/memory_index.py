"""Memory indexing and semantic search functionality."""

import json
import logging
import numpy as np
from pathlib import Path
from typing import List, Dict, Any, Optional, Tuple
from datetime import datetime
from sentence_transformers import SentenceTransformer

from adapters.memory.file_storage import FileStorage

logger = logging.getLogger(__name__)


class EmbeddingCache:
    """Cache for computed embeddings."""

    def __init__(self, model_name: str = "all-MiniLM-L6-v2"):
        """Initialize embedding model.
        
        Args:
            model_name: Sentence transformer model name
                - all-MiniLM-L6-v2: Fast, 384 dimensions (default)
                - all-mpnet-base-v2: Better quality, 768 dimensions
        """
        logger.info(f"Loading embedding model: {model_name}")
        self.model = SentenceTransformer(model_name)
        self.dimension = self.model.get_sentence_embedding_dimension()
        logger.info(f"Embedding dimension: {self.dimension}")

    def encode(self, text: str) -> List[float]:
        """Encode text to embedding vector."""
        embedding = self.model.encode(text, convert_to_numpy=True)
        return embedding.tolist()

    def encode_batch(self, texts: List[str]) -> List[List[float]]:
        """Encode multiple texts efficiently."""
        embeddings = self.model.encode(texts, convert_to_numpy=True)
        return embeddings.tolist()


class SemanticSearch:
    """Semantic similarity search using cosine similarity."""

    @staticmethod
    def cosine_similarity(vec1: List[float], vec2: List[float]) -> float:
        """Compute cosine similarity between two vectors."""
        v1 = np.array(vec1)
        v2 = np.array(vec2)
        
        dot_product = np.dot(v1, v2)
        norm_v1 = np.linalg.norm(v1)
        norm_v2 = np.linalg.norm(v2)
        
        if norm_v1 == 0 or norm_v2 == 0:
            return 0.0
        
        return float(dot_product / (norm_v1 * norm_v2))

    @staticmethod
    def search(
        query_embedding: List[float],
        facts: List[Dict[str, Any]],
        top_k: int = 5,
        min_similarity: float = 0.3
    ) -> List[Tuple[Dict[str, Any], float]]:
        """Search facts by semantic similarity.
        
        Args:
            query_embedding: Query vector
            facts: List of fact dicts with 'embedding' field
            top_k: Number of top results to return
            min_similarity: Minimum similarity threshold
            
        Returns:
            List of (fact, similarity_score) tuples, sorted by score
        """
        results = []
        
        for fact in facts:
            if "embedding" not in fact:
                continue
            
            similarity = SemanticSearch.cosine_similarity(
                query_embedding, 
                fact["embedding"]
            )
            
            if similarity >= min_similarity:
                results.append((fact, similarity))
        
        # Sort by similarity (highest first)
        results.sort(key=lambda x: x[1], reverse=True)
        
        return results[:top_k]


class MemoryIndex:
    """Index manager for fast memory retrieval."""

    def __init__(self, index_file: Path, embedding_cache: EmbeddingCache):
        self.index_file = index_file
        self.embedding_cache = embedding_cache
        self._index_data: Optional[Dict[str, Any]] = None

    def load(self) -> Dict[str, Any]:
        """Load index from file."""
        if self._index_data is not None:
            return self._index_data
        
        data = FileStorage.read_json(self.index_file)
        if data is None:
            # Create new index
            data = {
                "facts": [],
                "last_updated": datetime.utcnow().isoformat(),
                "version": "1.0"
            }
        
        self._index_data = data
        return data

    def save(self) -> None:
        """Save index to file."""
        if self._index_data is None:
            return
        
        self._index_data["last_updated"] = datetime.utcnow().isoformat()
        FileStorage.atomic_write_json(self.index_file, self._index_data)

    def add_fact(
        self, 
        fact_id: str, 
        content: str, 
        fact_type: str,
        metadata: Optional[Dict[str, Any]] = None,
        embedding: Optional[List[float]] = None
    ) -> None:
        """Add or update a fact in the index.
        
        Args:
            fact_id: Unique fact identifier
            content: Fact content text
            fact_type: Type of fact (e.g., USER_PREFERENCE, USER_INFO)
            metadata: Optional metadata
            embedding: Pre-computed embedding (will compute if not provided)
        """
        index = self.load()
        
        # Compute embedding if not provided
        if embedding is None:
            embedding = self.embedding_cache.encode(content)
        
        # Check if fact already exists
        existing_idx = None
        for idx, fact in enumerate(index["facts"]):
            if fact["fact_id"] == fact_id:
                existing_idx = idx
                break
        
        fact_entry = {
            "fact_id": fact_id,
            "content": content,
            "fact_type": fact_type,
            "embedding": embedding,
            "metadata": metadata or {},
            "updated_at": datetime.utcnow().isoformat()
        }
        
        if existing_idx is not None:
            # Update existing
            index["facts"][existing_idx] = fact_entry
        else:
            # Add new
            index["facts"].append(fact_entry)
        
        self._index_data = index
        self.save()

    def remove_fact(self, fact_id: str) -> bool:
        """Remove a fact from the index.
        
        Returns:
            True if fact was found and removed, False otherwise
        """
        index = self.load()
        
        original_count = len(index["facts"])
        index["facts"] = [
            f for f in index["facts"] 
            if f["fact_id"] != fact_id
        ]
        
        if len(index["facts"]) < original_count:
            self._index_data = index
            self.save()
            return True
        
        return False

    def search(
        self, 
        query: str, 
        top_k: int = 5,
        min_similarity: float = 0.3,
        fact_types: Optional[List[str]] = None
    ) -> List[Tuple[Dict[str, Any], float]]:
        """Search facts by semantic similarity.
        
        Args:
            query: Search query text
            top_k: Number of results to return
            min_similarity: Minimum similarity threshold
            fact_types: Optional list of fact types to filter by
            
        Returns:
            List of (fact, similarity_score) tuples
        """
        index = self.load()
        
        # Filter by fact type if specified
        facts = index["facts"]
        if fact_types:
            facts = [f for f in facts if f.get("fact_type") in fact_types]
        
        if not facts:
            return []
        
        # Encode query
        query_embedding = self.embedding_cache.encode(query)
        
        # Search
        results = SemanticSearch.search(
            query_embedding,
            facts,
            top_k=top_k,
            min_similarity=min_similarity
        )
        
        return results

    def get_all_facts(self) -> List[Dict[str, Any]]:
        """Get all facts from index."""
        index = self.load()
        return index["facts"]

    def rebuild_from_files(self, facts_dir: Path) -> None:
        """Rebuild index from fact files.
        
        Args:
            facts_dir: Directory containing fact JSON files
        """
        logger.info(f"Rebuilding index from {facts_dir}")
        
        # Reset index
        self._index_data = {
            "facts": [],
            "last_updated": datetime.utcnow().isoformat(),
            "version": "1.0"
        }
        
        # Load all fact files
        if not facts_dir.exists():
            self.save()
            return
        
        fact_files = list(facts_dir.glob("*.json"))
        # Exclude index.json itself
        fact_files = [f for f in fact_files if f.name != "index.json"]
        
        logger.info(f"Found {len(fact_files)} fact files")
        
        for fact_file in fact_files:
            fact_data = FileStorage.read_json(fact_file)
            if fact_data is None:
                continue
            
            # Add to index
            self.add_fact(
                fact_id=fact_data.get("fact_id", fact_file.stem),
                content=fact_data.get("content", ""),
                fact_type=fact_data.get("fact_type", "UNKNOWN"),
                metadata=fact_data.get("metadata", {}),
                embedding=fact_data.get("embedding")
            )
        
        logger.info(f"Index rebuilt with {len(self._index_data['facts'])} facts")
