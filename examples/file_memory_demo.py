"""Example usage of file-based memory system."""

import asyncio
import sys
from pathlib import Path

# Add src to path
sys.path.insert(0, str(Path(__file__).parent.parent / "src"))

from adapters.memory.file_memory_adapter import FileMemoryAdapter
from adapters.llm_providers.groq import GroqLLMAdapter


async def main():
    """Demonstrate file-based memory usage."""
    
    print("🧠 File-Based Memory System Demo\n")
    
    # Initialize memory adapter
    print("Initializing memory adapter...")
    llm_client = GroqLLMAdapter()
    memory = FileMemoryAdapter(
        llm_client=llm_client,
        data_dir="./data/memory",
        embedding_model="all-MiniLM-L6-v2"
    )
    print("✓ Memory adapter initialized\n")
    
    # Example 1: Store long-term memories
    print("=" * 50)
    print("Example 1: Storing Long-Term Memories")
    print("=" * 50)
    
    user_id = "demo_user"
    
    memories_to_store = [
        ("The user's favorite color is blue", "USER_PREFERENCE"),
        ("The user works as a software engineer", "USER_INFO"),
        ("The user enjoys hiking and photography", "USER_PREFERENCE"),
        ("The user prefers morning meetings", "USER_PREFERENCE"),
    ]
    
    for content, fact_type in memories_to_store:
        success = await memory.store_long_term_memory(
            user_id=user_id,
            content=content,
            metadata={"fact_type": fact_type}
        )
        print(f"✓ Stored: {content}")
    
    print()
    
    # Example 2: Retrieve all memories
    print("=" * 50)
    print("Example 2: Retrieving All Memories")
    print("=" * 50)
    
    all_memories = await memory.get_longterm_memory(user_id=user_id)
    if all_memories:
        print(all_memories)
    else:
        print("No memories found")
    
    print()
    
    # Example 3: Semantic search
    print("=" * 50)
    print("Example 3: Semantic Search")
    print("=" * 50)
    
    queries = [
        "What color does the user like?",
        "What is the user's job?",
        "What hobbies does the user have?",
    ]
    
    for query in queries:
        print(f"\nQuery: {query}")
        results = await memory.search_longterm_memory(
            user_id=user_id,
            query=query,
            top_k=2,
            min_similarity=0.2
        )
        if results:
            print(f"Results:\n{results}")
        else:
            print("No relevant memories found")
    
    print()
    
    # Example 4: Conversation history
    print("=" * 50)
    print("Example 4: Conversation History")
    print("=" * 50)
    
    thread_id = "demo_thread_001"
    
    # Simulate a conversation
    conversation = [
        {"role": "user", "content": "Hello! What's my favorite color?"},
        {"role": "assistant", "content": "Based on my memory, your favorite color is blue!"},
        {"role": "user", "content": "That's right! What do I do for work?"},
        {"role": "assistant", "content": "You work as a software engineer."},
    ]
    
    for message in conversation:
        memory.append_message(thread_id, message, user_id)
        print(f"✓ Added: [{message['role']}] {message['content']}")
    
    print()
    
    # Retrieve conversation history
    print("Retrieving conversation history...")
    history = await memory.get_conversation_history(thread_id)
    
    if history:
        print(f"\nConversation has {len(history)} messages:")
        for i, msg in enumerate(history, 1):
            print(f"  {i}. [{msg['role']}]: {msg['content']}")
    
    print()
    
    # Example 5: File structure
    print("=" * 50)
    print("Example 5: File Structure")
    print("=" * 50)
    
    data_dir = Path("./data/memory")
    
    print(f"\nMemory files stored in: {data_dir.absolute()}")
    print("\nDirectory structure:")
    
    if data_dir.exists():
        for item in sorted(data_dir.rglob("*")):
            if item.is_file():
                rel_path = item.relative_to(data_dir)
                size = item.stat().st_size
                print(f"  📄 {rel_path} ({size:,} bytes)")
    
    print("\n✓ Demo complete!")
    print(f"\nYou can inspect the files in: {data_dir.absolute()}")


if __name__ == "__main__":
    asyncio.run(main())
