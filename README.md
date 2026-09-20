# 🌌 Ruòxī / 若曦 — point at anything, then ask

<div style="text-align:center">

<p align="center">
<img src="assets/Ruoxi Circle.png" alt="Ruoxi" style="border-radius:50%; width:200px">
</p>

**A menu-bar AI companion for your screen**

Ruòxī lives in your menu bar. Press a shortcut and box something on screen, hold one key and say what you want to know, and the answer appears in a panel beside your work — cited to the capture it came from — instead of in another window. The voice and the models run on this Mac; the brain underneath is a pure-Python agent loop you can read end to end.

No account, no cloud sync, no capture you did not ask for.

</div>

---

## 🎬 Demo

<p align="center">
  <a href="docs/demo/ruoxi-demo.mp4">
    <img src="docs/demo/ruoxi-demo.webp" alt="Ruòxī demo — the panel opens ready, hold to talk, and the answer arrives beside your work" width="900">
  </a>
  <br>
  <sub>Click the preview for the full-quality MP4 (19.5s · 1920×1080). Every frame is a real screenshot of an app window — no desktop.</sub>
</p>

The clip walks the loop panel-first in under 20 seconds: the result panel opens **Ready** with your capture as its context, you hold <kbd>⌥⇧V</kbd> and speak, it **Transcribes** your question, and the answer lands beside your work — cited to the capture you made. It closes on the six-step setup, where the speech, voice and language-model choices stay on this Mac.

Made with [HyperFrames](https://github.com/heygen-com/hyperframes) — HTML in, deterministic MP4 out. Rebuild it any time:

```bash
cd docs/demo && npx hyperframes render -o ruoxi-demo.mp4
```

The composition, its fonts and the source captures live in [`docs/demo/`](docs/demo/).

---

## ✨ What it does

| | |
|---|---|
| **Capture** | <kbd>⌥⇧R</kbd> box a region · <kbd>⌥⇧W</kbd> the frontmost window · <kbd>⌥⇧F</kbd> the whole screen. All three are rebindable in Settings. |
| **Ask by voice** | Hold <kbd>⌥⇧V</kbd> — or hold the pill in the panel — and speak. The question is transcribed locally (Whisper or Parakeet, on this Mac). |
| **The panel** | Opens **Ready** with your capture as context, then **Listening → Transcribing → Thinking → Writing → Complete**. The answer cites the capture it came from. |
| **Never in the way** | The panel floats above every app, follows you across Spaces, and never takes keyboard focus from what you are doing. |
| **Out of the way, fast** | <kbd>Esc</kbd> cancels an in-flight ask (it stops listening/transcribing without dismissing), or dismisses the panel. |
| **Read aloud** | On-device Kokoro, or the macOS system voice — with a speaking state in the panel. <kbd>Esc</kbd> stops the audio. |
| **Keep it** | Every capture lands in a local, date-sharded store with sha256 dedupe; the Timeline window browses it, and answers can be grounded in what you kept. |
| **Your call** | Offline mode keeps everything on this Mac — cloud answers are simply unavailable while it is on. Auto-capture is off for every app by default. |

---

## 🖥 The app surfaces

| Surface | What it is |
|---|---|
| **Result panel** | The floating answer card: orb + state label, trust chip (`Local Only` / `Sending to Cloud` / `Offline`), the capture context row, the hold-to-talk pill, the answer with its citation, and `Save to memory` / `Read aloud`. |
| **Region overlay** | The dim-and-box overlay for <kbd>⌥⇧R</kbd>: sharp selection, corner handles, a live `w × h` chip, and an `Esc` hint below the notch. |
| **Setup** | The six-step first-run ritual — **Welcome · Access · Capture · Defaults · Models · Ready** — each permission with its reason, one guided capture, and the model choices. Replayable from Settings → Permissions → *Run setup again*. |
| **Settings** | Voice & answers, **Models** (speech-to-text, text-to-speech, language model), Permissions, Display, theme and hotkeys. |
| **Timeline** | Everything you captured, day by day — thumbnails, stats, and delete. |
| **Menu bar** | `Capture Region · Capture Window · Capture Whole Screen · Timeline · Pause Captures · Offline Mode · Settings… · Quit Ruòxī`, and a status glyph that mirrors listening / paused. |

---

## 🚀 Install & run

**Prerequisites:** macOS for the tray shell; Python 3.11+ with [`uv`](https://docs.astral.sh/uv/); Node + `pnpm` and a Rust toolchain for the desktop shell.

From the repo root, the `make` targets cover the common paths:

| Command | What it does |
|---------|--------------|
| `make shell` | Install + build the UI, then launch the tray app (static build, no hot reload) |
| `make shell-dev` | Same app in dev mode: Vite hot-reload on `:5173` + file watcher (Rust changes rebuild + relaunch) |
| `make run-cli` | Terminal chat (`uv run nova`) |
| `make test` | Python test suite (`uv run pytest`) |
| `make shell-test` | Rust shell test suite |

The desktop shell spawns the Python brain sidecar over stdio JSON-RPC and supervises it, so there is no second command to run. Ship a build with:

```bash
cd shell/src-tauri && npx --yes @tauri-apps/cli@2 build
# → target/release/bundle/dmg/Ruoxi_0.1.0_aarch64.dmg
```

Shell details (dev URLs, the `index.html?view=panel|overlay|setup|settings|timeline` harness, packaging notes) live in [`shell/README.md`](shell/README.md).

### 🔐 Permissions — and why each one exists

| Permission | Why Ruòxī asks | How it is used |
|---|---|---|
| **Screen Recording** | So it can see the part of the screen you point at — and only that part. | Without it macOS hands the app a wallpaper-only image. |
| **Microphone** | So it can hear the question while you hold the key. | Push-to-talk only; the audio is deleted about a minute after you release. |
| **Accessibility** | So the global hotkeys work while you are inside another app. | Hotkey registration only — never clicks or typing. |

### 🧩 Models — on this Mac, or on a cloud you choose

| Piece | Runs | Choices |
|---|---|---|
| **Speech to text** | On-device (GGML) | Whisper Base / Small, or Parakeet TDT 0.6B — downloaded once, with size shown before you start and a Cancel while it runs. |
| **Text to speech** | On-device, or macOS | Kokoro voices (on-device) or the built-in system voice. |
| **Language model** | Your endpoint | Any OpenAI-compatible API — DeepSeek by default. The key lives in the macOS Keychain; each question sends the capture and your question, and nothing else. |

With **Offline mode** on, the cloud model is unavailable by design: captures, memory and on-device speech still work, and the Models screen says so.

---

## 🧠 How it is put together

```
   ┌──────────────────────────────────────────────────────────┐
   │  Tauri shell (Rust)                                      │
   │  tray · global hotkeys · capture · overlay · panel       │
   │  windows · capture store (files + index, sha256 dedupe)   │
   └───────────────┬──────────────────────────────────────────┘
                   │  stdio JSON-RPC (session.ask / session.abort,
                   │  streamed back as panel:* events)
   ┌───────────────▼──────────────────────────────────────────┐
   │  Brain sidecar (pure Python)                             │
   │  AgentLoop · tool registry · LiteLLM adapter · memory     │
   └──────────────────────────────────────────────────────────┘
```

The React UI (`shell/ui`) renders every surface — panel, overlay, setup, settings, timeline — against the design tokens in [`docs/DESIGN.md`](docs/DESIGN.md) (`python3 scripts/verify_design_md.py` re-validates them).

| Path | What lives there |
|---|---|
| `shell/` | The desktop app — `src-tauri/` (Rust: tray, hotkeys, capture, windows, models, TTS/STT) and `ui/` (React + Tailwind surfaces) |
| `src/` | The Python brain — agent loop, tools, memory, model router, sidecar |
| `docs/` | `DESIGN.md` (the design system), `SYSTEM_FLOW.md`, `rfc/` (RFC-0002 platform shell, RFC-0004 voice, RFC-0007 memory) |
| `docs/demo/` | The README demo video — a HyperFrames composition over real captures |
| `scripts/` | Design-token verification, tray-icon export, helpers |
| `tests/` | Python test suite; Rust tests live beside the shell code |

---

## 🎓 Research: a framework-less agentic loop

Beyond the app, this repository is a working study in building an **agentic AI system without an agent framework** — pure Python and LiteLLM. The desktop shell and the CLI chat both run on the same loop.

### Why framework-less?

Most agent tutorials reach for LangChain, LangGraph or AutoGPT. Those are powerful, but they can hide implementation details behind abstractions, make failures hard to trace, lock you into their architecture, and add complexity for simple cases. This project keeps the loop small enough to read in one sitting:

- **~500 lines of core code** for the agent loop
- **Clear, readable Python** without framework magic
- **Full control** over tool calling, memory and orchestration
- **Easy debugging** — everything is explicit

### Key architecture decisions

| Decision | Why |
|----------|-----|
| **Pure Python Agent Loop** | A simple `while` loop for tool calling — no graph abstractions |
| **LiteLLM as Adapter** | One interface for 100+ LLM providers |
| **Tool Registry Pattern** | Explicit tool registration and schema generation |
| **Two-Layer Memory** | Fast context (MEMORY.md) + searchable history (HISTORY.md) |
| **Session Persistence** | JSON files — no database required |

### The loop

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

### The core, simplified

```python
async def _run_agent_loop(self, messages: List[Dict], tools: List[Dict]):
    iteration = 0

    while iteration < self.max_iterations:
        iteration += 1

        tool_definitions = self.tool_registry.get_definitions()
        response = await self.llm_client.chat_completion(
            messages=messages,
            tools=tool_definitions,
        )

        tool_calls = response.get("tool_calls")

        if tool_calls:
            for tc in tool_calls:
                result = await self.tool_registry.execute(
                    tc.function.name,
                    tc.function.arguments,
                )
                messages.append({
                    "role": "tool",
                    "tool_call_id": tc.id,
                    "content": result,
                })
        else:
            return response.get("response")

    return "Max iterations reached"
```

The tool registry is a dict of `Tool` objects that each publish a schema (`tool.to_schema()`), and the LLM adapter wraps `litellm.acompletion` — the whole abstraction is two small files.

### What you'll learn here

1. **Tool-calling protocol** — schemas from Pydantic models, how the model picks a tool, how results are fed back.
2. **Loop mechanics** — iterative tool execution, message-history management, context building.
3. **LLM abstraction** — why a Port/Adapter split pays off, and how provider quirks are absorbed in one place.
4. **Memory systems** — MEMORY.md for facts, HISTORY.md for events, consolidation between them, and a citation guard so answers point back at what they used.
5. **Session management** — JSON persistence, session isolation, history windowing.

### Framework vs. pure Python

| Aspect | LangChain / LangGraph | Pure Python (this project) |
|--------|---------------------|----------------------------|
| **Learning curve** | Steep | Gentle |
| **Lines of code** | 10-50 | 100-200 |
| **Debugging** | Abstractions hide issues | Explicit, easy to trace |
| **Flexibility** | Constrained by the framework | Full control |
| **LLM calls** | Hidden in the framework | Explicit `await llm.chat_completion()` |
| **Tool calling** | Managed by the framework | A manual loop you can read |
| **Best for** | Complex workflows, production | Learning, small agents you own |

---

## 🧭 Entry points

| Entry point | Command | File | Use case |
|-------------|---------|------|----------|
| **Desktop tray companion** | `make shell` / `make shell-dev` | `shell/` | The app in this README: tray, hotkeys, capture, panel, setup, settings, timeline — it supervises the brain sidecar |
| **Brain sidecar** | `python -m app.interfaces.sidecar` | `src/app/interfaces/sidecar/` | stdio JSON-RPC brain used by the desktop shell (spawned automatically) |
| **CLI chat** | `nova` / `uv run nova` | `src/app/interfaces/cli/app.py` | Interactive terminal chat |
| **Telegram daemon** | `python -m src.app.main` | `src/app/main.py` | Bus + AgentLoop + Telegram channel |

For the CLI, copy `.env.example` to `.env` and set `LITE_LLM_API_KEY` (plus `BRAVE_API_KEY` for web search). `langgraph.json` is stale (it references a missing `src/studio.py`) and is not used — the live runtime is the pure-Python loop above.

---

## ✅ Status

**Working today**

- Tray app with capture (region · window · whole screen), the region overlay, and a capture store with sha256 dedupe
- The voice-ask panel flow end to end — `Ready → Listening → Transcribing → Thinking → Writing → Complete` — with the capture as context and a citation on the answer
- The panel floats above other apps without stealing focus; `Esc` cancels an ask or dismisses the panel
- On-device speech-to-text (Whisper / Parakeet) and text-to-speech (Kokoro, or the system voice), with managed downloads
- Cloud language model over any OpenAI-compatible endpoint, key in the Keychain, blocked by Offline mode
- The six-step setup ritual, Settings, and the Timeline
- Local memory with a citation guard, and memory-grounded answers when offline

**Known gaps**

- The panel's **Save to memory** button is not wired yet — it still shows its M2 placeholder; the memory store and its commands already exist
- The `.dmg` does not bundle a Python runtime: the brain needs a Python with the repo dependencies (set `sidecar_command` to your venv)
- Builds are unsigned — expect a Gatekeeper warning, and a Keychain prompt the first time a freshly built binary starts
- Parakeet is English-only

---

## 🔗 Documentation

- [Design system](docs/DESIGN.md) — tokens, type, motion; `python3 scripts/verify_design_md.py` re-validates
- [System flow](docs/SYSTEM_FLOW.md) — how capture, panel, brain and memory talk to each other
- [Shell README](shell/README.md) — dev harness, packaging, permissions
- [RFCs](docs/rfc) — RFC-0002 platform shell · RFC-0004 voice · RFC-0007 memory
- [Demo video project](docs/demo) — the HyperFrames composition behind the clip above
