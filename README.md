# Linear CLI Tool

A comprehensive command-line interface for Linear's project management API, built in Rust. This tool allows you to interact with Linear from your terminal, manage issues, projects, teams, and more.

## Features

✅ **Authentication**: Support for API keys and secure configuration  
✅ **Issue Management**: Create, list, filter, and search issues  
✅ **Project Management**: Create and list projects  
✅ **Team Management**: List teams and their information  
✅ **Advanced Filtering**: Filter by status, assignee, team, priority  
✅ **Multiple Output Formats**: Simple, table, and JSON output  
✅ **Real-time Data**: Direct integration with Linear's GraphQL API  

## Installation

### Prerequisites
- Rust and Cargo installed ([Install Rust](https://rustup.rs/))
- Linear API key ([Get your API key](https://linear.app/settings/api))

### Build from Source

1. **Clone or create the project:**
```bash
mkdir linear-cli && cd linear-cli
# Copy the main.rs content to src/main.rs
# Copy the Cargo.toml content to Cargo.toml
```

2. **Build the project:**
```bash
cargo build --release
```

3. **Install globally (optional):**
```bash
cargo install --path .
```

Or copy the binary to your PATH:
```bash
cp target/release/linear-cli /usr/local/bin/linear
```

## Configuration

### Set up Authentication

1. **Get your Linear API key:**
   - Go to [Linear Settings > API](https://linear.app/settings/api)
   - Create a new personal API key
   - Copy the key

2. **Configure the CLI:**
```bash
# Method 1: Use the auth command
linear auth --api-key lin_api_your_key_here

# Method 2: Set environment variable
export LINEAR_API_KEY=lin_api_your_key_here
```

3. **Verify authentication:**
```bash
linear whoami
linear auth --show
```

## Usage

### Authentication Commands

```bash
# Set API key
linear auth --api-key lin_api_your_key_here

# Show current API key (masked)
linear auth --show
```

### Issue Commands

#### List Issues
```bash
# List all issues (default: 50 most recent)
linear issues

# Filter by status
linear issues --todo           # Todo/Backlog issues
linear issues --triage         # Issues in triage
linear issues --progress       # Issues in progress
linear issues --done           # Completed issues

# Filter by assignee
linear issues --mine           # Issues assigned to you
linear issues --assignee user@example.com

# Filter by team
linear issues --team ENG       # Issues from ENG team

# Search issues
linear issues --search "bug"   # Search in titles

# Combine filters
linear issues --mine --progress --team ENG

# Limit results and format output
linear issues --limit 10 --format table
linear issues --format json > issues.json
```

#### Create Issues
```bash
# Basic issue creation
linear create issue "Fix login bug" "Users can't log in"

# With additional parameters
linear create issue "New feature" "Implement dark mode" \
  --team ENG \
  --priority high \
  --assignee user_id_here \
  --labels label_id_1 label_id_2

# Priority levels: none/0, low/1, medium/2, high/3, urgent/4
linear create issue "Urgent fix" "Critical bug" --priority urgent
```

### Project Commands

#### List Projects
```bash
# List all projects
linear projects
```

#### Create Projects
```bash
# Basic project creation
linear create project "Q4 Initiative" "Major improvements for Q4"

# With teams
linear create project "Mobile App" "iOS and Android apps" \
  --teams team_id_1 team_id_2
```

### Team Commands

```bash
# List all teams
linear teams
```

### User Commands

```bash
# Show current user info
linear whoami
```

## Output Formats

The CLI supports multiple output formats:

### Simple Format (Default)
```bash
linear issues
# • issue-id - Issue Title (Status)
```

### Table Format
```bash
linear issues --format table
# ID                   Title                        State     Team    Assignee
# -------------------------------------------------------------------------
# abc123              Fix login bug                Todo      ENG     John Doe
```

### JSON Format
```bash
linear issues --format json
# Full JSON output with all issue data
```

## Examples

### Daily Workflow Examples

```bash
# Check your assigned issues
linear issues --mine

# Check what's in triage for your team
linear issues --triage --team ENG

# Create a bug report
linear create issue "Login button not working" \
  "The login button doesn't respond on mobile devices" \
  --team ENG --priority high

# Check team progress
linear issues --progress --team ENG --format table

# Search for specific issues
linear issues --search "authentication" --format table
```

### Project Management Examples

```bash
# Review all projects
linear projects

# Create a new project
linear create project "Website Redesign" \
  "Complete overhaul of company website" \
  --teams design_team_id eng_team_id

# Check issues for a specific project
linear issues --search "website"
```

### Reporting Examples

```bash
# Generate JSON report of all completed issues
linear issues --done --format json > completed_issues.json

# Get table view of current sprint
linear issues --progress --format table

# Export team's backlog
linear issues --backlog --team ENG --limit 100 --format json > backlog.json
```

## Configuration File

The CLI stores configuration in `~/.linear-cli-config.json`:

```json
{
  "api_key": "lin_api_your_key_here",
  "default_team_id": "team_id_here"
}
```

## Error Handling

The CLI provides helpful error messages:

```bash
# No API key configured
$ linear issues
Error: No API key found. Set LINEAR_API_KEY environment variable or run 'linear auth' to configure.

# Invalid team
$ linear create issue "Test" --team INVALID
Error: Team 'INVALID' not found

# GraphQL errors are displayed clearly
$ linear create issue ""
Error: GraphQL errors: Issue title cannot be empty
```

## Advanced Features

### Environment Variables

```bash
# Set API key via environment
export LINEAR_API_KEY=lin_api_your_key_here

# Override default team
export LINEAR_DEFAULT_TEAM=ENG
```

### Scripting Examples

```bash
#!/bin/bash
# Daily standup script

echo "=== My Issues in Progress ==="
linear issues --mine --progress --format table

echo -e "\n=== Triage Items for My Team ==="
linear issues --triage --team ENG --format table

echo -e "\n=== Recently Completed ==="
linear issues --mine --done --limit 5
```

```bash
#!/bin/bash
# Create issue from git commit
COMMIT_MSG=$(git log -1 --pretty=%B)
BRANCH_NAME=$(git branch --show-current)

linear create issue "Fix: $BRANCH_NAME" "$COMMIT_MSG" \
  --team ENG --priority medium
```

## API Coverage

This CLI covers the major Linear API operations:

### Queries
- ✅ Get viewer information
- ✅ List issues with filtering
- ✅ List teams
- ✅ List projects
- ✅ Search functionality

### Mutations
- ✅ Create issues
- ✅ Create projects
- 🔄 Update issues (planned)
- 🔄 Update projects (planned)
- 🔄 Delete operations (planned)

### Filters
- ✅ State-based filtering (todo, triage, progress, done)
- ✅ Assignee filtering
- ✅ Team filtering
- ✅ Search/text filtering
- ✅ Pagination with limits

## Troubleshooting

### Common Issues

1. **Authentication Errors**
   ```bash
   # Verify your API key
   linear auth --show
   linear whoami
   ```

2. **Team Not Found**
   ```bash
   # List available teams first
   linear teams
   ```

3. **Rate Limiting**
   The CLI respects Linear's rate limits. If you hit limits, wait a moment and try again.

4. **Network Issues**
   ```bash
   # Check connectivity to Linear's API
   curl -I https://api.linear.app/graphql
   ```

### Debug Mode

For debugging, you can inspect the API calls by modifying the code to add debug logging or use tools like `RUST_LOG=debug cargo run`.

## Contributing

This is a single-file CLI tool that can be easily extended. Some areas for improvement:

- [ ] Add issue update/delete operations
- [ ] Add comment management
- [ ] Add label management
- [ ] Add more sophisticated filtering
- [ ] Add configuration for default values
- [ ] Add shell completion scripts
- [ ] Add more output formatting options

## License

MIT License - feel free to modify and distribute.
