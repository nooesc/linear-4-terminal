# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Linear CLI is a comprehensive terminal client for Linear.app's GraphQL API, built with Rust. It provides both command-line interface and an interactive TUI mode for managing Linear issues, projects, teams, and comments.

## Build and Development Commands

```bash
# Build debug version
cargo build

# Build release version (recommended for testing)
cargo build --release

# Run directly without installing
cargo run -- <command>

# Install to ~/.cargo/bin
cargo install --path .

# Install system-wide (requires sudo)
sudo cp target/release/linear /usr/local/bin/

# Run interactive mode
cargo run -- interactive
# or just
cargo run
```

## Architecture Overview

### Core Components

1. **Client Layer** (`src/client/`)
   - `LinearClient` handles all GraphQL API communication
   - Uses reqwest for HTTP requests
   - Implements parallel API calls for performance (see `InteractiveApp::new()`)
   - Methods like `update_issue_with_project()` handle special cases (e.g., null values)

2. **Command Structure** (`src/commands/`)
   - Each command module (issues, projects, auth, etc.) handles CLI subcommands
   - Commands parse args, call client methods, and format output
   - Special commands: `bulk.rs` for batch operations, `git.rs` for Git integration

3. **Interactive Mode** (`src/interactive/`)
   - `app.rs`: Core state management (`InteractiveApp` struct)
   - `ui.rs`: Terminal UI rendering with ratatui
   - `handlers.rs`: Keyboard event handling
   - Features dynamic column width calculation based on terminal size
   - Implements responsive design with visibility flags

4. **Models** (`src/models/`)
   - GraphQL response types matching Linear's API schema
   - Key types: `Issue`, `Project`, `User`, `WorkflowState`, `Label`
   - `Connection<T>` wrapper for paginated responses

5. **Configuration** (`src/config/`)
   - Stores API key in `~/.linear-cli-config.json`
   - Environment variable support: `LINEAR_API_KEY`

### Key Design Patterns

1. **State Management in Interactive Mode**
   - `AppMode` enum tracks current UI state
   - `previous_mode` field enables proper navigation flow
   - Quick edit shortcuts (s/c/l/p) preserve navigation context

2. **Responsive Table Layout**
   - `ColumnWidths` struct dynamically calculates column sizes
   - Terminal width breakpoints: <80, <100, <120, <150, <180, 180+
   - Proportional space distribution for optimal readability

3. **API Optimization**
   - Parallel API calls using `tokio::join!` on startup
   - GraphQL field constants in `constants.rs` ensure consistent queries
   - Error handling preserves partial results (e.g., if labels fetch fails)

4. **Color System**
   - Status colors: backlog(Gray), unstarted(LightBlue), started(Yellow), completed(Green)
   - Priority colors: none(Gray), low(Blue), medium(Yellow), high(Orange), urgent(Red)
   - Project(LightGreen), Labels(Magenta), Assignee(Cyan)

### Recent Enhancements

1. **Performance**: Parallel API calls reduced startup time by 3x
2. **UX**: Quick edit returns to original view instead of detail view
3. **Features**: Project editing, label management, age column
4. **Layout**: Separate project/labels columns with optimized widths

## Critical Implementation Details

### GraphQL Mutations
- Removing project requires sending `projectId: null` (see `update_issue_with_project`)
- Label updates replace all labels (not additive)
- Some operations use `move_issue` vs `update_issue` depending on field

### Terminal UI Considerations
- Always account for 2-char border when calculating widths
- Priority symbols: ◦ (low), • (medium), ■ (high), ▲ (urgent)
- Links are extracted from markdown descriptions using regex
- Browser opening is OS-specific (open/xdg-open/start)

### State Persistence
- Selected issue title shown in header during all modes
- Scroll position maintained in links viewport
- Filter/search state preserved during operations