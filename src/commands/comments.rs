use clap::ArgMatches;
use crate::client::LinearClient;
use crate::config::get_api_key;
use crate::formatting::markdown::format_markdown;
use crate::formatting::utils::format_relative_time;
use colored::*;

pub async fn handle_list_comments(matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let api_key = get_api_key()?;
    let client = LinearClient::new(api_key);
    
    let issue_identifier = matches.get_one::<String>("issue")
        .ok_or("Issue identifier is required")?;
    
    // First get the issue to get its ID
    let issue = client.get_issue_by_identifier(issue_identifier).await?;
    let comments = client.get_comments(&issue.id).await?;
    
    if comments.is_empty() {
        println!("No comments found on issue {}.", issue_identifier);
    } else {
        println!("Comments on {} - {}:", issue.identifier, issue.title);
        println!("{}", "─".repeat(80));
        
        for comment in comments {
            println!("\n{} {} - {}", 
                "▸".bright_blue(),
                comment.user.as_ref().map(|u| u.name.as_str()).unwrap_or("Unknown").bright_cyan(),
                format_relative_time(&comment.created_at).dimmed()
            );
            if comment.created_at != comment.updated_at {
                println!("  {} {}", 
                    "Updated:".dimmed(),
                    format_relative_time(&comment.updated_at).dimmed()
                );
            }
            println!("\n{}", format_markdown(&comment.body));
            println!("{}", "─".repeat(40).dimmed());
        }
    }
    
    Ok(())
}

pub async fn handle_add_comment(matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let api_key = get_api_key()?;
    let client = LinearClient::new(api_key);
    
    let issue_identifier = matches.get_one::<String>("issue")
        .ok_or("Issue identifier is required")?;
    let body = matches.get_one::<String>("body")
        .ok_or("Comment body is required")?;
    
    // First get the issue to get its ID
    let issue = client.get_issue_by_identifier(issue_identifier).await?;
    let comment = client.create_comment(&issue.id, body).await?;
    
    println!("✅ Comment added successfully!");
    println!("Issue: {} - {}", issue.identifier, issue.title);
    println!("Comment by: {}", comment.user.as_ref().map(|u| u.name.as_str()).unwrap_or("Unknown"));
    println!("\n{}", format_markdown(&comment.body));
    
    Ok(())
}

pub async fn handle_update_comment(matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let api_key = get_api_key()?;
    let client = LinearClient::new(api_key);
    
    let comment_id = matches.get_one::<String>("id")
        .ok_or("Comment ID is required")?;
    let body = matches.get_one::<String>("body")
        .ok_or("Comment body is required")?;
    
    let comment = client.update_comment(comment_id, body).await?;
    
    println!("✅ Comment updated successfully!");
    println!("Comment ID: {}", comment.id);
    println!("Updated by: {}", comment.user.as_ref().map(|u| u.name.as_str()).unwrap_or("Unknown"));
    println!("\n{}", format_markdown(&comment.body));
    
    Ok(())
}

pub async fn handle_delete_comment(matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let api_key = get_api_key()?;
    let client = LinearClient::new(api_key);
    
    let comment_id = matches.get_one::<String>("id")
        .ok_or("Comment ID is required")?;
    
    let success = client.delete_comment(comment_id).await?;
    
    if success {
        println!("✅ Comment deleted successfully!");
        println!("Comment ID: {}", comment_id);
    } else {
        return Err("Failed to delete comment".into());
    }
    
    Ok(())
}