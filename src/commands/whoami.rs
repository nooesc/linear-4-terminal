use clap::ArgMatches;
use crate::cli_context::CliContext;
use crate::error::{LinearError, LinearResult, ErrorContext};

pub async fn handle_whoami(_matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    handle_whoami_impl(_matches).await.map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}

async fn handle_whoami_impl(_matches: &ArgMatches) -> LinearResult<()> {
    let mut context = CliContext::load().context("Failed to load CLI context")?;
    let client = context.verified_client().context("Failed to get Linear client")?;

    let user = client.get_viewer().await
        .map_err(|e| LinearError::ApiError(format!("Failed to get current user: {}", e)))
        .context("Getting viewer information")?;
    println!("Logged in as: {} ({})", user.name, user.email);
    println!("User ID: {}", user.id);

    Ok(())
}