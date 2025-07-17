use clap::ArgMatches;
use crate::client::LinearClient;
use crate::config::get_api_key;

pub async fn handle_whoami(_matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let api_key = get_api_key()?;
    let client = LinearClient::new(api_key);

    let user = client.get_viewer().await?;
    println!("Logged in as: {} ({})", user.name, user.email);
    println!("User ID: {}", user.id);

    Ok(())
}