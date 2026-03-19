"""CLI interface for Nova Agent."""

import asyncio
import json
import logging
import os
import sys
from pathlib import Path
from typing import Optional

# IMPORTANT: Configure logging BEFORE importing any app modules
logging.basicConfig(
    level=logging.INFO, format="%(asctime)s - %(name)s - %(levelname)s - %(message)s"
)

# Set LITELLM_LOG environment variable BEFORE importing litellm
from app.adapters.config import settings

if settings.litellm_log.upper() == "ERROR":
    os.environ["LITELLM_LOG"] = "ERROR"

import typer
from rich.console import Console
from rich.panel import Panel
from rich.prompt import Prompt

# Add src to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent.parent.parent))

from app.adapters.llm_providers.litellm_adapter import LiteLLMAdapter
from app.infrastructure.memory import MemoryStore
from app.infrastructure.session import SessionManager
from app.infrastructure.skills import SkillsLoader, ContextBuilder
from app.infrastructure.tools import (
    ToolRegistry,
    ReadFileTool,
    WriteFileTool,
    EditFileTool,
    ListDirTool,
    ExecTool,
    WebSearchTool,
    WebFetchTool,
)

# ONLY use "LiteLLM" logger (capital L), ignore "litellm" (lowercase)
lite_llm_log_level = getattr(logging, settings.litellm_log.upper(), logging.ERROR)

# Configure "LiteLLM" logger (capital L) - this is the main verbose_logger
verbose_logger = logging.getLogger("LiteLLM")
verbose_logger.setLevel(lite_llm_log_level)
verbose_logger.propagate = True

logger = logging.getLogger(__name__)

# Typer app
app = typer.Typer(help="Nova Agent CLI - Your personal AI assistant")
console = Console()

# Global workspace path
WORKSPACE = Path(os.getenv("NOVA_WORKSPACE", Path.home() / ".nova"))


def get_workspace() -> Path:
    """Get or create workspace directory."""
    workspace = Path(WORKSPACE)
    workspace.mkdir(parents=True, exist_ok=True)

    # Create subdirectories
    (workspace / "workspace").mkdir(exist_ok=True)
    (workspace / "memory").mkdir(exist_ok=True)
    (workspace / "sessions").mkdir(exist_ok=True)
    (workspace / "bootstrap").mkdir(exist_ok=True)

    return workspace


def setup_tools(workspace: Path, api_key: str) -> ToolRegistry:
    """Setup and register tools."""
    registry = ToolRegistry()

    # Filesystem tools
    allowed_dir = workspace / "workspace"
    registry.register(ReadFileTool(allowed_dir=allowed_dir))
    registry.register(WriteFileTool(allowed_dir=allowed_dir))
    registry.register(EditFileTool(allowed_dir=allowed_dir))
    registry.register(ListDirTool(allowed_dir=allowed_dir))

    # Shell tool
    registry.register(
        ExecTool(
            working_dir=str(allowed_dir),
            restrict_to_workspace=True,
            allowed_dir=allowed_dir,
        )
    )

    # Web tools
    brave_api_key = os.getenv("BRAVE_API_KEY")
    registry.register(WebSearchTool(api_key=brave_api_key))
    registry.register(WebFetchTool())

    return registry


def _onboard_bootstrap_files(ws: Path) -> dict:
    """Interactive onboarding to fill bootstrap files."""
    config = {}

    console.print("\n[bold cyan]🎉 Welcome to Nova Onboarding![/bold cyan]")
    console.print(
        "Let's personalize your AI assistant. You can skip any section by pressing Enter.\n"
    )

    # SOUL.md - Personality
    console.print("[bold]1. Personality Configuration[/bold] (SOUL.md)")
    customize_personality = Prompt.ask(
        "Would you like to customize Nova's personality? [y/n]",
        default="n",
    )

    if customize_personality == "y":
        personality = Prompt.ask(
            "Describe Nova's personality (e.g., 'friendly and professional', 'witty and casual')",
            default="helpful and friendly",
        )
        values = Prompt.ask(
            "What values should Nova prioritize? (comma-separated)",
            default="accuracy, user privacy, transparency",
        )
        comm_style = Prompt.ask(
            "Communication style [concise/detailed/balanced]",
            default="concise",
        )

        soul_content = f"""# Soul

I am Nova 🤖, a personal AI assistant.

## Personality

- {personality}
- Concise and to the point
- Curious and eager to learn

## Values

- {values}

## Communication Style

- Be clear and {comm_style}
- Explain reasoning when helpful
- Ask clarifying questions when needed
"""
        (ws / "SOUL.md").write_text(soul_content, encoding="utf-8")
        config["personality"] = personality
        console.print("[green]✓ Personality configured[/green]\n")

    # USER.md - User Profile
    console.print("[bold]2. User Profile[/bold] (USER.md)")
    fill_profile = Prompt.ask(
        "Would you like to fill in your profile? [y/n]", default="y"
    )

    if fill_profile == "y":
        name = Prompt.ask("Your name", default="")
        timezone = Prompt.ask("Your timezone", default="UTC")
        language = Prompt.ask("Preferred language", default="English")
        role = Prompt.ask("Your primary role (e.g., developer, researcher)", default="")

        comm_style = ""
        if Prompt.ask("Set communication style preference? [y/n]", default="n") == "y":
            style = Prompt.ask(
                "Style [casual/professional/technical]", default="professional"
            )
            comm_style = f"\n### Communication Style\n\n- [x] {style.capitalize()}"

        tech_level = ""
        if Prompt.ask("Set technical level? [y/n]", default="n") == "y":
            level = Prompt.ask(
                "Level [beginner/intermediate/expert]",
                default="intermediate",
            )
            tech_level = f"\n### Technical Level\n\n- [x] {level.capitalize()}"

        user_content = f"""# User Profile

Information about the user to help personalize interactions.

## Basic Information

- **Name**: {name or "(your name)"}
- **Timezone**: {timezone}
- **Language**: {language}

## Preferences
{comm_style}
{tech_level}

## Work Context

- **Primary Role**: {role or "(your role)"}
- **Main Projects**: (what you're working on)
- **Tools You Use**: (IDEs, languages, frameworks)

## Topics of Interest

-
-
-

## Special Instructions

(Any specific instructions for how the assistant should behave)

---

*Edit this file to customize Nova's behavior for your needs.*
"""
        (ws / "USER.md").write_text(user_content, encoding="utf-8")
        config["user"] = name or "User"
        console.print("[green]✓ User profile created[/green]\n")

    # AGENTS.md - Optional advanced
    console.print("[bold]3. Agent Instructions[/bold] (AGENTS.md)")
    if Prompt.ask("Customize agent instructions? [y/n]", default="n") == "y":
        guidelines = Prompt.ask(
            "Any specific guidelines for Nova?",
            default="Always explain what you're doing before taking actions",
        )

        agents_content = f"""# Agent Instructions

You are Nova 🤖, a helpful AI assistant. Be concise, accurate, and friendly.

## Guidelines

- {guidelines}
- Ask for clarification when the request is ambiguous
- Use tools to help accomplish tasks
- Remember important information in your memory files

## Tools Available

You have access to:
- File operations (read, write, edit, list)
- Shell commands (exec)
- Web access (search, fetch)
- Messaging (message)
- Background tasks (spawn)
- Scheduled reminders (cron)

## Memory

- `memory/MEMORY.md` — long-term facts (preferences, context, relationships)
- `memory/HISTORY.md` — append-only event log, search with grep to recall past events

## Scheduled Reminders

When user asks for a reminder at a specific time, use `exec` to run:
```
nova cron add --name "reminder" --message "Your message" --at "YYYY-MM-DDTHH:MM:SS" --deliver --to "USER_ID" --channel "CHANNEL"
```
Get USER_ID and CHANNEL from the current session (e.g., `8281248569` and `telegram` from `telegram:8281248569`).

**Do NOT just write reminders to MEMORY.md** — that won't trigger actual notifications.

## Heartbeat Tasks

`HEARTBEAT.md` is checked every 30 minutes. You can manage periodic tasks by editing this file:

- **Add a task**: Use `edit_file` to append new tasks to `HEARTBEAT.md`
- **Remove a task**: Use `edit_file` to remove completed or obsolete tasks
- **Rewrite tasks**: Use `write_file` to completely rewrite the task list

Task format examples:
```
- [ ] Check calendar and remind of upcoming events
- [ ] Scan inbox for urgent emails
- [ ] Check weather forecast for today
```

When the user asks you to add a recurring/periodic task, update `HEARTBEAT.md` instead of creating a one-time reminder. Keep the file small to minimize token usage.
"""
        (ws / "AGENTS.md").write_text(agents_content, encoding="utf-8")
        console.print("[green]✓ Agent instructions customized[/green]\n")

    return config


@app.command(name="onboard")
def init(
    workspace: Optional[Path] = typer.Option(
        None, "--workspace", "-w", help="Workspace directory"
    ),
    skip_onboarding: bool = typer.Option(
        False, "--skip-onboarding", help="Skip interactive onboarding"
    ),
):
    """Initialize Nova workspace and configuration."""
    if workspace:
        global WORKSPACE
        WORKSPACE = workspace

    ws = get_workspace()

    # Check if this is first-time setup BEFORE copying files
    is_fresh_setup = not (ws / "SOUL.md").exists() or not (ws / "USER.md").exists()

    # Ensure bootstrap files are copied (via SkillsLoader)
    # This copies AGENTS.md, SOUL.md, TOOLS.md, USER.md, etc.
    _ = SkillsLoader(ws)

    # Run interactive onboarding for fresh setups
    config = {}
    if is_fresh_setup and not skip_onboarding:
        config = _onboard_bootstrap_files(ws)

    # Summary panel
    setup_msg = (
        f"[bold green]Nova Workspace Initialized![/bold green]\nLocation: {ws}\n\n"
    )

    if config:
        setup_msg += "[dim]Configured:[/dim]\n"
        if "personality" in config:
            setup_msg += f"  • Personality: {config['personality']}\n"
        if "user" in config:
            setup_msg += f"  • User: {config['user']}\n"
        setup_msg += "\n"

    setup_msg += (
        f"[yellow]Next steps:[/yellow]\n"
        f"1. Set your API key: export LITE_LLM_API_KEY=your-key\n"
        f"2. Optional: Set BRAVE_API_KEY for web search\n"
        f"3. Start chatting: nova chat\n\n"
        f"[dim]You can edit bootstrap files anytime in {ws}[/dim]"
    )

    console.print(Panel.fit(setup_msg, title="Nova Agent", border_style="green"))


@app.command()
def chat(
    message: Optional[str] = typer.Option(
        None, "--message", "-m", help="Single message to send"
    ),
    workspace: Optional[Path] = typer.Option(
        None, "--workspace", "-w", help="Workspace directory"
    ),
    model: str = typer.Option(settings.lite_llm_model, "--model", help="Model to use"),
):
    """Chat with Nova agent."""
    if workspace:
        global WORKSPACE
        WORKSPACE = workspace

    ws = get_workspace()

    # Check for API key
    api_key = os.getenv("LITE_LLM_API_KEY")
    if not api_key:
        console.print(
            "[bold red]Error:[/bold red] LITE_LLM_API_KEY not set. Set it via environment variable or .env file."
        )
        console.print("Example: export LITE_LLM_API_KEY=your-api-key")
        console.print(
            "\nSupported providers: groq, openai, anthropic, zhipu, and 100+ more"
        )
        console.print(
            "Model format: provider/model-name (e.g., groq/openai/gpt-oss-20b)"
        )
        raise typer.Exit(1)

    # Initialize components
    console.print(f"[dim]Loading Nova with model: {model}...[/dim]\n")

    try:
        # Setup components
        llm = LiteLLMAdapter(
            model=model,
            api_key=api_key,
        )

        memory_store = MemoryStore(ws)
        session_manager = SessionManager(ws)
        skills_loader = SkillsLoader(ws)
        context_builder = ContextBuilder(ws, skills_loader=skills_loader)
        tools = setup_tools(ws, api_key)

        # Show status
        skills = skills_loader.load_all()
        console.print(
            f"[dim]Loaded {len(skills)} skills, {len(tools.list_tools())} tools[/dim]\n"
        )

    except Exception as e:
        console.print(f"[bold red]Error initializing Nova:[/bold red] {e}")
        raise typer.Exit(1)

    # Single message mode
    if message:
        asyncio.run(
            _process_message(
                message, llm, context_builder, session_manager, tools, "cli:single"
            )
        )
        return

    # Interactive mode
    console.print(
        Panel.fit(
            "[bold]Nova Agent[/bold] - Interactive Mode\n"
            "Type your message or use commands:\n"
            "  [dim]/new - Start new conversation[/dim]\n"
            "  [dim]/tools - List available tools[/dim]\n"
            "  [dim]/skills - List loaded skills[/dim]\n"
            "  [dim]/memory - Show long-term memory[/dim]\n"
            "  [dim]/exit - Exit chat[/dim]",
            border_style="blue",
        )
    )

    session_key = "cli:interactive"

    while True:
        try:
            user_input = Prompt.ask("\n[bold blue]You[/bold blue]").strip()

            if not user_input:
                continue

            # Handle commands
            if user_input.lower() in ["/exit", "/quit", "exit", "quit"]:
                console.print("[dim]Goodbye! 👋[/dim]")
                break

            if user_input.lower() == "/new":
                session_manager.get_or_create(session_key).clear()
                console.print("[dim]Started new conversation[/dim]")
                continue

            if user_input.lower() == "/tools":
                tool_list = "\n".join([f"  • {t}" for t in tools.list_tools()])
                console.print(
                    Panel(tool_list, title="Available Tools", border_style="green")
                )
                continue

            if user_input.lower() == "/skills":
                skill_list = "\n".join(
                    [f"  • {s.name}: {s.description}" for s in skills]
                )
                console.print(
                    Panel(skill_list, title="Loaded Skills", border_style="green")
                )
                continue

            if user_input.lower() == "/memory":
                memory_content = memory_store.read_long_term()
                if memory_content.strip():
                    console.print(
                        Panel(
                            memory_content[:500] + "..."
                            if len(memory_content) > 500
                            else memory_content,
                            title="Long-term Memory",
                            border_style="yellow",
                        )
                    )
                else:
                    console.print("[dim]No long-term memory yet[/dim]")
                continue

            # Process message
            asyncio.run(
                _process_message(
                    user_input,
                    llm,
                    context_builder,
                    session_manager,
                    tools,
                    session_key,
                )
            )

        except KeyboardInterrupt:
            console.print("\n[dim]Goodbye! 👋[/dim]")
            break
        except Exception as e:
            console.print(f"[bold red]Error:[/bold red] {e}")


async def _process_message(
    message: str,
    llm: LiteLLMAdapter,
    context_builder: ContextBuilder,
    session_manager: SessionManager,
    tools: ToolRegistry,
    session_key: str,
):
    """Process a single message with tool calling support."""
    try:
        # Get session
        session = session_manager.get_or_create(session_key)

        # Build context
        messages = context_builder.build_messages(
            history=session.get_history(max_messages=20),
            current_message=message,
        )

        # Agent loop for tool calling
        max_iterations = 10
        iteration = 0
        tools_used = []
        final_content = None

        tool_definitions = tools.get_definitions()

        while iteration < max_iterations:
            iteration += 1

            # Get response from LLM with tools
            response = await llm.chat_completion(
                messages=messages,
                tools=tool_definitions if tool_definitions else None,
                streaming=False,
                thread_id=session_key,
            )

            tool_calls = response.get("tool_calls")

            if tool_calls:
                console.print(f"\n[dim]🔧 Using {len(tool_calls)} tool(s)...[/dim]")

                # Add assistant message with tool calls
                messages.append(
                    {
                        "role": "assistant",
                        "content": response.get("response", ""),
                        "tool_calls": [
                            {
                                "id": getattr(tc, "id", f"call_{i}"),
                                "type": "function",
                                "function": {
                                    "name": tc.function.name,
                                    "arguments": json.dumps(tc.function.arguments),
                                },
                            }
                            if hasattr(tc, "function")
                            else tc
                            for i, tc in enumerate(tool_calls)
                        ],
                    }
                )

                # Execute each tool call
                for tc in tool_calls:
                    tool_name = None
                    tool_args = {}
                    tool_id = f"call_{len(tools_used)}"

                    if hasattr(tc, "function"):
                        tool_name = tc.function.name
                        tool_args = (
                            tc.function.arguments
                            if isinstance(tc.function.arguments, dict)
                            else json.loads(tc.function.arguments)
                        )
                        tool_id = getattr(tc, "id", tool_id)
                    elif isinstance(tc, dict):
                        func_info = tc.get("function", {})
                        tool_name = func_info.get("name", "")
                        tool_args = func_info.get("arguments", {})
                        tool_id = tc.get("id", tool_id)

                    if not tool_name:
                        logger.warning(f"Skipping tool call with no name: {tc}")
                        continue

                    console.print(f"  [dim]→ {tool_name}[/dim]")
                    tools_used.append(tool_name)

                    try:
                        result = await tools.execute(tool_name, tool_args)
                        tool_result = str(result)
                    except Exception as e:
                        tool_result = f"Error executing {tool_name}: {str(e)}"
                        logger.error(tool_result)

                    messages.append(
                        {
                            "role": "tool",
                            "tool_call_id": tool_id,
                            "name": tool_name,
                            "content": tool_result,
                        }
                    )
            else:
                # No tool calls, we have the final response
                final_content = response.get("response", "")
                break

        if iteration >= max_iterations:
            console.print("[yellow]⚠ Max iterations reached[/yellow]")
            final_content = "I needed too many tool uses to complete this task."

        # Display final response
        console.print("\n[bold green]Nova[/bold green]", end=" ")

        if final_content:
            console.print(final_content)
        else:
            console.print("[dim](no response)[/dim]")

        # Save to session
        session.add_message("user", message)
        session.add_message("assistant", final_content or "", tools_used=tools_used)
        session_manager.save(session)

    except Exception as e:
        console.print(f"\n[bold red]Error:[/bold red] {e}")
        logger.error(f"Error in _process_message: {e}", exc_info=True)


@app.command()
def status(
    workspace: Optional[Path] = typer.Option(
        None, "--workspace", "-w", help="Workspace directory"
    ),
):
    """Show Nova status and configuration."""
    if workspace:
        global WORKSPACE
        WORKSPACE = workspace

    ws = get_workspace()

    # Check API key
    api_key = os.getenv("LITE_LLM_API_KEY")
    api_status = "[green]✓ Configured[/green]" if api_key else "[red]✗ Not set[/red]"

    # Check Brave API
    brave_key = os.getenv("BRAVE_API_KEY")
    brave_status = (
        "[green]✓ Configured[/green]" if brave_key else "[yellow]○ Optional[/yellow]"
    )

    # Load skills
    skills_loader = SkillsLoader(ws)
    skills = skills_loader.load_all()

    # Load sessions
    session_manager = SessionManager(ws)
    sessions = session_manager.list_sessions()

    # Get active model and provider
    model = settings.lite_llm_model
    provider = "Unknown"
    model_name = model

    # Parse provider from model format "provider/model-name"
    print(model)
    if "/" in model:
        parts = model.split("/")
        provider = parts[0].capitalize()
        model_name = parts[-1]

    status_text = f"""
[bold]Workspace:[/bold] {ws}
[bold]API Key:[/bold] {api_status}
[bold]Brave API:[/bold] {brave_status}
[bold]Active Model:[/bold] {model_name}
[bold]Provider:[/bold] {provider} (using LiteLLM)
[bold]Skills:[/bold] {len(skills)} loaded
[bold]Sessions:[/bold] {len(sessions)} saved

[bold]Loaded Skills:[/bold]
"""
    for skill in skills:
        status_text += f"  • {skill.name}"
        if skill.always_load:
            status_text += " [dim](always)[/dim]"
        status_text += "\n"

    console.print(Panel(status_text, title="Nova Status", border_style="blue"))


if __name__ == "__main__":
    app()
