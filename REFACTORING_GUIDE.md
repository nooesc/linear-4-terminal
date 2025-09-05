# Linear CLI Refactoring Guide

This guide demonstrates how to refactor Linear CLI commands to use the new CliContext pattern and improved error handling.

## Key Changes

### 1. Import Changes

Replace:
```rust
use crate::client::LinearClient;
use crate::config::get_api_key;
```

With:
```rust
use crate::cli_context::CliContext;
use crate::error::{LinearError, LinearResult, ErrorContext};
```

### 2. Client Creation Pattern

Replace:
```rust
let api_key = get_api_key()?;
let client = LinearClient::new(api_key);
```

With:
```rust
// Create CLI context and get verified client
let mut context = CliContext::load()
    .context("Failed to load CLI context")?;
let client = context.verified_client()
    .context("Failed to get Linear client")?;
```

### 3. Error Handling Pattern

Replace generic error handling:
```rust
match parse_filter_query(filter_query) {
    Ok(filters) => {
        filter = build_graphql_filter(filters);
    }
    Err(e) => {
        eprintln!("Error parsing filter: {}", e);
        eprintln!("Use --help to see filter syntax examples");
        return Err(e.into());
    }
}
```

With specific error types and context:
```rust
let filters = parse_filter_query(filter_query)
    .map_err(|e| LinearError::InvalidInput(format!("Failed to parse filter: {}", e)))
    .with_context(|| format!("Filter query: {}", filter_query))?;
filter = build_graphql_filter(filters);
```

### 4. API Call Error Handling

Replace:
```rust
let viewer = client.get_viewer().await?;
```

With:
```rust
let viewer = client.get_viewer().await
    .map_err(|e| LinearError::ApiError(format!("Failed to get current user: {}", e)))
    .context("Getting viewer information for --mine filter")?;
```

### 5. Function Signatures

Keep function signatures as `Result<(), Box<dyn std::error::Error>>` for compatibility with main.rs:
```rust
pub async fn handle_issues(matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    // Use LinearError internally, it will be automatically converted
}
```

## Example: Refactored issues.rs

The `src/commands/issues.rs` file has been fully refactored to demonstrate the pattern:

1. Uses CliContext for client management
2. Provides specific error types (InvalidInput, ApiError)
3. Adds context to all errors for better debugging
4. Maintains backward compatibility with existing command interface

## Benefits

1. **Centralized Configuration**: CliContext manages API keys and client instances
2. **Better Error Messages**: Users get specific, contextual error messages
3. **Type Safety**: LinearError enum ensures all errors are handled consistently
4. **Easier Testing**: CliContext can be mocked for unit tests
5. **Future Extensibility**: CliContext can be extended with additional features (caching, rate limiting, etc.)

## Next Steps

Apply this pattern to other command modules:
- [ ] auth.rs
- [ ] bulk.rs
- [ ] comments.rs
- [ ] create.rs
- [ ] delete.rs
- [ ] git.rs
- [ ] projects.rs
- [ ] search.rs
- [ ] teams.rs
- [ ] update.rs
- [ ] whoami.rs