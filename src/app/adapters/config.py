"""Configuration settings for Nova Agent API."""

import os
from dotenv import load_dotenv
from pydantic_settings import BaseSettings

# Load environment variables from .env file
load_dotenv()


class Settings(BaseSettings):
    """Application settings."""

    # API Settings
    api_title: str = "Nova Agent API"
    api_version: str = "0.1.0"
    api_description: str = "A powerful agent-based API service"

    # Server Settings
    host: str = "0.0.0.0"
    port: int = 8000
    reload: bool = True

    # LiteLLM Settings (Multi-Provider)
    # Format: "provider/model-name" (e.g., "groq/openai/gpt-oss-20b")
    lite_llm_model: str = os.getenv("LITE_LLM_MODEL", "groq/openai/gpt-oss-20b")
    lite_llm_api_key: str = os.getenv(
        "LITE_LLM_API_KEY", ""
    )  # Universal API key for all providers
    litellm_temperature: float = float(os.getenv("LITELLM_TEMPERATURE", "0.7"))
    litellm_max_tokens: int = int(os.getenv("LITELLM_MAX_TOKENS", "4096"))

    # Workspace Settings
    workspace_dir: str = os.getenv("NOVA_WORKSPACE", os.path.expanduser("~/.nova"))

    class Config:
        case_sensitive = False


# Global settings instance
settings = Settings()
