# Unused Dependencies Removal Plan

## Executive Summary

After analyzing the codebase, I've identified **10 dependencies** that are declared in [`pyproject.toml`](../pyproject.toml) but are not actively used in the current codebase. This plan details the removal of these unused dependencies to reduce bloat and potential security vulnerabilities.

## Analysis Methodology

1. ✅ Reviewed all dependencies in [`pyproject.toml`](../pyproject.toml)
2. ✅ Searched for imports across the entire `src/` directory
3. ✅ Checked test files in `tests/` directory
4. ✅ Verified configuration files (`.env.example`, `docker-compose.yaml`)
5. ✅ Cross-referenced with documentation ([`README.md`](../README.md))

## Dependencies to Remove

### 1. Database & Migration (2 packages)

| Package | Version | Current Usage | Reason for Removal |
|---------|---------|---------------|-------------------|
| `sqlalchemy` | >=2.0.0 | ❌ Not used | No database ORM implementation found |
| `alembic` | >=1.12.0 | ❌ Not used | No database migrations found |

**Impact:** None - these packages are not imported anywhere in the codebase.

### 2. Security & Authentication (2 packages)

| Package | Version | Current Usage | Reason for Removal |
|---------|---------|---------------|-------------------|
| `python-jose[cryptography]` | >=3.3.0 | ❌ Not used | No JWT/token authentication implementation |
| `python-multipart` | >=0.0.6 | ❌ Not used | No file upload endpoints found |

**Impact:** None - no authentication or file upload features implemented.

### 3. LangChain Ecosystem (2 packages)

| Package | Version | Current Usage | Reason for Removal |
|---------|---------|---------------|-------------------|
| `langchain-community` | >=0.0.20 | ❌ Not used | No community integrations imported |
| `langchain-ollama` | >=1.0.0 | ❌ Not used | Ollama not used (LiteLLM handles Ollama) |

**Impact:** None - LiteLLM adapter handles all LLM provider integrations including Ollama.

### 4. AI/ML Integrations (3 packages)

| Package | Version | Current Usage | Reason for Removal |
|---------|---------|---------------|-------------------|
| `opik` | >=1.9.3 | ⚠️ Only in legacy/ | Used only in deprecated legacy code |
| `letta-client` | >=0.1.324 | ❌ Not used | No Letta integration found |
| `langchain-pinecone` | >=0.2.13 | ❌ Not used | No Pinecone vector DB usage |

**Impact:** 
- `opik` is only used in [`legacy/src/app/application/services/langgraph_orchestrator.py`](../legacy/src/app/application/services/langgraph_orchestrator.py:24)
- Other packages have no usage

### 5. LangGraph Checkpoint (1 package)

| Package | Version | Current Usage | Reason for Removal |
|---------|---------|---------------|-------------------|
| `langgraph-checkpoint-postgres` | >=3.0.1 | ❌ Not used | Postgres checkpointing not implemented |

**Impact:** None - the code uses `InMemorySaver` instead of postgres checkpointing.

## Dependencies to Keep

### Kept for Active Use

The following dependencies are actively used and should **NOT** be removed:

- ✅ `fastapi`, `uvicorn` - Web framework and server
- ✅ `pydantic`, `pydantic-settings` - Data validation and settings
- ✅ `langchain`, `langchain-core`, `langgraph` - Core AI framework
- ✅ `litellm` - Multi-provider LLM adapter
- ✅ `json-repair` - Memory consolidation
- ✅ `readability-lxml`, `beautifulsoup4` - Web scraping tools
- ✅ `pyyaml`, `python-frontmatter` - Skills system
- ✅ `typer`, `rich` - CLI interface
- ✅ `python-telegram-bot` - Telegram integration
- ✅ `sse-starlette` - Server-Sent Events for API
- ✅ `python-dotenv` - Environment configuration
- ✅ `langgraph-cli` - LangGraph CLI tool
- ✅ `langmem` - Memory management tools
- ✅ `pytest`, `pytest-asyncio` - Testing framework
- ✅ `ruff` - Code linting

### Kept for Future Use

The following dependencies are not currently used but are kept for planned features:

- 🔄 `redis` - Referenced in [`docker-compose.yaml`](../docker-compose.yaml:17) for future caching
- 🔄 `psycopg` - Used in [`dependencies.py`](../src/app/dependencies.py:11) for postgres store (feature in development)

## Implementation Plan

### Step 1: Update pyproject.toml

Remove the following lines from [`pyproject.toml`](../pyproject.toml):

```toml
# Line 19: Remove
"sqlalchemy>=2.0.0",

# Line 20: Remove
"alembic>=1.12.0",

# Line 23: Remove
"python-jose[cryptography]>=3.3.0",

# Line 24: Remove
"python-multipart>=0.0.6",

# Line 27: Remove
"langchain-community>=0.0.20",

# Line 52: Remove
"langchain-ollama>=1.0.0",

# Line 54: Remove
"opik>=1.9.3",

# Line 55: Remove
"letta-client>=0.1.324",

# Line 56: Remove
"langchain-pinecone>=0.2.13",

# Line 57: Remove
"langgraph-checkpoint-postgres>=3.0.1",
```

### Step 2: Update Dependency Lock

Run the following command to update the lock file:

```bash
uv sync
```

This will remove the unused packages and their transitive dependencies from [`uv.lock`](../uv.lock).

### Step 3: Verify Application

After removal, verify the application still works:

```bash
# Run tests
uv run pytest

# Start the CLI
uv run nova --help

# Start the API server
uv run nova-api
```

### Step 4: Clean Up Legacy Code (Optional)

Since `opik` is only used in legacy code, consider removing the import from:

[`legacy/src/app/application/services/langgraph_orchestrator.py`](../legacy/src/app/application/services/langgraph_orchestrator.py:24)

```python
# Line 24: Remove this import
from opik.integrations.langchain import OpikTracer
```

## Risk Assessment

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Breaking changes | Low | Packages are not imported anywhere |
| Transitive dependency issues | Low | `uv sync` will handle resolution |
| Feature regression | None | No active features use these packages |
| Legacy code impact | Low | Only `opik` in legacy folder |

## Expected Benefits

1. **Reduced attack surface** - Fewer dependencies = fewer potential vulnerabilities
2. **Smaller package size** - Reduced disk space and installation time
3. **Cleaner dependency tree** - Easier to understand and maintain
4. **Faster builds** - Fewer packages to download and install

## Post-Removal Validation Checklist

- [ ] Run `uv sync` successfully
- [ ] All tests pass: `uv run pytest`
- [ ] CLI works: `uv run nova --help`
- [ ] API starts: `uv run nova-api`
- [ ] No import errors in application logs
- [ ] Verify [`uv.lock`](../uv.lock) no longer contains removed packages

## Rollback Plan

If issues arise after removal:

1. Restore the original [`pyproject.toml`](../pyproject.toml)
2. Run `uv sync` to restore dependencies
3. Investigate any errors that occurred
4. Document any dependencies that were actually needed

## Summary

This plan removes **10 unused dependencies** totaling approximately **0 lines of actual code usage**, reducing the dependency bloat without impacting any active functionality. The removal is low-risk and will improve the maintainability of the project.
