# Archive Summary - 2026-03-21

## Components Moved to `legacy/archive/`

### Total Files Archived: 18

### Application Services (4files)
- ✅ `enhanced_orchestrator.py` - LangGraph workflow orchestrator
- ✅ `agent_loop.py` - Alternative agent loop with message bus
- ✅ `nodes/context_builder_node.py` - Context builder node
- ✅ `nodes/tool_execution_node.py` - Tool execution node

### Infrastructure (6 files)
- ✅ `bus/queue.py` - Message bus queue
- ✅ `bus/events.py` - Message bus events
- ✅ `bus/__init__.py` - Bus module init
- ✅ `channels/telegram.py` - Telegram adapter
- ✅ `channels/base.py` - Channel interface
- ✅ `channels/__init__.py` - Channels module init

### Domain Entities (4 files)
- ✅ `entities/agent_state.py` - Agent state model
- ✅ `entities/agent_state_models.py` - Pydantic models
- ✅ `entities/plan.py` - Planning entity
- ✅ `entities/chat_message.py` - Chat message entity

### Domain Ports (3 files)
- ✅ `ports/memory_port.py` - Memory interface
- ✅ `ports/executor_port.py` - Executor interface
- ✅ `ports/planner_port.py` - Planner interface

### Dependencies (1 file)
- ✅ `dependencies.py` - Dependency injection setup

## Directories Cleaned Up

### Removed Empty Directories
- `src/app/application/services/nodes/` -workflow nodes
- `src/app/domain/entities/` - LangGraph entities
- `src/app/infrastructure/bus/` - Message bus
- `src/app/infrastructure/channels/` - Channel adapters

### Still Active
- `src/app/domain/ports/` - Only `llm_client_port.py` remains (used byLiteLLMAdapter)
- `src/app/application/services/` - Only `llm_validator.py` remains (used by tools)

## Active Components (Used by `nova chat`)

### Infrastructure (40 files)
- `adapters/` - Config, LiteLLM adapter
- `application/services/llm_validator.py` - Pydantic AI validation
- `domain/ports/llm_client_port.py` - LLM interface
- `domain/parsers/` - Command parsers
- `infrastructure/cron/` - Cron jobs
- `infrastructure/heartbeat/` - Heartbeat service
- `infrastructure/memory/` - Memory store
- `infrastructure/session/` - Session manager
- `infrastructure/skills/` - Skills loader
- `infrastructure/tools/` - Tool registry
- `interfaces/cli/app.py` - CLI entry point

## Why Archived?

These components implement a **LangGraph workflow architecture** that is NOT used by the current `nova chat` CLI:

### Current CLI Architecture
```
CLI → LiteLLMAdapter → Direct LLM Call → Tool Registry → Response
```

### Archived Architecture
```
CLI → Message Bus → Agent Loop → LangGraph Orchestrator
                                     ↓
                              Context Builder Node
                                     ↓
                              Intent Detector Node
                                     ↓
                              Memory Gate Node → Memory Recall Node
                                     ↓
                              LLM Node
                                     ↓
                              Tool Execution Node
                                     ↓
                              Memory Write Node
                                     ↓
                              Final Output Node
```

## Impact

- ✅ **Reduced complexity** - 18 files removed from active codebase
- ✅ **Cleaner structure** - Only CLI-relevant components remain
- ✅ **Preserved code** - All files archived for future use
- ✅ **No breaking changes** - CLI functionality unchanged

## Restoration Path

To restore archived components:

```bash
# Move files back
mv legacy/archive/src/app/* src/app/

# Recreate __init__.py files
touch src/app/domain/entities/__init__.py
touch src/app/infrastructure/bus/__init__.py
touch src/app/infrastructure/channels/__init__.py
touch src/app/application/services/nodes/__init__.py
```

## See Also

- `legacy/archive/README.md` - Detailed documentation
- `docs/ARCHITECTURE_SUMMARY.md` - Architecture overview
- `src/app/interfaces/cli/app.py` - Active CLI implementation

