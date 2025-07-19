use colored::*;
use crate::models::{Issue, Team, Project, WorkflowState};
use super::utils::*;
use super::markdown::*;

pub fn format_state_color(state: &WorkflowState) -> ColoredString {
    match state.state_type.as_str() {
        "started" => state.name.yellow(),
        "completed" => state.name.green(),
        "canceled" => state.name.red().dimmed(),
        "unstarted" => state.name.normal(),
        "backlog" => state.name.dimmed(),
        _ => state.name.normal(),
    }
}

pub fn get_state_icon(state_type: &str) -> &'static str {
    match state_type {
        "started" => "◐",
        "completed" => "✓",
        "canceled" => "✗",
        "unstarted" => "○",
        _ => "•",
    }
}

fn print_issue_line(issue: &Issue) {
    let assignee = issue
        .assignee
        .as_ref()
        .map(|a| extract_first_name(&a.name))
        .unwrap_or("Unassigned");

    // Format labels
    let labels = if !issue.labels.nodes.is_empty() {
        let label_str = issue.labels.nodes
            .iter()
            .map(|l| l.name.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        format!(" [{}]", label_str.cyan())
    } else {
        String::new()
    };

    // Format description preview
    let desc_preview = if let Some(desc) = &issue.description {
        let cleaned = clean_description(desc);
        if !cleaned.is_empty() {
            format!("\n    {}", cleaned.dimmed())
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    println!(
        "{} {} - {}{} ({}){}{}",
        format_priority_indicator(issue.priority),
        issue.identifier.blue(),
        issue.title,
        labels,
        if assignee == "Unassigned" {
            assignee.dimmed()
        } else {
            assignee.green()
        },
        desc_preview,
        if issue.priority.unwrap_or(0) >= 3 {
            format!(" {}", format_priority(issue.priority))
        } else {
            String::new()
        }
    );
}

pub fn print_issues(issues: &[Issue], format: &str, group_by: &str) {
    if issues.is_empty() {
        println!("{}", "No issues found.".dimmed());
        return;
    }

    match format {
        "json" => {
            let json = serde_json::to_string_pretty(&issues).unwrap();
            println!("{}", json);
        }
        "table" => {
            // Print header
            println!("{}", "─".repeat(120).dimmed());
            println!(
                "{:<20} {:<40} {:<12} {:<8} {:<20}",
                "ID".bold(),
                "Title".bold(),
                "State".bold(),
                "Team".bold(),
                "Assignee".bold()
            );
            println!("{}", "─".repeat(120).dimmed());

            // Print rows
            for issue in issues {
                let assignee = issue
                    .assignee
                    .as_ref()
                    .map(|a| extract_first_name(&a.name))
                    .unwrap_or("Unassigned");

                println!(
                    "{:<20} {:<40} {:<12} {:<8} {:<20}",
                    issue.identifier.blue(),
                    truncate(&issue.title, 40),
                    format_state_color(&issue.state),
                    issue.team.key.cyan(),
                    if assignee == "Unassigned" {
                        assignee.dimmed()
                    } else {
                        assignee.green()
                    }
                );
            }
            println!("{}", "─".repeat(120).dimmed());
        }
        _ => {
            // Group issues based on group_by parameter
            let mut grouped: std::collections::HashMap<String, Vec<&Issue>> = std::collections::HashMap::new();
            
            match group_by {
                "project" => {
                    for issue in issues {
                        let group_key = issue.project.as_ref()
                            .map(|p| p.name.clone())
                            .unwrap_or_else(|| "No Project".to_string());
                        grouped.entry(group_key).or_default().push(issue);
                    }
                }
                _ => { // default to "status"
                    for issue in issues {
                        grouped.entry(issue.state.name.clone()).or_default().push(issue);
                    }
                }
            }

            // Print groups based on grouping type
            match group_by {
                "project" => {
                    // Sort project names alphabetically, with "No Project" last
                    let mut project_names: Vec<String> = grouped.keys().cloned().collect();
                    project_names.sort_by(|a, b| {
                        if a == "No Project" { std::cmp::Ordering::Greater }
                        else if b == "No Project" { std::cmp::Ordering::Less }
                        else { a.cmp(b) }
                    });
                    
                    for project_name in project_names {
                        if let Some(group_issues) = grouped.get(&project_name) {
                            // Print project header
                            println!("\n📁 {} ({})", 
                                project_name.bold().cyan(),
                                group_issues.len()
                            );
                            println!("{}", "─".repeat(50).dimmed());
                            
                            // Print issues in this project
                            for issue in group_issues {
                                print_issue_line(issue);
                            }
                        }
                    }
                }
                _ => { // status grouping
                    // Define state order for status grouping
                    let state_order = vec!["In Progress", "Todo", "Backlog", "Done", "Canceled"];
                    
                    // Print groups in order
                    for state_name in &state_order {
                        if let Some(group_issues) = grouped.get(*state_name) {
                            // Print state header
                            println!("\n{} {} ({})", 
                                get_state_icon(&group_issues[0].state.state_type),
                                state_name.bold(),
                                group_issues.len()
                            );
                            println!("{}", "─".repeat(50).dimmed());

                            // Print issues in this state
                            for issue in group_issues {
                                print_issue_line(issue);
                            }
                        }
                    }

                    // Print any states not in our predefined order
                    for (state_name, group_issues) in &grouped {
                        if !state_order.contains(&state_name.as_str()) {
                            println!("\n{} {} ({})", 
                                get_state_icon(&group_issues[0].state.state_type),
                                state_name.bold(),
                                group_issues.len()
                            );
                            println!("{}", "─".repeat(50).dimmed());

                            for issue in group_issues {
                                print_issue_line(issue);
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn print_single_issue(issue: &Issue) {
    println!("\n{}", "═".repeat(80).blue());
    println!("{} {}", issue.identifier.blue().bold(), issue.title.bold());
    println!("{}", "─".repeat(80).dimmed());
    
    // Metadata row
    println!(
        "{}: {} | {}: {} | {}: {} | {}: {}",
        "State".dimmed(),
        format_state_color(&issue.state),
        "Team".dimmed(),
        issue.team.name.cyan(),
        "Priority".dimmed(),
        format_priority(issue.priority),
        "Created".dimmed(),
        format_relative_time(&issue.created_at).dimmed()
    );
    
    // Assignee
    if let Some(assignee) = &issue.assignee {
        println!("{}: {} ({})", "Assignee".dimmed(), assignee.name.green(), assignee.email.dimmed());
    } else {
        println!("{}: {}", "Assignee".dimmed(), "Unassigned".dimmed());
    }
    
    // Labels
    if !issue.labels.nodes.is_empty() {
        let labels: Vec<String> = issue.labels.nodes
            .iter()
            .map(|l| format!("{}", l.name.on_truecolor(
                u8::from_str_radix(&l.color[1..3], 16).unwrap_or(128),
                u8::from_str_radix(&l.color[3..5], 16).unwrap_or(128),
                u8::from_str_radix(&l.color[5..7], 16).unwrap_or(128)
            ).black()))
            .collect();
        println!("{}: {}", "Labels".dimmed(), labels.join(" "));
    }
    
    // URL
    println!("{}: {}", "URL".dimmed(), issue.url.blue().underline());
    
    // Description
    if let Some(desc) = &issue.description {
        if !desc.trim().is_empty() {
            println!("\n{}", "Description".bold());
            println!("{}", "─".repeat(40).dimmed());
            print_formatted_markdown(desc);
        }
    }
    
    println!("\n{}", "═".repeat(80).blue());
}

pub fn print_teams(teams: &[Team]) {
    println!("{}", "Teams:".bold());
    for team in teams {
        println!("  {} - {} ({})", team.key.cyan(), team.name, team.id.dimmed());
    }
}

pub fn print_projects(projects: &[Project]) {
    if projects.is_empty() {
        println!("{}", "No projects found.".dimmed());
        return;
    }

    println!("\n{}", "Projects".bold().blue());
    println!("{}", "═".repeat(80).blue());

    for project in projects {
        println!("\n{} {}", "▸".cyan(), project.name.bold());
        
        if let Some(desc) = &project.description {
            if !desc.trim().is_empty() {
                // Take first line of description
                let first_line = desc.lines().next().unwrap_or("");
                let preview = truncate(first_line, 70);
                println!("  {}", preview.dimmed());
            }
        }
        
        println!(
            "  {}: {} | {}: {} | {}: {}",
            "State".dimmed(),
            match project.state.as_str() {
                "planned" => project.state.yellow(),
                "started" => project.state.green(),
                "completed" => project.state.blue(),
                "canceled" => project.state.red().dimmed(),
                _ => project.state.normal(),
            },
            "Created".dimmed(),
            format_relative_time(&project.created_at).dimmed(),
            "URL".dimmed(),
            project.url.blue().underline()
        );
    }
    
    println!("\n{}", "═".repeat(80).blue());
}