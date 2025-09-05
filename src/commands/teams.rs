use clap::ArgMatches;
use crate::cli_context::CliContext;
use crate::error::{LinearError, LinearResult, ErrorContext};
use crate::formatting::issues::print_teams;

pub async fn handle_teams(_matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    handle_teams_impl(_matches).await.map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}

async fn handle_teams_impl(_matches: &ArgMatches) -> LinearResult<()> {
    let mut context = CliContext::load().context("Failed to load CLI context")?;
    let client = context.verified_client().context("Failed to get Linear client")?;

    let teams = client.get_teams().await
        .map_err(|e| LinearError::ApiError(format!("Failed to get teams: {}", e)))
        .context("Getting teams")?;
    
    if teams.is_empty() {
        println!("No teams found.");
    } else {
        println!("Found {} teams:", teams.len());
        print_teams(&teams);
    }

    Ok(())
}