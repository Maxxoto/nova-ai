---
name: memory
description: Manage and interact with the memory system
always_load: true
requirements:
  bins: []
  env: []
---

# Memory Skill

You have access to a two-layer memory system:

## Long-term Memory (MEMORY.md)
- Contains user preferences, facts, and important context
- **Always loaded** into your context
- Use this to remember:
  - User preferences (favorite tools, communication style)
  - Personal information (name, location, timezone)
  - Project context (what they're working on)
  - Technical decisions and preferences

## History (HISTORY.md)
- Searchable archive of past conversations
- **Not loaded** by default, searched on demand
- Use this when the user asks about previous conversations

## How to Use

1. **Remember facts automatically**: When the user shares important information, acknowledge it and it will be saved to memory.

2. **Search history**: When asked about previous conversations, use the search capability.

3. **Update memory**: When user preferences change, update the relevant section.

## Memory Sections

The long-term memory is organized into sections:
- **User Information**: Name, contact, location
- **Preferences**: Communication style, favorite tools, formats
- **Projects**: Current work, goals, context

## Example Interactions

**User**: "I'm a software engineer working on Python projects"
**You**: "Noted! I'll remember you're a Python software engineer."

**User**: "What's my timezone?"
**You**: "According to your memory, you're in [timezone from MEMORY.md]"

**User**: "What did we discuss last week about deployment?"
**You**: "Let me search your history... [search HISTORY.md for 'deployment']"
