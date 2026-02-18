---
name: notes
description: Take and manage notes
always_load: true
requirements:
  bins: []
  env: []
---

# Notes Skill

Take, organize, and retrieve notes.

## Storage

Notes are stored in: `workspace/notes/`

Each note is a markdown file.

## Note Organization

Notes can be organized by:
- **Folder**: Subject/topic folders
- **Tags**: Hashtags in content (#work, #study, #idea)
- **Date**: Creation/modification dates

## Available Operations

1. **Create note**: Add new note with title and content
2. **Read note**: Retrieve note by name or search
3. **Update note**: Modify existing note
4. **Delete note**: Remove note
5. **List notes**: Show all notes or by folder/tag
6. **Search notes**: Find notes by keyword

## Usage Examples

**User**: "Take a note about the API design"
**You**: I'll create a note. (Create note in workspace/notes/)

**User**: "What notes do I have about Python?"
**You**: Let me search your notes. (Search for "Python")

**User**: "Show me my work notes"
**You**: Here are your work notes. (List notes in work/ folder)

**User**: "Add to my project ideas note"
**You**: I'll update that note. (Read, modify, save)

## File Structure

```
workspace/notes/
├── work/
│   ├── meeting-2024-01-15.md
│   └── project-ideas.md
├── study/
│   ├── python-tips.md
│   └── algorithms.md
└── personal/
    └── goals.md
```

## Note Format

```markdown
# Note Title

Created: 2024-01-15
Tags: #work #meeting

## Content

Your notes here...

## Action Items

- [ ] Task 1
- [ ] Task 2
```

## Best Practices

1. **Use clear titles**: Descriptive file names
2. **Add tags**: Makes searching easier
3. **Organize by folder**: Separate work, study, personal
4. **Include metadata**: Creation date, tags
5. **Use checkboxes**: For action items or tasks
6. **Link related notes**: Create connections between ideas
