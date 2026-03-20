"""
Pydantic models for validated tool results.

NOTE: These models are designed for future use when tools return structured data.
Currently, tools return human-readable strings, so result validation is disabled by default.

To use these models:
1. Update tools to return structured dict/Pydantic model data
2. Enable validation in ToolRegistry: ToolRegistry(enable_validation=True)
3. Tool results will be validated against these models

Example future implementation:
    async def execute(self, command: str) -> str:
        # Execute command...
        return json.dumps({
            "command": command,
            "exit_code": process.returncode,
            "stdout": stdout_str,
            "stderr": stderr_str,
            "success": process.returncode == 0
        })
"""

from pydantic import BaseModel, Field
from typing import Optional, List


class WebSearchResult(BaseModel):
    """Structured web search result."""

    title: str = Field(description="Result title")
    url: str = Field(description="Result URL")
    description: str = Field(description="Result description/snippet")
    rank: int = Field(description="Result rank/position")


class WebSearchResults(BaseModel):
    """Multiple web search results."""

    results: List[WebSearchResult] = Field(description="List of search results")
    total_results: int = Field(description="Total number of results found")
    query: str = Field(description="Original search query")


class WebFetchResult(BaseModel):
    """Result from fetching web content."""

    url: str = Field(description="URL that was fetched")
    title: Optional[str] = Field(default=None, description="Page title")
    content: str = Field(description="Extracted content")
    content_length: int = Field(description="Length of extracted content in characters")
    success: bool = Field(description="Whether the fetch was successful")


class FileReadResult(BaseModel):
    """Result from reading a file."""

    path: str = Field(description="File path that was read")
    content: str = Field(description="File content")
    size_bytes: int = Field(description="File size in bytes")
    encoding: str = Field(default="utf-8", description="Text encoding")
    success: bool = Field(description="Whether the read was successful")


class FileWriteResult(BaseModel):
    """Result from writing a file."""

    path: str = Field(description="File path that was written")
    bytes_written: int = Field(description="Number of bytes written")
    success: bool = Field(description="Whether the write was successful")
    encoding: str = Field(default="utf-8", description="Text encoding used")


class FileEditResult(BaseModel):
    """Result from editing a file."""

    path: str = Field(description="File path that was edited")
    old_text_length: int = Field(description="Length of replaced text")
    new_text_length: int = Field(description="Length of new text")
    replacements_made: int = Field(default=1, description="Number of replacements made")
    success: bool = Field(description="Whether the edit was successful")


class DirectoryListResult(BaseModel):
    """Result from listing directory contents."""

    path: str = Field(description="Directory path that was listed")
    entries: List[str] = Field(description="List of file/directory names")
    total_count: int = Field(description="Total number of entries")
    success: bool = Field(description="Whether the listing was successful")


class ExecResult(BaseModel):
    """Result from executing shell command."""

    command: str = Field(description="Command that was executed")
    exit_code: int = Field(description="Process exit code")
    stdout: Optional[str] = Field(default=None, description="Standard output")
    stderr: Optional[str] = Field(default=None, description="Standard error output")
    success: bool = Field(
        description="Whether command executed successfully (exit code 0)"
    )
    working_dir: Optional[str] = Field(default=None, description="Working directory")
    timeout_occurred: bool = Field(
        default=False, description="Whether command timed out"
    )
