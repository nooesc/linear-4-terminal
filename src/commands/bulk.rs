use clap::ArgMatches;
use colored::*;
use crate::client::LinearClient;
use crate::config::get_api_key;

fn parse_issue_ids(matches: &ArgMatches) -> Vec<String> {
    let mut ids = Vec::new();
    
    if let Some(id_values) = matches.get_many::<String>("ids") {
        for id_value in id_values {
            // Split by comma if provided
            for id in id_value.split(',') {
                let trimmed = id.trim();
                if !trimmed.is_empty() {
                    ids.push(trimmed.to_string());
                }
            }
        }
    }
    
    ids
}

pub async fn handle_bulk_update(matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let api_key = get_api_key()?;
    let client = LinearClient::new(api_key);
    
    let issue_ids = parse_issue_ids(matches);
    if issue_ids.is_empty() {
        return Err("No issue IDs provided".into());
    }
    
    let state_id = matches.get_one::<String>("state");
    let assignee_id = matches.get_one::<String>("assignee");
    let priority = matches.get_one::<String>("priority")
        .and_then(|p| p.parse::<u8>().ok());
    let labels = matches.get_one::<String>("labels")
        .map(|l| l.split(',').map(|s| s.trim().to_string()).collect::<Vec<_>>());
    let remove_labels = matches.get_one::<String>("remove-labels")
        .map(|l| l.split(',').map(|s| s.trim().to_string()).collect::<Vec<_>>());
    
    if state_id.is_none() && assignee_id.is_none() && priority.is_none() && labels.is_none() && remove_labels.is_none() {
        return Err("No update parameters provided. Use --state, --assignee, --priority, --labels, or --remove-labels".into());
    }
    
    println!("Updating {} issues...", issue_ids.len());
    
    let mut success_count = 0;
    let mut failed_ids = Vec::new();
    
    for issue_id in &issue_ids {
        match client.update_issue_bulk(
            issue_id,
            state_id.map(|s| s.as_str()),
            assignee_id.map(|s| s.as_str()),
            priority,
            labels.as_ref().map(|v| v.as_slice()),
            remove_labels.as_ref().map(|v| v.as_slice()),
        ).await {
            Ok(_) => {
                success_count += 1;
                println!("  ✓ Updated {}", issue_id.bright_green());
            }
            Err(e) => {
                failed_ids.push(issue_id.clone());
                println!("  ✗ Failed to update {}: {}", issue_id.bright_red(), e);
            }
        }
    }
    
    println!("\n✅ Successfully updated {} out of {} issues", success_count, issue_ids.len());
    
    if !failed_ids.is_empty() {
        println!("❌ Failed to update: {}", failed_ids.join(", "));
    }
    
    Ok(())
}

pub async fn handle_bulk_move(matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let api_key = get_api_key()?;
    let client = LinearClient::new(api_key);
    
    let issue_ids = parse_issue_ids(matches);
    if issue_ids.is_empty() {
        return Err("No issue IDs provided".into());
    }
    
    let team_id = matches.get_one::<String>("team");
    let project_id = matches.get_one::<String>("project");
    
    if team_id.is_none() && project_id.is_none() {
        return Err("No move parameters provided. Use --team or --project".into());
    }
    
    println!("Moving {} issues...", issue_ids.len());
    
    let mut success_count = 0;
    let mut failed_ids = Vec::new();
    
    for issue_id in &issue_ids {
        match client.move_issue(
            issue_id,
            team_id.map(|s| s.as_str()),
            project_id.map(|s| s.as_str()),
        ).await {
            Ok(_) => {
                success_count += 1;
                println!("  ✓ Moved {}", issue_id.bright_green());
            }
            Err(e) => {
                failed_ids.push(issue_id.clone());
                println!("  ✗ Failed to move {}: {}", issue_id.bright_red(), e);
            }
        }
    }
    
    println!("\n✅ Successfully moved {} out of {} issues", success_count, issue_ids.len());
    
    if !failed_ids.is_empty() {
        println!("❌ Failed to move: {}", failed_ids.join(", "));
    }
    
    Ok(())
}

pub async fn handle_bulk_archive(matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let api_key = get_api_key()?;
    let client = LinearClient::new(api_key);
    
    let issue_ids = parse_issue_ids(matches);
    if issue_ids.is_empty() {
        return Err("No issue IDs provided".into());
    }
    
    println!("Archiving {} issues...", issue_ids.len());
    
    let mut success_count = 0;
    let mut failed_ids = Vec::new();
    
    for issue_id in &issue_ids {
        match client.archive_issue(issue_id).await {
            Ok(success) => {
                if success {
                    success_count += 1;
                    println!("  ✓ Archived {}", issue_id.bright_green());
                } else {
                    failed_ids.push(issue_id.clone());
                    println!("  ✗ Failed to archive {}", issue_id.bright_red());
                }
            }
            Err(e) => {
                failed_ids.push(issue_id.clone());
                println!("  ✗ Failed to archive {}: {}", issue_id.bright_red(), e);
            }
        }
    }
    
    println!("\n✅ Successfully archived {} out of {} issues", success_count, issue_ids.len());
    
    if !failed_ids.is_empty() {
        println!("❌ Failed to archive: {}", failed_ids.join(", "));
    }
    
    Ok(())
}