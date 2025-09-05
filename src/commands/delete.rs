use clap::ArgMatches;
use crate::cli_context::CliContext;
use crate::error::{LinearError, LinearResult, ErrorContext};

pub async fn handle_delete(matches: &ArgMatches, resource_type: &str) -> Result<(), Box<dyn std::error::Error>> {
    handle_delete_impl(matches, resource_type).await.map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}

async fn handle_delete_impl(matches: &ArgMatches, resource_type: &str) -> LinearResult<()> {
    let mut context = CliContext::load().context("Failed to load CLI context")?;
    let client = context.verified_client().context("Failed to get Linear client")?;

    let id = matches.get_one::<String>("id")
        .ok_or_else(|| LinearError::InvalidInput(format!("{} ID is required", resource_type)))?;
    
    let success = match resource_type {
        "Issue" => client.archive_issue(id).await
            .map_err(|e| LinearError::ApiError(format!("Failed to archive issue: {}", e)))
            .context("Archiving issue")?,
        "Project" => client.archive_project(id).await
            .map_err(|e| LinearError::ApiError(format!("Failed to archive project: {}", e)))
            .context("Archiving project")?,
        _ => return Err(LinearError::InvalidInput("Invalid resource type".to_string())),
    };
    
    if success {
        println!("✅ {} archived successfully!", resource_type);
        println!("{} ID: {}", resource_type, id);
    } else {
        return Err(LinearError::ApiError(format!("Failed to archive {}", resource_type.to_lowercase())));
    }

    Ok(())
}