use clap::ArgMatches;
use crate::client::LinearClient;
use crate::config::get_api_key;
use crate::formatting::issues::print_teams;

pub async fn handle_teams(_matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let api_key = get_api_key()?;
    let client = LinearClient::new(api_key);

    let teams = client.get_teams().await?;
    
    if teams.is_empty() {
        println!("No teams found.");
    } else {
        println!("Found {} teams:", teams.len());
        print_teams(&teams);
    }

    Ok(())
}