# Archived Components

This directory contains components that are **NOT used** by `nova chat` (the CLI chat interface).

## What's Archived Here

### Application Services (LangGraph & Agent Loop)

#### `/src/app/application/services/`
- **`enhanced_orchestrator.py`** - LangGraph-based workflow orchestrator with state management
- **`agent_loop.py`** - Alternative agent loop with message bus (not used by CLI)

#### `/src/app/application/services/nodes/`
- **`context_builder_node.py`** - LangGraph node for building context with skills/memory
- **`tool_execution_node.py`** - LangGraph node for tool execution

### Infrastructure

#### `/src/app/infrastructure/bus/`
- **`queue.py`** - Message bus queue implementation
- **`events.py`** - Message bus events (InboundMessage, OutboundMessage)
- **`__init__.py`** - Bus module init

#### `/src/app/infrastructure/channels/`
- **`telegram.py`** - Telegram bot channel adapter
- **`base.py`** - Base channel interface
- **`__init__.py`** - Channels module init

### Domain Layer

#### `/src/app/domain/entities/`
- **`agent_state.py`** - Agent state model for LangGraph workflow
- **`agent_state_models.py`** - Pydantic models (ToolCall, ToolResult, MemoryEntry)
- **`plan.py`** - Planning entity
- **`chat_message.py`** - Chat message entity

#### `/src/app/domain/ports/`
- **`memory_port.py`** - Memory port interface
- **`executor_port.py`** - Executor port interface
- **`planner_port.py`** - Planner port interface

### Dependencies
- **`dependencies.py`** - Dependency injection setup for services (not used by CLI)

## What's Still Active (Used by CLI)

The following components remain in `src/app/` and are actively used by `nova chat`:

### CLI Interface
- `src/app/interfaces/cli/app.py` - Main CLI entry point

### Application Services
- `src/app/application/services/llm_validator.py` - Pydantic AI validation service

### Infrastructure Layer
- **Tools**: `src/app/infrastructure/tools/` - Tool registry and implementations
- **Session**: `src/app/infrastructure/session/` - Session management
- **Memory**: `src/app/infrastructure/memory/` - Memory storage and consolidation
- **Skills**: `src/app/infrastructure/skills/` - Skills loading and context building

### Adapters
- **LLM**: `src/app/adapters/llm_providers/litellm_adapter.py` - LiteLLM multi-provider adapter
- **Config**: `src/app/adapters/config.py` - Configuration settings

### Domain Ports (Active)
- **`src/app/domain/ports/llm_client_port.py`** - LLM client interface (used by LiteLLMAdapter)

### Domain Parsers
- `src/app/domain/parsers/` - Command parsers (if present)

## Architecture Difference

### Active Architecture (CLI)
```
CLI (app.py)
  ├── LiteLLMAdapter (direct calls)
  ├── ToolRegistry (direct execution)
  ├── SessionManager (persistence)
  ├── MemoryStore (long-term memory)
  ├── ContextBuilder (skills + memory)
  └── SkillsLoader (load skills)
```

### Archived Architecture (LangGraph)
```
Enhanced Orchestrator (LangGraph)
  ├── Context Builder Node → Build context
  ├── Intent Detector Node → Classify intent
  ├── Memory Gate Node → Decide memory operations
  ├── Memory Recall Node → Retrieve memories
  ├── LLM Node → Call LLM
  ├── Tool Execution Node → Execute tools
  ├── Memory Write Node → Store memories
  └── Final Output Node → Return response
```

## Why These Were Archived

1. **Not Used by CLI**: `nova chat` uses a simple direct LLM calling approach in `_process_message()`
2. **LangGraph Complexity**: The LangGraph workflow adds complexity that's not needed for simple chat
3. **Future Use**: These components can be reactivated if/when implementing:
   - Web interface with LangGraph workflow
   - Telegram bot with message bus
   - Advanced planning and execution
   - Multi-turn workflow orchestration

## How to Reactivate

If you need these components:

```bash
# Move back from archive
mv legacy/archive/src/app/application/services/enhanced_orchestrator.py src/app/application/services/
mv legacy/archive/src/app/application/services/agent_loop.py src/app/application/services/
mv legacy/archive/src/app/application/services/nodes/* src/app/application/services/nodes/
mv legacy/archive/src/app/infrastructure/bus/* src/app/infrastructure/bus/
mv legacy/archive/src/app/infrastructure/channels/* src/app/infrastructure/channels/
mv legacy/archive/src/app/domain/entities/* src/app/domain/entities/
mv legacy/archive/src/app/domain/ports/* src/app/domain/ports/

# Recreate __init__.py files as needed
touch src/app/application/services/nodes/__init__.py
touch src/app/infrastructure/bus/__init__.py
touch src/app/infrastructure/channels/__init__.py
touch src/app/domain/entities/__init__.py
```

## Related Documentation

- **LangGraph Workflow**: See `docs/PYDANTIC_AI_INTEGRATION.md`
- **Agent State Models**: See `src/app/domain/entities/agent_state_models.py`
- **Tool Execution**: Active in `src/app/infrastructure/tools/`

---

**Archived**: 2026-03-21
**Reason**: Not used by `nova chat` CLI
**Status**: Can be reactivated for future interfaces (web, API, Telegram)