"""Low-level file storage utilities for memory system."""

import json
import os
from pathlib import Path
from typing import Any, Dict, List, Optional, Iterator
from filelock import FileLock
import tempfile
import shutil
from datetime import datetime


class FileStorage:
    """Base class for atomic file operations."""

    @staticmethod
    def ensure_dir(path: Path) -> None:
        """Ensure directory exists."""
        path.mkdir(parents=True, exist_ok=True)

    @staticmethod
    def atomic_write(file_path: Path, content: str) -> None:
        """Write file atomically using temp file + rename."""
        FileStorage.ensure_dir(file_path.parent)
        
        # Write to temp file first
        temp_fd, temp_path = tempfile.mkstemp(
            dir=file_path.parent, 
            prefix=f".{file_path.name}.",
            suffix=".tmp"
        )
        
        try:
            with os.fdopen(temp_fd, 'w', encoding='utf-8') as f:
                f.write(content)
            # Atomic rename
            shutil.move(temp_path, file_path)
        except Exception:
            # Cleanup temp file on error
            if os.path.exists(temp_path):
                os.unlink(temp_path)
            raise

    @staticmethod
    def atomic_write_json(file_path: Path, data: Dict[str, Any], indent: int = 2) -> None:
        """Write JSON file atomically."""
        content = json.dumps(data, indent=indent, ensure_ascii=False)
        FileStorage.atomic_write(file_path, content)

    @staticmethod
    def read_json(file_path: Path) -> Optional[Dict[str, Any]]:
        """Read JSON file safely."""
        if not file_path.exists():
            return None
        
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                return json.load(f)
        except (json.JSONDecodeError, IOError):
            return None


class JSONLWriter:
    """Append-only JSONL (JSON Lines) writer with file locking."""

    def __init__(self, file_path: Path):
        self.file_path = file_path
        self.lock_path = file_path.with_suffix(file_path.suffix + '.lock')
        FileStorage.ensure_dir(file_path.parent)

    def append(self, data: Dict[str, Any]) -> None:
        """Append a JSON object as a new line."""
        lock = FileLock(self.lock_path, timeout=10)
        
        with lock:
            with open(self.file_path, 'a', encoding='utf-8') as f:
                json_line = json.dumps(data, ensure_ascii=False)
                f.write(json_line + '\n')

    def append_many(self, data_list: List[Dict[str, Any]]) -> None:
        """Append multiple JSON objects."""
        lock = FileLock(self.lock_path, timeout=10)
        
        with lock:
            with open(self.file_path, 'a', encoding='utf-8') as f:
                for data in data_list:
                    json_line = json.dumps(data, ensure_ascii=False)
                    f.write(json_line + '\n')


class JSONLReader:
    """Stream JSONL (JSON Lines) reader."""

    def __init__(self, file_path: Path):
        self.file_path = file_path

    def read_all(self) -> List[Dict[str, Any]]:
        """Read all lines as list of dicts."""
        if not self.file_path.exists():
            return []
        
        result = []
        with open(self.file_path, 'r', encoding='utf-8') as f:
            for line in f:
                line = line.strip()
                if line:
                    try:
                        result.append(json.loads(line))
                    except json.JSONDecodeError:
                        # Skip malformed lines
                        continue
        return result

    def stream(self) -> Iterator[Dict[str, Any]]:
        """Stream lines one at a time (memory efficient)."""
        if not self.file_path.exists():
            return
        
        with open(self.file_path, 'r', encoding='utf-8') as f:
            for line in f:
                line = line.strip()
                if line:
                    try:
                        yield json.loads(line)
                    except json.JSONDecodeError:
                        continue

    def read_last_n(self, n: int) -> List[Dict[str, Any]]:
        """Read last N lines efficiently."""
        if not self.file_path.exists():
            return []
        
        # For small files, just read all and slice
        all_lines = self.read_all()
        return all_lines[-n:] if len(all_lines) > n else all_lines


class MemoryFileManager:
    """High-level file manager for memory system."""

    def __init__(self, base_dir: Path):
        self.base_dir = Path(base_dir)
        self.conversations_dir = self.base_dir / "conversations"
        self.memories_dir = self.base_dir / "memories"
        
        # Ensure base directories exist
        FileStorage.ensure_dir(self.conversations_dir)
        FileStorage.ensure_dir(self.memories_dir)

    def get_conversation_dir(self, thread_id: str) -> Path:
        """Get directory for a conversation thread."""
        return self.conversations_dir / thread_id

    def get_messages_file(self, thread_id: str) -> Path:
        """Get messages JSONL file for a thread."""
        return self.get_conversation_dir(thread_id) / "messages.jsonl"

    def get_thread_metadata_file(self, thread_id: str) -> Path:
        """Get metadata JSON file for a thread."""
        return self.get_conversation_dir(thread_id) / "metadata.json"

    def get_thread_state_file(self, thread_id: str) -> Path:
        """Get state JSON file for a thread."""
        return self.get_conversation_dir(thread_id) / "state.json"

    def get_user_memory_dir(self, user_id: str) -> Path:
        """Get directory for user's long-term memories."""
        return self.memories_dir / user_id

    def get_facts_dir(self, user_id: str) -> Path:
        """Get directory for user's memory facts."""
        facts_dir = self.get_user_memory_dir(user_id) / "facts"
        FileStorage.ensure_dir(facts_dir)
        return facts_dir

    def get_fact_file(self, user_id: str, fact_id: str) -> Path:
        """Get file path for a specific fact."""
        return self.get_facts_dir(user_id) / f"{fact_id}.json"

    def get_facts_index_file(self, user_id: str) -> Path:
        """Get index file for user's facts."""
        return self.get_facts_dir(user_id) / "index.json"

    def get_user_preferences_file(self, user_id: str) -> Path:
        """Get preferences file for a user."""
        return self.get_user_memory_dir(user_id) / "preferences.json"

    def list_conversation_threads(self) -> List[str]:
        """List all conversation thread IDs."""
        if not self.conversations_dir.exists():
            return []
        
        return [
            d.name for d in self.conversations_dir.iterdir() 
            if d.is_dir()
        ]

    def list_user_facts(self, user_id: str) -> List[str]:
        """List all fact IDs for a user."""
        facts_dir = self.get_facts_dir(user_id)
        if not facts_dir.exists():
            return []
        
        return [
            f.stem for f in facts_dir.glob("*.json")
            if f.name != "index.json"
        ]

    def create_thread_metadata(
        self, 
        thread_id: str, 
        user_id: str, 
        metadata: Optional[Dict[str, Any]] = None
    ) -> None:
        """Create metadata file for a new thread."""
        meta = {
            "thread_id": thread_id,
            "user_id": user_id,
            "created_at": datetime.utcnow().isoformat(),
            "last_updated": datetime.utcnow().isoformat(),
            **(metadata or {})
        }
        
        file_path = self.get_thread_metadata_file(thread_id)
        FileStorage.atomic_write_json(file_path, meta)

    def update_thread_metadata(
        self, 
        thread_id: str, 
        updates: Dict[str, Any]
    ) -> None:
        """Update thread metadata."""
        file_path = self.get_thread_metadata_file(thread_id)
        current = FileStorage.read_json(file_path) or {}
        
        current.update(updates)
        current["last_updated"] = datetime.utcnow().isoformat()
        
        FileStorage.atomic_write_json(file_path, current)
