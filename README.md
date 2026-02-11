# Nova Agent API

A powerful agent-based API service built with FastAPI and a simplified Hexagonal Architecture.

## Architecture Structure

This project follows a flattened architecture for better clarity while maintaining separation of concerns:

- **`src/api/`**: Entry points for HTTP interfaces (FastAPI).
- **`src/cli/`**: Entry points for Command Line interfaces.
- **`src/core/`**: Core business logic, including entities, ports, and services (orchestrators, nodes, parsers, chat service).
- **`src/adapters/`**: Implementations of ports for LLMs and Memory providers.
- **`src/config.py`**: Centralized application configuration.
- **`src/di.py`**: Dependency Injection container.

## Quick Start with uv

### Installation

1. **Clone and install dependencies:**
   ```bash
   git clone <repository-url>
   cd nova-api
   uv sync
   ```

2. **Activate virtual environment:**
   ```bash
   source .venv/bin/activate  # Unix/macOS
   # or
   .venv\Scripts\activate     # Windows
   ```

### Running the Application

**Start the API:**
```bash
uv run uvicorn api.fastapi_app:app --reload --host 0.0.0.0 --port 8000
```

**Run the CLI interface:**
```bash
uv run python src/cli/chat_interface.py
```

### Development Commands

```bash
# Run tests
uv run pytest

# Format code
uv run black src/ tests/

# Lint code
uv run flake8 src/ tests/
```

## License

MIT License
