use clap::ArgMatches;
use crate::client::LinearClient;
use crate::config::get_api_key;

pub async fn handle_delete(matches: &ArgMatches, resource_type: &str) -> Result<(), Box<dyn std::error::Error>> {
    let api_key = get_api_key()?;
    let client = LinearClient::new(api_key);

    let id = matches.get_one::<String>("id")
        .ok_or(format!("{} ID is required", resource_type))?;
    
    let success = match resource_type {
        "Issue" => client.archive_issue(id).await?,
        "Project" => client.archive_project(id).await?,
        _ => return Err("Invalid resource type".into()),
    };
    
    if success {
        println!("✅ {} archived successfully!", resource_type);
        println!("{} ID: {}", resource_type, id);
    } else {
        return Err(format!("Failed to archive {}", resource_type.to_lowercase()).into());
    }

    Ok(())
}