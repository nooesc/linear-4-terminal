use clap::ArgMatches;
use colored::*;
use crate::cli_context::CliContext;
use crate::error::{LinearError, LinearResult, ErrorContext};

pub async fn handle_update_issue(matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    handle_update_issue_impl(matches).await.map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}

async fn handle_update_issue_impl(matches: &ArgMatches) -> LinearResult<()> {
    let mut context = CliContext::load().context("Failed to load CLI context")?;
    let client = context.verified_client().context("Failed to get Linear client")?;

    let issue_id = matches.get_one::<String>("id")
        .ok_or_else(|| LinearError::InvalidInput("Issue ID is required".to_string()))?;
    
    let title = matches.get_one::<String>("title");
    let description = matches.get_one::<String>("description");
    let state_id = matches.get_one::<String>("state");
    let priority = matches.get_one::<String>("priority")
        .and_then(|p| match p.as_str() {
            "none" | "0" => Some(0),
            "low" | "1" => Some(1),
            "medium" | "2" => Some(2),
            "high" | "3" => Some(3),
            "urgent" | "4" => Some(4),
            _ => None,
        });
    let assignee_id = matches.get_one::<String>("assignee");
    let label_ids: Option<Vec<&str>> = matches.get_many::<String>("labels")
        .map(|labels| labels.map(|s| s.as_str()).collect());

    // Check if at least one field is being updated
    if title.is_none() && description.is_none() && state_id.is_none() && 
       priority.is_none() && assignee_id.is_none() && label_ids.is_none() {
        return Err(LinearError::InvalidInput("No fields to update. Provide at least one field to update.".to_string()));
    }

    let issue = client.update_issue(
        issue_id,
        title.map(|s| s.as_str()),
        description.map(|s| s.as_str()),
        state_id.map(|s| s.as_str()),
        priority,
        assignee_id.map(|s| s.as_str()),
        label_ids,
    ).await
        .map_err(|e| LinearError::ApiError(format!("Failed to update issue: {}", e)))
        .context("Updating issue")?;

    println!("{} {}", "✅".green(), "Issue updated successfully!".green().bold());
    println!("{}: {}", "ID".bold(), issue.identifier.bright_blue().bold());
    println!("{}: {}", "Title".bold(), issue.title);
    println!("{}: {}", "URL".bold(), issue.url.bright_black());
    println!("{}: {}", "State".bold(), issue.state.name);

    Ok(())
}

pub async fn handle_update_project(matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    handle_update_project_impl(matches).await.map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}

async fn handle_update_project_impl(matches: &ArgMatches) -> LinearResult<()> {
    let mut context = CliContext::load().context("Failed to load CLI context")?;
    let client = context.verified_client().context("Failed to get Linear client")?;

    let project_id = matches.get_one::<String>("id")
        .ok_or_else(|| LinearError::InvalidInput("Project ID is required".to_string()))?;
    
    let name = matches.get_one::<String>("name");
    let description = matches.get_one::<String>("description");
    let state = matches.get_one::<String>("state");

    // Check if at least one field is being updated
    if name.is_none() && description.is_none() && state.is_none() {
        return Err(LinearError::InvalidInput("No fields to update. Provide at least one field to update.".to_string()));
    }

    let project = client.update_project(
        project_id,
        name.map(|s| s.as_str()),
        description.map(|s| s.as_str()),
        state.map(|s| s.as_str()),
    ).await
        .map_err(|e| LinearError::ApiError(format!("Failed to update project: {}", e)))
        .context("Updating project")?;

    println!("✅ Project updated successfully!");
    println!("ID: {}", project.id);
    println!("Name: {}", project.name);
    println!("URL: {}", project.url);
    println!("State: {}", project.state);

    Ok(())
}