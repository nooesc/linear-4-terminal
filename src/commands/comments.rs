use clap::ArgMatches;
use crate::cli_context::CliContext;
use crate::error::{LinearError, LinearResult, ErrorContext};
use crate::formatting::markdown::format_markdown;
use crate::formatting::utils::format_relative_time;
use colored::*;

pub async fn handle_list_comments(matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    handle_list_comments_impl(matches).await.map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}

async fn handle_list_comments_impl(matches: &ArgMatches) -> LinearResult<()> {
    let mut context = CliContext::load().context("Failed to load CLI context")?;
    let client = context.verified_client().context("Failed to get Linear client")?;
    
    let issue_identifier = matches.get_one::<String>("issue")
        .ok_or_else(|| LinearError::InvalidInput("Issue identifier is required".to_string()))?;
    
    // First get the issue to get its ID
    let issue = client.get_issue_by_identifier(issue_identifier).await
        .map_err(|e| LinearError::ApiError(format!("Failed to get issue: {}", e)))
        .context("Getting issue by identifier")?;
    let comments = client.get_comments(&issue.id).await
        .map_err(|e| LinearError::ApiError(format!("Failed to get comments: {}", e)))
        .context("Getting comments for issue")?;
    
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
    handle_add_comment_impl(matches).await.map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}

async fn handle_add_comment_impl(matches: &ArgMatches) -> LinearResult<()> {
    let mut context = CliContext::load().context("Failed to load CLI context")?;
    let client = context.verified_client().context("Failed to get Linear client")?;
    
    let issue_identifier = matches.get_one::<String>("issue")
        .ok_or_else(|| LinearError::InvalidInput("Issue identifier is required".to_string()))?;
    let body = matches.get_one::<String>("body")
        .ok_or_else(|| LinearError::InvalidInput("Comment body is required".to_string()))?;
    
    // First get the issue to get its ID
    let issue = client.get_issue_by_identifier(issue_identifier).await
        .map_err(|e| LinearError::ApiError(format!("Failed to get issue: {}", e)))
        .context("Getting issue by identifier")?;
    let comment = client.create_comment(&issue.id, body).await
        .map_err(|e| LinearError::ApiError(format!("Failed to create comment: {}", e)))
        .context("Creating comment")?;
    
    println!("✅ Comment added successfully!");
    println!("Issue: {} - {}", issue.identifier, issue.title);
    println!("Comment by: {}", comment.user.as_ref().map(|u| u.name.as_str()).unwrap_or("Unknown"));
    println!("\n{}", format_markdown(&comment.body));
    
    Ok(())
}

pub async fn handle_update_comment(matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    handle_update_comment_impl(matches).await.map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}

async fn handle_update_comment_impl(matches: &ArgMatches) -> LinearResult<()> {
    let mut context = CliContext::load().context("Failed to load CLI context")?;
    let client = context.verified_client().context("Failed to get Linear client")?;
    
    let comment_id = matches.get_one::<String>("id")
        .ok_or_else(|| LinearError::InvalidInput("Comment ID is required".to_string()))?;
    let body = matches.get_one::<String>("body")
        .ok_or_else(|| LinearError::InvalidInput("Comment body is required".to_string()))?;
    
    let comment = client.update_comment(comment_id, body).await
        .map_err(|e| LinearError::ApiError(format!("Failed to update comment: {}", e)))
        .context("Updating comment")?;
    
    println!("✅ Comment updated successfully!");
    println!("Comment ID: {}", comment.id);
    println!("Updated by: {}", comment.user.as_ref().map(|u| u.name.as_str()).unwrap_or("Unknown"));
    println!("\n{}", format_markdown(&comment.body));
    
    Ok(())
}

pub async fn handle_delete_comment(matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    handle_delete_comment_impl(matches).await.map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}

async fn handle_delete_comment_impl(matches: &ArgMatches) -> LinearResult<()> {
    let mut context = CliContext::load().context("Failed to load CLI context")?;
    let client = context.verified_client().context("Failed to get Linear client")?;
    
    let comment_id = matches.get_one::<String>("id")
        .ok_or_else(|| LinearError::InvalidInput("Comment ID is required".to_string()))?;
    
    let success = client.delete_comment(comment_id).await
        .map_err(|e| LinearError::ApiError(format!("Failed to delete comment: {}", e)))
        .context("Deleting comment")?;
    
    if success {
        println!("✅ Comment deleted successfully!");
        println!("Comment ID: {}", comment_id);
    } else {
        return Err(LinearError::ApiError("Failed to delete comment".to_string()));
    }
    
    Ok(())
}