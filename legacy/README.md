# Legacy Files Archive

These files were archived on 2025-02-19 as part of the nanobot-style refactoring.
They represent the old LangGraph-based orchestration system that has been replaced
by a simpler AgentLoop architecture.

## Files Archived:

### Orchestrators
- `langgraph_orchestrator.py` - Old LangGraph workflow with multiple nodes
- `enhanced_orchestrator.py` - Alternative orchestrator (kept in main, not archived)

### Service Layer
- `chat_service.py` - Unnecessary wrapper around orchestrator

### CLI Interface
- `chat_interface.py` - Standalone CLI (replaced by unified app.py)

### Nodes (Old Orchestrator Components)
- `final_output_node.py` - Output formatting node
- `intent_detector.py` - Intent classification node
- `llm_node.py` - LLM calling node
- `memory_recall_node.py` - Memory retrieval node
- `memory_write_node.py` - Memory storage node
- `memory_gate_node.py` - Memory decision node

## New Architecture:

The new system uses a simpler approach inspired by nanobot:

```
Channels (Telegram, Discord, etc.)
    ↓
MessageBus (inbound/outbound queues)
    ↓
AgentLoop (message consumer/processor)
    ↓
EnhancedOrchestrator (LLM + tools + memory)
```

## Restoration:

To restore these files, move them back to their original locations in `src/`.
Note that dependencies and imports may need to be updated.

## Rationale:

1. **Simplicity**: Removed unnecessary abstraction layers
2. **Maintainability**: Single AgentLoop is easier to understand than 7+ nodes
3. **Performance**: Less overhead, direct LLM calling
4. **Alignment**: Matches nanobot's proven architecture
