# Linear CLI Guide for Claude

This guide is written by Claude for Claude to effectively use the Linear CLI tool when helping with project management, issue tracking, and documentation.

## Overview

The Linear CLI tool allows me to interact with your company's Linear project management system directly from the terminal. This enables me to:
- View and track issues across all projects
- Monitor project progress
- Create and update issues
- Help with documentation and task management

## Essential Commands

### 1. Viewing Issues

**See all current issues (grouped by status):**
```bash
linear issues
```

**Filter issues by status:**
```bash
linear issues --todo        # Backlog/unstarted issues
linear issues --progress    # Issues currently being worked on
linear issues --done        # Completed issues
```

**View your assigned issues:**
```bash
linear issues --mine
```

**Search for specific issues:**
```bash
linear issues --search "bug"
linear issues --search "feature"
```

**View a specific issue with full details:**
```bash
linear issue INF-36  # Replace with actual issue ID
```

**Advanced filtering with query language:**
```bash
linear issues -f "assignee:john@example.com AND priority:>2"
linear issues -f "title:~bug AND created:>1week"
linear issues -f "has-label:urgent AND no-assignee"
```

### 2. Creating Issues

**Basic issue creation:**
```bash
linear create issue "Bug: Login timeout" "Users experiencing timeouts after 5 minutes"
```

**Create issue with priority and team:**
```bash
linear create issue "Implement caching" "Add Redis caching layer" --team ENG --priority high
```

Priority levels: `none`, `low`, `medium`, `high`, `urgent`

### 3. Updating Issues

**Update issue details:**
```bash
linear update issue INF-36 --title "Updated title"
linear update issue INF-36 --description "New description"
linear update issue INF-36 --priority urgent
```

### 4. Project Management

**View all projects:**
```bash
linear projects
```

**Create a new project:**
```bash
linear create project "Q1 2024 Features" "New features for Q1"
```

### 5. Team Information

**List all teams:**
```bash
linear teams
```

### 6. Comments Management

**View comments on an issue:**
```bash
linear comment list INF-36
```

**Add a comment:**
```bash
linear comment add INF-36 "Implementation complete, ready for review"
```

**Update a comment:**
```bash
linear comment update comment_id "Updated: Implementation complete with tests"
```

### 7. Bulk Operations

**Update multiple issues:**
```bash
linear bulk update INF-1,INF-2,INF-3 --state done
linear bulk update INF-4 INF-5 --priority high --assignee user_id
```

**Archive multiple issues:**
```bash
linear bulk archive INF-10,INF-11,INF-12
```

### 8. Saved Searches

**Save a frequently used search:**
```bash
linear search save my-urgent "assignee:me AND priority:urgent"
linear search save bugs-this-week "title:~bug AND created:<1week"
```

**Run a saved search:**
```bash
linear search run my-urgent
linear search list  # See all saved searches
```

## Workflow Examples for Claude

### When starting a work session:
1. Check current issues to understand priorities:
   ```bash
   linear issues --progress --limit 10
   ```

2. Look for urgent/high priority items:
   ```bash
   linear issues --todo --search "urgent"
   ```

### When helping with a specific task:
1. Find the relevant issue:
   ```bash
   linear issues --search "component name"
   ```

2. View full details:
   ```bash
   linear issue INF-XX
   ```

3. Update progress as needed:
   ```bash
   linear update issue INF-XX --description "Added implementation notes..."
   ```

### For documentation tasks:
1. Create documentation issues:
   ```bash
   linear create issue "Document API endpoints" "Need to document all REST API endpoints" --priority medium --team ENG
   ```

2. Track documentation progress:
   ```bash
   linear issues --search "document" --mine
   ```

## Output Formats

**Default (simple) format:**
```bash
linear issues  # Clean, readable output with colors
```

**Table format:**
```bash
linear issues --format table  # Structured table view
```

**JSON format (for parsing):**
```bash
linear issues --format json  # Machine-readable output
```

## Tips for Effective Use

1. **Status Grouping**: Issues are automatically grouped by status (Todo, In Progress, Done) for better organization.

2. **Color Coding**:
   - Blue: Issue identifiers
   - Green: Assignee names and completed states
   - Yellow: Medium priority and in-progress states
   - Red: High/urgent priority
   - Cyan: Labels

3. **First Sentence Summary**: Issue descriptions show only the first sentence for quick scanning.

4. **Combining Filters**: Multiple filters can be combined:
   ```bash
   linear issues --mine --progress --team ENG
   ```

5. **Batch Operations**: When working on multiple related issues, use search to find them all:
   ```bash
   linear issues --search "refactor" --limit 20
   ```

## Common Patterns

### Daily Status Check
```bash
# What's being worked on
linear issues --progress

# What's coming up
linear issues --todo --limit 5

# Recent completions
linear issues --done --limit 5
```

### Project Overview
```bash
# See all projects and their progress
linear projects

# Find issues for a specific project
linear issues --search "project name"
```

### Issue Investigation
```bash
# Find an issue
linear issues --search "error message"

# Get full details
linear issue INF-123

# Check related issues
linear issues --search "similar keyword"
```

## New v2 Features Summary

### Advanced Search
- Use `-f` flag with query language for complex filters
- Operators: `:` (equals), `:>` (greater), `:<` (less), `:~` (contains), `:!=` (not equals)
- Fields: assignee, state, priority, title, description, created, updated, label
- Combine with AND: `"field1:value1 AND field2:value2"`
- Relative dates: `"created:>1week"`, `"updated:<2days"`

### Comments
- Full comment thread management on issues
- Markdown support in comments
- View comment history with timestamps

### Bulk Operations
- Process multiple issues in one command
- Supports comma-separated IDs or multiple arguments
- Progress tracking for each operation

### Saved Searches
- Store frequently used complex queries
- Quick access to common filters
- Share search patterns with team

## Notes for Claude

- Always check issue status before making updates
- Use descriptive titles when creating issues
- Include relevant context in descriptions
- Tag issues with appropriate priority levels
- Remember that all changes are tracked in Linear's audit log
- The tool uses Cole's Linear API key for authentication
- Use advanced search for complex queries instead of multiple simple filters
- Add comments to issues for async communication
- Use bulk operations when dealing with multiple related issues

This tool enables me to be more helpful with project management, staying informed about ongoing work, and creating proper documentation for tasks and issues.