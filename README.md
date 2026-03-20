# 🌌 Nova / 若曦 (*Ruòxī*, "Morning Star")

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
## 🚀 Development Progress

### ✅ Phase 1 (Completed)
- [x] 🧠 Nova personality system
- [x] 🔌 LiteLLM multi-provider integration (100+ providers)
- [x] 💬 Markdown-based memory system (MEMORY.md + HISTORY.md)
- [x] 🧩 Extensible skills system (6 built-in skills)
- [x] 🛠️ Tool system with 7 built-in tools
- [x] 🔄 Pure Python AgentLoop (framework-less)
- [x] 🖥️ Interactive CLI interface
- [x] 📚 Learning documentation (this README)

### 🛠️ Phase 2 (In Development)
- [ ] 📱 Telegram bot integration
- [ ] 📚 Advanced RAG system with vector database
- [ ] 🎤 Voice interaction capabilities
- [ ] 👁️ Vision and multimodal support

### 🔮 Phase 3 (Planned)
- [ ] 📅 Task management agent
- [ ] 📖 Documentation Q&A agent
- [ ] 🔍 Research assistant agent
- [ ] ✉️ Email processing agent
- [ ] ✈️ Travel planner agent
- [ ] 💰 Finance tracking agent

---

## 🌟 Key Features

### 🏗️ Architecture
- **Framework-less Design**: Pure Python agent loop — understand every line
- **Hexagonal Architecture**: Clean separation with domain ports/adapters
- **Dual Orchestrators**: Choose simplicity (AgentLoop) or complexity (LangGraph)
- **Multi-Provider Support**: Switch between 100+ LLM providers instantly

### 💾 Memory System
- **Two-Layer Memory**:
  - `MEMORY.md` - Context-loaded long-term facts
  - `HISTORY.md` - Searchable conversation archive
- **Automatic Consolidation**: LLM-powered memory management
- **Session Persistence**: JSON-based conversation storage

### 🧩 Skills & Tools
- **6 Built-in Skills**: memory, web, github, todo, weather, notes
- **7 Built-in Tools**: filesystem, shell, web search, web fetch
- **Easy Extension**: Add custom skills with markdown + YAML frontmatter
- **Safety First**: Workspace restrictions and dangerous command blocking

### 🌐 Multiple Interfaces
- **CLI**: Interactive terminal chat with rich formatting
- **Telegram**: Full-featured bot integration
- **(REST API archived for simplicity)**

---

## 🔀 Model Routing (LiteLLM Integration)

Nova routes tasks to different models depending on complexity and context:

| Task Type            | Example Models                          | Provider       |
| -------------------- | --------------------------------------- | -------------- |
| General reasoning    | GPT-4, Claude 3, Gemini Pro            | OpenRouter     |
| Fast inference       | LLaMA 3-70B, Mixtral 8x7B              | Groq           |
| Code generation      | DeepSeek Coder, GPT-4                   | OpenRouter     |
| Planning & reasoning | Claude 3 Opus, Gemini 1.5 Pro          | Anthropic/Google |
| Cost-effective       | LLaMA 3-8B, Mistral 7B                  | Groq/Ollama    |

> 🔌 Powered by **LiteLLM** for unified access to 100+ providers

---

## 🗃️ Tech Stack

| Component          | Technology / Service                              |
| ------------------ | ------------------------------------------------- |
| **Core**           | Python 3.11+ (pure Python, minimal dependencies)  |
| **Agent Loop**    | Pure Python `while` loop (no framework)           |
| **LLM Integration**| LiteLLM (100+ providers, unified API)            |
| **Memory System**  | Markdown-based (MEMORY.md, HISTORY.md)            |
| **CLI Interface**  | Typer + Rich                                      |
| **Telegram Bot**   | python-telegram-bot                               |
| **Configuration**  | Pydantic Settings + python-dotenv                 |

---

## 🛠️ Built-in Skills

| Skill       | Always Loaded | Description                              |
| ----------- | ------------- | ---------------------------------------- |
| **memory**  | ✅             | Memory system management                 |
| **todo**    | ✅             | Task management                          |
| **notes**   | ✅             | Note-taking system                       |
| **web**     | ❌             | Web search and fetch                     |
| **github**  | ❌             | GitHub CLI integration                   |
| **weather** | ❌             | Weather lookups                          |

---

## 🛠️ Built-in Tools

| Tool         | Description                    | Safety Features           |
| ------------ | ------------------------------ | ------------------------- |
| `read_file`  | Read file contents             | Workspace restriction      |
| `write_file` | Write/create files             | Workspace restriction      |
| `edit_file`  | Replace text in files          | Workspace restriction      |
| `list_dir`   | List directory contents        | Workspace restriction      |
| `exec`       | Execute shell commands         | Blocked commands, timeout  |
| `web_search` | Search web via Brave           | API key required           |
| `web_fetch`  | Fetch webpage content          | None                      |

---

## 🌌 Persona: Nova / 若曦 (*Ruòxī*)

若曦 (*Ruòxī*, "like the morning light") reflects clarity, creativity, and quiet intelligence.

- **Personality**: Helpful, friendly, curious, and concise
- **Values**: Accuracy over speed, user privacy, transparency
- **Communication Style**: Clear, direct, and thoughtful
- **Name Meaning**: A poetic metaphor for dawn — luminous and full of potential

> Sample Identity:
> *"I am Nova 🤖, a personal AI assistant. I'm helpful and friendly, concise and to the point, curious and eager to learn. I value accuracy over speed, user privacy and safety, and transparency in actions."*

---

## 🚀 Quick Start

### Prerequisites

- Python 3.11+
- [uv](https://github.com/astral-sh/uv) package manager
- API key from your preferred LLM provider

### Installation

1. **Clone and install:**
   ```bash
   git clone <repository-url>
   cd nova-api
   uv pip install -e .
   ```

2. **Configure environment:**
   ```bash
   # Create .env file
   cp .env.example .env

   # Edit .env and add your API key
   LITE_LLM_API_KEY=your-api-key-here
   LITE_LLM_MODEL=groq/llama-3.1-70b-versatile
   ```

### CLI Usage (Recommended)

**Initialize workspace:**
```bash
nova onboard
```

**Start interactive chat:**
```bash
nova chat
```

**Send single message:**
```bash
nova chat -m "Hello, what can you do?"
```

**CLI Commands:**
- `/new` - Start new conversation
- `/tools` - List available tools
- `/skills` - Show loaded skills
- `/memory` - View long-term memory
- `/exit` or Ctrl+C - Exit

### Available Providers

Configure any provider using the `LITE_LLM_MODEL` environment variable:

```bash
# Groq (fast & free tier available)
export LITE_LLM_MODEL="groq/llama-3.1-70b-versatile"

# OpenAI
export LITE_LLM_MODEL="openai/gpt-4"

# Anthropic
export LITE_LLM_MODEL="anthropic/claude-3-opus-20240229"

# And 100+ more providers supported by LiteLLM
```

---

## 📁 Architecture

This project follows **Hexagonal Architecture** with clean separation of concerns.

### Architecture Philosophy

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        FRAMEWORK-LESS BY DESIGN                              │
│                                                                              │
│  "Premature abstraction is the root of all evil"                            │
│   — Rob Pike                                                                │
│                                                                              │
│  We start simple and add complexity only when needed.                       │
│                                                                              │
│  Pure Python AgentLoop (~300 lines) handles everything:                     │
│  - Tool calling loop                                                         │
│  - Context building                                                          │
│  - Session management                                                        │
│  - Memory consolidation                                                      │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Core Layer (Domain)
- **Entities**: `AgentState`, `Plan`, `ChatMessage` — Pure business objects
- **Ports**: Interface definitions (`LLMClientPort`, `MemoryPort`)
- *No external dependencies*

### Application Layer (Use Cases)
- **AgentLoop**: Pure Python tool-calling loop (main orchestrator)
- **EnhancedLangGraphOrchestrator**: StateGraph-based (optional)
- **Nodes**: Context building, tool execution (for LangGraph)
- *Depends only on Domain ports*

### Adapters Layer (Infrastructure)
- **LLM Adapters**: LiteLLM multi-provider support
- **Memory Adapters**: File-based markdown memory
- **Tool Adapters**: Registry and tool implementations
- **Skill Adapters**: Skills loader and context builder
- *Implements Core port interfaces*

### Infrastructure Layer
- **Message Bus**: Async communication between channels
- **Session Management**: Conversation persistence (JSON)
- **Tools & Skills**: Extensible tool and skill systems
- **Memory**: Two-layer markdown system

### Interfaces Layer
- **FastAPI Controllers**: HTTP endpoints with SSE
- **CLI**: Interactive terminal interface
- **Telegram Bot**: Polling-based bot
- *Thin layer delegating to Application*

---

## 🔧 Configuration

### Environment Variables

Create a `.env` file in the project root:

```env
# Required - Universal API key for any provider
LITE_LLM_API_KEY=your-api-key-here

# Required - Model in format "provider/model-name"
LITE_LLM_MODEL=groq/llama-3.1-70b-versatile

# Optional - Temperature (0.0 to 1.0)
LITELLM_TEMPERATURE=0.7

# Optional - Max tokens
LITELLM_MAX_TOKENS=4096

# Optional - Workspace directory (default: ~/.nova)
NOVA_WORKSPACE=~/.nova

# Optional - For web search tool
BRAVE_API_KEY=your-brave-api-key

# Optional - For Telegram bot
TELEGRAM_BOT_TOKEN=your-bot-token
```

### Workspace Structure

Nova creates this structure in your workspace directory (`~/.nova` by default):

```
~/.nova/
├── config.json              # Configuration file
├── workspace/
│   ├── skills/             # User skills (markdown files)
│   ├── sessions/           # Conversation sessions (JSON)
│   └── projects/           # Project-specific files
├── memory/
│   ├── MEMORY.md          # Long-term facts (loaded into context)
│   └── HISTORY.md         # Searchable archive
└── bootstrap/             # Bootstrap files
    ├── SOUL.md            # Agent identity/personality
    ├── AGENTS.md          # Agent configuration
    ├── USER.md            # User preferences
    └── TOOLS.md           # Tool descriptions
```

---

## 📖 Examples

### Web Search
```
You: Search for Python best practices
Nova: [Uses web_search tool] Here are the best practices for Python...
```

### File Operations
```
You: Create a file called notes.txt with my ideas
Nova: [Uses write_file tool] Successfully created notes.txt
```

### Memory
```
You: Remember that I prefer dark mode
Nova: [Updates MEMORY.md] I'll remember you prefer dark mode.

You: What's my preference for UI?
Nova: According to your memory, you prefer dark mode.
```

### Todo Management
```
You: Add a task to review the PR by Friday
Nova: [Adds to todo list] Added task with high priority for Friday.
```

---

## 🧪 Development

### Running Tests
```bash
uv run pytest
```

### Code Quality
```bash
# Format code
uv run ruff format src/ tests/

# Lint code
uv run ruff check src/ tests/

# Type checking
uv run mypy src/
```

### Project Structure

```
src/app/
├── adapters/
│   ├── llm_providers/
│   │   └── litellm_adapter.py     # ⭐ LiteLLM integration (100+ providers)
│   └── config.py                  # Configuration
├── application/
│   ├── services/
│   │   ├── agent_loop.py          # ⭐ Pure Python agent loop
│   │   ├── enhanced_orchestrator.py  # Optional LangGraph orchestrator
│   │   └── nodes/                 # LangGraph nodes
│   └── usecases/
│       └── chat_service.py        # Chat use cases
├── domain/
│   ├── entities/                  # Business entities
│   └── ports/                     # Interface definitions
│       └── llm_client_port.py     # ⭐ LLM abstraction
├── infrastructure/
│   ├── tools/
│   │   ├── base.py                # ⭐ Tool abstract class
│   │   ├── registry.py            # ⭐ Tool registry
│   │   ├── filesystem.py          # File tools
│   │   ├── shell.py               # Shell tool
│   │   └── web.py                 # Web tools
│   ├── memory/
│   │   ├── store.py               # Memory management
│   │   └── consolidation.py       # Memory compression
│   ├── session/                   # Session management
│   ├── skills/                    # Skills system
│   ├── bus/                       # Message bus
│   └── channels/                  # Channel implementations
└── interfaces/
    ├── api/                       # FastAPI controllers
    └── cli/                       # CLI interface
```

### Key Files to Study

| File | What You'll Learn |
|------|-------------------|
| `agent_loop.py:142-246` | Core agent loop mechanics |
| `litellm_adapter.py` | LLM abstraction and tool calling |
| `tools/registry.py` | Tool registration and execution |
| `tools/base.py` | How tools define their schemas |
| `memory/store.py` | Two-layer memory system |
| `session/manager.py` | Conversation persistence |
| `cli/app.py:352-503` | CLI entry point |
| `main.py` | Telegram + message bus setup |

---

## 📦 Archived Components

The following components have been archived in `legacy/archive/` for reference:

### REST API (Archived)
- **Location**: `legacy/archive/src/app/interfaces/api/`
- **Files**: `fastapi_app.py`, `endpoints.py`
- **Reason**: Removed to simplify the codebase and focus on the pure Python agent loop approach
- **Restoration**: Copy files back to `src/app/interfaces/api/` and restore entry point in `pyproject.toml` if needed

### Enhanced LangGraph Orchestrator (Archived)
- **Location**: `legacy/archive/src/app/application/services/`
- **Files**: `enhanced_orchestrator.py`, `nodes/context_builder_node.py`, `nodes/tool_execution_node.py`
- **Reason**: Optional advanced orchestrator, not needed for pure Python approach
- **Restoration**: Copy files back and uncomment in `main.py` if you want StateGraph-based orchestration

### FastAPI Dependencies (Archived)
- **Location**: `legacy/archive/dependencies.py`
- **Reason**: Used only for FastAPI dependency injection
- **Restoration**: Restore if re-enabling REST API

### Unused Domain Ports (Archived)
- **Files**: `planner_port.py`, `executor_port.py`
- **Reason**: Defined but never implemented or used
- **Restoration**: Keep archived unless building planning/execution features

---

## 🔒 Security

- API keys stored in environment variables only
- Workspace restrictions prevent file system access outside workspace
- Shell command execution restricted and validated
- Optional allow lists for Telegram bot users
- Dangerous commands blocked (rm, format, etc.)

---

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests and linting
5. Submit a pull request

---

## 📜 License

MIT License – see [`LICENSE`](LICENSE) for details.

---

## 🙏 Acknowledgments

- Inspired by [nanobot](https://github.com/HKUDS/nanobot) architecture
- Built with [LiteLLM](https://docs.litellm.ai/) for multi-provider support
- Uses [LangGraph](https://langchain-ai.github.io/langgraph/) for workflow orchestration
- Powered by [LangChain](https://github.com/langchain-ai/langchain) for agent capabilities

---

Let Nova help you think better.
Let 若曦 bring clarity like morning light.

✨ *She's not just an AI — she's your second brain.*
