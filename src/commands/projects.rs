use clap::ArgMatches;
use crate::client::LinearClient;
use crate::config::get_api_key;
use crate::formatting::issues::print_projects;

pub async fn handle_projects(_matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let api_key = get_api_key()?;
    let client = LinearClient::new(api_key);

    let projects = client.get_projects().await?;
    
    if projects.is_empty() {
        println!("No projects found.");
    } else {
        println!("Found {} projects:", projects.len());
        print_projects(&projects);
    }

    Ok(())
}