use clap::ArgMatches;
use crate::client::LinearClient;
use crate::config::{load_config, save_config};

pub async fn handle_auth(matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(api_key) = matches.get_one::<String>("api-key") {
        let mut config = load_config();
        config.api_key = Some(api_key.clone());
        save_config(&config)?;
        println!("API key saved successfully!");
        
        // Test the API key
        let client = LinearClient::new(api_key.clone());
        match client.get_viewer().await {
            Ok(user) => println!("✅ Connected as: {} ({})", user.name, user.email),
            Err(e) => println!("❌ Failed to authenticate: {}", e),
        }
    } else if matches.get_flag("show") {
        let config = load_config();
        match config.api_key {
            Some(key) => println!("API Key: {}...{}", &key[..8], &key[key.len()-4..]),
            None => println!("No API key configured"),
        }
    } else {
        println!("Usage: linear auth --api-key <KEY> or linear auth --show");
    }
    Ok(())
}