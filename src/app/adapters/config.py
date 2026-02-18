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

    # Groq Settings
    groq_api_key: str = os.getenv("GROQ_API_KEY", "your-groq-api-key-here")
    groq_model: str = os.getenv("GROQ_MODEL", "openai/gpt-oss-20b")

    class Config:
        case_sensitive = False


# Global settings instance
settings = Settings()
