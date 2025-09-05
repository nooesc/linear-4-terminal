# Linear CLI Filter System Guide

The Linear CLI includes a powerful filter system that allows you to search and filter issues using a variety of operators and conditions.

## Basic Syntax

Filters follow the pattern: `field:operator:value` or `field operator value`

### Simple Examples

```bash
# Issues with specific status
linear issues --filter "status:completed"

# Issues not completed
linear issues --filter "status!=completed"

# High priority issues
linear issues --filter "priority>2"

# Issues assigned to a specific person
linear issues --filter "assignee=john@example.com"

# Issues with "bug" in the title
linear issues --filter "title~bug"
```

## Supported Fields

- `title` - Issue title
- `description`, `desc` - Issue description  
- `status`, `state` - Issue status/workflow state
- `priority`, `p` - Issue priority (0-4)
- `assignee`, `assigned` - Assigned user
- `label`, `labels`, `tag`, `tags` - Issue labels
- `project` - Associated project
- `team` - Team
- `created`, `created_at` - Creation date
- `updated`, `updated_at` - Last update date
- `due`, `due_date` - Due date
- `id`, `identifier` - Issue identifier

## Operators

### Equality
- `:` or `=` - Equals
- `!=` - Not equals

### Comparison
- `>` - Greater than
- `>=` - Greater than or equals
- `<` - Less than
- `<=` - Less than or equals

### String Matching
- `~` - Contains (case-insensitive)
- `!~` - Does not contain
- `^=` - Starts with
- `$=` - Ends with

### List Operations
- `in:` - Value in list (e.g., `status in:backlog,unstarted,started`)

### Existence
- `:null` or `:empty` - Field is null/empty
- `!=null` or `!=empty` - Field is not null/empty

## Date Filtering

The filter system supports relative date filtering using shorthand notation:

- `7d` - 7 days
- `2w` - 2 weeks  
- `1m` - 1 month (30 days)
- `24h` - 24 hours

### Examples

```bash
# Issues created in the last 7 days
linear issues --filter "created>7d"

# Issues not updated for 2 weeks
linear issues --filter "updated<2w"

# Issues created in last month and updated in last week
linear issues --filter "created>1m AND updated>1w"
```

## Compound Filters

Use `AND` and `OR` to combine multiple conditions:

```bash
# Open high-priority issues
linear issues --filter "status!=completed AND priority>2"

# Issues that are either in backlog or unstarted
linear issues --filter "status:backlog OR status:unstarted"

# Complex filter
linear issues --filter "(priority>2 OR label:urgent) AND status!=completed"
```

## Negation

Use `NOT` to negate conditions:

```bash
# Not completed
linear issues --filter "NOT status:completed"

# Not high priority and not assigned
linear issues --filter "NOT (priority>2 OR assignee:null)"
```

## Special Filters

### Assignee Filters
```bash
# Unassigned issues
linear issues --filter "assignee:null"

# Assigned issues
linear issues --filter "assignee!=null"

# Assigned to specific person
linear issues --filter "assignee=user@example.com"
```

### Label Filters
```bash
# Has specific label
linear issues --filter "label:bug"

# Has any of these labels
linear issues --filter "label in:bug,critical,urgent"

# No labels
linear issues --filter "label:null"
```

### Priority Filters

Priority values:
- 0 or "none" - No priority
- 1 or "low" - Low priority
- 2 or "medium" - Medium priority  
- 3 or "high" - High priority
- 4 or "urgent" - Urgent priority

```bash
# High or urgent priority
linear issues --filter "priority>=3"

# Medium priority
linear issues --filter "priority:medium"
```

## Quoted Values

Use quotes for values containing spaces:

```bash
# Title contains "bug fix"
linear issues --filter 'title~"bug fix"'

# Assigned to user with space in email
linear issues --filter 'assignee="john doe@example.com"'
```

## Saving Searches

You can save frequently used filters:

```bash
# Save a filter
linear search save "high-priority-bugs" "priority>2 AND label:bug AND status!=completed"

# Run saved search
linear search run high-priority-bugs

# List saved searches
linear search list

# Delete saved search
linear search delete high-priority-bugs
```

## Examples

### Common Use Cases

```bash
# My open issues
linear issues --filter "assignee=me@example.com AND status!=completed"

# Urgent issues without assignee
linear issues --filter "priority:urgent AND assignee:null"

# Recently created bugs
linear issues --filter "label:bug AND created>7d"

# Stale issues (not updated in 30 days)
linear issues --filter "updated<30d AND status!=completed"

# Issues in specific projects
linear issues --filter "project:Backend AND status:started"

# Issues with multiple statuses
linear issues --filter "status in:backlog,unstarted,started"
```

### Complex Queries

```bash
# High priority bugs created this week that aren't assigned
linear issues --filter "(priority>2 OR label:urgent) AND label:bug AND created>7d AND assignee:null"

# Old issues that need attention
linear issues --filter "created<30d AND updated<7d AND status!=completed AND priority>=2"

# Issues in review or testing
linear issues --filter "status in:review,testing,qa AND assignee!=null"
```

## Filter Builder API (For Developers)

The Linear CLI also provides a programmatic filter builder API:

```rust
use linear_cli::filtering::FilterBuilder;

let filter = FilterBuilder::new()
    .status().not_equals("completed")
    .and()
    .priority().greater_than(2)
    .and()
    .created_at().within_days(7)
    .build()?;

let graphql = filter.to_graphql()?;
```

## Tips

1. **Use saved searches** for filters you use frequently
2. **Combine filters** to narrow down results effectively
3. **Use relative dates** for dynamic time-based filtering
4. **Quote values** that contain spaces or special characters
5. **Test filters** with small result sets first to ensure they work as expected

## Troubleshooting

If a filter doesn't work as expected:

1. Check field names are spelled correctly
2. Ensure operators are valid for the field type
3. Use quotes around values with spaces
4. Try simpler filters first, then build up complexity
5. Use `--debug` flag to see the generated GraphQL filter

For help with filter syntax:
```bash
linear filter-help
```