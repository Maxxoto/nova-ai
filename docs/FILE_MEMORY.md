# 🗂️ File-Based Memory System

## Overview

The Nova API now uses a **file-based memory system** for storing conversations and long-term memories. This replaces the previous PostgreSQL-based approach with a simpler, more portable solution optimized for personal assistants.

## Key Features

✅ **Conversation Persistence** - All conversations saved as JSONL files  
✅ **Long-Term Memory** - Facts stored as individual JSON files with embeddings  
✅ **Semantic Search** - Find relevant memories using vector similarity  
✅ **No Database Required** - Everything stored as files on disk  
✅ **Version Control Friendly** - Human-readable files that can be committed to git  
✅ **Portable** - Easy to backup, restore, and transfer  

## Directory Structure

```
data/memory/
├── conversations/
│   ├── <thread_id>/
│   │   ├── metadata.json          # Thread metadata
│   │   ├── messages.jsonl         # Conversation messages
│   │   └── state.json             # Conversation state
│   └── index.json                 # Quick lookup index
├── memories/
│   ├── <user_id>/
│   │   ├── facts/
│   │   │   ├── <fact_id>.json    # Individual memory facts
│   │   │   └── index.json        # Fact index with embeddings
│   │   └── preferences.json       # User preferences
└── archived/                       # Archived conversations
```

## Configuration

Add these environment variables to your `.env` file:

```bash
# Memory data directory (default: ./data/memory)
MEMORY_DATA_DIR=./data/memory

# Embedding model for semantic search (default: all-MiniLM-L6-v2)
# Options: all-MiniLM-L6-v2 (fast, 384 dims) or all-mpnet-base-v2 (better, 768 dims)
EMBEDDING_MODEL=all-MiniLM-L6-v2
```

## Usage

### Basic Example

```python
from adapters.memory.file_memory_adapter import FileMemoryAdapter
from adapters.llm_providers.groq import GroqLLMAdapter

# Initialize
llm_client = GroqLLMAdapter()
memory = FileMemoryAdapter(
    llm_client=llm_client,
    data_dir="./data/memory",
    embedding_model="all-MiniLM-L6-v2"
)

# Store a long-term memory
await memory.store_long_term_memory(
    user_id="user_123",
    content="The user's favorite color is blue",
    metadata={"fact_type": "USER_PREFERENCE"}
)

# Retrieve all memories
memories = await memory.get_longterm_memory(user_id="user_123")

# Semantic search
results = await memory.search_longterm_memory(
    user_id="user_123",
    query="What color does the user like?",
    top_k=5
)

# Conversation history
memory.append_message(
    thread_id="thread_001",
    message={"role": "user", "content": "Hello!"},
    user_id="user_123"
)

history = await memory.get_conversation_history("thread_001")
```

### Running the Demo

```bash
# Run the example script
uv run python examples/file_memory_demo.py
```

This will demonstrate:
- Storing long-term memories
- Retrieving all memories
- Semantic search
- Conversation history
- File structure inspection

## Testing

Run the test suite:

```bash
# Run all memory tests
uv run pytest tests/adapters/memory/test_file_memory_adapter.py -v

# Run specific test
uv run pytest tests/adapters/memory/test_file_memory_adapter.py::test_semantic_search -v
```

## Migration from PostgreSQL

If you have existing data in PostgreSQL, you can migrate it:

1. **Export existing data** (if any)
2. **Update environment variables** - Remove `POSTGRES_URL`, add `MEMORY_DATA_DIR`
3. **Restart the application** - The new system will create the directory structure automatically

## File Formats

### Conversation Message (JSONL)

Each line in `messages.jsonl`:

```json
{"role": "user", "content": "Hello!", "timestamp": "2026-02-12T02:10:31Z"}
{"role": "assistant", "content": "Hi there!", "timestamp": "2026-02-12T02:10:32Z"}
```

### Memory Fact (JSON)

Individual fact file (`<fact_id>.json`):

```json
{
  "fact_id": "abc-123",
  "user_id": "user_001",
  "content": "The user's favorite color is blue",
  "fact_type": "USER_PREFERENCE",
  "embedding": [0.234, -0.567, 0.891, ...],
  "metadata": {},
  "created_at": "2026-02-12T02:10:31Z"
}
```

### Memory Index (JSON)

The `index.json` file aggregates all facts for fast search:

```json
{
  "facts": [
    {
      "fact_id": "abc-123",
      "content": "The user's favorite color is blue",
      "fact_type": "USER_PREFERENCE",
      "embedding": [0.234, -0.567, ...],
      "metadata": {},
      "updated_at": "2026-02-12T02:10:31Z"
    }
  ],
  "last_updated": "2026-02-12T02:10:31Z",
  "version": "1.0"
}
```

## API Reference

### FileMemoryAdapter

Main adapter implementing the `MemoryPort` interface.

**Methods:**

- `get_conversation_history(thread_id)` - Get conversation messages
- `append_message(thread_id, message, user_id)` - Add message to conversation
- `get_longterm_memory(user_id, max_tokens)` - Get all long-term memories
- `search_longterm_memory(user_id, query, top_k, min_similarity)` - Semantic search
- `store_long_term_memory(user_id, content, metadata)` - Store a new memory
- `clear_conversation_memory(thread_id)` - Archive conversation
- `rebuild_user_index(user_id)` - Rebuild memory index from files
- `list_user_threads(user_id)` - List all threads for a user

## Performance Considerations

- **Embedding Model**: `all-MiniLM-L6-v2` is fast (384 dims), `all-mpnet-base-v2` is more accurate (768 dims)
- **Index Updates**: Incremental - only changed facts are re-indexed
- **File Locking**: Uses `filelock` for safe concurrent access
- **Memory Usage**: Indexes are loaded into memory for fast search
- **Scale**: Optimized for personal use (thousands of memories, not millions)

## Troubleshooting

### Issue: Slow first startup

**Cause**: Downloading embedding model on first run  
**Solution**: Wait for model download to complete (one-time only)

### Issue: Permission errors

**Cause**: Insufficient permissions on data directory  
**Solution**: Ensure write permissions: `chmod -R 755 data/memory`

### Issue: Index out of sync

**Cause**: Manual file modifications  
**Solution**: Rebuild index:

```python
memory.rebuild_user_index(user_id="user_123")
```

## Benefits Over PostgreSQL

1. **Simplicity** - No database setup required
2. **Portability** - Easy to backup and transfer
3. **Debugging** - Human-readable files
4. **Version Control** - Can commit memory snapshots
5. **Privacy** - All data stays local
6. **Deployment** - Fewer dependencies

## Limitations

- Not suitable for millions of conversations (use PostgreSQL for that)
- File locking may be slower than database transactions
- No built-in replication (use file sync tools)

## Next Steps

- ✅ Basic file-based storage
- ✅ Semantic search with embeddings
- ✅ Conversation persistence
- 🔄 Automatic archival of old conversations
- 🔄 Compression for large conversation files
- 🔄 Migration script from PostgreSQL
