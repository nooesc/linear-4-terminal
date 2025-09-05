use std::os::unix::fs::PermissionsExt;
use clap::ArgMatches;
use colored::*;
use regex::Regex;
use std::process::Command;
use crate::cli_context::CliContext;
use crate::error::{LinearError, LinearResult, ErrorContext};

// Common Linear issue ID patterns
const ISSUE_PATTERN: &str = r"([A-Z]{2,}-\d+)";

pub async fn handle_git_commit(matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    handle_git_commit_impl(matches).await.map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}

async fn handle_git_commit_impl(matches: &ArgMatches) -> LinearResult<()> {
    let message = matches.get_one::<String>("message")
        .ok_or_else(|| LinearError::InvalidInput("Commit message is required".to_string()))?;
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
        return Err(LinearError::Unknown(format!("Git commit failed: {}", String::from_utf8_lossy(&output.stderr))));
    }
    
    println!("✅ Commit created successfully!");
    println!("Message: {}", formatted_message);
    
    // Update Linear issue status if requested
    if matches.get_flag("update-status") {
        if let Some(new_state) = matches.get_one::<String>("status") {
            let mut context = CliContext::load().context("Failed to load CLI context")?;
            let client = context.verified_client().context("Failed to get Linear client")?;
            
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
            return Err(LinearError::Unknown(format!("Git push failed: {}", String::from_utf8_lossy(&push_output.stderr))));
        }
    }
    
    Ok(())
}

pub async fn handle_git_branch(matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    handle_git_branch_impl(matches).await.map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}

async fn handle_git_branch_impl(matches: &ArgMatches) -> LinearResult<()> {
    let issue_id = matches.get_one::<String>("issue")
        .ok_or_else(|| LinearError::InvalidInput("Issue ID is required".to_string()))?;
    let prefix = matches.get_one::<String>("prefix")
        .map(|s| s.as_str())
        .unwrap_or("feature");
    
    // Get issue details from Linear
    let mut context = CliContext::load().context("Failed to load CLI context")?;
    let client = context.verified_client().context("Failed to get Linear client")?;
    let issue = client.get_issue_by_identifier(issue_id).await
        .map_err(|e| LinearError::ApiError(format!("Failed to get issue: {}", e)))
        .context("Getting issue details")?;
    
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
            return Err(LinearError::Unknown(format!("Failed to create/checkout branch: {}", 
                String::from_utf8_lossy(&output.stderr))));
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
    handle_git_pr_impl(matches).await.map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}

async fn handle_git_pr_impl(matches: &ArgMatches) -> LinearResult<()> {
    let title = matches.get_one::<String>("title");
    let body = matches.get_one::<String>("body");
    let draft = matches.get_flag("draft");
    let web = matches.get_flag("web");
    
    // Get current branch
    let branch_output = Command::new("git")
        .args(&["branch", "--show-current"])
        .output()?;
    
    if !branch_output.status.success() {
        return Err(LinearError::Unknown("Failed to get current branch".to_string()));
    }
    
    let current_branch = String::from_utf8_lossy(&branch_output.stdout).trim().to_string();
    
    // Extract issue ID from branch name
    let issue_ids = extract_issue_ids(&current_branch);
    
    // Get issue details if we found an ID
    let (pr_title, pr_body) = if !issue_ids.is_empty() {
        let mut context = CliContext::load().context("Failed to load CLI context")?;
        let client = context.verified_client().context("Failed to get Linear client")?;
        
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
        return Err(LinearError::Unknown(format!("Failed to create PR: {}", String::from_utf8_lossy(&output.stderr))));
    }
    
    println!("✅ Pull request created successfully!");
    print!("{}", String::from_utf8_lossy(&output.stdout));
    
    Ok(())
}

pub async fn handle_git_hook(_matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    handle_git_hook_impl(_matches).await.map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}

async fn handle_git_hook_impl(_matches: &ArgMatches) -> LinearResult<()> {
    // Read commit message from stdin (for commit-msg hook)
    use std::io::{self, Read};
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)
        .context("Failed to read commit message from stdin")?;
    
    // Extract issue IDs
    let issue_ids = extract_issue_ids(&buffer);
    
    if issue_ids.is_empty() {
        println!("No Linear issue IDs found in commit message");
        return Ok(());
    }
    
    println!("Found Linear issues: {}", issue_ids.join(", "));
    
    // Update issue status based on keywords
    let mut context = CliContext::load().context("Failed to load CLI context")?;
    let client = context.verified_client().context("Failed to get Linear client")?;
    
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

pub async fn handle_install_hook(_matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    handle_install_hook_impl(_matches).await.map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}

async fn handle_install_hook_impl(_matches: &ArgMatches) -> LinearResult<()> {
    let git_dir = Command::new("git")
        .args(&["rev-parse", "--git-dir"])
        .output()
        .context("Failed to execute git command")?;

    if !git_dir.status.success() {
        return Err(LinearError::InvalidInput("Not a git repository".to_string()));
    }

    let git_dir_path = String::from_utf8_lossy(&git_dir.stdout).trim().to_string();
    let hooks_path = std::path::Path::new(&git_dir_path).join("hooks");
    let commit_msg_hook_path = hooks_path.join("commit-msg");

    // Create hooks directory if it doesn't exist
    if !hooks_path.exists() {
        std::fs::create_dir_all(&hooks_path)
            .context("Failed to create hooks directory")?;
    }

    let hook_script = r##"#!/bin/sh
#
# Automatically update Linear issue status from commit message.
# Installed by `linear git install-hook`

# Read commit message from the file passed as the first argument
COMMIT_MSG_FILE=$1
COMMIT_MSG=$(cat "$COMMIT_MSG_FILE")

# Check if the linear CLI is in the PATH
if ! command -v linear >/dev/null 2>&1; then
    echo "linear-cli not found in PATH"
    exit 1
fi

# Call the linear git hook command with the commit message
echo "$COMMIT_MSG" | linear git hook
"##;

    std::fs::write(&commit_msg_hook_path, hook_script)
        .context("Failed to write hook script")?;
    std::fs::set_permissions(&commit_msg_hook_path, std::fs::Permissions::from_mode(0o755))
        .context("Failed to set hook permissions")?;

    println!("✅ commit-msg hook installed successfully at: {:?}", commit_msg_hook_path);
    println!("You can now automatically update Linear issues by mentioning them in your commit messages.");
    println!("Example: `git commit -m \"Fixes ENG-123: Implement the new feature\"`");

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
