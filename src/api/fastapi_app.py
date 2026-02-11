"""Main FastAPI application module."""

import logging
from contextlib import asynccontextmanager
from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware

from config import settings
from .endpoints import chat_router


@asynccontextmanager
async def lifespan(app: FastAPI):
    """Lifespan context manager for startup and shutdown events."""
    # Startup
    logger = logging.getLogger("uvicorn.access")
    logging.basicConfig(format="{levelname:7} {message}", style="{", level=logging.INFO)
    handler = logging.StreamHandler()
    logger.addHandler(handler)
    yield


def create_app() -> FastAPI:
    """Create and configure the FastAPI application."""

    app = FastAPI(
        title=settings.api_title,
        version=settings.api_version,
        description=settings.api_description,
        lifespan=lifespan,
    )

    # Add CORS middleware
    app.add_middleware(
        CORSMiddleware,
        allow_origins=["*"],  # Configure appropriately for production
        allow_credentials=True,
        allow_methods=["*"],
        allow_headers=["*"],
    )

    # Health check endpoint
    @app.get("/health")
    async def health_check():
        return {"status": "healthy", "version": settings.api_version}

    # Root endpoint
    @app.get("/")
    async def root():
        return {
            "message": "Welcome to Nova Agent API",
            "version": settings.api_version,
            "docs": "/docs",
        }

    # Include routers
    app.include_router(chat_router)

    return app


# Create the application instance
app = create_app()
