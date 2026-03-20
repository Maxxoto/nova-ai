"""FastAPI endpoints for Nova Agent API."""

import json

from fastapi import APIRouter, Request, Depends
from sse_starlette.sse import EventSourceResponse

from app.domain.entities.chat_message import ChatCompletionRequest
from app.application.services.enhanced_orchestrator import EnhancedLangGraphOrchestrator
from app.dependencies import get_orchestrator


# Create routers
chat_router = APIRouter(prefix="/sse", tags=["chat"])


@chat_router.post("/chat-completion")
async def chat_completion_stream(
    request: ChatCompletionRequest,
    request_obj: Request,
    orchestrator: EnhancedLangGraphOrchestrator = Depends(get_orchestrator),
):
    """Stream chat completion using EnhancedOrchestrator via Server-Sent Events."""

    async def event_generator():
        """Generate SSE events from orchestrator streaming response."""
        thread_id = None
        try:
            # Convert messages to dict format
            messages_dict = [
                {"role": msg.role, "content": msg.content} for msg in request.messages
            ]

            # Stream the response using orchestrator
            async for stream_dict in orchestrator.stream_chat_completion(
                messages=messages_dict, thread_id=request.thread_id
            ):
                if await request_obj.is_disconnected():
                    break

                # Store thread_id for completion/error events
                thread_id = stream_dict.get("thread_id", request.thread_id)

                # Send each chunk as an SSE event with thread_id
                yield {
                    "event": "message",
                    "data": json.dumps(
                        {
                            "content": stream_dict.get("content", ""),
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
