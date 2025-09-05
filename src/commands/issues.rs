use clap::ArgMatches;
use serde_json::json;
use crate::cli_context::CliContext;
use crate::error::{LinearError, ErrorContext};
use crate::filtering::FilterAdapter;
use crate::formatting::issues::{print_issues, print_single_issue};

pub async fn handle_issues(matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    // Create CLI context and get verified client
    let mut context = CliContext::load()
        .context("Failed to load CLI context")?;
    let client = context.verified_client()
        .context("Failed to get Linear client")?;
    
    let format = matches.get_one::<String>("format").map(|s| s.as_str()).unwrap_or("simple");
    let group_by = matches.get_one::<String>("group-by").map(|s| s.as_str()).unwrap_or("status");
    let limit = matches.get_one::<String>("limit")
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(50);

    let mut filter = json!({});
    
    // Check if advanced filter is provided
    if let Some(filter_query) = matches.get_one::<String>("filter") {
        // Try new filter system first, fall back to legacy if needed
        filter = FilterAdapter::parse_and_build(filter_query)
            .map_err(|e| LinearError::InvalidInput(format!("Failed to parse filter: {}", e)))
            .with_context(|| format!("Filter query: {}", filter_query))?;
    } else {
        // Handle legacy filters for backward compatibility
        // Handle state filters
        if matches.get_flag("todo") || matches.get_flag("backlog") {
            filter["state"] = json!({"type": {"in": ["backlog", "unstarted"]}});
        } else if matches.get_flag("triage") {
            filter["state"] = json!({"type": {"eq": "triage"}});
        } else if matches.get_flag("progress") || matches.get_flag("started") {
            filter["state"] = json!({"type": {"eq": "started"}});
        } else if matches.get_flag("done") || matches.get_flag("completed") {
            filter["state"] = json!({"type": {"eq": "completed"}});
        }

        // Handle assignee filters
        if matches.get_flag("mine") {
            let viewer = client.get_viewer().await
                .map_err(|e| LinearError::ApiError(format!("Failed to get current user: {}", e)))
                .context("Getting viewer information for --mine filter")?;
            filter["assignee"] = json!({"id": {"eq": viewer.id}});
        } else if let Some(assignee) = matches.get_one::<String>("assignee") {
            filter["assignee"] = json!({"email": {"eq": assignee}});
        }

        // Handle team filter
        if let Some(team) = matches.get_one::<String>("team") {
            filter["team"] = json!({"key": {"eq": team}});
        }

        // Handle search
        if let Some(search) = matches.get_one::<String>("search") {
            filter["title"] = json!({"containsIgnoreCase": search});
        }
    }

    let filter_param = if filter.as_object().unwrap().is_empty() {
        None
    } else {
        Some(filter)
    };

    let issues = client.get_issues(filter_param, Some(limit)).await
        .map_err(|e| LinearError::ApiError(format!("Failed to fetch issues: {}", e)))
        .context("Fetching issues from Linear API")?;
    
    if issues.is_empty() {
        println!("No issues found matching your criteria.");
    } else {
        println!("Found {} issues:", issues.len());
        print_issues(&issues, format, group_by);
    }

    Ok(())
}

pub async fn handle_issue(matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    // Create CLI context and get verified client
    let mut context = CliContext::load()
        .context("Failed to load CLI context")?;
    let client = context.verified_client()
        .context("Failed to get Linear client")?;
    
    let identifier = matches.get_one::<String>("identifier")
        .ok_or_else(|| LinearError::InvalidInput("Issue identifier is required".to_string()))?;
    
    let issue = client.get_issue_by_identifier(identifier).await
        .map_err(|e| LinearError::ApiError(format!("Failed to fetch issue: {}", e)))
        .with_context(|| format!("Fetching issue with identifier: {}", identifier))?;
    print_single_issue(&issue);
    
    Ok(())
}