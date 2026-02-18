---
name: github
description: Interact with GitHub using the gh CLI
always_load: false
requirements:
  bins:
    - gh
  env:
    - GITHUB_TOKEN
---

# GitHub Skill

Interact with GitHub repositories, issues, and pull requests.

## Prerequisites

- GitHub CLI (`gh`) must be installed
- `GITHUB_TOKEN` environment variable set with appropriate permissions

## Common Operations

### Repository Management
- List repositories: `gh repo list`
- View repository: `gh repo view owner/repo`
- Clone repository: `gh repo clone owner/repo`

### Issues
- List issues: `gh issue list`
- View issue: `gh issue view <number>`
- Create issue: `gh issue create --title "..." --body "..."`
- Comment on issue: `gh issue comment <number> --body "..."`

### Pull Requests
- List PRs: `gh pr list`
- View PR: `gh pr view <number>`
- Checkout PR: `gh pr checkout <number>`
- Create PR: `gh pr create --title "..." --body "..."`

### Code Search
- Search code: `gh search code "query" --repo owner/repo`

## Usage Examples

**User**: "Show me my recent issues"
**You**: I'll list your recent GitHub issues. (Use `gh issue list`)

**User**: "Create a bug report for the login issue"
**You**: I'll help you create a GitHub issue. (Use `gh issue create`)

**User**: "What's the status of PR #42?"
**You**: Let me check that PR. (Use `gh pr view 42`)

## Best Practices

1. **Check permissions**: Ensure GITHUB_TOKEN has required scopes
2. **Be careful**: Creating issues/PRs is permanent
3. **Use templates**: Respect issue/PR templates if they exist
4. **Provide context**: Include relevant details in issues
