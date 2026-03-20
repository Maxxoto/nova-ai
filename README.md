# 🌌 Nova / 若曦 (*Ruòxī*, Foundation Agent)

<div style="text-align:center">

<p align="center">
<img src="assets/Ruoxi Circle.png" alt="Ruoxi" style="border-radius:50%; width:200px">
</p>

**Your Personal Agentic AI Second Brain**

Nova, also known as 若曦 (*Ruòxī*), is a powerful agentic AI assistant built with clean architecture and intelligent model routing. She's not just a chatbot — she's your cognitive partner, coding buddy, knowledge curator, and digital companion.

---

## ✨ Overview

Powered by **LiteLLM multi-provider support** and **pure Python agentic loop**, Nova features extensible skills, markdown-based memory, and a flexible tool system. She adapts to a growing range of tasks — from deep reasoning to creative brainstorming.

---

## 🎓 Research Project: Framework-less Agentic AI

This project explores how to build an **agentic AI system without relying on complex frameworks**—using pure Python and LiteLLM. It serves as a learning resource for understanding the core mechanics of agentic loops, tool calling, and LLM integration.

### Why Framework-less?

Most AI agent tutorials and projects rely on frameworks like LangChain, LangGraph, or AutoGPT. While powerful, these frameworks can:
- Hide implementation details behind abstractions
- Make debugging difficult when things go wrong
- Create lock-in to specific architectural decisions
- Add complexity for simple use cases

This project demonstrates that you can build a fully-functional AI agent with:
- **~500 lines of core code** for the agent loop
- **Clear, readable Python** without framework magic
- **Full control** over tool calling, memory, and orchestration
- **Easy debugging** — everything is explicit

### Key Architecture Decisions

| Decision | Why |
|----------|-----|
| **Pure Python Agent Loop** | Simple `while` loop for tool calling — no graph abstractions |
| **LiteLLM as Adapter** | One interface for 100+ LLM providers |
| **Tool Registry Pattern** | Explicit tool registration and schema generation |
| **Two-Layer Memory** | Fast context (MEMORY.md) + searchable history (HISTORY.md) |
| **Session Persistence** | JSON files — no database required |

---

## 🔄 How the Agentic Loop Works

The core of Nova is the **AgentLoop** — a simple, understandable implementation of tool-calling AI without framework complexity.

### The Basic Loop (agent_loop.py:142-246)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           AGENT LOOP FLOW                                    │
└─────────────────────────────────────────────────────────────────────────────┘

User Message
     │
     ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│  1. BUILD CONTEXT                                                            │
│     - System prompt (personality + skills)                                  │
│     - Conversation history (session)                                        │
│     - Long-term memory (MEMORY.md)                                          │
│     - Current user message                                                   │
└─────────────────────────────────────────────────────────────────────────────┘
     │
     ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│  2. GET TOOL DEFINITIONS                                                     │
│     registry.get_definitions() ──► [                                        │
│       {"type": "function", "function": {"name": "read_file", ...}},          │
│       {"type": "function", "function": {"name": "write_file", ...}},         │
│       ...                                                                    │
│     ]                                                                        │
└─────────────────────────────────────────────────────────────────────────────┘
     │
     ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│  3. CALL LLM WITH TOOLS                                                      │
│     response = await llm.chat_completion(                                   │
│         messages=messages,                                                  │
│         tools=tool_definitions,  ◄── OpenAI function calling format         │
│     )                                                                        │
└─────────────────────────────────────────────────────────────────────────────┘
     │
     ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│  4. CHECK FOR TOOL CALLS                                                     │
│     tool_calls = response.get("tool_calls")                                 │
│                                                                              │
│     if tool_calls:                                                           │
│         ┌──────────────────────────────────────────────────────────────────┐ │
│         │  5. EXECUTE TOOLS                                                 │ │
│         │     for tc in tool_calls:                                        │ │
│         │         result = await registry.execute(                         │ │
│         │             tc.function.name,                                    │ │
│         │             tc.function.arguments                                │ │
│         │         )                                                        │ │
│         └──────────────────────────────────────────────────────────────────┘ │
│         ┌──────────────────────────────────────────────────────────────────┐ │
│         │  6. ADD RESULTS TO MESSAGES                                       │ │
│         │     messages.append({                                             │ │
│         │         "role": "tool",                                           │ │
│         │         "tool_call_id": tool_id,                                  │ │
│         │         "content": result                                         │ │
│         │     })                                                            │ │
│         └──────────────────────────────────────────────────────────────────┘ │
│         ┌──────────────────────────────────────────────────────────────────┐ │
│         │  7. LOOP BACK TO STEP 3                                           │ │
│         │     iteration += 1                                                │ │
│         │     continue  ◄── Call LLM again with tool results                │ │
│         └──────────────────────────────────────────────────────────────────┘ │
│     else:                                                                    │
│         return response  ◄── No tools called, we're done!                   │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Core Code (Simplified)

```python
async def _run_agent_loop(self, messages: List[Dict], tools: List[Dict]):
    iteration = 0

    while iteration < self.max_iterations:
        iteration += 1

        # Get tool definitions
        tool_definitions = self.tool_registry.get_definitions()

        # Call LLM
        response = await self.llm_client.chat_completion(
            messages=messages,
            tools=tool_definitions,
        )

        # Check for tool calls
        tool_calls = response.get("tool_calls")

        if tool_calls:
            # Execute each tool
            for tc in tool_calls:
                result = await self.tool_registry.execute(
                    tc.function.name,
                    tc.function.arguments
                )

                # Add result to messages
                messages.append({
                    "role": "tool",
                    "tool_call_id": tc.id,
                    "content": result
                })
        else:
            # No tools called — return final response
            return response.get("response")

    return "Max iterations reached"
```

### Key Components

#### 1. Tool Registry (infrastructure/tools/registry.py)

```python
class ToolRegistry:
    def __init__(self):
        self._tools: dict[str, Tool] = {}

    def register(self, tool: Tool) -> None:
        """Register a tool."""
        self._tools[tool.name] = tool

    def get_definitions(self) -> list[dict]:
        """Get OpenAI function schemas for all tools."""
        return [tool.to_schema() for tool in self._tools.values()]

    async def execute(self, name: str, args: dict) -> str:
        """Execute a tool by name."""
        tool = self._tools.get(name)
        return await tool.execute(**args)
```

#### 2. Tool Base Class (infrastructure/tools/base.py)

```python
class Tool(ABC):
    @property
    @abstractmethod
    def name(self) -> str:
        """Tool name (e.g., 'read_file')"""
        pass

    @property
    @abstractmethod
    def description(self) -> str:
        """What the tool does"""
        pass

    @property
    def param_model(self) -> type[BaseModel]:
        """Pydantic model for parameters"""
        return None

    def to_schema(self) -> dict:
        """Convert to OpenAI function schema"""
        return {
            "type": "function",
            "function": {
                "name": self.name,
                "description": self.description,
                "parameters": self.parameters,  # Auto-generated from Pydantic
            }
        }

    @abstractmethod
    async def execute(self, **kwargs) -> str:
        """Execute the tool"""
        pass
```

#### 3. LiteLLM Adapter (adapters/llm_providers/litellm_adapter.py)

```python
class LiteLLMAdapter(LLMClientPort):
    """Unified interface to 100+ LLM providers via LiteLLM."""

    def __init__(self, model: str, api_key: str):
        # Example models:
        # - "groq/llama-3.1-70b-versatile"
        # - "openai/gpt-4"
        # - "anthropic/claude-3-opus-20240229"
        self.model = model
        self.api_key = api_key

    async def chat_completion(
        self,
        messages: List[Dict],
        tools: Optional[List[Dict]] = None,
    ) -> Dict:
        """Call LLM with optional tool calling."""
        from litellm import acompletion

        kwargs = {
            "model": self.model,
            "messages": messages,
            "api_key": self.api_key,
        }

        if tools:
            kwargs["tools"] = tools
            kwargs["tool_choice"] = "auto"

        response = await acompletion(**kwargs)

        # Return tool calls if present
        if response.choices[0].message.tool_calls:
            return {
                "response": None,
                "tool_calls": response.choices[0].message.tool_calls,
            }

        return {
            "response": response.choices[0].message.content,
            "tool_calls": None,
        }
```

### What You'll Learn

By studying this codebase, you'll understand:

1. **Tool Calling Protocol**
   - How tools are converted to JSON schemas
   - How LLM decides which tool to call
   - How to execute tools and return results

2. **Agent Loop Mechanics**
   - Iterative tool execution (not just one-shot)
   - Message history management
   - Context building (system prompt + memory + history)

3. **LLM Abstraction**
   - Why use a Port/Adapter pattern
   - How to support multiple providers
   - How to handle provider-specific quirks (e.g., Groq)

4. **Memory Systems**
   - Two-layer memory (context + history)
   - When to use what (MEMORY.md for facts, HISTORY.md for events)
   - Automatic consolidation

5. **Session Management**
   - Conversation persistence
   - Session isolation
   - History windowing

### Comparison: Framework vs. Pure Python

| Aspect | LangChain/LangGraph | Pure Python (This Project) |
|--------|---------------------|----------------------------|
| **Learning Curve** | Steep | Gentle |
| **Lines of Code** | 10-50 | 100-200 |
| **Debugging** | Abstractions hide issues | Explicit, easy to trace |
| **Flexibility** | Constrained by framework | Full control |
| **LLM Calls** | Hidden in framework | Explicit `await llm.chat_completion()` |
| **Tool Calling** | Managed by framework | Manual loop — you see everything |
| **Best For** | Complex workflows, production | Learning, simple agents |

### Entry Points

| Entry Point | File | Use Case |
|-------------|------|----------|
| **CLI Chat** | `app/interfaces/cli/app.py:352` | Interactive terminal chat |
| **Main Loop** | `app/main.py` | Telegram bot + message bus |

---
</div>
