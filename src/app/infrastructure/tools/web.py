"""Web tools for searching and fetching content."""

import json
from typing import Any, Optional
from urllib.parse import urlparse

import aiohttp
from readability import Document
from bs4 import BeautifulSoup

from app.infrastructure.tools.base import Tool


class WebSearchTool(Tool):
    """Tool to search the web using Brave Search API."""

    BRAVE_SEARCH_URL = "https://api.search.brave.com/res/v1/web/search"

    def __init__(self, api_key: Optional[str] = None):
        """Initialize web search tool.

        Args:
            api_key: Brave Search API key
        """
        self.api_key = api_key

    @property
    def name(self) -> str:
        return "web_search"

    @property
    def description(self) -> str:
        if not self.api_key:
            return "Web search (Brave API key not configured)"
        return "Search the web using Brave Search API. Returns search results with titles, URLs, and snippets."

    @property
    def parameters(self) -> dict[str, Any]:
        return {
            "type": "object",
            "properties": {
                "query": {"type": "string", "description": "Search query"},
                "count": {
                    "type": "integer",
                    "description": "Number of results to return (default: 5, max: 20)",
                    "default": 5,
                    "minimum": 1,
                    "maximum": 20,
                },
            },
            "required": ["query"],
        }

    async def execute(self, query: str, count: int = 5) -> str:
        """Execute web search.

        Args:
            query: Search query
            count: Number of results

        Returns:
            Search results or error
        """
        if not self.api_key:
            return "Error: Brave Search API key not configured. Set BRAVE_API_KEY environment variable."

        try:
            headers = {
                "Accept": "application/json",
                "X-Subscription-Token": self.api_key,
            }

            params = {
                "q": query,
                "count": min(count, 20),
            }

            async with aiohttp.ClientSession() as session:
                async with session.get(
                    self.BRAVE_SEARCH_URL,
                    headers=headers,
                    params=params,
                    timeout=aiohttp.ClientTimeout(total=10),
                ) as response:
                    if response.status != 200:
                        error_text = await response.text()
                        return f"Error: Search API returned {response.status}: {error_text}"

                    data = await response.json()

                    results = data.get("web", {}).get("results", [])

                    if not results:
                        return f"No results found for: {query}"

                    formatted_results = []
                    for i, result in enumerate(results[:count], 1):
                        title = result.get("title", "No title")
                        url = result.get("url", "")
                        description = result.get("description", "No description")

                        formatted_results.append(
                            f"{i}. {title}\n   URL: {url}\n   {description}\n"
                        )

                    return "\n".join(formatted_results)

        except Exception as e:
            return f"Error searching web: {str(e)}"


class WebFetchTool(Tool):
    """Tool to fetch and extract content from web pages."""

    def __init__(self):
        """Initialize web fetch tool."""
        pass

    @property
    def name(self) -> str:
        return "web_fetch"

    @property
    def description(self) -> str:
        return "Fetch and extract readable content from a URL. Useful for reading articles, documentation, etc."

    @property
    def parameters(self) -> dict[str, Any]:
        return {
            "type": "object",
            "properties": {"url": {"type": "string", "description": "URL to fetch"}},
            "required": ["url"],
        }

    def _is_valid_url(self, url: str) -> bool:
        """Check if URL is valid."""
        try:
            parsed = urlparse(url)
            return bool(parsed.netloc) and parsed.scheme in ("http", "https")
        except:
            return False

    async def execute(self, url: str) -> str:
        """Fetch and extract content from URL.

        Args:
            url: URL to fetch

        Returns:
            Extracted content or error
        """
        if not self._is_valid_url(url):
            return f"Error: Invalid URL: {url}"

        try:
            headers = {
                "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36"
            }

            async with aiohttp.ClientSession() as session:
                async with session.get(
                    url,
                    headers=headers,
                    timeout=aiohttp.ClientTimeout(total=30),
                    allow_redirects=True,
                ) as response:
                    if response.status != 200:
                        return f"Error: HTTP {response.status} when fetching {url}"

                    html = await response.text()

                    # Use readability to extract main content
                    doc = Document(html)
                    title = doc.title() or "No title"
                    content_html = doc.summary()

                    # Clean up HTML with BeautifulSoup
                    soup = BeautifulSoup(content_html, "html.parser")

                    # Remove script and style elements
                    for script in soup(["script", "style"]):
                        script.decompose()

                    # Get text
                    text = soup.get_text()

                    # Clean up whitespace
                    lines = (line.strip() for line in text.splitlines())
                    chunks = (
                        phrase.strip() for line in lines for phrase in line.split("  ")
                    )
                    text = "\n".join(chunk for chunk in chunks if chunk)

                    # Truncate if too long
                    max_length = 10000
                    if len(text) > max_length:
                        text = (
                            text[:max_length]
                            + f"\n\n[Content truncated. Total length: {len(text)} chars]"
                        )

                    return f"Title: {title}\n\nURL: {url}\n\n{text}"

        except aiohttp.ClientError as e:
            return f"Error fetching URL: Network error - {str(e)}"
        except Exception as e:
            return f"Error fetching URL: {str(e)}"
