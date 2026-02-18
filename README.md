# 🌌 Nova / 若曦 (*Ruòxī*, "Morning Star")

<div style="text-align:center">

<p align="center">
<img src="assets/Ruoxi Circle.png" alt="Ruoxi" style="border-radius:50%; width:200px">
</p>

**Your Personal Agentic AI Second Brain**

Nova, also known as 若曦 (*Ruòxī*), is a powerful agentic AI assistant built with clean architecture and intelligent model routing. She's not just a chatbot — she's your cognitive partner, coding buddy, knowledge curator, and digital companion.

---

## ✨ Overview

Powered by **LiteLLM multi-provider support** and **LangGraph orchestration**, Nova features extensible skills, markdown-based memory, and a flexible tool system. She adapts to a growing range of tasks — from deep reasoning to creative brainstorming.

---
</div>
## 🚀 Development Progress

### ✅ Phase 1 (Completed)
- [x] 🧠 Nova personality system
- [x] 🔌 LiteLLM multi-provider integration (100+ providers)
- [x] 💬 Markdown-based memory system (MEMORY.md + HISTORY.md)
- [x] 🧩 Extensible skills system (6 built-in skills)
- [x] 🛠️ Tool system with 7 built-in tools
- [x] 🔄 LangGraph workflow orchestration
- [x] 🖥️ Interactive CLI interface
- [x] 🌐 FastAPI REST API with SSE support

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

### 🧠 Intelligent Architecture
- **Hexagonal Architecture**: Clean separation of concerns with domain-driven design
- **LangGraph Orchestration**: Complex workflow management with stateful agents
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
- **FastAPI**: REST API with Server-Sent Events (SSE)
- **Telegram**: Full-featured bot integration (coming soon)

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
| **Core Framework** | Python 3.11+, FastAPI, LangGraph, LangChain      |
| **LLM Integration**| LiteLLM (100+ providers)                          |
| **Memory System**  | Markdown-based (MEMORY.md, HISTORY.md)            |
| **Orchestration**  | LangGraph with stateful agents                    |
| **CLI Interface**  | Typer + Rich                                      |
| **API Interface**  | FastAPI + SSE Starlette                           |
| **Configuration**  | Pydantic Settings + python-dotenv                 |
| **Testing**        | Pytest + pytest-asyncio                           |

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

This project follows **Hexagonal Architecture** with clear separation of concerns:

### Core Layer (Domain)
- **Entities**: `AgentState`, `Plan`, `ChatMessage` - Pure business objects
- **Ports**: Interface definitions (`LLMClientPort`, `MemoryPort`, `ToolPort`)
- *No external dependencies*

### Application Layer (Use Cases)
- **Services**: LangGraph orchestration, enhanced workflow
- **Use Cases**: Business logic orchestration
- *Depends only on Core ports*

### Adapters Layer (Infrastructure)
- **LLM Adapters**: LiteLLM multi-provider support
- **Memory Adapters**: File-based markdown memory
- **Tool Adapters**: Registry and tool implementations
- **Skill Adapters**: Skills loader and context builder
- *Implements Core port interfaces*

### Infrastructure Layer
- **Message Bus**: Async communication between channels
- **Session Management**: Conversation persistence
- **Tools & Skills**: Extensible tool and skill systems

### Interfaces Layer
- **FastAPI Controllers**: HTTP endpoints with SSE
- **CLI**: Interactive terminal interface
- **Telegram Bot**: Polling-based bot (coming soon)
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
│   │   └── litellm_adapter.py    # LiteLLM multi-provider
│   └── config.py                  # Configuration
├── application/
│   ├── services/
│   │   └── langgraph_orchestrator.py  # Workflow orchestration
│   └── usecases/
│       └── chat_service.py        # Chat use cases
├── domain/
│   ├── entities/                  # Business entities
│   └── ports/                     # Interface definitions
├── infrastructure/
│   ├── tools/                     # Tool implementations
│   ├── memory/                    # Memory system
│   ├── session/                   # Session management
│   ├── skills/                    # Skills system
│   ├── bus/                       # Message bus
│   └── channels/                  # Channel implementations
└── interfaces/
    ├── api/                       # FastAPI controllers
    └── cli/                       # CLI interface
```

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
