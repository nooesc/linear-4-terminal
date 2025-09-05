use clap::ArgMatches;
use crate::cli_context::CliContext;
use crate::error::{LinearResult, ErrorContext};

pub async fn handle_auth(matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    handle_auth_impl(matches).await.map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}

async fn handle_auth_impl(matches: &ArgMatches) -> LinearResult<()> {
    if let Some(api_key) = matches.get_one::<String>("api-key") {
        let mut context = CliContext::new();
        context.set_api_key(api_key.clone())
            .context("Failed to save API key")?;
        println!("API key saved successfully!");
        
        // Test the API key
        let client = context.verified_client()
            .context("Failed to get Linear client")?;
        match client.get_viewer().await {
            Ok(user) => println!("✅ Connected as: {} ({})", user.name, user.email),
            Err(e) => println!("❌ Failed to authenticate: {}", e),
        }
    } else if matches.get_flag("show") {
        let mut context = CliContext::load()
            .context("Failed to load CLI context")?;
        match context.api_key() {
            Ok(key) => println!("API Key: {}...{}", &key[..8], &key[key.len()-4..]),
            Err(_) => println!("No API key configured"),
        }
    } else {
        println!("Usage: linear auth --api-key <KEY> or linear auth --show");
    }
    Ok(())
}