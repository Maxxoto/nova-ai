---
name: todo
description: Manage tasks and to-do lists
always_load: true
requirements:
  bins: []
  env: []
---

# Todo Skill

Manage personal tasks and to-do lists.

## Storage

Tasks are stored in: `workspace/todos.json`

## Task Structure

Each task has:
- `id`: Unique identifier
- `title`: Task description
- `status`: todo, in_progress, done
- `priority`: low, medium, high
- `due_date`: Optional due date (ISO format)
- `tags`: List of tags
- `created_at`: Creation timestamp
- `completed_at`: Completion timestamp

## Available Operations

1. **List tasks**: Show all tasks or filter by status/priority
2. **Add task**: Create new task with title, priority, due date
3. **Update task**: Change status, priority, or details
4. **Complete task**: Mark task as done
5. **Delete task**: Remove task from list
6. **Search tasks**: Find tasks by keyword or tag

## Usage Examples

**User**: "What do I need to do today?"
**You**: I'll check your todo list. (Read workspace/todos.json)

**User**: "Add a task to review the PR by Friday"
**You**: I'll add that task with high priority. (Add task with due_date)

**User**: "Mark the documentation task as done"
**You**: I'll update that task status. (Update task status to done)

**User**: "Show me all high priority tasks"
**You**: Here are your high priority tasks. (Filter by priority: high)

## File Format

Tasks are stored as JSON:
```json
{
  "tasks": [
    {
      "id": "uuid",
      "title": "Task description",
      "status": "todo",
      "priority": "medium",
      "due_date": "2024-01-15",
      "tags": ["work", "urgent"],
      "created_at": "2024-01-01T10:00:00",
      "completed_at": null
    }
  ]
}
```

## Best Practices

1. **Always confirm**: Ask before deleting tasks
2. **Set priorities**: Default to medium, use high sparingly
3. **Be specific**: Clear task titles help with completion
4. **Use tags**: Help categorize tasks (work, personal, study)
5. **Due dates**: Only set when there's a real deadline
