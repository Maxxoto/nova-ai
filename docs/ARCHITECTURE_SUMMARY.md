# Nova AI Architecture Summary

## Active Components (Used by `nova chat`)

```
nova chat
  │
  ├─ CLI Interface (app.py)
  │   └─ Interactive loop with direct LLM calling
  │
  ├─ LLM Adapter
  │   └─ LiteLLMAdapter → Multi-provider support (Groq, OpenAI, Anthropic, Zhipu)
  │
  ├─ Tools
  │   ├─ ToolRegistry → Register and execute tools
  │   ├─ Filesystem Tools → Read, write, edit, list
  │   ├─ Shell Tools → Execute commands
  │   └─ Web Tools → Search, fetch
  │
  ├─ Session Management
  │   └─ SessionManager → Persist conversation history
  │       └─ ~/.nova/sessions/*.json
  │
  ├─ Memory System
  │   ├─ MemoryStore → Long-term memory storage
  │   │   └─ ~/.nova/memory/*.json
  │   └─ MemoryConsolidator → Summarize old conversations
  │
  └─ Skills System
      ├─ SkillsLoader → Load SKILL.md files
      │   └─ ~/.nova/skills/*/SKILL.md
      └─ ContextBuilder → Build system prompt with skills + memory
```

## Archived Components (NOTused by `nova chat`)

Moved to `legacy/archive/` on 2026-03-21.

### LangGraph Workflow
- `enhanced_orchestrator.py` - State-based workflow orchestrator
- `nodes/context_builder_node.py` - Build context inworkflow
- `nodes/tool_execution_node.py` - Execute tools in workflow

### Alternative Agent Loop
- `agent_loop.py` - Message bus-based agent loop

### Message Bus
- `bus/queue.py` - Async message queue
- `bus/events.py` - Inbound/outbound message events

### Channels
- `channels/telegram.py` - Telegram bot adapter
- `channels/base.py` - Channel interface

### Domain Entities
- `entities/agent_state.py` - LangGraph state model
- `entities/agent_state_models.py` - Pydantic models
- `entities/plan.py` - Planning entity
- `entities/chat_message.py` - Chat message entity

### Domain Ports (Unused)
- `ports/memory_port.py` - Memory interface
- `ports/executor_port.py` - Executor interface
- `ports/planner_port.py` - Planner interface

### Dependency Injection
- `dependencies.py` - Service container setup

## Key Differences

| Aspect | Active (CLI) | Archived (LangGraph) |
|--------|-------------|----------------------|
| **Architecture** | Simple loop | State machine |
| **LLM Calls** | Direct via LiteLLM | Through workflow nodes |
| **Tool Execution** | Inline in `_process_message` | Dedicated nodes |
| **State Management** | Session-based | LangGraph checkpoint |
| **Complexity** | Low | High |
| **Flexibility** | Limited | Extensible |

## Why Archived?

1. **Not Used**: CLI uses direct LLM calling, not LangGraph workflow
2. **Simpler is Better**: Current CLI architecture is simpler and faster
3. **Future Ready**: Can be reactivated for web/API/Telegram interfaces

## Benefits of Current Architecture

- ✅ **Simpler**: Direct LLM calls without workflow overhead
- ✅ **Faster**: No state machine transitions
- ✅ **Easier to Debug**: Linear flow instead of node graph
- ✅ **Less Memory**: No state checkpoints
- ✅ **More Reliable**: Fewer moving parts

## When to Reactivate

Consider reactivating archived components when:

1. **Adding Web Interface**
   - Need LangGraph workflow for complex multi-step operations
   - Need state management for concurrent users

2. **Adding Telegram Bot**
   - Need message bus for async message handling
   - Need channel adapter for Telegram API

3. **Adding Planning System**
   - Need intent detection and planning nodes
   - Need multi-turn workfloworchestration

4. **Adding API Interface**
   - Need structured input/output
   - Need workflow for complex requests

## Directory Structure (After Archiving)

```
src/app/
├── adapters/
│   ├── config.py
│   └── llm_providers/
│       └── litellm_adapter.py
│
├── application/services/
│   └── llm_validator.py
│
├── domain/
│   ├── parsers/
│   └── ports/
│       └── llm_client_port.py
│
├── infrastructure/
│   ├── memory/
│   ├── session/
│   ├── skills/
│   └── tools/
│
└── interfaces/
    └── cli/
        └── app.py
```

## Related Documentation

- **Active Architecture**: See `README.md` and `docs/PYDANTIC_AI_INTEGRATION.md`
- **Archived Components**: See `legacy/archive/README.md`
- **Tool System**: See `src/app/infrastructure/tools/README.md` (if exists)
- **Skills System**: See `src/app/infrastructure/skills/` (SKILL.md files)

---

**Last Updated**: 2026-03-21
**Archived Components**: 18 files moved to `legacy/archive/`
**Active Components**: CLI + Infrastructure + Adapters