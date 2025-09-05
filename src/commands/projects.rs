use clap::ArgMatches;
use crate::cli_context::CliContext;
use crate::error::{LinearError, LinearResult, ErrorContext};
use crate::formatting::issues::print_projects;

pub async fn handle_projects(_matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    handle_projects_impl(_matches).await.map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}

async fn handle_projects_impl(_matches: &ArgMatches) -> LinearResult<()> {
    let mut context = CliContext::load().context("Failed to load CLI context")?;
    let client = context.verified_client().context("Failed to get Linear client")?;

    let projects = client.get_projects().await
        .map_err(|e| LinearError::ApiError(format!("Failed to get projects: {}", e)))
        .context("Getting projects")?;
    
    if projects.is_empty() {
        println!("No projects found.");
    } else {
        println!("Found {} projects:", projects.len());
        print_projects(&projects);
    }

    Ok(())
}