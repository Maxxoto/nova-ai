"""FastAPI endpoints for Nova Agent API."""

import json

from fastapi import APIRouter, Request, Depends
from sse_starlette.sse import EventSourceResponse

from core.entities.chat_message import ChatCompletionRequest
from core.services.chat_service import ChatService


# Create routers
chat_router = APIRouter(prefix="/sse", tags=["chat"])


# Dependency for ChatService
async def get_chat_service() -> ChatService:
    """Get chat service instance to avoid circular imports."""
    from di import get_chat_service as _get_chat_service

    return await _get_chat_service()


@chat_router.post("/chat-completion")
async def chat_completion_stream(
    request: ChatCompletionRequest,
    request_obj: Request,
    chat_service: ChatService = Depends(get_chat_service),
):
    """Stream chat completion using hexagonal architecture via Server-Sent Events."""

    async def event_generator():
        """Generate SSE events from chat service streaming response."""
        thread_id = None
        try:
            # Convert messages to dict format
            messages_dict = [
                {"role": msg.role, "content": msg.content} for msg in request.messages
            ]

            # Stream the response using chat service
            async for stream_dict in chat_service.stream_chat_completion(
                messages=messages_dict, thread_id=request.thread_id
            ):
                if await request_obj.is_disconnected():
                    break

                # Store thread_id for completion/error events
                thread_id = stream_dict["thread_id"]

                # Send each chunk as an SSE event with thread_id
                yield {
                    "event": "message",
                    "data": json.dumps(
                        {
                            "content": stream_dict["content"],
                            "type": "chunk",
                            "thread_id": thread_id,
                        }
                    ),
                }

            # Send completion event with thread_id
            yield {
                "event": "complete",
                "data": json.dumps(
                    {
                        "type": "complete",
                        "message": "Stream completed and message history updated",
                        "thread_id": thread_id,
                    }
                ),
            }

        except Exception as e:
            # Send error event with thread_id
            yield {
                "event": "error",
                "data": json.dumps(
                    {
                        "type": "error",
                        "message": f"Error during streaming: {str(e)}",
                        "thread_id": thread_id,
                    }
                ),
            }

    return EventSourceResponse(event_generator())
