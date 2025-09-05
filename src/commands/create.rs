use clap::ArgMatches;
use colored::*;
use crate::cli_context::CliContext;
use crate::config::load_config;
use crate::error::{LinearError, LinearResult, ErrorContext};

pub async fn handle_create_issue(matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    handle_create_issue_impl(matches).await.map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}

async fn handle_create_issue_impl(matches: &ArgMatches) -> LinearResult<()> {
    let mut context = CliContext::load().context("Failed to load CLI context")?;
    let client = context.verified_client().context("Failed to get Linear client")?;

    let title = matches.get_one::<String>("title")
        .ok_or_else(|| LinearError::InvalidInput("Title is required".to_string()))?;
    let description = matches.get_one::<String>("description");
    
    // Get team ID
    let team_id = if let Some(team_key) = matches.get_one::<String>("team") {
        let teams = client.get_teams().await
            .map_err(|e| LinearError::ApiError(format!("Failed to get teams: {}", e)))
            .context("Getting teams for team lookup")?;
        teams.iter()
            .find(|t| t.key == *team_key)
            .map(|t| t.id.clone())
            .ok_or_else(|| LinearError::InvalidInput(format!("Team '{}' not found", team_key)))?
    } else {
        let config = load_config();
        config.default_team_id
            .ok_or_else(|| LinearError::InvalidInput("No team specified and no default team configured".to_string()))?
    };

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

    let issue = client.create_issue(
        title,
        description.map(|s| s.as_str()),
        &team_id,
        priority,
        assignee_id.map(|s| s.as_str()),
        label_ids,
    ).await
        .map_err(|e| LinearError::ApiError(format!("Failed to create issue: {}", e)))
        .context("Creating issue")?;

    println!("{} {}", "✅".green(), "Issue created successfully!".green().bold());
    println!("{}: {}", "ID".bold(), issue.identifier.bright_blue().bold());
    println!("{}: {}", "Title".bold(), issue.title);
    println!("{}: {}", "URL".bold(), issue.url.bright_black());
    println!("{}: {}", "Team".bold(), issue.team.name);
    println!("{}: {}", "State".bold(), issue.state.name);

    Ok(())
}

pub async fn handle_create_project(matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    handle_create_project_impl(matches).await.map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}

async fn handle_create_project_impl(matches: &ArgMatches) -> LinearResult<()> {
    let mut context = CliContext::load().context("Failed to load CLI context")?;
    let client = context.verified_client().context("Failed to get Linear client")?;

    let name = matches.get_one::<String>("name")
        .ok_or_else(|| LinearError::InvalidInput("Project name is required".to_string()))?;
    let description = matches.get_one::<String>("description");
    
    let mut team_ids: Vec<String> = matches.get_many::<String>("teams")
        .map(|teams| teams.cloned().collect())
        .unwrap_or_else(Vec::new);

    // If no teams specified, get the first available team
    if team_ids.is_empty() {
        let teams = client.get_teams().await
            .map_err(|e| LinearError::ApiError(format!("Failed to get teams: {}", e)))
            .context("Getting teams list")?;
        if teams.is_empty() {
            return Err(LinearError::InvalidInput("No teams found. Projects require at least one team.".to_string()));
        }
        eprintln!("No team specified. Using default team: {} ({})", teams[0].name, teams[0].key);
        team_ids.push(teams[0].id.clone());
    }

    let team_refs: Vec<&str> = team_ids.iter().map(|s| s.as_str()).collect();

    match client.create_project(
        name,
        description.map(|s| s.as_str()),
        Some(team_refs),
    ).await {
        Ok(project) => {
            println!("✅ Project created successfully!");
            println!("ID: {}", project.id);
            println!("Name: {}", project.name);
            println!("URL: {}", project.url);
            Ok(())
        }
        Err(e) => {
            eprintln!("Failed to create project: {}", e);
            eprintln!("\nTip: Projects require at least one team. Use --teams flag with team ID.");
            eprintln!("Run 'linear teams' to see available teams.");
            Err(LinearError::ApiError(format!("Failed to create project: {}", e)))
        }
    }
}