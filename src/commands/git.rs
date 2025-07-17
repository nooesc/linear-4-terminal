use clap::ArgMatches;
use colored::*;
use regex::Regex;
use std::process::Command;
use crate::client::LinearClient;
use crate::config::get_api_key;

// Common Linear issue ID patterns
const ISSUE_PATTERN: &str = r"([A-Z]{2,}-\d+)";

pub async fn handle_git_commit(matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let message = matches.get_one::<String>("message")
        .ok_or("Commit message is required")?;
    let issue_id = matches.get_one::<String>("issue");
    let push = matches.get_flag("push");
    
    // Extract issue IDs from message or use provided one
    let issue_ids = if let Some(id) = issue_id {
        vec![id.clone()]
    } else {
        extract_issue_ids(message)
    };
    
    // Format commit message with issue references
    let formatted_message = if !issue_ids.is_empty() && !message.contains(&issue_ids[0]) {
        format!("{}: {}", issue_ids.join(", "), message)
    } else {
        message.clone()
    };
    
    // Create the commit
    let output = Command::new("git")
        .args(&["commit", "-m", &formatted_message])
        .output()?;
    
    if !output.status.success() {
        return Err(format!("Git commit failed: {}", String::from_utf8_lossy(&output.stderr)).into());
    }
    
    println!("✅ Commit created successfully!");
    println!("Message: {}", formatted_message);
    
    // Update Linear issue status if requested
    if matches.get_flag("update-status") {
        if let Some(new_state) = matches.get_one::<String>("status") {
            let api_key = get_api_key()?;
            let client = LinearClient::new(api_key);
            
            for issue_id in &issue_ids {
                match client.update_issue(
                    issue_id,
                    None,
                    None,
                    Some(new_state),
                    None,
                    None,
                    None,
                ).await {
                    Ok(_) => println!("  ✓ Updated {} status to {}", issue_id, new_state),
                    Err(e) => eprintln!("  ✗ Failed to update {}: {}", issue_id, e),
                }
            }
        }
    }
    
    // Push if requested
    if push {
        println!("\nPushing to remote...");
        let push_output = Command::new("git")
            .args(&["push"])
            .output()?;
        
        if push_output.status.success() {
            println!("✅ Pushed successfully!");
        } else {
            return Err(format!("Git push failed: {}", String::from_utf8_lossy(&push_output.stderr)).into());
        }
    }
    
    Ok(())
}

pub async fn handle_git_branch(matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let issue_id = matches.get_one::<String>("issue")
        .ok_or("Issue ID is required")?;
    let prefix = matches.get_one::<String>("prefix")
        .map(|s| s.as_str())
        .unwrap_or("feature");
    
    // Get issue details from Linear
    let api_key = get_api_key()?;
    let client = LinearClient::new(api_key);
    let issue = client.get_issue_by_identifier(issue_id).await?;
    
    // Create branch name from issue title
    let sanitized_title = sanitize_branch_name(&issue.title);
    let branch_name = format!("{}/{}-{}", prefix, issue.identifier.to_lowercase(), sanitized_title);
    
    // Create and checkout the branch
    let output = Command::new("git")
        .args(&["checkout", "-b", &branch_name])
        .output()?;
    
    if !output.status.success() {
        // Try just checking out if branch already exists
        let checkout_output = Command::new("git")
            .args(&["checkout", &branch_name])
            .output()?;
        
        if !checkout_output.status.success() {
            return Err(format!("Failed to create/checkout branch: {}", 
                String::from_utf8_lossy(&output.stderr)).into());
        }
        println!("Switched to existing branch: {}", branch_name);
    } else {
        println!("✅ Created and checked out new branch: {}", branch_name);
    }
    
    println!("\nIssue: {} - {}", issue.identifier.blue(), issue.title);
    println!("Branch: {}", branch_name.green());
    
    Ok(())
}

pub async fn handle_git_pr(matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let title = matches.get_one::<String>("title");
    let body = matches.get_one::<String>("body");
    let draft = matches.get_flag("draft");
    let web = matches.get_flag("web");
    
    // Get current branch
    let branch_output = Command::new("git")
        .args(&["branch", "--show-current"])
        .output()?;
    
    if !branch_output.status.success() {
        return Err("Failed to get current branch".into());
    }
    
    let current_branch = String::from_utf8_lossy(&branch_output.stdout).trim().to_string();
    
    // Extract issue ID from branch name
    let issue_ids = extract_issue_ids(&current_branch);
    
    // Get issue details if we found an ID
    let (pr_title, pr_body) = if !issue_ids.is_empty() {
        let api_key = get_api_key()?;
        let client = LinearClient::new(api_key);
        
        match client.get_issue_by_identifier(&issue_ids[0]).await {
            Ok(issue) => {
                let default_title = title.cloned().unwrap_or_else(|| {
                    format!("{}: {}", issue.identifier, issue.title)
                });
                
                let default_body = body.cloned().unwrap_or_else(|| {
                    format!(
                        "## Summary\n{}\n\n## Linear Issue\n{}\n\n## Changes\n- \n\n## Testing\n- ",
                        issue.description.as_deref().unwrap_or(""),
                        issue.url
                    )
                });
                
                (default_title, default_body)
            }
            Err(_) => {
                (title.cloned().unwrap_or_default(), body.cloned().unwrap_or_default())
            }
        }
    } else {
        (title.cloned().unwrap_or_default(), body.cloned().unwrap_or_default())
    };
    
    // Create PR using gh CLI
    let mut args = vec!["pr", "create"];
    
    if !pr_title.is_empty() {
        args.push("--title");
        args.push(&pr_title);
    }
    
    if !pr_body.is_empty() {
        args.push("--body");
        args.push(&pr_body);
    }
    
    if draft {
        args.push("--draft");
    }
    
    if web {
        args.push("--web");
    }
    
    let output = Command::new("gh")
        .args(&args)
        .output()?;
    
    if !output.status.success() {
        return Err(format!("Failed to create PR: {}", String::from_utf8_lossy(&output.stderr)).into());
    }
    
    println!("✅ Pull request created successfully!");
    print!("{}", String::from_utf8_lossy(&output.stdout));
    
    Ok(())
}

pub async fn handle_git_hook(_matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    // Read commit message from stdin (for commit-msg hook)
    use std::io::{self, Read};
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;
    
    // Extract issue IDs
    let issue_ids = extract_issue_ids(&buffer);
    
    if issue_ids.is_empty() {
        println!("No Linear issue IDs found in commit message");
        return Ok(());
    }
    
    println!("Found Linear issues: {}", issue_ids.join(", "));
    
    // Update issue status based on keywords
    let api_key = get_api_key()?;
    let client = LinearClient::new(api_key);
    
    for issue_id in issue_ids {
        // Check for status keywords
        let lower_message = buffer.to_lowercase();
        
        let new_state = if lower_message.contains("fixes") || 
                          lower_message.contains("closes") || 
                          lower_message.contains("resolves") {
            Some("Done")
        } else if lower_message.contains("wip") || 
                  lower_message.contains("in progress") {
            Some("In Progress")
        } else {
            None
        };
        
        if let Some(state) = new_state {
            match client.update_issue(
                &issue_id,
                None,
                None,
                Some(state),
                None,
                None,
                None,
            ).await {
                Ok(_) => println!("  ✓ Updated {} to {}", issue_id, state),
                Err(e) => eprintln!("  ✗ Failed to update {}: {}", issue_id, e),
            }
        }
    }
    
    Ok(())
}

// Helper function to extract Linear issue IDs from text
fn extract_issue_ids(text: &str) -> Vec<String> {
    let re = Regex::new(ISSUE_PATTERN).unwrap();
    re.captures_iter(text)
        .map(|cap| cap[1].to_string())
        .collect()
}

// Helper function to sanitize branch names
fn sanitize_branch_name(title: &str) -> String {
    title
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .take(5) // Limit to 5 words
        .collect::<Vec<_>>()
        .join("-")
}