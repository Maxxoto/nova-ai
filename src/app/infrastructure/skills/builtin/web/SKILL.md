---
name: web
description: Search and fetch content from the web
always_load: false
requirements:
  bins: []
  env:
    - BRAVE_API_KEY
---

# Web Skill

Use web search and fetch to get information from the internet.

## Tools Available

### web_search
Search the web using Brave Search API.

**When to use:**
- Current events and news
- Finding documentation
- Researching topics
- Fact-checking

**Parameters:**
- `query`: Search query (required)
- `count`: Number of results (1-20, default: 5)

### web_fetch
Fetch and extract readable content from a URL.

**When to use:**
- Reading articles or blog posts
- Accessing documentation pages
- Extracting content from web pages

**Parameters:**
- `url`: URL to fetch (required)

## Usage Examples

**User**: "What happened in AI this week?"
**You**: I'll search for recent AI news. (Use web_search with "AI news this week")

**User**: "Summarize this article: https://example.com/article"
**You**: I'll fetch that article for you. (Use web_fetch with the URL)

**User**: "How do I use React hooks?"
**You**: Let me search for React hooks documentation. (Use web_search)

## Best Practices

1. **Always verify**: Web content can be outdated or incorrect
2. **Be specific**: Better search results with specific queries
3. **Summarize**: Don't just return raw search results
4. **Attribute**: Mention sources when providing information
5. **Check dates**: Especially for technical documentation
